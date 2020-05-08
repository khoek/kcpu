use super::lang::Lang;
use crate::assembler::model::{
    Alias, Arg, Const, ConstBinding, Family, ResolvedArg, ResolvedBlob, Slot, Virtual,
};
use crate::spec::{
    types::{
        hw::{Inst, Word, IU},
        schema::InstDef,
    },
    ucode::UCode,
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
                "Aliases '{}' and '{}' are incompariable w.r.t. the specificity partial order",
                a1, a2,
            ),
        }
    }
}

pub fn next_resolved_blob<'a>(
    insts: &mut impl Iterator<Item = Word>,
) -> Result<ResolvedBlob, Error> {
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
    blob: ResolvedBlob,
    idef: &'a InstDef,
    args: EnumMap<IU, Option<ResolvedArg>>,
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

#[derive(Debug, Clone)]
struct PartialDisassembledAlias<'a> {
    alias: &'a Alias,
    args: Vec<Option<ResolvedArg>>,
}

impl<'a> Display for PartialDisassembledAlias<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.alias.name)?;
        for arg in &self.args {
            match arg {
                Some(arg) => write!(f, " {}", arg)?,
                None => write!(f, " <unresolved>")?,
            }
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

    fn unwrap(self) -> DisassembledAlias<'a> {
        DisassembledAlias {
            alias: self.alias,
            args: self.args.into_iter().map(|arg| arg.unwrap()).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisassembledAlias<'a> {
    alias: &'a Alias,
    args: Vec<ResolvedArg>,
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
            args: self.args.iter().cloned().map(Option::Some).collect(),
        }
    }
}

impl<'a> DisassembledAlias<'a> {
    fn specificity_score(&self) -> (usize, isize) {
        (self.alias.vinsts.len(), -(self.alias.arg_count as isize))
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

// Returns `false` if we found a contradiction, so reconstruction is not possible (it doesn't even make sense).
fn reconstruct_args_using_blob_partials(
    args: &mut Vec<Option<ResolvedArg>>,
    virt: &Virtual,
    blob: &DisassembledBlob,
) -> bool {
    if virt.opclass != blob.idef.opclass {
        return false;
    }

    // The opclass check we have just performed means that we just need to check
    // the `Slot::Arg`s in `virt`---all others are automatically known to be
    // compatible.
    // RUSTFIX THIS COMMENT IS WRONG.
    //
    // FIXME FIXME FIXME
    //
    for (iu, slot) in virt.slots {
        match (&blob.args[iu], slot) {
            (Some(arg), Some(Slot::Arg(idx))) => match &args[idx] {
                Some(blob_arg) if arg != blob_arg => return false,
                Some(_) => (),
                None => args[idx] = blob.args[iu].clone(),
            },
            // If the slot is not `Slot::Arg` then it is bound, so it is
            // safe to `unwrap()` after `Slot::to_arg()`.
            (Some(arg), Some(slot)) if slot.to_arg().unwrap() != *arg => return false,
            (None, None) => (),
            _ => panic!(
                "argument disagree for same opcode: '{:?}' vs '{:?}'",
                slot, blob.args[iu]
            ),
        }
    }

    true
}

/// Returns the number of `PartialDisassembledAlias` which have not been ruled out nor matched.
fn resolve_partials_against_blob<'a>(
    mut matches: Option<&mut Vec<DisassembledAlias<'a>>>,
    candidates: &mut Vec<Option<PartialDisassembledAlias<'a>>>,
    idx: usize,
    blob: &DisassembledBlob,
) -> Result<usize, Error> {
    // RUSTFIX change `candidates` to use a `LinkedList` with `drain_filter` when it arrives.

    let mut remaining_unresolved = 0;
    for cand in candidates {
        if let Some(mut partial) = cand.take() {
            match partial.alias.vinsts.get(idx) {
                Some(virt) => {
                    if reconstruct_args_using_blob_partials(&mut partial.args, virt, blob) {
                        remaining_unresolved += 1;
                        // Put the option back
                        *cand = Some(partial);
                    }
                }
                None => {
                    {
                        // BUGCHECK Consider making this whole thing a debug assertion.
                        for i in 0..partial.args.len() {
                            if partial.args[i].is_none() {
                                return Err(Error::CouldNotResolveAliasArgs(
                                    partial.to_string(),
                                    i,
                                ));
                            }
                        }
                    }

                    if let Some(matches) = matches.as_mut() {
                        matches.push(partial.unwrap());
                    }
                }
            }
        }
    }

    Ok(remaining_unresolved)
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
        let remaining = resolve_partials_against_blob(
            Some(&mut matches),
            &mut candidates,
            blobs.len(),
            &new_blob,
        )?;
        blobs.push(new_blob);

        if remaining == 0 {
            break;
        }
    }

    if matches.len() == 0 {
        // Recalulate the alias candidate list excluding las `new_blob` we read,
        // and generate a formatted error.
        candidates = Lang::get()
            .alias_iter()
            .map(PartialDisassembledAlias::new)
            .map(Some)
            .collect::<Vec<_>>();
        for idx in 0..blobs.len() - 1 {
            resolve_partials_against_blob(None, &mut candidates, idx, &blobs[idx])?;
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
                    let cmp = best
                        .partial_cmp(&next)
                        .ok_or(Error::AmbiguousAliasSpecificity(
                            best.to_string(),
                            next.to_string(),
                        ))?;
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

/// Container for stateful version of `disassemble_alias`, which keeps track of when we are "inside"
/// a multiple-instruction `Alias`, so that the computed disassembly doesn't change (and remains correct).
/// The `DisassemblyContext` stores the current `Alias` and current instruction, so that when we are
/// inside a multi-instruction alias we can show this in a pretty print.
#[derive(Debug)]
pub struct SteppingDisassembler<'a> {
    context: Context<'a>,
}

#[derive(Debug)]
pub struct Context<'a> {
    family: &'a str,
    alias: DisassembledAlias<'a>,
    blob_queue: VecDeque<DisassembledBlob<'a>>,
    inst_count: usize,
}

impl<'a> Display for Context<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.alias)?;
        if self.inst_count != 1 {
            write!(f, "\t{}", self.blob_queue.front().unwrap())?;
        }
        Ok(())
    }
}

impl<'a> SteppingDisassembler<'a> {
    pub fn new(it: &mut impl Iterator<Item = Word>) -> Result<Self, Error> {
        Ok(Self {
            context: Self::compute_context(None, it)?,
        })
    }

    fn compute_context(
        first_blob: Option<DisassembledBlob<'a>>,
        it: &mut impl Iterator<Item = Word>,
    ) -> Result<Context<'a>, Error> {
        let mut it = it.peekable();

        // Recompute the current disassembly context, assuming that the first word from `it`
        // is the beginning of an `Alias`.
        let blobs_it = first_blob
            .map(Result::Ok)
            .into_iter()
            .chain(std::iter::from_fn(move || {
                if it.peek().is_none() {
                    None
                } else {
                    Some(disassemble_blob(&mut it))
                }
            }));

        let (alias, blobs) = disassemble_alias(blobs_it)?;
        Ok(Context {
            family: &family_reverse_lookup(alias.alias).name,
            alias,
            inst_count: blobs.len(),
            blob_queue: VecDeque::from(blobs),
        })
    }

    pub fn step(&mut self, mut it: impl Iterator<Item = Word>) -> Result<(), Error> {
        match self.context.blob_queue.pop_front() {
            None => {
                self.context = SteppingDisassembler::compute_context(None, &mut it)?;
                return Ok(());
            }
            Some(front_blob) => {
                // Check that the current blob matches the expected on from the top of the `blob_queue`.
                // If not, recompute everything (calling `recompute_context`), but don't forget that we've already read a `ResolvedBlob` so we'll need
                // to pass that to some version of `recompute_context` and consequenrly `disassemble_alias`.
                let blob = disassemble_blob(&mut it)?;
                if blob.blob.to_words() == front_blob.blob.to_words() {
                    Ok(())
                } else {
                    // In principle (e.g. if the code itself was overwritten, or we jumped) this could happen and we would need
                    // to recalculate, but for now let's just panic to find likely bugs.
                    panic!("The blob cache was invalidated, did we jump/or did the code itself change? (otherwise, this is a bug)");
                }
            }
        }
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
        todo!();
    }
}
