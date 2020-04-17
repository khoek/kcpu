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
    logger.logf("CBITS: %c%c%c%c%c PINT(latch): %d AINT: %d\n",
        cbits[CBIT_INSTMASK] ? 'M' : 'm',
        cbits[CBIT_IE]       ? 'I' : 'i',
        cbits[CBIT_IO_WAIT]  ? 'W' : 'w',
        cbits[CBIT_HALTED]   ? 'H' : 'h',
        cbits[CBIT_ABORTED]  ? 'A' : 'a',
        pint_latch_val ? 1 : 0,
        is_aint_active() ? 1 : 0
    );
}

// HARDWARE NOTE: Ensure that the INT instruction has SP in IU1! AND RSP_DEC FLAG!
#define LOAD_INSTVAL INST_MK(false,                  0x0,      0, 0, 0)
#define INT_INSTVAL  INST_MK(false, P_PRE_I_RSPDEC | 0x1, REG_SP, 0, 0)

regval_t mod_ctl::get_inst() {
    // HARDWARE NOTE: CBIT_IO_WAIT inhibits CBIT_INSTMASK, for obvious reasons,
    // EXCEPT WHEN IO_DONE IS ASSERTED, WHEN CBIT_INSTMASK BEHAVES AS NORMAL.
    // (This is the actual behaviour as emulated, see the hardware note in `offclock_pulse()`.)
    return (cbits[CBIT_INSTMASK] && !cbits[CBIT_IO_WAIT]) ? (pint_latch_val ? INT_INSTVAL : LOAD_INSTVAL) : reg[REG_IR];
}

uinst_t mod_ctl::get_uinst() {
    return uinst_latch_val;
}

bool mod_ctl::is_first_uop() {
    return ((get_uinst() & MASK_CTRL_ACTION) != ACTION_GCTRL_RIP_BUSA_O) && (reg[REG_UC] == 0x0);
}

/*
    HARDWARE NOTE: the first boolean below is morally
    neccesary, but as of writing we never latch pint unless the
    INSTMASK is going high, and the pint latch is always cleared
    by the time INSTMASK is cleared.

    (So, at least right now, it can be safely commented.)
*/
bool mod_ctl::is_aint_active() {
    return cbits[CBIT_INSTMASK] && pint_latch_val;
}

void mod_ctl::clock_outputs(uinst_t ui, bus_state &s) {
    if((ui & MASK_GCTRL_FTJM) == GCTRL_JM_P_RIP_BUSB_O) {
        s.assign(BUS_B, reg[REG_IP]);
    }

    switch(ui & MASK_GCTRL_CREG) {
        case GCTRL_CREG_NONE:
        case GCTRL_CREG_P_IE: {
            break;
        }
        case GCTRL_CREG_FG:
        case GCTRL_CREG_IHPR: {
            if(GCTRL_CREG_IS_OUTPUT(ui)) {
                s.assign(BUS_B, reg[GCTRL_DECODE_CREG(ui)]);
            }
            break;
        }
        default: throw vm_error("unknown GCTRL CREG");
    }

    switch(ui & MASK_CTRL_ACTION) {
        case ACTION_CTRL_NONE:
        case ACTION_MCTRL_BUSMODE_X: {
            break;
        }
        case ACTION_GCTRL_RIP_BUSA_O: {
            if(!cbits[CBIT_IO_WAIT]) {
                s.assign(BUS_A, reg[REG_IP]);
            }
            break;
        }
        default: throw vm_error("unknown GCTRL_ACTION");
    }
}

void mod_ctl::set_instmask_enabled(uinst_t ui, bool state, bool pint) {
    /*
        We only examine pint (the second condition) when we execute the
        first instruction of a NOP without the INSTMASK set.

        HARDWARE NOTE: Remember though, this function is "called" when
                        either the INTMASK_SET or INTMASK_CLEAR lines are
                        asserted.
    */
    if(((ui & MASK_CTRL_ACTION) != ACTION_GCTRL_RIP_BUSA_O) || pint) {
        reg[REG_UC] = 0;
    }

    /*
        The check of CBIT_IO_WAIT here prevents interrupts from being
        silently dropped during IO. (The CPU would issue AINT since
        the INSTMASK is up during an IO_WAIT, but unlatch the PINT
        line as the IO finishes, forgetting that it had come in in the
        first place and not running _DO_INT.)
    */
    if(state && !cbits[CBIT_IO_WAIT]) {
        pint_latch_val = pint;
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

void mod_ctl::clock_inputs(uinst_t ui, bus_state &s, pic_out_interface &pic) {
    // HARDWARE NOTE: interrupt_enable is simply AND-ed with the incoming PINT line.
    bool pint = pic.is_pint_active() && (pic.is_pnmi_active() || cbits[CBIT_IE]);

    if(!cbits[CBIT_IO_WAIT]) {
        // HARDWARE NOTE This register can be simultaneously reset under the GCTRL_FT_ENTER/MAYBEEXIT/EXIT conditions, but we
        // assume that (presumably async) reset signal dominates this increment.
        reg[REG_UC]++;
    }

    // HARDWARE NOTE: As per comment at definition, CBIT_IO_WAIT must be set AFTER it is checked to incrememnt REG_UC.
    switch(ui & MASK_CTRL_COMMAND) {
        case COMMAND_NONE:
        case COMMAND_RCTRL_RSP_INC: {
            break;
        }
        case COMMAND_IO_READ:
        case COMMAND_IO_WRITE: {
            cbits[CBIT_IO_WAIT] = true;
            break;
        }
        default: throw vm_error("unknown CTRL_COMMAND");
    }

    switch(ui & MASK_GCTRL_CREG) {
        case GCTRL_CREG_NONE: {
            break;
        }
        case GCTRL_CREG_P_IE: {
            cbits[CBIT_IE] = (ui & MASK_GCTRL_DIR) == GCTRL_CREG_I;
            break;
        }
        case GCTRL_CREG_FG:
        case GCTRL_CREG_IHPR: {
            if(GCTRL_CREG_IS_INPUT(ui)) {
                reg[GCTRL_DECODE_CREG(ui)] = s.read(BUS_B);
            }
            break;
        }
        default: throw vm_error("unknown GCTRL CREG");
    }

    switch(ui & MASK_GCTRL_FTJM) {
        case GCTRL_FT_NONE: {
            break;
        }
        case GCTRL_FT_ENTER: {
            set_instmask_enabled(ui, true, pint);
            break;
        }
        case GCTRL_FT_EXIT: {
            reg[REG_IP] += 2;
            set_instmask_enabled(ui, false, pint);
            break;
        }
        case GCTRL_FT_MAYBEEXIT: {
            reg[REG_IP] += 2;
            reg[REG_IR] = s.read(BUS_B);
            if(!(reg[REG_IR] & P_I_LOADDATA)) {
                set_instmask_enabled(ui, false, pint);
            }
            break;
        }
        case GCTRL_JM_YES: {
            reg[REG_IP] = s.read(BUS_B);
            set_instmask_enabled(ui, true, pint);
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
            set_instmask_enabled(ui, true, pint);
        }
    }
}

void mod_ctl::offclock_pulse(bool io_done) {
    if(io_done) {
        cbits[CBIT_IO_WAIT] = false;
    }

    // HARDWARE NOTE: note that io_done overrides CBIT_IO_WAIT here, and then immediately clears it.
    if(io_done || !cbits[CBIT_IO_WAIT]) {
        uinst_latch_val = arch::self().ucode_read(get_inst(), reg[REG_UC]);
        if(logger.dump_bus) {
            logger.logf("uinst latch <- 0x%X@0x%X\n", get_inst(), uinst_latch_val);
        }
    }
}

}