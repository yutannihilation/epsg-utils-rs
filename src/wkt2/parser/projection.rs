use crate::crs::{MapProjection, MapProjectionMethod, MapProjectionParameter};
use crate::error::ParseError;

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_map_projection(&mut self) -> Result<MapProjection, ParseError> {
        self.bracketed(&["CONVERSION"], |p| {
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

    pub(crate) fn parse_map_projection_method(
        &mut self,
    ) -> Result<MapProjectionMethod, ParseError> {
        self.bracketed(&["METHOD", "PROJECTION"], |p| {
            let name = p.parse_quoted_string()?;
            let identifiers = p.trailing_identifiers()?;
            Ok(MapProjectionMethod { name, identifiers })
        })
        .map(|(_, m)| m)
    }

    pub(crate) fn parse_map_projection_parameter(
        &mut self,
    ) -> Result<MapProjectionParameter, ParseError> {
        self.bracketed(&["PARAMETER"], |p| {
            let name = p.parse_quoted_string()?;
            let value = p.comma_then(|p| p.parse_number())?;
            let (unit, identifiers) = p.parse_trailing_unit_and_identifiers()?;

            Ok(MapProjectionParameter {
                name,
                value,
                unit,
                identifiers,
            })
        })
        .map(|(_, param)| param)
    }
}
