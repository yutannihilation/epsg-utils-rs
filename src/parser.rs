use crate::error::ParseError;
use crate::wkt2::ProjectedCrs;

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn parse_projected_crs(&mut self) -> Result<ProjectedCrs, ParseError> {
        self.skip_whitespace();
        let keyword = self.parse_keyword()?;
        match keyword.as_str() {
            "PROJCRS" => {}
            "PROJECTEDCRS" => {
                return Err(ParseError::UnexpectedKeyword {
                    keyword,
                    pos: self.pos - "PROJECTEDCRS".len(),
                });
            }
            _ => {
                return Err(ParseError::ExpectedKeyword {
                    pos: self.pos - keyword.len(),
                });
            }
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        // <crs name>
        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // <base geodetic crs>
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let base_geodetic_crs = self.parse_bracketed_node()?;

        // <map projection>
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let map_projection = self.parse_bracketed_node()?;

        // <coordinate system>
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let coordinate_system = self.parse_bracketed_node()?;

        // <scope extent identifier remark> — zero or more comma-separated nodes
        let mut scope_extent_identifier_remark = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();
            let node = self.parse_bracketed_node()?;
            scope_extent_identifier_remark.push(node);
        }

        self.expect_char(']')?;

        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err(ParseError::TrailingInput { pos: self.pos });
        }

        Ok(ProjectedCrs {
            name,
            base_geodetic_crs,
            map_projection,
            coordinate_system,
            scope_extent_identifier_remark,
        })
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn expect_char(&mut self, expected: char) -> Result<(), ParseError> {
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

    fn parse_keyword(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_uppercase() {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(ParseError::ExpectedKeyword { pos: start });
        }
        Ok(self.input[start..self.pos].to_string())
    }

    fn parse_quoted_string(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        self.expect_char('"')?;
        let content_start = self.pos;
        while self.pos < self.input.len() {
            if self.input.as_bytes()[self.pos] == b'"' {
                let s = self.input[content_start..self.pos].to_string();
                self.pos += 1; // consume closing quote
                return Ok(s);
            }
            self.pos += 1;
        }
        Err(ParseError::UnterminatedString { pos: start })
    }

    /// Parse a KEYWORD[...] node as a raw string, preserving the original text.
    fn parse_bracketed_node(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        self.parse_keyword()?;
        self.skip_whitespace();
        self.expect_char('[')?;
        let mut depth = 1u32;
        while self.pos < self.input.len() && depth > 0 {
            match self.input.as_bytes()[self.pos] {
                b'[' => depth += 1,
                b']' => depth -= 1,
                b'"' => {
                    // skip over quoted strings so brackets inside them don't count
                    self.pos += 1;
                    while self.pos < self.input.len() && self.input.as_bytes()[self.pos] != b'"' {
                        self.pos += 1;
                    }
                }
                _ => {}
            }
            self.pos += 1;
        }
        if depth != 0 {
            return Err(ParseError::UnexpectedEnd);
        }
        Ok(self.input[start..self.pos].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_projcrs() {
        let wkt = r#"PROJCRS["WGS 84 / UTM zone 31N",
            BASEGEOGCRS["WGS 84", DATUM["World Geodetic System 1984", ELLIPSOID["WGS 84",6378137,298.257223563]]],
            CONVERSION["UTM zone 31N", METHOD["Transverse Mercator"]],
            CS[Cartesian, 2, AXIS["easting", east], AXIS["northing", north]]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        assert_eq!(result.name, "WGS 84 / UTM zone 31N");
        assert!(result.base_geodetic_crs.starts_with("BASEGEOGCRS["));
        assert!(result.map_projection.starts_with("CONVERSION["));
        assert!(result.coordinate_system.starts_with("CS["));
        assert!(result.scope_extent_identifier_remark.is_empty());
    }

    #[test]
    fn parse_projcrs_with_trailing_nodes() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["x"],
            CONVERSION["y"],
            CS[Cartesian, 2],
            ID["EPSG", 32631]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        assert_eq!(result.scope_extent_identifier_remark.len(), 1);
        assert!(result.scope_extent_identifier_remark[0].starts_with("ID["));
    }

    #[test]
    fn reject_projectedcrs() {
        let wkt = r#"PROJECTEDCRS["test", BASEGEOGCRS["x"], CONVERSION["y"], CS[Cartesian, 2]]"#;
        let mut parser = Parser::new(wkt);
        let err = parser.parse_projected_crs().unwrap_err();
        assert!(matches!(err, ParseError::UnexpectedKeyword { .. }));
        assert!(err.to_string().contains("PROJECTEDCRS"));
    }

    #[test]
    fn reject_wrong_keyword() {
        let wkt = r#"GEOGCRS["test"]"#;
        let mut parser = Parser::new(wkt);
        let err = parser.parse_projected_crs().unwrap_err();
        assert!(matches!(err, ParseError::ExpectedKeyword { .. }));
    }

    #[test]
    fn trailing_input_error() {
        let wkt = r#"PROJCRS["test", BASEGEOGCRS["x"], CONVERSION["y"], CS[Cartesian, 2]] extra"#;
        let mut parser = Parser::new(wkt);
        let err = parser.parse_projected_crs().unwrap_err();
        assert!(matches!(err, ParseError::TrailingInput { .. }));
    }
}
