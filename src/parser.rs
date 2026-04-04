use crate::error::ParseError;
use crate::wkt2::{BaseGeodeticCrs, BaseGeodeticCrsKeyword, ProjectedCrs};

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
        let base_geodetic_crs = self.parse_base_geodetic_crs()?;

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

    fn parse_base_geodetic_crs(&mut self) -> Result<BaseGeodeticCrs, ParseError> {
        let keyword_str = self.parse_keyword()?;
        let keyword = match keyword_str.as_str() {
            "BASEGEODCRS" => BaseGeodeticCrsKeyword::BaseGeodCrs,
            "BASEGEOGCRS" => BaseGeodeticCrsKeyword::BaseGeogCrs,
            _ => {
                return Err(ParseError::ExpectedKeyword {
                    pos: self.pos - keyword_str.len(),
                });
            }
        };

        self.skip_whitespace();
        self.expect_char('[')?;

        // <base crs name>
        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // Next is either DYNAMIC[...] (dynamic CRS) or DATUM[...]/ENSEMBLE[...] (static CRS)
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();

        let peeked = self.peek_keyword();
        let dynamic = if peeked.as_deref() == Some("DYNAMIC") {
            let d = self.parse_bracketed_node()?;
            self.skip_whitespace();
            self.expect_char(',')?;
            self.skip_whitespace();
            Some(d)
        } else {
            None
        };

        // <geodetic reference frame> or <geodetic datum ensemble>
        let datum = self.parse_bracketed_node()?;

        // Optional components: ellipsoidal CS unit and identifiers
        let mut ellipsoidal_cs_unit = None;
        let mut identifiers = Vec::new();

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();

            let peeked = self.peek_keyword();
            match peeked.as_deref() {
                Some("ANGLEUNIT") => {
                    ellipsoidal_cs_unit = Some(self.parse_bracketed_node()?);
                }
                Some("ID") => {
                    identifiers.push(self.parse_bracketed_node()?);
                }
                _ => {
                    // Unknown optional node, consume it
                    identifiers.push(self.parse_bracketed_node()?);
                }
            }
        }

        self.expect_char(']')?;

        Ok(BaseGeodeticCrs {
            keyword,
            name,
            dynamic,
            datum,
            ellipsoidal_cs_unit,
            identifiers,
        })
    }

    /// Peek at the next keyword without advancing the position.
    fn peek_keyword(&self) -> Option<String> {
        let mut pos = self.pos;
        let start = pos;
        while pos < self.input.len() && self.input.as_bytes()[pos].is_ascii_uppercase() {
            pos += 1;
        }
        if pos == start {
            None
        } else {
            Some(self.input[start..pos].to_string())
        }
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
    use crate::wkt2::BaseGeodeticCrsKeyword;

    #[test]
    fn parse_static_geogcrs() {
        let wkt = r#"PROJCRS["WGS 84 / UTM zone 31N",
            BASEGEOGCRS["WGS 84", DATUM["World Geodetic System 1984", ELLIPSOID["WGS 84",6378137,298.257223563]]],
            CONVERSION["UTM zone 31N", METHOD["Transverse Mercator"]],
            CS[Cartesian, 2, AXIS["easting", east], AXIS["northing", north]]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        assert_eq!(result.name, "WGS 84 / UTM zone 31N");
        let base = &result.base_geodetic_crs;
        assert_eq!(base.keyword, BaseGeodeticCrsKeyword::BaseGeogCrs);
        assert_eq!(base.name, "WGS 84");
        assert!(base.dynamic.is_none());
        assert!(base.datum.starts_with("DATUM["));
        assert!(base.ellipsoidal_cs_unit.is_none());
        assert!(base.identifiers.is_empty());
    }

    #[test]
    fn parse_dynamic_geodcrs() {
        let wkt = r#"PROJCRS["test",
            BASEGEODCRS["WGS 84",
                DYNAMIC[FRAMEEPOCH[2010.0]],
                DATUM["World Geodetic System 1984", ELLIPSOID["WGS 84",6378137,298.257223563]]],
            CONVERSION["y"],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let base = &result.base_geodetic_crs;
        assert_eq!(base.keyword, BaseGeodeticCrsKeyword::BaseGeodCrs);
        assert!(base.dynamic.is_some());
        assert!(base.dynamic.as_ref().unwrap().starts_with("DYNAMIC["));
        assert!(base.datum.starts_with("DATUM["));
    }

    #[test]
    fn parse_base_crs_with_unit_and_id() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["WGS 84",
                DATUM["WGS 1984", ELLIPSOID["WGS 84",6378137,298.257223563]],
                ANGLEUNIT["degree", 0.0174532925199433],
                ID["EPSG", 4326]],
            CONVERSION["y"],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let base = &result.base_geodetic_crs;
        assert!(base.ellipsoidal_cs_unit.is_some());
        assert!(
            base.ellipsoidal_cs_unit
                .as_ref()
                .unwrap()
                .starts_with("ANGLEUNIT[")
        );
        assert_eq!(base.identifiers.len(), 1);
        assert!(base.identifiers[0].starts_with("ID["));
    }

    #[test]
    fn parse_base_crs_with_ensemble() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["WGS 84", ENSEMBLE["World Geodetic System 1984 ensemble", MEMBER["WGS 84 (G730)"], ELLIPSOID["WGS 84",6378137,298.257223563]]],
            CONVERSION["y"],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let base = &result.base_geodetic_crs;
        assert!(base.datum.starts_with("ENSEMBLE["));
    }

    #[test]
    fn parse_projcrs_with_trailing_nodes() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["x", DATUM["d"]],
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
        let wkt = r#"PROJECTEDCRS["test", BASEGEOGCRS["x", DATUM["d"]], CONVERSION["y"], CS[Cartesian, 2]]"#;
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
        let wkt = r#"PROJCRS["test", BASEGEOGCRS["x", DATUM["d"]], CONVERSION["y"], CS[Cartesian, 2]] extra"#;
        let mut parser = Parser::new(wkt);
        let err = parser.parse_projected_crs().unwrap_err();
        assert!(matches!(err, ParseError::TrailingInput { .. }));
    }
}
