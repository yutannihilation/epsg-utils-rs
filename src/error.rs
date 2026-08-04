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
    InvalidJson {
        message: String,
    },
    UnknownEpsgCode {
        code: i32,
    },
    /// A WKT1 node that carries information not representable in [`crate::Crs`]
    /// (e.g. `TOWGS84`, `EXTENSION`). Returned by [`crate::parse_wkt1`];
    /// [`crate::parse_wkt1_lossy`] discards such nodes instead.
    LossyWkt1Node {
        keyword: String,
        pos: usize,
    },
    /// A WKT1 construct that this crate does not support at all
    /// (e.g. `LOCAL_CS`, `FITTED_CS`, a 3-axis `GEOGCS`).
    UnsupportedWkt1Node {
        keyword: String,
        pos: usize,
    },
    /// A `PROJECTION` name that is not in the WKT1 method mapping tables.
    UnknownProjectionMethod {
        name: String,
    },
    /// A `PARAMETER` name that the matched projection method does not define.
    UnknownParameter {
        method: String,
        name: String,
    },
    /// A `PARAMETER` whose name is known but whose value no interpretation of
    /// the projection method accepts.
    UnsupportedParameterValue {
        method: String,
        name: String,
        value: f64,
    },
    /// A WKT1 node that this crate requires in order to build a complete
    /// [`crate::Crs`] is absent (e.g. `UNIT`, which OGC 01-009 mandates).
    MissingWkt1Node {
        keyword: String,
        parent: String,
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
                write!(f, "unexpected keyword '{keyword}' at position {pos}")
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
            ParseError::InvalidJson { message } => {
                write!(f, "invalid PROJJSON: {message}")
            }
            ParseError::UnknownEpsgCode { code } => {
                write!(f, "unknown EPSG code: {code}")
            }
            ParseError::LossyWkt1Node { keyword, pos } => {
                write!(
                    f,
                    "WKT1 node '{keyword}' at position {pos} cannot be represented without data loss (use parse_wkt1_lossy to discard it)"
                )
            }
            ParseError::UnsupportedWkt1Node { keyword, pos } => {
                write!(
                    f,
                    "unsupported WKT1 construct '{keyword}' at position {pos}"
                )
            }
            ParseError::UnknownProjectionMethod { name } => {
                write!(f, "unknown WKT1 projection method: '{name}'")
            }
            ParseError::UnsupportedParameterValue {
                method,
                name,
                value,
            } => {
                write!(
                    f,
                    "unsupported value {value} for parameter '{name}' of projection method '{method}'"
                )
            }
            ParseError::MissingWkt1Node {
                keyword,
                parent,
                pos,
            } => {
                write!(
                    f,
                    "required WKT1 node '{keyword}' is missing from '{parent}' at position {pos}"
                )
            }
            ParseError::UnknownParameter { method, name } => {
                write!(
                    f,
                    "unknown parameter '{name}' for projection method '{method}'"
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}
