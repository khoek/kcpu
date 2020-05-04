use super::{conductor::Statement, token::Token};
use crate::common;
use std::num::{ParseIntError, TryFromIntError};

#[derive(Debug)]
pub(super) enum Error {
    MalformedToken(String, &'static str),
    UnexpectedToken(Token, &'static str),
    UnexpectedEndOfStream(&'static str),
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::MalformedToken(err.to_string(), "could not parse numeric")
    }
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        // FIXME? right now this is blanket, but correct
        Error::MalformedToken(err.to_string(), "integral value out of bounds")
    }
}

enum Disposition {
    Continue,
    ExpectExhausted,
}

impl Statement {
    pub fn parse_iter(
        mut tokens: impl Iterator<Item = Result<Token, Error>>,
    ) -> Result<Vec<Statement>, Error> {
        let mut elems = Vec::new();
        let mut disp = Disposition::Continue;
        loop {
            match (disp, tokens.next().transpose()?) {
                (_, None) => {
                    return Ok(elems);
                }
                (Disposition::ExpectExhausted, Some(tk)) => {
                    return Err(Error::UnexpectedToken(
                        tk,
                        "Too many tokens! Expected end of line",
                    ))
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
        first: Token,
        tokens: &mut impl Iterator<Item = Result<Token, Error>>,
    ) -> Result<(Statement, Disposition), Error> {
        match first {
            Token::LabelDef(label) => Ok((Statement::LabelDef(label), Disposition::Continue)),
            Token::SpecialName(name) => Ok((
                Statement::parse_special(name, tokens)?,
                Disposition::ExpectExhausted,
            )),
            Token::Name(name) => Ok((
                Statement::Inst(
                    name,
                    tokens
                        .map(|tk| tk?.to_arg())
                        .collect::<Result<Vec<_>, Error>>()?,
                ),
                Disposition::ExpectExhausted,
            )),
            tk => Err(Error::UnexpectedToken(
                tk,
                "Expected a label definition, macro call, or instruction",
            )),
        }
    }

    fn parse_special(
        name: String,
        tokens: &mut impl Iterator<Item = Result<Token, Error>>,
    ) -> Result<Statement, Error> {
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
                    .unwrap_or(Err(Error::UnexpectedEndOfStream("string literal")))?
                    .to_string()?,
            )),
            _ => Err(Error::MalformedToken(
                name.to_owned(),
                "unknown special command name",
            )),
        }
    }
}

pub(super) fn parse(preproc_source: &str) -> Result<Vec<Statement>, Error> {
    common::accumulate(
        preproc_source
            .lines()
            .map(Token::parse_line)
            .map(Statement::parse_iter),
    )
}
