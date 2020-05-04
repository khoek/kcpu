use crate::common;
use crate::spec::{
    types::{
        hw::{Byte, Inst, PReg, Word, IU},
        schema::{ArgKind, ConstPolicy, Half, InstDef, OpClass, Width},
    },
    ucode::UCode,
};
use derive_more::Constructor;
use enum_map::EnumMap;
use std::cmp;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use strum::IntoEnumIterator;

/*
    The architecture of the assembly language representation:

        There are two kinds of "argument" representations inside the assembler:

            1. `ArgKind`s: Specifies the *kinds* of arguments a particular alias ain a family accepts.
               Thus in particular, there isn't a notion of "byte argument" and "word argument" directly;
               instead the options are "word", "lobyte", or "hibyte", and each maps to a specific alias.
               This is just a more flexible way to do things in general, considering that we always have
               specific, distinct instructions for lo vs hi byte operations.

               That is, an `ArgKind` is a *width* (Word/LoByte/HiByte) and a *const mode* (None/Allow/Only).
               The latter explains whether the argument should always be a constant, can never be, or
               can be either constant or nonconstant.

            2. `Slot`s: RUSTFIX for `Virtual`s, explain: Can be: Const(value), Reg(PReg), Arg(idx)

        There are a few kinds of "instructions" which are represented inside the assembler:

            1. `InstDef`s: These are the structures declared in `crate::spec::defs::inst`, which
                are the true "instructions". They are very small programs (currently at most 4
                steps/`UInst`s), which are burned into the ucode ROMs.

                Nonetheless, the `InstDef`s carry more information than is needed to generate
                the ucode ROMs---in particular, they specify how many arguments and of which kind
                the instruction expects.

            2.  `Family`s: These each consist of a name and a map between various structures of
                acceptable types of passed arguments

                    e.g. `[RUSTFIX example of some ArgKinds]`

                and an `Alias` which supports this argument list.

            3.  `Alias`s: These each consist of a name, a list specifying their supported arguments
                and their types (`ArgKind`s), and a list of `Virtual`s which constitutes their body.

            4.  `Virtual`s: These each consist of an `OpClass` for a true `InstDef`, wrapped in a
                list of arguments for the `OpClass`.

        And, finally (a code generation intermediate):

            5.  `Blob`s: A "compiled" instruction code (an `Inst`)
                along with potential supporting data (i.e. a constant which must be loaded for
                the instruction). This constant data might not have yet been "resolved", e.g. label
                positions cannot be computed until all blobs are generated and their lengths are
                known, so `Blob`s take a generic type parameter.

                `Blob`s resolve into `Word`s representing the actual program binary.

        More on `Blob`s: when assembling, we'd like a way to resolve a list of actual arguments
        against a `Family` (and consequently, the underlying `Alias`es and and `Virtual`s). However,
        certain constants (coming from labels, but we implement a generic interface which could
        support e.g. relocation/linking) must be resolved late and depends on the number of words our
        big list of generated instructions take up. (e.g., instructions with constants cost double
        the number of words, etc. so this phase must be completred before the label positions
        can actually be determined---if we'd like labels to be able to be referenced before they
        they are defined).

        Thus our `Virtual`s each compile into `Blob` intermediates, for resolution into `Word`s to
        be performed subsequently.
*/

#[derive(Debug, Clone, Copy, Constructor)]
pub struct RegRef {
    preg: PReg,
    width: Width,
}

#[derive(Debug, Clone, Copy)]
pub enum Const {
    Byte(Byte, Half),
    Word(Word),
}

#[derive(Debug, Clone, Copy)]
pub enum ConstBinding<Tag> {
    Resolved(Const),
    Unresolved(Tag),
}

#[derive(Debug, Clone, Copy)]
pub enum Arg<Tag> {
    Const(ConstBinding<Tag>),
    Reg(RegRef),
}

#[derive(Debug, Constructor)]
pub struct Family {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Debug)]
pub struct Alias {
    pub name: String,
    arg_count: usize,
    vinsts: Vec<Virtual>,
}

pub type ArgIdx = u8;

#[derive(Debug, Clone, Copy)]
pub enum Slot {
    Const(Const),
    Reg(RegRef),
    Arg(ArgIdx),
}

#[derive(Debug, Constructor)]
pub struct Virtual {
    opclass: OpClass,
    args: EnumMap<IU, Option<Slot>>,
}

#[derive(Debug, Constructor)]
pub struct Blob<Tag> {
    inst: Inst,
    binding: Option<ConstBinding<Tag>>,
}

impl ConstPolicy {
    pub fn matches<Tag>(&self, arg: &Arg<Tag>) -> bool {
        match (self, arg) {
            (ConstPolicy::Only, Arg::Reg(_)) => false,
            (ConstPolicy::Never, Arg::Const(_)) => false,
            _ => true,
        }
    }
}

impl Const {
    pub fn to_width(&self) -> Width {
        match self {
            Const::Word(_) => Width::Word,
            Const::Byte(_, half) => Width::Byte(*half),
        }
    }

    pub fn encode(&self) -> Word {
        match self {
            Const::Word(val) => *val,
            Const::Byte(val, half) => (*val as Word) << half.shift(),
        }
    }
}

impl<T> ConstBinding<T> {
    pub fn to_width(&self) -> Width {
        match self {
            ConstBinding::Resolved(c) => c.to_width(),
            ConstBinding::Unresolved(_) => Width::Word,
        }
    }
}

impl ArgKind {
    pub fn collides(&self, other: &Self) -> bool {
        if self.width != other.width {
            return false;
        }

        if !(self.policy >= other.policy) && !(self.policy <= other.policy) {
            return false;
        }

        true
    }

    fn matches<Tag>(&self, arg: &Arg<Tag>) -> bool {
        self.policy.matches(arg) && self.width == arg.to_width()
    }
}

impl<Tag> Arg<Tag> {
    pub fn to_width(&self) -> Width {
        match self {
            Arg::Reg(r) => r.width,
            Arg::Const(cb) => cb.to_width(),
        }
    }

    pub fn to_preg(&self) -> PReg {
        match self {
            Arg::Reg(r) => r.preg,
            Arg::Const(_) => PReg::ID,
        }
    }
}

impl Family {
    pub fn with(name: &str, variants: Vec<&str>) -> Self {
        Self::new(
            name.to_owned(),
            variants.into_iter().map(ToOwned::to_owned).collect(),
        )
    }
}

impl From<InstDef> for Alias {
    fn from(idef: InstDef) -> Self {
        let mut slots = EnumMap::new();
        for iu in IU::iter() {
            slots[iu] = idef.args[iu].map(|_| Slot::Arg(iu as ArgIdx));
        }

        Alias::new(idef.name, vec![Virtual::new(idef.opclass, slots)])
    }
}

impl Alias {
    pub fn new(name: String, vinsts: Vec<Virtual>) -> Self {
        // Check that we can infer the type of `a`. This verifies
        // that the type of `a` "makes sense", in that the unbound
        // slots in the `Virtual` list are not contradictory in
        // type when referred to multiple times, and do not skip
        // indicies.
        let typ = Self::infer_type_from_virtuals(&vinsts);

        Self {
            name,
            arg_count: typ.len(),
            vinsts,
        }
    }

    pub fn with(name: &str, vinsts: Vec<Virtual>) -> Self {
        Self::new(name.to_owned(), vinsts)
    }

    pub fn with_single(name: &str, vinst: Virtual) -> Self {
        Self::new(name.to_owned(), vec![vinst])
    }

    fn infer_type_from_virtuals(vinsts: &Vec<Virtual>) -> Vec<ArgKind> {
        let mut max_idx = None;
        let mut idxs = HashMap::new();
        for vi in vinsts {
            // RUSTFIX EVIL? encapsulation breaking
            let idef = UCode::get()
                .get_inst_defs()
                .find(|idef| idef.opclass == vi.opclass)
                .unwrap();
            for iu in IU::iter() {
                // We can safely unwrap here because of the checking performed when the `InstDef` was created.
                if let Some(Slot::Arg(i)) = vi.args[iu] {
                    max_idx = match max_idx {
                        None => Some(i),
                        Some(max_idx) => Some(cmp::max(max_idx, i)),
                    };

                    match idxs.entry(i) {
                        Entry::Vacant(v) => {
                            v.insert(vec![idef.args[iu].unwrap()]);
                        }
                        Entry::Occupied(o) => o.into_mut().push(idef.args[iu].unwrap()),
                    }
                }
            }
        }

        // In particular, the first `unwrap` makes sure that there are no "holes" in the unbound arg indexes.
        let mut kinds = Vec::new();
        if let Some(max_idx) = max_idx {
            for i in 0..max_idx + 1 {
                let mut it = idxs.get(&i).unwrap().iter();
                let first = it.next().unwrap();
                assert!(it.all(|typ| typ == first));
                kinds.push(*first);
            }
        }

        kinds
    }

    pub fn infer_type(&self) -> Vec<ArgKind> {
        Self::infer_type_from_virtuals(&self.vinsts)
    }

    // RUSTFIX this function re-infers the type, do we reall need it?
    // pub fn matches<T>(&self, args: &Vec<Arg<Tag>>) -> bool {
    //     iproduct!(self.infer_type().iter(), args).all(|(kind, arg)| kind.matches(arg))
    // }

    pub fn instantiate<Tag: Clone>(&self, args: &Vec<Arg<Tag>>) -> Option<Vec<Blob<Tag>>> {
        // We need to check the argument list length against our internally stored argument count,
        // since `Virtual::instantiate` panics when the argument list is too short, and we could
        // erroneously match argument lists which are too long.
        if self.arg_count == args.len() {
            self.vinsts.iter().map(|vi| vi.instantiate(args)).collect()
        } else {
            None
        }
    }
}

impl Virtual {
    pub fn with_slots(
        opclass: OpClass,
        iu1: Option<Slot>,
        iu2: Option<Slot>,
        iu3: Option<Slot>,
    ) -> Self {
        let mut args = EnumMap::new();
        args[IU::ONE] = iu1;
        args[IU::TWO] = iu2;
        args[IU::THREE] = iu3;

        assert!(UCode::get()
            .get_inst_defs()
            .find(|inst| inst.opclass == opclass)
            .unwrap()
            .bound_slots_match(args));

        Virtual { opclass, args }
    }

    pub fn with_0(opclass: OpClass) -> Self {
        Virtual::with_slots(opclass, None, None, None)
    }

    pub fn with_1(opclass: OpClass, iu1: Slot) -> Self {
        Virtual::with_slots(opclass, Some(iu1), None, None)
    }

    pub fn with_2(opclass: OpClass, iu1: Slot, iu2: Slot) -> Self {
        Virtual::with_slots(opclass, Some(iu1), Some(iu2), None)
    }

    pub fn with_3(opclass: OpClass, iu1: Slot, iu2: Slot, iu3: Slot) -> Self {
        Virtual::with_slots(opclass, Some(iu1), Some(iu2), Some(iu3))
    }

    /// Turn this `Virtual` instruction into a `Blob` by binding against the passed
    /// argument list (against which we resolve our unbound `Slot`s). Returns `None`
    /// if there is a type mismatch between the argument list and the instruction
    /// represented by this virtual instruction. Panics if the argument list is not
    /// long enough to resolve a bound variable.
    pub fn instantiate<Tag: Clone>(&self, args: &Vec<Arg<Tag>>) -> Option<Blob<Tag>> {
        // RUSTFIX we want to be performing runtime checks with `opclass.is_compatible` in this method
        //         oop, actually, if an opcode has a "bind" instruction where it binds an EnumMap of ius,
        //         then we could just see that that failed.

        // We also want to check that we aren't trying to bind multiple constants to the same uinst.
        //
        // Because we use type inference, this could arise in practice even with compile-time safeguards if there are just
        // two arguments to an alias either of which could be a constant, but if they both are would cause a clash
        // in a single instruction.
        let mut maybe_cb: Option<ConstBinding<Tag>> = None;
        let mut ius: EnumMap<IU, Option<PReg>> = EnumMap::new();
        for (iu, slot) in self.args {
            match slot {
                None => (),
                Some(slot) => {
                    let arg = slot.bind(args);

                    ius[iu] = Some(arg.to_preg());

                    if let Arg::Const(c) = arg {
                        assert!(maybe_cb.is_none());
                        maybe_cb = Some(c);
                    }
                }
            }
        }

        let idef = common::unwrap_singleton(
            &mut UCode::get()
                .get_inst_defs()
                .filter(|inst| inst.opclass == self.opclass),
        );

        Some(Blob::new(
            idef.instantiate(maybe_cb.is_some(), ius)?,
            maybe_cb,
        ))
    }
}

impl InstDef {
    // RUSTFIX Is this function for compile time checking?
    fn bound_slots_match(&self, slots: EnumMap<IU, Option<Slot>>) -> bool {
        // Check whether all of the slots which are not an unbound `ArgIdx` actually match the
        // corresponding argument of the instruction which they claim to.
        //
        // In particular, we make sure that the used slots exactly correspond to occupied
        // `ArgKind`s in the `InstDef`.
        if !slots.iter().all(|(iu, slot)| {
            // If `Slot::bound_to_arg` gives `None` then this just means we can't decide
            // wether the argument typechecks at startup-time; the argument is unbound.
            // On the other hand, if there is no argument at all then we can still
            // decided whether we should actually have one there, and vice versa if we
            // are trying to bind too many arguments.
            match (self.args[iu], slot.map(|s| s.bound_to_arg::<()>())) {
                (None, None) => true,
                (Some(_), Some(None)) => true,
                (Some(a), Some(Some(b))) => a.matches(&b),
                _ => false,
            }
        }) {
            return false;
        }

        // Also, we want to check that if this slot tries to bind an IU3 and the opclass
        // supports only a single IU3 then the `PReg` values match. We do this using the
        // generic `OpClass::is_compatible` functionality, which probes `IU`s one-by-one,
        // since we don't know about the IUs corresponding to the unbound `Slot`s right
        // now (it will depend on how this `InstDef` is instantiated), and need to
        // distinguish not binding the IU3 at all (which e.g. might be prohibited by the
        // opclass) compared to just not knowing whether we will bind it right now.
        //
        // This all leaves room for extensibility in terms of the IU interface---despite
        // this comment, the code in this file doesn't actually know IU3 is special at
        // all.
        for (iu, slot) in slots {
            if let Some(reg) = slot
                .map(|slot| slot.bound_to_arg::<()>().map(|arg| arg.to_preg()))
                .flatten()
            {
                if !self.opclass.is_compatible(iu, Some(reg)) {
                    return false;
                }
            }
        }

        true
    }

    fn instantiate(&self, load_data: bool, args: EnumMap<IU, Option<PReg>>) -> Option<Inst> {
        // RUSTFIX type checking!

        Some(Inst::new(
            load_data,
            self.opclass.instantiate(args)?,
            args[IU::ONE],
            args[IU::TWO],
            args[IU::THREE],
        ))
    }
}

impl Slot {
    pub fn with_wreg(reg: PReg) -> Self {
        Slot::Reg(RegRef::new(reg, Width::Word))
    }

    pub fn with_breg(reg: PReg, half: Half) -> Self {
        Slot::Reg(RegRef::new(reg, Width::Byte(half)))
    }

    pub fn with_wconst(val: Word) -> Self {
        Slot::Const(Const::Word(val))
    }

    pub fn with_bconst(val: Byte, half: Half) -> Self {
        Slot::Const(Const::Byte(val, half))
    }

    pub fn with_arg(idx: ArgIdx) -> Self {
        Slot::Arg(idx)
    }

    /// Weaker version of `bind` which returns `Some(arg)`
    /// exactly when it is not an unbound argument already---i.e. a
    /// `Reg` or `Const`.
    pub fn bound_to_arg<Tag>(&self) -> Option<Arg<Tag>> {
        match self {
            Slot::Reg(r) => Some(Arg::Reg(*r)),
            Slot::Const(c) => Some(Arg::Const(ConstBinding::Resolved(*c))),
            _ => None,
        }
    }

    /// Binds the passed argument list (i.e resolving unbound arguments
    /// against the passed list), returning `None` only if the argument
    /// list was not long enough.
    pub fn bind<Tag: Clone>(&self, args: &Vec<Arg<Tag>>) -> Arg<Tag> {
        match self {
            Slot::Arg(idx) => (*args.get(*idx as usize).unwrap()).clone(),
            _ => self.bound_to_arg().unwrap(),
        }
    }
}

impl<Tag> Blob<Tag> {
    pub fn words(&self) -> usize {
        match self.binding {
            None => 1,
            Some(_) => 2,
        }
    }

    pub fn resolve<Err, F: Fn(Tag) -> Result<Word, Err>>(
        self,
        resolver: F,
    ) -> Result<Vec<Word>, Err> {
        let inst = self.inst.encode();
        let extra = self
            .binding
            .map(|bi| match bi {
                ConstBinding::Resolved(c) => Ok(c.encode()),
                ConstBinding::Unresolved(tag) => resolver(tag),
            })
            .transpose()?;

        Ok(match extra {
            None => vec![inst],
            Some(extra) => vec![inst, extra],
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // RUSTFIX as part of the internal test suite here, check all the "matching" interactions
    // between the various argument type representations and their associated functions.
    //
    // For example:

    #[test]
    fn const_policy_partial_order() {
        assert!(ConstPolicy::Allow >= ConstPolicy::Only);

        todo!();
    }
}