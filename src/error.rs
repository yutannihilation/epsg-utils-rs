#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedEnd,
    UnexpectedChar {
        expected: char,
        found: char,
        pos: usize,
    },
    UnexpectedKeyword {
        keyword: String,
        pos: usize,
    },
    ExpectedKeyword {
        pos: usize,
    },
    UnterminatedString {
        pos: usize,
    },
    TrailingInput {
        pos: usize,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedEnd => write!(f, "unexpected end of input"),
            ParseError::UnexpectedChar {
                expected,
                found,
                pos,
            } => {
                write!(
                    f,
                    "expected '{expected}', found '{found}' at position {pos}"
                )
            }
            ParseError::UnexpectedKeyword { keyword, pos } => {
                write!(
                    f,
                    "unexpected keyword '{keyword}' at position {pos} (hint: use PROJCRS instead of PROJECTEDCRS)"
                )
            }
            ParseError::ExpectedKeyword { pos } => {
                write!(f, "expected keyword at position {pos}")
            }
            ParseError::UnterminatedString { pos } => {
                write!(f, "unterminated string starting at position {pos}")
            }
            ParseError::TrailingInput { pos } => {
                write!(f, "trailing input at position {pos}")
            }
        }
    }
}

impl std::error::Error for ParseError {}
