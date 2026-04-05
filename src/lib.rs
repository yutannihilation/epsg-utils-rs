//! Utilities for working with EPSG coordinate reference system definitions.
//!
//! This crate provides three main capabilities:
//!
//! 1. **EPSG lookup** -- look up the WKT2 or PROJJSON representation of a CRS
//!    by its EPSG code (via [`epsg_to_wkt2`] and [`epsg_to_projjson`]).
//! 2. **Parsing** -- parse OGC WKT2 strings ([`parse_wkt2`]) or PROJJSON
//!    strings ([`parse_projjson`]) into structured Rust types.
//! 3. **Conversion** -- convert between WKT2 and PROJJSON using
//!    [`Crs::to_wkt2`] and [`Crs::to_projjson`].
//!
//! # Crate structure
//!
//! The top-level [`Crs`] enum is the entry point for all parsed CRS data. It
//! dispatches to one of the concrete CRS types:
//!
//! - [`ProjectedCrs`] -- a projected CRS (`PROJCRS`)
//! - [`GeogCrs`] -- a geographic CRS (`GEOGCRS`)
//! - [`GeodCrs`] -- a geodetic CRS (`GEODCRS`)
//! - [`VertCrs`] -- a vertical CRS (`VERTCRS`)
//! - [`CompoundCrs`] -- a compound CRS (`COMPOUNDCRS`)
//!
//! These types and their components (datums, coordinate systems, ellipsoids,
//! etc.) live in the [`crs`] module and are all publicly accessible.
//!
//! # Features
//!
//! - **`wkt2-definitions`** (enabled by default) -- embeds compressed WKT2
//!   strings for all supported EPSG codes, enabling [`epsg_to_wkt2`].
//! - **`projjson-definitions`** (enabled by default) -- embeds compressed
//!   PROJJSON strings for all supported EPSG codes, enabling [`epsg_to_projjson`].
//!
//! # Examples
//!
//! ## Look up an EPSG code
//!
//! ```
//! # #[cfg(feature = "wkt2-definitions")]
//! # {
//! let wkt = epsg_utils::epsg_to_wkt2(6678).unwrap();
//! # }
//! # #[cfg(feature = "projjson-definitions")]
//! # {
//! let projjson = epsg_utils::epsg_to_projjson(6678).unwrap();
//! # }
//! ```
//!
//! ## Parse WKT2
//!
//! ```
//! let crs = epsg_utils::parse_wkt2(r#"PROJCRS["WGS 84 / UTM zone 31N",
//!     BASEGEOGCRS["WGS 84", DATUM["World Geodetic System 1984",
//!         ELLIPSOID["WGS 84", 6378137, 298.257223563]]],
//!     CONVERSION["UTM zone 31N", METHOD["Transverse Mercator"]],
//!     CS[Cartesian, 2],
//!     ID["EPSG", 32631]]"#).unwrap();
//!
//! assert_eq!(crs.to_epsg(), Some(32631));
//! ```
//!
//! ## Parse PROJJSON
//!
//! ```
//! # #[cfg(feature = "projjson-definitions")]
//! # {
//! let projjson = epsg_utils::epsg_to_projjson(6678).unwrap();
//! let crs = epsg_utils::parse_projjson(projjson).unwrap();
//! assert_eq!(crs.name, "JGD2011 / Japan Plane Rectangular CS X");
//! # }
//! ```
//!
//! ## Convert between WKT2 and PROJJSON
//!
//! ```
//! # #[cfg(feature = "wkt2-definitions")]
//! # {
//! # let wkt = epsg_utils::epsg_to_wkt2(6678).unwrap();
//! let crs = epsg_utils::parse_wkt2(wkt).unwrap();
//!
//! // To PROJJSON (serde_json::Value)
//! let projjson_value = crs.to_projjson();
//!
//! // Back to WKT2
//! let wkt2 = crs.to_wkt2();
//! # }
//! ```

pub mod crs;
mod error;
mod projjson;
#[cfg(feature = "projjson-definitions")]
mod projjson_definitions;
mod wkt2;
#[cfg(feature = "wkt2-definitions")]
mod wkt2_definitions;

pub use crs::{CompoundCrs, Crs, GeodCrs, GeogCrs, ProjectedCrs, VertCrs};
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
