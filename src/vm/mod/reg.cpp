#include "../../spec/ucode.hpp"
#include "../../spec/inst.hpp"
#include "../common.hpp"
#include "reg.hpp"

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

static bool should_perform_rsp_inc(uinst_t ui) {
    return (ui & MASK_CTRL_COMMAND) == COMMAND_RCTRL_RSP_INC;
}

static bool should_perform_rsp_dec(uinst_t ui) {
    return (ui & MASK_CTRL_COMMAND) == COMMAND_RCTRL_RSP_DEC;
}

void mod_reg::maybe_assign(bus_state &s, regval_t inst, uinst_t ui, uint8_t iunum, uint8_t iu, preg_t r) {
    if(RCTRL_IU_IS_EN(iu) && RCTRL_IU_IS_OUTPUT(iu)) {
        if(logger.dump_bus) logger.logf("  iu%d: %s <- %s:", iunum, BUS_NAMES[RCTRL_IU_GET_BUS(iu)], PREG_NAMES[r]);
        s.assign(RCTRL_IU_GET_BUS(iu), reg[r]);

        if(r == REG_SP) {
            vm_assert(!should_perform_rsp_inc(ui));
            vm_assert(!should_perform_rsp_dec(ui));
            // NOTE don't need to check for P_I_RSPDEC here, since there
            // would be no timing problem (the DEC occurs during the FT LOAD).
        }
    }
}

void mod_reg::maybe_read(bus_state &s, regval_t inst, uinst_t ui, uint8_t iunum, uint8_t iu, preg_t r) {
    if(RCTRL_IU_IS_EN(iu) && RCTRL_IU_IS_INPUT(iu)) {
        if(logger.dump_bus) logger.logf("  iu%d: %s -> %s:", iunum, BUS_NAMES[RCTRL_IU_GET_BUS(iu)], PREG_NAMES[r]);
        reg[r] = s.read(RCTRL_IU_GET_BUS(iu));

        if(r == REG_SP) {
            vm_assert(!should_perform_rsp_inc(ui));
            vm_assert(!should_perform_rsp_dec(ui));
            // NOTE don't need to check for P_I_RSPDEC here, since there
            // would be no timing problem (the DEC occurs during the FT LOAD).
        }
    }
}

void mod_reg::offclock_pulse(regval_t inst, bool first_uop) {
    if((inst & P_I_RSPDEC) && first_uop) {
        reg[REG_SP] -= 2;
    }
}

void mod_reg::clock_outputs(uinst_t ui, bus_state &s, regval_t inst) {
    maybe_assign(s, inst, ui, 1, RCTRL_DECODE_IU1(ui), INST_GET_IU1(inst));
    maybe_assign(s, inst, ui, 2, RCTRL_DECODE_IU2(ui), INST_GET_IU2(inst));
    maybe_assign(s, inst, ui, 3, RCTRL_DECODE_IU3(ui), INST_GET_IU3(inst));
}

void mod_reg::clock_inputs(uinst_t ui, bus_state &s, regval_t inst) {
    maybe_read(s, inst, ui, 1, RCTRL_DECODE_IU1(ui), INST_GET_IU1(inst));
    maybe_read(s, inst, ui, 2, RCTRL_DECODE_IU2(ui), INST_GET_IU2(inst));
    maybe_read(s, inst, ui, 3, RCTRL_DECODE_IU3(ui), INST_GET_IU3(inst));

    vm_assert(!should_perform_rsp_inc(ui) || !should_perform_rsp_dec(ui));

    if(should_perform_rsp_inc(ui)) {
        reg[REG_SP] += 2;
    }

    if(should_perform_rsp_dec(ui)) {
        reg[REG_SP] -= 2;
    }
}

}