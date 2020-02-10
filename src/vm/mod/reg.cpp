#include "../../spec/ucode.h"
#include "reg.h"
#include <cassert>

const char * PREG_NAMES[] = {
    "rid",
    "r1",
    "ra",
    "rb",
    "rc",
    "rd",
    "rsp",
    "rbp",
};

mod_reg::mod_reg(vm_logger &logger) : logger(logger) {
    for(int i = 0; i < NUM_PREGS; i++) {
        reg[i] = 0;
    }
    reg[REG_ONE] = ~0;
}

void mod_reg::dump_registers() {
    logger.logf("RID: %04X\n", reg[REG_ID]);
    logger.logf("RA:  %04X RB:  %04X\n", reg[REG_A], reg[REG_B]);
    logger.logf("RC:  %04X RD:  %04X\n", reg[REG_C], reg[REG_D]);
    logger.logf("RSP: %04X RBP: %04X\n", reg[REG_SP], reg[REG_BP]);
}

regval_t mod_reg::get(preg_t r) {
    if(r >= NUM_PREGS) {
        throw "invalid preg id";
    }

    return reg[r];
}

void mod_reg::maybe_assign(bus_state &s, uinst_t ui, uint8_t iu, preg_t r) {
    if(RCTRL_IU_IS_EN(iu) && RCTRL_IU_IS_OUTPUT(iu)) {
        logger.logf("  %s <- %s:", BUS_NAMES[RCTRL_IU_GET_BUS(iu)], PREG_NAMES[r]);
        s.assign(RCTRL_IU_GET_BUS(iu), reg[r]);

        if(r == REG_SP && (ui & RCTRL_RSP_INC)) {
            assert(!(ui & RCTRL_RSP_INC));
        }
    }
}

void mod_reg::maybe_read(bus_state &s, uinst_t ui, uint8_t iu, preg_t r) {
    if(RCTRL_IU_IS_EN(iu) && RCTRL_IU_IS_INPUT(iu)) {
        logger.logf("  %s -> %s:", BUS_NAMES[RCTRL_IU_GET_BUS(iu)], PREG_NAMES[r]);
        reg[r] = s.read(RCTRL_IU_GET_BUS(iu));

        if(r == REG_SP && (ui & RCTRL_RSP_INC)) {
            assert(!(ui & RCTRL_RSP_INC));
        }
    }
}

void mod_reg::clock_outputs(regval_t inst, uinst_t ui, bus_state &s) {
    maybe_assign(s, ui, RCTRL_DECODE_IU1(ui), INST_GET_IU1(inst));
    maybe_assign(s, ui, RCTRL_DECODE_IU2(ui), INST_GET_IU2(inst));
    maybe_assign(s, ui, RCTRL_DECODE_IU3(ui), INST_GET_IU3(inst));
}

void mod_reg::clock_inputs(regval_t inst, uinst_t ui, bus_state &s) {
    maybe_read(s, ui, RCTRL_DECODE_IU1(ui), INST_GET_IU1(inst));
    maybe_read(s, ui, RCTRL_DECODE_IU2(ui), INST_GET_IU2(inst));
    maybe_read(s, ui, RCTRL_DECODE_IU3(ui), INST_GET_IU3(inst));

    if(ui & RCTRL_RSP_INC) {
        reg[REG_SP] += 2;
    }
}
