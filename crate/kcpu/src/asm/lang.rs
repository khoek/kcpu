use super::{
    defs,
    model::{Alias, Family},
};
use crate::common;
use crate::spec::{types::schema::ArgKind, ucode::UCode};
use itertools::iproduct;
use once_cell::sync::Lazy;
use std::collections::HashMap;

static STORAGE: Lazy<Lang> = Lazy::new(Lang::new);

pub struct Lang {
    aliases: HashMap<String, Alias>,
    families: HashMap<String, Family>,
}

impl Lang {
    fn sanitize_name(name: &str) -> String {
        name.to_lowercase()
    }

    fn new() -> Self {
        let mut builder = Builder::new();
        defs::alias::register(&mut builder);
        defs::family::register(&mut builder);
        builder.build()
    }

    pub fn get() -> &'static Lang {
        Lazy::force(&STORAGE)
    }

    pub fn lookup_alias(&self, name: &str) -> Option<&Alias> {
        self.aliases.get(&Self::sanitize_name(name))
    }

    pub fn lookup_family(&self, name: &str) -> Option<&Family> {
        self.families.get(&Self::sanitize_name(name))
    }
}

pub struct Builder {
    lang: Lang,
}

impl Builder {
    fn new() -> Self {
        let mut builder = Builder {
            lang: Lang {
                aliases: HashMap::new(),
                families: HashMap::new(),
            },
        };

        // RUSTFIX EVIL? breaking out of encapsulation
        for idef in UCode::get().get_inst_defs() {
            builder.register_alias(Alias::from((*idef).clone()));
        }

        builder
    }

    fn build(self) -> Lang {
        self.lang
    }

    pub(super) fn register_alias(&mut self, a: Alias) {
        let name = Lang::sanitize_name(&a.name);

        assert!(self.lang.aliases.insert(name.clone(), a).is_none());

        self.register_family(Family::new(name.clone(), vec![name]))
    }

    fn arg_kind_lists_collide(us: &Vec<ArgKind>, vs: &Vec<ArgKind>) -> bool {
        us.len() == vs.len() && iproduct!(us.iter(), vs.iter()).all(|(u, v)| u.collides(v))
    }

    pub(super) fn register_family(&mut self, f: Family) {
        let arglists = f
            .variants
            .iter()
            .map(|v| self.lang.lookup_alias(v).expect(&format!("Unknown alias: \"{}\"", v)))
            .map(Alias::infer_type)
            .collect();

        assert!(common::vec_pairwise_iter(&arglists)
            .all(|(a, b)| !Builder::arg_kind_lists_collide(a, b)));

        // RUSTFIX use expect_none once it stabilises
        assert!(self
            .lang
            .families
            .insert(Lang::sanitize_name(&f.name), f)
            .is_none());
    }
}
