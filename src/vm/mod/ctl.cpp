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
    logger.logf("RIR: %04X RFG: %04X\n", reg[REG_IR], get_reg_fg());
    logger.logf("CBITS: %c%c%c%c%c%c%c%c\n",
        cbits[CBIT_INSTMASK  ] ? 'M' : 'm',
        cbits[CBIT_IE        ] ? 'I' : 'i',
        cbits[CBIT_HNMI      ] ? 'N' : 'n',
        cbits[CBIT_IO_WAIT   ] ? 'W' : 'w',
        cbits[CBIT_HALTED    ] ? 'H' : 'h',
        cbits[CBIT_ABORTED   ] ? 'A' : 'a',
        cbits[CBIT_PINT_LATCH] ? 'P' : 'p',
        cbits[CBIT_INT_ENTER ] ? 'E' : 'e'
    );
}

// HARDWARE NOTE: Ensure that the INT instruction has SP in IU1!
#define LOAD_INSTVAL INST_MK(false, 0x0,      0, 0, 0)
#define INT_INSTVAL  INST_MK(false, 0x1, REG_SP, 0, 0)

regval_t mod_ctl::get_inst() {
    // HARDWARE NOTE: CBIT_IO_WAIT inhibits CBIT_INSTMASK, for obvious reasons,
    // EXCEPT WHEN IO_DONE IS ASSERTED, WHEN CBIT_INSTMASK BEHAVES AS NORMAL.
    // (This is the actual behaviour as emulated, see the hardware note in `offclock_pulse()`.)
    return (cbits[CBIT_INSTMASK] && !cbits[CBIT_IO_WAIT]) ? (cbits[CBIT_PINT_LATCH] ? INT_INSTVAL : LOAD_INSTVAL) : reg[REG_IR];
}

uinst_t mod_ctl::get_uinst() {
    return uinst_latch_val;
}

/*
    HARDWARE NOTE: the first boolean below is morally
    neccesary, but as of writing we never latch pint unless the
    INSTMASK is going high, and the pint latch is always cleared
    by the time INSTMASK is cleared.

    (So, at least right now, it can be safely commented.)
*/
bool mod_ctl::is_aint_active() {
    return /* cbits[CBIT_INSTMASK] && */ cbits[CBIT_INT_ENTER];
}

regval_t mod_ctl::get_reg_fg() {
    return (reg[REG_FG_RAW] & 0x00FF) | (cbits[CBIT_IE] ? FG_CBIT_IE : 0);
}

void mod_ctl::set_reg_fg_alu(regval_t val) {
    reg[REG_FG_RAW] = val & 0x00FF;

    vm_assert(!(val & ~0x00FF));
}

void mod_ctl::set_reg_fg_entire(regval_t val) {
    reg[REG_FG_RAW] = val & 0x00FF;
    cbits[CBIT_IE] = val & FG_CBIT_IE;

    vm_assert(!(val & ~0x01FF));
}

void mod_ctl::clock_outputs(uinst_t ui, bus_state &s) {
    if((ui & MASK_GCTRL_FTJM) == GCTRL_JM_P_RIP_BUSB_O) {
        s.assign(BUS_B, reg[REG_IP]);
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
        case ACTION_GCTRL_CREG_EN: {
            switch(ui & MASK_GCTRL_CREG) {
                case GCTRL_CREG_P_IE:
                case GCTRL_CREG_P_O_CHNMI_OR_I_ALUFG: {
                    break;
                }
                case GCTRL_CREG_FG: {
                    if(GCTRL_CREG_IS_OUTPUT(ui)) {
                        s.assign(BUS_B, get_reg_fg());
                    }
                    break;
                }
                case GCTRL_CREG_IHPR: {
                    if(GCTRL_CREG_IS_OUTPUT(ui)) {
                        s.assign(BUS_B, reg[REG_IHP]);
                    }
                    break;
                }
                default: throw vm_error("unknown GCTRL CREG");
            }
            break;
        }
        default: throw vm_error("unknown GCTRL_ACTION");
    }
}

void mod_ctl::set_instmask_enabled(uinst_t ui, bool state, bool pint, bool nmi) {
    /*
        HARDWARE NOTE: Remember, this function is "called" when
                        either the INTMASK_SET or INTMASK_CLEAR lines are
                        asserted.

    /*
        This prevents the JM_YES from taking effect halfway through
        the _DO_INT handler.

        HARDWARE NOTE: note that we do not just inhibit UC reset, we also do not raise any of the
                       CBITS here.
    */
    if((ui & MASK_CTRL_COMMAND) == COMMAND_RCTRL_RSP_EARLY_DEC_NOIM) {
        return;
    }

    /*
        This prevents UC getting reset when we execute the FT_ENTER
        in the first uinst of a NOP (in turn required to lift the INTMASK
        so that we do not leave the NOP as we load RIR), which would cause an
        infinite loop. The exception is when PINT is asserted, in which case we
        stutter for a uop (the first uop of NOP is wasted), and pass directly to
        the interrupt handling _DO_INT instruction next clock (and we need UC to
        be set back to 0 when this happens).
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
        cbits[CBIT_PINT_LATCH] = pint;
        cbits[CBIT_INT_ENTER ] = pint;
        /*
            HARDWARE NOTE: SERIOUS EASY MISTAKE POSSIBLE!
            The below is not(!!) the same as
                `cbits[CBIT_HNMI] = nmi && pint;`
            since we *NEED* CBIT_HNMI to stay high even as PINT_LATCH
            is reset low shortly (and INT_ENTER is cleared even sooner).

            CBIT_HNMI needs to stay high until an IRET.
        */
        if(nmi && pint) {
            cbits[CBIT_HNMI] = true;
        }
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
    if(reg[REG_UC] == 0x0) {
        cbits[CBIT_INT_ENTER] = false;
    }

    bool nmi = pic.is_pnmi_active();
    // HARDWARE NOTE: interrupt_enable is AND-ed with the incoming PINT line,
    // with IE overriden by the NMI line, and all of this is overriden by CBIT_HNMI!
    bool pint = pic.is_pint_active() && (nmi || cbits[CBIT_IE]) && !cbits[CBIT_HNMI];

    if(!cbits[CBIT_IO_WAIT]) {
        // HARDWARE NOTE This register can be simultaneously reset under the GCTRL_FT_ENTER/MAYBEEXIT/EXIT conditions, but we
        // assume that (presumably async) reset signal dominates this increment.
        reg[REG_UC]++;
    }

    // HARDWARE NOTE: As per comment at definition, CBIT_IO_WAIT must be set AFTER it is checked to incrememnt REG_UC.
    switch(ui & MASK_CTRL_COMMAND) {
        case COMMAND_NONE:
        case COMMAND_RCTRL_RSP_EARLY_DEC_NOIM:
        case COMMAND_RCTRL_RSP_EARLY_INC: {
            break;
        }
        case COMMAND_IO_READWRITE: {
            cbits[CBIT_IO_WAIT] = true;
            break;
        }
        default: throw vm_error("unknown CTRL_COMMAND");
    }

    switch(ui & MASK_CTRL_ACTION) {
        case ACTION_CTRL_NONE:
        case ACTION_MCTRL_BUSMODE_X:
        case ACTION_GCTRL_RIP_BUSA_O: {
            break;
        }
        case ACTION_GCTRL_CREG_EN: {
            switch(ui & MASK_GCTRL_CREG) {
                case GCTRL_CREG_P_IE: {
                    cbits[CBIT_IE] = (ui & MASK_GCTRL_DIR) == GCTRL_CREG_I;
                    break;
                }
                case GCTRL_CREG_P_O_CHNMI_OR_I_ALUFG: {
                    if((ui & MASK_GCTRL_DIR) == GCTRL_CREG_O) {
                        cbits[CBIT_HNMI] = false;
                    } else {
                        set_reg_fg_alu(s.read(BUS_B));
                    }
                    break;
                }
                case GCTRL_CREG_FG: {
                    if(GCTRL_CREG_IS_INPUT(ui)) {
                        set_reg_fg_entire(s.read(BUS_B));
                    }
                    break;
                }
                case GCTRL_CREG_IHPR: {
                    if(GCTRL_CREG_IS_INPUT(ui)) {
                        reg[REG_IHP] = s.read(BUS_B);
                    }
                    break;
                }
                default: throw vm_error("unknown GCTRL CREG");
            }
            break;
        }
        default: throw vm_error("unknown GCTRL_ACTION");
    }

    switch(ui & MASK_GCTRL_FTJM) {
        case GCTRL_FT_NONE: {
            break;
        }
        case GCTRL_FT_ENTER: {
            set_instmask_enabled(ui, true, pint, nmi);
            break;
        }
        case GCTRL_FT_EXIT: {
            reg[REG_IP] += 2;
            set_instmask_enabled(ui, false, pint, nmi);
            break;
        }
        case GCTRL_FT_MAYBEEXIT: {
            reg[REG_IP] += 2;
            reg[REG_IR] = s.read(BUS_B);
            if(!(reg[REG_IR] & P_I_LOADDATA)) {
                set_instmask_enabled(ui, false, pint, nmi);
            }
            break;
        }
        case GCTRL_JM_YES: {
            reg[REG_IP] = s.read(BUS_B);
            set_instmask_enabled(ui, true, pint, nmi);
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
            if((!!(get_reg_fg() /* only care about the low "ALU" bits of FG */ & decode_jcond_mask(ui))) ^ (!!(ui & GCTRL_JM_INVERTCOND))) {
                reg[REG_IP] = s.read(BUS_B);
            }
            set_instmask_enabled(ui, true, pint, nmi);
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
            logger.logf("uinst latch <- 0x%X@0x%lX\n", get_inst(), uinst_latch_val);
        }
    }
}

}