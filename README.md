epsg-utils
==========

This crate contains 3 major features:

1. Provide EPSG database (i.e. returns the corresponding OGC WKT2 / PROJJSON representation to a given EPSG)
2. Convert a OGC WKT2 from/to a PROJJSON
3. Extract ESPG code from OGC WKT2 / PROJJSON

## EPSG Dataset

The EPSG Dataset is owned by the [International Association of Oil & Gas
Producers (IOGP)](https://www.iogp.org/). The source definitions included in
this crate were downloaded from <https://epsg.org/download-dataset.html>.

## References

- OGC WKT2: https://www.ogc.org/standards/wkt-crs/
- PROJJSON: https://proj.org/en/stable/specifications/projjson.html
  - v0.6: https://proj.org/en/latest/schemas/v0.6/projjson.schema.json
