use enum_map::{Enum, EnumMap};

use super::interface;
use super::types::*;
use crate::spec::{defs::usig, types::hw::*, ucode};

#[derive(Clone, Copy, Enum)]
pub enum CBit {
    Halted,
    Aborted,
    /*
        When this bit is set, REG_RIR is disconnected from the ucode eeprom address bus,
        and some other instruction code source is used instead.

        (On a clock rising edge:)
        As CBit::Instmask is SET, the state of the PINT external signal line is latched,
        and the value of the low instruction bit (AFTER THE BEING SHIFTED PAST THE IUS)
        passed to the ucode is connected to the latched value. All other instruction bits
        are made zero. As a result, a NOP opcode (0x0) is read if there is no PINT, and
        _DO_INT opcode (0x1) is read if there is a PINT.

        While CBit::Instmask is high, we assert AINT with the latched value of PINT.

        CBit::Instmask can only be cleared by JM_EXIT or JM_MAYBEEXIT, so only when a NOP
        is executing.

    ****************
    HARDWARE NOTE: ACTUALLY, THIS HAS CHANGED A BIT, SEE IMPLEMENTATION of `set_instmask_state` (and the places where it is called)
                                                     FOR WHEN THE "Latch value" can be set (it's not really a latch anymore, I think?)
                                                    ESPECIALLY NOTE THE CONDITIONAL EXPRESSION IN THAT FUNCTION where the latch val is set
    *************


        If instead _DO_INT is executing due to the presence of the mask, then an ordinary
        JM_ENTER will be issued at the completion of the hardware interrupt handling.
        Consequently, we will attempt to set CBit::Instmask when it is already set. When
        this happens, the PINT external signal line will be relatched as normal, and
        there will have been enough time for the AINT issued earlier to have been noted
        by the PIC (so long as the ucode for _DO_INT is longer than 1 instruction,
        which it must be if we want to save RIP and then set its value to something
        else). Thus, PINT will already be low, and no further handling logic is requried.

        This brings us to the instruction loading ucode of the next NOP, and everything
        then works nicely.
    */
    Instmask,
    /*
        Interrupt enable.
    */
    Ie,
    /*
        Handling NMI. (Inhibits further NMIs.)
    */
    Hnmi,
    /*
        This bit is a bit tricky.
        It is set on CLK rising edge whenever IO_READ or IO_WRITE are asserted.
        It is cleared on CLK falling edge whenever IO_DONE is asserted.

        Moreover, it masks the UC increment and UC unlatching, subject to: if IO_DONE is asserted on a CLK
        falling edge (so that CBit::IoWait should be cleared) simultaneously the its UC unlatching-inhbit
        function does not occur (that is, IO_DONE hard overrides the UC unlatching-inhibit function of this bit,
        and clears the bit at the same time).

        On the other hand, the UC should inc once on a rising edge of CLK at which time IO_WAIT is simultaneously set.
    */
    IoWait,
    PintLatch,
    /*
        Is set whenever CBit::PintLatch is set, but is cleared when `regs[SReg::UC] == 0`.
        Thus, is only ever on for one clock cycle at a time.
    */
    IntEnter,
}

impl CBit {
    fn to_string(&self) -> &str {
        match self {
            CBit::Halted => "h",
            CBit::Aborted => "a",
            CBit::Instmask => "m",
            CBit::Ie => "i",
            CBit::Hnmi => "n",
            CBit::IoWait => "w",
            CBit::PintLatch => "p",
            CBit::IntEnter => "e",
        }
    }
}

// RUSTFIX
// #define FG_CBit_IE (1 << 8)

// RUSTFIX? (put this in the spec like before, or? I am actually relucant, since this is module private.) Acutally good idea?
#[derive(Enum)]
pub enum SReg {
    // First 0-1 are "c(ontrol)reg"s, remainder are private.
    // HARDWARE NOTE: the CREG-codes in the ucode depend
    //                on this order of the first 4.

    // HARDWARE NOTE: REG_FG has its low byte connected to the ALU,
    // and its high byte connected to CTL (currently, the latter is
    // to control CBit::Ie only). This means that the only the low byte
    // of the memory of REG_FG_RAW is actually ever nonzero while
    // we are simulating in the VM.
    RawFG = 0,
    IHP = 1,

    IP = 2,
    UC = 3,
    IR = 4,
    // HARDWARE NOTE: 3 unused registers
}

pub struct Ctl<'a> {
    logger: &'a Logger,
    uinst_latch_val: UInst,

    // FIXME it is unfortunate that these need to be public for the run_vm/simulation tools.
    // But it is nice to keep the member functions in this class only representative of
    // actual hardware functions. Somehow resolve this?
    // UPDATE THIS IS EASY TO FIX NOW, JUST EXPOSE SOME FUNCTIONS TO the `Instance`
    pub cbits: EnumMap<CBit, bool>,
    pub regs: EnumMap<SReg, Word>,
}

impl<'a> Ctl<'a> {
    pub fn new(logger: &'a Logger) -> Self {
        let mut cbits = EnumMap::new();
        // I think it is not neccesary to implement this on real hardware, so long as
        // all of the registers (in particular RIR) are initialized to zero. (Since then
        // NOP will be the first instruction executed.)
        cbits[CBit::Instmask] = true;

        Ctl {
            logger,
            uinst_latch_val: 0,
            regs: EnumMap::new(),
            cbits,
        }
    }

    // RUSTFIX find a nice way to remove this, probably after the whole ucode overhaul (remove in the same way)
    const FG_CBIT_IE: Word = 1 << 0;

    fn get_reg_fg(&self) -> Word {
        (self.regs[SReg::RawFG] & 0x00FF) | (self.cbits[CBit::Ie] as Word * Ctl::FG_CBIT_IE)
    }

    fn set_reg_fg_alu(&mut self, val: Word) {
        self.regs[SReg::RawFG] = val & 0x00FF;

        assert!(val & !0x00FF == 0);
    }

    fn set_reg_fg_entire(&mut self, val: Word) {
        self.regs[SReg::RawFG] = val & 0x00FF;
        self.cbits[CBit::Ie] = val & Ctl::FG_CBIT_IE != 0;

        assert!(val & !0x01FF == 0);
    }

    fn get_cbit_format(&self, b: CBit) -> String {
        let s = b.to_string();
        if self.cbits[b] {
            s.to_uppercase()
        } else {
            s.to_lowercase()
        }
    }

    // RUSTFIX remove this, just implement the display trait for a module, probably put this trait inside a `Module` trait which implements clock_out/inputs etc
    pub fn dump_registers(&self) {
        println!(
            "RIP: {:#04X} RUC: {:#04X}",
            self.regs[SReg::IP],
            self.regs[SReg::UC]
        );
        println!(
            "RIR: {:#04X} RFG: {:#04X}",
            self.regs[SReg::IR],
            self.get_reg_fg()
        );
        println!(
            "CBITS: {}{}{}{}{}{}{}{}",
            self.get_cbit_format(CBit::Instmask),
            self.get_cbit_format(CBit::Ie),
            self.get_cbit_format(CBit::Hnmi),
            self.get_cbit_format(CBit::IoWait),
            self.get_cbit_format(CBit::Halted),
            self.get_cbit_format(CBit::Aborted),
            self.get_cbit_format(CBit::PintLatch),
            self.get_cbit_format(CBit::IntEnter)
        );
    }

    pub fn get_uinst(&self) -> UInst {
        self.uinst_latch_val
    }

    pub fn clock_outputs(&self, ui: UInst, s: &mut BusState) {
        if ui & usig::MASK_GCTRL_FTJM == usig::GCTRL_JM_P_RIP_BUSB_O {
            s.assign(Bus::B, self.regs[SReg::IP]);
        }

        if ui & usig::MASK_CTRL_ACTION == usig::ACTION_GCTRL_USE_ALT {
            match ui & usig::MASK_GCTRL_MODE {
                usig::GCTRL_ALT_P_IE | usig::GCTRL_ALT_P_O_CHNMI_OR_I_ALUFG => (),
                usig::GCTRL_ALT_CREG_FG => {
                    if usig::gctrl_creg_is_output(ui) {
                        s.assign(Bus::B, self.get_reg_fg());
                    }
                }
                usig::GCTRL_ALT_CREG_IHPR => {
                    if usig::gctrl_creg_is_output(ui) {
                        s.assign(Bus::B, self.regs[SReg::IHP]);
                    }
                }
                _ => panic!("unknown GCTRL ALT mode"),
            }
        } else {
            match ui & usig::MASK_GCTRL_MODE {
                usig::GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED | usig::GCTRL_NRM_NONE => (),
                usig::GCTRL_NRM_IO_READWRITE => {
                    // The condition `is_gctrl_nrm_io_readwrite()` above actually exactly handles this case.
                }
                _ => panic!("unknown GCTRL NRM mode"),
            }
        }

        match ui & usig::MASK_CTRL_ACTION {
            usig::ACTION_CTRL_NONE | usig::ACTION_MCTRL_BUSMODE_X => (),
            usig::ACTION_GCTRL_USE_ALT => {
                // This is handled in the `if` above.
            }
            usig::ACTION_GCTRL_RIP_BUSA_O => {
                if !self.cbits[CBit::IoWait] {
                    s.assign(Bus::A, self.regs[SReg::IP]);
                }
            }
            _ => panic!("unknown usig::GCTRL_ACTION"),
        }
    }

    /*
        HARDWARE NOTE: Remember, `pint` and `nmi` here are not the direct lines, but have been
        passed through some logic in `clock_inputs`
    */
    fn set_instmask_enabled(&mut self, ui: UInst, state: bool, pint: bool, nmi: bool) {
        /*
            HARDWARE NOTE: Remember, this function is "called" when
                            either the INTMASK_SET or INTMASK_CLEAR lines are
                            asserted.

            This prevents the JM_YES from taking effect halfway through
            the _DO_INT handler.

            HARDWARE NOTE: note that we do not just inhibit UC reset, we also do not raise any of the
                           CBITS here.
        */
        if (ui & usig::MASK_CTRL_COMMAND) == usig::COMMAND_INHIBIT_JMFT {
            return;
        }

        /*
            This prevents UC getting reset when we execute the FT_ENTER
            in the first uinst of a NOP (in turn required to lift the INTMASK
            so that we do not leave the NOP as we load RIR), which would cause an
            infinite loop. The exception is when PINT is asserted, in which we
            stutter for a uop (the first uop of NOP is wasted), and pass directly to
            the interrupt handling _DO_INT instruction next clock (and we need UC to
            be set back to 0 when this happens).
        */
        if ((ui & usig::MASK_CTRL_ACTION) != usig::ACTION_GCTRL_RIP_BUSA_O) || pint {
            self.regs[SReg::UC] = 0;
        }

        /*
            The check of CBit::IoWait here prevents interrupts from being
            silently dropped during IO. (The CPU would issue AINT since
            the INSTMASK is up during an IO_WAIT, but unlatch the PINT
            line as the IO finishes, forgetting that it had come in in the
            first place and not running _DO_INT.)
        */
        if state && !self.cbits[CBit::IoWait] {
            self.cbits[CBit::PintLatch] = pint;
            self.cbits[CBit::IntEnter] = pint;
            /*
                HARDWARE NOTE: SERIOUS EASY MISTAKE POSSIBLE!
                The below is not(!!) the same as
                    `self.cbits[CBit::Hnmi] = nmi && pint;`
                since we *NEED* CBit::Hnmi to stay high even as PINT_LATCH
                is reset low shortly (and INT_ENTER is cleared even sooner).

                CBit::Hnmi needs to stay high until an IRET.
            */
            if nmi && pint {
                self.cbits[CBit::Hnmi] = true;
            }
        }

        self.cbits[CBit::Instmask] = state;
    }

    // RUSTFIX remove this from this file, unify the constants.
    fn decode_jcond_mask(ui: UInst) -> Word {
        match (ui & usig::MASK_GCTRL_FTJM) & !usig::GCTRL_JM_INVERTCOND {
            usig::GCTRL_JCOND_CARRY => (1 << 0),
            usig::GCTRL_JCOND_N_ZERO => (1 << 1),
            usig::GCTRL_JCOND_SIGN => (1 << 2),
            usig::GCTRL_JCOND_N_OVFLW => (1 << 3),
            _ => panic!("unknown JCOND!"),
        }
    }

    pub fn clock_inputs(&mut self, ui: UInst, s: &BusState, pic: &dyn interface::Pic) {
        if self.regs[SReg::UC] == 0x0 {
            self.cbits[CBit::IntEnter] = false;
        }

        let nmi: bool = pic.is_pnmi_active();
        // HARDWARE NOTE: interrupt_enable is AND-ed with the incoming PINT line,
        // with IE overriden by the NMI line, and all of this is overriden by CBit::Hnmi!
        let pint: bool =
            pic.is_pint_active() && (nmi || self.cbits[CBit::Ie]) && !self.cbits[CBit::Hnmi];

        if !self.cbits[CBit::IoWait] {
            // HARDWARE NOTE This register can be simultaneously reset under the usig::GCTRL_FT_ENTER/MAYBEEXIT/EXIT conditions, but we
            // assume that (presumably async) reset signal dominates this increment.
            self.regs[SReg::UC] += 1;
        }

        // HARDWARE NOTE: As per comment at definition, CBit::IoWait must be set AFTER it is checked to incrememnt REG_UC.
        if usig::is_gctrl_nrm_io_readwrite(ui) {
            self.cbits[CBit::IoWait] = true;
        }

        if (ui & usig::MASK_CTRL_ACTION) == usig::ACTION_GCTRL_USE_ALT {
            match ui & usig::MASK_GCTRL_MODE {
                usig::GCTRL_ALT_P_IE => {
                    self.cbits[CBit::Ie] = (ui & usig::MASK_GCTRL_DIR) == usig::GCTRL_CREG_I;
                }
                usig::GCTRL_ALT_P_O_CHNMI_OR_I_ALUFG => {
                    // RUSTFIX use matches of typesafe data to remove logic like this, which only checks for one case and has the other implicit.
                    if (ui & usig::MASK_GCTRL_DIR) == usig::GCTRL_CREG_O {
                        self.cbits[CBit::Hnmi] = false;
                    } else {
                        self.set_reg_fg_alu(s.read(Bus::B));
                    }
                }
                usig::GCTRL_ALT_CREG_FG => {
                    if usig::gctrl_creg_is_input(ui) {
                        self.set_reg_fg_entire(s.read(Bus::B));
                    }
                }
                usig::GCTRL_ALT_CREG_IHPR => {
                    if usig::gctrl_creg_is_input(ui) {
                        self.regs[SReg::IHP] = s.read(Bus::B);
                    }
                }
                _ => panic!("unknown GCTRL ALT mode"),
            }
        } else {
            match ui & usig::MASK_GCTRL_MODE {
                usig::GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED | usig::GCTRL_NRM_NONE => (),
                usig::GCTRL_NRM_IO_READWRITE => {
                    // The condition `is_gctrl_nrm_io_readwrite()` above actually exactly handles this case.
                }
                _ => panic!("unknown GCTRL NRM mode"),
            }
        }

        match ui & usig::MASK_GCTRL_FTJM {
            usig::GCTRL_FT_NONE => (),
            usig::GCTRL_FT_ENTER => {
                self.set_instmask_enabled(ui, true, pint, nmi);
            }
            usig::GCTRL_FT_EXIT => {
                self.regs[SReg::IP] += 2;
                self.set_instmask_enabled(ui, false, pint, nmi);
            }
            usig::GCTRL_FT_MAYBEEXIT => {
                self.regs[SReg::IP] += 2;
                self.regs[SReg::IR] = s.read(Bus::B);
                if self.regs[SReg::IR] & Inst::P_LOAD_DATA == 0 {
                    self.set_instmask_enabled(ui, false, pint, nmi);
                }
            }
            usig::GCTRL_JM_YES => {
                self.regs[SReg::IP] = s.read(Bus::B);
                self.set_instmask_enabled(ui, true, pint, nmi);
            }
            usig::GCTRL_JM_P_RIP_BUSB_O => {}
            usig::GCTRL_JM_HALT => {
                self.cbits[CBit::Halted] = true;
            }
            usig::GCTRL_JM_ABRT => {
                self.cbits[CBit::Halted] = true;
                self.cbits[CBit::Aborted] = true;
            }
            _ => {
                // It was one of the 8 JCOND codes
                if ((self.get_reg_fg() /* only care about the low "ALU" bits of FG */ & Ctl::decode_jcond_mask(ui))
                    != 0)
                    != (ui & usig::GCTRL_JM_INVERTCOND != 0)
                {
                    self.regs[SReg::IP] = s.read(Bus::B);
                }
                self.set_instmask_enabled(ui, true, pint, nmi);
            }
        }
    }

    pub fn offclock_pulse(&mut self, ioc: &dyn interface::Ioc) {
        let io_done = ioc.is_io_done();

        if io_done {
            self.cbits[CBit::IoWait] = false;
        }

        // HARDWARE NOTE: note that io_done overrides CBit::IoWait here, and then immediately clears it.
        if io_done || !self.cbits[CBit::IoWait] {
            if self.logger.dump_bus {
                print!("uinst latch <- {:#04X}", interface::Ctl::get_inst(self));
            }

            self.uinst_latch_val = ucode::UCode::get().read(PUAddr::new(
                Inst::decode(interface::Ctl::get_inst(self)).opcode,
                self.regs[SReg::UC] as UCVal,
            ));

            if self.logger.dump_bus {
                println!("@{:#04X}", self.uinst_latch_val);
            }
        }
    }

    // RUSTFIX make these two `Word`s once we can make `Inst::encode` const.

    // RUSTFIXFIXME use the NOP opcode and _DO_INT opcodes here.
    // We used to have to ensure that the _DO_INT instruction had SP in IU1---this is no longer the case.
    const LOAD_INSTVAL: Inst = Inst::new(false, 0x0, None, None, None);
    const INT_INSTVAL: Inst = Inst::new(false, 0x1, None, None, None);
}

impl<'a> interface::Ctl for Ctl<'a> {
    /*
        HARDWARE NOTE: the first boolean below is morally
        neccesary, but as of writing we never latch pint unless the
        INSTMASK is going high, and the pint latch is always cleared
        by the time INSTMASK is cleared.

        (So, at least right now, it can be safely commented.)
    */
    fn is_aint_active(&self) -> bool {
        /* self.cbits[CBit::Instmask] && */
        self.cbits[CBit::IntEnter]
    }

    /*
        "True uinstruction". High on the falling edge before
        a clock where a uinst which is part of a "true instruction",
        i.e. not an instruction fetch or interrupt handling.

        HARDWARE NOTE: This signal should only be inspected when
        the clock is going LOW.
    */
    fn is_tui_active(&self) -> bool {
        !self.cbits[CBit::Hnmi] && !self.cbits[CBit::Instmask]
    }

    fn get_inst(&self) -> Word {
        // HARDWARE NOTE: CBit::IoWait inhibits CBit::Instmask, for obvious reasons,
        // EXCEPT WHEN IO_DONE IS ASSERTED, WHEN CBit::Instmask BEHAVES AS NORMAL.
        // (This is the actual behaviour as emulated, see the hardware note in `offclock_pulse()`.)
        if self.cbits[CBit::Instmask] && !self.cbits[CBit::IoWait] {
            if self.cbits[CBit::PintLatch] {
                // RUSTFIX make this and the next const
                Ctl::INT_INSTVAL.encode()
            } else {
                Ctl::LOAD_INSTVAL.encode()
            }
        } else {
            self.regs[SReg::IR]
        }
    }
}
