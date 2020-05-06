use super::defs;
use super::types::{
    hw::{self, Inst, PUAddr, UCVal, UInst, Word, IU},
    schema::InstDef,
};
use crate::common;
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

    pub fn inst_def_iter(&self) -> impl Iterator<Item = &InstDef> {
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

impl Inst {
    // Since we do not know if `IU3` is in use or not, we cannot reliably decode the `iu`s
    // without looking up the opcode as an `InstDef`.
    pub fn decode(inst: Word) -> Inst {
        let opcode = Inst::decode_opcode(inst);
        // RUSTFIX EVIL? encapsulation breaking
        let idef = common::unwrap_singleton(
            UCode::get()
                .inst_def_iter()
                .filter(|idef| idef.opclass.to_opcodes().any(|oc| oc == opcode)),
        );

        let (iu1, iu2, iu3) = IU::decode_all(inst);
        Inst::new(
            Inst::decode_load_data(inst),
            opcode,
            idef.args[IU::ONE].map(|_| iu1),
            idef.args[IU::TWO].map(|_| iu2),
            idef.args[IU::THREE].map(|_| iu3),
        )
    }
}
