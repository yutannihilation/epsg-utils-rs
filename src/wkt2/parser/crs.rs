use crate::crs::{
    BaseGeodeticCrs, BaseGeodeticCrsKeyword, CoordinateSystem, Crs, Datum, GeodCrs, GeodCrsKeyword,
    GeogCrs, GeogCrsKeyword, ProjectedCrs,
};
use crate::error::ParseError;

use super::Parser;

impl<'a> Parser<'a> {
    pub fn parse_crs(&mut self) -> Result<Crs, ParseError> {
        self.skip_whitespace();
        match self.peek_keyword().as_deref() {
            Some("PROJCRS") => Ok(Crs::ProjectedCrs(Box::new(self.parse_projected_crs()?))),
            Some("GEOGCRS") | Some("GEOGRAPHICCRS") => {
                Ok(Crs::GeogCrs(Box::new(self.parse_geog_crs()?)))
            }
            Some("GEODCRS") | Some("GEODETICCRS") => {
                Ok(Crs::GeodCrs(Box::new(self.parse_geod_crs()?)))
            }
            _ => {
                // Consume the keyword so we can report it
                let keyword = self.parse_keyword()?;
                Err(ParseError::ExpectedKeyword {
                    pos: self.pos - keyword.len(),
                })
            }
        }
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

        self.skip_whitespace();
        let name = self.parse_quoted_string()?;
        let base_geodetic_crs = self.comma_then(|p| p.parse_base_geodetic_crs())?;
        let map_projection = self.comma_then(|p| p.parse_map_projection())?;
        let (cs_type, dimension, cs_identifiers) = self.comma_then(|p| p.parse_cs_header())?;

        let mut axes = Vec::new();
        let mut cs_unit = None;
        let mut usages = Vec::new();
        let mut identifiers = Vec::new();
        let mut remark = None;

        self.trailing_items(|p, kw| match kw {
            "AXIS" => {
                axes.push(p.parse_axis()?);
                Ok(())
            }
            kw if Self::is_unit_keyword(kw) => {
                cs_unit = Some(p.parse_unit()?);
                Ok(())
            }
            "USAGE" => {
                usages.push(p.parse_usage()?);
                Ok(())
            }
            "ID" => {
                identifiers.push(p.parse_identifier_node()?);
                Ok(())
            }
            "REMARK" => {
                remark = Some(p.parse_remark()?);
                Ok(())
            }
            _ => {
                p.parse_bracketed_node()?;
                Ok(())
            }
        })?;

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

    pub fn parse_geog_crs(&mut self) -> Result<GeogCrs, ParseError> {
        self.skip_whitespace();
        let keyword_str = self.parse_keyword()?;
        let keyword = match keyword_str.as_str() {
            "GEOGCRS" => GeogCrsKeyword::GeogCrs,
            "GEOGRAPHICCRS" => GeogCrsKeyword::GeographicCrs,
            _ => {
                return Err(ParseError::ExpectedKeyword {
                    pos: self.pos - keyword_str.len(),
                });
            }
        };

        self.skip_whitespace();
        self.expect_char('[')?;

        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // DYNAMIC or DATUM/ENSEMBLE
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

        let mut datum = match self.peek_keyword().as_deref() {
            Some("ENSEMBLE") => Datum::Ensemble(self.parse_datum_ensemble()?),
            _ => Datum::ReferenceFrame(self.parse_geodetic_reference_frame()?),
        };

        // PRIMEM may appear as sibling after the DATUM/ENSEMBLE node
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();

        if matches!(
            self.peek_keyword().as_deref(),
            Some("PRIMEM") | Some("PRIMEMERIDIAN")
        ) {
            let pm = self.parse_prime_meridian()?;
            match &mut datum {
                Datum::ReferenceFrame(rf) => rf.prime_meridian = Some(pm),
                Datum::Ensemble(ens) => ens.prime_meridian = Some(pm),
            }
            self.skip_whitespace();
            self.expect_char(',')?;
            self.skip_whitespace();
        }

        // Coordinate system
        let (cs_type, dimension, cs_identifiers) = self.parse_cs_header()?;

        let mut axes = Vec::new();
        let mut cs_unit = None;
        let mut usages = Vec::new();
        let mut identifiers = Vec::new();
        let mut remark = None;

        self.trailing_items(|p, kw| match kw {
            "AXIS" => {
                axes.push(p.parse_axis()?);
                Ok(())
            }
            kw if Self::is_unit_keyword(kw) => {
                cs_unit = Some(p.parse_unit()?);
                Ok(())
            }
            "USAGE" => {
                usages.push(p.parse_usage()?);
                Ok(())
            }
            "ID" => {
                identifiers.push(p.parse_identifier_node()?);
                Ok(())
            }
            "REMARK" => {
                remark = Some(p.parse_remark()?);
                Ok(())
            }
            _ => {
                p.parse_bracketed_node()?;
                Ok(())
            }
        })?;

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

        Ok(GeogCrs {
            keyword,
            name,
            dynamic,
            datum,
            coordinate_system,
            usages,
            identifiers,
            remark,
        })
    }

    pub fn parse_geod_crs(&mut self) -> Result<GeodCrs, ParseError> {
        self.skip_whitespace();
        let keyword_str = self.parse_keyword()?;
        let keyword = match keyword_str.as_str() {
            "GEODCRS" => GeodCrsKeyword::GeodCrs,
            "GEODETICCRS" => GeodCrsKeyword::GeodeticCrs,
            _ => {
                return Err(ParseError::ExpectedKeyword {
                    pos: self.pos - keyword_str.len(),
                });
            }
        };

        self.skip_whitespace();
        self.expect_char('[')?;

        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // DYNAMIC or DATUM/ENSEMBLE
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

        let mut datum = match self.peek_keyword().as_deref() {
            Some("ENSEMBLE") => Datum::Ensemble(self.parse_datum_ensemble()?),
            _ => Datum::ReferenceFrame(self.parse_geodetic_reference_frame()?),
        };

        // PRIMEM may appear as sibling after the DATUM/ENSEMBLE node
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();

        if matches!(
            self.peek_keyword().as_deref(),
            Some("PRIMEM") | Some("PRIMEMERIDIAN")
        ) {
            let pm = self.parse_prime_meridian()?;
            match &mut datum {
                Datum::ReferenceFrame(rf) => rf.prime_meridian = Some(pm),
                Datum::Ensemble(ens) => ens.prime_meridian = Some(pm),
            }
            self.skip_whitespace();
            self.expect_char(',')?;
            self.skip_whitespace();
        }

        // Coordinate system
        let (cs_type, dimension, cs_identifiers) = self.parse_cs_header()?;

        let mut axes = Vec::new();
        let mut cs_unit = None;
        let mut usages = Vec::new();
        let mut identifiers = Vec::new();
        let mut remark = None;

        self.trailing_items(|p, kw| match kw {
            "AXIS" => {
                axes.push(p.parse_axis()?);
                Ok(())
            }
            kw if Self::is_unit_keyword(kw) => {
                cs_unit = Some(p.parse_unit()?);
                Ok(())
            }
            "USAGE" => {
                usages.push(p.parse_usage()?);
                Ok(())
            }
            "ID" => {
                identifiers.push(p.parse_identifier_node()?);
                Ok(())
            }
            "REMARK" => {
                remark = Some(p.parse_remark()?);
                Ok(())
            }
            _ => {
                p.parse_bracketed_node()?;
                Ok(())
            }
        })?;

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

        Ok(GeodCrs {
            keyword,
            name,
            dynamic,
            datum,
            coordinate_system,
            usages,
            identifiers,
            remark,
        })
    }

    pub(crate) fn parse_base_geodetic_crs(&mut self) -> Result<BaseGeodeticCrs, ParseError> {
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

        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // DYNAMIC or DATUM/ENSEMBLE
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

        let mut datum = match self.peek_keyword().as_deref() {
            Some("ENSEMBLE") => Datum::Ensemble(self.parse_datum_ensemble()?),
            _ => Datum::ReferenceFrame(self.parse_geodetic_reference_frame()?),
        };

        let mut ellipsoidal_cs_unit = None;
        let mut identifiers = Vec::new();

        self.trailing_items(|p, kw| match kw {
            "PRIMEM" | "PRIMEMERIDIAN" => {
                let pm = p.parse_prime_meridian()?;
                match &mut datum {
                    Datum::ReferenceFrame(rf) => rf.prime_meridian = Some(pm),
                    Datum::Ensemble(ens) => ens.prime_meridian = Some(pm),
                }
                Ok(())
            }
            kw if Self::is_unit_keyword(kw) => {
                ellipsoidal_cs_unit = Some(p.parse_unit()?);
                Ok(())
            }
            "ID" => {
                identifiers.push(p.parse_identifier_node()?);
                Ok(())
            }
            _ => {
                p.parse_bracketed_node()?;
                Ok(())
            }
        })?;

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
}
