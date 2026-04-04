mod crs;
mod error;
mod projjson;
#[cfg(feature = "projjson-definitions")]
mod projjson_definitions;
mod wkt2;
#[cfg(feature = "wkt2-definitions")]
mod wkt2_definitions;

pub use crs::{
    AuthorityId, Axis, BBox, BaseGeodeticCrs, BaseGeodeticCrsKeyword, CompoundCrs,
    CoordinateSystem, Crs, CsType, Datum, DatumEnsemble, DatumKeyword, DeformationModel,
    DynamicCrs, Ellipsoid, EnsembleMember, GeodCrs, GeodCrsKeyword, GeodeticReferenceFrame,
    GeogCrs, GeogCrsKeyword, GeoidModel, Identifier, MapProjection, MapProjectionMethod,
    MapProjectionParameter, Meridian, PrimeMeridian, ProjectedCrs, RangeMeaning, SingleCrs,
    TemporalExtent, Unit, UnitKeyword, Usage, VertCrs, VertCrsKeyword, VerticalDatum,
    VerticalExtent, VerticalReferenceFrame, VerticalReferenceFrameKeyword,
};
pub use error::ParseError;

/// Parse a WKT2 string into a [`Crs`].
pub fn parse_wkt2(input: &str) -> Result<Crs, ParseError> {
    wkt2::Parser::new(input).parse_crs()
}

/// Parse a PROJJSON string into a [`ProjectedCrs`].
pub fn parse_projjson(input: &str) -> Result<ProjectedCrs, ParseError> {
    projjson::reader::parse_projjson(input)
}

/// Look up the WKT2 string for an EPSG projected CRS code.
///
/// Returns the static WKT2 string, or an error if the code is not found.
#[cfg(feature = "wkt2-definitions")]
pub fn epsg_to_wkt2(code: i32) -> Result<&'static str, ParseError> {
    wkt2_definitions::lookup(code).ok_or(ParseError::UnknownEpsgCode { code })
}

/// Look up the PROJJSON string for an EPSG projected CRS code.
///
/// Returns the static PROJJSON string, or an error if the code is not found.
#[cfg(feature = "projjson-definitions")]
pub fn epsg_to_projjson(code: i32) -> Result<&'static str, ParseError> {
    projjson_definitions::lookup(code).ok_or(ParseError::UnknownEpsgCode { code })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_epsg_found() {
        let crs = parse_wkt2(
            r#"PROJCRS["test",
                BASEGEOGCRS["x",DATUM["d",ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y",METHOD["m"]],
                CS[Cartesian,2],
                ID["EPSG",32631]]"#,
        )
        .unwrap();
        assert_eq!(crs.to_epsg(), Some(32631));
    }

    #[test]
    fn to_epsg_not_found() {
        let crs = parse_wkt2(
            r#"PROJCRS["test",
                BASEGEOGCRS["x",DATUM["d",ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y",METHOD["m"]],
                CS[Cartesian,2]]"#,
        )
        .unwrap();
        assert_eq!(crs.to_epsg(), None);
    }

    #[test]
    #[cfg(feature = "wkt2-definitions")]
    fn epsg_to_wkt2_6678() {
        let wkt = epsg_to_wkt2(6678).unwrap();
        assert!(wkt.starts_with("PROJCRS["));
        let Crs::ProjectedCrs(crs) = parse_wkt2(wkt).unwrap() else {
            panic!("expected ProjectedCrs");
        };
        assert_eq!(crs.name, "JGD2011 / Japan Plane Rectangular CS X");
    }

    #[test]
    #[cfg(feature = "projjson-definitions")]
    fn epsg_to_projjson_6678() {
        let json = epsg_to_projjson(6678).unwrap();
        assert!(json.contains("\"ProjectedCRS\""));
        let crs = parse_projjson(json).unwrap();
        assert_eq!(crs.name, "JGD2011 / Japan Plane Rectangular CS X");
    }

    #[test]
    #[cfg(feature = "wkt2-definitions")]
    fn epsg_to_wkt2_unknown() {
        assert!(matches!(
            epsg_to_wkt2(99999),
            Err(ParseError::UnknownEpsgCode { code: 99999 })
        ));
    }

    #[test]
    #[cfg(feature = "projjson-definitions")]
    fn epsg_to_projjson_unknown() {
        assert!(matches!(
            epsg_to_projjson(99999),
            Err(ParseError::UnknownEpsgCode { code: 99999 })
        ));
    }
}
