mod crs;
mod error;
mod projjson;
mod projjson_definitions;
mod wkt2;
mod wkt2_definitions;

pub use crs::{
    AuthorityId, Axis, BBox, BaseGeodeticCrs, BaseGeodeticCrsKeyword, CoordinateSystem, CsType,
    Datum, DatumEnsemble, DatumKeyword, DeformationModel, DynamicCrs, Ellipsoid, EnsembleMember,
    GeodeticReferenceFrame, Identifier, MapProjection, MapProjectionMethod, MapProjectionParameter,
    Meridian, PrimeMeridian, ProjectedCrs, RangeMeaning, TemporalExtent, Unit, UnitKeyword, Usage,
    VerticalExtent,
};
pub use error::ParseError;

/// Parse a WKT2 string into a [`ProjectedCrs`].
pub fn parse_wkt2(input: &str) -> Result<ProjectedCrs, ParseError> {
    wkt2::Parser::new(input).parse_projected_crs()
}

/// Parse a PROJJSON string into a [`ProjectedCrs`].
pub fn parse_projjson(input: &str) -> Result<ProjectedCrs, ParseError> {
    projjson::reader::parse_projjson(input)
}

/// Look up the WKT2 string for an EPSG projected CRS code.
///
/// Returns the static WKT2 string, or an error if the code is not found.
pub fn epsg_to_wkt2(code: i32) -> Result<&'static str, ParseError> {
    wkt2_definitions::lookup(code).ok_or(ParseError::UnknownEpsgCode { code })
}

/// Look up the PROJJSON string for an EPSG projected CRS code.
///
/// Returns the static PROJJSON string, or an error if the code is not found.
pub fn epsg_to_projjson(code: i32) -> Result<&'static str, ParseError> {
    projjson_definitions::lookup(code).ok_or(ParseError::UnknownEpsgCode { code })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epsg_to_wkt2_6678() {
        let wkt = epsg_to_wkt2(6678).unwrap();
        assert!(wkt.starts_with("PROJCRS["));
        // Verify it parses
        let crs = parse_wkt2(wkt).unwrap();
        assert_eq!(crs.name, "JGD2011 / Japan Plane Rectangular CS X");
    }

    #[test]
    fn epsg_to_projjson_6678() {
        let json = epsg_to_projjson(6678).unwrap();
        assert!(json.contains("\"ProjectedCRS\""));
        // Verify it parses
        let crs = parse_projjson(json).unwrap();
        assert_eq!(crs.name, "JGD2011 / Japan Plane Rectangular CS X");
    }

    #[test]
    fn epsg_unknown() {
        assert!(matches!(
            epsg_to_wkt2(99999),
            Err(ParseError::UnknownEpsgCode { code: 99999 })
        ));
        assert!(matches!(
            epsg_to_projjson(99999),
            Err(ParseError::UnknownEpsgCode { code: 99999 })
        ));
    }
}
