mod error;
mod parser;
mod projjson_writer;
#[cfg(test)]
mod projjson_writer_tests;
mod wkt2;
mod wkt2_writer;
#[cfg(test)]
mod wkt2_writer_tests;

pub use error::ParseError;
pub use wkt2::{
    AuthorityId, Axis, BBox, BaseGeodeticCrs, BaseGeodeticCrsKeyword, CoordinateSystem, CsType,
    Datum, DatumEnsemble, DatumKeyword, DeformationModel, DynamicCrs, Ellipsoid, EnsembleMember,
    GeodeticReferenceFrame, Identifier, MapProjection, MapProjectionMethod, MapProjectionParameter,
    Meridian, PrimeMeridian, ProjectedCrs, RangeMeaning, TemporalExtent, Unit, UnitKeyword, Usage,
    VerticalExtent,
};

/// Parse a WKT2 string into a [`ProjectedCrs`].
pub fn parse_wkt2(input: &str) -> Result<ProjectedCrs, ParseError> {
    parser::Parser::new(input).parse_projected_crs()
}
