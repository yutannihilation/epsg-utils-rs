#[derive(Debug, PartialEq)]
pub struct ProjectedCrs {
    pub name: String,
    pub base_geodetic_crs: BaseGeodeticCrs,
    pub map_projection: MapProjection,
    pub coordinate_system: CoordinateSystem,
    pub scope_extent_identifier_remark: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct CoordinateSystem {
    pub cs_type: CsType,
    pub dimension: u8,
    pub identifiers: Vec<String>,
    pub axes: Vec<Axis>,
    pub cs_unit: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum CsType {
    Affine,
    Cartesian,
    Cylindrical,
    Ellipsoidal,
    Linear,
    Parametric,
    Polar,
    Spherical,
    Vertical,
    TemporalCount,
    TemporalMeasure,
    Ordinal,
    TemporalDateTime,
}

#[derive(Debug, PartialEq)]
pub struct Axis {
    pub name_abbrev: String,
    pub direction: String,
    pub meridian: Option<String>,
    pub bearing: Option<String>,
    pub order: Option<u32>,
    pub unit: Option<String>,
    pub identifiers: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct MapProjection {
    pub name: String,
    pub method: MapProjectionMethod,
    pub parameters: Vec<MapProjectionParameter>,
    pub identifiers: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct MapProjectionMethod {
    pub name: String,
    pub identifiers: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct MapProjectionParameter {
    pub name: String,
    pub value: f64,
    pub unit: Option<String>,
    pub identifiers: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum BaseGeodeticCrsKeyword {
    BaseGeodCrs,
    BaseGeogCrs,
}

#[derive(Debug, PartialEq)]
pub struct BaseGeodeticCrs {
    pub keyword: BaseGeodeticCrsKeyword,
    pub name: String,
    /// Present only for dynamic CRS (e.g. DYNAMIC[...])
    pub dynamic: Option<String>,
    /// Either a geodetic reference frame or datum ensemble (ENSEMBLE[...], still raw)
    pub datum: GeodeticReferenceFrame,
    /// Optional ellipsoidal CS unit (ANGLEUNIT[...])
    pub ellipsoidal_cs_unit: Option<String>,
    /// Zero or more ID[...] nodes
    pub identifiers: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum DatumKeyword {
    Datum,
    Trf,
    GeodeticDatum,
}

#[derive(Debug, PartialEq)]
pub struct GeodeticReferenceFrame {
    pub keyword: DatumKeyword,
    pub name: String,
    pub ellipsoid: Ellipsoid,
    pub anchor: Option<String>,
    pub anchor_epoch: Option<f64>,
    pub identifiers: Vec<String>,
    pub prime_meridian: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Ellipsoid {
    pub name: String,
    pub semi_major_axis: f64,
    pub inverse_flattening: f64,
    pub unit: Option<String>,
    pub identifiers: Vec<String>,
}
