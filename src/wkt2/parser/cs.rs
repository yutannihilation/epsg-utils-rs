use crate::crs::{Axis, CsType, Identifier, Meridian};
use crate::error::ParseError;

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_cs_header(&mut self) -> Result<(CsType, u8, Vec<Identifier>), ParseError> {
        self.bracketed(&["CS"], |p| {
            let type_str = p.parse_identifier()?;
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
                        pos: p.pos - len,
                    });
                }
            };

            let dimension = p.comma_then(|p| p.parse_number())? as u8;
            let identifiers = p.trailing_identifiers()?;

            Ok((cs_type, dimension, identifiers))
        })
        .map(|(_, result)| result)
    }

    pub(crate) fn parse_axis(&mut self) -> Result<Axis, ParseError> {
        self.bracketed(&["AXIS"], |p| {
            let name_abbrev = p.parse_quoted_string()?;
            let direction = p.comma_then(|p| p.parse_identifier())?;

            let mut meridian = None;
            let mut bearing = None;
            let mut order = None;
            let mut unit = None;
            let mut axis_min_value = None;
            let mut axis_max_value = None;
            let mut range_meaning = None;
            let mut identifiers = Vec::new();

            p.trailing_items(|p, kw| match kw {
                "MERIDIAN" => {
                    meridian = Some(p.parse_meridian()?);
                    Ok(())
                }
                "BEARING" => {
                    bearing = Some(p.parse_keyword_number("BEARING")?);
                    Ok(())
                }
                "ORDER" => {
                    order = Some(p.parse_keyword_number("ORDER")? as u32);
                    Ok(())
                }
                kw if Self::is_unit_keyword(kw) => {
                    unit = Some(p.parse_unit()?);
                    Ok(())
                }
                "AXISMINVALUE" => {
                    axis_min_value = Some(p.parse_keyword_number("AXISMINVALUE")?);
                    Ok(())
                }
                "AXISMAXVALUE" => {
                    axis_max_value = Some(p.parse_keyword_number("AXISMAXVALUE")?);
                    Ok(())
                }
                "RANGEMEANING" => {
                    range_meaning = Some(p.parse_range_meaning()?);
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
        })
        .map(|(_, a)| a)
    }

    pub(crate) fn parse_meridian(&mut self) -> Result<Meridian, ParseError> {
        self.bracketed(&["MERIDIAN"], |p| {
            let value = p.parse_number()?;
            let unit = p.comma_then(|p| p.parse_unit())?;
            Ok(Meridian { value, unit })
        })
        .map(|(_, m)| m)
    }
}
