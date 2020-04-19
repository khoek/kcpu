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

struct iu_state {
    regval_t dec[3];
    preg_t iu[3];
};

static iu_state parse_ius(uinst_t ui, regval_t inst) {
    iu_state state;
    state.dec[0] = RCTRL_DECODE_IU1(ui);
    state.iu[0] = INST_GET_IU1(inst);
    state.dec[1] = RCTRL_DECODE_IU2(ui);
    state.iu[1] = INST_GET_IU2(inst);
    state.dec[2] = RCTRL_DECODE_IU3(ui);
    state.iu[2] = consider_iu3_override(ui, inst);
    return state;
}

/*
    HARDWARE NOTE: This function emulates the much simpler hardware
    protection mechanism, which is just to inhbit the I line to each
    register (after passing through the IU machinery) if the O line
    is also asserted.
*/
static iu_state filter_simultaneous_i_and_o(iu_state is) {
    for(int i = 0; i < 3; i++) {
        if(!RCTRL_IU_IS_EN(is.dec[i])) {
            continue;
        }

        for(int j = 0; j < 3; j++) {
            if(i == j || !RCTRL_IU_IS_EN(is.dec[j])) {
                continue;
            }

            if(is.iu[i] == is.iu[j]) {
                bool has_i = RCTRL_IU_IS_INPUT(is.iu[i]) || RCTRL_IU_IS_INPUT(is.iu[j]);
                bool has_o = RCTRL_IU_IS_OUTPUT(is.iu[i]) || RCTRL_IU_IS_OUTPUT(is.iu[j]);
                if(has_i && has_o) {
                    is.dec[RCTRL_IU_IS_INPUT(is.iu[i]) ? i : j] = 0;
                }
            }
        }
    }
    return is;
}

void mod_reg::clock_outputs(uinst_t ui, bus_state &s, regval_t inst) {
    iu_state is = parse_ius(ui, inst);
    is = filter_simultaneous_i_and_o(is);

    maybe_assign(s, inst, ui, 1, is.dec[0], is.iu[0]);
    maybe_assign(s, inst, ui, 2, is.dec[1], is.iu[1]);
    maybe_assign(s, inst, ui, 3, is.dec[2], is.iu[2]);
}

void mod_reg::clock_inputs(uinst_t ui, bus_state &s, regval_t inst) {
    iu_state is = parse_ius(ui, inst);
    is = filter_simultaneous_i_and_o(is);

    maybe_read(s, inst, ui, 1, is.dec[0], is.iu[0]);
    maybe_read(s, inst, ui, 2, is.dec[1], is.iu[1]);
    maybe_read(s, inst, ui, 3, is.dec[2], is.iu[2]);
}

}