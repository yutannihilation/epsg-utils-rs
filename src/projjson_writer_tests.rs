#[cfg(test)]
mod tests {
    use crate::parse_wkt2;

    fn parse_to_projjson(wkt: &str) -> serde_json::Value {
        parse_wkt2(wkt).unwrap().to_projjson()
    }

    #[test]
    fn minimal_structure() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian, 2]]"#,
        );

        assert_eq!(j["type"], "ProjectedCRS");
        assert_eq!(j["name"], "test");
        assert_eq!(j["base_crs"]["type"], "GeographicCRS");
        assert_eq!(j["base_crs"]["name"], "x");
        assert_eq!(j["base_crs"]["datum"]["type"], "GeodeticReferenceFrame");
        assert_eq!(j["base_crs"]["datum"]["name"], "d");
        assert_eq!(j["base_crs"]["datum"]["ellipsoid"]["name"], "e");
        assert_eq!(
            j["base_crs"]["datum"]["ellipsoid"]["semi_major_axis"],
            6378137.0
        );
        assert_eq!(
            j["base_crs"]["datum"]["ellipsoid"]["inverse_flattening"],
            298.257
        );
        assert_eq!(j["conversion"]["name"], "y");
        assert_eq!(j["conversion"]["method"]["name"], "m");
        assert_eq!(j["coordinate_system"]["subtype"], "Cartesian");
    }

    #[test]
    fn full_utm() {
        let j = parse_to_projjson(
            r#"PROJCRS["WGS 84 / UTM zone 31N",
                BASEGEOGCRS["WGS 84",
                    DATUM["World Geodetic System 1984",
                        ELLIPSOID["WGS 84",6378137,298.257223563]]],
                CONVERSION["UTM zone 31N",
                    METHOD["Transverse Mercator",ID["EPSG",9807]],
                    PARAMETER["Latitude of natural origin",0,ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8801]],
                    PARAMETER["Longitude of natural origin",3,ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8802]],
                    PARAMETER["Scale factor at natural origin",0.9996,SCALEUNIT["unity",1],ID["EPSG",8805]],
                    PARAMETER["False easting",500000,LENGTHUNIT["metre",1],ID["EPSG",8806]],
                    PARAMETER["False northing",0,LENGTHUNIT["metre",1],ID["EPSG",8807]]],
                CS[Cartesian,2],
                    AXIS["easting (E)",east,ORDER[1]],
                    AXIS["northing (N)",north,ORDER[2]],
                    LENGTHUNIT["metre",1],
                ID["EPSG",32631]]"#,
        );

        assert_eq!(j["name"], "WGS 84 / UTM zone 31N");

        // method ID
        assert_eq!(j["conversion"]["method"]["id"]["authority"], "EPSG");
        assert_eq!(j["conversion"]["method"]["id"]["code"], 9807);

        // parameters
        let params = j["conversion"]["parameters"].as_array().unwrap();
        assert_eq!(params.len(), 5);
        assert_eq!(params[0]["name"], "Latitude of natural origin");
        assert_eq!(params[0]["value"], 0.0);
        assert_eq!(params[0]["unit"], "degree"); // shorthand
        assert_eq!(params[0]["id"]["authority"], "EPSG");
        assert_eq!(params[0]["id"]["code"], 8801);

        assert_eq!(params[2]["unit"], "unity"); // shorthand for scale
        assert_eq!(params[3]["unit"], "metre"); // shorthand for length

        // axes
        let axes = j["coordinate_system"]["axis"].as_array().unwrap();
        assert_eq!(axes.len(), 2);
        assert_eq!(axes[0]["name"], "easting");
        assert_eq!(axes[0]["abbreviation"], "E");
        assert_eq!(axes[0]["direction"], "east");
        assert_eq!(axes[0]["unit"], "metre"); // from cs_unit

        // top-level ID
        assert_eq!(j["id"]["authority"], "EPSG");
        assert_eq!(j["id"]["code"], 32631);
    }

    #[test]
    fn unit_shorthand() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"],
                    PARAMETER["p1",1,LENGTHUNIT["metre",1]],
                    PARAMETER["p2",2,ANGLEUNIT["degree",0.0174532925199433]],
                    PARAMETER["p3",3,SCALEUNIT["unity",1]]],
                CS[Cartesian,2]]"#,
        );

        let params = j["conversion"]["parameters"].as_array().unwrap();
        assert_eq!(params[0]["unit"], "metre");
        assert_eq!(params[1]["unit"], "degree");
        assert_eq!(params[2]["unit"], "unity");
    }

    #[test]
    fn unit_full_object() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"],
                    PARAMETER["p1",1,ANGLEUNIT["grad",0.015707963267949]]],
                CS[Cartesian,2]]"#,
        );

        let unit = &j["conversion"]["parameters"][0]["unit"];
        assert_eq!(unit["type"], "AngularUnit");
        assert_eq!(unit["name"], "grad");
        assert_eq!(unit["conversion_factor"], 0.015707963267949);
    }

    #[test]
    fn axis_name_abbreviation_split() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                    AXIS["easting (E)",east],
                    AXIS["northing (N)",north]]"#,
        );

        let axes = j["coordinate_system"]["axis"].as_array().unwrap();
        assert_eq!(axes[0]["name"], "easting");
        assert_eq!(axes[0]["abbreviation"], "E");
        assert_eq!(axes[1]["name"], "northing");
        assert_eq!(axes[1]["abbreviation"], "N");
    }

    #[test]
    fn axis_no_abbreviation() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                    AXIS["easting",east],
                    AXIS["northing",north]]"#,
        );

        let axes = j["coordinate_system"]["axis"].as_array().unwrap();
        assert_eq!(axes[0]["name"], "easting");
        assert_eq!(axes[0]["abbreviation"], "");
    }

    #[test]
    fn dynamic_crs() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEODCRS["WGS 84",
                    DYNAMIC[FRAMEEPOCH[2010]],
                    DATUM["World Geodetic System 1984",
                        ELLIPSOID["WGS 84",6378137,298.257223563]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );

        assert_eq!(j["base_crs"]["type"], "GeodeticCRS");
        assert_eq!(
            j["base_crs"]["datum"]["type"],
            "DynamicGeodeticReferenceFrame"
        );
        assert_eq!(j["base_crs"]["datum"]["frame_reference_epoch"], 2010.0);
    }

    #[test]
    fn dynamic_crs_with_deformation_model() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEODCRS["NAD83",
                    DYNAMIC[FRAMEEPOCH[2010],MODEL["NAD83(CSRS)v6 velocity grid"]],
                    DATUM["NAD83",ELLIPSOID["GRS 1980",6378137,298.257222101]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );

        let models = j["base_crs"]["deformation_models"].as_array().unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0]["name"], "NAD83(CSRS)v6 velocity grid");
    }

    #[test]
    fn datum_ensemble() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["WGS 84",
                    ENSEMBLE["WGS 84 ensemble",
                        MEMBER["WGS 84 (TRANSIT)"],
                        MEMBER["WGS 84 (G730)",ID["EPSG",1152]],
                        ELLIPSOID["WGS 84",6378137,298.2572236,LENGTHUNIT["metre",1]],
                        ENSEMBLEACCURACY[2]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );

        assert!(j["base_crs"]["datum"].is_null());
        let ens = &j["base_crs"]["datum_ensemble"];
        assert_eq!(ens["type"], "DatumEnsemble");
        assert_eq!(ens["name"], "WGS 84 ensemble");
        assert_eq!(ens["accuracy"], "2"); // string per schema

        let members = ens["members"].as_array().unwrap();
        assert_eq!(members.len(), 2);
        assert_eq!(members[0]["name"], "WGS 84 (TRANSIT)");
        assert!(members[0].get("id").is_none());
        assert_eq!(members[1]["name"], "WGS 84 (G730)");
        assert_eq!(members[1]["id"]["authority"], "EPSG");
        assert_eq!(members[1]["id"]["code"], 1152);

        assert_eq!(ens["ellipsoid"]["name"], "WGS 84");
    }

    #[test]
    fn prime_meridian_with_unit() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["Tananarive 1925",
                    GEODETICDATUM["Tananarive 1925",
                        ELLIPSOID["International 1924",6378388,297,LENGTHUNIT["metre",1]]],
                    PRIMEM["Paris",2.5969213,ANGLEUNIT["grad",0.015707963267949]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );

        let pm = &j["base_crs"]["datum"]["prime_meridian"];
        assert_eq!(pm["name"], "Paris");
        // Has a non-default unit, so should be {value, unit}
        let lon = &pm["longitude"];
        assert_eq!(lon["value"], 2.5969213);
        assert_eq!(lon["unit"]["type"], "AngularUnit");
        assert_eq!(lon["unit"]["name"], "grad");
    }

    #[test]
    fn prime_meridian_no_unit() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["WGS 84",
                    TRF["WGS 1984",ELLIPSOID["WGS 84",6378137,298.257223563]],
                    PRIMEM["Greenwich",0]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );

        let pm = &j["base_crs"]["datum"]["prime_meridian"];
        assert_eq!(pm["name"], "Greenwich");
        assert_eq!(pm["longitude"], 0.0); // bare number (degrees implied)
    }

    #[test]
    fn ellipsoid_with_unit() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x",
                    DATUM["d",ELLIPSOID["WGS 84",6378137,298.257223563,LENGTHUNIT["metre",1]]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );

        let ell = &j["base_crs"]["datum"]["ellipsoid"];
        let sma = &ell["semi_major_axis"];
        // Has explicit unit, so should be {value, unit}
        assert_eq!(sma["value"], 6378137.0);
        assert_eq!(sma["unit"], "metre"); // shorthand
    }

    #[test]
    fn single_id_vs_multiple_ids() {
        // Single ID → "id"
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                ID["EPSG",32631]]"#,
        );
        assert!(j.get("ids").is_none());
        assert_eq!(j["id"]["authority"], "EPSG");
        assert_eq!(j["id"]["code"], 32631);

        // No ID → neither present
        let j2 = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );
        assert!(j2.get("id").is_none());
        assert!(j2.get("ids").is_none());
    }

    #[test]
    fn identifier_with_text_code_and_version() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2,ID["Authority","Abcd_Ef",7.1]]]"#,
        );

        let id = &j["coordinate_system"]["id"];
        assert_eq!(id["authority"], "Authority");
        assert_eq!(id["code"], "Abcd_Ef");
        assert_eq!(id["version"], 7.1);
    }

    #[test]
    fn identifier_with_uri() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2,ID["EPSG",4400,URI["urn:ogc:def:cs:EPSG::4400"]]]]"#,
        );

        let id = &j["coordinate_system"]["id"];
        assert_eq!(id["uri"], "urn:ogc:def:cs:EPSG::4400");
    }

    #[test]
    fn usage_with_area() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                USAGE[SCOPE["Engineering"],AREA["Netherlands"]]]"#,
        );

        let usages = j["usages"].as_array().unwrap();
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0]["scope"], "Engineering");
        assert_eq!(usages[0]["area"], "Netherlands");
    }

    #[test]
    fn usage_with_bbox() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                USAGE[SCOPE["Cadastre"],AREA["Finland"],BBOX[60.36,26.5,70.05,27.5]]]"#,
        );

        let bb = &j["usages"][0]["bbox"];
        assert_eq!(bb["south_latitude"], 60.36);
        assert_eq!(bb["west_longitude"], 26.5);
        assert_eq!(bb["north_latitude"], 70.05);
        assert_eq!(bb["east_longitude"], 27.5);
    }

    #[test]
    fn usage_with_vertical_extent() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                USAGE[SCOPE["Offshore"],VERTICALEXTENT[-1000,0,LENGTHUNIT["metre",1]]]]"#,
        );

        let ve = &j["usages"][0]["vertical_extent"];
        assert_eq!(ve["minimum"], -1000.0);
        assert_eq!(ve["maximum"], 0.0);
        assert_eq!(ve["unit"], "metre");
    }

    #[test]
    fn usage_with_temporal_extent() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                USAGE[SCOPE["Survey"],TIMEEXTENT[1976-01,2001-04]]]"#,
        );

        let te = &j["usages"][0]["temporal_extent"];
        assert_eq!(te["start"], "1976-01");
        assert_eq!(te["end"], "2001-04");
    }

    #[test]
    fn remark() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                REMARK["This is a test"]]"#,
        );

        assert_eq!(j["remarks"], "This is a test");
    }

    #[test]
    fn axis_with_meridian_and_range() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2],
                    AXIS["x",clockwise,BEARING[90],AXISMINVALUE[0],AXISMAXVALUE[360],RANGEMEANING[wraparound]],
                    AXIS["y",north,MERIDIAN[90,ANGLEUNIT["degree",0.0174532925199433]]]]"#,
        );

        let ax0 = &j["coordinate_system"]["axis"][0];
        assert!(ax0.get("meridian").is_none()); // no meridian on this axis
        assert_eq!(ax0["minimum_value"], 0.0);
        assert_eq!(ax0["maximum_value"], 360.0);
        assert_eq!(ax0["range_meaning"], "wraparound");

        let ax1 = &j["coordinate_system"]["axis"][1];
        assert_eq!(ax1["meridian"]["longitude"], 90.0);
        assert_eq!(ax1["meridian"]["unit"], "degree");
    }

    #[test]
    fn datum_with_anchor() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x",
                    DATUM["d",
                        ELLIPSOID["e",6378137,298.257],
                        ANCHOR["Station A"]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );

        assert_eq!(j["base_crs"]["datum"]["anchor"], "Station A");
    }

    #[test]
    fn datum_with_anchor_epoch() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x",
                    DATUM["d",
                        ELLIPSOID["e",6378137,298.257],
                        ANCHOREPOCH[2010]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );

        assert_eq!(j["base_crs"]["datum"]["anchor_epoch"], 2010.0);
    }

    #[test]
    fn schema_field_present() {
        let j = parse_to_projjson(
            r#"PROJCRS["test",
                BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
                CONVERSION["y", METHOD["m"]],
                CS[Cartesian,2]]"#,
        );

        assert!(
            j["$schema"]
                .as_str()
                .unwrap()
                .contains("projjson.schema.json")
        );
    }
}
