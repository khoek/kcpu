#include "../../lang/arch.hpp"
#include "../../spec/inst.hpp"
#include "../../spec/ucode.hpp"
#include "ctl.hpp"

namespace kcpu {

mod_ctl::mod_ctl(vm_logger &logger) : logger(logger) {
    for(int i = 0; i < NUM_SREGS; i++) {
        reg[i] = 0;
    }

    for(int i = 0; i < NUM_CBITS; i++) {
        cbits[i] = false;
    }

    // I think it is not neccesary to implement this on real hardware, so long as
    // all of the registers (in particular RIR) are initialized to zero. (Since then
    // NOP will be the first instruction executed.)
    cbits[CBIT_INSTMASK] = true;
}

void mod_ctl::dump_registers() {
    logger.logf("RIP: %04X RUC: %04X\n", reg[REG_IP], reg[REG_UC]);
    logger.logf("RIR: %04X RFG: %04X\n", reg[REG_IR], reg[REG_FG]);
    logger.logf("CBITS: %c%c\n", cbits[CBIT_INSTMASK] ? 'M' : 'm', cbits[CBIT_HALTED] ? 'H' : 'h');
}

#define LOAD_INSTVAL 0

regval_t mod_ctl::get_inst() {
    return cbits[CBIT_INSTMASK] ? LOAD_INSTVAL : reg[REG_IR];
}

uinst_t mod_ctl::get_uinst() {
    return arch::self().ucode_read(get_inst(), reg[REG_UC]);
}

void mod_ctl::clock_outputs(uinst_t ui, bus_state &s) {
    // HARDWARE NOTE: in real life this could be toggled-on during a
    // cycle (I think only if we enter a NOP without the mask up),
    // so make sure we wont get a short/bus collision if that happens.
    if(cbits[CBIT_INSTMASK] && !(reg[REG_UC] & 0x1)) {
        s.assign(BUS_A, reg[REG_IP]);
    }

    if((ui & MASK_GCTRL_FTJM) == GCTRL_JM_P_RIP_BUSB_O) {
        s.assign(BUS_B, reg[REG_IP]);
    }
}

void mod_ctl::set_instmask_enabled(bool state) {
    // NOTE implement this condition by funneling it through the command signals latch, without it actually coming
    // from the EEPROMs---thus we won't need any fancy edge-detection stuff. (No race between the next instruction
    // propagating and this condition check reaching the UC reg.)
    if(state != cbits[CBIT_INSTMASK]) {
        reg[REG_UC] = 0;
    }

    cbits[CBIT_INSTMASK] = state;
}

static regval_t decode_jcond_mask(uinst_t ui) {
    switch((ui & MASK_GCTRL_FTJM) & ~GCTRL_JM_INVERTCOND) {
        case GCTRL_JCOND_CARRY:   return (1 << 0);
        case GCTRL_JCOND_N_ZERO:  return (1 << 1);
        case GCTRL_JCOND_SIGN:    return (1 << 2);
        case GCTRL_JCOND_N_OVFLW: return (1 << 3);
        default: throw vm_error("unknown JCOND!");
    }
}

void mod_ctl::ft_enter() {
    set_instmask_enabled(true);
}

void mod_ctl::clock_inputs(uinst_t ui, bus_state &s) {
    // HARDWARE NOTE This register can be simultaneously reset under the GCTRL_FT_ENTER/MAYBEEXIT/EXIT conditions, but we
    // assume that (presumably async) reset signal dominates this increment.
    reg[REG_UC]++;

    switch(ui & MASK_CTRL_ACTION) {
        case ACTION_GCTRL_RFG_BUSB_I: {
            reg[REG_FG] = /* FIXME low mask? */ s.read(BUS_B);
            break;
        }
        case ACTION_CTRL_NONE:
        case ACTION_RCTRL_RSP_INC:
        case ACTION_RCTRL_RSP_DEC: {
            break;
        }
        default: throw vm_error("unknown GCTRL_ACTION");
    }

    switch(ui & MASK_GCTRL_FTJM) {
        case GCTRL_FT_NONE: {
            break;
        }
        case GCTRL_FT_ENTER: {
            ft_enter();
            break;
        }
        case GCTRL_FT_EXIT: {
            reg[REG_IP] += 2;
            set_instmask_enabled(false);
            break;
        }
        case GCTRL_FT_MAYBEEXIT: {
            reg[REG_IP] += 2;
            reg[REG_IR] = s.read(BUS_B);
            set_instmask_enabled(reg[REG_IR] & P_I_LOADDATA);
            break;
        }
        case GCTRL_JM_YES: {
            reg[REG_IP] = s.read(BUS_B);
            ft_enter();
            break;
        }
        case GCTRL_JM_P_RIP_BUSB_O: {
            break;
        }
        case GCTRL_JM_HALT: {
            cbits[CBIT_HALTED] = true;
            break;
        }
        case GCTRL_JM_ABRT: {
            cbits[CBIT_HALTED] = true;
            cbits[CBIT_ABORTED] = true;
            break;
        }
        default: {
            // It was one of the 8 JCOND codes
            if((!!(reg[REG_FG] & decode_jcond_mask(ui))) ^ (!!(ui & GCTRL_JM_INVERTCOND))) {
                reg[REG_IP] = s.read(BUS_B);
            }
            ft_enter();
        }
    }
}

}