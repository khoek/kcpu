use super::defs;
use super::types::{
    hw::{self, PUAddr, UCVal, UInst},
    schema::InstDef,
};
use once_cell::sync::Lazy;

static STORAGE: Lazy<UCode> = Lazy::new(UCode::new);

pub struct UCode {
    insts: Vec<InstDef>,
    data: [Option<UInst>; hw::UCODE_LEN],
}

impl UCode {
    fn new() -> Self {
        let mut builder = Builder::new();
        defs::inst::register(&mut builder);
        builder.build()
    }

    pub fn get() -> &'static UCode {
        Lazy::force(&STORAGE)
    }

    pub fn get_inst_defs(&self) -> impl Iterator<Item = &InstDef> {
        self.insts.iter()
    }

    pub fn read(&self, uaddr: PUAddr) -> UInst {
        self.data[usize::from(uaddr)].expect("executing undefined ucode instruction!")
    }
}

pub struct Builder {
    ucode: UCode,
}

impl Builder {
    fn new() -> Self {
        Builder {
            ucode: UCode {
                insts: Vec::new(),
                data: [None; hw::UCODE_LEN],
            },
        }
    }

    fn build(self) -> UCode {
        self.ucode
    }

    pub(super) fn register(&mut self, i: InstDef) {
        assert!(i.uis.len() <= hw::UCVAL_MAX + 1);

        let ui_count = i.uis.len() as UCVal;
        for oc in i.opclass.to_opcodes() {
            for uc in 0..ui_count {
                let loc = usize::from(PUAddr::new(oc, uc));
                assert!(self.ucode.data[loc].is_none());
                self.ucode.data[loc] = Some(i.uis[uc as usize]);
            }
        }

        self.ucode.insts.push(i);
    }
}
