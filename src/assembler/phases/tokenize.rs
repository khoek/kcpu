use super::types::{Loc, Located};
use crate::assembler::model::{Const, RegRef};
use crate::common;
use crate::spec::types::{
    hw::{word_from_i64_wrapping, Byte, PReg},
    schema::{Half, Width},
};
use std::convert::TryFrom;
use std::{
    fmt::Display,
    num::{ParseIntError, TryFromIntError},
};
use strum::IntoEnumIterator;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    MalformedToken(String, &'static str),
    UnterminatedStringLiteral,
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

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MalformedToken(raw, msg) => write!(f, "Malformed token '{}': {}", raw, msg),
            Error::UnterminatedStringLiteral => {
                write!(f, "Encountered unterminated string literal")
            }
        }
    }
}

// This enum models the kinds of tokens we can encounter and unambiguously distinguish between
// as we parse a stream. It *does not* model the semantics of the assemble language, where
// e.g. there is a distinction between an instruction name and a reference to a label
// (which for us is inferred from context).
#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    LabelDef(String),
    SpecialName(String),

    RegRef(RegRef),
    Const(Const),

    String(String),
    Name(String),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::LabelDef(label) => write!(f, "LabelDef({})", label),
            Token::SpecialName(name) => write!(f, "SpecialName({})", name),

            Token::RegRef(r) => write!(f, "RegRef({})", r),
            Token::Const(c) => write!(f, "Const({})", c),

            Token::String(s) => write!(f, "String({})", s),
            Token::Name(s) => write!(f, "Name({})", s),
        }
    }
}

enum CommandChar<'a> {
    Containing(&'a str),
    Starting(&'a str),
    Ending(&'a str),
}

impl<'a> CommandChar<'a> {
    fn to_str(&self) -> &str {
        match self {
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
        match self {
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

type CommandCharHandler = (CommandChar<'static>, fn(&str) -> Result<Token, Error>);

impl Token {
    const COMMAND_CHARS: [CommandCharHandler; 9] = [
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
        assert!(!raw.is_empty());

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
        if label.is_empty() {
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
}

#[derive(Debug, PartialEq, Eq)]
enum RawToken<'a> {
    Value(Located<&'a str>),
    Nothing,
    EndOfStream,
}

enum TerminatorKind {
    Hard,
    Whitespace,
}

impl TerminatorKind {
    /// Note that we don't have to check whether invalid characters
    /// are present at this stage, this occurs when the raw tokens
    /// are converted into `Token`s.
    fn from_char(c: Option<char>) -> Option<Self> {
        match c {
            None | Some(RawToken::COMMENT_CHAR) | Some(RawToken::NEWLINE_CHAR) => {
                Some(TerminatorKind::Hard)
            }
            Some(c) => {
                if c.is_whitespace() {
                    Some(TerminatorKind::Whitespace)
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug)]
enum SeekMode {
    Comment,
    Whitespace,
    StringLiteral,
    Name,
}

impl From<char> for SeekMode {
    fn from(c: char) -> Self {
        match c {
            RawToken::COMMENT_CHAR => SeekMode::Comment,
            RawToken::STRING_LITERAL_CHAR => SeekMode::StringLiteral,
            c => {
                if c.is_whitespace() {
                    SeekMode::Whitespace
                } else {
                    SeekMode::Name
                }
            }
        }
    }
}

impl SeekMode {
    // RUSTFIX This is much better than it was: can it be simplified any further?
    fn should_terminate(&self, cur: Option<char>) -> Result<Option<SeekEnd>, Error> {
        match (self, TerminatorKind::from_char(cur)) {
            (SeekMode::Comment, _) => Ok(Some(SeekEnd::SkipEverything)),
            (SeekMode::Whitespace, Some(TerminatorKind::Whitespace)) => Ok(None),
            (SeekMode::Whitespace, _) => Ok(Some(SeekEnd::Skip)),
            (SeekMode::StringLiteral, term_kind) => match (cur, term_kind) {
                (Some(RawToken::STRING_LITERAL_CHAR), _) => Ok(Some(SeekEnd::AdvanceOne)),
                (_, Some(TerminatorKind::Hard)) => Err(Error::UnterminatedStringLiteral),
                _ => Ok(None),
            },
            (SeekMode::Name, Some(_)) => Ok(Some(SeekEnd::Current)),
            (SeekMode::Name, None) => Ok(None),
        }
    }
}

enum SeekEnd {
    Current,
    AdvanceOne,
    SkipEverything,
    Skip,
}

impl SeekEnd {
    fn build_raw_token<F>(
        self,
        line_no: usize,
        line: &'_ str,
        start_idx: usize,
        cur_idx: Option<usize>,
        advance: F,
    ) -> RawToken<'_>
    where
        F: FnOnce() -> Option<usize>,
    {
        let end_idx = match self {
            SeekEnd::SkipEverything => return RawToken::EndOfStream,
            SeekEnd::Skip => return RawToken::Nothing,
            SeekEnd::Current => cur_idx,
            SeekEnd::AdvanceOne => advance(),
        }
        .unwrap_or_else(|| line.len());

        if end_idx == start_idx {
            return RawToken::Nothing;
        }

        RawToken::Value(Located::with_loc(
            Loc::new(line_no, start_idx + 1),
            &line[start_idx..end_idx],
        ))
    }
}

impl<'a> RawToken<'a> {
    const COMMENT_CHAR: char = '#';
    const NEWLINE_CHAR: char = '\n';
    const STRING_LITERAL_CHAR: char = '"';

    fn consume_one<'b>(
        line_no: usize,
        line: &'b str,
        chars: &mut std::iter::Peekable<impl Iterator<Item = (usize, char)> + 'b>,
    ) -> Result<RawToken<'b>, Located<Error>> {
        match chars.next() {
            None => Ok(RawToken::EndOfStream),
            Some((col_start, c)) => {
                let sm = SeekMode::from(c);
                loop {
                    let (idx, c) = match chars.peek().copied() {
                        Some((idx, c)) => (Some(idx), Some(c)),
                        None => (None, None),
                    };

                    if let Some(seek) = sm.should_terminate(c)? {
                        return Ok(seek.build_raw_token(line_no, line, col_start, idx, || {
                            chars.next();
                            chars.peek().copied().map(|(idx, _)| idx)
                        }));
                    }

                    chars.next();
                }
            }
        }
    }

    fn line_to_iter(
        line_no: usize,
        line: &str,
    ) -> impl Iterator<Item = Result<Located<&str>, Located<Error>>> {
        let mut chars = line.char_indices().peekable();
        std::iter::from_fn(move || -> Option<Result<Located<&str>, Located<Error>>> {
            loop {
                match RawToken::consume_one(line_no, line, &mut chars) {
                    Ok(RawToken::Nothing) => (),
                    Ok(RawToken::EndOfStream) => return None,
                    Ok(RawToken::Value(slice)) => return Some(Ok(slice)),
                    Err(err) => return Some(Err(err)),
                }
            }
        })
    }

    fn source_to_iters(
        source: &str,
    ) -> impl Iterator<Item = impl Iterator<Item = Result<Located<&str>, Located<Error>>>> {
        // NOTE If we want to support multi-line string literals, we can't just use `lines()` here.
        source
            .lines()
            .enumerate()
            .map(|(line_no, line)| Self::line_to_iter(line_no + 1, line))
    }
}

pub(super) fn tokenize_to_iters(
    source: &'_ str,
) -> impl Iterator<Item = impl Iterator<Item = Result<Located<Token>, Located<Error>>> + '_> {
    RawToken::source_to_iters(source).map(|line| line.map(|raw| raw?.try_map(Token::parse)))
}

pub fn tokenize(source: &str) -> Result<Vec<Vec<Located<Token>>>, Located<Error>> {
    tokenize_to_iters(source).map(Iterator::collect).collect()
}

#[cfg(test)]
mod tests {
    use super::super::types::{Loc, Located};
    use super::RawToken;

    #[test]
    fn consume_single_simple() {
        let line = "MOV %ra";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "MOV"))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 5), "%ra"))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_comment() {
        let line = "MOV %ra #my comment text";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "MOV"))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 5), "%ra"))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_comment_nospace() {
        let line = "MOV %ra#my comment text";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "MOV"))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 5), "%ra"))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_comment_no_chars() {
        let line = "MOV %ra#";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "MOV"))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 5), "%ra"))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_comment_one_char() {
        let line = "MOV %ra#m";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "MOV"))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 5), "%ra"))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_comment_start() {
        let line = "#";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_name_one() {
        let line = "M";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "M"))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_name_one_then_space() {
        let line = "M ";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "M"))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_name_one_space() {
        let line = " ";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_nothing() {
        let line = "";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_string() {
        let line = "\"test\"";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "\"test\""))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_string_space() {
        let line = "\"test\" ";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "\"test\""))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_multi_1() {
        let line = "\"test\" MOV #mycomment";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "\"test\""))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 8), "MOV"))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_multi_2() {
        let line = "\"test\" #MOV mycomment";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "\"test\""))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_string_empty() {
        let line = "\"\"";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "\"\""))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_string_with_spaces() {
        let line = "\"hello there friends\" \"hi there\" ";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "\"hello there friends\""))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 23), "\"hi there\""))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    fn consume_single_string_adjacent() {
        let line = "\"hello there friends\"\"hi there\" ";
        let mut line_it = &mut line.char_indices().peekable();
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 1), "\"hello there friends\""))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Value(Located::with_loc(Loc::new(0, 22), "\"hi there\""))
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::Nothing
        );
        assert_eq!(
            RawToken::consume_one(0, line, &mut line_it).unwrap(),
            RawToken::EndOfStream
        );
    }

    #[test]
    #[should_panic]
    fn consume_single_string_comment_inside() {
        let line = "\"te#st\" MOV mycomment";
        let mut line_it = &mut line.char_indices().peekable();
        drop(RawToken::consume_one(0, line, &mut line_it).unwrap());
    }

    #[test]
    #[should_panic]
    fn consume_single_string_unterminated_1() {
        let line = "\"";
        let mut line_it = &mut line.char_indices().peekable();
        drop(RawToken::consume_one(0, line, &mut line_it).unwrap());
    }

    #[test]
    #[should_panic]
    fn consume_single_string_unterminated_2() {
        let line = "\"te";
        let mut line_it = &mut line.char_indices().peekable();
        drop(RawToken::consume_one(0, line, &mut line_it).unwrap());
    }

    #[test]
    #[should_panic]
    fn consume_single_string_unterminated_3() {
        let line = "\"te#";
        let mut line_it = &mut line.char_indices().peekable();
        drop(RawToken::consume_one(0, line, &mut line_it).unwrap());
    }
}
