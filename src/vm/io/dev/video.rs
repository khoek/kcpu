use super::super::types::*;
use crate::{
    spec::types::hw::{self, Word},
    vm::interface,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::cell::Ref;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const PORT_BASE: isize = 0xC0;

#[derive(Debug, FromPrimitive, EnumIter)]
enum Reg {
    Cmd = PORT_BASE,
    AddrHi,
    AddrLo,
    Data,
    Stream,
}

#[derive(Debug, FromPrimitive, EnumIter)]
enum Cmd {
    StreamReset,
}

pub struct Video {
    addr_hi: Word,
    addr_lo: Word,

    vram: Vec<Word>,
}

impl Video {
    pub fn new() -> Self {
        Self {
            addr_hi: 0,
            addr_lo: 0,

            vram: vec![0; interface::VIDEO_WIDTH * 2 * interface::VIDEO_HEIGHT],
        }
    }

    fn addr(&self) -> usize {
        (self.addr_hi as usize) << hw::WORD_WIDTH | (self.addr_lo as usize)
    }

    fn inc_addr(&mut self) {
        if self.addr_lo == hw::WORD_MAX {
            assert!(self.addr_hi < hw::WORD_MAX);

            self.addr_lo = 0;
            self.addr_hi += 1;
        } else {
            self.addr_lo += 1;
        }
    }

    fn handle_command(&mut self, cmd: Word) {
        match Cmd::from_u16(cmd).expect("command out of range") {
            Cmd::StreamReset => {
                // TODO This should a) flip the video buffer, and b) reset the address registers
                // (which the stream mode uses and increments as it loads data).
                unimplemented!("Streaming is not yet implemented")
            }
        }
    }
}

impl Device for Video {
    fn reserved_ports(&self) -> Vec<Word> {
        Reg::iter().map(|p| p as u16).collect()
    }

    fn write(&mut self, port: Word, val: Word) -> HalfcycleCount {
        match Reg::from_u16(port).expect("port out of range") {
            Reg::Cmd => self.handle_command(val),
            Reg::AddrHi => self.addr_hi = val,
            Reg::AddrLo => self.addr_lo = val,
            Reg::Data => {
                let addr = self.addr();
                self.vram[addr] = val
            }
            Reg::Stream => {
                let addr = self.addr();
                self.vram[addr] = val;
                self.inc_addr();
            }
        }

        0
    }

    fn read(&mut self, port: Word) -> (HalfcycleCount, Word) {
        match Reg::from_u16(port).expect("port out of range") {
            Reg::Data => (0, self.vram[self.addr()]),
            _ => panic!("cannot read from that graphics register"),
        }
    }

    fn process_halfcycle(&mut self, _: ClockedSignals) {}
}

impl interface::Video for Handle<Video> {
    fn vram(&self) -> Ref<[Word]> {
        Ref::map::<[Word], _>(self.rc.borrow(), |video| &video.vram)
    }
}
