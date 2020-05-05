use super::super::types::*;
use super::types::*;
use crate::spec::types::hw::*;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub enum Command {
    Read { port: Word },
    Write { port: Word, value: Word },
}

#[derive(Clone, Copy)]
enum Operation {
    Read { result: Word },
    Write,
}

impl Operation {
    fn agrees_with(self, cmd: Command) -> bool {
        match (self, cmd) {
            (Operation::Read { result: _ }, Command::Read { port: _ }) => true,
            (Operation::Write, Command::Write { port: _, value: _ }) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy)]
enum Status {
    Ongoing(
        /* Number of cycles remaining for this operation. */ HalfcycleCount,
    ),
    Presenting,
}

#[derive(Clone, Copy)]
enum State {
    Idle,
    // State which means that we have presented during a clock rising
    // edge, but that IO_DONE should not go low until a clock falling edge.
    Returning,
    Active(Status, Operation),
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Read { result } => write!(f, "Read[{:#04X}]", result),
            Operation::Write => write!(f, "Write"),
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Ongoing(left) => write!(f, "Ongoing[{}]", left),
            Status::Presenting => write!(f, "Presenting"),
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Idle => write!(f, "Idle"),
            State::Returning => write!(f, "Returning"),
            State::Active(status, op) => write!(f, "Active ({}, {})", op, status),
        }
    }
}

pub struct Manager<'a> {
    logger: &'a Logger,

    devices: Vec<Handle<dyn Device + 'a>>,
    ports: HashMap<Word, Handle<dyn Device + 'a>>,

    state: State,
}

impl<'a> Manager<'a> {
    pub fn new(logger: &Logger) -> Manager {
        Manager {
            logger,
            devices: Vec::new(),
            ports: HashMap::new(),
            state: State::Idle,
        }
    }

    pub fn dump_registers(&self) {
        println!("IO: {}", self.state);
    }

    pub fn add_device<T: Device + 'a>(&mut self, d: T) -> Handle<T> {
        let h = Handle::new(d);

        for port in &h.get_reserved_ports() {
            assert!(self.ports.insert(*port, h.forget()).is_none());
        }

        self.devices.push(h.forget());

        h
    }

    pub fn get_registered_ports(&self) -> Vec<Word> {
        self.ports.keys().copied().collect()
    }

    pub fn is_io_done(&self) -> bool {
        match self.state {
            State::Returning | State::Active(Status::Presenting, _) => true,
            _ => false,
        }
    }

    pub fn get_read_result(&self) -> Option<Word> {
        match self.state {
            // NOTE: I think this might be too constrained, erroring when we are trying to read but waiting. Of course we could return a dummy value, but there are still some checks we can do
            // e.g. whether we are actually reading and not writing or something.
            // Maybe I can return an Option::None while we are waiting, which results in the bus not being commited to in the ioc?
            // State::Active(Status::Ongoing(_), Operation::Read { result }) => None,
            State::Active(Status::Presenting, Operation::Read { result }) => Some(result),
            _ => panic!(
                "interrogating IO manager for result when no result presented! (state: {})",
                self.state
            ),
        }
    }

    fn get_device(&self, port: Word) -> &Handle<dyn Device + 'a> {
        match self.ports.get(&port) {
            None => panic!("command to floating port: {:#0X}", port),
            Some(dev) => dev,
        }
    }

    pub fn before_clock_outputs(&mut self, cmd: Option<Command>) {
        match (self.state, cmd) {
            (_, None) => (),
            (State::Active(_, op), Some(cmd)) => {
                assert!(op.agrees_with(cmd));
            }
            (State::Idle, Some(cmd)) => {
                match cmd {
                    Command::Read { port } => {
                        let (cycles, result) = self.get_device(port).read(port);
                        self.state =
                            State::Active(Status::Ongoing(cycles), Operation::Read { result });
                    }
                    Command::Write { port, value } => {
                        let cycles = self.get_device(port).write(port, value);
                        self.state = State::Active(Status::Ongoing(cycles), Operation::Write);
                    }
                }

                if self.logger.dump_bus {
                    println!("io {} starting", self.state);
                }
            }
            _ => panic!("Unacceptable `before_clock_outputs` state"),
        }
    }

    pub fn after_clock_outputs(&mut self, cmd: Option<Command>) {
        match (self.state, cmd) {
            (State::Idle, _) => (),
            (State::Active(Status::Ongoing(_), op), Some(cmd)) => {
                assert!(op.agrees_with(cmd));
            }
            (State::Active(Status::Presenting, op), Some(cmd)) => {
                assert!(op.agrees_with(cmd));

                self.state = State::Returning;
                if self.logger.dump_bus {
                    println!("io {} ending presentation", op);
                }
            }
            _ => panic!("Unacceptable `after_clock_outputs` state"),
        }
    }

    pub fn process_halfcycle(&mut self, sigs: ClockedSignals) {
        for dev in &self.devices {
            dev.process_halfcycle(sigs);
        }

        match self.state {
            State::Idle => (),
            State::Returning => {
                if let ClockedSignals::OffClock(_, _) = sigs {
                    self.state = State::Idle;
                    if self.logger.dump_bus {
                        println!("io resetting io_done");
                    }
                }
            }
            State::Active(Status::Presenting, _) => {}
            State::Active(Status::Ongoing(cycles), op) => {
                if cycles > 0 {
                    self.state = State::Active(Status::Ongoing(cycles - 1), op);
                    if self.logger.dump_bus {
                        println!("io {} ongoing, {} hcycles remaining", op, cycles - 1);
                    }
                }

                if cycles == 0 {
                    self.state = State::Active(Status::Presenting, op);

                    if self.logger.dump_bus {
                        println!("io {} now presenting", op);
                    }
                }
            }
        }
    }
}
