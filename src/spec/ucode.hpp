#ifndef SPEC_UCODE_H
#define SPEC_UCODE_H

#include "../types.hpp"

namespace kcpu {

#define UCODE_BITS ((sizeof(uinst_t) / sizeof(uint8_t)) * 8)

// BEGIN DECLS

#define CTRL_BASE 0
#define CTRL_END (CTRL_BASE + 4)

// Random mutually exclusive "ACTION"s
#define ACTION_CTRL_NONE        (0b00ULL << (0 + CTRL_BASE))
#define ACTION_GCTRL_USE_ALT    (0b01ULL << (0 + CTRL_BASE))
#define ACTION_GCTRL_RIP_BUSA_O (0b10ULL << (0 + CTRL_BASE))
#define ACTION_MCTRL_BUSMODE_X  (0b11ULL << (0 + CTRL_BASE))

#define COMMAND_NONE         (0b00ULL << (2 + CTRL_BASE))
/*
    HARDWARE NOTE: `COMMAND_INHIBIT_JMFT` disallows the instmask-setting
    and UC-resetting behaviour of all JMs/FTs, just for that uop. This
    is currently used to implement `_DO_INT`.
*/
#define COMMAND_INHIBIT_JMFT (0b01ULL << (2 + CTRL_BASE))
/*
    There next two increment/decrement RSP ON THE CLOCK RISING EDGE.
    (RSP is usually decremented on the offclock cycle by an instruction register bit.)

    HARDWARE NOTE: COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP implicitly activates the
    IU3_OVERRIDE_O_SELECT_RSP behaviour.
*/
#define COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP (0b10ULL << (2 + CTRL_BASE))
#define COMMAND_RCTRL_RSP_EARLY_INC (0b11ULL << (2 + CTRL_BASE))

// NONBIT: CTRL decoding
#define MASK_CTRL_ACTION (0b11ULL << (0 + CTRL_BASE))
#define MASK_CTRL_COMMAND (0b11ULL << (2 + CTRL_BASE))

// `fetchtransitions` (FTs) and `jumpmodes` (JMs) share the same bits.

#define GCTRL_BASE CTRL_END
#define GCTRL_END (GCTRL_BASE + 7)

#define GCTRL_FT_NONE      (0b0000ULL << (0 + GCTRL_BASE))
// During an ENTER:
// 1. The INSTMASK flag is set. (Thus it is safe to set while already set.)
#define GCTRL_FT_ENTER     (0b0001ULL << (0 + GCTRL_BASE))
// During a MAYBEEXIT/EXIT:
// 1. RIP is INC'd in any case.
// 2. If (is an EXIT || the high RIR bit is set) the INSTMASK flag is unset.
// 3. ONLY IF MAYBEEXIT: Store in RIR the value on BUS_B
//                 ----> is BUS_B access sufficient for jumping for some offset?
#define GCTRL_FT_MAYBEEXIT (0b0010ULL << (0 + GCTRL_BASE))
#define GCTRL_FT_EXIT      (0b0011ULL << (0 + GCTRL_BASE))
// REMEMBER: EVERY JUMPMODE IMPLIES FT_ENTER!
//
// Note: A jumpmode is the condition under which a store
// (input) to RIP from BUSB occurs. A 'P' (if present) stands for
// "pseudo", where we have fake jumpmode which is mutually-
// exclusive with any FT or JM.
#define GCTRL_JM_YES          (0b0100ULL << (0 + GCTRL_BASE))
// HARDWARE NOTE: despite the different names, GCTRL_JM_P_RIP_BUSB_O is the dual output version of GCTRL_JM_YES (input to RIP)!
#define GCTRL_JM_P_RIP_BUSB_O (0b0101ULL << (0 + GCTRL_BASE))   //FIXME consider integrating with the CREG output mechanism (can't see a way to make this work off the top of my head)
#define GCTRL_JM_HALT         (0b0110ULL << (0 + GCTRL_BASE))
// ABRT halts and sets the abort flag as well.
// Aborts are useful as a breakpoint in the VM, but along
// with an LED or two will help with hardware debugging as well.
#define GCTRL_JM_ABRT         (0b0111ULL << (0 + GCTRL_BASE))

#define GCTRL_JCOND_CARRY   (0b1000ULL << (0 + GCTRL_BASE))
#define GCTRL_JCOND_N_ZERO  (0b1001ULL << (0 + GCTRL_BASE))
#define GCTRL_JCOND_SIGN    (0b1010ULL << (0 + GCTRL_BASE))
#define GCTRL_JCOND_N_OVFLW (0b1011ULL << (0 + GCTRL_BASE))

// NOT A REAL BIT, JUST A HELPER FOR THE 4 FLAG JMs
#define GCTRL_JM_INVERTCOND  (0b0100ULL << (0 + GCTRL_BASE))

// FIXME switch the function of ACTION_GCTRL_CREG_EN to more of a switching between two alternate sets of actions
// type thing (then this field is not active, then the zero-val of the GCTRL_CREG of the selected set better do
// nothing!).
//
// Then make COMMAND_IO_READWRITE one of these functions (we will then have an unused COMMAND_xxxxx). Make
// IU3_OVERRIDE_SELECT_RSP another. Use IU3_OVERRIDE_SELECT_RSP to remove all of the X_blah instructions.
// We will have to keep X_ENTERFR (since the alias passes the basepointer as well), but we can remove it off the
// IU3_SINGLE opclass list and change its opcode to a normal one, fitting in with the other ENTER/LEAVE stuff.
// (We will still need X_ENTER and X_LEAVE as well.)
//
// NOTE when we change stuff like X_PUSH over to use IU3 to get RSP, remember to rename the IU2 reference to an IU1 reference.
//
// Finally, make the _DO_INT handler use this mechanism as well, and thus change its code in mod_ctl (no longer)
// needs to be passed REG_RSP in IU1.

// The GCTRL modes

/*
    These "normal" modes are selected by the absence of ACTION_GCTRL_USE_ALT.
*/
#define GCTRL_NRM_NONE         (0b00ULL << (4 + GCTRL_BASE))
/*
    GCTRL_CREG_I means do an IO read. GCTRL_CREG_O means do an IO write.
*/
#define GCTRL_NRM_IO_READWRITE (0b01ULL << (4 + GCTRL_BASE))
/*
    GCTRL_CREG_O means to force IU3 to RSP. GCTRL_CREG_I is UNUSED.
*/
#define GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED (0b10ULL << (4 + GCTRL_BASE))
#define GCTRL_NRM__UNUSED      (0b11ULL << (4 + GCTRL_BASE))

/*
    These "alternate" modes occupy the same bits as the "normal" modes,
    and are selected by ACTION_GCTRL_USE_ALT.
*/
#define GCTRL_ALT_CREG_FG     (0b00ULL << (4 + GCTRL_BASE))
#define GCTRL_ALT_CREG_IHPR   (0b01ULL << (4 + GCTRL_BASE))
/*
    NOTE next two bits not registers. The first is just a bit (IE)
    which is set with I and cleared with O below (not actually
    inputing or outputting). The other is an ad-hoc compactification
    of two mutually exclusive operations.
    P_IE is the "interrupt enable" flag.
*/
#define GCTRL_ALT_P_IE   (0b10ULL << (4 + GCTRL_BASE))
// HARDWARE NOTE
// GCTRL_CREG_P_O_CHNMI_OR_I_ALUFG on O: clears "handling NMI" flag,
// on I: loads only the ALU bits of FG
#define GCTRL_ALT_P_O_CHNMI_OR_I_ALUFG (0b11ULL << (4 + GCTRL_BASE))

// HARDWARE NOTE
// These two bits, when a normal CREG (FG or IHPR) are selected,
// indicate whether there will be output or input from the reg
// to BUS_B.
#define GCTRL_CREG_O (0ULL << (6 + GCTRL_BASE))
#define GCTRL_CREG_I (1ULL << (6 + GCTRL_BASE))

// NONBIT: GCTRL decoding
#define MASK_GCTRL_FTJM (0b1111ULL << (0 + GCTRL_BASE))
#define MASK_GCTRL_MODE (0b11ULL << (4 + GCTRL_BASE))
#define MASK_GCTRL_DIR (0b1ULL << (6 + GCTRL_BASE))
#define GCTRL_CREG_IS_INPUT(dec) ((dec & MASK_GCTRL_DIR) == GCTRL_CREG_I)
#define GCTRL_CREG_IS_OUTPUT(dec) ((dec & MASK_GCTRL_DIR) == GCTRL_CREG_O)

// HARDWARE NOTE: These are helper macros, but they should be actual signal lines on the board.
static bool is_gctrl_nrm_io_readwrite(uinst_t ui) {
    return ((ui & MASK_CTRL_ACTION) != ACTION_GCTRL_USE_ALT) && ((ui & MASK_GCTRL_MODE) == GCTRL_NRM_IO_READWRITE);
}

static bool is_gctrl_nrm_iu3_override_o_select_rsp(uinst_t ui) {
    return ((ui & MASK_CTRL_ACTION) != ACTION_GCTRL_USE_ALT) && ((ui & MASK_GCTRL_MODE) == GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED);
}

static bool does_override_iu3_via_command(uinst_t ui) {
    return ((ui & MASK_CTRL_COMMAND) == COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP);
}

static bool does_override_iu3_via_gctrl_alt(uinst_t ui) {
    return ((ui & MASK_CTRL_ACTION) != ACTION_GCTRL_USE_ALT)
                && ((ui & MASK_GCTRL_MODE) == GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED)
                && ((ui & MASK_GCTRL_DIR) == GCTRL_CREG_O);
}

// RCTRL
#define RCTRL_BASE GCTRL_END
#define RCTRL_END (RCTRL_BASE + 9)

#define RCTRL_IU1_BUSA_I (0b100ULL << (0 + RCTRL_BASE))
#define RCTRL_IU1_BUSA_O (0b101ULL << (0 + RCTRL_BASE))
#define RCTRL_IU1_BUSB_I (0b110ULL << (0 + RCTRL_BASE))
#define RCTRL_IU1_BUSB_O (0b111ULL << (0 + RCTRL_BASE))
#define RCTRL_IU2_BUSA_I (0b100ULL << (3 + RCTRL_BASE))
#define RCTRL_IU2_BUSA_O (0b101ULL << (3 + RCTRL_BASE))
#define RCTRL_IU2_BUSB_I (0b110ULL << (3 + RCTRL_BASE))
#define RCTRL_IU2_BUSB_O (0b111ULL << (3 + RCTRL_BASE))
#define RCTRL_IU3_BUSA_I (0b100ULL << (6 + RCTRL_BASE))
#define RCTRL_IU3_BUSA_O (0b101ULL << (6 + RCTRL_BASE))
#define RCTRL_IU3_BUSB_I (0b110ULL << (6 + RCTRL_BASE))
#define RCTRL_IU3_BUSB_O (0b111ULL << (6 + RCTRL_BASE))

// NONBIT: RCTRL decoding
#define MASK_RCTRL_IU 0b111ULL
#define MASK_RCTRL_IU1 (MASK_RCTRL_IU << (0 + RCTRL_BASE))
#define MASK_RCTRL_IU2 (MASK_RCTRL_IU << (3 + RCTRL_BASE))
#define MASK_RCTRL_IU3 (MASK_RCTRL_IU << (6 + RCTRL_BASE))
#define RCTRL_DECODE_IU1(val) (((val) & (MASK_RCTRL_IU << (0 + RCTRL_BASE))) >> (0 + RCTRL_BASE))
#define RCTRL_DECODE_IU2(val) (((val) & (MASK_RCTRL_IU << (3 + RCTRL_BASE))) >> (3 + RCTRL_BASE))
#define RCTRL_DECODE_IU3(val) (((val) & (MASK_RCTRL_IU << (6 + RCTRL_BASE))) >> (6 + RCTRL_BASE))
#define RCTRL_IU_IS_EN(dec) (!!(0b100ULL & dec))
#define RCTRL_IU_GET_BUS(dec) (0b010ULL & dec ? BUS_B : BUS_A)
#define RCTRL_IU_IS_INPUT(dec) (!(0b001ULL & dec))
#define RCTRL_IU_IS_OUTPUT(dec) (!!(0b001ULL & dec))

// MCTRL
#define MCTRL_BASE RCTRL_END
#define MCTRL_END (MCTRL_BASE + 6)

#define MCTRL_MODE_STPFX     (0b000ULL << (0 + MCTRL_BASE))
#define MCTRL_MODE_STPFX_FAR (0b010ULL << (0 + MCTRL_BASE))
#define MCTRL_MODE_FO        (0b100ULL << (0 + MCTRL_BASE))
#define MCTRL_MODE_FO_MI     (0b101ULL << (0 + MCTRL_BASE))
#define MCTRL_MODE_FO_MI_FAR (0b001ULL << (0 + MCTRL_BASE))
#define MCTRL_MODE_FI        (0b110ULL << (0 + MCTRL_BASE))
#define MCTRL_MODE_FI_MO     (0b111ULL << (0 + MCTRL_BASE))
#define MCTRL_MODE_FI_MO_FAR (0b011ULL << (0 + MCTRL_BASE))

// HARDWARE NOTE: the same it is used for FO and FI, but NOTE
// STPFX is weird and uses different flags to indicate we are
// using far or not!
#define MCTRL_FLAG_MODE_N_FAR (0b100ULL << (0 + MCTRL_BASE))

// HARDWARE NOTE JUMPER ---- MCTRL_BIOS_WRITE

// HARDWARE NOTE: MCTRL_BUSMODE_DISABLE inhibits the mode setting as well!!!
// HARDWARE NOTE: if the mode is MODE_STPFX or MODE_STPFX_FAR, then ignore MCTRL_BUSMODE_CONW_BUSB.
#define MCTRL_BUSMODE_DISABLE             (0b000ULL << (3 + MCTRL_BASE))
#define MCTRL_BUSMODE_CONW_BUSM           (0b001ULL << (3 + MCTRL_BASE))
#define MCTRL_BUSMODE_CONW_BUSB           (0b011ULL << (3 + MCTRL_BASE))
#define MCTRL_BUSMODE_CONW_BUSB_MAYBEFLIP (0b010ULL << (3 + MCTRL_BASE))
#define MCTRL_BUSMODE_CONH                (0b100ULL << (3 + MCTRL_BASE))
#define MCTRL_BUSMODE__CONH_NO_X          (0b100ULL << (3 + MCTRL_BASE))
#define MCTRL_BUSMODE__CONH_X             (0b101ULL << (3 + MCTRL_BASE))

#define MCTRL_BUSMODE__UNUSED_1           (0b110ULL << (3 + MCTRL_BASE))
#define MCTRL_BUSMODE__UNUSED_2           (0b111ULL << (3 + MCTRL_BASE))

// NOTE: this bit position must be chosen with the actual values of the BUSMODE_xxx values
#define MCTRL_BUSMODE_WRITE  (0b001ULL << (3 + MCTRL_BASE))

// NOBIT:
// FLAGS for the CONH busmode
#define MASK_MCTRL_MODE      (0b111ULL << (0 + MCTRL_BASE))
#define MASK_MCTRL_BUSMODE   (0b111ULL << (3 + MCTRL_BASE))

// ACTRL
#define ACTRL_BASE MCTRL_END
#define ACTRL_END (ACTRL_BASE + 6)

#define ACTRL_INPUT_EN  (1ULL << (0 + ACTRL_BASE))
#define ACTRL_DATA_OUT  (1ULL << (1 + ACTRL_BASE))
#define ACTRL_FLAGS_OUT (1ULL << (2 + ACTRL_BASE))

#define ACTRL_MODE_ADD  (0ULL << (3 + ACTRL_BASE))
#define ACTRL_MODE_SUB  (1ULL << (3 + ACTRL_BASE))
#define ACTRL_MODE_AND  (2ULL << (3 + ACTRL_BASE))
#define ACTRL_MODE_OR   (3ULL << (3 + ACTRL_BASE))
#define ACTRL_MODE_XOR  (4ULL << (3 + ACTRL_BASE))
#define ACTRL_MODE_LSFT (5ULL << (3 + ACTRL_BASE))
#define ACTRL_MODE_RSFT (6ULL << (3 + ACTRL_BASE))
#define ACTRL_MODE_TST  (7ULL << (3 + ACTRL_BASE))

// NONBIT: ACTRL decoding
#define MASK_ACTRL_MODE (0b111ULL << (3 + ACTRL_BASE))
#define DECODE_ACTRL_MODE(ui) (((ui & MASK_ACTRL_MODE) >> (3 + ACTRL_BASE)))

#define UCODE_END ACTRL_END

// END DECLS

// Bits which are active low, and thus we should invert during
// instruction registration (to prevent having to include them in
// every uinst in which they should be disabled).
#define MASK_I_INVERT (0)

static_assert(UCODE_END <= UCODE_BITS, "The UCODE is too long!");

}

#endif