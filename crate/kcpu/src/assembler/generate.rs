use super::types::{BinaryElement, LabelName, Located, Statement};
use crate::asm::lang::Lang;
use crate::asm::model::Arg;
use crate::common;
use crate::spec::types::{
    hw::{self, Byte, Word},
    schema::ArgKind,
};
use colored::Colorize;
use itertools::{EitherOrBoth, Itertools};
use std::{fmt::Display, iter};

#[derive(Debug)]
pub(super) enum Error {
    BadDataParity,
    LabelNameCollidesWithInst(LabelName),
    InstUnknown(String),
    InstInvalidArgs(String, Vec<Arg<LabelName>>),
}

impl Error {
    fn fmt_arg_kinds_given_args(
        f: &mut std::fmt::Formatter<'_>,
        kinds: &[ArgKind],
        args: &[Arg<LabelName>],
    ) -> std::fmt::Result {
        let fmted = kinds
            .iter()
            .zip_longest(args.iter())
            .map(|res| match res {
                EitherOrBoth::Left(kind) => format!("{}", kind).yellow(),
                EitherOrBoth::Right(_) => format!("<extra>").yellow(),
                EitherOrBoth::Both(kind, arg) => {
                    let kind_fmt = format!("{}", kind);
                    if kind.matches(arg) {
                        kind_fmt.green()
                    } else {
                        kind_fmt.red()
                    }
                }
            })
            .map(|ts| ts.to_string())
            .collect::<Vec<_>>();
        write!(f, "{}", fmted.join(", "))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadDataParity => write!(f, "Bad data parity"),
            Error::LabelNameCollidesWithInst(ln) => write!(
                f,
                "Unacceptable label name '{}', it collides with an instruction name",
                ln
            ),
            Error::InstUnknown(name) => write!(f, "Unknown instruction '{}'", name),
            Error::InstInvalidArgs(name, args) => {
                writeln!(
                    f,
                    "Invalid arguments passed to instruction '{}', arguments were:",
                    name
                )?;
                writeln!(
                    f,
                    "\t{}",
                    args.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )?;
                writeln!(f, "candidate argument lists were:")?;
                let family = Lang::get().lookup_family(&name).unwrap();
                for alias in family
                    .variants
                    .iter()
                    .map(|alias| Lang::get().lookup_alias(alias).unwrap())
                {
                    write!(f, "\t{: <3}: ", alias.name)?;
                    Error::fmt_arg_kinds_given_args(f, &alias.infer_type(), args)?;
                    writeln!(f)?;
                }
                Ok(())
            }
        }
    }
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
                .unwrap_or(Err(Error::BadDataParity))?,
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

        let matches = family
            .variants
            .iter()
            .filter_map(|alias| Lang::get().lookup_alias(alias).unwrap().instantiate(&args));

        let blobs = common::unwrap_at_most_one(matches);

        // RUSTFIX list candiates when there is no match.
        let blobs = blobs
            .map(Result::Ok)
            .unwrap_or_else(|| Err(Error::InstInvalidArgs(inst, args)))?;

        Ok(blobs.into_iter().map(BinaryElement::Inst).collect())

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
