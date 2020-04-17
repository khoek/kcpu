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
#define ACTION__UNUSED          (0b01ULL << (0 + CTRL_BASE))
#define ACTION_GCTRL_RIP_BUSA_O (0b10ULL << (0 + CTRL_BASE))
#define ACTION_MCTRL_BUSMODE_X  (0b11ULL << (0 + CTRL_BASE))

#define COMMAND_NONE            (0b00ULL << (2 + CTRL_BASE))
#define COMMAND_IO_READ         (0b01ULL << (2 + CTRL_BASE))
#define COMMAND_IO_WRITE        (0b10ULL << (2 + CTRL_BASE))
#define COMMAND_RCTRL_RSP_INC   (0b11ULL << (2 + CTRL_BASE))

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

// Note there is space here for one more possibility, CREG_NONE active but GCTRL_CREG_BUSB_O selected
#define GCTRL_CREG_NONE   (0b00ULL << (4 + GCTRL_BASE))
#define GCTRL_CREG_FG     (0b01ULL << (4 + GCTRL_BASE))
#define GCTRL_CREG_IHPR   (0b10ULL << (4 + GCTRL_BASE))
// P_IE is the "interrupt enable" flag.
// NOTE not a real register, just a bit which is set with I and
// cleared with O below (not actually inputing or outputting).
#define GCTRL_CREG_P_IE (0b11ULL << (4 + GCTRL_BASE))

// HARDWARE NOTE
// These two bits, when a normal CREG (FG or IHPR) are selected,
// indicate whether there will be output or input from the reg
// to BUS_B.
#define GCTRL_CREG_I (0ULL << (6 + GCTRL_BASE))
#define GCTRL_CREG_O (1ULL << (6 + GCTRL_BASE))

// NONBIT: GCTRL decoding
#define MASK_GCTRL_FTJM (0b1111ULL << (0 + GCTRL_BASE))
#define MASK_GCTRL_CREG (0b11ULL << (4 + GCTRL_BASE))
#define MASK_GCTRL_DIR (0b1ULL << (6 + GCTRL_BASE))
#define GCTRL_DECODE_CREG(val) (((val) & MASK_GCTRL_CREG) >> (4 + GCTRL_BASE))
#define GCTRL_CREG_IS_INPUT(dec) (!(dec & MASK_GCTRL_DIR))
#define GCTRL_CREG_IS_OUTPUT(dec) (!!(dec & MASK_GCTRL_DIR))

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