#[derive(Debug, PartialEq)]
pub struct ProjectedCrs {
    pub name: String,
    pub base_geodetic_crs: BaseGeodeticCrs,
    pub map_projection: String,
    pub coordinate_system: String,
    pub scope_extent_identifier_remark: Vec<String>,
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
