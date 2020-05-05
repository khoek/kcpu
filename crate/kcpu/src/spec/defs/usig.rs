use super::super::types::hw::*;
use static_assertions::const_assert;



// RUSTFIX the future of this file:
// 
// 
// BEFORE DOING THIS, PORT AND TEST THE VM, SINCE MISTAKES HERE AND THEN THE FOLLOW ON CHANGES TO CODE GENERATION WILL MAKE THIS HELL
// 
// DO CTL, MEM, REG, then IO LAST
// 
// 
// 
// 
// The overriding pricinple is to keep the encoding really simple, to not interfere with the hardware interpretation everything has.
// I think we can do this:
// Define some enums in order to implement mutual exclusion and defined "fields" in the ucode. Have each take a single parameter,
// the bits they represent, unshifted. Of course, there parameters will be constants, which is invalid rust code, but....
// 
// Then, definite a macro (a first foray!) which does a few things:
// 1) have the macro take a "shift", and then when the enum-ish structure is passed:
// 2) define a member function .bits() which is just a boilerplate match statement which spits out the bits we wrote
//    in the enum-ish syntax
//
// The enums would have titles like: Action, Command, Gctrl_Ft, Gctrl_Jm, Gctrl_Mode(including both NRM and ALT), etc.
//
// Finally, define a simple way to express relations of mutual exclusion or relations of requirement (e.g. use of a Gctrl_Mode::Alt_xxx requires Action::Gctrl_Use_Alt)---maybe that's it? grep for some vm_asserts.
// NOTE THAT specifically for Gctrl_Mode::Alt_xxx and Gctrl_Mode::Nrm_xxx there will be DOUBLE UP CODES. So, keep NRM and ALT separate structures, and make the function which eats all of this data
// to make the actual ucode just take an enum which wraps either an Alt or a Nrm (or a None).
//
// This is a good, type safe way to improve but stay close to the underlying representation. This could all be converted directly to bits,
// so it just remains to think about how we go the other way. Well, we just have this macro generate a `.from_bits()` field, which panics
// if it tries to parse some bits which do not correspond to an actual value (for most enums the cases are exhaustive so this will not be possible).
// Of course, `from_bits()` will account for the shift.
//
// 
// 
// NOTE instead of passing a shift (maybe we do that initially and then change), just pass the name of the previous one of these
// structs, and the width (also have the macro check that the enum fields fit within the specified width). This way they can daisy chain and we don't have to keep track and prevent overlaps.
// 
// 
// 
// Since everything will still be an enum, we can define case specific additional instance methods to e.g. select the correct IU1/2/3.
//


const fn mk_val(base: u32, off: u32, val: u16) -> UInst {
    (val as UInst) << (base + off)
}

const fn dec_val(base: u32, off: u32, mask: UInst, raw: UInst) -> u16 {
    ((raw & mask) >> (base + off)) as u16
}



// BEGIN DECLS

// CTRL

pub const CTRL_BASE : u32 = 0;
pub const CTRL_END  : u32 = CTRL_BASE + 4;

// Random mutually exclusive "ACTION"s
pub const ACTION_CTRL_NONE        : UInst = mk_val(CTRL_BASE, 0, 0b00);
pub const ACTION_GCTRL_USE_ALT    : UInst = mk_val(CTRL_BASE, 0, 0b01);
pub const ACTION_GCTRL_RIP_BUSA_O : UInst = mk_val(CTRL_BASE, 0, 0b10);
pub const ACTION_MCTRL_BUSMODE_X  : UInst = mk_val(CTRL_BASE, 0, 0b11);

pub const _COMMAND_NONE        : UInst = mk_val(CTRL_BASE, 2, 0b00);
/*
    HARDWARE NOTE: `COMMAND_INHIBIT_JMFT` disallows the instmask-setting
    and UC-resetting behaviour of all JMs/FTs, just for that uop. This
    is currently used to implement `_DO_INT`.
*/
pub const COMMAND_INHIBIT_JMFT: UInst = mk_val(CTRL_BASE, 2, 0b01);
/*
    There next two increment/decrement RSP ON THE CLOCK RISING EDGE.
    (RSP is usually decremented on the offclock cycle by an instruction register bit.);

    HARDWARE NOTE: COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP implicitly activates the
    IU3_OVERRIDE_O_SELECT_RSP behaviour.
*/
pub const COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP: UInst = mk_val(CTRL_BASE, 2, 0b10);
pub const COMMAND_RCTRL_RSP_EARLY_INC: UInst = mk_val(CTRL_BASE, 2, 0b11);

// NONBIT: CTRL decoding
pub const MASK_CTRL_ACTION: UInst = mk_val(CTRL_BASE, 0, 0b11);
pub const MASK_CTRL_COMMAND: UInst = mk_val(CTRL_BASE, 2, 0b11);

// GCTRL

pub const GCTRL_BASE : u32 = CTRL_END;
pub const GCTRL_END  : u32 = GCTRL_BASE + 7;

// `fetchtransitions` (FTs) and `jumpmodes` (JMs) share the same bits.

pub const GCTRL_FT_NONE     : UInst = mk_val(GCTRL_BASE, 0, 0b0000);
// During an ENTER:
// 1. The INSTMASK flag is set. (Thus it is safe to set while already set.);
pub const GCTRL_FT_ENTER    : UInst = mk_val(GCTRL_BASE, 0, 0b0001);
// During a MAYBEEXIT/EXIT:
// 1. RIP is INC'd in any case.
// 2. If (is an EXIT || the high RIR bit is set) the INSTMASK flag is unset.
// 3. ONLY IF MAYBEEXIT: Store in RIR the value on Bus::B
//                 ----> is Bus::B access sufficient for jumping for some offset?
pub const GCTRL_FT_MAYBEEXIT: UInst = mk_val(GCTRL_BASE, 0, 0b0010);
pub const GCTRL_FT_EXIT     : UInst = mk_val(GCTRL_BASE, 0, 0b0011);
// REMEMBER: EVERY JUMPMODE IMPLIES FT_ENTER!
//
// Note: A jumpmode is the condition under which a store
// (input) to RIP from BUSB occurs. A 'P' (if present) stands for
// "pseudo", where we have fake jumpmode which is mutually-
// exclusive with any FT or JM.
pub const GCTRL_JM_YES         : UInst = mk_val(GCTRL_BASE, 0, 0b0100);
// HARDWARE NOTE: despite the different names, GCTRL_JM_P_RIP_BUSB_O is the dual output version of GCTRL_JM_YES (input to RIP)!
pub const GCTRL_JM_P_RIP_BUSB_O: UInst = mk_val(GCTRL_BASE, 0, 0b0101);   //FIXME consider integrating with the CREG output mechanism (can't see a way to make this work off the top of my head);
pub const GCTRL_JM_HALT        : UInst = mk_val(GCTRL_BASE, 0, 0b0110);
// ABRT halts and sets the abort flag as well.
// Aborts are useful as a breakpoint in the VM, but along
// with an LED or two will help with hardware debugging as well.
pub const GCTRL_JM_ABRT        : UInst = mk_val(GCTRL_BASE, 0, 0b0111);

pub const GCTRL_JCOND_CARRY  : UInst = mk_val(GCTRL_BASE, 0, 0b1000);
pub const GCTRL_JCOND_N_ZERO : UInst = mk_val(GCTRL_BASE, 0, 0b1001);
pub const GCTRL_JCOND_SIGN   : UInst = mk_val(GCTRL_BASE, 0, 0b1010);
pub const GCTRL_JCOND_N_OVFLW: UInst = mk_val(GCTRL_BASE, 0, 0b1011);

// NOT A REAL BIT, JUST A HELPER FOR THE 4 FLAG JMs
pub const GCTRL_JM_INVERTCOND : UInst = mk_val(GCTRL_BASE, 0, 0b0100);

// The GCTRL modes

/*
    These "normal" modes are selected by the absence of ACTION_GCTRL_USE_ALT.
*/
pub const GCTRL_NRM_NONE        : UInst = mk_val(GCTRL_BASE, 4, 0b00);
/*
    GCTRL_CREG_I means do an IO read. GCTRL_CREG_O means do an IO write.
*/
pub const GCTRL_NRM_IO_READWRITE: UInst = mk_val(GCTRL_BASE, 4, 0b01);
/*
    GCTRL_CREG_O means to force IU3 to RSP. GCTRL_CREG_I is UNUSED.
*/
pub const GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED: UInst = mk_val(GCTRL_BASE, 4, 0b10);
pub const _GCTRL_NRM__UNUSED     : UInst = mk_val(GCTRL_BASE, 4, 0b11);

/*
    These "alternate" modes occupy the same bits as the "normal" modes,
    and are selected by ACTION_GCTRL_USE_ALT.
*/
pub const GCTRL_ALT_CREG_FG    : UInst = mk_val(GCTRL_BASE, 4, 0b00);
pub const GCTRL_ALT_CREG_IHPR  : UInst = mk_val(GCTRL_BASE, 4, 0b01);
/*
    NOTE next two bits not registers. The first is just a bit (IE);
    which is set with I and cleared with O below (not actually
    inputing or outputting). The other is an ad-hoc compactification
    of two mutually exclusive operations.
    P_IE is the "interrupt enable" flag.
*/
pub const GCTRL_ALT_P_IE  : UInst = mk_val(GCTRL_BASE, 4, 0b10);
// HARDWARE NOTE
// GCTRL_CREG_P_O_CHNMI_OR_I_ALUFG on O: clears "handling NMI" flag,
// on I: loads only the ALU bits of FG
pub const GCTRL_ALT_P_O_CHNMI_OR_I_ALUFG: UInst = mk_val(GCTRL_BASE, 4, 0b11);

// HARDWARE NOTE
// These two bits, when a normal CREG (FG or IHPR) are selected,
// indicate whether there will be output or input from the reg
// to Bus::B.
pub const GCTRL_CREG_O : UInst = mk_val(GCTRL_BASE, 6, 0);
pub const GCTRL_CREG_I : UInst = mk_val(GCTRL_BASE, 6, 1);

// NONBIT: GCTRL decoding
pub const MASK_GCTRL_FTJM: UInst = mk_val(GCTRL_BASE, 0, 0b1111);
pub const MASK_GCTRL_MODE: UInst = mk_val(GCTRL_BASE, 4, 0b11);
pub const MASK_GCTRL_DIR:  UInst = mk_val(GCTRL_BASE, 6, 0b1);

pub const fn gctrl_creg_is_input(ui: UInst) -> bool {
    (ui & MASK_GCTRL_DIR) == GCTRL_CREG_I
}

pub const fn gctrl_creg_is_output(ui: UInst) -> bool {
    (ui & MASK_GCTRL_DIR) == GCTRL_CREG_O
}

// HARDWARE NOTE: These are helper macros, but they should be actual signal lines on the board.
pub fn is_gctrl_nrm_io_readwrite(ui: UInst) -> bool {
    ((ui & MASK_CTRL_ACTION) != ACTION_GCTRL_USE_ALT) && ((ui & MASK_GCTRL_MODE) == GCTRL_NRM_IO_READWRITE)
}

pub fn does_override_iu3_via_command(ui: UInst) -> bool {
    (ui & MASK_CTRL_COMMAND) == COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP
}

pub fn does_override_iu3_via_gctrl_alt(ui: UInst) -> bool {
    ((ui & MASK_CTRL_ACTION) != ACTION_GCTRL_USE_ALT)
                && ((ui & MASK_GCTRL_MODE) == GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED)
                && ((ui & MASK_GCTRL_DIR) == GCTRL_CREG_O)
}

// RCTRL
pub const RCTRL_BASE : u32 = GCTRL_END;
pub const RCTRL_END : u32 = RCTRL_BASE + 9;

pub const RCTRL_IU1_BUSA_I: UInst = mk_val(RCTRL_BASE, 0, 0b100);
pub const RCTRL_IU1_BUSA_O: UInst = mk_val(RCTRL_BASE, 0, 0b101);
pub const RCTRL_IU1_BUSB_I: UInst = mk_val(RCTRL_BASE, 0, 0b110);
pub const RCTRL_IU1_BUSB_O: UInst = mk_val(RCTRL_BASE, 0, 0b111);
pub const RCTRL_IU2_BUSA_I: UInst = mk_val(RCTRL_BASE, 3, 0b100);
pub const RCTRL_IU2_BUSA_O: UInst = mk_val(RCTRL_BASE, 3, 0b101);
pub const RCTRL_IU2_BUSB_I: UInst = mk_val(RCTRL_BASE, 3, 0b110);
pub const RCTRL_IU2_BUSB_O: UInst = mk_val(RCTRL_BASE, 3, 0b111);
pub const RCTRL_IU3_BUSA_I: UInst = mk_val(RCTRL_BASE, 6, 0b100);
pub const RCTRL_IU3_BUSA_O: UInst = mk_val(RCTRL_BASE, 6, 0b101);
pub const RCTRL_IU3_BUSB_I: UInst = mk_val(RCTRL_BASE, 6, 0b110);
pub const RCTRL_IU3_BUSB_O: UInst = mk_val(RCTRL_BASE, 6, 0b111);

// HARDWARE NOTE: In hardware we prohibit inputing a
// register referenced by an IU at the same time it is
// commanded to output as referenced by a different IU,
// by just inhibiting each individual register store pin
// when the output pin is active.

// NONBIT: RCTRL decoding
pub const MASK_RCTRL_IU : u16 = 0b111;

pub const fn rctrl_decode_iu(iu: IU, val : UInst) -> u16 {
    let off = IU::WIDTH * (iu as u32);
    dec_val(RCTRL_BASE, off, mk_val(RCTRL_BASE, off, MASK_RCTRL_IU), val) as u16
}

pub const fn rctrl_decode_iu1(val : UInst) -> u16 {
    rctrl_decode_iu(IU::ONE, val)
}

pub const fn rctrl_decode_iu2(val : UInst) -> u16 {
    rctrl_decode_iu(IU::TWO, val)
}

pub const fn rctrl_decode_iu3(val : UInst) -> u16 {
    rctrl_decode_iu(IU::THREE, val)
}

pub const fn rctrl_iu_is_en(dec: u16) -> bool {
    0b100 & dec != 0
}

pub fn rctrl_iu_get_bus(dec: u16) -> Bus {
    if 0b010 & dec == 0 { Bus::A } else { Bus::B }
}

pub const fn rctrl_iu_is_input(dec: u16) -> bool {
    0b001 & dec == 0
}

pub const fn rctrl_iu_is_output(dec: u16) -> bool {
    0b001 & dec != 0
}

// MCTRL
pub const MCTRL_BASE : u32 = RCTRL_END;
pub const MCTRL_END : u32 = MCTRL_BASE + 6;

pub const MCTRL_MODE_STPFX    : UInst = mk_val(MCTRL_BASE, 0, 0b000);
pub const MCTRL_MODE_STPFX_FAR: UInst = mk_val(MCTRL_BASE, 0, 0b010);
pub const MCTRL_MODE_FO       : UInst = mk_val(MCTRL_BASE, 0, 0b100);
pub const MCTRL_MODE_FO_MI    : UInst = mk_val(MCTRL_BASE, 0, 0b101);
pub const MCTRL_MODE_FO_MI_FAR: UInst = mk_val(MCTRL_BASE, 0, 0b001);
pub const MCTRL_MODE_FI       : UInst = mk_val(MCTRL_BASE, 0, 0b110);
pub const MCTRL_MODE_FI_MO    : UInst = mk_val(MCTRL_BASE, 0, 0b111);
pub const MCTRL_MODE_FI_MO_FAR: UInst = mk_val(MCTRL_BASE, 0, 0b011);

// HARDWARE NOTE: the same it is used for FO and FI, but NOTE
// STPFX is weird and uses different flags to indicate we are
// using far or not!
pub const MCTRL_FLAG_MODE_N_FAR: UInst = mk_val(MCTRL_BASE, 0, 0b100);

// HARDWARE NOTE JUMPER ---- MCTRL_BIOS_WRITE

// HARDWARE NOTE: MCTRL_BUSMODE_DISABLE inhibits the mode setting as well!!!
// HARDWARE NOTE: if the mode is MODE_STPFX or MODE_STPFX_FAR, then ignore MCTRL_BUSMODE_CONW_BUSB.
pub const MCTRL_BUSMODE_DISABLE            : UInst = mk_val(MCTRL_BASE, 3, 0b000);
pub const MCTRL_BUSMODE_CONW_BUSM          : UInst = mk_val(MCTRL_BASE, 3, 0b001);
pub const MCTRL_BUSMODE_CONW_BUSB          : UInst = mk_val(MCTRL_BASE, 3, 0b011);
pub const MCTRL_BUSMODE_CONW_BUSB_MAYBEFLIP: UInst = mk_val(MCTRL_BASE, 3, 0b010);
pub const MCTRL_BUSMODE_CONH               : UInst = mk_val(MCTRL_BASE, 3, 0b100);
pub const _MCTRL_BUSMODE__CONH_NO_X         : UInst = mk_val(MCTRL_BASE, 3, 0b100);
pub const _MCTRL_BUSMODE__CONH_X            : UInst = mk_val(MCTRL_BASE, 3, 0b101);

pub const _MCTRL_BUSMODE__UNUSED_1          : UInst = mk_val(MCTRL_BASE, 3, 0b110);
pub const _MCTRL_BUSMODE__UNUSED_2          : UInst = mk_val(MCTRL_BASE, 3, 0b111);

// NOTE: this bit position must be chosen with the actual values of the BUSMODE_xxx values
pub const MCTRL_BUSMODE_WRITE : UInst = mk_val(MCTRL_BASE, 3, 0b001);

// NOBIT:
// FLAGS for the CONH busmode
pub const MASK_MCTRL_MODE     : UInst = mk_val(MCTRL_BASE, 0, 0b111);
pub const MASK_MCTRL_BUSMODE  : UInst = mk_val(MCTRL_BASE, 3, 0b111);

// ACTRL
pub const ACTRL_BASE : u32 = MCTRL_END;
pub const ACTRL_END : u32 = ACTRL_BASE + 6;

pub const ACTRL_INPUT_EN: UInst = mk_val(ACTRL_BASE, 0, 1);
pub const ACTRL_DATA_OUT: UInst =  mk_val(ACTRL_BASE, 1, 1);
pub const ACTRL_FLAGS_OUT: UInst = mk_val(ACTRL_BASE, 2, 1);

pub const ACTRL_MODE_ADD: UInst =  mk_val(ACTRL_BASE, 3, 0);
pub const ACTRL_MODE_SUB: UInst =  mk_val(ACTRL_BASE, 3, 1);
pub const ACTRL_MODE_AND: UInst =  mk_val(ACTRL_BASE, 3, 2);
pub const ACTRL_MODE_OR: UInst =   mk_val(ACTRL_BASE, 3, 3);
pub const ACTRL_MODE_XOR: UInst =  mk_val(ACTRL_BASE, 3, 4);
pub const ACTRL_MODE_LSFT: UInst = mk_val(ACTRL_BASE, 3, 5);
pub const ACTRL_MODE_RSFT: UInst = mk_val(ACTRL_BASE, 3, 6);
pub const ACTRL_MODE_TST: UInst =  mk_val(ACTRL_BASE, 3, 7);

// NONBIT: ACTRL decoding
pub const MASK_ACTRL_MODE: UInst = mk_val(ACTRL_BASE, 3, 0b111);

pub const fn decode_actrl_mode(ui: UInst) -> u8 {
    ((ui & MASK_ACTRL_MODE) >> (3 + ACTRL_BASE)) as u8 
}

#[allow(unused)]
pub const UCODE_END : u32 = ACTRL_END;

pub const _UCODE_TYPE_BITS : usize = std::mem::size_of::<UInst>() * 8;
pub const _UCODE_MAX_BITS : usize = 32;

const_assert!(UCODE_END as usize <= _UCODE_TYPE_BITS);
const_assert!(UCODE_END as usize <= _UCODE_MAX_BITS);

// END DECLS