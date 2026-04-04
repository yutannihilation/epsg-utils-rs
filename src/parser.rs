use crate::error::ParseError;
use crate::wkt2::{
    Axis, BaseGeodeticCrs, BaseGeodeticCrsKeyword, CoordinateSystem, CsType, Datum, DatumEnsemble,
    DatumKeyword, DeformationModel, DynamicCrs, Ellipsoid, EnsembleMember,
    GeodeticReferenceFrame, MapProjection, MapProjectionMethod, MapProjectionParameter,
    ProjectedCrs,
};

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
        let map_projection = self.parse_map_projection()?;

        // <coordinate system> header: CS[type, dimension, identifiers...]
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let (cs_type, dimension, cs_identifiers) = self.parse_cs_header()?;

        // Axes, cs unit, and scope/extent/identifier/remark are all
        // comma-separated siblings at the PROJCRS level
        let mut axes = Vec::new();
        let mut cs_unit = None;
        let mut scope_extent_identifier_remark = Vec::new();

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();

            match self.peek_keyword().as_deref() {
                Some("AXIS") => {
                    axes.push(self.parse_axis()?);
                }
                Some("LENGTHUNIT" | "ANGLEUNIT" | "SCALEUNIT" | "PARAMETRICUNIT" | "TIMEUNIT") => {
                    cs_unit = Some(self.parse_bracketed_node()?);
                }
                _ => {
                    scope_extent_identifier_remark.push(self.parse_bracketed_node()?);
                }
            }
        }

        let coordinate_system = CoordinateSystem {
            cs_type,
            dimension,
            identifiers: cs_identifiers,
            axes,
            cs_unit,
        };

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

    fn parse_map_projection(&mut self) -> Result<MapProjection, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "CONVERSION" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        // <map projection name>
        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // <map projection method>
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let method = self.parse_map_projection_method()?;

        // Optional parameters and identifiers
        let mut parameters = Vec::new();
        let mut identifiers = Vec::new();

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();

            match self.peek_keyword().as_deref() {
                Some("PARAMETER") => {
                    parameters.push(self.parse_map_projection_parameter()?);
                }
                Some("ID") => {
                    identifiers.push(self.parse_bracketed_node()?);
                }
                _ => {
                    identifiers.push(self.parse_bracketed_node()?);
                }
            }
        }

        self.expect_char(']')?;

        Ok(MapProjection {
            name,
            method,
            parameters,
            identifiers,
        })
    }

    fn parse_map_projection_method(&mut self) -> Result<MapProjectionMethod, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "METHOD" && keyword != "PROJECTION" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        let mut identifiers = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();
            identifiers.push(self.parse_bracketed_node()?);
        }

        self.expect_char(']')?;

        Ok(MapProjectionMethod { name, identifiers })
    }

    fn parse_map_projection_parameter(&mut self) -> Result<MapProjectionParameter, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "PARAMETER" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        // <parameter name>
        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // <parameter value>
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let value = self.parse_number()?;

        // Optional unit and identifiers
        let mut unit = None;
        let mut identifiers = Vec::new();

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();

            match self.peek_keyword().as_deref() {
                Some("ANGLEUNIT" | "LENGTHUNIT" | "SCALEUNIT") => {
                    unit = Some(self.parse_bracketed_node()?);
                }
                Some("ID") => {
                    identifiers.push(self.parse_bracketed_node()?);
                }
                _ => {
                    identifiers.push(self.parse_bracketed_node()?);
                }
            }
        }

        self.expect_char(']')?;

        Ok(MapProjectionParameter {
            name,
            value,
            unit,
            identifiers,
        })
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        let start = self.pos;
        // optional sign
        if self.pos < self.input.len()
            && (self.input.as_bytes()[self.pos] == b'-' || self.input.as_bytes()[self.pos] == b'+')
        {
            self.pos += 1;
        }
        // digits, optional decimal point, more digits
        while self.pos < self.input.len()
            && (self.input.as_bytes()[self.pos].is_ascii_digit()
                || self.input.as_bytes()[self.pos] == b'.')
        {
            self.pos += 1;
        }
        // optional exponent
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

    fn parse_cs_header(&mut self) -> Result<(CsType, u8, Vec<String>), ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "CS" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        // <cs type> — unquoted identifier like "Cartesian", "ellipsoidal"
        self.skip_whitespace();
        let type_str = self.parse_identifier()?;
        let cs_type = match type_str.as_str() {
            "affine" => CsType::Affine,
            "Cartesian" => CsType::Cartesian,
            "cylindrical" => CsType::Cylindrical,
            "ellipsoidal" => CsType::Ellipsoidal,
            "linear" => CsType::Linear,
            "parametric" => CsType::Parametric,
            "polar" => CsType::Polar,
            "spherical" => CsType::Spherical,
            "vertical" => CsType::Vertical,
            "temporalCount" => CsType::TemporalCount,
            "temporalMeasure" => CsType::TemporalMeasure,
            "ordinal" => CsType::Ordinal,
            "temporalDateTime" => CsType::TemporalDateTime,
            _ => {
                let len = type_str.len();
                return Err(ParseError::UnexpectedKeyword {
                    keyword: type_str,
                    pos: self.pos - len,
                });
            }
        };

        // <dimension>
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let dim = self.parse_number()?;
        let dimension = dim as u8;

        // Optional identifiers
        let mut identifiers = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();
            identifiers.push(self.parse_bracketed_node()?);
        }

        self.expect_char(']')?;

        Ok((cs_type, dimension, identifiers))
    }

    fn parse_axis(&mut self) -> Result<Axis, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "AXIS" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        // <axis nameAbbrev>
        self.skip_whitespace();
        let name_abbrev = self.parse_quoted_string()?;

        // <axis direction>
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let direction = self.parse_identifier()?;

        // Optional components
        let mut meridian = None;
        let mut bearing = None;
        let mut order = None;
        let mut unit = None;
        let mut identifiers = Vec::new();

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();

            match self.peek_keyword().as_deref() {
                Some("MERIDIAN") => {
                    meridian = Some(self.parse_bracketed_node()?);
                }
                Some("BEARING") => {
                    bearing = Some(self.parse_bracketed_node()?);
                }
                Some("ORDER") => {
                    order = Some(self.parse_order()?);
                }
                Some("LENGTHUNIT" | "ANGLEUNIT" | "SCALEUNIT" | "PARAMETRICUNIT" | "TIMEUNIT") => {
                    unit = Some(self.parse_bracketed_node()?);
                }
                Some("ID") => {
                    identifiers.push(self.parse_bracketed_node()?);
                }
                _ => {
                    // AXISMINVALUE, AXISMAXVALUE, RANGEMEANING, or unknown
                    self.parse_bracketed_node()?;
                }
            }
        }

        self.expect_char(']')?;

        Ok(Axis {
            name_abbrev,
            direction,
            meridian,
            bearing,
            order,
            unit,
            identifiers,
        })
    }

    fn parse_order(&mut self) -> Result<u32, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "ORDER" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }
        self.skip_whitespace();
        self.expect_char('[')?;
        self.skip_whitespace();
        let value = self.parse_number()? as u32;
        self.skip_whitespace();
        self.expect_char(']')?;
        Ok(value)
    }

    /// Parse an unquoted identifier (mixed case, alphabetic).
    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_alphabetic() {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(ParseError::ExpectedKeyword { pos: start });
        }
        Ok(self.input[start..self.pos].to_string())
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

        let dynamic = if self.peek_keyword().as_deref() == Some("DYNAMIC") {
            let d = self.parse_dynamic_crs()?;
            self.skip_whitespace();
            self.expect_char(',')?;
            self.skip_whitespace();
            Some(d)
        } else {
            None
        };

        // <geodetic reference frame> or <geodetic datum ensemble>
        let mut datum = match self.peek_keyword().as_deref() {
            Some("ENSEMBLE") => Datum::Ensemble(self.parse_datum_ensemble()?),
            _ => Datum::ReferenceFrame(self.parse_geodetic_reference_frame()?),
        };

        // Prime meridian and other optional components are siblings after DATUM/ENSEMBLE[...]
        let mut ellipsoidal_cs_unit = None;
        let mut identifiers = Vec::new();

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();

            match self.peek_keyword().as_deref() {
                Some("PRIMEM") => {
                    let pm = self.parse_bracketed_node()?;
                    match &mut datum {
                        Datum::ReferenceFrame(rf) => rf.prime_meridian = Some(pm),
                        Datum::Ensemble(ens) => ens.prime_meridian = Some(pm),
                    }
                }
                Some("ANGLEUNIT") => {
                    ellipsoidal_cs_unit = Some(self.parse_bracketed_node()?);
                }
                Some("ID") => {
                    identifiers.push(self.parse_bracketed_node()?);
                }
                _ => {
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

    fn parse_dynamic_crs(&mut self) -> Result<DynamicCrs, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "DYNAMIC" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        // FRAMEEPOCH[epoch]
        self.skip_whitespace();
        let fe_keyword = self.parse_keyword()?;
        if fe_keyword != "FRAMEEPOCH" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - fe_keyword.len(),
            });
        }
        self.skip_whitespace();
        self.expect_char('[')?;
        self.skip_whitespace();
        let frame_reference_epoch = self.parse_number()?;
        self.skip_whitespace();
        self.expect_char(']')?;

        // Optional deformation model
        let mut deformation_model = None;
        self.skip_whitespace();
        if self.peek_char() != Some(']') {
            self.expect_char(',')?;
            self.skip_whitespace();

            let model_keyword = self.parse_keyword()?;
            if model_keyword != "MODEL" && model_keyword != "VELOCITYGRID" {
                return Err(ParseError::ExpectedKeyword {
                    pos: self.pos - model_keyword.len(),
                });
            }
            self.skip_whitespace();
            self.expect_char('[')?;
            self.skip_whitespace();
            let name = self.parse_quoted_string()?;

            let mut identifiers = Vec::new();
            loop {
                self.skip_whitespace();
                if self.peek_char() == Some(']') {
                    break;
                }
                self.expect_char(',')?;
                self.skip_whitespace();
                identifiers.push(self.parse_bracketed_node()?);
            }
            self.expect_char(']')?;

            deformation_model = Some(DeformationModel { name, identifiers });
        }

        self.skip_whitespace();
        self.expect_char(']')?;

        Ok(DynamicCrs {
            frame_reference_epoch,
            deformation_model,
        })
    }

    fn parse_datum_ensemble(&mut self) -> Result<DatumEnsemble, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "ENSEMBLE" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        // <datum ensemble name>
        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // Members, ellipsoid, accuracy, identifiers
        let mut members = Vec::new();
        let mut ellipsoid = None;
        let mut accuracy = None;
        let mut identifiers = Vec::new();

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();

            match self.peek_keyword().as_deref() {
                Some("MEMBER") => {
                    members.push(self.parse_ensemble_member()?);
                }
                Some("ELLIPSOID" | "SPHEROID") => {
                    ellipsoid = Some(self.parse_ellipsoid()?);
                }
                Some("ENSEMBLEACCURACY") => {
                    self.parse_keyword()?;
                    self.skip_whitespace();
                    self.expect_char('[')?;
                    self.skip_whitespace();
                    accuracy = Some(self.parse_number()?);
                    self.skip_whitespace();
                    self.expect_char(']')?;
                }
                Some("ID") => {
                    identifiers.push(self.parse_bracketed_node()?);
                }
                _ => {
                    identifiers.push(self.parse_bracketed_node()?);
                }
            }
        }

        self.expect_char(']')?;

        let accuracy = accuracy.ok_or(ParseError::UnexpectedEnd)?;

        Ok(DatumEnsemble {
            name,
            members,
            ellipsoid,
            accuracy,
            identifiers,
            prime_meridian: None, // filled in by caller
        })
    }

    fn parse_ensemble_member(&mut self) -> Result<EnsembleMember, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "MEMBER" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        let mut identifiers = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();
            identifiers.push(self.parse_bracketed_node()?);
        }

        self.expect_char(']')?;

        Ok(EnsembleMember { name, identifiers })
    }

    fn parse_geodetic_reference_frame(&mut self) -> Result<GeodeticReferenceFrame, ParseError> {
        let keyword_str = self.parse_keyword()?;
        let keyword = match keyword_str.as_str() {
            "DATUM" => DatumKeyword::Datum,
            "TRF" => DatumKeyword::Trf,
            "GEODETICDATUM" => DatumKeyword::GeodeticDatum,
            _ => {
                return Err(ParseError::ExpectedKeyword {
                    pos: self.pos - keyword_str.len(),
                });
            }
        };

        self.skip_whitespace();
        self.expect_char('[')?;

        // <datum name>
        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // <ellipsoid>
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let ellipsoid = self.parse_ellipsoid()?;

        // Optional: anchor, anchor epoch, identifiers
        let mut anchor = None;
        let mut anchor_epoch = None;
        let mut identifiers = Vec::new();

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();

            match self.peek_keyword().as_deref() {
                Some("ANCHOR") => {
                    self.parse_keyword()?;
                    self.skip_whitespace();
                    self.expect_char('[')?;
                    self.skip_whitespace();
                    anchor = Some(self.parse_quoted_string()?);
                    self.skip_whitespace();
                    self.expect_char(']')?;
                }
                Some("ANCHOREPOCH") => {
                    self.parse_keyword()?;
                    self.skip_whitespace();
                    self.expect_char('[')?;
                    self.skip_whitespace();
                    anchor_epoch = Some(self.parse_number()?);
                    self.skip_whitespace();
                    self.expect_char(']')?;
                }
                Some("ID") => {
                    identifiers.push(self.parse_bracketed_node()?);
                }
                _ => {
                    identifiers.push(self.parse_bracketed_node()?);
                }
            }
        }

        self.expect_char(']')?;

        Ok(GeodeticReferenceFrame {
            keyword,
            name,
            ellipsoid,
            anchor,
            anchor_epoch,
            identifiers,
            prime_meridian: None, // filled in by caller
        })
    }

    fn parse_ellipsoid(&mut self) -> Result<Ellipsoid, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "ELLIPSOID" && keyword != "SPHEROID" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        // <ellipsoid name>
        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // <semi-major axis>
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let semi_major_axis = self.parse_number()?;

        // <inverse flattening>
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let inverse_flattening = self.parse_number()?;

        // Optional unit and identifiers
        let mut unit = None;
        let mut identifiers = Vec::new();

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();

            match self.peek_keyword().as_deref() {
                Some("LENGTHUNIT") => {
                    unit = Some(self.parse_bracketed_node()?);
                }
                Some("ID") => {
                    identifiers.push(self.parse_bracketed_node()?);
                }
                _ => {
                    identifiers.push(self.parse_bracketed_node()?);
                }
            }
        }

        self.expect_char(']')?;

        Ok(Ellipsoid {
            name,
            semi_major_axis,
            inverse_flattening,
            unit,
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
            CS[Cartesian, 2],
                AXIS["easting (E)", east, ORDER[1]],
                AXIS["northing (N)", north, ORDER[2]],
                LENGTHUNIT["metre", 1.0]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        assert_eq!(result.name, "WGS 84 / UTM zone 31N");
        let base = &result.base_geodetic_crs;
        assert_eq!(base.keyword, BaseGeodeticCrsKeyword::BaseGeogCrs);
        assert_eq!(base.name, "WGS 84");
        assert!(base.dynamic.is_none());
        let Datum::ReferenceFrame(ref rf) = base.datum else {
            panic!("expected ReferenceFrame");
        };
        assert_eq!(rf.keyword, DatumKeyword::Datum);
        assert_eq!(rf.name, "World Geodetic System 1984");
        assert_eq!(rf.ellipsoid.name, "WGS 84");
        assert_eq!(rf.ellipsoid.semi_major_axis, 6378137.0);
        assert_eq!(rf.ellipsoid.inverse_flattening, 298.257223563);

        let cs = &result.coordinate_system;
        assert_eq!(cs.cs_type, CsType::Cartesian);
        assert_eq!(cs.dimension, 2);
        assert_eq!(cs.axes.len(), 2);
        assert_eq!(cs.axes[0].name_abbrev, "easting (E)");
        assert_eq!(cs.axes[0].direction, "east");
        assert_eq!(cs.axes[0].order, Some(1));
        assert_eq!(cs.axes[1].name_abbrev, "northing (N)");
        assert_eq!(cs.axes[1].direction, "north");
        assert_eq!(cs.axes[1].order, Some(2));
        assert!(cs.cs_unit.as_ref().unwrap().starts_with("LENGTHUNIT["));
    }

    #[test]
    fn parse_cs_with_axis_units() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["WGS 84", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2],
                AXIS["easting", east, LENGTHUNIT["metre", 1.0]],
                AXIS["northing", north, LENGTHUNIT["metre", 1.0]]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let cs = &result.coordinate_system;
        assert_eq!(cs.axes.len(), 2);
        assert!(cs.axes[0].unit.as_ref().unwrap().starts_with("LENGTHUNIT["));
        assert!(cs.axes[1].unit.as_ref().unwrap().starts_with("LENGTHUNIT["));
        assert!(cs.cs_unit.is_none());
    }

    #[test]
    fn parse_cs_ellipsoidal() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["WGS 84", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
            CONVERSION["y", METHOD["m"]],
            CS[ellipsoidal, 2],
                AXIS["latitude", north, ORDER[1], ANGLEUNIT["degree", 0.0174532925199433]],
                AXIS["longitude", east, ORDER[2], ANGLEUNIT["degree", 0.0174532925199433]]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let cs = &result.coordinate_system;
        assert_eq!(cs.cs_type, CsType::Ellipsoidal);
        assert_eq!(cs.dimension, 2);
        assert_eq!(cs.axes[0].direction, "north");
        assert!(cs.axes[0].unit.as_ref().unwrap().starts_with("ANGLEUNIT["));
    }

    #[test]
    fn parse_cs_with_meridian() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["WGS 84", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2],
                AXIS["x", north, MERIDIAN[90, ANGLEUNIT["degree", 0.0174532925199433]]],
                AXIS["y", north, MERIDIAN[0, ANGLEUNIT["degree", 0.0174532925199433]]]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let cs = &result.coordinate_system;
        assert!(
            cs.axes[0]
                .meridian
                .as_ref()
                .unwrap()
                .starts_with("MERIDIAN[")
        );
        assert!(
            cs.axes[1]
                .meridian
                .as_ref()
                .unwrap()
                .starts_with("MERIDIAN[")
        );
    }

    #[test]
    fn parse_cs_no_axes() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let cs = &result.coordinate_system;
        assert_eq!(cs.cs_type, CsType::Cartesian);
        assert_eq!(cs.dimension, 2);
        assert!(cs.axes.is_empty());
        assert!(cs.cs_unit.is_none());
    }

    #[test]
    fn parse_cs_with_id() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2, ID["EPSG", 4400]],
                AXIS["easting", east],
                AXIS["northing", north],
                LENGTHUNIT["metre", 1.0]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let cs = &result.coordinate_system;
        assert_eq!(cs.identifiers.len(), 1);
        assert!(cs.identifiers[0].starts_with("ID["));
        assert_eq!(cs.axes.len(), 2);
    }

    #[test]
    fn parse_dynamic_geodcrs() {
        let wkt = r#"PROJCRS["test",
            BASEGEODCRS["WGS 84",
                DYNAMIC[FRAMEEPOCH[2010.0]],
                DATUM["World Geodetic System 1984", ELLIPSOID["WGS 84",6378137,298.257223563]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let base = &result.base_geodetic_crs;
        assert_eq!(base.keyword, BaseGeodeticCrsKeyword::BaseGeodCrs);
        let dynamic = base.dynamic.as_ref().unwrap();
        assert_eq!(dynamic.frame_reference_epoch, 2010.0);
        assert!(dynamic.deformation_model.is_none());
        assert!(matches!(base.datum, Datum::ReferenceFrame(_)));
    }

    #[test]
    fn parse_dynamic_with_deformation_model() {
        let wkt = r#"PROJCRS["test",
            BASEGEODCRS["NAD83",
                DYNAMIC[FRAMEEPOCH[2010.0],MODEL["NAD83(CSRS)v6 velocity grid"]],
                DATUM["NAD83", ELLIPSOID["GRS 1980",6378137,298.257222101]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let dynamic = result.base_geodetic_crs.dynamic.as_ref().unwrap();
        assert_eq!(dynamic.frame_reference_epoch, 2010.0);
        let model = dynamic.deformation_model.as_ref().unwrap();
        assert_eq!(model.name, "NAD83(CSRS)v6 velocity grid");
        assert!(model.identifiers.is_empty());
    }

    #[test]
    fn parse_base_crs_with_unit_and_id() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["WGS 84",
                DATUM["WGS 1984", ELLIPSOID["WGS 84",6378137,298.257223563]],
                ANGLEUNIT["degree", 0.0174532925199433],
                ID["EPSG", 4326]],
            CONVERSION["y", METHOD["m"]],
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
    fn parse_geodetic_datum_ensemble() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["WGS 84",
                ENSEMBLE["WGS 84 ensemble",
                    MEMBER["WGS 84 (TRANSIT)"],
                    MEMBER["WGS 84 (G730)", ID["EPSG",1152]],
                    MEMBER["WGS 84 (G834)"],
                    ELLIPSOID["WGS 84",6378137,298.2572236,LENGTHUNIT["metre",1.0]],
                    ENSEMBLEACCURACY[2]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let Datum::Ensemble(ref ens) = result.base_geodetic_crs.datum else {
            panic!("expected Ensemble");
        };
        assert_eq!(ens.name, "WGS 84 ensemble");
        assert_eq!(ens.members.len(), 3);
        assert_eq!(ens.members[0].name, "WGS 84 (TRANSIT)");
        assert!(ens.members[0].identifiers.is_empty());
        assert_eq!(ens.members[1].name, "WGS 84 (G730)");
        assert_eq!(ens.members[1].identifiers.len(), 1);
        assert_eq!(ens.ellipsoid.as_ref().unwrap().name, "WGS 84");
        assert_eq!(ens.accuracy, 2.0);
        assert!(ens.prime_meridian.is_none());
    }

    #[test]
    fn parse_vertical_datum_ensemble() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["x",
                ENSEMBLE["EVRS ensemble",
                    MEMBER["EVRF2000"],
                    MEMBER["EVRF2007"],
                    ENSEMBLEACCURACY[0.01]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let Datum::Ensemble(ref ens) = result.base_geodetic_crs.datum else {
            panic!("expected Ensemble");
        };
        assert_eq!(ens.name, "EVRS ensemble");
        assert_eq!(ens.members.len(), 2);
        assert!(ens.ellipsoid.is_none());
        assert_eq!(ens.accuracy, 0.01);
    }

    #[test]
    fn parse_datum_with_anchor() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["Tananarive 1925",
                GEODETICDATUM["Tananarive 1925",
                    ELLIPSOID["International 1924",6378388.0,297.0,LENGTHUNIT["metre",1.0]],
                    ANCHOR["Tananarive observatory:21.0191667gS, 50.23849537gE of Paris"]],
                PRIMEM["Paris",2.5969213,ANGLEUNIT["grad",0.015707963267949]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let Datum::ReferenceFrame(ref rf) = result.base_geodetic_crs.datum else {
            panic!("expected ReferenceFrame");
        };
        assert_eq!(rf.keyword, DatumKeyword::GeodeticDatum);
        assert_eq!(rf.name, "Tananarive 1925");
        assert_eq!(rf.ellipsoid.name, "International 1924");
        assert_eq!(rf.ellipsoid.semi_major_axis, 6378388.0);
        assert_eq!(rf.ellipsoid.inverse_flattening, 297.0);
        assert!(rf.ellipsoid.unit.as_ref().unwrap().starts_with("LENGTHUNIT["));
        assert_eq!(
            rf.anchor.as_deref(),
            Some("Tananarive observatory:21.0191667gS, 50.23849537gE of Paris")
        );
        assert!(rf.prime_meridian.as_ref().unwrap().starts_with("PRIMEM["));
    }

    #[test]
    fn parse_datum_with_anchor_epoch() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["NAD83",
                DATUM["NAD83 (National Spatial Reference System 2011)",
                    ELLIPSOID["GRS 1980",6378137,298.257222101,LENGTHUNIT["metre",1.0]],
                    ANCHOREPOCH[2010.0]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let Datum::ReferenceFrame(ref rf) = result.base_geodetic_crs.datum else {
            panic!("expected ReferenceFrame");
        };
        assert_eq!(rf.anchor_epoch, Some(2010.0));
        assert!(rf.anchor.is_none());
    }

    #[test]
    fn parse_datum_trf_keyword() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["WGS 84",
                TRF["World Geodetic System 1984",
                    ELLIPSOID["WGS 84",6378388.0,298.257223563,LENGTHUNIT["metre",1.0]]],
                PRIMEM["Greenwich",0.0]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let Datum::ReferenceFrame(ref rf) = result.base_geodetic_crs.datum else {
            panic!("expected ReferenceFrame");
        };
        assert_eq!(rf.keyword, DatumKeyword::Trf);
        assert!(rf.prime_meridian.as_ref().unwrap().starts_with("PRIMEM["));
    }

    #[test]
    fn parse_map_projection_with_parameters() {
        let wkt = r#"PROJCRS["WGS 84 / UTM zone 10N",
            BASEGEOGCRS["WGS 84", DATUM["WGS 1984", ELLIPSOID["WGS 84",6378137,298.257223563]]],
            CONVERSION["UTM zone 10N",
                METHOD["Transverse Mercator", ID["EPSG",9807]],
                PARAMETER["Latitude of natural origin",0,
                    ANGLEUNIT["degree",0.0174532925199433],
                    ID["EPSG",8801]],
                PARAMETER["Longitude of natural origin",-123,
                    ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8802]],
                PARAMETER["Scale factor at natural origin",0.9996,
                    SCALEUNIT["unity",1.0],ID["EPSG",8805]],
                PARAMETER["False easting",500000,
                    LENGTHUNIT["metre",1.0],ID["EPSG",8806]],
                PARAMETER["False northing",0,LENGTHUNIT["metre",1.0],ID["EPSG",8807]]],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let proj = &result.map_projection;
        assert_eq!(proj.name, "UTM zone 10N");
        assert_eq!(proj.method.name, "Transverse Mercator");
        assert_eq!(proj.method.identifiers.len(), 1);
        assert_eq!(proj.parameters.len(), 5);

        assert_eq!(proj.parameters[0].name, "Latitude of natural origin");
        assert_eq!(proj.parameters[0].value, 0.0);
        assert!(
            proj.parameters[0]
                .unit
                .as_ref()
                .unwrap()
                .starts_with("ANGLEUNIT[")
        );
        assert_eq!(proj.parameters[0].identifiers.len(), 1);

        assert_eq!(proj.parameters[1].name, "Longitude of natural origin");
        assert_eq!(proj.parameters[1].value, -123.0);

        assert_eq!(proj.parameters[2].name, "Scale factor at natural origin");
        assert_eq!(proj.parameters[2].value, 0.9996);
        assert!(
            proj.parameters[2]
                .unit
                .as_ref()
                .unwrap()
                .starts_with("SCALEUNIT[")
        );

        assert_eq!(proj.parameters[3].name, "False easting");
        assert_eq!(proj.parameters[3].value, 500000.0);
        assert!(
            proj.parameters[3]
                .unit
                .as_ref()
                .unwrap()
                .starts_with("LENGTHUNIT[")
        );

        assert_eq!(proj.parameters[4].name, "False northing");
        assert_eq!(proj.parameters[4].value, 0.0);
    }

    #[test]
    fn parse_map_projection_with_conversion_id() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["WGS 84", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
            CONVERSION["UTM zone 10N",
                METHOD["Transverse Mercator"],
                PARAMETER["False easting",500000,LENGTHUNIT["metre",1.0]],
                ID["EPSG",16010]],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let proj = &result.map_projection;
        assert_eq!(proj.parameters.len(), 1);
        assert_eq!(proj.identifiers.len(), 1);
        assert!(proj.identifiers[0].starts_with("ID["));
    }

    #[test]
    fn parse_map_projection_method_only() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
            CONVERSION["y", METHOD["Transverse Mercator"]],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let proj = &result.map_projection;
        assert_eq!(proj.name, "y");
        assert_eq!(proj.method.name, "Transverse Mercator");
        assert!(proj.parameters.is_empty());
        assert!(proj.identifiers.is_empty());
    }

    #[test]
    fn parse_projcrs_with_trailing_nodes() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2],
                AXIS["easting", east],
                AXIS["northing", north],
                LENGTHUNIT["metre", 1.0],
            ID["EPSG", 32631]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        assert_eq!(result.coordinate_system.axes.len(), 2);
        assert_eq!(result.scope_extent_identifier_remark.len(), 1);
        assert!(result.scope_extent_identifier_remark[0].starts_with("ID["));
    }

    #[test]
    fn reject_projectedcrs() {
        let wkt = r#"PROJECTEDCRS["test", BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]], CONVERSION["y", METHOD["m"]], CS[Cartesian, 2]]"#;
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
        let wkt = r#"PROJCRS["test", BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]], CONVERSION["y", METHOD["m"]], CS[Cartesian, 2]] extra"#;
        let mut parser = Parser::new(wkt);
        let err = parser.parse_projected_crs().unwrap_err();
        assert!(matches!(err, ParseError::TrailingInput { .. }));
    }
}
