#[derive(Debug, PartialEq)]
pub struct ProjectedCrs {
    pub name: String,
    pub base_geodetic_crs: String,
    pub map_projection: String,
    pub coordinate_system: String,
    pub scope_extent_identifier_remark: Vec<String>,
}
