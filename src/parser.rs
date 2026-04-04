use crate::error::ParseError;
use crate::wkt2::{
    AuthorityId, Axis, BaseGeodeticCrs, BaseGeodeticCrsKeyword, CoordinateSystem, CsType, Datum,
    DatumEnsemble, DatumKeyword, DeformationModel, DynamicCrs, Ellipsoid, EnsembleMember,
    GeodeticReferenceFrame, Identifier, MapProjection, MapProjectionMethod, MapProjectionParameter,
    Meridian, PrimeMeridian, ProjectedCrs, RangeMeaning, Unit, UnitKeyword, Usage,
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
        let mut usages = Vec::new();
        let mut identifiers = Vec::new();
        let mut remark = None;

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
                Some(kw) if Self::is_unit_keyword(kw) => {
                    cs_unit = Some(self.parse_unit()?);
                }
                Some("USAGE") => {
                    usages.push(self.parse_usage()?);
                }
                Some("ID") => {
                    identifiers.push(self.parse_identifier_node()?);
                }
                Some("REMARK") => {
                    remark = Some(self.parse_remark()?);
                }
                _ => {
                    // Unknown node, skip it
                    self.parse_bracketed_node()?;
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
            usages,
            identifiers,
            remark,
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
                    identifiers.push(self.parse_identifier_node()?);
                }
                _ => {
                    // Unknown node, skip it
                    self.parse_bracketed_node()?;
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
            identifiers.push(self.parse_identifier_node()?);
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
                Some(kw) if Self::is_unit_keyword(kw) => {
                    unit = Some(self.parse_unit()?);
                }
                Some("ID") => {
                    identifiers.push(self.parse_identifier_node()?);
                }
                _ => {
                    // Unknown node, skip it
                    self.parse_bracketed_node()?;
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

    fn parse_usage(&mut self) -> Result<Usage, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "USAGE" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }
        self.skip_whitespace();
        self.expect_char('[')?;

        // <scope> — a bracketed node like SCOPE[...]
        self.skip_whitespace();
        let scope = self.parse_bracketed_node()?;

        // <extent> — a bracketed node like AREA[...] or BBOX[...]
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let extent = self.parse_bracketed_node()?;

        self.skip_whitespace();
        self.expect_char(']')?;

        Ok(Usage { scope, extent })
    }

    fn parse_remark(&mut self) -> Result<String, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "REMARK" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }
        self.skip_whitespace();
        self.expect_char('[')?;
        self.skip_whitespace();
        let text = self.parse_quoted_string()?;
        self.skip_whitespace();
        self.expect_char(']')?;
        Ok(text)
    }

    fn parse_cs_header(&mut self) -> Result<(CsType, u8, Vec<Identifier>), ParseError> {
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
            identifiers.push(self.parse_identifier_node()?);
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
        let mut axis_min_value = None;
        let mut axis_max_value = None;
        let mut range_meaning = None;
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
                    meridian = Some(self.parse_meridian()?);
                }
                Some("BEARING") => {
                    bearing = Some(self.parse_keyword_number("BEARING")?);
                }
                Some("ORDER") => {
                    order = Some(self.parse_keyword_number("ORDER")? as u32);
                }
                Some(kw) if Self::is_unit_keyword(kw) => {
                    unit = Some(self.parse_unit()?);
                }
                Some("AXISMINVALUE") => {
                    axis_min_value = Some(self.parse_keyword_number("AXISMINVALUE")?);
                }
                Some("AXISMAXVALUE") => {
                    axis_max_value = Some(self.parse_keyword_number("AXISMAXVALUE")?);
                }
                Some("RANGEMEANING") => {
                    range_meaning = Some(self.parse_range_meaning()?);
                }
                Some("ID") => {
                    identifiers.push(self.parse_identifier_node()?);
                }
                _ => {
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
            axis_min_value,
            axis_max_value,
            range_meaning,
            identifiers,
        })
    }

    /// Parse KEYWORD[number] — used for ORDER, BEARING, AXISMINVALUE, AXISMAXVALUE.
    fn parse_keyword_number(&mut self, expected: &str) -> Result<f64, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != expected {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }
        self.skip_whitespace();
        self.expect_char('[')?;
        self.skip_whitespace();
        let value = self.parse_number()?;
        self.skip_whitespace();
        self.expect_char(']')?;
        Ok(value)
    }

    fn parse_meridian(&mut self) -> Result<Meridian, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "MERIDIAN" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }
        self.skip_whitespace();
        self.expect_char('[')?;
        self.skip_whitespace();
        let value = self.parse_number()?;
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let unit = self.parse_unit()?;
        self.skip_whitespace();
        self.expect_char(']')?;
        Ok(Meridian { value, unit })
    }

    fn parse_range_meaning(&mut self) -> Result<RangeMeaning, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "RANGEMEANING" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }
        self.skip_whitespace();
        self.expect_char('[')?;
        self.skip_whitespace();
        let value = self.parse_identifier()?;
        let meaning = match value.as_str() {
            "exact" => RangeMeaning::Exact,
            "wraparound" => RangeMeaning::Wraparound,
            _ => {
                let len = value.len();
                return Err(ParseError::UnexpectedKeyword {
                    keyword: value,
                    pos: self.pos - len,
                });
            }
        };
        self.skip_whitespace();
        self.expect_char(']')?;
        Ok(meaning)
    }

    fn parse_identifier_node(&mut self) -> Result<Identifier, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "ID" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        // <authority name> — quoted string
        self.skip_whitespace();
        let authority_name = self.parse_quoted_string()?;

        // <authority unique identifier> — number or quoted string
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let authority_unique_id = self.parse_number_or_text()?;

        // Optional: version, citation, uri
        let mut version = None;
        let mut citation = None;
        let mut uri = None;

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();

            match self.peek_keyword().as_deref() {
                Some("CITATION") => {
                    self.parse_keyword()?;
                    self.skip_whitespace();
                    self.expect_char('[')?;
                    self.skip_whitespace();
                    citation = Some(self.parse_quoted_string()?);
                    self.skip_whitespace();
                    self.expect_char(']')?;
                }
                Some("URI") => {
                    self.parse_keyword()?;
                    self.skip_whitespace();
                    self.expect_char('[')?;
                    self.skip_whitespace();
                    uri = Some(self.parse_quoted_string()?);
                    self.skip_whitespace();
                    self.expect_char(']')?;
                }
                _ => {
                    // version — number or quoted string
                    version = Some(self.parse_number_or_text()?);
                }
            }
        }

        self.expect_char(']')?;

        Ok(Identifier {
            authority_name,
            authority_unique_id,
            version,
            citation,
            uri,
        })
    }

    /// Parse a value that is either a number or a quoted string.
    fn parse_number_or_text(&mut self) -> Result<AuthorityId, ParseError> {
        if self.peek_char() == Some('"') {
            Ok(AuthorityId::Text(self.parse_quoted_string()?))
        } else {
            Ok(AuthorityId::Number(self.parse_number()?))
        }
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
                Some("PRIMEM" | "PRIMEMERIDIAN") => {
                    let pm = self.parse_prime_meridian()?;
                    match &mut datum {
                        Datum::ReferenceFrame(rf) => rf.prime_meridian = Some(pm),
                        Datum::Ensemble(ens) => ens.prime_meridian = Some(pm),
                    }
                }
                Some(kw) if Self::is_unit_keyword(kw) => {
                    ellipsoidal_cs_unit = Some(self.parse_unit()?);
                }
                Some("ID") => {
                    identifiers.push(self.parse_identifier_node()?);
                }
                _ => {
                    // Unknown node, skip it
                    self.parse_bracketed_node()?;
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
                identifiers.push(self.parse_identifier_node()?);
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
                    identifiers.push(self.parse_identifier_node()?);
                }
                _ => {
                    // Unknown node, skip it
                    self.parse_bracketed_node()?;
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
            identifiers.push(self.parse_identifier_node()?);
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
                    identifiers.push(self.parse_identifier_node()?);
                }
                _ => {
                    // Unknown node, skip it
                    self.parse_bracketed_node()?;
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

    fn parse_unit(&mut self) -> Result<Unit, ParseError> {
        let keyword_str = self.parse_keyword()?;
        let keyword = match keyword_str.as_str() {
            "ANGLEUNIT" => UnitKeyword::AngleUnit,
            "LENGTHUNIT" => UnitKeyword::LengthUnit,
            "PARAMETRICUNIT" => UnitKeyword::ParametricUnit,
            "SCALEUNIT" => UnitKeyword::ScaleUnit,
            "TIMEUNIT" | "TEMPORALQUANTITY" => UnitKeyword::TimeUnit,
            "UNIT" => UnitKeyword::Unit,
            _ => {
                return Err(ParseError::ExpectedKeyword {
                    pos: self.pos - keyword_str.len(),
                });
            }
        };

        self.skip_whitespace();
        self.expect_char('[')?;

        // <unit name>
        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // Optional conversion factor and identifiers
        let mut conversion_factor = None;
        let mut identifiers = Vec::new();

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();

            // If the next char starts a keyword (uppercase), it's an ID node
            if self.peek_keyword().is_some() {
                identifiers.push(self.parse_identifier_node()?);
            } else {
                // It's a number (conversion factor)
                conversion_factor = Some(self.parse_number()?);
            }
        }

        self.expect_char(']')?;

        Ok(Unit {
            keyword,
            name,
            conversion_factor,
            identifiers,
        })
    }

    fn is_unit_keyword(keyword: &str) -> bool {
        matches!(
            keyword,
            "ANGLEUNIT"
                | "LENGTHUNIT"
                | "PARAMETRICUNIT"
                | "SCALEUNIT"
                | "TIMEUNIT"
                | "TEMPORALQUANTITY"
                | "UNIT"
        )
    }

    fn parse_prime_meridian(&mut self) -> Result<PrimeMeridian, ParseError> {
        let keyword = self.parse_keyword()?;
        if keyword != "PRIMEM" && keyword != "PRIMEMERIDIAN" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        // <prime meridian name>
        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // <irm longitude>
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();
        let irm_longitude = self.parse_number()?;

        // Optional angle unit and identifiers
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
                Some(kw) if Self::is_unit_keyword(kw) => {
                    unit = Some(self.parse_unit()?);
                }
                Some("ID") => {
                    identifiers.push(self.parse_identifier_node()?);
                }
                _ => {
                    // Unknown node, skip it
                    self.parse_bracketed_node()?;
                }
            }
        }

        self.expect_char(']')?;

        Ok(PrimeMeridian {
            name,
            irm_longitude,
            unit,
            identifiers,
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
                Some(kw) if Self::is_unit_keyword(kw) => {
                    unit = Some(self.parse_unit()?);
                }
                Some("ID") => {
                    identifiers.push(self.parse_identifier_node()?);
                }
                _ => {
                    // Unknown node, skip it
                    self.parse_bracketed_node()?;
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
        let cs_unit = cs.cs_unit.as_ref().unwrap();
        assert_eq!(cs_unit.keyword, UnitKeyword::LengthUnit);
        assert_eq!(cs_unit.name, "metre");
        assert_eq!(cs_unit.conversion_factor, Some(1.0));
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
        assert_eq!(
            cs.axes[0].unit.as_ref().unwrap().keyword,
            UnitKeyword::LengthUnit
        );
        assert_eq!(
            cs.axes[1].unit.as_ref().unwrap().keyword,
            UnitKeyword::LengthUnit
        );
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
        assert_eq!(
            cs.axes[0].unit.as_ref().unwrap().keyword,
            UnitKeyword::AngleUnit
        );
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
        let m0 = cs.axes[0].meridian.as_ref().unwrap();
        assert_eq!(m0.value, 90.0);
        assert_eq!(m0.unit.keyword, UnitKeyword::AngleUnit);
        let m1 = cs.axes[1].meridian.as_ref().unwrap();
        assert_eq!(m1.value, 0.0);
        assert_eq!(m1.unit.keyword, UnitKeyword::AngleUnit);
    }

    #[test]
    fn parse_axis_bearing_and_range() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2],
                AXIS["x", clockwise, BEARING[90], ORDER[1],
                    AXISMINVALUE[0], AXISMAXVALUE[360], RANGEMEANING[wraparound]],
                AXIS["y", clockwise, BEARING[0], ORDER[2]]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let ax0 = &result.coordinate_system.axes[0];
        assert_eq!(ax0.direction, "clockwise");
        assert_eq!(ax0.bearing, Some(90.0));
        assert_eq!(ax0.order, Some(1));
        assert_eq!(ax0.axis_min_value, Some(0.0));
        assert_eq!(ax0.axis_max_value, Some(360.0));
        assert_eq!(ax0.range_meaning, Some(RangeMeaning::Wraparound));

        let ax1 = &result.coordinate_system.axes[1];
        assert_eq!(ax1.bearing, Some(0.0));
        assert!(ax1.axis_min_value.is_none());
        assert!(ax1.range_meaning.is_none());
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
        assert_eq!(cs.identifiers[0].authority_name, "EPSG");
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
        let eu = base.ellipsoidal_cs_unit.as_ref().unwrap();
        assert_eq!(eu.keyword, UnitKeyword::AngleUnit);
        assert_eq!(eu.name, "degree");
        assert_eq!(base.identifiers.len(), 1);
        assert_eq!(base.identifiers[0].authority_name, "EPSG");
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
        assert_eq!(
            rf.ellipsoid.unit.as_ref().unwrap().keyword,
            UnitKeyword::LengthUnit
        );
        assert_eq!(
            rf.anchor.as_deref(),
            Some("Tananarive observatory:21.0191667gS, 50.23849537gE of Paris")
        );
        let pm = rf.prime_meridian.as_ref().unwrap();
        assert_eq!(pm.name, "Paris");
        assert_eq!(pm.irm_longitude, 2.5969213);
        assert_eq!(pm.unit.as_ref().unwrap().keyword, UnitKeyword::AngleUnit);
        assert_eq!(pm.unit.as_ref().unwrap().name, "grad");
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
        let pm = rf.prime_meridian.as_ref().unwrap();
        assert_eq!(pm.name, "Greenwich");
        assert_eq!(pm.irm_longitude, 0.0);
        assert!(pm.unit.is_none());
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
        assert_eq!(
            proj.parameters[0].unit.as_ref().unwrap().keyword,
            UnitKeyword::AngleUnit
        );
        assert_eq!(proj.parameters[0].identifiers.len(), 1);

        assert_eq!(proj.parameters[1].name, "Longitude of natural origin");
        assert_eq!(proj.parameters[1].value, -123.0);

        assert_eq!(proj.parameters[2].name, "Scale factor at natural origin");
        assert_eq!(proj.parameters[2].value, 0.9996);
        assert_eq!(
            proj.parameters[2].unit.as_ref().unwrap().keyword,
            UnitKeyword::ScaleUnit
        );

        assert_eq!(proj.parameters[3].name, "False easting");
        assert_eq!(proj.parameters[3].value, 500000.0);
        assert_eq!(
            proj.parameters[3].unit.as_ref().unwrap().keyword,
            UnitKeyword::LengthUnit
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
        assert_eq!(proj.identifiers[0].authority_name, "EPSG");
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
        assert_eq!(result.identifiers.len(), 1);
        assert_eq!(result.identifiers[0].authority_name, "EPSG");
        assert_eq!(
            result.identifiers[0].authority_unique_id,
            AuthorityId::Number(32631.0)
        );
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
    fn parse_usage_and_remark() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2],
            USAGE[SCOPE["Engineering survey"], AREA["Netherlands"]],
            USAGE[SCOPE["Cadastre"], AREA["Germany"]],
            ID["EPSG", 32631],
            REMARK["This is a test CRS"]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        assert_eq!(result.usages.len(), 2);
        assert!(result.usages[0].scope.starts_with("SCOPE["));
        assert!(result.usages[0].extent.starts_with("AREA["));
        assert!(result.usages[1].scope.starts_with("SCOPE["));
        assert_eq!(result.identifiers.len(), 1);
        assert_eq!(result.remark.as_deref(), Some("This is a test CRS"));
    }

    #[test]
    fn trailing_input_error() {
        let wkt = r#"PROJCRS["test", BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]], CONVERSION["y", METHOD["m"]], CS[Cartesian, 2]] extra"#;
        let mut parser = Parser::new(wkt);
        let err = parser.parse_projected_crs().unwrap_err();
        assert!(matches!(err, ParseError::TrailingInput { .. }));
    }

    #[test]
    fn parse_identifier_number_id() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257],
                ID["EPSG",6326]]],
            CONVERSION["y", METHOD["m", ID["EPSG",9807]]],
            CS[Cartesian, 2]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let Datum::ReferenceFrame(ref rf) = result.base_geodetic_crs.datum else {
            panic!("expected ReferenceFrame");
        };
        assert_eq!(rf.identifiers.len(), 1);
        assert_eq!(rf.identifiers[0].authority_name, "EPSG");
        assert_eq!(
            rf.identifiers[0].authority_unique_id,
            AuthorityId::Number(6326.0)
        );

        assert_eq!(
            result.map_projection.method.identifiers[0].authority_name,
            "EPSG"
        );
        assert_eq!(
            result.map_projection.method.identifiers[0].authority_unique_id,
            AuthorityId::Number(9807.0)
        );
    }

    #[test]
    fn parse_identifier_with_version_and_uri() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2, ID["EPSG",4400,URI["urn:ogc:def:cs:EPSG::4400"]]]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let id = &result.coordinate_system.identifiers[0];
        assert_eq!(id.authority_name, "EPSG");
        assert_eq!(id.authority_unique_id, AuthorityId::Number(4400.0));
        assert_eq!(id.uri.as_deref(), Some("urn:ogc:def:cs:EPSG::4400"));
    }

    #[test]
    fn parse_identifier_text_id_with_version() {
        let wkt = r#"PROJCRS["test",
            BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
            CONVERSION["y", METHOD["m"]],
            CS[Cartesian, 2, ID["Authority name","Abcd_Ef",7.1]]]"#;

        let mut parser = Parser::new(wkt);
        let result = parser.parse_projected_crs().unwrap();

        let id = &result.coordinate_system.identifiers[0];
        assert_eq!(id.authority_name, "Authority name");
        assert_eq!(
            id.authority_unique_id,
            AuthorityId::Text("Abcd_Ef".to_string())
        );
        assert_eq!(id.version, Some(AuthorityId::Number(7.1)));
    }
}
