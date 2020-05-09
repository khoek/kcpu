use bitflags::bitflags;
use std::{fmt, num::Wrapping};

use super::types::*;
use crate::{spec::defs::usig, spec::types::hw::*};
use fmt::Display;

bitflags! {
    #[derive(Default)]
    struct Flags: Word {
        const CARRY      = 1 << 0;
        const N_ZERO     = 1 << 1;
        const SIGN       = 1 << 2;
        const N_OVERFLOW = 1 << 3;
    }
}

#[rustfmt::skip]
impl Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if self.contains(Flags::CARRY) { 'C' } else { 'c' })?;
        write!(f, "{}", if self.contains(Flags::N_ZERO) { 'z' } else { 'Z' })?;
        write!(f, "{}", if self.contains(Flags::SIGN) { 'S' } else { 's' })?;
        write!(f, "{}", if self.contains(Flags::N_OVERFLOW) { 'o' } else { 'O' })?;
        Ok(())
    }
}

impl From<Flags> for Word {
    fn from(f: Flags) -> Word {
        f.bits()
    }
}

struct OpResult {
    val: u16,
    flags: Flags,
}

impl Display for OpResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OpResult({:#06X}:{:#06X})", self.val, self.flags)
    }
}

impl Default for OpResult {
    fn default() -> Self {
        Self {
            val: 0,
            flags: Default::default(),
        }
    }
}

enum OpFunc {
    Arithmetic(fn(i16, i16) -> (i16, u32)),
    Logic(fn(u16, u16) -> u16),
    Shift(fn(u16, u16) -> (u16, bool)),
}

fn is_flag_n_zero(val: u16) -> bool {
    val != 0
}

fn is_flag_sign(val: u16) -> bool {
    val & 0x8000 != 0
}

// HARDWARE NOTE FORMERLY BUG IN ALL SIMULATORS (|| not &&), LIKELY BUG IN
// CURRENT HARDWARE DESIGN.
#[allow(clippy::nonminimal_bool)]
fn is_flag_n_overflow(val: i16, a: i16, b: i16) -> bool {
    !(a >= 0 && b >= 0 && val < 0) && !(a < 0 && b < 0 && val >= 0)
}

fn encode_flags(carry: bool, n_zero: bool, sign: bool, n_overflow: bool) -> Flags {
    let mut f: Flags = Default::default();
    f.set(Flags::CARRY, carry);
    f.set(Flags::N_ZERO, n_zero);
    f.set(Flags::SIGN, sign);
    f.set(Flags::N_OVERFLOW, n_overflow);
    f
}

impl OpFunc {
    // RUSTFIX proper docs?
    /// `eval` takes `Word`s and outputs a `Word` (plus `Flags`), and thus provides the bridge between the
    /// VM and the arithmetic implementation in Rust.
    fn eval(&self, a: Word, b: Word) -> OpResult {
        let (val, carry) = match self {
            Self::Arithmetic(f) => {
                let (val, carry_val) = f(a as i16, b as i16);
                (val as u16, carry_val > u16::MAX as u32)
            }
            Self::Logic(f) => {
                let val = f(a as u16, b as u16);
                (val, val & 0x0001 != 0)
            }
            Self::Shift(f) => {
                let (val, dropped) = f(a as u16, b as u16);
                (val, dropped)
            }
        };

        let n_zero = is_flag_n_zero(val);
        let sign = is_flag_sign(val);
        let n_overflow = is_flag_n_overflow(val as i16, a as i16, b as i16);

        OpResult {
            val,
            flags: encode_flags(carry, n_zero, sign, n_overflow),
        }
    }
}

struct Op<'a> {
    ui_mode: UInst,
    name: &'a str,
    func: OpFunc,
}

impl<'a> Display for Op<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Op({})", self.name)
    }
}

impl<'a> Op<'a> {
    pub const fn new(ui_mode: UInst, name: &'a str, func: OpFunc) -> Self {
        Op {
            ui_mode,
            name,
            func,
        }
    }
}

// RUSTFIX it was very easy to make an error here, write good tests for these and compare the Rust and C++ versions.

#[rustfmt::skip]
static OPS: [Op; 8] = [
    // RUSTFIX FIXME FIXME FIXME CHECK CHECK CHEK
    // HARDWARE NOTE IN THE RUST IMPLEMENTATION WE added "(a as u16)" (used to be just "a") and similarly for "b" in the calculatiuon of the carry val below, due to a
    // sign extension-caused overflow. CHECK THAT THE ARDUINO CODE VERIFIES THE HARDWARE OBEYS THIS, AND FIX THE C++ VERISON AS WELL.
    Op::new(usig::ACTRL_MODE_ADD , "+" , OpFunc::Arithmetic(|a, b| ((Wrapping(a) + Wrapping(b)).0, ((a as u16) as u32) + ((b as u16) as u32)))),
    // RUSTFIX FIXME FIXME FIXME CHECK CHECK CHEK
    // HARDWARE NOTE IN THE RUST IMPLEMENTATION WE added "(b as u16)" (used to be just "b") in the calculatiuon of the carry val below, due to a
    // sign extension overflow. CHECK THAT THE ARDUINO CODE VERIFIES THE HARDWARE OBEYS THIS, AND FIX THE C++ VERISON AS WELL.
    Op::new(usig::ACTRL_MODE_SUB , "-" , OpFunc::Arithmetic(|a, b| ((Wrapping(b) - Wrapping(a)).0, (((!a as u16) as u32) + 1) + ((b as u16) as u32)))),
    Op::new(usig::ACTRL_MODE_AND , "&" , OpFunc::Logic(|a, b| (a & b))),
    Op::new(usig::ACTRL_MODE_OR  , "|" , OpFunc::Logic(|a, b| (a | b))),
    Op::new(usig::ACTRL_MODE_XOR , "^" , OpFunc::Logic(|a, b| (a ^ b))),
    Op::new(usig::ACTRL_MODE_LSFT, "<<", OpFunc::Shift(|a, _| (a << 1, a & 0x8000 != 0))),
    Op::new(usig::ACTRL_MODE_RSFT, ">>", OpFunc::Shift(|a, _| (a >> 1, a & 0x0001 != 0))),
    // Not implemented in hardware:
    // Op::new(usig::ACTRL_MODE_ARSFT, "A>>", OpFunc::Shift(|a, _| (((a as i16) >> 1) as u16, a & 0x0001 != 0))),
    Op::new(usig::ACTRL_MODE_TST, "TST", OpFunc::Logic(|a, _| a)),
];

pub struct Alu<'a> {
    log_level: &'a LogLevel,
    result: OpResult,
}

impl<'a> Display for Alu<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ADATA: {:#06X} AFLAGS: {}({:#04X})",
            self.result.val, self.result.flags, self.result.flags
        )
    }
}

impl<'a> Alu<'a> {
    pub fn new(log_level: &'a LogLevel) -> Self {
        Self {
            log_level,
            result: Default::default(),
        }
    }

    pub fn clock_outputs(&self, ui: UInst, s: &mut BusState) {
        if ui & usig::ACTRL_DATA_OUT != 0 {
            assert!(ui & usig::ACTRL_INPUT_EN == 0);
            s.assign(Bus::A, self.result.val);
        }

        if ui & usig::ACTRL_FLAGS_OUT != 0 {
            assert!(ui & usig::ACTRL_INPUT_EN == 0);
            s.assign(Bus::B, Word::from(self.result.flags));
        }
    }

    pub fn clock_inputs(&mut self, ui: UInst, s: &BusState) {
        if ui & usig::ACTRL_INPUT_EN != 0 {
            let mode = usig::decode_actrl_mode(ui);
            let op = &OPS[mode as usize];
            assert!(mode == usig::decode_actrl_mode(op.ui_mode));

            let (bus_a, bus_b) = (s.read(Bus::A), s.read(Bus::B));
            self.result = op.func.eval(bus_a, bus_b);

            if self.log_level.internals {
                println!("{}({:#06X}, {:#06X}) -> {}", op, bus_a, bus_b, self.result);
            }
        }
    }
}
