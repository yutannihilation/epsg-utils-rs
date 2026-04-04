#[derive(Debug, PartialEq)]
pub struct ProjectedCrs {
    pub name: String,
    pub base_geodetic_crs: BaseGeodeticCrs,
    pub map_projection: MapProjection,
    pub coordinate_system: String,
    pub scope_extent_identifier_remark: Vec<String>,
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
    /// Either a geodetic reference frame (DATUM[...]) or datum ensemble (ENSEMBLE[...])
    pub datum: String,
    /// Optional ellipsoidal CS unit (ANGLEUNIT[...])
    pub ellipsoidal_cs_unit: Option<String>,
    /// Zero or more ID[...] nodes
    pub identifiers: Vec<String>,
}
