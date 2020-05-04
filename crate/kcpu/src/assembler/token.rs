use super::parse::Error;
use crate::asm::model::{Arg, Const, ConstBinding, RegRef};
use crate::common;
use crate::spec::types::{
    hw::{word_from_i64_wrapping, Byte, PReg},
    schema::{Half, Width},
};
use std::convert::TryFrom;
use strum::IntoEnumIterator;

// This enum models the kinds of tokens we can encounter and unambiguously distinguish between
// as we parse a stream. It *does not* model the semantics of the assemble language, where
// e.g. there is a distinction between an instruction name and a reference to a label
// (which for us is inferred from context).
#[derive(Debug)]
pub(super) enum Token {
    LabelDef(String),
    SpecialName(String),

    RegRef(RegRef),
    Const(Const),

    String(String),
    Name(String),
}

enum CommandChar<'a> {
    Containing(&'a str),
    Starting(&'a str),
    Ending(&'a str),
}

impl<'a> CommandChar<'a> {
    fn to_str(&self) -> &str {
        match *self {
            CommandChar::Containing(c) => c,
            CommandChar::Starting(c) => c,
            CommandChar::Ending(c) => c,
        }
    }

    fn to_char(&self) -> Option<char> {
        let mut cs = self.to_str().chars();
        match (cs.next(), cs.next()) {
            (Some(c), None) => Some(c),
            _ => None,
        }
    }

    fn matches<'b>(&self, s: &'b str) -> Option<&'b str> {
        match *self {
            CommandChar::Containing(c) => {
                if s.contains(c) {
                    Some(s)
                } else {
                    None
                }
            }
            CommandChar::Starting(c) => {
                // RUSTFIX this is unicode unsafe
                if s.starts_with(c) {
                    Some(&s[c.len()..])
                } else {
                    None
                }
            }
            CommandChar::Ending(c) => {
                // RUSTFIX this is unicode unsafe
                if s.ends_with(c) {
                    Some(&s[..s.len() - c.len()])
                } else {
                    None
                }
            }
        }
    }
}

impl Token {
    // RUSTFIX Don't split up strings with spaces!
    fn str_to_raw_tokens<'a>(line: &'a str) -> impl Iterator<Item = &str> + 'a {
        line.split(" ").filter(|s| !s.is_empty())
    }

    pub fn parse_line<'a>(line: &'a str) -> impl Iterator<Item = Result<Token, Error>> + 'a {
        Token::str_to_raw_tokens(line).map(Token::parse)
    }

    const COMMAND_CHARS: [(CommandChar<'static>, fn(&str) -> Result<Self, Error>); 9] = [
        (CommandChar::Containing(" "), Token::parse_error),
        (CommandChar::Containing("#"), Token::parse_error),
        (CommandChar::Ending(":"), Token::parse_label_def),
        (CommandChar::Starting("!"), Token::parse_special),
        (CommandChar::Starting("%"), Token::parse_reg_ref),
        (CommandChar::Starting("$"), |s| {
            Token::parse_numeric(s, Width::Word)
        }),
        (CommandChar::Starting("l$"), |s| {
            Token::parse_numeric(s, Width::Byte(Half::Lo))
        }),
        (CommandChar::Starting("h$"), |s| {
            Token::parse_numeric(s, Width::Byte(Half::Hi))
        }),
        (CommandChar::Starting("\""), Token::parse_string),
    ];

    fn parse(raw: &str) -> Result<Self, Error> {
        assert!(raw.len() > 0);

        for (c, parser) in Token::COMMAND_CHARS.iter() {
            if let Some(raw) = c.matches(raw) {
                return parser(raw);
            }
        }

        Token::parse_name(raw)
    }

    fn parse_error(raw: &str) -> Result<Self, Error> {
        Err(Error::MalformedToken(raw.to_owned(), "unexpected token"))
    }

    fn parse_label_def(raw: &str) -> Result<Self, Error> {
        Ok(Token::LabelDef(Token::parse_label_string(raw)?))
    }

    fn parse_label_string(label: &str) -> Result<String, Error> {
        if label.len() == 0 {
            return Err(Error::MalformedToken(
                label.to_owned(),
                "Label names must have nonzero length",
            ));
        }

        Ok(Token::parse_name_string(&label)?)
    }

    fn parse_reg_ref(raw: &str) -> Result<Self, Error> {
        let mut it = raw.chars();
        let first = match it.next() {
            Some(first) => first,
            None => {
                return Err(Error::MalformedToken(
                    raw.to_owned(),
                    "expected register reference",
                ))
            }
        };

        let width = match first {
            'r' => Width::Word,
            'l' => Width::Byte(Half::Lo),
            'h' => Width::Byte(Half::Hi),
            _ => {
                return Err(Error::MalformedToken(
                    raw.to_owned(),
                    "expected 'r', 'l', or 'h'",
                ))
            }
        };

        // RUSTFIX prohibit direct reference to %rid?
        let reg_name = it.collect::<String>();
        let preg =
            match PReg::iter().find(|reg| common::eq_ignore_case(&reg_name, &reg.to_string())) {
                Some(preg) => preg,
                _ => {
                    return Err(Error::MalformedToken(
                        raw.to_owned(),
                        "expected register name",
                    ))
                }
            };

        Ok(Token::RegRef(RegRef::new(preg, width)))
    }

    fn parse_numeric(raw: &str, width: Width) -> Result<Self, Error> {
        let val = if raw.starts_with("0x") {
            i64::from_str_radix(&raw[2..], 16)
        } else if raw.starts_with("0o") {
            i64::from_str_radix(&raw[2..], 8)
        } else if raw.starts_with("0b") {
            i64::from_str_radix(&raw[2..], 2)
        } else {
            i64::from_str_radix(raw, 10)
        }?;

        let val = word_from_i64_wrapping(val)?;

        match width {
            // RUSTFIX this would be a perfect place to add messages to the errors these give off using
            // the `anyhow` crate, since you can't distinguish between `Word` and `Byte` from the message...
            Width::Word => Ok(Token::Const(Const::Word(val))),
            Width::Byte(half) => Ok(Token::Const(Const::Byte(Byte::try_from(val)?, half))),
        }
    }

    fn parse_string(raw: &str) -> Result<Self, Error> {
        if !raw.ends_with('"') {
            return Err(Error::MalformedToken(
                raw.to_owned(),
                "no terminating '\"' while parsing string literal",
            ));
        }

        // RUSTFIX this is unicode unsafe
        Ok(Token::String(raw[0..raw.len() - 1].to_owned()))
    }

    fn parse_special(raw: &str) -> Result<Self, Error> {
        Ok(Token::SpecialName(Token::parse_name_string(raw)?))
    }

    fn parse_name(raw: &str) -> Result<Self, Error> {
        Ok(Token::Name(Token::parse_name_string(raw)?))
    }

    fn parse_name_string(raw: &str) -> Result<String, Error> {
        for c in raw.chars() {
            for (cmd, _) in Token::COMMAND_CHARS.iter() {
                if cmd.to_char().map_or(false, |cmd| c == cmd) {
                    return Err(Error::MalformedToken(
                        raw.to_owned(),
                        "name, found unacceptable special character",
                    ));
                }
            }
        }

        Ok(raw.to_owned())
    }

    pub fn to_string(self) -> Result<String, Error> {
        match self {
            Token::String(s) => Ok(s),
            tk => Err(Error::UnexpectedToken(tk, "string literal")),
        }
    }

    pub fn to_arg(self) -> Result<Arg<String>, Error> {
        match self {
            Token::RegRef(r) => Ok(Arg::Reg(r)),
            Token::Const(c) => Ok(Arg::Const(ConstBinding::Resolved(c))),
            Token::Name(n) => Ok(Arg::Const(ConstBinding::Unresolved(n))),
            tk => Err(Error::UnexpectedToken(tk, "argument")),
        }
    }
}
