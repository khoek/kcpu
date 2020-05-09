use super::types::Located;
use super::{tokenize::Token, types::Statement};
use crate::assembler::model::{Arg, ConstBinding};
use crate::common;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    UnknownSpecialCommandName(String),
    UnexpectedToken(Token, &'static str),
    UnexpectedEndOfStream(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownSpecialCommandName(cmd) => {
                write!(f, "Unknown special command: '{}'", cmd)
            }
            Error::UnexpectedToken(tk, msg) => write!(f, "Unexpected token: '{}': {}", tk, msg),
            Error::UnexpectedEndOfStream(msg) => {
                write!(f, "Unexpectedly encountered end of stream: {}", msg)
            }
        }
    }
}

impl Token {
    pub fn into_string(self) -> Result<String, Error> {
        match self {
            Token::String(s) => Ok(s),
            tk => Err(Error::UnexpectedToken(tk, "string literal")),
        }
    }

    pub fn into_arg(self) -> Result<Arg<String>, Error> {
        match self {
            Token::RegRef(r) => Ok(Arg::Reg(r)),
            Token::Const(c) => Ok(Arg::Const(ConstBinding::Resolved(c))),
            Token::Name(n) => Ok(Arg::Const(ConstBinding::Unresolved(n))),
            tk => Err(Error::UnexpectedToken(tk, "argument")),
        }
    }
}

enum Disposition {
    Continue,
    ExpectExhausted,
}

impl Statement {
    pub fn parse_iter(
        mut tokens: impl Iterator<Item = Located<Token>>,
    ) -> Result<Vec<Located<Statement>>, Located<Error>> {
        let mut elems = Vec::new();
        let mut disp = Disposition::Continue;
        loop {
            match (disp, tokens.next()) {
                (_, None) => return Ok(elems),

                (Disposition::ExpectExhausted, Some(tk)) => {
                    return Err(tk.map(|tk| {
                        Error::UnexpectedToken(tk, "Too many tokens! Expected end of line")
                    }))
                }

                (Disposition::Continue, Some(tk)) => {
                    let (e, d) = Statement::parse_once(tk, &mut tokens)?;
                    elems.push(e);
                    disp = d;
                }
            }
        }
    }

    fn parse_once(
        first: Located<Token>,
        tokens: &mut impl Iterator<Item = Located<Token>>,
    ) -> Result<(Located<Statement>, Disposition), Located<Error>> {
        first
            .split_result(|first| match first {
                Token::LabelDef(label) => Ok((Statement::LabelDef(label), Disposition::Continue)),
                Token::SpecialName(name) => Ok((
                    Statement::parse_special(name, tokens)?,
                    Disposition::ExpectExhausted,
                )),
                Token::Name(name) => Ok((
                    Statement::Inst(
                        name,
                        tokens
                            .map(|tk| tk.try_map(Token::into_arg))
                            .collect::<Result<Vec<_>, Located<Error>>>()?,
                    ),
                    Disposition::ExpectExhausted,
                )),
                tk => Err(Located::from(Error::UnexpectedToken(
                    tk,
                    "Expected a label definition, macro call, or instruction",
                ))),
            })
            .map_err(Located::flatten)
    }

    fn parse_special(
        name: String,
        tokens: &mut impl Iterator<Item = Located<Token>>,
    ) -> Result<Statement, Located<Error>> {
        match name.as_str() {
            "warray" => {
                // RUSTFIX implement, consume numeric tokens which we expect to be word-size
                Ok(Statement::RawWords(vec![]))
            }
            "barray" => {
                // RUSTFIX implement, consume numeric tokens which we expect to be word-size
                // NOTE we don't have to do parity checking here, just parsing :) That is the job of the generator
                Ok(Statement::RawBytes(vec![]))
            }
            "string" => Ok(Statement::RawString(
                tokens
                    .next()
                    .map(Result::Ok)
                    .unwrap_or(Err(Error::UnexpectedEndOfStream("string literal")))?
                    .try_map_err(|tk| tk.into_string())?,
            )),
            _ => Err(Located::from(Error::UnknownSpecialCommandName(
                name.to_owned(),
            ))),
        }
    }
}

pub fn parse(tokens: Vec<Vec<Located<Token>>>) -> Result<Vec<Located<Statement>>, Located<Error>> {
    common::accumulate_vecs(
        tokens
            .into_iter()
            .map(|line| Statement::parse_iter(line.into_iter())),
    )
}
