use crate::error::ParseError;
use crate::wkt2::{Unit, UnitKeyword};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_unit(&mut self) -> Result<Unit, ParseError> {
        let (kw_str, (name, conversion_factor, identifiers)) =
            self.bracketed(Self::UNIT_KEYWORDS, |p| {
                let name = p.parse_quoted_string()?;

                let mut conversion_factor = None;
                let mut identifiers = Vec::new();

                p.trailing_items(|p, _kw| {
                    if p.peek_keyword().is_some() {
                        identifiers.push(p.parse_identifier_node()?);
                    } else {
                        conversion_factor = Some(p.parse_number()?);
                    }
                    Ok(())
                })?;

                Ok((name, conversion_factor, identifiers))
            })?;

        let keyword = match kw_str.as_str() {
            "ANGLEUNIT" => UnitKeyword::AngleUnit,
            "LENGTHUNIT" => UnitKeyword::LengthUnit,
            "PARAMETRICUNIT" => UnitKeyword::ParametricUnit,
            "SCALEUNIT" => UnitKeyword::ScaleUnit,
            "TIMEUNIT" | "TEMPORALQUANTITY" => UnitKeyword::TimeUnit,
            "UNIT" => UnitKeyword::Unit,
            _ => unreachable!(),
        };

        Ok(Unit {
            keyword,
            name,
            conversion_factor,
            identifiers,
        })
    }

    pub(crate) const UNIT_KEYWORDS: &'static [&'static str] = &[
        "ANGLEUNIT",
        "LENGTHUNIT",
        "PARAMETRICUNIT",
        "SCALEUNIT",
        "TIMEUNIT",
        "TEMPORALQUANTITY",
        "UNIT",
    ];

    pub(crate) fn is_unit_keyword(keyword: &str) -> bool {
        Self::UNIT_KEYWORDS.contains(&keyword)
    }
}
