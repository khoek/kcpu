#include "../../spec/ucode.hpp"
#include "../../spec/inst.hpp"
#include "../common.hpp"
#include "reg.hpp"

namespace kcpu {

mod_reg::mod_reg(vm_logger &logger) : logger(logger) {
    for(int i = 0; i < NUM_PREGS; i++) {
        reg[i] = 0;
    }
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

static bool should_perform_rsp_early_inc(uinst_t ui) {
    return (ui & MASK_CTRL_COMMAND) == COMMAND_RCTRL_RSP_EARLY_INC;
}

static bool should_perform_rsp_early_dec(uinst_t ui) {
    return (ui & MASK_CTRL_COMMAND) == COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP;
}

void mod_reg::maybe_assign(bus_state &s, regval_t inst, uinst_t ui, uint8_t iunum, uint8_t iu, preg_t r) {
    if(RCTRL_IU_IS_EN(iu) && RCTRL_IU_IS_OUTPUT(iu)) {
        if(logger.dump_bus) logger.logf("  iu%d: %s <- %s:", iunum, BUS_NAMES[RCTRL_IU_GET_BUS(iu)], PREG_NAMES[r]);
        s.assign(RCTRL_IU_GET_BUS(iu), reg[r]);

        // NOTE Even if `r == REG_SP` we don't need to check for should_perform_rsp_inc/dec() here,
        // since there would be no timing problem (the DEC occurs on the offclock just before this clock).
    }
}

void mod_reg::maybe_read(bus_state &s, regval_t inst, uinst_t ui, uint8_t iunum, uint8_t iu, preg_t r) {
    if(RCTRL_IU_IS_EN(iu) && RCTRL_IU_IS_INPUT(iu)) {
        if(logger.dump_bus) logger.logf("  iu%d: %s -> %s:", iunum, BUS_NAMES[RCTRL_IU_GET_BUS(iu)], PREG_NAMES[r]);
        reg[r] = s.read(RCTRL_IU_GET_BUS(iu));

        // NOTE Even if `r == REG_SP` we don't need to check for should_perform_rsp_inc/dec() here,
        // since there would be no timing problem (the DEC occurs on the offclock just before this clock).
    }
}

/*
    HARDWARE NOTE: This RSP INC/DEC occurs on the falling edge of the clock, when
    the new ucode is about to be latched, but nonetheless depends on the NEW VALUE
    of `ui & MASK_CTRL_COMMAND`. So, it peeks ahead in this way.
*/
void mod_reg::offclock_pulse(uinst_t ui) {
    vm_assert(!should_perform_rsp_early_inc(ui) || !should_perform_rsp_early_dec(ui));

    if(should_perform_rsp_early_dec(ui)) {
        reg[REG_SP] -= 2;
    }

    if(should_perform_rsp_early_inc(ui)) {
        reg[REG_SP] += 2;
    }
}

static preg_t consider_iu3_override(uinst_t ui, regval_t inst) {
    preg_t iu3_reg = INST_GET_IU3(inst);

    if(does_override_iu3_via_command(ui) || does_override_iu3_via_gctrl_alt(ui)) {
        iu3_reg = REG_SP;
    }

    return iu3_reg;
}

void mod_reg::clock_outputs(uinst_t ui, bus_state &s, regval_t inst) {
    maybe_assign(s, inst, ui, 1, RCTRL_DECODE_IU1(ui), INST_GET_IU1(inst));
    maybe_assign(s, inst, ui, 2, RCTRL_DECODE_IU2(ui), INST_GET_IU2(inst));
    maybe_assign(s, inst, ui, 3, RCTRL_DECODE_IU3(ui), consider_iu3_override(ui, inst));
}

void mod_reg::clock_inputs(uinst_t ui, bus_state &s, regval_t inst) {
    maybe_read(s, inst, ui, 1, RCTRL_DECODE_IU1(ui), INST_GET_IU1(inst));
    maybe_read(s, inst, ui, 2, RCTRL_DECODE_IU2(ui), INST_GET_IU2(inst));
    maybe_read(s, inst, ui, 3, RCTRL_DECODE_IU3(ui), consider_iu3_override(ui, inst));
}

}