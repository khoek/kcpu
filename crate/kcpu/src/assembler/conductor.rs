use super::{generate, parse, preprocess, resolve};
use crate::asm::model::Arg;
use crate::asm::model::Blob;
use crate::spec::types::hw::*;
use derive_more::Constructor;

/*
    UPDATE:  FIX THE TEXT BELOW, THIS IS HOW WE DO IT NOW

    Phases:

        1.  Tokenization: The source file is stripped of comments, newlines are converted into semicolons,
            the source is then tokenized into a sequence split at each newline (into statement streams), and then
            each statement is tokenized at spaces.

        2.  Parsing: Each statement stream is parsed into a `Statement`, either:

            `LabelDef`
            `BinaryData`
            `StringData`
            `Inst`

            In this stage we check for things like the use of reserved instruction names in labels,
            but avoid trying to understand the semantic meaning of the statements.

        3.  Expansion: Each `Statement` is expanded into a `Vec<BinaryElement>` (a local operation).

        4.  Resolution: The locations of the labels are read from the `BinaryElement` list, and givne
            this data each `BinaryElement` is resolved into a `Vec<Word>` of binary data.

        5.  Concatenation: Each of these lists of words are concatenated to give the final assembled binary.










    The architecture of the assembler:

        When we want to assemble a program, we need to find a way to get from a stream of tokens
        (strings) to the binary data of the program binary. We divide the input file into token
        "streams", begun by ';' or newlines (at this point comments have already been removed).

        Given a fresh token stream, we expect that the first token (we split on spaces) is one of:

            * a label definition (as specified by the token ending in a ':'), or
            * a macro definition (as specified by the token beginning with a '@'), or
            * an instruction name.

        If we encounter a label we remove it from the stream, record it, and consider the resulting
        stream again "fresh". Otherwise control is transfered to the macro or instruction stream
        parser.

            Macros:
                Currently there are no macros.

            Instructions:
                We first check whether an instruction family by the specified name exists.
                Then we continue to parse until the end of the stream; our notation for constants,
                registers (including high/low byte), and label references means that the type of each kind
                of argument (in the sense of `super::inst::ArgKind`) is unambiguous. Then we query the
                family for an alias matching these argument types. Assuming we obtain a result, we bind
                our actual parsed argument list against the returned `Alias`, producing a `Vec<Blob>` of
                compiled binary data. (Under the hood, the `Virtual`s inside the `Alias` are bound to the
                arguments we supplied to the `Alias` holding them.)

                Finally, the resulting blobs are recorded in the assembler.

        Whether we found a label, a macro, or an instruction, each is saved inside an `Statement`, and we
        accumulate a big list of `Statement`s as we make progress.

        Once we have run out of token streams, we build metadata for the resulting the list of `Statement`s
        (currently, we just save the final locations of the labels by looking for `Blob`s in order). Then
        we resolve the `Blob`s against this metadata, producing a `Vec<Word>` for each. Concatenating these
        we obtain the final program binary as a `Vec<Word>`.
*/

// RUSTFIX no zero length label names!

pub(super) type LabelName = String;

pub(super) enum Statement {
    LabelDef(LabelName),
    RawWords(Vec<Word>),
    RawBytes(Vec<Byte>),
    RawString(String),
    Inst(String, Vec<Arg<LabelName>>),
}

pub(super) enum BinaryElement {
    LabelDef(LabelName),
    Inst(Blob<LabelName>),
    Data(Vec<Word>),
}

#[derive(Debug, Constructor)]
pub struct Loc {
    line: usize,
    col: usize,
}

#[derive(Debug, Constructor)]
pub struct Located<T> {
    loc: Loc,
    val: T,
}

impl<T> Located<T> {
    pub fn map<S>(self, f: fn(T) -> S) -> Located<S> {
        Located {
            loc: self.loc,
            val: f(self.val),
        }
    }
}

impl std::fmt::Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line: {}, col: {}", self.line, self.col)
    }
}

#[derive(Debug)]
pub enum Error {
    Preprocess(Located<String>),
    Parse(Located<String>),
    Expand(Located<String>),
    Resolve(String), // Does a `Loc` make sense for this?
}

impl From<Located<preprocess::Error>> for Error {
    fn from(err: Located<preprocess::Error>) -> Self {
        Error::Preprocess(err.map(|err| format!("{:?}", err)))
    }
}

impl From<Located<parse::Error>> for Error {
    fn from(err: Located<parse::Error>) -> Self {
        Error::Parse(err.map(|err| format!("{:?}", err)))
    }
}

impl From<Located<generate::Error>> for Error {
    fn from(err: Located<generate::Error>) -> Self {
        Error::Expand(err.map(|err| format!("{:?}", err)))
    }
}

impl From<resolve::Error> for Error {
    fn from(err: resolve::Error) -> Self {
        Error::Resolve(format!("{:?}", err))
    }
}

// RUSTFIX have the token-generating stream convert `Error`s into `Located<Error>`s.
// RUSTFIX remove
fn dummy_wrap<T>(t: T) -> Located<T> {
    Located::new(Loc::new(0, 0), t)
}

// RUSTFIX ERROR OVERHAUL: 1. Make the `Located<xxx>`s actually get injected in the  right places, and
//                         2. Exception overhaul, just use `format!()` in-place to generate the messages,
//                            since we are just doing `to_owned` spam everywhere now and the slices were
//                            limiting in some places when I was originally writing the messages.

pub fn assemble(source: &str) -> Result<Vec<Word>, Error> {
    let preproc_source = preprocess::preprocess(source)
        .map_err(dummy_wrap)
        .map_err(Error::from)?;

    let statements = parse::parse(&preproc_source)
        .map_err(dummy_wrap)
        .map_err(Error::from)?;

    let elems = generate::generate(statements)
        .map_err(dummy_wrap)
        .map_err(Error::from)?;

    let bins = resolve::resolve(elems).map_err(Error::from)?;

    Ok(bins)
}
