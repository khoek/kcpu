#ifndef SPEC_HW_H
#define SPEC_HW_H

#include "../types.hpp"

namespace kcpu {

#define ADDR_WIDTH 13
#define UCVAL_WIDTH 2
#define INST_WIDTH 9
#define CHIP_SELECT_WIDTH (ADDR_WIDTH - (UCVAL_WIDTH + INST_WIDTH))

#define NUM_IUS 3
#define IU_WIDTH 3

#define UCVAL_MAX ((1 << UCVAL_WIDTH) - 1)
#define INST_MAX ((1 << INST_WIDTH) - 1)
#define UCODE_LEN (1 << ADDR_WIDTH)
#define OPCODE_LEN (1 << INST_WIDTH)

#define IU_MASK 0b111

#define INST_MK_IU1(reg) (((reg) & IU_MASK) << (0 * IU_WIDTH))
#define INST_MK_IU2(reg) (((reg) & IU_MASK) << (1 * IU_WIDTH))
#define INST_MK_IU3(reg) (((reg) & IU_MASK) << (2 * IU_WIDTH))

#define INST_GET_IU1(inst) ((kcpu::preg_t) (((inst) & (IU_MASK << (0 * IU_WIDTH))) >> (0 * IU_WIDTH)))
#define INST_GET_IU2(inst) ((kcpu::preg_t) (((inst) & (IU_MASK << (1 * IU_WIDTH))) >> (1 * IU_WIDTH)))
#define INST_GET_IU3(inst) ((kcpu::preg_t) (((inst) & (IU_MASK << (2 * IU_WIDTH))) >> (2 * IU_WIDTH)))
#define INST_GET_IUS(inst) { INST_GET_IU1(inst), INST_GET_IU2(inst), INST_GET_IU3(inst), }

#define NUM_BUSES 4
enum bus_t {
    BUS_A = 0,
    BUS_B = 1,
// The below busses float.
#define BUS_FIRST_FLOATER BUS_M
    BUS_M = 2,
    BUS_F = 3,
};

#define NUM_PREGS 8
enum preg_t {
// NOTE PREG_NAMES must be kept in-sync with this list.
    REG_ID = 0,
    REG_SP = 1,
    REG_BP = 2,
    REG_A  = 3,
    REG_B  = 4,
    REG_C  = 5,
    REG_D  = 6,
    REG_E  = 7,
};

static const char * PREG_NAMES[] = {
    "id",
    "sp",
    "bp",
    "a",
    "b",
    "c",
    "d",
    "e",
};

#define NUM_SREGS 6
enum sreg_t {
// First 0-1 are "c(ontrol)reg"s, remainder are private.
// HARDWARE NOTE: the CREG-codes in the ucode depend
//                on this order of the first 4.

// HARDWARE NOTE: REG_FG has its low byte connected to the ALU,
// and its high byte connected to CTL (currently, the latter is
// to control CBIT_IE only). This means that the only the low byte
// of the memory of REG_FG_RAW is actually ever nonzero while
// we are simulating in the VM.
    REG_FG_RAW   = 0,
    REG_IHP      = 1,

    REG_IP       = 2,
    REG_UC       = 3,
    REG_IR       = 4,
// HARDWARE NOTE: 3 unused registers
};

}

#endif