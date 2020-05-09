use super::{lang::Lang, model::RegRef};
use crate::assembler::model::{
    Alias, Arg, Const, ConstBinding, Family, ResolvedArg, ResolvedBlob, Slot,
};
use crate::{
    common,
    spec::{
        types::{
            hw::{Inst, PReg, Word, IU},
            schema::{ConstPolicy, InstDef},
        },
        ucode::UCode,
    },
};
use enum_map::EnumMap;
use std::{cmp::Ordering, collections::VecDeque, fmt::Display};

#[derive(Debug)]
pub enum Error {
    InvalidOpcode(Word),
    UnexpectedEndOfStream,
    NoSuitableAlias(Vec<String>, Vec<String>),
    CouldNotResolveAliasArgs(String, usize),
    AmbiguousAliasSpecificity(String, String),
}

impl<'a> Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidOpcode(raw) => write!(f, "Invalid Opcode: {:#06X}", raw),
            Error::UnexpectedEndOfStream => write!(f, "Unexpectedly encountered end of stream"),
            // RUSTFIX nice messages for these next two:
            Error::CouldNotResolveAliasArgs(alias, args) => write!(
                f,
                "Could not resolve alias '{}' arguments: {:#?}",
                alias, args
            ),
            Error::NoSuitableAlias(blobs, former_candidates) => write!(
                f,
                "No suitable alias for blobs: {:#?}\nformer candidates were: {:#?}",
                blobs, former_candidates
            ),
            Error::AmbiguousAliasSpecificity(a1, a2) => write!(
                f,
                "Aliases '{}' and '{}' are incomparable w.r.t. the specificity partial order",
                a1, a2,
            ),
        }
    }
}

pub fn next_resolved_blob(insts: &mut impl Iterator<Item = Word>) -> Result<ResolvedBlob, Error> {
    let raw_inst = insts.next().ok_or(Error::UnexpectedEndOfStream)?;

    let inst = Inst::decode(raw_inst);
    let (raw_data, data) = if inst.load_data {
        let raw_data = insts.next().ok_or(Error::UnexpectedEndOfStream)?;
        (
            Some(raw_data),
            Some(ConstBinding::Resolved(Const::Word(raw_data))),
        )
    } else {
        (None, None)
    };

    let blob = ResolvedBlob::new(inst, data);

    {
        // BUGCHECK Consider making this whole thing a debug assertion.
        assert_eq!(
            blob.to_words(),
            std::iter::once(raw_inst)
                .chain(raw_data.into_iter())
                .collect::<Vec<_>>()
        );
    }

    Ok(blob)
}

#[derive(Debug, Clone)]
pub struct DisassembledBlob<'a> {
    pub blob: ResolvedBlob,
    pub idef: &'a InstDef,
    pub args: EnumMap<IU, Option<ResolvedArg>>,
}

impl<'a> Display for DisassembledBlob<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.idef.name)?;
        for (_, arg) in &self.args {
            if let Some(arg) = arg {
                write!(f, " {}", arg)?;
            }
        }
        Ok(())
    }
}

// Does not consume the iterator, since it knows precisely how much it needs to consume (and will error if it finds something unexpected)
pub fn disassemble_blob<'a>(
    insts: &mut impl Iterator<Item = Word>,
) -> Result<DisassembledBlob<'a>, Error> {
    let blob = next_resolved_blob(insts)?;
    // FIXME EVIL? encapsulation breaking (pass this data in, and all families and aliases have an attached marker to it?)
    let idef = UCode::get()
        .inst_def_iter()
        .find(|idef| idef.opclass.to_opcodes().any(|oc| oc == blob.inst.opcode))
        .ok_or(Error::InvalidOpcode(blob.inst.opcode))?;

    let mut args = EnumMap::new();
    for (iu, kind) in idef.args {
        args[iu] = kind.map(|kind| Arg::disassemble(&blob, iu, kind));
    }

    Ok(DisassembledBlob { blob, idef, args })
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisassembledAlias<'a> {
    pub alias: &'a Alias,
    pub args: Vec<ResolvedArg>,
}

impl<'a> Display for DisassembledAlias<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_partial())
    }
}

impl<'a> DisassembledAlias<'a> {
    fn to_partial(&self) -> PartialDisassembledAlias<'a> {
        PartialDisassembledAlias {
            alias: self.alias,
            args: self.args.iter().cloned().map(Some).collect(),
        }
    }
}

impl<'a> DisassembledAlias<'a> {
    fn specificity_score(&self) -> (usize, isize, bool) {
        (
            self.alias.vinsts.len(),
            -(self.alias.arg_count as isize),
            self.alias.from_idef,
        )
    }
}

/// Partial ordering on `Alias`es using `specificity_score()`, designed to find
/// aliases which are more specific in their semantics than others. We compare
/// `specificity_score()`s and return if they are not equal, otherwise we `None`
/// unless we literally have `self == other`.
impl<'a> PartialOrd for DisassembledAlias<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.specificity_score().cmp(&other.specificity_score()) {
            Ordering::Equal if self == other => Some(Ordering::Equal),
            Ordering::Equal => None,
            ord => Some(ord),
        }
    }
}

fn fmt_option_arg(f: &mut std::fmt::Formatter<'_>, arg: Option<&ResolvedArg>) -> std::fmt::Result {
    match arg {
        Some(arg) => write!(f, " {}", arg),
        None => write!(f, " <unresolved>"),
    }
}

#[derive(Debug, Clone)]
struct PartialDisassembledAlias<'a> {
    alias: &'a Alias,
    args: Vec<Option<ResolvedArg>>,
}

impl<'a> Display for PartialDisassembledAlias<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.alias.name)?;
        for arg in &self.args {
            fmt_option_arg(f, arg.as_ref())?;
        }
        Ok(())
    }
}

impl<'a> PartialDisassembledAlias<'a> {
    fn new(alias: &'a Alias) -> Self {
        Self {
            alias,
            args: vec![None; alias.arg_count],
        }
    }

    /// Reconstructs some of its arguments using the passed index in `self.alias` for a `Virtual`
    /// and a `DisassembledBlob`, with the latter supposedly obtained by instantiating the former.
    ///
    /// Destroys itself if it finds a contradiction, so reconstruction is not possible (it doesn't
    /// even make sense).
    fn reconstruct_args(mut self, idx: usize, blob: &DisassembledBlob) -> Option<Self> {
        let virt = match self.alias.vinsts.get(idx) {
            Some(virt) => virt,
            None => return None,
        };

        if virt.opclass != blob.idef.opclass {
            return None;
        }

        for (iu, slot) in virt.slots {
            match (&blob.args[iu], slot) {
                (Some(arg), Some(Slot::Arg(idx))) => match &self.args[idx] {
                    Some(blob_arg) if arg != blob_arg => return None,
                    Some(_) => (),
                    None => {
                        // Instructions like `MOV $0xBEEF %rid` will trip us up here, since we
                        // could naively bind the constant `$0xBEEF` to both arguments of the
                        // `MOV`. Instead we use the `ConstPolicy` of the `InstDef` for `virt`.

                        // RUSTFIX EVIL? encapsulation breaking
                        let kind = common::unwrap_singleton(
                            UCode::get()
                                .inst_def_iter()
                                .filter(|idef| idef.opclass == virt.opclass),
                        )
                        .args[iu]
                            .unwrap();

                        let correct_arg = if kind.matches(arg) {
                            (*arg).clone()
                        } else {
                            match (arg, kind.policy) {
                                (Arg::Const(cb), ConstPolicy::Never) => {
                                    // As warned against above, a constant has been bound to a non-const
                                    // argument: we infer that the argument was actually a reference to
                                    // `%rid` instead.
                                    Arg::Reg(RegRef::new(PReg::ID, cb.to_width()))
                                },
                                _ => panic!("Argument in correct slot violates ArgKind declaration in corresponding InstDef"),
                            }
                        };

                        self.args[idx] = Some(correct_arg);
                    }
                },
                // If the slot is not `Slot::Arg` then it is bound, so it is
                // safe to `unwrap()` after `Slot::to_arg()`.
                (Some(arg), Some(slot)) => {
                    if slot.to_arg().as_ref().unwrap() != arg {
                        return None;
                    }
                }
                (None, None) => (),
                _ => panic!(
                    "argument disagree for same opcode: '{:?}' vs '{:?}'",
                    slot, blob.args[iu]
                ),
            }
        }

        Some(self)
    }

    fn into_complete(self) -> Result<DisassembledAlias<'a>, Error> {
        let fmt = self.to_string();
        let args = self
            .args
            .into_iter()
            .enumerate()
            .map(|(i, arg)| arg.ok_or_else(|| Error::CouldNotResolveAliasArgs(fmt.clone(), i)))
            .collect::<Result<Vec<_>, _>>()?;

        {
            // BUGCHECK Consider making this whole thing a debug assertion.
            if args.iter().filter(|arg| arg.is_const()).count() > 1 {
                panic!("Attempting to complete partial alias '{}' which has multiple constant arguments", fmt);
            }
        }

        Ok(DisassembledAlias {
            alias: self.alias,
            args,
        })
    }
}

/// Returns the number of `PartialDisassembledAlias`es which have not yet been ruled out nor matched.
fn resolve_partials_against_blob<'a>(
    candidates: &mut [Option<PartialDisassembledAlias<'a>>],
    mut matches: Option<&mut Vec<DisassembledAlias<'a>>>,
    idx: usize,
    blob: &DisassembledBlob,
) -> Result<bool, Error> {
    // RUSTFIX change `candidates` to use a `LinkedList` with `drain_filter` when it arrives.
    // RUSTFIX this is a real mess

    let mut any_remaining = false;

    for cand in candidates {
        if let Some(partial) = cand.take() {
            // If `partial` is still consistent given this new information,
            // put it back, otherwise destroy it.
            *cand = partial.reconstruct_args(idx, blob);
        }

        if let Some(partial) = cand {
            // Has this alias run out of `vinsts`?
            if partial.alias.vinsts.get(idx + 1).is_none() {
                if let Some(matches) = matches.as_mut() {
                    matches.push(cand.take().unwrap().into_complete()?);
                }
            } else {
                any_remaining = true;
            }
        }
    }

    Ok(any_remaining)
}

/// Statically disassemble the given stream of `Word`s from main memory, taking ownership
/// of the iterator (we need to read past the whole alias in order to determine when the
/// alias ends).
///
/// Returns a `DisassembledAlias`, giving the `Alias` which appears to correspond to the
/// current set of incoming instructions
///
/// Produces errors for invalid opcodes, etc. Consumes the iterator (we have no idea how
/// far we'll need to read the instruction stream before we are confident in our chosen
/// alias).
pub fn disassemble_alias<'a>(
    mut it: impl Iterator<Item = Result<DisassembledBlob<'a>, Error>>,
) -> Result<(DisassembledAlias<'a>, Vec<DisassembledBlob<'a>>), Error> {
    let mut blobs = Vec::new();
    let mut matches = Vec::new();

    // RUSTFIX use a `LinkedList` with `drain_filter` when it arrives.
    // FIXME EVIL? encapsulation breaking (pass this data in, and all families and aliases have an attached marker to it?)
    let mut candidates = Lang::get()
        .alias_iter()
        .map(PartialDisassembledAlias::new)
        .map(Some)
        .collect::<Vec<_>>();

    loop {
        let new_blob = it.next().ok_or(Error::UnexpectedEndOfStream)??;

        let any_remaining = resolve_partials_against_blob(
            &mut candidates,
            Some(&mut matches),
            blobs.len(),
            &new_blob,
        )?;

        blobs.push(new_blob);

        if !any_remaining {
            break;
        }
    }

    if matches.is_empty() {
        // Recalulate the alias candidate list excluding last `new_blob` we read, and generate a formatted error.
        candidates = Lang::get()
            .alias_iter()
            .map(PartialDisassembledAlias::new)
            .map(Some)
            .collect::<Vec<_>>();
        for (idx, blob) in blobs.iter().enumerate().take(blobs.len() - 1) {
            resolve_partials_against_blob(&mut candidates, None, idx, blob)?;
        }

        return Err(Error::NoSuitableAlias(
            blobs.iter().map(ToString::to_string).collect(),
            candidates
                .into_iter()
                .filter_map(|x| x)
                .map(|partial| partial.alias.name.clone())
                .collect(),
        ));
    }

    let disasm = matches
        .into_iter()
        .try_fold(None, |best: Option<DisassembledAlias>, next| {
            Ok(match best {
                None => Some(next),
                Some(best) => {
                    let cmp = best.partial_cmp(&next).ok_or_else(|| {
                        Error::AmbiguousAliasSpecificity(best.to_string(), next.to_string())
                    })?;
                    Some(if cmp == Ordering::Greater { best } else { next })
                }
            })
        })?
        .unwrap();

    // The `blobs` list may contain more `ResolvedBlob`s then actually match the finally-selected
    // `DisassembledAlias`---consider the case where `matches` consists of just 1 blob, thus
    // containing the single finally-selected alias, but other aliases of length 2 matched the first
    // blob read, so we read one more blob and then happened to rule all other aliases out.
    //
    // Thus, we drop the blobs at the end of the blobs list which aren't part of `disasm`.
    blobs.resize_with(disasm.alias.vinsts.len(), || unreachable!());

    {
        // BUGCHECK Consider making this whole thing a debug assertion.
        if !disasm
            .alias
            .instantiate(&disasm.args)
            .unwrap()
            .iter()
            .eq(blobs.iter().map(|dblob| &dblob.blob))
        {
            panic!(
                "re-instantiation disagree, disassembled:\n\t{}\nfrom:\n\t{:#?}",
                disasm,
                blobs.iter().map(ToString::to_string).collect::<Vec<_>>()
            );
        }
    }

    Ok((disasm, blobs))
}

/// Try to find the "most general" `Family` which includes this `Alias`, which will obviously be imperfect.
/// Since such a `Family` should always exist (a default one should be registered when the `Alias` was), we
/// panic if there are no results.
pub fn family_reverse_lookup(alias: &Alias) -> &Family {
    // Find the family containing `alias` which has the largest number of members.
    // FIXME EVIL? encapsulation breaking (pass this data in, and all families and aliases have an attached marker to it?)
    Lang::get()
        .family_iter()
        .filter(|fam| fam.variants.contains(&alias.name))
        .max_by_key(|fam| fam.variants.len())
        .expect("Alias is not in any families!")
}

#[derive(Debug)]
pub struct Context<'a> {
    family: &'a str,
    alias: DisassembledAlias<'a>,
    pub current_blob: Option<DisassembledBlob<'a>>,
    blob_queue: VecDeque<DisassembledBlob<'a>>,
    inst_count: usize,
}

impl<'a> Display for Context<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inst_count != 1 {
            write!(f, "<")?;
        }
        write!(f, "{}", self.family.to_uppercase())?;
        for arg in &self.alias.args {
            fmt_option_arg(f, Some(arg))?;
        }
        if self.inst_count != 1 {
            write!(
                f,
                ":{}/{}> {}",
                self.alias.alias.vinsts.len() - self.blob_queue.len(),
                self.alias.alias.vinsts.len(),
                self.current_blob().unwrap()
            )?;
        }
        Ok(())
    }
}

impl<'a> Context<'a> {
    fn new(alias: DisassembledAlias<'a>, blobs: Vec<DisassembledBlob<'a>>) -> Self {
        Context {
            family: &family_reverse_lookup(alias.alias).name,
            alias,
            inst_count: blobs.len(),
            blob_queue: VecDeque::from(blobs),
            current_blob: None,
        }
    }

    pub fn current_blob(&self) -> Option<&DisassembledBlob<'a>> {
        self.current_blob.as_ref()
    }

    fn advance_blob_queue(&mut self) {
        self.current_blob = self.blob_queue.pop_front();
    }
}

/// Container for stateful version of `disassemble_alias`, which keeps track of when we are "inside"
/// a multiple-instruction `Alias`, so that the computed disassembly doesn't change (and remains correct).
/// The `DisassemblyContext` stores the current `Alias` and current instruction, so that when we are
/// inside a multi-instruction alias we can show this in a pretty print.
#[derive(Debug)]
pub struct SteppingDisassembler<'a> {
    context: Context<'a>,
}

impl<'a> SteppingDisassembler<'a> {
    pub fn new(it: impl Iterator<Item = Word>) -> Result<Self, Error> {
        Ok(Self {
            context: Self::compute_context(None, it)?,
        })
    }

    fn compute_context(
        first_blob: Option<DisassembledBlob<'a>>,
        it: impl Iterator<Item = Word>,
    ) -> Result<Context<'a>, Error> {
        let mut it = it.peekable();

        // Recompute the current disassembly context, assuming that the first word from `it`
        // is the beginning of an `Alias`.
        let blobs_it = first_blob
            .map(Ok)
            .into_iter()
            .chain(std::iter::from_fn(move || {
                if it.peek().is_none() {
                    None
                } else {
                    Some(disassemble_blob(&mut it))
                }
            }));

        let (alias, blobs) = disassemble_alias(blobs_it)?;
        Ok(Context::new(alias, blobs))
    }

    pub fn step(&mut self, mut it: impl Iterator<Item = Word>) -> Result<(), Error> {
        let actual_blob = disassemble_blob(&mut it)?;

        self.context.advance_blob_queue();
        if self.context.current_blob().is_none() {
            self.context = SteppingDisassembler::compute_context(Some(actual_blob.clone()), it)?;
            self.context.advance_blob_queue();
        }

        // Check that the current blob matches the expected on from the top of the `blob_queue`.
        // If not, if we haven't already, recompute everything (calling `compute_context`), but
        // don't forget that we've already read a `ResolvedBlob` so we'll need to pass that to
        // some version of `recompute_context` and consequently `disassemble_alias`.
        let front_blob = self.context.current_blob().unwrap();
        if actual_blob.blob.to_words() != front_blob.blob.to_words() {
            // In principle (e.g. if the code itself was overwritten, or we jumped) this could happen and we would need
            // to recalculate, but for now let's just panic to find likely bugs.
            panic!(
                "{}\n\tactual_blob was: {:#}\n\tfront_blob was: {:#}",
                "The blob cache was invalidated, did we jump/or did the code itself change? (otherwise, this is a bug):",
                actual_blob,
                front_blob,
            );
        }

        Ok(())
    }

    pub fn context(&self) -> &Context<'a> {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    // RUSTFIX How do we test `SteppingDisassembler` in particular? Some small inline binaries would actually be possible.
    #[test]
    fn it_works() {
        // RUSTFIX implement
        // todo!();
    }
}
