use super::lang::Lang;
use crate::assembler::model::{
    Alias, Arg, Const, ConstBinding, Family, ResolvedArg, ResolvedBlob, Slot, Virtual,
};
use crate::common;
use crate::spec::{
    types::{
        hw::{Inst, Word, IU},
        schema::InstDef,
    },
    ucode::UCode,
};
use enum_map::EnumMap;
use std::{collections::VecDeque, fmt::Display, iter::Peekable};

#[derive(Debug)]
pub enum Error {
    InvalidOpcode(Word),
    UnexpectedEndOfStream,
    NoSuitableAlias(Vec<String>),
    CouldNotResolveAliasArgs(String, Vec<Option<ResolvedArg>>),
}

impl<'a> Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidOpcode(raw) => write!(f, "Invalid Opcode: {:#04X}", raw),
            Error::UnexpectedEndOfStream => write!(f, "Unexpectedly encountered end of stream"),
            // RUSTFIX nice messages for these next two:
            Error::CouldNotResolveAliasArgs(alias, args) => write!(
                f,
                "Could not resolve alias '{}' arguments: {:#?}",
                alias, args
            ),
            Error::NoSuitableAlias(blobs) => write!(f, "No suitable alias blobs: {:#?}", blobs),
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
        // RUSTFIX (Only consider:) make this a debug assertion, or just make a whole bunch of tests which check this?
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
    for slot in virt.slots {
        if let (iu, Some(Slot::Arg(idx))) = slot {
            match &args[idx] {
                Some(var) => {
                    if !blob.args[iu]
                        .as_ref()
                        .map(|arg| arg == var)
                        .unwrap_or(false)
                    {
                        return false;
                    }
                }
                None => args[idx] = blob.args[iu].clone(),
            }
        }
    }

    true
}

#[derive(Debug)]
pub struct DisassembledAlias<'a> {
    alias: &'a Alias,
    args: Vec<ResolvedArg>,
    blobs: Vec<DisassembledBlob<'a>>,
}

impl<'a> Display for DisassembledAlias<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.alias.name)?;
        for arg in &self.args {
            write!(f, " {}", arg)?;
        }
        Ok(())
    }
}

/// Statically disassemble the given stream of `Word`s from main memory, taking ownership
/// of the iterator (we need to read past the whole alias in order to determine when the
/// alias ends).
///
/// Returns a `DisassembledAlias`, giving the `Alias` which appears to correspond to the
/// current set of incoming instructions
///
/// Produces errors for invalid opcodes, etc.
// Does not consume the iterator so long as it uses the `Peekable`-ness
pub fn disassemble_alias<'a>(
    it: &mut Peekable<impl Iterator<Item = Result<DisassembledBlob<'a>, Error>>>,
) -> Result<DisassembledAlias<'a>, Error> {
    let mut blobs = Vec::new();

    // RUSTFIX use a `LinkedList` with `drain_filter` when it arrives.
    // FIXME EVIL? encapsulation breaking (pass this data in, and all families and aliases have an attached marker to it?)
    let mut candidates = Lang::get()
        .alias_iter()
        .map(|alias| Some((alias, vec![None; alias.arg_count])))
        .collect::<Vec<_>>();

    let mut idx = 0;
    let (alias, args) = loop {
        let new_blob = it.next().ok_or(Error::UnexpectedEndOfStream)??;
        for cand in &mut candidates {
            if let Some((alias, args)) = cand {
                let virt = alias.vinsts.get(idx);
                if virt.is_none()
                    || !reconstruct_args_using_blob_partials(args, virt.unwrap(), &new_blob)
                {
                    *cand = None;
                }
            }
        }
        blobs.push(new_blob);
        idx += 1;

        // RUSTFIX remove!
        // println!("candidates:");
        // for cand in &candidates {
        //     if let Some(cand) = cand {
        //         print!("{}, ", cand.0.name);
        //     }
        // }
        // println!();

        match common::find_is_unique(candidates.iter_mut(), |cand| cand.is_some()) {
            // RUSTFIX nicer formating for the blobs, print their disassembled instructions!
            None => Err(Error::NoSuitableAlias(
                blobs.iter().map(ToString::to_string).collect(),
            ))?,
            Some((cand, true)) => break cand.take().unwrap(),
            Some((_, false)) => (),
        }
    };

    let args_err = args.clone();
    let args = args
        .into_iter()
        .map(|arg| {
            arg.ok_or_else(|| Error::CouldNotResolveAliasArgs(alias.name.clone(), args_err.clone()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    {
        // for b in alias.instantiate(&args).unwrap() {
        //     println!("A {:#04X} {:?}", b.inst.encode(), b.binding);
        // }

        // for b in blobs.iter().map(|dblob| &dblob.blob) {
        //     println!("B {:#04X} {:?}", b.inst.encode(), b.binding);
        // }

        // let aa = alias.instantiate(&args).unwrap();
        // let l1 =  aa.iter().next().unwrap();
        // let l2 = blobs.iter().map(|dblob| &dblob.blob).next().unwrap();

        // println!("A {:#04X} {:?}", l1.inst.encode(), l1.binding);
        // println!("B {:#04X} {:?}", l2.inst.encode(), l2.binding);

        // println!("C {:#?} {:#?} {:#?}", l1.inst,  l2.inst, Inst::decode(l1.inst.encode()));

        // Verify that when the alias we have identified is instantiated with the resolved arguments we
        // recover the original list of `ResolvedBlob`s.

        // RUSTFIX (Only consider:) make this a debug assertion, or just make a whole bunch of tests which check this?
        assert!(alias
            .instantiate(&args)
            .unwrap()
            .iter()
            .eq(blobs.iter().map(|dblob| &dblob.blob)));
    }

    Ok(DisassembledAlias { alias, args, blobs })
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
/// Should have an API which returns the current `Alias` and current instruction, so that when we are
/// inside a multi-instruction alias we can show this in a pretty print (as a `DisassemblyContext`).
///
/// RUSTFIX How are we going to deal with writes over the code we have already disassembled?
/// I think we should store the pre-disassembled raw opcodes and check at each iteration (eating
/// at each stage a brand new iterator pointing to the current instruction) that they have not
/// changed. As a first measure we could just panic if they change, and later if we need we can
/// just recompute the disassemble when this is detected.
///
/// In particular, the `SteppingDissasembler` doesn't need to know anything about current RIP.
#[derive(Debug)]
pub struct SteppingDisassembler<'a> {
    context: Option<Context<'a>>,
}

// RUSTFIX implement a nice `Display` for this struct, which displays the current sub-instruction
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

#[derive(Debug)]
pub enum RelativePos {
    AliasBoundary,
    InsideAlias,
}

impl<'a> SteppingDisassembler<'a> {
    pub fn new() -> Self {
        SteppingDisassembler { context: None }
    }

    fn recompute_context(
        &mut self,
        first_blob: Option<DisassembledBlob<'a>>,
        it: &mut impl Iterator<Item = Word>,
    ) -> Result<&Context<'a>, Error> {
        let mut it = it.peekable();

        // Recompute the current disassembly context, assuming that the first word from `it`
        // is the beginning of an `Alias`.
        let blobs = first_blob
            .map(Result::Ok)
            .into_iter()
            .chain(std::iter::from_fn(move || {
                if it.peek().is_none() {
                    None
                } else {
                    Some(disassemble_blob(&mut it))
                }
            }));

        let alias = disassemble_alias(&mut blobs.peekable())?;
        let family = &family_reverse_lookup(alias.alias).name;
        let blob_queue: VecDeque<_> = alias.blobs.iter().cloned().collect();

        self.context = Some(Context {
            family,
            alias,
            inst_count: blob_queue.len(),
            blob_queue,
        });

        Ok(self.context.as_ref().unwrap())
    }

    pub fn step(
        &mut self,
        mut it: impl Iterator<Item = Word>,
    ) -> Result<(&Context<'a>, RelativePos), Error> {
        let mut context = self.context.as_mut();
        context
            .as_mut()
            .map(|context| context.blob_queue.pop_front());

        if context.is_none() || context.as_ref().unwrap().blob_queue.front().is_none() {
            return Ok((
                self.recompute_context(None, &mut it)?,
                RelativePos::AliasBoundary,
            ));
        }

        let front_blob = context.unwrap().blob_queue.pop_front().unwrap();

        // Check that the current blob matches the expected on from the top of the `blob_queue`, if so pop it.
        // If not, recompute everything (calling `recompute_context`), but don't forget that we've already read a `ResolvedBlob` so we'll need
        // to pass that to some version of `recompute_context` and consequenrly `disassemble_alias`.

        let blob = disassemble_blob(&mut it)?;

        if blob.blob.to_words() == front_blob.blob.to_words() {
            Ok((self.context.as_ref().unwrap(), RelativePos::InsideAlias))
        } else {
            // In principle (e.g. if the code itself was overwritten, or we jumped) this could happen and we would need
            // to recalculate, but for now let's just panic to find likely bugs.
            panic!("The blob cache was invalidated, did we jump/or did the code itself change? (otherwise, this is a bug)");
        }
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
