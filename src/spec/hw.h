#ifndef SPEC_HW_H
#define SPEC_HW_H

#define UCVAL_WIDTH 4
#define ADDR_WIDTH 13

#define NUM_IUS 3
#define IU_WIDTH 3

#define INST_WIDTH (ADDR_WIDTH - UCVAL_WIDTH)
#define UCVAL_MAX ((1 << UCVAL_WIDTH) - 1)
#define INST_MAX ((1 << INST_WIDTH) - 1)
#define UCODE_LEN (1 << (ADDR_WIDTH - 1))
#define OPCODE_LEN (1 << (INST_WIDTH - 1))

#define INST_SHIFT (2 * IU_WIDTH)

#define IU_MASK 0b111

#define INST_MK_IU1(reg) (((reg) & IU_MASK) << (0 * IU_WIDTH))
#define INST_MK_IU2(reg) (((reg) & IU_MASK) << (1 * IU_WIDTH))
#define INST_MK_IU3(reg) (((reg) & IU_MASK) << (2 * IU_WIDTH))

#define INST_GET_IU1(inst) ((preg_t) (((inst) & (IU_MASK << (0 * IU_WIDTH))) >> (0 * IU_WIDTH)))
#define INST_GET_IU2(inst) ((preg_t) (((inst) & (IU_MASK << (1 * IU_WIDTH))) >> (1 * IU_WIDTH)))
#define INST_GET_IU3(inst) ((preg_t) (((inst) & (IU_MASK << (2 * IU_WIDTH))) >> (2 * IU_WIDTH)))

#define INST_MK(opcode, iu1, iu2, iu3) ((opcode << INST_SHIFT) | INST_MK_IU1(iu1) | INST_MK_IU2(iu2) | INST_MK_IU3(iu3))

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
    REG_ID = 0,
    REG_ONE = 1,
    REG_A = 2,
    REG_B = 3,
    REG_C = 4,
    REG_D = 5,
    REG_SP = 6,
    REG_BP = 7,
};
#define PREG_NUL ((preg_t) 0)

extern const char * PREG_NAMES[];

#define NUM_SREGS 5
enum sreg_t {
    REG_IP = 0,
    REG_UC = 1,
    REG_IR = 2,
    REG_FG = 3,
};

#endif