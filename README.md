epsg-utils
==========

[![Test](https://github.com/yutannihilation/epsg-utils-rs/actions/workflows/test.yml/badge.svg)](https://github.com/yutannihilation/epsg-utils-rs/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/epsg-utils.svg)](https://crates.io/crates/epsg-utils)
[![docs.rs](https://docs.rs/epsg-utils/badge.svg)](https://docs.rs/epsg-utils)

This crate provides three main capabilities:

1. **EPSG lookup** -- look up the WKT2 or PROJJSON representation of a CRS by its EPSG code.
2. **Parsing** -- parse OGC WKT2 strings or PROJJSON strings into structured Rust types.
3. **Conversion** -- convert between WKT2 and PROJJSON.

## Examples

### Look up EPSG code

```rust
// Get WKT2 representation (requires "wkt2-definitions" feature, enabled by default)
let wkt = epsg_utils::epsg_to_wkt2(6678).unwrap();

// Get PROJJSON representation (requires "projjson-definitions" feature, enabled by default)
let projjson = epsg_utils::epsg_to_projjson(6678).unwrap();
```

### Parse WKT2

```rust
let crs = epsg_utils::parse_wkt2(r#"PROJCRS["WGS 84 / UTM zone 31N",
    BASEGEOGCRS["WGS 84", DATUM["World Geodetic System 1984",
        ELLIPSOID["WGS 84", 6378137, 298.257223563]]],
    CONVERSION["UTM zone 31N", METHOD["Transverse Mercator"]],
    CS[Cartesian, 2],
    ID["EPSG", 32631]]"#).unwrap();

assert_eq!(crs.to_epsg(), Some(32631));
```

### Parse PROJJSON

```rust
let crs = epsg_utils::parse_projjson(projjson).unwrap();
assert_eq!(crs.name, "JGD2011 / Japan Plane Rectangular CS X");
```

### Convert between WKT2 and PROJJSON

```rust
let crs = epsg_utils::parse_wkt2(wkt).unwrap();

// To PROJJSON (serde_json::Value)
let projjson_value = crs.to_projjson();

// Back to WKT2
let wkt2 = crs.to_wkt2();
```

## EPSG Dataset

The definitions in this crate is based on the EPSG Dataset v12.054, and covers
99.5% (7365/7396) of the EPSG codes (engineering CRS and derived projected CRS
are not supported).

The EPSG Dataset is owned by the [International Association of Oil & Gas
Producers (IOGP)](https://www.iogp.org/). The source definitions included in
this crate were downloaded from <https://epsg.org/download-dataset.html>.

## References

- OGC WKT2: https://www.ogc.org/standards/wkt-crs/
- PROJJSON: https://proj.org/en/stable/specifications/projjson.html
  - v0.7: https://proj.org/en/latest/schemas/v0.7/projjson.schema.json

## Prior work

- [@developmentseed/epsg](https://github.com/developmentseed/deck.gl-raster/tree/main/packages/epsg): This gives me most of the ideas, from the existence of EPSG dataset and including gzip-compressed data.
- [crs-definitions](https://crates.io/crates/crs-definitions)
- [epsg](https://crates.io/crates/epsg)
