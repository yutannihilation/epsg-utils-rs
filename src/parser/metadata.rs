use crate::error::ParseError;
use crate::wkt2::{BBox, Identifier, RangeMeaning, TemporalExtent, Usage, VerticalExtent};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_identifier_node(&mut self) -> Result<Identifier, ParseError> {
        self.bracketed(&["ID"], |p| {
            let authority_name = p.parse_quoted_string()?;
            let authority_unique_id = p.comma_then(|p| p.parse_number_or_text())?;

            let mut version = None;
            let mut citation = None;
            let mut uri = None;

            p.trailing_items(|p, kw| match kw {
                "CITATION" => {
                    citation = Some(p.parse_keyword_quoted_string("CITATION")?);
                    Ok(())
                }
                "URI" => {
                    uri = Some(p.parse_keyword_quoted_string("URI")?);
                    Ok(())
                }
                _ => {
                    version = Some(p.parse_number_or_text()?);
                    Ok(())
                }
            })?;

            Ok(Identifier {
                authority_name,
                authority_unique_id,
                version,
                citation,
                uri,
            })
        })
        .map(|(_, id)| id)
    }

    pub(crate) fn parse_usage(&mut self) -> Result<Usage, ParseError> {
        self.bracketed(&["USAGE"], |p| {
            let scope = p.parse_scope()?;

            let mut area = None;
            let mut bbox = None;
            let mut vertical_extent = None;
            let mut temporal_extent = None;

            p.trailing_items(|p, kw| match kw {
                "AREA" => {
                    area = Some(p.parse_keyword_quoted_string("AREA")?);
                    Ok(())
                }
                "BBOX" => {
                    bbox = Some(p.parse_bbox()?);
                    Ok(())
                }
                "VERTICALEXTENT" => {
                    vertical_extent = Some(p.parse_vertical_extent()?);
                    Ok(())
                }
                "TIMEEXTENT" => {
                    temporal_extent = Some(p.parse_temporal_extent()?);
                    Ok(())
                }
                _ => {
                    p.parse_bracketed_node()?;
                    Ok(())
                }
            })?;

            Ok(Usage {
                scope,
                area,
                bbox,
                vertical_extent,
                temporal_extent,
            })
        })
        .map(|(_, u)| u)
    }

    pub(crate) fn parse_scope(&mut self) -> Result<String, ParseError> {
        self.parse_keyword_quoted_string("SCOPE")
    }

    pub(crate) fn parse_remark(&mut self) -> Result<String, ParseError> {
        self.parse_keyword_quoted_string("REMARK")
    }

    pub(crate) fn parse_bbox(&mut self) -> Result<BBox, ParseError> {
        self.bracketed(&["BBOX"], |p| {
            let lower_left_latitude = p.parse_number()?;
            let lower_left_longitude = p.comma_then(|p| p.parse_number())?;
            let upper_right_latitude = p.comma_then(|p| p.parse_number())?;
            let upper_right_longitude = p.comma_then(|p| p.parse_number())?;
            Ok(BBox {
                lower_left_latitude,
                lower_left_longitude,
                upper_right_latitude,
                upper_right_longitude,
            })
        })
        .map(|(_, b)| b)
    }

    pub(crate) fn parse_vertical_extent(&mut self) -> Result<VerticalExtent, ParseError> {
        self.bracketed(&["VERTICALEXTENT"], |p| {
            let minimum_height = p.parse_number()?;
            let maximum_height = p.comma_then(|p| p.parse_number())?;

            let mut unit = None;
            p.skip_whitespace();
            if p.peek_char() != Some(']') {
                unit = Some(p.comma_then(|p| p.parse_unit())?);
            }

            Ok(VerticalExtent {
                minimum_height,
                maximum_height,
                unit,
            })
        })
        .map(|(_, v)| v)
    }

    pub(crate) fn parse_temporal_extent(&mut self) -> Result<TemporalExtent, ParseError> {
        self.bracketed(&["TIMEEXTENT"], |p| {
            let start = p.parse_datetime_or_text()?;
            let end = p.comma_then(|p| p.parse_datetime_or_text())?;
            Ok(TemporalExtent { start, end })
        })
        .map(|(_, t)| t)
    }

    pub(crate) fn parse_range_meaning(&mut self) -> Result<RangeMeaning, ParseError> {
        self.bracketed(&["RANGEMEANING"], |p| {
            let value = p.parse_identifier()?;
            match value.as_str() {
                "exact" => Ok(RangeMeaning::Exact),
                "wraparound" => Ok(RangeMeaning::Wraparound),
                _ => {
                    let len = value.len();
                    Err(ParseError::UnexpectedKeyword {
                        keyword: value,
                        pos: p.pos - len,
                    })
                }
            }
        })
        .map(|(_, r)| r)
    }

    /// Parse trailing items that may include unit, identifiers, or unknown nodes.
    /// Common pattern used by ellipsoid, prime meridian, map projection parameter, etc.
    pub(crate) fn parse_trailing_unit_and_identifiers(
        &mut self,
    ) -> Result<(Option<crate::wkt2::Unit>, Vec<Identifier>), ParseError> {
        let mut unit = None;
        let mut identifiers = Vec::new();

        self.trailing_items(|p, kw| {
            if Self::is_unit_keyword(kw) {
                unit = Some(p.parse_unit()?);
            } else if kw == "ID" {
                identifiers.push(p.parse_identifier_node()?);
            } else {
                p.parse_bracketed_node()?;
            }
            Ok(())
        })?;

        Ok((unit, identifiers))
    }
}
