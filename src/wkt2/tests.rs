#[cfg(test)]
mod tests {
    use crate::parse_wkt2;

    /// Parse WKT2, emit it back, parse again, and assert the two structs are equal.
    fn assert_roundtrip(wkt: &str) {
        let parsed = parse_wkt2(wkt).unwrap();
        let emitted = parsed.to_string();
        let reparsed = parse_wkt2(&emitted)
            .unwrap_or_else(|e| panic!("Failed to re-parse emitted WKT2:\n{emitted}\nError: {e}"));
        assert_eq!(
            parsed, reparsed,
            "Round-trip mismatch.\nOriginal:\n{wkt}\nEmitted:\n{emitted}"
        );
    }

    #[test]
    fn roundtrip_minimal() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian, 2]]"#,
        );
    }

    #[test]
    fn roundtrip_full_utm() {
        assert_roundtrip(
            r#"PROJCRS["WGS 84 / UTM zone 31N",
                BASEGEOGCRS["WGS 84",
                    DATUM["World Geodetic System 1984",
                        ELLIPSOID["WGS 84",6378137,298.257223563,LENGTHUNIT["metre",1]]]],
                CONVERSION["UTM zone 31N",
                    METHOD["Transverse Mercator",ID["EPSG",9807]],
                    PARAMETER["Latitude of natural origin",0,
                        ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8801]],
                    PARAMETER["Longitude of natural origin",3,
                        ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8802]],
                    PARAMETER["Scale factor at natural origin",0.9996,
                        SCALEUNIT["unity",1],ID["EPSG",8805]],
                    PARAMETER["False easting",500000,
                        LENGTHUNIT["metre",1],ID["EPSG",8806]],
                    PARAMETER["False northing",0,
                        LENGTHUNIT["metre",1],ID["EPSG",8807]]],
                CS[Cartesian,2],
                    AXIS["easting (E)",east,ORDER[1]],
                    AXIS["northing (N)",north,ORDER[2]],
                    LENGTHUNIT["metre",1],
                ID["EPSG",32631]]"#,
        );
    }

    #[test]
    fn roundtrip_with_axes_units() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["WGS 84", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                    AXIS["easting",east,LENGTHUNIT["metre",1]],
                    AXIS["northing",north,LENGTHUNIT["metre",1]]]"#,
        );
    }

    #[test]
    fn roundtrip_ellipsoidal_cs() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["WGS 84", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[ellipsoidal,2],
                    AXIS["latitude",north,ORDER[1],ANGLEUNIT["degree",0.0174532925199433]],
                    AXIS["longitude",east,ORDER[2],ANGLEUNIT["degree",0.0174532925199433]]]"#,
        );
    }

    #[test]
    fn roundtrip_with_meridian() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["WGS 84", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                    AXIS["x",north,MERIDIAN[90,ANGLEUNIT["degree",0.0174532925199433]]],
                    AXIS["y",north,MERIDIAN[0,ANGLEUNIT["degree",0.0174532925199433]]]]"#,
        );
    }

    #[test]
    fn roundtrip_bearing_and_range() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                    AXIS["x",clockwise,BEARING[90],ORDER[1],AXISMINVALUE[0],AXISMAXVALUE[360],RANGEMEANING[wraparound]],
                    AXIS["y",clockwise,BEARING[0],ORDER[2]]]"#,
        );
    }

    #[test]
    fn roundtrip_dynamic_crs() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEODCRS["WGS 84",
                    DYNAMIC[FRAMEEPOCH[2010]],
                    DATUM["World Geodetic System 1984",
                        ELLIPSOID["WGS 84",6378137,298.257223563]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );
    }

    #[test]
    fn roundtrip_dynamic_with_model() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEODCRS["NAD83",
                    DYNAMIC[FRAMEEPOCH[2010],MODEL["NAD83(CSRS)v6 velocity grid"]],
                    DATUM["NAD83",ELLIPSOID["GRS 1980",6378137,298.257222101]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );
    }

    #[test]
    fn roundtrip_datum_with_anchor() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["Tananarive 1925",
                    GEODETICDATUM["Tananarive 1925",
                        ELLIPSOID["International 1924",6378388,297,LENGTHUNIT["metre",1]],
                        ANCHOR["Tananarive observatory"]],
                    PRIMEM["Paris",2.5969213,ANGLEUNIT["grad",0.015707963267949]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );
    }

    #[test]
    fn roundtrip_datum_with_anchor_epoch() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["NAD83",
                    DATUM["NAD83 (NSRS2011)",
                        ELLIPSOID["GRS 1980",6378137,298.257222101,LENGTHUNIT["metre",1]],
                        ANCHOREPOCH[2010]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );
    }

    #[test]
    fn roundtrip_trf_keyword() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["WGS 84",
                    TRF["World Geodetic System 1984",
                        ELLIPSOID["WGS 84",6378388,298.257223563,LENGTHUNIT["metre",1]]],
                    PRIMEM["Greenwich",0]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );
    }

    #[test]
    fn roundtrip_geodetic_datum_ensemble() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["WGS 84",
                    ENSEMBLE["WGS 84 ensemble",
                        MEMBER["WGS 84 (TRANSIT)"],
                        MEMBER["WGS 84 (G730)",ID["EPSG",1152]],
                        MEMBER["WGS 84 (G834)"],
                        ELLIPSOID["WGS 84",6378137,298.2572236,LENGTHUNIT["metre",1]],
                        ENSEMBLEACCURACY[2]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );
    }

    #[test]
    fn roundtrip_vertical_datum_ensemble() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["x",
                    ENSEMBLE["EVRS ensemble",
                        MEMBER["EVRF2000"],
                        MEMBER["EVRF2007"],
                        ENSEMBLEACCURACY[0.01]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );
    }

    #[test]
    fn roundtrip_base_crs_with_unit_and_id() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["WGS 84",
                    DATUM["WGS 1984",ELLIPSOID["WGS 84",6378137,298.257223563]],
                    ANGLEUNIT["degree",0.0174532925199433],
                    ID["EPSG",4326]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );
    }

    #[test]
    fn roundtrip_cs_with_id() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2,ID["EPSG",4400]],
                    AXIS["easting",east],
                    AXIS["northing",north],
                    LENGTHUNIT["metre",1]]"#,
        );
    }

    #[test]
    fn roundtrip_usage_and_remark() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                USAGE[SCOPE["Engineering survey"],AREA["Netherlands"]],
                USAGE[SCOPE["Cadastre"],AREA["Germany"]],
                ID["EPSG",32631],
                REMARK["This is a test CRS"]]"#,
        );
    }

    #[test]
    fn roundtrip_usage_with_bbox() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                USAGE[SCOPE["Cadastre"],AREA["Finland"],BBOX[60.36,26.5,70.05,27.5]]]"#,
        );
    }

    #[test]
    fn roundtrip_usage_with_vertical_extent() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                USAGE[SCOPE["Offshore"],VERTICALEXTENT[-1000,0,LENGTHUNIT["metre",1]]]]"#,
        );
    }

    #[test]
    fn roundtrip_usage_with_temporal_dates() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                USAGE[SCOPE["Spatial referencing"],TIMEEXTENT[1976-01,2001-04]]]"#,
        );
    }

    #[test]
    fn roundtrip_usage_with_temporal_quoted() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                USAGE[SCOPE["Geology"],TIMEEXTENT["Jurassic","Quaternary"]]]"#,
        );
    }

    #[test]
    fn roundtrip_identifier_with_version_and_uri() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2,ID["EPSG",4400,URI["urn:ogc:def:cs:EPSG::4400"]]]]"#,
        );
    }

    #[test]
    fn roundtrip_identifier_text_with_version() {
        assert_roundtrip(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2,ID["Authority name","Abcd_Ef",7.1]]]"#,
        );
    }

    // -----------------------------------------------------------------------
    // GEOGCRS roundtrips
    // -----------------------------------------------------------------------

    #[test]
    fn roundtrip_geogcrs_minimal() {
        assert_roundtrip(
            r#"GEOGCRS["WGS 84",
                DATUM["World Geodetic System 1984",
                    ELLIPSOID["WGS 84",6378137,298.257223563]],
                CS[ellipsoidal,2],
                    AXIS["latitude",north],
                    AXIS["longitude",east],
                    ANGLEUNIT["degree",0.0174532925199433]]"#,
        );
    }

    #[test]
    fn roundtrip_geogcrs_with_prime_meridian() {
        assert_roundtrip(
            r#"GEOGCRS["NTF (Paris)",
                DATUM["Nouvelle Triangulation Francaise",
                    ELLIPSOID["Clarke 1880 (IGN)",6378249.2,293.4660213]],
                PRIMEM["Paris",2.5969213,ANGLEUNIT["grad",0.015707963267949]],
                CS[ellipsoidal,2],
                    AXIS["latitude",north],
                    AXIS["longitude",east],
                    ANGLEUNIT["grad",0.015707963267949]]"#,
        );
    }

    #[test]
    fn roundtrip_geogcrs_dynamic() {
        assert_roundtrip(
            r#"GEOGCRS["ITRF2014",
                DYNAMIC[FRAMEEPOCH[2010]],
                TRF["International Terrestrial Reference Frame 2014",
                    ELLIPSOID["GRS 1980",6378137,298.257222101]],
                CS[ellipsoidal,3],
                    AXIS["latitude",north],
                    AXIS["longitude",east],
                    AXIS["ellipsoidal height",up],
                    ANGLEUNIT["degree",0.0174532925199433]]"#,
        );
    }

    #[test]
    fn roundtrip_geogcrs_with_ensemble() {
        assert_roundtrip(
            r#"GEOGCRS["WGS 84",
                ENSEMBLE["WGS 84 ensemble",
                    MEMBER["WGS 84 (TRANSIT)"],
                    MEMBER["WGS 84 (G730)"],
                    ELLIPSOID["WGS 84",6378137,298.257223563],
                    ENSEMBLEACCURACY[2]],
                CS[ellipsoidal,2],
                    AXIS["latitude",north],
                    AXIS["longitude",east],
                    ANGLEUNIT["degree",0.0174532925199433],
                ID["EPSG",4326]]"#,
        );
    }

    #[test]
    fn roundtrip_geogcrs_with_usage_and_remark() {
        assert_roundtrip(
            r#"GEOGCRS["WGS 84",
                DATUM["WGS 1984",ELLIPSOID["WGS 84",6378137,298.257223563]],
                CS[ellipsoidal,2],
                    AXIS["latitude",north],
                    AXIS["longitude",east],
                    ANGLEUNIT["degree",0.0174532925199433],
                USAGE[SCOPE["Horizontal component of 3D system"],
                    AREA["World"],BBOX[-90,-180,90,180]],
                ID["EPSG",4326],
                REMARK["WGS 84 geographic 2D"]]"#,
        );
    }

    #[test]
    fn roundtrip_geogcrs_datum_with_anchor() {
        assert_roundtrip(
            r#"GEOGCRS["Tananarive 1925",
                GEODETICDATUM["Tananarive 1925",
                    ELLIPSOID["International 1924",6378388,297,LENGTHUNIT["metre",1]],
                    ANCHOR["Tananarive observatory"]],
                PRIMEM["Paris",2.5969213,ANGLEUNIT["grad",0.015707963267949]],
                CS[ellipsoidal,2],
                    AXIS["latitude",north],
                    AXIS["longitude",east],
                    ANGLEUNIT["grad",0.015707963267949]]"#,
        );
    }

    #[test]
    fn roundtrip_geographiccrs_keyword() {
        assert_roundtrip(
            r#"GEOGRAPHICCRS["WGS 84",
                DATUM["WGS 1984",ELLIPSOID["WGS 84",6378137,298.257223563]],
                CS[ellipsoidal,2],
                    AXIS["latitude",north],
                    AXIS["longitude",east],
                    ANGLEUNIT["degree",0.0174532925199433]]"#,
        );
    }
}
