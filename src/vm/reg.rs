use enum_map::EnumMap;
use std::{fmt::Display, num::Wrapping};

use super::{interface, types::*};
use crate::spec::{defs::usig, types::hw::*};

pub struct Reg<'a> {
    log_level: &'a LogLevel,
    regs: EnumMap<PReg, Word>,
}

impl<'a> Display for Reg<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "RID: {:#06X}", self.regs[PReg::ID])?;
        writeln!(
            f,
            "RA:  {:#06X} RB:  {:#06X}",
            self.regs[PReg::A],
            self.regs[PReg::B]
        )?;
        writeln!(
            f,
            "RC:  {:#06X} RD:  {:#06X}",
            self.regs[PReg::C],
            self.regs[PReg::D]
        )?;
        writeln!(
            f,
            "RSP: {:#06X} RBP: {:#06X}",
            self.regs[PReg::SP],
            self.regs[PReg::BP]
        )?;

        Ok(())
    }
}

impl<'a> Reg<'a> {
    pub fn new(log_level: &LogLevel) -> Reg {
        Reg {
            log_level,
            regs: EnumMap::new(),
        }
    }

    const fn should_perform_rsp_early_inc(ui: UInst) -> bool {
        ui & usig::MASK_CTRL_COMMAND == usig::COMMAND_RCTRL_RSP_EARLY_INC
    }

    const fn should_perform_rsp_early_dec(ui: UInst) -> bool {
        ui & usig::MASK_CTRL_COMMAND == usig::COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP
    }

    fn maybe_assign(&self, iunum: u8, s: &mut BusState, iu: u16, r: PReg) {
        if usig::rctrl_iu_is_en(iu) && usig::rctrl_iu_is_output(iu) {
            if self.log_level.internals {
                println!("  iu{}: {} <- r{}:", iunum, usig::rctrl_iu_to_bus(iu), r);
            }
            s.assign(usig::rctrl_iu_to_bus(iu), self.regs[r]);

            // NOTE Even if `r == REG_SP` we don't need to check for should_perform_rsp_inc/dec() here,
            // since there would be no timing problem (the DEC occurs on the offclock just before this clock).
        }
    }

    fn maybe_read(&mut self, iunum: u8, s: &BusState, iu: u16, r: PReg) {
        if usig::rctrl_iu_is_en(iu) && usig::rctrl_iu_is_input(iu) {
            if self.log_level.internals {
                println!("  iu{}: {} -> r{}:", iunum, usig::rctrl_iu_to_bus(iu), r);
            }
            self.regs[r] = s.read(usig::rctrl_iu_to_bus(iu));

            // NOTE Even if `r == REG_SP` we don't need to check for should_perform_rsp_inc/dec() here,
            // since there would be no timing problem (the DEC occurs on the offclock just before this clock).
        }
    }

    fn consider_iu3_override(ui: UInst, iu3: PReg) -> PReg {
        if usig::does_override_iu3_via_command(ui) || usig::does_override_iu3_via_gctrl_alt(ui) {
            return PReg::SP;
        }
        iu3
    }

    // RUSTFIX reenable, or just directly implement a real IU decoder simulator (quite easy)
    /*
        HARDWARE NOTE: This function emulates the much simpler hardware
        protection mechanism, which is just to inhbit the I line to each
        register (after passing through the IU machinery) if the O line
        is also asserted.
    */
    // fn filter_simultaneous_i_and_o(log_level: &vm_log_level, is: iu_state) -> iu_state {
    //     for(uint i = 0; i < 3; i++) {
    //         if(!rctrl_iu_is_en(is.dec[i])) {
    //             continue;
    //         }

    //         for(uint j = 0; j < 3; j++) {
    //             if(i == j || !rctrl_iu_is_en(is.dec[j])) {
    //                 continue;
    //             }

    //             if(is.iu[i] == is.iu[j]) {
    //                 bool has_i = usig::rctrl_iu_is_input(is.dec[i]) || usig::rctrl_iu_is_input(is.dec[j]);
    //                 bool has_o = usig::rctrl_iu_is_output(is.dec[i]) || usig::rctrl_iu_is_output(is.dec[j]);
    //                 if(has_i && has_o) {
    //                     if(log_level.dump_bus) println!("filtering simultaneous i:%d and o:%d", usig::rctrl_iu_is_input(is.dec[i]) ? i : j, usig::rctrl_iu_is_input(is.dec[i]) ? j : i);
    //                     is.dec[rctrl_iu_is_input(is.dec[i]) ? i : j] = 0;
    //                 }
    //             }
    //         }
    //     }
    //     return is;
    // }

    pub fn clock_outputs(&self, ui: UInst, s: &mut BusState, ctl: &dyn interface::Ctl) {
        // RUSTFIX remove this duplication
        let inst = ctl.inst();
        let (iu1, iu2, iu3) = IU::decode_all(inst);
        let iu3 = Reg::consider_iu3_override(ui, iu3);
        let (dec1, dec2, dec3) = (
            usig::rctrl_decode_iu1(ui),
            usig::rctrl_decode_iu2(ui),
            usig::rctrl_decode_iu3(ui),
        );

        // RUSTFIX re-enable
        // is = filter_simultaneous_i_and_o(log_level, is);

        self.maybe_assign(1, s, dec1, iu1);
        self.maybe_assign(2, s, dec2, iu2);
        self.maybe_assign(3, s, dec3, iu3);
    }

    pub fn clock_inputs(&mut self, ui: UInst, s: &BusState, ctl: &dyn interface::Ctl) {
        let inst = ctl.inst();
        let (iu1, iu2, iu3) = IU::decode_all(inst);
        let iu3 = Reg::consider_iu3_override(ui, iu3);
        let (dec1, dec2, dec3) = (
            usig::rctrl_decode_iu1(ui),
            usig::rctrl_decode_iu2(ui),
            usig::rctrl_decode_iu3(ui),
        );

        // RUSTFIX re-enable
        // is = filter_simultaneous_i_and_o(log_level, is);

        self.maybe_read(1, s, dec1, iu1);
        self.maybe_read(2, s, dec2, iu2);
        self.maybe_read(3, s, dec3, iu3);
    }

    /*
        HARDWARE NOTE: This RSP INC/DEC occurs on the falling edge of the clock, when
        the new ucode is about to be latched, but nonetheless depends on the NEW VALUE
        of `ui & usig::MASK_CTRL_COMMAND`. So, it peeks ahead in this way.
    */
    pub fn offclock_pulse(&mut self, ui: UInst) {
        assert!(!Reg::should_perform_rsp_early_inc(ui) || !Reg::should_perform_rsp_early_dec(ui));

        if Reg::should_perform_rsp_early_dec(ui) {
            self.regs[PReg::SP] = (Wrapping(self.regs[PReg::SP]) - Wrapping(2)).0;
        }

        if Reg::should_perform_rsp_early_inc(ui) {
            self.regs[PReg::SP] = (Wrapping(self.regs[PReg::SP]) + Wrapping(2)).0;
        }
    }
}
