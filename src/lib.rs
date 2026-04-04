mod crs;
mod error;
mod projjson;
mod wkt2;

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
