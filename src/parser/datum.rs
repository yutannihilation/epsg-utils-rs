use crate::error::ParseError;
use crate::wkt2::{
    DatumEnsemble, DatumKeyword, DeformationModel, DynamicCrs, Ellipsoid, EnsembleMember,
    GeodeticReferenceFrame, PrimeMeridian,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_geodetic_reference_frame(
        &mut self,
    ) -> Result<GeodeticReferenceFrame, ParseError> {
        let (kw_str, (name, ellipsoid, anchor, anchor_epoch, identifiers)) =
            self.bracketed(&["DATUM", "TRF", "GEODETICDATUM"], |p| {
                let name = p.parse_quoted_string()?;
                let ellipsoid = p.comma_then(|p| p.parse_ellipsoid())?;

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

                Ok((name, ellipsoid, anchor, anchor_epoch, identifiers))
            })?;

        let keyword = match kw_str.as_str() {
            "DATUM" => DatumKeyword::Datum,
            "TRF" => DatumKeyword::Trf,
            "GEODETICDATUM" => DatumKeyword::GeodeticDatum,
            _ => unreachable!(),
        };

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

    pub(crate) fn parse_datum_ensemble(&mut self) -> Result<DatumEnsemble, ParseError> {
        self.bracketed(&["ENSEMBLE"], |p| {
            let name = p.parse_quoted_string()?;

            let mut members = Vec::new();
            let mut ellipsoid = None;
            let mut accuracy = None;
            let mut identifiers = Vec::new();

            p.trailing_items(|p, kw| match kw {
                "MEMBER" => {
                    members.push(p.parse_ensemble_member()?);
                    Ok(())
                }
                "ELLIPSOID" | "SPHEROID" => {
                    ellipsoid = Some(p.parse_ellipsoid()?);
                    Ok(())
                }
                "ENSEMBLEACCURACY" => {
                    accuracy = Some(p.parse_keyword_number("ENSEMBLEACCURACY")?);
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

            let accuracy = accuracy.ok_or(ParseError::UnexpectedEnd)?;

            Ok(DatumEnsemble {
                name,
                members,
                ellipsoid,
                accuracy,
                identifiers,
                prime_meridian: None,
            })
        })
        .map(|(_, e)| e)
    }

    pub(crate) fn parse_ensemble_member(&mut self) -> Result<EnsembleMember, ParseError> {
        self.bracketed(&["MEMBER"], |p| {
            let name = p.parse_quoted_string()?;
            let identifiers = p.trailing_identifiers()?;
            Ok(EnsembleMember { name, identifiers })
        })
        .map(|(_, m)| m)
    }

    pub(crate) fn parse_dynamic_crs(&mut self) -> Result<DynamicCrs, ParseError> {
        self.bracketed(&["DYNAMIC"], |p| {
            let frame_reference_epoch = p.parse_keyword_number("FRAMEEPOCH")?;

            let mut deformation_model = None;
            p.skip_whitespace();
            if p.peek_char() != Some(']') {
                p.expect_char(',')?;
                p.skip_whitespace();

                let (_, (name, identifiers)) = p.bracketed(&["MODEL", "VELOCITYGRID"], |p| {
                    let name = p.parse_quoted_string()?;
                    let identifiers = p.trailing_identifiers()?;
                    Ok((name, identifiers))
                })?;
                deformation_model = Some(DeformationModel { name, identifiers });
            }

            Ok(DynamicCrs {
                frame_reference_epoch,
                deformation_model,
            })
        })
        .map(|(_, d)| d)
    }

    pub(crate) fn parse_ellipsoid(&mut self) -> Result<Ellipsoid, ParseError> {
        self.bracketed(&["ELLIPSOID", "SPHEROID"], |p| {
            let name = p.parse_quoted_string()?;
            let semi_major_axis = p.comma_then(|p| p.parse_number())?;
            let inverse_flattening = p.comma_then(|p| p.parse_number())?;
            let (unit, identifiers) = p.parse_trailing_unit_and_identifiers()?;

            Ok(Ellipsoid {
                name,
                semi_major_axis,
                inverse_flattening,
                unit,
                identifiers,
            })
        })
        .map(|(_, e)| e)
    }

    pub(crate) fn parse_prime_meridian(&mut self) -> Result<PrimeMeridian, ParseError> {
        self.bracketed(&["PRIMEM", "PRIMEMERIDIAN"], |p| {
            let name = p.parse_quoted_string()?;
            let irm_longitude = p.comma_then(|p| p.parse_number())?;
            let (unit, identifiers) = p.parse_trailing_unit_and_identifiers()?;

            Ok(PrimeMeridian {
                name,
                irm_longitude,
                unit,
                identifiers,
            })
        })
        .map(|(_, pm)| pm)
    }
}
