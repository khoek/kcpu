use super::hw::{OpCode, PReg, UInst, Word, IU};
use derive_more::Constructor;
use enum_map::EnumMap;
use std::cmp::Ordering;
use strum::IntoEnumIterator;

// RUSTFIX make this a 4-bit type. use `typenum` crate to emulate const generics?
pub type Segment = Word;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iu3Prefix {
    // RUSTFIX make this a 1-bit type. use `typenum` crate to emulate const generics?
    val: Word,
}

impl Iu3Prefix {
    // RUSTFIX make this a 1-bit type. use `typenum` crate to emulate const generics?
    pub const fn new(val: Word) -> Iu3Prefix {
        Iu3Prefix { val }
    }

    /// Returns an incomplete piece of an opcode (the IKind part) obtained by binding this
    /// prefix to the passed IU3 value.
    pub fn bind(&self, iu3: PReg) -> OpCode {
        // RUSTFIX move this to the constructor once `if` is allowed in const
        assert!(self.val == 0 || self.val == 1);
        (self.val << IU::WIDTH) | (iu3 as OpCode)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IKind {
    NoIu3(Segment),
    Iu3(Iu3Prefix, PReg),
    AllIu3(Iu3Prefix),
}

//RUSTFIX privacy on all of this?
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpClass {
    flags: Word,
    itype: Segment,
    ikind: IKind,
}

impl OpClass {
    // RUSTFIX the shift need to get the `itype` in the right position in an `OpCode`. Hopefully remove later?
    pub const ITYPE_SHIFT: usize = 4;

    pub const fn new(itype: Segment, icode: Segment) -> OpClass {
        OpClass {
            flags: 0,
            itype,
            ikind: IKind::NoIu3(icode),
        }
    }

    #[allow(dead_code)]
    pub const fn with_iu3(
        itype: Segment,
        pfx: Word, /* RUSTFIX, just generic 1-bit type */
        reg: PReg,
    ) -> OpClass {
        OpClass {
            flags: 0,
            itype,
            ikind: IKind::Iu3(Iu3Prefix::new(pfx), reg),
        }
    }

    pub const fn with_iu3_all(
        itype: Segment,
        pfx: Word, /* RUSTFIX, just generic 1-bit type */
    ) -> OpClass {
        OpClass {
            flags: 0,
            itype,
            ikind: IKind::AllIu3(Iu3Prefix::new(pfx)),
        }
    }

    // RUSTFIX hackish, make this type safe using custom ITypes/IKinds as described in `defs::opclass`
    // RUSTFIX hack, this does nothing to prevent collisions.
    pub fn add_flag(&self, flag: Word) -> OpClass {
        OpClass {
            flags: self.flags | flag,
            ..*self
        }
    }

    /// Internal function for converting an `OpClass` into an `OpCode` given an `IU::THREE` `PReg` value.
    /// Returns `None` if the `OpClass` cannot bind (currently, this can only happen because an
    /// `IU:THREE` was passed when it shouldn't have been, or wasn't passed when it should have, or
    /// the `IKind` is `Iu3` and the wrong `PReg` value was passed).
    fn instantiate_with_iu3(&self, iu3: Option<PReg>) -> Option<OpCode> {
        let bound_ikind = match (self.ikind, iu3) {
            (IKind::NoIu3(kind), None) => Some(kind),
            (IKind::Iu3(kind, fixed), Some(reg)) => {
                if reg == fixed {
                    Some(kind.bind(reg))
                } else {
                    None
                }
            }
            (IKind::AllIu3(kind), Some(reg)) => Some(kind.bind(reg)),
            _ => None,
        }?;

        let itype = self.itype;
        let flags = self.flags;
        Some((itype << OpClass::ITYPE_SHIFT) | bound_ikind | flags)
    }

    pub fn instantiate(&self, regs: EnumMap<IU, Option<PReg>>) -> Option<OpCode> {
        self.instantiate_with_iu3(regs[IU::THREE])
    }

    pub fn to_opcodes<'a>(&'a self) -> impl Iterator<Item = OpCode> + 'a {
        let iu3s: Box<dyn Iterator<Item = Option<PReg>>> = match self.ikind {
            IKind::NoIu3(_) => Box::new(std::iter::once(None)),
            IKind::Iu3(_, reg) => Box::new(std::iter::once(Some(reg))),
            IKind::AllIu3(_) => Box::new(PReg::iter().map(Some)),
        };

        iu3s.map(move |reg| self.instantiate_with_iu3(reg).unwrap())
    }

    // RUSTFIX who is using this? Is it neccesary?
    pub fn supports(&self, iu: IU, kind: Option<ArgKind>) -> bool {
        match (iu, self.ikind) {
            (IU::THREE, IKind::NoIu3(_)) => kind.is_none(),
            (IU::THREE, IKind::Iu3(_, _)) => kind.is_some(),
            (IU::THREE, IKind::AllIu3(_)) => kind.is_some(),
            _ => true,
        }
    }

    pub fn is_compatible(&self, iu: IU, reg: Option<PReg>) -> bool {
        match iu {
            IU::THREE => self.instantiate_with_iu3(reg).is_some(),
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Half {
    Lo,
    Hi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Width {
    Byte(Half),
    Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstPolicy {
    Never,
    Only,
    Allow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Constructor)]
pub struct ArgKind {
    pub width: Width,
    pub policy: ConstPolicy,
}

#[derive(Debug, Clone)]
pub struct InstDef {
    pub name: String,
    pub opclass: OpClass,
    pub args: EnumMap<IU, Option<ArgKind>>,
    pub uis: Vec<UInst>,
}

impl Half {
    pub fn shift(&self) -> usize {
        match self {
            Half::Lo => 0,
            Half::Hi => 8,
        }
    }
}

impl PartialOrd for ConstPolicy {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        if self == rhs {
            return Some(Ordering::Equal);
        }

        match (self, rhs) {
            (ConstPolicy::Allow, _) => Some(Ordering::Greater),
            (_, ConstPolicy::Allow) => Some(Ordering::Less),
            (_, _) => None,
        }
    }
}

impl ConstPolicy {
    pub fn is_only(&self) -> bool {
        match self {
            ConstPolicy::Only => true,
            _ => false,
        }
    }
}

impl ArgKind {
    pub fn new_word(c: ConstPolicy) -> Self {
        Self::new(Width::Word, c)
    }

    pub fn new_byte(h: Half, c: ConstPolicy) -> Self {
        Self::new(Width::Byte(h), c)
    }
}

impl InstDef {
    fn any_arg_holes<T>(slots: EnumMap<IU, Option<T>>) -> bool {
        slots
            .iter()
            .fold(
                (false, false),
                |(seen_none, seen_some_after), (_, o)| match o {
                    None => (true, seen_some_after),
                    Some(_) => {
                        if seen_none {
                            (true, true)
                        } else {
                            (seen_none, seen_some_after)
                        }
                    }
                },
            )
            .1
    }

    pub fn new(
        name: &str,
        opclass: OpClass,
        arg1: Option<ArgKind>,
        arg2: Option<ArgKind>,
        arg3: Option<ArgKind>,
        uis: Vec<UInst>,
    ) -> Self {
        let mut args = EnumMap::new();
        args[IU::ONE] = arg1;
        args[IU::TWO] = arg2;
        args[IU::THREE] = arg3;

        assert!(!InstDef::any_arg_holes(args));
        assert!(args.iter().all(|(iu, kind)| opclass.supports(iu, *kind)));

        // Cannot declare multiple only-const arguments for the same instruction.
        assert!(
            args.iter()
                .map(|(_, kind)| kind.map(|kind| kind.policy.is_only()).unwrap_or(false) as usize)
                .sum::<usize>()
                <= 1
        );

        InstDef {
            name: name.to_owned(),
            opclass,
            args,
            uis,
        }
    }

    pub fn with_vec(name: &str, opclass: OpClass, arg_vec: Vec<ArgKind>, uis: Vec<UInst>) -> Self {
        Self::new(
            name,
            opclass,
            arg_vec.get(0).copied(),
            arg_vec.get(1).copied(),
            arg_vec.get(2).copied(),
            uis,
        )
    }

    pub fn with_0(name: &str, opclass: OpClass, uis: Vec<UInst>) -> Self {
        InstDef::new(name, opclass, None, None, None, uis)
    }

    pub fn with_1(name: &str, opclass: OpClass, arg1: ArgKind, uis: Vec<UInst>) -> Self {
        InstDef::new(name, opclass, Some(arg1), None, None, uis)
    }

    pub fn with_2(
        name: &str,
        opclass: OpClass,
        arg1: ArgKind,
        arg2: ArgKind,
        uis: Vec<UInst>,
    ) -> Self {
        InstDef::new(name, opclass, Some(arg1), Some(arg2), None, uis)
    }

    pub fn with_3(
        name: &str,
        opclass: OpClass,
        arg1: ArgKind,
        arg2: ArgKind,
        arg3: ArgKind,
        uis: Vec<UInst>,
    ) -> Self {
        InstDef::new(name, opclass, Some(arg1), Some(arg2), Some(arg3), uis)
    }

    pub fn with_single_0(name: &str, opclass: OpClass, ui: UInst) -> Self {
        InstDef::new(name, opclass, None, None, None, vec![ui])
    }

    pub fn with_single_1(name: &str, opclass: OpClass, arg1: ArgKind, ui: UInst) -> Self {
        InstDef::new(name, opclass, Some(arg1), None, None, vec![ui])
    }

    pub fn with_single_2(
        name: &str,
        opclass: OpClass,
        arg1: ArgKind,
        arg2: ArgKind,
        ui: UInst,
    ) -> Self {
        InstDef::new(name, opclass, Some(arg1), Some(arg2), None, vec![ui])
    }

    pub fn with_single_3(
        name: &str,
        opclass: OpClass,
        arg1: ArgKind,
        arg2: ArgKind,
        arg3: ArgKind,
        ui: UInst,
    ) -> Self {
        InstDef::new(name, opclass, Some(arg1), Some(arg2), Some(arg3), vec![ui])
    }
}
