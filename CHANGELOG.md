# Changelog

<!-- next-header -->
## [Unreleased] (ReleaseDate)

### New features

- **WKT1 parsing**: `parse_wkt1()` and `parse_wkt1_lossy()` parse WKT1
  strings — both the GDAL dialect (deriving from OGC 01-009) and the ESRI
  dialect (`.prj` files), detected automatically — into the same `Crs` types
  as `parse_wkt2()`, enabling WKT1 → WKT2 / PROJJSON conversion. Method and
  parameter names are mapped to their EPSG equivalents, and datum, ellipsoid,
  and unit names (e.g. `D_NAD_1983_2011`) are restored to official EPSG names
  via embedded alias tables; CRS names are kept as written. The strict
  `parse_wkt1()` errors on nodes that cannot be represented without data loss
  (`TOWGS84`, `EXTENSION`, `METADATA`); `parse_wkt1_lossy()` discards them.
  Validated against PROJ's WKT1 output for the whole embedded EPSG dataset
  (13,816 strings, > 99.5% exact match). Mapping tables are transcribed from
  [PROJ](https://github.com/OSGeo/PROJ) (MIT); the ESRI aliases derive from
  [projection-engine-db-doc](https://github.com/Esri/projection-engine-db-doc)
  (Apache 2.0).

### Breaking changes

- `ParseError` has six new variants (emitted only by the WKT1 parser). The
  enum is not `#[non_exhaustive]`, so exhaustive `match`es on `ParseError`
  need new arms even if you never call `parse_wkt1()`; code using a `_` arm
  or `Display` is unaffected. No other existing API changed.

## [v0.0.3] (2026-07-13)

- Update the EPSG Dataset to v12.057

## [v0.0.2] (2026-04-22)

- Update the EPSG Dataset to v12.055

## v0.0.1 (2026-04-05)

- Initial release

<!-- next-url -->
[Unreleased]: https://github.com/yutannihilation/epsg-utils-rs/compare/v0.0.3...HEAD
[v0.0.3]: https://github.com/yutannihilation/epsg-utils-rs/compare/v0.0.2...v0.0.3
[v0.0.2]: https://github.com/yutannihilation/rusqlite-gpkg/compare/v0.0.1...v0.0.2
