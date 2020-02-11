#ifndef SPEC_UCODE_H
#define SPEC_UCODE_H

#include "../types.h"

#define UCODE_BITS ((sizeof(uinst_t) / sizeof(uint8_t)) * 8)

#define GCTRL_BASE 0
#define GCTRL_END (GCTRL_BASE + 8)

// `fetchtransitions` (FTs) and `jumpmodes` (JMs) share the same bits.

#define GCTRL_FT_NONE      (0b000LL << (0 + GCTRL_BASE))
// During an ENTER:
// 1. The INSTMASK flag is set. (Thus it is safe to set while already set.)
#define GCTRL_FT_ENTER     (0b001LL << (0 + GCTRL_BASE))
// During a MAYBEEXIT/EXIT:
// 1. RIP is INC'd in any case.
// 2. If (is an EXIT || the high RIR bit is set) the INSTMASK flag is unset.
// 3. ONLY IF MAYBEEXIT: Store in RIR the value on BUS_B
//                 ----> is BUS_B access sufficient for jumping for some offset?
#define GCTRL_FT_MAYBEEXIT (0b010LL << (0 + GCTRL_BASE))
#define GCTRL_FT_EXIT      (0b011LL << (0 + GCTRL_BASE))

// REMEMBER: EVERY JUMPMODE IMPLIES FT_ENTER!
//
// Note: A jumpmode is the condition under which a store
// (input) to RIP from BUSB occurs. The 'P' stands for
// "pseudo", where we have fake jumpmode which is mutually-
// exclusive with any FT or JM.
#define GCTRL_JM_YES          (0b100LL << (0 + GCTRL_BASE))
#define GCTRL_JM_ON_TRUE      (0b101LL << (0 + GCTRL_BASE))
#define GCTRL_JM_ON_FALSE     (0b110LL << (0 + GCTRL_BASE))
#define GCTRL_JM_P_RIP_BUSB_O (0b111LL << (0 + GCTRL_BASE))

#define GCTRL_JCOND_CARRY   (0b00LL << (3 + GCTRL_BASE))
#define GCTRL_JCOND_N_ZERO  (0b01LL << (3 + GCTRL_BASE))
#define GCTRL_JCOND_SIGN    (0b10LL << (3 + GCTRL_BASE))
#define GCTRL_JCOND_N_OVFLW (0b11LL << (3 + GCTRL_BASE))

#define GCTRL_ACTION_NONE       (0b00LL << (5 + GCTRL_BASE))
#define GCTRL_ACTION_RIP_BUSA_O (0b01LL << (5 + GCTRL_BASE))
#define GCTRL_ACTION_RFG_BUSB_I (0b10LL << (5 + GCTRL_BASE))
// A GCTRL_ACTION_STOP without GCTRL_FT_ENTER halts
// the computer. GCTRL_ACTION_STOP with GCTRL_FT_ENTER
// is an "abort", which halts and sets the abort flag as well.
// Aborts are useful as a breakpoint in the VM, but along
// with an LED or two will help with hardware debugging as well.
#define GCTRL_ACTION_STOP       (0b11LL << (5 + GCTRL_BASE))

// NONBIT: GCTRL decoding
#define MASK_GCTRL_FTJM (0b111LL << (0 + GCTRL_BASE))
#define MASK_GCTRL_JCOND (0b11LL << (3 + GCTRL_BASE))
#define MASK_GCTRL_ACTION (0b11LL << (5 + GCTRL_BASE))

// RCTRL
#define RCTRL_BASE GCTRL_END
#define RCTRL_END (RCTRL_BASE + 11)
#define RCTRL_IU1_BUSA_I (0b100LL << (0 + RCTRL_BASE))
#define RCTRL_IU1_BUSA_O (0b101LL << (0 + RCTRL_BASE))
#define RCTRL_IU1_BUSB_I (0b110LL << (0 + RCTRL_BASE))
#define RCTRL_IU1_BUSB_O (0b111LL << (0 + RCTRL_BASE))
#define RCTRL_IU2_BUSA_I (0b100LL << (3 + RCTRL_BASE))
#define RCTRL_IU2_BUSA_O (0b101LL << (3 + RCTRL_BASE))
#define RCTRL_IU2_BUSB_I (0b110LL << (3 + RCTRL_BASE))
#define RCTRL_IU2_BUSB_O (0b111LL << (3 + RCTRL_BASE))
#define RCTRL_IU3_BUSA_I (0b100LL << (6 + RCTRL_BASE))
#define RCTRL_IU3_BUSA_O (0b101LL << (6 + RCTRL_BASE))
#define RCTRL_IU3_BUSB_I (0b110LL << (6 + RCTRL_BASE))
#define RCTRL_IU3_BUSB_O (0b111LL << (6 + RCTRL_BASE))
#define RCTRL_RSP_INC (1LL << (9 + RCTRL_BASE))
#define RCTRL_RSP_DEC (1LL << (10 + RCTRL_BASE))

// NONBIT: RCTRL decoding
#define MASK_RCTRL_IU 0b111LL
#define MASK_RCTRL_IU1 (MASK_RCTRL_IU << (0 + RCTRL_BASE))
#define MASK_RCTRL_IU2 (MASK_RCTRL_IU << (3 + RCTRL_BASE))
#define MASK_RCTRL_IU3 (MASK_RCTRL_IU << (6 + RCTRL_BASE))
#define RCTRL_DECODE_IU1(val) (((val) & (MASK_RCTRL_IU << (0 + RCTRL_BASE))) >> (0 + RCTRL_BASE))
#define RCTRL_DECODE_IU2(val) (((val) & (MASK_RCTRL_IU << (3 + RCTRL_BASE))) >> (3 + RCTRL_BASE))
#define RCTRL_DECODE_IU3(val) (((val) & (MASK_RCTRL_IU << (6 + RCTRL_BASE))) >> (6 + RCTRL_BASE))
#define RCTRL_IU_IS_EN(dec) (!!(0b100LL & dec))
#define RCTRL_IU_GET_BUS(dec) (0b010LL & dec ? BUS_B : BUS_A)
#define RCTRL_IU_IS_INPUT(dec) (!(0b001LL & dec))
#define RCTRL_IU_IS_OUTPUT(dec) (!!(0b001LL & dec))

// MCTRL
#define MCTRL_BASE RCTRL_END
#define MCTRL_END (MCTRL_BASE + 11)
#define MCTRL_USE_PREFIX_FAR (1LL << (0 + MCTRL_BASE))
#define MCTRL_PREFIX_STORE   (1LL << (1 + MCTRL_BASE))
#define MCTRL_N_MAIN_OUT     (1LL << (2 + MCTRL_BASE))
#define MCTRL_MAIN_STORE     (1LL << (3 + MCTRL_BASE))
#define MCTRL_FIDD_STORE     (1LL << (4 + MCTRL_BASE))
#define MCTRL_N_FIDD_OUT     (1LL << (5 + MCTRL_BASE))
// TODO JUMPER #define MCTRL_BIOS_WRITE
#define MCTRL_BUSMODE_WRITE  (1LL << (6 + MCTRL_BASE))
#define MCTRL_BUSMODE_X      (1LL << (7 + MCTRL_BASE))

#define OFF_MCTRL_BUSMODE (8 + MCTRL_BASE)
#define MASK_MCTRL_BUSMODE (0b111LL << OFF_MCTRL_BUSMODE)
#define MCTRL_BUSMODE (8 + MCTRL_BASE)
#define MCTRL_BUSMODE_NONE                (0LL << OFF_MCTRL_BUSMODE)
#define MCTRL_BUSMODE_CONH                (1LL << OFF_MCTRL_BUSMODE)
#define MCTRL_BUSMODE_CONW_BUSM           (2LL << OFF_MCTRL_BUSMODE)
#define MCTRL_BUSMODE_CONW_BUSB           (3LL << OFF_MCTRL_BUSMODE)
#define MCTRL_BUSMODE_CONW_BUSB_MAYBEFLIP (4LL << OFF_MCTRL_BUSMODE)

// ACTRL
#define ACTRL_BASE MCTRL_END
#define ACTRL_END (ACTRL_BASE + 6)
#define ACTRL_INPUT_EN  (1LL << (0 + ACTRL_BASE))
#define ACTRL_DATA_OUT  (1LL << (1 + ACTRL_BASE))
#define ACTRL_FLAGS_OUT (1LL << (2 + ACTRL_BASE))

#define ACTRL_MODE_ADD  (0LL << (3 + ACTRL_BASE))
#define ACTRL_MODE_SUB  (1LL << (3 + ACTRL_BASE))
#define ACTRL_MODE_AND  (2LL << (3 + ACTRL_BASE))
#define ACTRL_MODE_OR   (3LL << (3 + ACTRL_BASE))
#define ACTRL_MODE_XOR  (4LL << (3 + ACTRL_BASE))
#define ACTRL_MODE_LSFT (5LL << (3 + ACTRL_BASE))
#define ACTRL_MODE_RSFT (6LL << (3 + ACTRL_BASE))
#define ACTRL_MODE_TST  (7LL << (3 + ACTRL_BASE))

// NONBIT: ACTRL decoding
#define MASK_ACTRL_MODE (0b111LL << (3 + ACTRL_BASE))
#define DECODE_ACTRL_MODE(ui) (((ui & MASK_ACTRL_MODE) >> (3 + ACTRL_BASE)))

#define UCODE_END ACTRL_END

static_assert(UCODE_END <= UCODE_BITS, "The UCODE is too long!");

#endif