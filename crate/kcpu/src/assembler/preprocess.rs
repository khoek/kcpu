const COMMENT_CHAR: char = '#';
const NEWLINE_CHAR: char = '\n';

#[derive(Debug)]
pub(super) enum Error {}

fn strip_comments<'a>(raw: &str) -> String {
    let mut s = String::new();

    let mut in_comment = false;
    for c in raw.chars() {
        in_comment = match c {
            COMMENT_CHAR => true,
            NEWLINE_CHAR => false,
            _ => in_comment,
        };

        if !in_comment {
            s.push(c);
        }
    }

    s
}

pub(super) fn preprocess(source: &str) -> Result<String, Error> {
    Ok(strip_comments(source))
}
