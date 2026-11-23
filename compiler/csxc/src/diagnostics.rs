use std::{iter::Peekable, path::Path, str::Chars};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

impl Position {
    pub const START: Self = Self { line: 1, column: 1 };

    pub fn annotate(self, error: impl Into<String>) -> String {
        let error = error.into();
        if error.starts_with("@position:") {
            return error;
        }
        format!("@position:{},{}|{error}", self.line, self.column)
    }
}

pub fn split_position(error: &str) -> (Option<Position>, &str) {
    if let Some((position, message)) = error
        .strip_prefix("@position:")
        .and_then(|text| text.split_once('|'))
    {
        if let Some((line, column)) = position.split_once(',') {
            if let (Ok(line), Ok(column)) = (line.parse(), column.parse()) {
                return (Some(Position { line, column }), message);
            }
        }
    }
    (None, error)
}

pub fn visual_studio(path: &Path, error: &str) -> String {
    let (position, message) = split_position(error);
    let (code, message) = message
        .split_once(": ")
        .filter(|(code, _)| {
            code.starts_with("CSX") && code[3..].bytes().all(|b| b.is_ascii_digit())
        })
        .unwrap_or(("CSX1000", message));
    match position {
        Some(position) => format!(
            "{}({},{}): error {code}: {message}",
            path.display(),
            position.line,
            position.column
        ),
        None => format!("{}: error {code}: {message}", path.display()),
    }
}

pub struct SourceChars<'a> {
    chars: Peekable<Chars<'a>>,
    pub position: Position,
}

impl<'a> SourceChars<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            position: Position::START,
        }
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }
}

impl Iterator for SourceChars<'_> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let ch = self.chars.next()?;
        if ch == '\n' {
            self.position.line += 1;
            self.position.column = 1;
        } else {
            self.position.column += ch.len_utf16() as u32;
        }
        Some(ch)
    }
}
