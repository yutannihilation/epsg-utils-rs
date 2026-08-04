epsg-utils
==========

[![Test](https://github.com/yutannihilation/epsg-utils-rs/actions/workflows/test.yml/badge.svg)](https://github.com/yutannihilation/epsg-utils-rs/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/epsg-utils.svg)](https://crates.io/crates/epsg-utils)
[![docs.rs](https://docs.rs/epsg-utils/badge.svg)](https://docs.rs/epsg-utils)

This crate provides three main capabilities:

1. **EPSG lookup** -- look up the WKT2 or PROJJSON representation of a CRS by its EPSG code.
2. **Parsing** -- parse OGC WKT2 strings, WKT1 strings (GDAL and ESRI dialects), or PROJJSON strings into structured Rust types.
3. **Conversion** -- convert between WKT2 and PROJJSON (and from WKT1 to either).

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

### Parse WKT1 (GDAL / ESRI dialect)

```rust
// An ESRI .prj string
let crs = epsg_utils::parse_wkt1(r#"PROJCS["WGS_1984_Web_Mercator_Auxiliary_Sphere",
    GEOGCS["GCS_WGS_1984",DATUM["D_WGS_1984",
        SPHEROID["WGS_1984",6378137.0,298.257223563]],
    PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]],
    PROJECTION["Mercator_Auxiliary_Sphere"],
    PARAMETER["False_Easting",0.0],PARAMETER["False_Northing",0.0],
    PARAMETER["Central_Meridian",0.0],PARAMETER["Standard_Parallel_1",0.0],
    PARAMETER["Auxiliary_Sphere_Type",0.0],UNIT["Meter",1.0]]"#).unwrap();

// Method/parameter/datum names are normalized to their EPSG equivalents,
// so the result converts cleanly to WKT2 or PROJJSON.
let wkt2 = crs.to_wkt2();
```

Datum, ellipsoid, and unit names are restored to official EPSG names via
embedded ESRI alias tables; CRS names are kept as written.

`parse_wkt1` fails on nodes that cannot be represented without data loss
(`TOWGS84`, `EXTENSION`, `METADATA`); use `parse_wkt1_lossy` to discard them
instead.

### Parse PROJJSON

```rust
let crs = epsg_utils::parse_projjson(projjson).unwrap();
assert_eq!(crs.name, "JGD2024 / Japan Plane Rectangular CS X");
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

The definitions in this crate is based on the EPSG Dataset v12.055, and covers
99.6% (7392/7423) of the EPSG codes (engineering CRS and derived projected CRS
are not supported).

The EPSG Dataset is owned by the [International Association of Oil & Gas
Producers (IOGP)](https://www.iogp.org/). The source definitions included in
this crate were downloaded from <https://epsg.org/download-dataset.html>.

## WKT1 name mappings

The WKT1 parser embeds two kinds of static tables:

- Projection method / parameter name mappings transcribed from
  [PROJ](https://github.com/OSGeo/PROJ) (MIT License).
- ESRI object-name aliases (datum, ellipsoid, unit names) extracted from
  PROJ's `esri.sql`, which is derived from ESRI's
  [projection-engine-db-doc](https://github.com/Esri/projection-engine-db-doc)
  (Apache License 2.0).

## References

- OGC WKT2: https://www.ogc.org/standards/wkt-crs/
- PROJJSON: https://proj.org/en/stable/specifications/projjson.html
  - v0.7: https://proj.org/en/latest/schemas/v0.7/projjson.schema.json

## Prior work

- [@developmentseed/epsg](https://github.com/developmentseed/deck.gl-raster/tree/main/packages/epsg): This gives me most of the ideas, from the existence of EPSG dataset and including gzip-compressed data.
- [crs-definitions](https://crates.io/crates/crs-definitions)
- [epsg](https://crates.io/crates/epsg)
