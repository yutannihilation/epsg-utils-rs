//! Parser for WKT1, covering the GDAL dialect (deriving from OGC 01-009) and
//! the ESRI dialect (as written to `.prj` files).

pub(crate) mod esri_alias_data;
pub(crate) mod mappings;
pub(crate) mod mappings_data;
mod parser;

pub(crate) use parser::parse;

#[cfg(test)]
mod tests;
