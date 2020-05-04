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



// BEGIN DECLS

// CTRL

pub const CTRL_BASE : u32 = 0;
pub const CTRL_END  : u32 = CTRL_BASE + 4;

// Random mutually exclusive "ACTION"s
pub const ACTION_CTRL_NONE        : UInst = 0b00 << (0 + CTRL_BASE);
pub const ACTION_GCTRL_USE_ALT    : UInst = 0b01 << (0 + CTRL_BASE);
pub const ACTION_GCTRL_RIP_BUSA_O : UInst = 0b10 << (0 + CTRL_BASE);
pub const ACTION_MCTRL_BUSMODE_X  : UInst = 0b11 << (0 + CTRL_BASE);

pub const COMMAND_NONE        : UInst = 0b00 << (2 + CTRL_BASE);
/*
    HARDWARE NOTE: `COMMAND_INHIBIT_JMFT` disallows the instmask-setting
    and UC-resetting behaviour of all JMs/FTs, just for that uop. This
    is currently used to implement `_DO_INT`.
*/
pub const COMMAND_INHIBIT_JMFT: UInst = 0b01 << (2 + CTRL_BASE);
/*
    There next two increment/decrement RSP ON THE CLOCK RISING EDGE.
    (RSP is usually decremented on the offclock cycle by an instruction register bit.);

    HARDWARE NOTE: COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP implicitly activates the
    IU3_OVERRIDE_O_SELECT_RSP behaviour.
*/
pub const COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP: UInst = 0b10 << (2 + CTRL_BASE);
pub const COMMAND_RCTRL_RSP_EARLY_INC: UInst = 0b11 << (2 + CTRL_BASE);

// NONBIT: CTRL decoding
pub const MASK_CTRL_ACTION: UInst = 0b11 << (0 + CTRL_BASE);
pub const MASK_CTRL_COMMAND: UInst = 0b11 << (2 + CTRL_BASE);

// GCTRL

pub const GCTRL_BASE : u32 = CTRL_END;
pub const GCTRL_END  : u32 = GCTRL_BASE + 7;

// `fetchtransitions` (FTs) and `jumpmodes` (JMs) share the same bits.

pub const GCTRL_FT_NONE     : UInst = 0b0000 << (0 + GCTRL_BASE);
// During an ENTER:
// 1. The INSTMASK flag is set. (Thus it is safe to set while already set.);
pub const GCTRL_FT_ENTER    : UInst = 0b0001 << (0 + GCTRL_BASE);
// During a MAYBEEXIT/EXIT:
// 1. RIP is INC'd in any case.
// 2. If (is an EXIT || the high RIR bit is set) the INSTMASK flag is unset.
// 3. ONLY IF MAYBEEXIT: Store in RIR the value on Bus::B
//                 ----> is Bus::B access sufficient for jumping for some offset?
pub const GCTRL_FT_MAYBEEXIT: UInst = 0b0010 << (0 + GCTRL_BASE);
pub const GCTRL_FT_EXIT     : UInst = 0b0011 << (0 + GCTRL_BASE);
// REMEMBER: EVERY JUMPMODE IMPLIES FT_ENTER!
//
// Note: A jumpmode is the condition under which a store
// (input) to RIP from BUSB occurs. A 'P' (if present) stands for
// "pseudo", where we have fake jumpmode which is mutually-
// exclusive with any FT or JM.
pub const GCTRL_JM_YES         : UInst = 0b0100 << (0 + GCTRL_BASE);
// HARDWARE NOTE: despite the different names, GCTRL_JM_P_RIP_BUSB_O is the dual output version of GCTRL_JM_YES (input to RIP)!
pub const GCTRL_JM_P_RIP_BUSB_O: UInst = 0b0101 << (0 + GCTRL_BASE);   //FIXME consider integrating with the CREG output mechanism (can't see a way to make this work off the top of my head);
pub const GCTRL_JM_HALT        : UInst = 0b0110 << (0 + GCTRL_BASE);
// ABRT halts and sets the abort flag as well.
// Aborts are useful as a breakpoint in the VM, but along
// with an LED or two will help with hardware debugging as well.
pub const GCTRL_JM_ABRT        : UInst = 0b0111 << (0 + GCTRL_BASE);

pub const GCTRL_JCOND_CARRY  : UInst = 0b1000 << (0 + GCTRL_BASE);
pub const GCTRL_JCOND_N_ZERO : UInst = 0b1001 << (0 + GCTRL_BASE);
pub const GCTRL_JCOND_SIGN   : UInst = 0b1010 << (0 + GCTRL_BASE);
pub const GCTRL_JCOND_N_OVFLW: UInst = 0b1011 << (0 + GCTRL_BASE);

// NOT A REAL BIT, JUST A HELPER FOR THE 4 FLAG JMs
pub const GCTRL_JM_INVERTCOND : UInst = 0b0100 << (0 + GCTRL_BASE);

// The GCTRL modes

/*
    These "normal" modes are selected by the absence of ACTION_GCTRL_USE_ALT.
*/
pub const GCTRL_NRM_NONE        : UInst = 0b00 << (4 + GCTRL_BASE);
/*
    GCTRL_CREG_I means do an IO read. GCTRL_CREG_O means do an IO write.
*/
pub const GCTRL_NRM_IO_READWRITE: UInst = 0b01 << (4 + GCTRL_BASE);
/*
    GCTRL_CREG_O means to force IU3 to RSP. GCTRL_CREG_I is UNUSED.
*/
pub const GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED: UInst = 0b10 << (4 + GCTRL_BASE);
pub const GCTRL_NRM__UNUSED     : UInst = 0b11 << (4 + GCTRL_BASE);

/*
    These "alternate" modes occupy the same bits as the "normal" modes,
    and are selected by ACTION_GCTRL_USE_ALT.
*/
pub const GCTRL_ALT_CREG_FG    : UInst = 0b00 << (4 + GCTRL_BASE);
pub const GCTRL_ALT_CREG_IHPR  : UInst = 0b01 << (4 + GCTRL_BASE);
/*
    NOTE next two bits not registers. The first is just a bit (IE);
    which is set with I and cleared with O below (not actually
    inputing or outputting). The other is an ad-hoc compactification
    of two mutually exclusive operations.
    P_IE is the "interrupt enable" flag.
*/
pub const GCTRL_ALT_P_IE  : UInst = 0b10 << (4 + GCTRL_BASE);
// HARDWARE NOTE
// GCTRL_CREG_P_O_CHNMI_OR_I_ALUFG on O: clears "handling NMI" flag,
// on I: loads only the ALU bits of FG
pub const GCTRL_ALT_P_O_CHNMI_OR_I_ALUFG: UInst = 0b11 << (4 + GCTRL_BASE);

// HARDWARE NOTE
// These two bits, when a normal CREG (FG or IHPR) are selected,
// indicate whether there will be output or input from the reg
// to Bus::B.
pub const GCTRL_CREG_O : UInst = 0 << (6 + GCTRL_BASE);
pub const GCTRL_CREG_I : UInst = 1 << (6 + GCTRL_BASE);

// NONBIT: GCTRL decoding
pub const MASK_GCTRL_FTJM: UInst = 0b1111 << (0 + GCTRL_BASE);
pub const MASK_GCTRL_MODE: UInst = 0b11 << (4 + GCTRL_BASE);
pub const MASK_GCTRL_DIR:  UInst = 0b1 << (6 + GCTRL_BASE);

pub const fn gctrl_creg_is_input(ui: UInst) -> bool {
    (ui & MASK_GCTRL_DIR) == GCTRL_CREG_I
}

pub const fn gctrl_creg_is_output(ui: UInst) -> bool {
    (ui & MASK_GCTRL_DIR) == GCTRL_CREG_O
}

// HARDWARE NOTE: These are helper macros, but they should be actual signal lines on the board.
pub fn is_gctrl_nrm_io_readwrite(ui: UInst) -> bool {
    return ((ui & MASK_CTRL_ACTION) != ACTION_GCTRL_USE_ALT) && ((ui & MASK_GCTRL_MODE) == GCTRL_NRM_IO_READWRITE);
}

// RUSTFIX what is this even used for, it seems to not even be used in either version of the VM!?!?!
pub fn is_gctrl_nrm_iu3_override_o_select_rsp(ui: UInst) -> bool {
    return ((ui & MASK_CTRL_ACTION) != ACTION_GCTRL_USE_ALT) && ((ui & MASK_GCTRL_MODE) == GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED);
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

pub const RCTRL_IU1_BUSA_I: UInst = 0b100 << (0 + RCTRL_BASE);
pub const RCTRL_IU1_BUSA_O: UInst = 0b101 << (0 + RCTRL_BASE);
pub const RCTRL_IU1_BUSB_I: UInst = 0b110 << (0 + RCTRL_BASE);
pub const RCTRL_IU1_BUSB_O: UInst = 0b111 << (0 + RCTRL_BASE);
pub const RCTRL_IU2_BUSA_I: UInst = 0b100 << (3 + RCTRL_BASE);
pub const RCTRL_IU2_BUSA_O: UInst = 0b101 << (3 + RCTRL_BASE);
pub const RCTRL_IU2_BUSB_I: UInst = 0b110 << (3 + RCTRL_BASE);
pub const RCTRL_IU2_BUSB_O: UInst = 0b111 << (3 + RCTRL_BASE);
pub const RCTRL_IU3_BUSA_I: UInst = 0b100 << (6 + RCTRL_BASE);
pub const RCTRL_IU3_BUSA_O: UInst = 0b101 << (6 + RCTRL_BASE);
pub const RCTRL_IU3_BUSB_I: UInst = 0b110 << (6 + RCTRL_BASE);
pub const RCTRL_IU3_BUSB_O: UInst = 0b111 << (6 + RCTRL_BASE);

// HARDWARE NOTE: In hardware we prohibit inputing a
// register referenced by an IU at the same time it is
// commanded to output as referenced by a different IU,
// by just inhibiting each individual register store pin
// when the output pin is active.

// NONBIT: RCTRL decoding
pub const MASK_RCTRL_IU : UInst = 0b111;
pub const MASK_RCTRL_IU1 : UInst = MASK_RCTRL_IU << (0 + RCTRL_BASE);
pub const MASK_RCTRL_IU2 : UInst = MASK_RCTRL_IU << (3 + RCTRL_BASE);
pub const MASK_RCTRL_IU3 : UInst = MASK_RCTRL_IU << (6 + RCTRL_BASE);

//RUSTFIX use the `IU` enum to parse a particular parameter set
pub const fn rctrl_decode_iu1(iu : UInst) -> u16 {
    ((iu & (MASK_RCTRL_IU << (0 + RCTRL_BASE))) >> (0 + RCTRL_BASE)) as u16
}

pub const fn rctrl_decode_iu2(iu : UInst) -> u16 {
    ((iu & (MASK_RCTRL_IU << (3 + RCTRL_BASE))) >> (3 + RCTRL_BASE)) as u16
}

pub const fn rctrl_decode_iu3(iu : UInst) -> u16 {
    ((iu & (MASK_RCTRL_IU << (6 + RCTRL_BASE))) >> (6 + RCTRL_BASE)) as u16
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

pub const MCTRL_MODE_STPFX    : UInst = 0b000 << (0 + MCTRL_BASE);
pub const MCTRL_MODE_STPFX_FAR: UInst = 0b010 << (0 + MCTRL_BASE);
pub const MCTRL_MODE_FO       : UInst = 0b100 << (0 + MCTRL_BASE);
pub const MCTRL_MODE_FO_MI    : UInst = 0b101 << (0 + MCTRL_BASE);
pub const MCTRL_MODE_FO_MI_FAR: UInst = 0b001 << (0 + MCTRL_BASE);
pub const MCTRL_MODE_FI       : UInst = 0b110 << (0 + MCTRL_BASE);
pub const MCTRL_MODE_FI_MO    : UInst = 0b111 << (0 + MCTRL_BASE);
pub const MCTRL_MODE_FI_MO_FAR: UInst = 0b011 << (0 + MCTRL_BASE);

// HARDWARE NOTE: the same it is used for FO and FI, but NOTE
// STPFX is weird and uses different flags to indicate we are
// using far or not!
pub const MCTRL_FLAG_MODE_N_FAR: UInst = 0b100 << (0 + MCTRL_BASE);

// HARDWARE NOTE JUMPER ---- MCTRL_BIOS_WRITE

// HARDWARE NOTE: MCTRL_BUSMODE_DISABLE inhibits the mode setting as well!!!
// HARDWARE NOTE: if the mode is MODE_STPFX or MODE_STPFX_FAR, then ignore MCTRL_BUSMODE_CONW_BUSB.
pub const MCTRL_BUSMODE_DISABLE            : UInst = 0b000 << (3 + MCTRL_BASE);
pub const MCTRL_BUSMODE_CONW_BUSM          : UInst = 0b001 << (3 + MCTRL_BASE);
pub const MCTRL_BUSMODE_CONW_BUSB          : UInst = 0b011 << (3 + MCTRL_BASE);
pub const MCTRL_BUSMODE_CONW_BUSB_MAYBEFLIP: UInst = 0b010 << (3 + MCTRL_BASE);
pub const MCTRL_BUSMODE_CONH               : UInst = 0b100 << (3 + MCTRL_BASE);
pub const MCTRL_BUSMODE__CONH_NO_X         : UInst = 0b100 << (3 + MCTRL_BASE);
pub const MCTRL_BUSMODE__CONH_X            : UInst = 0b101 << (3 + MCTRL_BASE);

pub const MCTRL_BUSMODE__UNUSED_1          : UInst = 0b110 << (3 + MCTRL_BASE);
pub const MCTRL_BUSMODE__UNUSED_2          : UInst = 0b111 << (3 + MCTRL_BASE);

// NOTE: this bit position must be chosen with the actual values of the BUSMODE_xxx values
pub const MCTRL_BUSMODE_WRITE : UInst = 0b001 << (3 + MCTRL_BASE);

// NOBIT:
// FLAGS for the CONH busmode
pub const MASK_MCTRL_MODE     : UInst = 0b111 << (0 + MCTRL_BASE);
pub const MASK_MCTRL_BUSMODE  : UInst = 0b111 << (3 + MCTRL_BASE);

// ACTRL
pub const ACTRL_BASE : u32 = MCTRL_END;
pub const ACTRL_END : u32 = ACTRL_BASE + 6;

pub const ACTRL_INPUT_EN: UInst = 1 << (0 + ACTRL_BASE);
pub const ACTRL_DATA_OUT: UInst =  1 << (1 + ACTRL_BASE);
pub const ACTRL_FLAGS_OUT: UInst = 1 << (2 + ACTRL_BASE);

pub const ACTRL_MODE_ADD: UInst =  0 << (3 + ACTRL_BASE);
pub const ACTRL_MODE_SUB: UInst =  1 << (3 + ACTRL_BASE);
pub const ACTRL_MODE_AND: UInst =  2 << (3 + ACTRL_BASE);
pub const ACTRL_MODE_OR: UInst =   3 << (3 + ACTRL_BASE);
pub const ACTRL_MODE_XOR: UInst =  4 << (3 + ACTRL_BASE);
pub const ACTRL_MODE_LSFT: UInst = 5 << (3 + ACTRL_BASE);
pub const ACTRL_MODE_RSFT: UInst = 6 << (3 + ACTRL_BASE);
pub const ACTRL_MODE_TST: UInst =  7 << (3 + ACTRL_BASE);

// NONBIT: ACTRL decoding
pub const MASK_ACTRL_MODE: UInst = 0b111 << (3 + ACTRL_BASE);

pub const fn decode_actrl_mode(ui: UInst) -> u8 {
    ((ui & MASK_ACTRL_MODE) >> (3 + ACTRL_BASE)) as u8 
}

pub const UCODE_END : u32 = ACTRL_END;

pub const MASK_I_INVERT : UInst = 0;


pub const UCODE_TYPE_BITS : usize = std::mem::size_of::<UInst>() * 8;
pub const UCODE_MAX_BITS : usize = 32;

const_assert!(UCODE_END as usize <= UCODE_TYPE_BITS);
const_assert!(UCODE_END as usize <= UCODE_MAX_BITS);

// END DECLS