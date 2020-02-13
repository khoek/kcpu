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
    REG_ONE = 1,
    REG_SP = 2,
    REG_BP = 3,
    REG_A = 4,
    REG_B = 5,
    REG_C = 6,
    REG_D = 7,
};
#define PREG_NUL ((preg_t) 0)

static const char * PREG_NAMES[] = {
    "id",
    "1",
    "sp",
    "bp",
    "a",
    "b",
    "c",
    "d",
};

#define NUM_SREGS 5
enum sreg_t {
    REG_IP = 0,
    REG_UC = 1,
    REG_IR = 2,
    REG_FG = 3,
};

}

#endif