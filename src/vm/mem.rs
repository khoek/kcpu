use super::types::*;
use crate::spec::{
    defs::usig,
    types::hw::{self, Bus, UInst, Word, BYTE_WIDTH},
};
use enum_map::{Enum, EnumMap};
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Enum, Clone, Copy)]
pub enum BankType {
    Bios,
    Prog,
}

impl BankType {
    // RUSTFIX make this const when const matches drop
    pub fn is_rom(self) -> bool {
        match self {
            BankType::Bios => true,
            BankType::Prog => false,
        }
    }

    // RUSTFIX make this const when const matches drop
    // Remember, this is in words!
    pub fn size(self) -> usize {
        match self {
            BankType::Bios => 1 << 13,
            BankType::Prog => 1 << 21, // FIXME what is the actual value?
        }
    }
}

pub struct Bank {
    typ: BankType,
    data: Vec<Word>,
}

impl Bank {
    pub fn new(typ: BankType, src: &[u8]) -> Self {
        if src.len() > 2 * typ.size() {
            panic!("overflow");
        }

        let mut data = vec![0; typ.size()];
        hw::bytes_to_words_into_buff(&mut data, src).expect("parity error");

        Self { typ, data }
    }

    fn load(&self, addr: Word) -> Word {
        if addr as usize >= self.data.len() {
            panic!("out of bounds memory load");
        }

        self.data[(addr >> 1) as usize]
    }

    fn store(&mut self, addr: Word, val: Word) {
        if addr as usize >= self.data.len() {
            panic!("out of bounds memory store");
        }

        if self.typ.is_rom() {
            panic!("cannot write to ROM!");
        }

        // HARDWARE NOTE: Note the division by 2 here.
        self.data[(addr >> 1) as usize] = val;
    }

    fn iter_at(&self, mut addr: Word) -> impl Iterator<Item = Word> + '_ {
        std::iter::from_fn(move || {
            let cur = self.load(addr);
            addr += 2;
            Some(cur)
        })
    }
}

pub struct Mem<'a> {
    log_level: &'a LogLevel,

    prefix: [Word; 2],
    fidd_adr: Word,
    fidd_val: Word,

    banks: EnumMap<BankType, Option<Bank>>,
}

impl<'a> Display for Mem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "LPFX: {:#06X} FPFX: {:#06X}",
            self.prefix[0], self.prefix[1]
        )?;
        write!(
            f,
            "FIDV: {:#06X} FIDA: {:#06X}",
            self.fidd_val, self.fidd_adr
        )?;

        Ok(())
    }
}

impl<'a> Mem<'a> {
    pub fn new(log_level: &'a LogLevel, bios_bin: &[u8], prog_bin: &[u8]) -> Self {
        let mut banks = EnumMap::new();
        banks[BankType::Bios] = Some(Bank::new(BankType::Bios, bios_bin));
        banks[BankType::Prog] = Some(Bank::new(BankType::Prog, prog_bin));
        Mem {
            log_level,
            prefix: [0, 0],
            fidd_adr: 0,
            fidd_val: 0,
            banks,
        }
    }

    // RUSTFIX remove this, also why is the shift 7 bits?
    const F_BANK_SELECT: Word = 1 << 7;

    fn selected_bank_type(&self, far: bool) -> BankType {
        if self.prefix[far as usize] & Mem::F_BANK_SELECT == 0 {
            BankType::Bios
        } else {
            BankType::Prog
        }
    }

    fn selected_bank(&self, far: bool) -> &Bank {
        (&self.banks[self.selected_bank_type(far)])
            .as_ref()
            .unwrap()
    }

    fn mut_selected_bank(&mut self, far: bool) -> &mut Bank {
        let typ = self.selected_bank_type(far);
        (&mut self.banks[typ]).as_mut().unwrap()
    }

    fn should_use_prefix_far(ui: UInst) -> bool {
        ui & usig::MCTRL_FLAG_MODE_N_FAR == 0
    }

    pub fn clock_outputs(&mut self, ui: UInst, s: &mut BusState) {
        if (ui & usig::MASK_MCTRL_BUSMODE) == usig::MCTRL_BUSMODE_DISABLE {
            return;
        }

        let use_far = Mem::should_use_prefix_far(ui);

        match ui & usig::MASK_MCTRL_MODE {
            usig::MCTRL_MODE_STPFX | usig::MCTRL_MODE_STPFX_FAR => (),
            usig::MCTRL_MODE_FO | usig::MCTRL_MODE_FO_MI | usig::MCTRL_MODE_FO_MI_FAR => {
                s.assign(Bus::F, self.fidd_val)
            }
            usig::MCTRL_MODE_FI | usig::MCTRL_MODE_FI_MO | usig::MCTRL_MODE_FI_MO_FAR => {
                // Note we are just doing "early" address latching,
                // with `fidd_val` to be updated at the normal time in the inputcall.
                self.fidd_adr = s.early_read(Bus::A);

                if (ui & usig::MASK_MCTRL_MODE) == usig::MCTRL_MODE_FI_MO
                    || (ui & usig::MASK_MCTRL_MODE) == usig::MCTRL_MODE_FI_MO_FAR
                {
                    if self.log_level.internals {
                        println!(
                            "  MB({}) -> {:#06X}@{:#06X}",
                            use_far,
                            self.fidd_adr,
                            self.selected_bank(use_far).load(self.fidd_adr)
                        );
                    }
                    s.assign(Bus::M, self.selected_bank(use_far).load(self.fidd_adr));
                }
            }
            _ => panic!("unknown memmode"),
        }
    }

    pub fn clock_connects(&self, ui: UInst, s: &mut BusState) {
        if (ui & usig::MASK_MCTRL_BUSMODE) == usig::MCTRL_BUSMODE_DISABLE {
            return;
        }

        // HARDWARE NOTE: remember this!!
        if (ui & usig::MASK_MCTRL_BUSMODE) == usig::MCTRL_BUSMODE_CONW_BUSB
            && ((ui & usig::MASK_MCTRL_MODE) == usig::MCTRL_MODE_STPFX
                || (ui & usig::MASK_MCTRL_MODE) == usig::MCTRL_MODE_STPFX_FAR)
        {
            return;
        }

        let bm_write = ui & usig::MCTRL_BUSMODE_WRITE != 0;
        let bm_x = (ui & usig::MASK_CTRL_ACTION) == usig::ACTION_MCTRL_BUSMODE_X;
        let low_bit_set = self.fidd_adr & 0x1 != 0;

        let connect_m_hi = low_bit_set != bm_write;
        let should_flip = low_bit_set != bm_x; // means "should flip" during usig::MCTRL_BUSMODE_CONW_BUSB_MAYBEFLIP
        let connect_b_lo = bm_write != bm_x;

        match ui & usig::MASK_MCTRL_BUSMODE {
            usig::MCTRL_BUSMODE_CONW_BUSM => {
                s.connect(Bus::F, Bus::M);
            }
            usig::MCTRL_BUSMODE_CONW_BUSB => {
                s.connect(Bus::F, Bus::B);
            }
            usig::MCTRL_BUSMODE_CONW_BUSB_MAYBEFLIP => {
                // Note that we do not need the flexibiltiy of s.connect()
                // here, since we only maybeflip during the second step of
                // a byte read, thus putting data onto Bus::B.
                //
                // That is, there is no reason this can't happen due to
                // ucode design, but we don't use it and don't support it
                // right now.
                if !should_flip {
                    s.assign(Bus::B, s.early_read(Bus::F));
                } else {
                    s.assign(Bus::B, hw::byte_flip(s.early_read(Bus::F)));
                }
            }
            _ => {
                // Similar to the previous, we only use this busmode to *load*
                // the fiddle register, hence our assumptions here are again
                // safe.
                let val_b = s.early_read(Bus::B);
                let val_m = s.early_read(Bus::M);

                let mut res = 0;

                if !connect_m_hi {
                    // M_LO_CONNECT
                    res |= val_m & 0x00FF;
                    if connect_b_lo {
                        // B_LO_TO_HI
                        res |= (val_b & 0x00FF) << BYTE_WIDTH;
                    } else {
                        // B_HI_TO_HI
                        res |= (val_b & 0xFF00) << 0;
                    }
                } else {
                    // M_HI_CONNECT
                    res |= val_m & 0xFF00;
                    if connect_b_lo {
                        // B_LO_TO_LO
                        res |= (val_b & 0x00FF) >> 0;
                    } else {
                        // B_HI_TO_LO
                        res |= (val_b & 0xFF00) >> BYTE_WIDTH;
                    }
                }
                s.assign(Bus::F, res);
            }
        }
    }

    pub fn clock_inputs(&mut self, ui: UInst, s: &BusState) {
        if (ui & usig::MASK_MCTRL_BUSMODE) == usig::MCTRL_BUSMODE_DISABLE {
            return;
        }

        let use_far = Mem::should_use_prefix_far(ui);

        match ui & usig::MASK_MCTRL_MODE {
            usig::MCTRL_MODE_STPFX => {
                self.prefix[0] = s.read(Bus::B);
            }
            usig::MCTRL_MODE_STPFX_FAR => {
                self.prefix[1] = s.read(Bus::B);
            }
            usig::MCTRL_MODE_FO => (),
            usig::MCTRL_MODE_FO_MI | usig::MCTRL_MODE_FO_MI_FAR => {
                if self.log_level.internals {
                    println!(
                        "  MB({}) <- {:#06X}@{:#06X}",
                        use_far,
                        self.fidd_adr,
                        s.read(Bus::M)
                    );
                }
                let adr = self.fidd_adr;
                self.mut_selected_bank(use_far).store(adr, s.read(Bus::M));
            }
            usig::MCTRL_MODE_FI | usig::MCTRL_MODE_FI_MO | usig::MCTRL_MODE_FI_MO_FAR => {
                // Note the address latching happens "early" in the outputcall,
                // so we are just left to update the actual value here.
                self.fidd_val = s.read(Bus::F);
            }
            _ => panic!("unknown memmode"),
        }
    }

    pub fn iter_at(&'a self, far: bool, addr: Word) -> impl Iterator<Item = Word> + 'a {
        self.selected_bank(far).iter_at(addr)
    }
}
