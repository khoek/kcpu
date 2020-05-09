use crate::spec::types::hw::*;
use enum_map::EnumMap;

#[derive(Debug, Clone, Copy)]
pub struct LogLevel {
    pub internals: bool,
}

pub struct BusState<'a> {
    log_level: &'a LogLevel,
    frozen: bool,

    bus: EnumMap<Bus, Option<Word>>,
}

impl<'a> BusState<'a> {
    pub fn new(log_level: &'a LogLevel) -> Self {
        Self {
            log_level,
            frozen: false,
            bus: EnumMap::new(),
        }
    }

    pub fn assign(&mut self, b: Bus, val: Word) {
        if self.log_level.internals {
            println!("  {} <- {:#06X}", b, val);
        }

        if self.frozen {
            panic!("bus state frozen!");
        }

        if self.bus[b].is_some() {
            panic!("out bus collision");
        }

        self.bus[b] = Some(val);
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
        let ret = self.bus[b].unwrap_or_else(|| b.pulled_value());
        if self.log_level.internals {
            println!("  {} -> {:#06X}", b, ret);
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
