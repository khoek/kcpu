#include "../../spec/ucode.hpp"
#include "reg.hpp"
#include <cassert>

namespace kcpu {

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
        throw vm_error("invalid preg id");
    }

    return reg[r];
}

void mod_reg::maybe_assign(bus_state &s, uinst_t ui, uint8_t iunum, uint8_t iu, preg_t r) {
    if(RCTRL_IU_IS_EN(iu) && RCTRL_IU_IS_OUTPUT(iu)) {
        if(logger.dump_bus) logger.logf("  iu%d: %s <- %s:", iunum, BUS_NAMES[RCTRL_IU_GET_BUS(iu)], PREG_NAMES[r]);
        s.assign(RCTRL_IU_GET_BUS(iu), reg[r]);

        if(r == REG_SP) {
            assert((ui & MASK_CTRL_ACTION) != ACTION_RCTRL_RSP_INC);
            assert((ui & MASK_CTRL_ACTION) != ACTION_RCTRL_RSP_DEC);
        }
    }
}

void mod_reg::maybe_read(bus_state &s, uinst_t ui, uint8_t iunum, uint8_t iu, preg_t r) {
    if(RCTRL_IU_IS_EN(iu) && RCTRL_IU_IS_INPUT(iu)) {
        if(logger.dump_bus) logger.logf("  iu%d: %s -> %s:", iunum, BUS_NAMES[RCTRL_IU_GET_BUS(iu)], PREG_NAMES[r]);
        reg[r] = s.read(RCTRL_IU_GET_BUS(iu));

        if(r == REG_SP) {
            assert((ui & MASK_CTRL_ACTION) != ACTION_RCTRL_RSP_INC);
            assert((ui & MASK_CTRL_ACTION) != ACTION_RCTRL_RSP_DEC);
        }
    }
}

void mod_reg::clock_outputs(regval_t inst, uinst_t ui, bus_state &s) {
    maybe_assign(s, ui, 1, RCTRL_DECODE_IU1(ui), INST_GET_IU1(inst));
    maybe_assign(s, ui, 2, RCTRL_DECODE_IU2(ui), INST_GET_IU2(inst));
    maybe_assign(s, ui, 3, RCTRL_DECODE_IU3(ui), INST_GET_IU3(inst));
}

void mod_reg::clock_inputs(regval_t inst, uinst_t ui, bus_state &s) {
    maybe_read(s, ui, 1, RCTRL_DECODE_IU1(ui), INST_GET_IU1(inst));
    maybe_read(s, ui, 2, RCTRL_DECODE_IU2(ui), INST_GET_IU2(inst));
    maybe_read(s, ui, 3, RCTRL_DECODE_IU3(ui), INST_GET_IU3(inst));

    switch(ui & MASK_CTRL_ACTION) {
        case ACTION_CTRL_NONE:
        case ACTION_GCTRL_RFG_BUSB_I: {
            break;
        }
        case ACTION_RCTRL_RSP_INC: {
            reg[REG_SP] += 2;
            break;
        }
        case ACTION_RCTRL_RSP_DEC: {
            reg[REG_SP] -= 2;
            break;
        }
        default: throw vm_error("unknown GCTRL_ACTION");
    }
}

}