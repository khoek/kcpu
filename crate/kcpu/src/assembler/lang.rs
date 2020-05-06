use super::{
    defs,
    model::{Alias, Family},
};
use crate::common;
use crate::spec::{types::schema::ArgKind, ucode::UCode};
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

    fn arg_kind_lists_collide(us: &[ArgKind], vs: &[ArgKind]) -> bool {
        us.len() == vs.len() && us.iter().zip(vs.iter()).all(|(u, v)| u.collides(*v))
    }

    pub(super) fn register_family(&mut self, f: Family) {
        let arglists: Vec<_> = f
            .variants
            .iter()
            .map(|v| {
                self.lang
                    .lookup_alias(v)
                    .unwrap_or_else(|| panic!("Unknown alias: \"{}\"", v))
            })
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

#[cfg(test)]
mod tests {
    use super::Builder;
    use crate::spec::types::schema::{ArgKind, ConstPolicy, Half, Width};

    #[test]
    fn it_finds_empty_collision() {
        assert!(Builder::arg_kind_lists_collide(&[], &[]));
    }

    #[test]
    fn it_finds_word_collision() {
        assert!(Builder::arg_kind_lists_collide(
            &[ArgKind::new(Width::Word, ConstPolicy::Allow),],
            &[ArgKind::new(Width::Word, ConstPolicy::Allow),],
        ));
    }

    #[test]
    fn it_finds_byte_collision() {
        assert!(Builder::arg_kind_lists_collide(
            &[ArgKind::new(Width::Byte(Half::Lo), ConstPolicy::Never),],
            &[ArgKind::new(Width::Byte(Half::Lo), ConstPolicy::Never),],
        ));
    }

    #[test]
    fn it_finds_interesting_identical_collision_simple() {
        assert!(Builder::arg_kind_lists_collide(
            &[
                ArgKind::new(Width::Word, ConstPolicy::Allow),
                ArgKind::new(Width::Byte(Half::Lo), ConstPolicy::Never),
            ],
            &[
                ArgKind::new(Width::Word, ConstPolicy::Allow),
                ArgKind::new(Width::Byte(Half::Lo), ConstPolicy::Never),
            ],
        ));
    }

    #[test]
    fn it_finds_interesting_identical_collision() {
        assert!(Builder::arg_kind_lists_collide(
            &[
                ArgKind::new(Width::Word, ConstPolicy::Allow),
                ArgKind::new(Width::Byte(Half::Lo), ConstPolicy::Never),
                ArgKind::new(Width::Word, ConstPolicy::Only)
            ],
            &[
                ArgKind::new(Width::Word, ConstPolicy::Allow),
                ArgKind::new(Width::Byte(Half::Lo), ConstPolicy::Never),
                ArgKind::new(Width::Word, ConstPolicy::Only)
            ],
        ));
    }

    #[test]
    fn it_finds_different_lengths_no_collision_1() {
        assert!(!Builder::arg_kind_lists_collide(
            &[
                ArgKind::new(Width::Word, ConstPolicy::Allow),
                ArgKind::new(Width::Byte(Half::Lo), ConstPolicy::Never),
                ArgKind::new(Width::Word, ConstPolicy::Only)
            ],
            &[
                ArgKind::new(Width::Word, ConstPolicy::Allow),
                ArgKind::new(Width::Byte(Half::Lo), ConstPolicy::Never),
            ],
        ));
    }

    #[test]
    fn it_finds_different_lenths_no_collision_2() {
        assert!(!Builder::arg_kind_lists_collide(
            &[
                ArgKind::new(Width::Word, ConstPolicy::Allow),
                ArgKind::new(Width::Byte(Half::Lo), ConstPolicy::Never),
            ],
            &[
                ArgKind::new(Width::Word, ConstPolicy::Allow),
                ArgKind::new(Width::Byte(Half::Lo), ConstPolicy::Never),
                ArgKind::new(Width::Word, ConstPolicy::Only)
            ],
        ));
    }

    #[test]
    fn it_finds_different_width_no_collision() {
        assert!(!Builder::arg_kind_lists_collide(
            &[ArgKind::new(Width::Word, ConstPolicy::Allow),],
            &[ArgKind::new(Width::Byte(Half::Lo), ConstPolicy::Allow),],
        ));
    }

    #[test]
    fn it_finds_opposite_half_no_collision() {
        assert!(!Builder::arg_kind_lists_collide(
            &[ArgKind::new(Width::Byte(Half::Lo), ConstPolicy::Allow),],
            &[ArgKind::new(Width::Byte(Half::Hi), ConstPolicy::Allow),],
        ));
    }

    #[test]
    fn it_finds_superset_policy_collision_only() {
        assert!(Builder::arg_kind_lists_collide(
            &[ArgKind::new(Width::Word, ConstPolicy::Allow),],
            &[ArgKind::new(Width::Word, ConstPolicy::Only),],
        ));
    }

    #[test]
    fn it_finds_superset_policy_collision_never() {
        assert!(Builder::arg_kind_lists_collide(
            &[ArgKind::new(Width::Word, ConstPolicy::Allow),],
            &[ArgKind::new(Width::Word, ConstPolicy::Never),],
        ));
    }

    #[test]
    fn it_finds_exclusive_policy_no_collision() {
        assert!(!Builder::arg_kind_lists_collide(
            &[ArgKind::new(Width::Word, ConstPolicy::Never),],
            &[ArgKind::new(Width::Word, ConstPolicy::Only),],
        ));
    }
}
