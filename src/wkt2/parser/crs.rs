use crate::crs::{
    BaseGeodeticCrs, BaseGeodeticCrsKeyword, BaseVertCrs, CompoundCrs, CoordinateSystem, Crs,
    Datum, GeodCrs, GeogCrs, GeoidModel, MapProjection, ProjectedCrs, SingleCrs, VertCrs,
    VertCrsSource, VerticalDatum, VerticalReferenceFrame,
};
use crate::error::ParseError;

use super::Parser;

impl<'a> Parser<'a> {
    pub fn parse_crs(&mut self) -> Result<Crs, ParseError> {
        let crs = self.parse_single_crs()?;

        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err(ParseError::TrailingInput { pos: self.pos });
        }

        Ok(crs)
    }

    /// Parse a single CRS without checking for trailing input.
    /// Used internally by `parse_crs` and for compound CRS components.
    fn parse_single_crs(&mut self) -> Result<Crs, ParseError> {
        self.skip_whitespace();
        match self.peek_keyword().as_deref() {
            Some("PROJCRS") => Ok(Crs::ProjectedCrs(Box::new(self.parse_projected_crs()?))),
            Some("GEOGCRS") | Some("GEOGRAPHICCRS") => {
                Ok(Crs::GeogCrs(Box::new(self.parse_geog_crs()?)))
            }
            Some("GEODCRS") | Some("GEODETICCRS") => {
                Ok(Crs::GeodCrs(Box::new(self.parse_geod_crs()?)))
            }
            Some("VERTCRS") | Some("VERTICALCRS") => {
                Ok(Crs::VertCrs(Box::new(self.parse_vert_crs()?)))
            }
            Some("COMPOUNDCRS") => Ok(Crs::CompoundCrs(Box::new(self.parse_compound_crs()?))),
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
        match keyword_str.as_str() {
            "GEOGCRS" | "GEOGRAPHICCRS" => {}
            _ => {
                return Err(ParseError::ExpectedKeyword {
                    pos: self.pos - keyword_str.len(),
                });
            }
        }

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

        Ok(GeogCrs {
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
        match keyword_str.as_str() {
            "GEODCRS" | "GEODETICCRS" => {}
            _ => {
                return Err(ParseError::ExpectedKeyword {
                    pos: self.pos - keyword_str.len(),
                });
            }
        }

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

        Ok(GeodCrs {
            name,
            dynamic,
            datum,
            coordinate_system,
            usages,
            identifiers,
            remark,
        })
    }

    pub fn parse_vert_crs(&mut self) -> Result<VertCrs, ParseError> {
        self.skip_whitespace();
        let keyword_str = self.parse_keyword()?;
        match keyword_str.as_str() {
            "VERTCRS" | "VERTICALCRS" => {}
            _ => {
                return Err(ParseError::ExpectedKeyword {
                    pos: self.pos - keyword_str.len(),
                });
            }
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // BASEVERTCRS (derived) or DYNAMIC/VDATUM/ENSEMBLE (standalone)
        self.skip_whitespace();
        self.expect_char(',')?;
        self.skip_whitespace();

        let source = if self.peek_keyword().as_deref() == Some("BASEVERTCRS") {
            let base_vert_crs = self.parse_base_vert_crs()?;
            let deriving_conversion = self.comma_then(|p| p.parse_deriving_conversion())?;
            VertCrsSource::Derived {
                base_vert_crs,
                deriving_conversion,
            }
        } else {
            let dynamic = if self.peek_keyword().as_deref() == Some("DYNAMIC") {
                let d = self.parse_dynamic_crs()?;
                self.skip_whitespace();
                self.expect_char(',')?;
                self.skip_whitespace();
                Some(d)
            } else {
                None
            };

            let datum = match self.peek_keyword().as_deref() {
                Some("ENSEMBLE") => VerticalDatum::Ensemble(Box::new(self.parse_datum_ensemble()?)),
                _ => VerticalDatum::ReferenceFrame(self.parse_vertical_reference_frame()?),
            };

            VertCrsSource::Datum { dynamic, datum }
        };

        // Coordinate system
        let (cs_type, dimension, cs_identifiers) = self.comma_then(|p| p.parse_cs_header())?;

        let mut axes = Vec::new();
        let mut cs_unit = None;
        let mut geoid_models = Vec::new();
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
            "GEOIDMODEL" => {
                let (_, model) = p.bracketed(&["GEOIDMODEL"], |p| {
                    let name = p.parse_quoted_string()?;
                    let identifiers = p.trailing_identifiers()?;
                    Ok(GeoidModel { name, identifiers })
                })?;
                geoid_models.push(model);
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

        Ok(VertCrs {
            name,
            source,
            coordinate_system,
            geoid_models,
            usages,
            identifiers,
            remark,
        })
    }

    fn parse_base_vert_crs(&mut self) -> Result<BaseVertCrs, ParseError> {
        self.bracketed(&["BASEVERTCRS"], |p| {
            let name = p.parse_quoted_string()?;

            p.skip_whitespace();
            p.expect_char(',')?;
            p.skip_whitespace();

            let dynamic = if p.peek_keyword().as_deref() == Some("DYNAMIC") {
                let d = p.parse_dynamic_crs()?;
                p.skip_whitespace();
                p.expect_char(',')?;
                p.skip_whitespace();
                Some(d)
            } else {
                None
            };

            let datum = match p.peek_keyword().as_deref() {
                Some("ENSEMBLE") => VerticalDatum::Ensemble(Box::new(p.parse_datum_ensemble()?)),
                _ => VerticalDatum::ReferenceFrame(p.parse_vertical_reference_frame()?),
            };

            let identifiers = p.trailing_identifiers()?;

            Ok(BaseVertCrs {
                name,
                dynamic,
                datum,
                identifiers,
            })
        })
        .map(|(_, base)| base)
    }

    fn parse_deriving_conversion(&mut self) -> Result<MapProjection, ParseError> {
        self.bracketed(&["DERIVINGCONVERSION"], |p| {
            let name = p.parse_quoted_string()?;
            let method = p.comma_then(|p| p.parse_map_projection_method())?;

            let mut parameters = Vec::new();
            let mut identifiers = Vec::new();

            p.trailing_items(|p, kw| match kw {
                "PARAMETER" => {
                    parameters.push(p.parse_map_projection_parameter()?);
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

            Ok(MapProjection {
                name,
                method,
                parameters,
                identifiers,
            })
        })
        .map(|(_, mp)| mp)
    }

    pub fn parse_compound_crs(&mut self) -> Result<CompoundCrs, ParseError> {
        self.skip_whitespace();
        let keyword = self.parse_keyword()?;
        if keyword != "COMPOUNDCRS" {
            return Err(ParseError::ExpectedKeyword {
                pos: self.pos - keyword.len(),
            });
        }

        self.skip_whitespace();
        self.expect_char('[')?;

        self.skip_whitespace();
        let name = self.parse_quoted_string()?;

        // Parse at least two component CRSs
        let mut components = Vec::new();
        // First component is required
        let first = self.comma_then(|p| p.parse_single_crs_component())?;
        components.push(first);
        // Second component is required
        let second = self.comma_then(|p| p.parse_single_crs_component())?;
        components.push(second);

        let mut usages = Vec::new();
        let mut identifiers = Vec::new();
        let mut remark = None;

        self.trailing_items(|p, kw| match kw {
            // Additional component CRSs or trailing metadata
            "PROJCRS" | "GEOGCRS" | "GEOGRAPHICCRS" | "GEODCRS" | "GEODETICCRS" | "VERTCRS"
            | "VERTICALCRS" => {
                components.push(p.parse_single_crs_component_with_keyword(kw)?);
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
                // Could be an unsupported CRS type as a component
                // or truly unknown metadata — store as Other component
                // if it looks like a CRS keyword (ends with CRS)
                if kw.ends_with("CRS") {
                    let wkt = p.parse_bracketed_node()?;
                    components.push(SingleCrs::Other(format!("{kw}{wkt}")));
                } else {
                    p.parse_bracketed_node()?;
                }
                Ok(())
            }
        })?;

        self.expect_char(']')?;

        Ok(CompoundCrs {
            name,
            components,
            usages,
            identifiers,
            remark,
        })
    }

    /// Parse a single CRS component (for compound CRS).
    fn parse_single_crs_component(&mut self) -> Result<SingleCrs, ParseError> {
        self.skip_whitespace();
        let kw = self.peek_keyword();
        match kw.as_deref() {
            Some("PROJCRS") => Ok(SingleCrs::ProjectedCrs(Box::new(
                self.parse_projected_crs()?,
            ))),
            Some("GEOGCRS") | Some("GEOGRAPHICCRS") => {
                Ok(SingleCrs::GeogCrs(Box::new(self.parse_geog_crs()?)))
            }
            Some("GEODCRS") | Some("GEODETICCRS") => {
                Ok(SingleCrs::GeodCrs(Box::new(self.parse_geod_crs()?)))
            }
            Some("VERTCRS") | Some("VERTICALCRS") => {
                Ok(SingleCrs::VertCrs(Box::new(self.parse_vert_crs()?)))
            }
            _ => {
                // Unsupported CRS type — capture raw WKT
                let raw = self.parse_bracketed_node()?;
                Ok(SingleCrs::Other(raw))
            }
        }
    }

    /// Parse a single CRS component when the keyword has already been peeked
    /// by `trailing_items` (keyword is already known but NOT consumed).
    fn parse_single_crs_component_with_keyword(
        &mut self,
        kw: &str,
    ) -> Result<SingleCrs, ParseError> {
        match kw {
            "PROJCRS" => Ok(SingleCrs::ProjectedCrs(Box::new(
                self.parse_projected_crs()?,
            ))),
            "GEOGCRS" | "GEOGRAPHICCRS" => Ok(SingleCrs::GeogCrs(Box::new(self.parse_geog_crs()?))),
            "GEODCRS" | "GEODETICCRS" => Ok(SingleCrs::GeodCrs(Box::new(self.parse_geod_crs()?))),
            "VERTCRS" | "VERTICALCRS" => Ok(SingleCrs::VertCrs(Box::new(self.parse_vert_crs()?))),
            _ => {
                let raw = self.parse_bracketed_node()?;
                Ok(SingleCrs::Other(raw))
            }
        }
    }

    pub(crate) fn parse_vertical_reference_frame(
        &mut self,
    ) -> Result<VerticalReferenceFrame, ParseError> {
        let (_, (name, anchor, anchor_epoch, identifiers)) =
            self.bracketed(&["VDATUM", "VRF", "VERTICALDATUM"], |p| {
                let name = p.parse_quoted_string()?;

                let mut anchor = None;
                let mut anchor_epoch = None;
                let mut identifiers = Vec::new();

                p.trailing_items(|p, kw| match kw {
                    "ANCHOR" => {
                        anchor = Some(p.parse_keyword_quoted_string("ANCHOR")?);
                        Ok(())
                    }
                    "ANCHOREPOCH" => {
                        anchor_epoch = Some(p.parse_keyword_number("ANCHOREPOCH")?);
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

                Ok((name, anchor, anchor_epoch, identifiers))
            })?;

        Ok(VerticalReferenceFrame {
            name,
            anchor,
            anchor_epoch,
            identifiers,
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
