use crate::spec::types::hw::*;
use enum_map::EnumMap;

#[derive(Clone, Copy)]
pub struct Logger {
    pub disassemble: bool,
    pub dump_registers: bool,
    pub dump_bus: bool,
}

impl Logger {
    pub const fn silent() -> Self {
        Self {
            disassemble: false,
            dump_registers: false,
            dump_bus: false,
        }
    }

    pub const fn only_machine_state() -> Self {
        Self {
            disassemble: false,
            dump_registers: true,
            dump_bus: true,
        }
    }

    pub const fn everything() -> Self {
        Self {
            disassemble: true,
            dump_registers: true,
            dump_bus: true,
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Logger::everything()
    }
}

pub struct BusState<'a> {
    logger: &'a Logger,
    frozen: bool,

    bus: EnumMap<Bus, Option<Word>>,
}

impl<'a> BusState<'a> {
    pub fn new(logger: &'a Logger) -> Self {
        Self {
            logger,
            frozen: false,
            bus: EnumMap::new(),
        }
    }

    pub fn assign(&mut self, b: Bus, val: Word) {
        if self.logger.dump_bus {
            println!("  {} <- {}", b, val);
        }

        if self.frozen {
            panic!("bus state frozen!");
        }

        if self.bus[b].is_some() {
            panic!("out bus collision");
        }

        self.bus[b] = Option::Some(val);
    }

    pub fn connect(&mut self, b1: Bus, b2: Bus) {
        match (self.bus[b1], self.bus[b2]) {
            (Some(_), Some(_)) => panic!("connect collision!"),
            (None, None) => {
                panic!("currently unimplemented, not needed (but one could have a pull-down)")
            }
            (Some(_), None) => self.assign(b2, self.early_read(b1)),
            (None, Some(_)) => self.assign(b1, self.early_read(b2)),
        }
    }

    pub fn freeze(&mut self) {
        if self.frozen {
            panic!("bus state already frozen!");
        }

        self.frozen = true;
    }

    pub fn early_read(&self, b: Bus) -> Word {
        let ret = self.bus[b].unwrap_or_else(|| b.get_pulled_value());
        if self.logger.dump_bus {
            println!("  {} -> {}", b, ret);
        }
        ret
    }

    pub fn read(&self, b: Bus) -> Word {
        if !self.frozen {
            panic!("bus state not yet frozen!");
        }

        self.early_read(b)
    }
}
