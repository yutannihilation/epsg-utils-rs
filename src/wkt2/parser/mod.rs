mod crs;
mod cs;
mod datum;
mod metadata;
mod projection;
mod unit;

#[cfg(test)]
mod tests;

use crate::crs::Identifier;
use crate::error::ParseError;

/// Which keyword character set the parser accepts.
///
/// WKT2 keywords are uppercase letters only; WKT1 keywords may also contain
/// digits and underscores (`VERT_CS`, `COMPD_CS`, `TOWGS84`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Dialect {
    Wkt2,
    Wkt1,
}

impl Dialect {
    /// Whether `b` may continue a keyword (the first byte must always be an
    /// uppercase letter, so that numbers are never read as keywords).
    fn continues_keyword(self, b: u8) -> bool {
        match self {
            Dialect::Wkt2 => b.is_ascii_uppercase(),
            Dialect::Wkt1 => b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_',
        }
    }
}

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
    dialect: Dialect,
}

// ---------------------------------------------------------------------------
// Core primitives
// ---------------------------------------------------------------------------

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            dialect: Dialect::Wkt2,
        }
    }

    /// A parser that accepts WKT1 keywords (see [`Dialect`]).
    pub(crate) fn new_wkt1(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            dialect: Dialect::Wkt1,
        }
    }

    /// Current byte offset into the input (for error reporting).
    pub(crate) fn pos(&self) -> usize {
        self.pos
    }

    pub(crate) fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    pub(crate) fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Peek at the next keyword without advancing the position.
    ///
    /// A keyword starts with an uppercase letter and continues with the
    /// bytes the current [`Dialect`] allows.
    pub(crate) fn peek_keyword(&self) -> Option<&'a str> {
        let bytes = self.input.as_bytes();
        if !bytes.get(self.pos)?.is_ascii_uppercase() {
            return None;
        }
        let mut pos = self.pos + 1;
        while pos < bytes.len() && self.dialect.continues_keyword(bytes[pos]) {
            pos += 1;
        }
        Some(&self.input[self.pos..pos])
    }

    pub(crate) fn expect_char(&mut self, expected: char) -> Result<(), ParseError> {
        match self.peek_char() {
            Some(c) if c == expected => {
                self.pos += c.len_utf8();
                Ok(())
            }
            Some(c) => Err(ParseError::UnexpectedChar {
                expected,
                found: c,
                pos: self.pos,
            }),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    /// Consume and return a keyword.
    pub(crate) fn parse_keyword(&mut self) -> Result<&'a str, ParseError> {
        let kw = self
            .peek_keyword()
            .ok_or(ParseError::ExpectedKeyword { pos: self.pos })?;
        self.pos += kw.len();
        Ok(kw)
    }

    /// Parse an unquoted identifier (mixed case, alphabetic).
    pub(crate) fn parse_identifier(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_alphabetic() {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(ParseError::ExpectedKeyword { pos: start });
        }
        Ok(self.input[start..self.pos].to_string())
    }

    pub(crate) fn parse_quoted_string(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        self.expect_char('"')?;
        let content_start = self.pos;
        while self.pos < self.input.len() {
            if self.input.as_bytes()[self.pos] == b'"' {
                let s = self.input[content_start..self.pos].to_string();
                self.pos += 1;
                return Ok(s);
            }
            self.pos += 1;
        }
        Err(ParseError::UnterminatedString { pos: start })
    }

    pub(crate) fn parse_number(&mut self) -> Result<f64, ParseError> {
        let start = self.pos;
        if self.pos < self.input.len()
            && (self.input.as_bytes()[self.pos] == b'-' || self.input.as_bytes()[self.pos] == b'+')
        {
            self.pos += 1;
        }
        while self.pos < self.input.len()
            && (self.input.as_bytes()[self.pos].is_ascii_digit()
                || self.input.as_bytes()[self.pos] == b'.')
        {
            self.pos += 1;
        }
        if self.pos < self.input.len()
            && (self.input.as_bytes()[self.pos] == b'e' || self.input.as_bytes()[self.pos] == b'E')
        {
            self.pos += 1;
            if self.pos < self.input.len()
                && (self.input.as_bytes()[self.pos] == b'-'
                    || self.input.as_bytes()[self.pos] == b'+')
            {
                self.pos += 1;
            }
            while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        if self.pos == start {
            return Err(ParseError::UnexpectedEnd);
        }
        self.input[start..self.pos]
            .parse::<f64>()
            .map_err(|_| ParseError::UnexpectedEnd)
    }

    /// Consume a `KEYWORD[...]` node without interpreting it, returning the
    /// raw text so callers can preserve the original spelling.
    pub(crate) fn skip_node(&mut self) -> Result<&'a str, ParseError> {
        let start = self.pos;
        self.parse_keyword()?;
        self.skip_whitespace();
        self.expect_char('[')?;

        let bytes = self.input.as_bytes();
        let mut depth = 1u32;
        while depth > 0 {
            if self.pos >= bytes.len() {
                return Err(ParseError::UnexpectedEnd);
            }
            match bytes[self.pos] {
                b'[' => depth += 1,
                b']' => depth -= 1,
                b'"' => {
                    // Skip the quoted string; an unterminated one must not
                    // advance `pos` past the end of the input.
                    self.pos += 1;
                    while self.pos < bytes.len() && bytes[self.pos] != b'"' {
                        self.pos += 1;
                    }
                    if self.pos >= bytes.len() {
                        return Err(ParseError::UnterminatedString { pos: start });
                    }
                }
                _ => {}
            }
            self.pos += 1;
        }
        Ok(&self.input[start..self.pos])
    }

    /// Parse a KEYWORD[...] node as a raw string, preserving the original text.
    pub(crate) fn parse_bracketed_node(&mut self) -> Result<String, ParseError> {
        self.skip_node().map(str::to_string)
    }

    /// Parse a datetime (unquoted, like 2013-01-01) or quoted text.
    pub(crate) fn parse_datetime_or_text(&mut self) -> Result<String, ParseError> {
        if self.peek_char() == Some('"') {
            self.parse_quoted_string()
        } else {
            let start = self.pos;
            while self.pos < self.input.len() {
                let ch = self.input.as_bytes()[self.pos];
                if ch == b',' || ch == b']' {
                    break;
                }
                self.pos += 1;
            }
            if self.pos == start {
                return Err(ParseError::UnexpectedEnd);
            }
            Ok(self.input[start..self.pos].trim().to_string())
        }
    }

    /// Parse a value that is either a number or a quoted string.
    pub(crate) fn parse_number_or_text(&mut self) -> Result<crate::crs::AuthorityId, ParseError> {
        if self.peek_char() == Some('"') {
            Ok(crate::crs::AuthorityId::Text(self.parse_quoted_string()?))
        } else {
            Ok(crate::crs::AuthorityId::Number(self.parse_number()?))
        }
    }
}

// ---------------------------------------------------------------------------
// Combinators
// ---------------------------------------------------------------------------

impl<'a> Parser<'a> {
    /// Expect one of the given keywords, then `[`, run `body`, then `]`.
    /// Returns `(matched_keyword, body_result)`.
    pub(crate) fn bracketed<T>(
        &mut self,
        keywords: &[&str],
        body: impl FnOnce(&mut Self) -> Result<T, ParseError>,
    ) -> Result<(&'a str, T), ParseError> {
        let kw = self.parse_keyword()?;
        if !keywords.contains(&kw) {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - kw.len(),
            });
        }
        self.skip_whitespace();
        self.expect_char('[')?;
        self.skip_whitespace();
        let result = body(self)?;
        self.skip_whitespace();
        self.expect_char(']')?;
        Ok((kw, result))
    }

    /// Skip whitespace, expect `,`, skip whitespace, then run `f`.
    pub(crate) fn comma_then<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, ParseError>,
    ) -> Result<T, ParseError> {
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        f(self)
    }

    /// Loop over comma-separated items until `]`. For each item, peeks the
    /// keyword and calls `handler`. The handler should consume the item.
    pub(crate) fn trailing_items(
        &mut self,
        mut handler: impl FnMut(&mut Self, &str) -> Result<(), ParseError>,
    ) -> Result<(), ParseError> {
        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();

            let kw = self.peek_keyword().unwrap_or_default();
            handler(self, kw)?;
        }
        Ok(())
    }

    /// Parse trailing comma-separated ID[...] nodes until `]`.
    pub(crate) fn trailing_identifiers(&mut self) -> Result<Vec<Identifier>, ParseError> {
        let mut identifiers = Vec::new();
        self.trailing_items(|p, _kw| {
            identifiers.push(p.parse_identifier_node()?);
            Ok(())
        })?;
        Ok(identifiers)
    }

    /// Parse `KEYWORD["text"]`.
    pub(crate) fn parse_keyword_quoted_string(
        &mut self,
        expected: &str,
    ) -> Result<String, ParseError> {
        self.bracketed(&[expected], |p| p.parse_quoted_string())
            .map(|(_, s)| s)
    }

    /// Parse `KEYWORD[number]`.
    pub(crate) fn parse_keyword_number(&mut self, expected: &str) -> Result<f64, ParseError> {
        self.bracketed(&[expected], |p| p.parse_number())
            .map(|(_, n)| n)
    }
}
