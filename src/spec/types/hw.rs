use derive_more::Display;
use enum_map::Enum;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use static_assertions::const_assert;
use std::{
    convert::{TryFrom, TryInto},
    num::{TryFromIntError, Wrapping},
};
use strum_macros::EnumIter;

pub type Byte = u8;
pub type Word = u16;

pub const BYTE_WIDTH: usize = 8;
pub const WORD_WIDTH: usize = 16;
pub const WORD_MAX: Word = 0xFFFF;

pub const fn byte_flip(v: Word) -> Word {
    ((v & 0x00FF) << BYTE_WIDTH) | ((v & 0xFF00) >> BYTE_WIDTH)
}

pub fn bytes_to_words_into_buff(buff: &mut [Word], bytes: &[Byte]) -> Option<()> {
    for (idx, ch) in bytes.chunks(2).enumerate() {
        buff[idx] = std::primitive::u16::from_le_bytes(ch.try_into().ok()?)
    }
    Some(())
}

// RUSTFIX make this const when possible
// Returns none if the data has bad parity.
pub fn bytes_to_words(bytes: &[Byte]) -> Option<Vec<Word>> {
    let mut buff = vec![0; bytes.len() / 2];
    bytes_to_words_into_buff(&mut buff, bytes).map(|_| buff)
}

pub fn words_to_bytes(v: Vec<Word>) -> Vec<Byte> {
    v.into_iter()
        .map(|w| {
            // RUSTFIX, we call `to_be_bytes` here, which actually needs to be
            // synchronized with how the VM parses `Vec<u8>`s, so move this out
            // of the frontend and into a pair of functions this package provides.
            // We already have `lib` for internal stuff, so like `util` or something.
            //
            // RUSTFIX Also make the VM use that when decoding!

            // RUSTFIX this is bad, but I think it has to be this way
            // due to the lack of const generics?
            let bs = std::primitive::u16::to_le_bytes(w);
            std::iter::once(bs[0]).chain(std::iter::once(bs[1]))
        })
        .flatten()
        .collect()
}

pub fn word_from_i64_wrapping(i: i64) -> Result<u16, TryFromIntError> {
    if i >= 0 {
        Word::try_from(i)
    } else {
        Ok((Wrapping(0) - Wrapping(Word::try_from(-i)?)).0)
    }
}

// RUSTFIX use a struct which checks the number of bits based on a fixed constant.
pub type UInst = u64;

// RUSTFIX use a struct which checks the number of bits based on a fixed constant. (2)
// RUSTFIX we aren't actually using this now, because we store the UC in a Word---change this?
pub type UCVal = u8;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, FromPrimitive, Enum, EnumIter)]
pub enum PReg {
    ID,
    SP,
    BP,
    A,
    B,
    C,
    D,
    E,
}

impl Default for PReg {
    fn default() -> PReg {
        PReg::ID
    }
}

// RUSTFIX put these in good places
pub const UADDR_WIDTH: u32 = 13;
pub const UCODE_LEN: usize = 1 << UADDR_WIDTH;

pub const UCVAL_WIDTH: u32 = 2;
pub const INST_WIDTH: u32 = 9;

#[allow(unused)]
pub const CHIP_SELECT_WIDTH: u32 = UADDR_WIDTH - (UCVAL_WIDTH + INST_WIDTH);
const_assert!(CHIP_SELECT_WIDTH == 2);

pub const UCVAL_MAX: usize = (1 << UCVAL_WIDTH) - 1;

#[derive(Debug, Clone, Copy, Enum, EnumIter)]
pub enum IU {
    ONE,
    TWO,
    THREE,
}

impl IU {
    pub const fn encode_all(iu1: PReg, iu2: PReg, iu3: PReg) -> Word {
        IU::ONE.encode(iu1) | IU::TWO.encode(iu2) | IU::THREE.encode(iu3)
    }

    pub fn decode_all(inst: Word) -> (PReg, PReg, PReg) {
        (
            IU::ONE.decode(inst),
            IU::TWO.decode(inst),
            IU::THREE.decode(inst),
        )
    }

    pub const WIDTH: u32 = 3;
    const MASK: Word = 0b111;

    pub const fn encode(self, reg: PReg) -> Word {
        let idx = self as u32;
        (reg as u16) << (idx * IU::WIDTH)
    }

    pub fn decode(self, inst: Word) -> PReg {
        let idx = self as u32;
        PReg::from_u16((inst & (IU::MASK << (idx * IU::WIDTH))) >> (idx * IU::WIDTH)).unwrap()
    }
}

/*
    Notes on instruction encoding/kinds:

    When defining new instructions, intepreting instructions, and looking up the UInsts which make up
    an instruction, there are three different concepts we need to keep in mind:

        1.  `OpCodes`: This is is the (currently) 9-bit code which encodes which instruction is actually
            executed, so is "just a number". We like to call the high 5 bits the `IType` (actually, the
            highest bit is currently unused), and the low 4 bits the `IKind`, and this might get actual
            type-safe recognition later.

            The only exception is that sometimes the lowest 3 bits are used to specify the value of IU3,
            when it is needed, but this is transparent to the hardware and we just duplicate the ucode
            in the ucode ROMs for each possible value of IU3 in this case.

        2.  `Inst`s: In hardware these are `Word`-size numbers, divided as follows (each character is
            a bit, and "_" is unused):

                L (_TTTTCCCC) III III
                   OOOOOOOOO

            From right-to-left we have (III) IU1, then (III) IU2, then the low 4 bits of the opcode
            (CCCC) called the `IKind`, the next 4 bits of the opcode (TTTT) called the `IType`, and
            finally an unused bit (_) and then the LOAD_DATA bit (L). This last bit instructs the machine
            to read an additional word after the instruction word and load this word into the instruction
            data register.

        3.  `UAddr`s: Not all of this information, like the LOAD_DATA flag (L) or the IUs (III, III), need
            need to be passed to the ucode ROMs, so they aren't. Also, the ucode ROMs need more data, and
            only have 13 address lines. They are laid out as:

                OOOOOOOOO UU BB

            As before, (OOOOOOOOO) is the 9-bit opcode. The 2-bit field (UU) is the value of the ucode
            counter. The final 2-bit field (BB) is "bank select", enabling 4x 8-bit EEPROMS to store
            the 32-bit `UInst` field. These bits are tied on the board.

    The first two of these structures (`OpCode` and `Inst`) are modelled directly below this comment. We
    don't model `UAddr`s directly, since we don't simulate the separate banks. Instead we use `PUAddr`s
    (psuedo-`UAddr`s), which omit the low (BB) bits.

    Finally, why isn't an `OpClass` here? `OpClass`es are used to state whether the last three bits of
    an opcode are needed for a used IU3 value or not; if they are, we don't then actually have to dulplicate
    out instruction definitions (`InstDef`s). Both `OpClass`es and `InstDef`s are defined along with the
    ucode builder, so don't need to appear here, as an actual fixed representation of the hardware
    instruction decoding system.
*/

// RUSTFIX use fixed (currently 9) number of bits
pub type OpCode = Word;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inst {
    pub load_data: bool,
    pub opcode: OpCode,
    // RUSTFIX convert to `EnumMap` once the neccesary thing has stabilised (very soon?)
    //         (the blocker is not being able to have `const` on `encode()` and `new()` otherwise)
    pub iu1: Option<PReg>,
    pub iu2: Option<PReg>,
    pub iu3: Option<PReg>,
}

impl Inst {
    pub const SHIFT: u32 = 2 * IU::WIDTH;

    // RUSTFIX this was originally in inst.cpp not hw.cpp, but it isn't configurable!
    pub const P_LOAD_DATA: Word = 1 << 15;

    // RUSTFIX use these in the newfangled instruction decoder module?
    // RUSTFIX also, this would make the disassembler nicer/might implement some of its functionality here
    pub const fn strip_iu3(inst: Word) -> Word {
        inst & !IU::MASK
    }

    pub const fn decode_load_data(inst: Word) -> bool {
        inst & Inst::P_LOAD_DATA != 0
    }

    pub const fn decode_opcode(inst: Word) -> Word {
        (inst & !Inst::P_LOAD_DATA) >> Inst::SHIFT
    }

    pub const fn new(
        load_data: bool,
        opcode: Word,
        iu1: Option<PReg>,
        iu2: Option<PReg>,
        iu3: Option<PReg>,
    ) -> Inst {
        Inst {
            load_data,
            opcode,
            iu1,
            iu2,
            iu3,
        }
    }

    // RUSTFIX remove this once we can make `new` const while using
    // `EnumMap` (or roll a better one ourself).
    pub fn iu(&self, iu: IU) -> Option<PReg> {
        match iu {
            IU::ONE => self.iu1,
            IU::TWO => self.iu2,
            IU::THREE => self.iu3,
        }
    }

    // RUSTFIX make this const once possible
    pub fn encode(&self) -> Word {
        ((self.load_data as Word) * Inst::P_LOAD_DATA)
            | (self.opcode << Inst::SHIFT)
            | IU::encode_all(
                self.iu1.unwrap_or_default(),
                self.iu2.unwrap_or_default(),
                self.iu3.unwrap_or_default(),
            )
    }
}

// RUSTFIX use fixed (currently 13 - 2 (bank sel) = 11) number of bits
pub struct PUAddr {
    val: Word,
}

impl PUAddr {
    pub fn new(opcode: OpCode, uc: UCVal) -> Self {
        // RUSTFIX use fixed (currently 13 - 2 (bank sel) = 11) number of bits
        Self {
            val: (opcode << UCVAL_WIDTH) | (uc as Word),
        }
    }
}

impl From<PUAddr> for usize {
    fn from(pua: PUAddr) -> Self {
        pua.val as usize
    }
}

#[derive(Debug, Clone, Copy, Enum, Display)]
pub enum Bus {
    A,
    B,

    F,
    M,
}

impl Bus {
    pub fn pulled_value(self) -> Word {
        match self {
            Bus::A => 0,
            Bus::B => 0,

            _ => panic!("Bus {:?} is floating!", self),
        }
    }
}
