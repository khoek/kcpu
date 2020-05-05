use super::conductor::{BinaryElement, LabelName, Located, Statement};
use crate::asm::lang::Lang;
use crate::asm::model::Arg;
use crate::common;
use crate::spec::types::hw::{self, Byte, Word};
use std::iter;

#[derive(Debug)]
pub(super) enum Error {
    BadDataParity(),
    LabelNameCollidesWithInst(LabelName),
    InstUnknown(String),
    InstInvalidArgs(String, Vec<Arg<LabelName>>),
}

impl Statement {
    pub(super) fn generate(self) -> Result<Vec<BinaryElement>, Error> {
        match self {
            Statement::LabelDef(label) => Statement::generate_label_def(label),
            Statement::RawWords(words) => Statement::generate_raw_words(words),
            Statement::RawBytes(bytes) => Statement::generate_raw_bytes(bytes),
            Statement::RawString(string) => Statement::generate_raw_string(string),
            Statement::Inst(inst, args) => Statement::generate_inst(inst, args),
        }
    }

    fn generate_label_def(label: String) -> Result<Vec<BinaryElement>, Error> {
        if Lang::get().lookup_family(&label).is_some() {
            return Err(Error::LabelNameCollidesWithInst(label));
        }

        Ok(vec![BinaryElement::LabelDef(label)])
    }

    fn generate_raw_words(words: Vec<Word>) -> Result<Vec<BinaryElement>, Error> {
        Ok(vec![BinaryElement::Data(words)])
    }

    fn generate_raw_bytes(bytes: Vec<Byte>) -> Result<Vec<BinaryElement>, Error> {
        Statement::generate_raw_words(
            hw::bytes_to_words(bytes)
                .map(Result::Ok)
                .unwrap_or(Err(Error::BadDataParity()))?,
        )
    }

    fn generate_raw_string(string: String) -> Result<Vec<BinaryElement>, Error> {
        Statement::generate_raw_bytes(
            string
                .bytes()
                .chain(iter::repeat(b'\0').take(if string.len() % 2 == 0 { 2 } else { 1 }))
                .collect(),
        )
    }

    fn generate_inst(
        inst: String,
        args: Vec<Located<Arg<LabelName>>>,
    ) -> Result<Vec<BinaryElement>, Error> {
        // RUSTFIX at the moment, we drop argument location information, since we don't have anything to do with it
        // (and, even if we could think of something(?), we'd have to peer into `Alias::instantiate` to assign it).
        // This latter thing could be find, but we don't want to do it yet.
        let args: Vec<Arg<LabelName>> = args.into_iter().map(Located::value).collect();

        // RUSTFIX Actually, don't make this a function in `Lang`, since it is meant to generate multiple kinds of errors
        //         and `Lang` shouldn't have to know about that.

        let family = Lang::get()
            .lookup_family(&inst)
            .map(Result::Ok)
            .unwrap_or_else(|| Err(Error::InstUnknown(inst.clone())))?;

        let mut matches = family.variants.iter().filter_map(|alias| {
            let alias = Lang::get().lookup_alias(alias).unwrap();
            alias.instantiate(&args)
        });

        let blobs = matches.next();
        assert!(matches.next().is_none());

        // RUSTFIX list candiates when there is no match.
        let blobs = blobs
            .map(Result::Ok)
            .unwrap_or_else(|| Err(Error::InstInvalidArgs(inst, args)))?;

        Ok(blobs.into_iter().map(BinaryElement::Inst).collect())

        // RUSTFIX remove these comments

        // NOTE NOTE NOTE I just went through a whole thing with "do we need any of this matching stuff at all,
        // we can just try to perform the resolution aren't we checking everything twice??"
        //
        // The answer is yes, we don't have to, we can just to try to bind against the aliases directly and
        // have them return None if they don't work or a Blob if they do.
        // The only subtly is in the declaration of aliases: we work out the number of arguments an alias has
        // from the number of unbound arguments to its virtual instructions it has!
        //
        // Yay! But this doesn't make the large porition of the matching code which compares `ArgKind`s and
        // `Arg`s redudant, since we can just use it to perform runtime checking that the particular `Arg` we
        // are using from the arg list to bind a `Slot` is compatible with what is declared in the `InstDef`
        // corresponding to the position for that slot.

        // OOOH, but if `Alias`es don't have argkind lists themselves, we can't check for collisions directly when
        // they are registered in a `Family`. Instead we need to make a function which does `type inference`
        // and computes the arglist for an alias (but I see no need to store this...?)

        // RUSTFIX, (DUPLICATED IN `inst.rs`) Split the `inst.rs` file in half, with all of the basic notions like `RegRef`s,
        // `Half`, `Width`, `Const`, even `ConstBinding`s put somewhere and the stuff only needed to specify
        // `Family` and `Alias` definitions in `Lang`. Since, the resolution-enty-point will have to be in
        // `Lang` (because families do not hold `Alias`es (they can be referenced by multiple families)),
        // so `Lang` will need to resolve them.
    }
}

pub(super) fn generate(
    stmts: Vec<Located<Statement>>,
) -> Result<Vec<BinaryElement>, Located<Error>> {
    common::accumulate_vecs(
        stmts
            .into_iter()
            .map(|stmt| Ok(stmt.map_result_value(Statement::generate)?)),
    )
}
