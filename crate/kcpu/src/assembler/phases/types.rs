use super::{generate, parse, resolve, tokenize};
use crate::assembler::model::{Arg, Blob};
use crate::spec::types::hw::*;
use derive_more::Constructor;
use std::fmt::Display;

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

pub type LabelName = String;

pub enum Statement {
    LabelDef(LabelName),
    RawWords(Vec<Word>),
    RawBytes(Vec<Byte>),
    RawString(String),
    Inst(String, Vec<Located<Arg<LabelName>>>),
}

pub enum BinaryElement {
    LabelDef(LabelName),
    Inst(Blob<LabelName>),
    Data(Vec<Word>),
}

#[derive(Debug, PartialEq, Clone, Eq, Constructor)]
pub struct Loc {
    line: usize,
    col: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Located<T: Sized> {
    loc: Option<Loc>,
    val: T,
}

impl Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(line: {}, col: {})", self.line, self.col)
    }
}

impl<T: Display> Display for Located<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.loc {
            None => write!(f, "@<unknown location>: {}", self.val),
            Some(loc) => write!(f, "@{}: {}", loc, self.val),
        }
    }
}

impl<T> Located<T> {
    fn new(loc: Option<Loc>, val: T) -> Self {
        Located { loc, val }
    }

    pub fn with_loc(loc: Loc, val: T) -> Self {
        Located::new(Some(loc), val)
    }

    pub fn value(self) -> T {
        self.val
    }

    pub fn proximate_to_option_loc(self, loc: Option<Loc>) -> Self {
        match self.loc {
            None => Self { loc, ..self },
            Some(_) => self,
        }
    }

    pub fn proximate_to_loc(self, loc: Loc) -> Self {
        self.proximate_to_option_loc(Some(loc))
    }

    pub fn proximate_to<S>(self, other: Located<S>) -> Self {
        self.proximate_to_option_loc(other.loc)
    }

    pub fn split<S, R, F>(self, f: F) -> (Located<S>, R)
    where
        F: FnOnce(T) -> (S, R),
    {
        let (s, r) = f(self.val);
        (Located::new(self.loc, s), r)
    }

    pub fn split_result<S, E, R, F>(self, f: F) -> Result<(Located<S>, R), Located<E>>
    where
        F: FnOnce(T) -> Result<(S, R), E>,
    {
        match f(self.val) {
            Err(err) => Err(Located::new(self.loc, err)),
            Ok((s, r)) => Ok((Located::new(self.loc, s), r)),
        }
    }

    pub fn map<S, F>(self, f: F) -> Located<S>
    where
        F: FnOnce(T) -> S,
    {
        Located::new(self.loc, f(self.val))
    }

    pub fn map_result<S, E, F>(self, f: F) -> Result<Located<S>, Located<E>>
    where
        F: FnOnce(T) -> Result<S, E>,
    {
        match f(self.val) {
            Ok(s) => Ok(Located::new(self.loc, s)),
            Err(err) => Err(Located::new(self.loc, err)),
        }
    }

    pub fn map_result_value<S, E, F>(self, f: F) -> Result<S, Located<E>>
    where
        F: FnOnce(T) -> Result<S, E>,
    {
        match f(self.val) {
            Ok(s) => Ok(s),
            Err(err) => Err(Located::new(self.loc, err)),
        }
    }

    pub fn transfer<S>(self, s: S) -> Located<S> {
        Located::new(self.loc, s)
    }
}

impl<T> From<T> for Located<T> {
    fn from(val: T) -> Self {
        Located { loc: None, val }
    }
}

impl<T> Located<Located<T>> {
    pub fn flatten(self) -> Located<T> {
        self.val.proximate_to_option_loc(self.loc)
    }
}

#[derive(Debug)]
pub enum Error {
    Tokenize(Located<String>),
    Parse(Located<String>),
    Generate(Located<String>),
    Resolve(String), // RUSTFIX Does a `Loc` make sense for this?
}

impl From<Located<tokenize::Error>> for Error {
    fn from(err: Located<tokenize::Error>) -> Self {
        Error::Tokenize(err.map(|err| format!("{}", err)))
    }
}

impl From<Located<parse::Error>> for Error {
    fn from(err: Located<parse::Error>) -> Self {
        Error::Parse(err.map(|err| format!("{}", err)))
    }
}

impl From<Located<generate::Error>> for Error {
    fn from(err: Located<generate::Error>) -> Self {
        Error::Generate(err.map(|err| format!("{}", err)))
    }
}

impl From<resolve::Error> for Error {
    fn from(err: resolve::Error) -> Self {
        Error::Resolve(format!("{}", err))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Assembly Error (in ")?;
        match self {
            Error::Tokenize(_) => write!(f, "Tokenizer"),
            Error::Parse(_) => write!(f, "Parser"),
            Error::Generate(_) => write!(f, "Generator"),
            Error::Resolve(_) => write!(f, "Resolver"),
        }?;
        write!(f, "): ")?;
        match self {
            Error::Tokenize(msg) => write!(f, "{}", msg),
            Error::Parse(msg) => write!(f, "{}", msg),
            Error::Generate(msg) => write!(f, "{}", msg),
            Error::Resolve(msg) => write!(f, "{}", msg),
        }
    }
}
