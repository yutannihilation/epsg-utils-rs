use super::*;
use crate::crs::{
    AuthorityId, BaseGeodeticCrsKeyword, CsType, Datum, DatumKeyword, RangeMeaning, UnitKeyword,
};

#[test]
fn parse_static_geogcrs() {
    let wkt = r#"PROJCRS["WGS 84 / UTM zone 31N",
        BASEGEOGCRS["WGS 84", DATUM["World Geodetic System 1984", ELLIPSOID["WGS 84",6378137,298.257223563]]],
        CONVERSION["UTM zone 31N", METHOD["Transverse Mercator"]],
        CS[Cartesian, 2],
            AXIS["easting (E)", east, ORDER[1]],
            AXIS["northing (N)", north, ORDER[2]],
            LENGTHUNIT["metre", 1.0]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    assert_eq!(result.name, "WGS 84 / UTM zone 31N");
    let base = &result.base_geodetic_crs;
    assert_eq!(base.keyword, BaseGeodeticCrsKeyword::BaseGeogCrs);
    assert_eq!(base.name, "WGS 84");
    assert!(base.dynamic.is_none());
    let Datum::ReferenceFrame(ref rf) = base.datum else {
        panic!("expected ReferenceFrame");
    };
    assert_eq!(rf.keyword, DatumKeyword::Datum);
    assert_eq!(rf.name, "World Geodetic System 1984");
    assert_eq!(rf.ellipsoid.name, "WGS 84");
    assert_eq!(rf.ellipsoid.semi_major_axis, 6378137.0);
    assert_eq!(rf.ellipsoid.inverse_flattening, 298.257223563);

    let cs = &result.coordinate_system;
    assert_eq!(cs.cs_type, CsType::Cartesian);
    assert_eq!(cs.dimension, 2);
    assert_eq!(cs.axes.len(), 2);
    assert_eq!(cs.axes[0].name_abbrev, "easting (E)");
    assert_eq!(cs.axes[0].direction, "east");
    assert_eq!(cs.axes[0].order, Some(1));
    assert_eq!(cs.axes[1].name_abbrev, "northing (N)");
    assert_eq!(cs.axes[1].direction, "north");
    assert_eq!(cs.axes[1].order, Some(2));
    let cs_unit = cs.cs_unit.as_ref().unwrap();
    assert_eq!(cs_unit.keyword, UnitKeyword::LengthUnit);
    assert_eq!(cs_unit.name, "metre");
    assert_eq!(cs_unit.conversion_factor, Some(1.0));
}

#[test]
fn parse_cs_with_axis_units() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["WGS 84", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2],
            AXIS["easting", east, LENGTHUNIT["metre", 1.0]],
            AXIS["northing", north, LENGTHUNIT["metre", 1.0]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let cs = &result.coordinate_system;
    assert_eq!(cs.axes.len(), 2);
    assert_eq!(
        cs.axes[0].unit.as_ref().unwrap().keyword,
        UnitKeyword::LengthUnit
    );
    assert_eq!(
        cs.axes[1].unit.as_ref().unwrap().keyword,
        UnitKeyword::LengthUnit
    );
    assert!(cs.cs_unit.is_none());
}

#[test]
fn parse_cs_ellipsoidal() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["WGS 84", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[ellipsoidal, 2],
            AXIS["latitude", north, ORDER[1], ANGLEUNIT["degree", 0.0174532925199433]],
            AXIS["longitude", east, ORDER[2], ANGLEUNIT["degree", 0.0174532925199433]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let cs = &result.coordinate_system;
    assert_eq!(cs.cs_type, CsType::Ellipsoidal);
    assert_eq!(cs.dimension, 2);
    assert_eq!(cs.axes[0].direction, "north");
    assert_eq!(
        cs.axes[0].unit.as_ref().unwrap().keyword,
        UnitKeyword::AngleUnit
    );
}

#[test]
fn parse_cs_with_meridian() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["WGS 84", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2],
            AXIS["x", north, MERIDIAN[90, ANGLEUNIT["degree", 0.0174532925199433]]],
            AXIS["y", north, MERIDIAN[0, ANGLEUNIT["degree", 0.0174532925199433]]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let cs = &result.coordinate_system;
    let m0 = cs.axes[0].meridian.as_ref().unwrap();
    assert_eq!(m0.value, 90.0);
    assert_eq!(m0.unit.keyword, UnitKeyword::AngleUnit);
    let m1 = cs.axes[1].meridian.as_ref().unwrap();
    assert_eq!(m1.value, 0.0);
    assert_eq!(m1.unit.keyword, UnitKeyword::AngleUnit);
}

#[test]
fn parse_axis_bearing_and_range() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2],
            AXIS["x", clockwise, BEARING[90], ORDER[1],
                AXISMINVALUE[0], AXISMAXVALUE[360], RANGEMEANING[wraparound]],
            AXIS["y", clockwise, BEARING[0], ORDER[2]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let ax0 = &result.coordinate_system.axes[0];
    assert_eq!(ax0.direction, "clockwise");
    assert_eq!(ax0.bearing, Some(90.0));
    assert_eq!(ax0.order, Some(1));
    assert_eq!(ax0.axis_min_value, Some(0.0));
    assert_eq!(ax0.axis_max_value, Some(360.0));
    assert_eq!(ax0.range_meaning, Some(RangeMeaning::Wraparound));

    let ax1 = &result.coordinate_system.axes[1];
    assert_eq!(ax1.bearing, Some(0.0));
    assert!(ax1.axis_min_value.is_none());
    assert!(ax1.range_meaning.is_none());
}

#[test]
fn parse_cs_no_axes() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let cs = &result.coordinate_system;
    assert_eq!(cs.cs_type, CsType::Cartesian);
    assert_eq!(cs.dimension, 2);
    assert!(cs.axes.is_empty());
    assert!(cs.cs_unit.is_none());
}

#[test]
fn parse_cs_with_id() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2, ID["EPSG", 4400]],
            AXIS["easting", east],
            AXIS["northing", north],
            LENGTHUNIT["metre", 1.0]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let cs = &result.coordinate_system;
    assert_eq!(cs.identifiers.len(), 1);
    assert_eq!(cs.identifiers[0].authority_name, "EPSG");
    assert_eq!(cs.axes.len(), 2);
}

#[test]
fn parse_dynamic_geodcrs() {
    let wkt = r#"PROJCRS["test",
        BASEGEODCRS["WGS 84",
            DYNAMIC[FRAMEEPOCH[2010.0]],
            DATUM["World Geodetic System 1984", ELLIPSOID["WGS 84",6378137,298.257223563]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let base = &result.base_geodetic_crs;
    assert_eq!(base.keyword, BaseGeodeticCrsKeyword::BaseGeodCrs);
    let dynamic = base.dynamic.as_ref().unwrap();
    assert_eq!(dynamic.frame_reference_epoch, 2010.0);
    assert!(dynamic.deformation_model.is_none());
    assert!(matches!(base.datum, Datum::ReferenceFrame(_)));
}

#[test]
fn parse_dynamic_with_deformation_model() {
    let wkt = r#"PROJCRS["test",
        BASEGEODCRS["NAD83",
            DYNAMIC[FRAMEEPOCH[2010.0],MODEL["NAD83(CSRS)v6 velocity grid"]],
            DATUM["NAD83", ELLIPSOID["GRS 1980",6378137,298.257222101]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let dynamic = result.base_geodetic_crs.dynamic.as_ref().unwrap();
    assert_eq!(dynamic.frame_reference_epoch, 2010.0);
    let model = dynamic.deformation_model.as_ref().unwrap();
    assert_eq!(model.name, "NAD83(CSRS)v6 velocity grid");
    assert!(model.identifiers.is_empty());
}

#[test]
fn parse_base_crs_with_unit_and_id() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["WGS 84",
            DATUM["WGS 1984", ELLIPSOID["WGS 84",6378137,298.257223563]],
            ANGLEUNIT["degree", 0.0174532925199433],
            ID["EPSG", 4326]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let base = &result.base_geodetic_crs;
    let eu = base.ellipsoidal_cs_unit.as_ref().unwrap();
    assert_eq!(eu.keyword, UnitKeyword::AngleUnit);
    assert_eq!(eu.name, "degree");
    assert_eq!(base.identifiers.len(), 1);
    assert_eq!(base.identifiers[0].authority_name, "EPSG");
}

#[test]
fn parse_geodetic_datum_ensemble() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["WGS 84",
            ENSEMBLE["WGS 84 ensemble",
                MEMBER["WGS 84 (TRANSIT)"],
                MEMBER["WGS 84 (G730)", ID["EPSG",1152]],
                MEMBER["WGS 84 (G834)"],
                ELLIPSOID["WGS 84",6378137,298.2572236,LENGTHUNIT["metre",1.0]],
                ENSEMBLEACCURACY[2]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let Datum::Ensemble(ref ens) = result.base_geodetic_crs.datum else {
        panic!("expected Ensemble");
    };
    assert_eq!(ens.name, "WGS 84 ensemble");
    assert_eq!(ens.members.len(), 3);
    assert_eq!(ens.members[0].name, "WGS 84 (TRANSIT)");
    assert!(ens.members[0].identifiers.is_empty());
    assert_eq!(ens.members[1].name, "WGS 84 (G730)");
    assert_eq!(ens.members[1].identifiers.len(), 1);
    assert_eq!(ens.ellipsoid.as_ref().unwrap().name, "WGS 84");
    assert_eq!(ens.accuracy, 2.0);
    assert!(ens.prime_meridian.is_none());
}

#[test]
fn parse_vertical_datum_ensemble() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x",
            ENSEMBLE["EVRS ensemble",
                MEMBER["EVRF2000"],
                MEMBER["EVRF2007"],
                ENSEMBLEACCURACY[0.01]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let Datum::Ensemble(ref ens) = result.base_geodetic_crs.datum else {
        panic!("expected Ensemble");
    };
    assert_eq!(ens.name, "EVRS ensemble");
    assert_eq!(ens.members.len(), 2);
    assert!(ens.ellipsoid.is_none());
    assert_eq!(ens.accuracy, 0.01);
}

#[test]
fn parse_datum_with_anchor() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["Tananarive 1925",
            GEODETICDATUM["Tananarive 1925",
                ELLIPSOID["International 1924",6378388.0,297.0,LENGTHUNIT["metre",1.0]],
                ANCHOR["Tananarive observatory:21.0191667gS, 50.23849537gE of Paris"]],
            PRIMEM["Paris",2.5969213,ANGLEUNIT["grad",0.015707963267949]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let Datum::ReferenceFrame(ref rf) = result.base_geodetic_crs.datum else {
        panic!("expected ReferenceFrame");
    };
    assert_eq!(rf.keyword, DatumKeyword::GeodeticDatum);
    assert_eq!(rf.name, "Tananarive 1925");
    assert_eq!(rf.ellipsoid.name, "International 1924");
    assert_eq!(rf.ellipsoid.semi_major_axis, 6378388.0);
    assert_eq!(rf.ellipsoid.inverse_flattening, 297.0);
    assert_eq!(
        rf.ellipsoid.unit.as_ref().unwrap().keyword,
        UnitKeyword::LengthUnit
    );
    assert_eq!(
        rf.anchor.as_deref(),
        Some("Tananarive observatory:21.0191667gS, 50.23849537gE of Paris")
    );
    let pm = rf.prime_meridian.as_ref().unwrap();
    assert_eq!(pm.name, "Paris");
    assert_eq!(pm.irm_longitude, 2.5969213);
    assert_eq!(pm.unit.as_ref().unwrap().keyword, UnitKeyword::AngleUnit);
    assert_eq!(pm.unit.as_ref().unwrap().name, "grad");
}

#[test]
fn parse_datum_with_anchor_epoch() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["NAD83",
            DATUM["NAD83 (National Spatial Reference System 2011)",
                ELLIPSOID["GRS 1980",6378137,298.257222101,LENGTHUNIT["metre",1.0]],
                ANCHOREPOCH[2010.0]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let Datum::ReferenceFrame(ref rf) = result.base_geodetic_crs.datum else {
        panic!("expected ReferenceFrame");
    };
    assert_eq!(rf.anchor_epoch, Some(2010.0));
    assert!(rf.anchor.is_none());
}

#[test]
fn parse_datum_trf_keyword() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["WGS 84",
            TRF["World Geodetic System 1984",
                ELLIPSOID["WGS 84",6378388.0,298.257223563,LENGTHUNIT["metre",1.0]]],
            PRIMEM["Greenwich",0.0]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let Datum::ReferenceFrame(ref rf) = result.base_geodetic_crs.datum else {
        panic!("expected ReferenceFrame");
    };
    assert_eq!(rf.keyword, DatumKeyword::Trf);
    let pm = rf.prime_meridian.as_ref().unwrap();
    assert_eq!(pm.name, "Greenwich");
    assert_eq!(pm.irm_longitude, 0.0);
    assert!(pm.unit.is_none());
}

#[test]
fn parse_map_projection_with_parameters() {
    let wkt = r#"PROJCRS["WGS 84 / UTM zone 10N",
        BASEGEOGCRS["WGS 84", DATUM["WGS 1984", ELLIPSOID["WGS 84",6378137,298.257223563]]],
        CONVERSION["UTM zone 10N",
            METHOD["Transverse Mercator", ID["EPSG",9807]],
            PARAMETER["Latitude of natural origin",0,
                ANGLEUNIT["degree",0.0174532925199433],
                ID["EPSG",8801]],
            PARAMETER["Longitude of natural origin",-123,
                ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8802]],
            PARAMETER["Scale factor at natural origin",0.9996,
                SCALEUNIT["unity",1.0],ID["EPSG",8805]],
            PARAMETER["False easting",500000,
                LENGTHUNIT["metre",1.0],ID["EPSG",8806]],
            PARAMETER["False northing",0,LENGTHUNIT["metre",1.0],ID["EPSG",8807]]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let proj = &result.map_projection;
    assert_eq!(proj.name, "UTM zone 10N");
    assert_eq!(proj.method.name, "Transverse Mercator");
    assert_eq!(proj.method.identifiers.len(), 1);
    assert_eq!(proj.parameters.len(), 5);

    assert_eq!(proj.parameters[0].name, "Latitude of natural origin");
    assert_eq!(proj.parameters[0].value, 0.0);
    assert_eq!(
        proj.parameters[0].unit.as_ref().unwrap().keyword,
        UnitKeyword::AngleUnit
    );
    assert_eq!(proj.parameters[0].identifiers.len(), 1);

    assert_eq!(proj.parameters[1].name, "Longitude of natural origin");
    assert_eq!(proj.parameters[1].value, -123.0);

    assert_eq!(proj.parameters[2].name, "Scale factor at natural origin");
    assert_eq!(proj.parameters[2].value, 0.9996);
    assert_eq!(
        proj.parameters[2].unit.as_ref().unwrap().keyword,
        UnitKeyword::ScaleUnit
    );

    assert_eq!(proj.parameters[3].name, "False easting");
    assert_eq!(proj.parameters[3].value, 500000.0);
    assert_eq!(
        proj.parameters[3].unit.as_ref().unwrap().keyword,
        UnitKeyword::LengthUnit
    );

    assert_eq!(proj.parameters[4].name, "False northing");
    assert_eq!(proj.parameters[4].value, 0.0);
}

#[test]
fn parse_map_projection_with_conversion_id() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["WGS 84", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["UTM zone 10N",
            METHOD["Transverse Mercator"],
            PARAMETER["False easting",500000,LENGTHUNIT["metre",1.0]],
            ID["EPSG",16010]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let proj = &result.map_projection;
    assert_eq!(proj.parameters.len(), 1);
    assert_eq!(proj.identifiers.len(), 1);
    assert_eq!(proj.identifiers[0].authority_name, "EPSG");
}

#[test]
fn parse_map_projection_method_only() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["Transverse Mercator"]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let proj = &result.map_projection;
    assert_eq!(proj.name, "y");
    assert_eq!(proj.method.name, "Transverse Mercator");
    assert!(proj.parameters.is_empty());
    assert!(proj.identifiers.is_empty());
}

#[test]
fn parse_projcrs_with_trailing_nodes() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2],
            AXIS["easting", east],
            AXIS["northing", north],
            LENGTHUNIT["metre", 1.0],
        ID["EPSG", 32631]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    assert_eq!(result.coordinate_system.axes.len(), 2);
    assert_eq!(result.identifiers.len(), 1);
    assert_eq!(result.identifiers[0].authority_name, "EPSG");
    assert_eq!(
        result.identifiers[0].authority_unique_id,
        AuthorityId::Number(32631.0)
    );
}

#[test]
fn reject_projectedcrs() {
    let wkt = r#"PROJECTEDCRS["test", BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]], CONVERSION["y", METHOD["m"]], CS[Cartesian, 2]]"#;
    let mut parser = Parser::new(wkt);
    let err = parser.parse_projected_crs().unwrap_err();
    assert!(matches!(err, ParseError::UnexpectedKeyword { .. }));
    assert!(err.to_string().contains("PROJECTEDCRS"));
}

#[test]
fn reject_wrong_keyword() {
    let wkt = r#"GEOGCRS["test"]"#;
    let mut parser = Parser::new(wkt);
    let err = parser.parse_projected_crs().unwrap_err();
    assert!(matches!(err, ParseError::ExpectedKeyword { .. }));
}

#[test]
fn parse_usage_and_remark() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2],
        USAGE[SCOPE["Engineering survey"], AREA["Netherlands"]],
        USAGE[SCOPE["Cadastre"], AREA["Germany"]],
        ID["EPSG", 32631],
        REMARK["This is a test CRS"]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    assert_eq!(result.usages.len(), 2);
    assert_eq!(result.usages[0].scope, "Engineering survey");
    assert_eq!(result.usages[0].area.as_deref(), Some("Netherlands"));
    assert_eq!(result.usages[1].scope, "Cadastre");
    assert_eq!(result.usages[1].area.as_deref(), Some("Germany"));
    assert_eq!(result.identifiers.len(), 1);
    assert_eq!(result.remark.as_deref(), Some("This is a test CRS"));
}

#[test]
fn parse_usage_with_bbox_and_temporal() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2],
        USAGE[SCOPE["Spatial referencing."],
            AREA["Netherlands offshore."],TIMEEXTENT[1976-01,2001-04]],
        USAGE[SCOPE["Cadastre."],
            AREA["Finland - onshore between 26 and 27."],
            BBOX[60.36,26.5,70.05,27.5]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    assert_eq!(result.usages.len(), 2);

    let u0 = &result.usages[0];
    assert_eq!(u0.scope, "Spatial referencing.");
    assert_eq!(u0.area.as_deref(), Some("Netherlands offshore."));
    let te = u0.temporal_extent.as_ref().unwrap();
    assert_eq!(te.start, "1976-01");
    assert_eq!(te.end, "2001-04");
    assert!(u0.bbox.is_none());

    let u1 = &result.usages[1];
    assert_eq!(u1.scope, "Cadastre.");
    let bb = u1.bbox.as_ref().unwrap();
    assert_eq!(bb.lower_left_latitude, 60.36);
    assert_eq!(bb.lower_left_longitude, 26.5);
    assert_eq!(bb.upper_right_latitude, 70.05);
    assert_eq!(bb.upper_right_longitude, 27.5);
    assert!(u1.temporal_extent.is_none());
}

#[test]
fn parse_usage_with_vertical_extent() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2],
        USAGE[SCOPE["Offshore engineering."],
            VERTICALEXTENT[-1000,0,LENGTHUNIT["metre",1.0]]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let ve = result.usages[0].vertical_extent.as_ref().unwrap();
    assert_eq!(ve.minimum_height, -1000.0);
    assert_eq!(ve.maximum_height, 0.0);
    assert_eq!(ve.unit.as_ref().unwrap().keyword, UnitKeyword::LengthUnit);
}

#[test]
fn parse_usage_with_temporal_quoted() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2],
        USAGE[SCOPE["Geology."],
            TIMEEXTENT["Jurassic","Quaternary"]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let te = result.usages[0].temporal_extent.as_ref().unwrap();
    assert_eq!(te.start, "Jurassic");
    assert_eq!(te.end, "Quaternary");
}

#[test]
fn trailing_input_error() {
    let wkt = r#"PROJCRS["test", BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]], CONVERSION["y", METHOD["m"]], CS[Cartesian, 2]] extra"#;
    let mut parser = Parser::new(wkt);
    let err = parser.parse_crs().unwrap_err();
    assert!(matches!(err, ParseError::TrailingInput { .. }));
}

#[test]
fn parse_identifier_number_id() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257],
            ID["EPSG",6326]]],
        CONVERSION["y", METHOD["m", ID["EPSG",9807]]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let Datum::ReferenceFrame(ref rf) = result.base_geodetic_crs.datum else {
        panic!("expected ReferenceFrame");
    };
    assert_eq!(rf.identifiers.len(), 1);
    assert_eq!(rf.identifiers[0].authority_name, "EPSG");
    assert_eq!(
        rf.identifiers[0].authority_unique_id,
        AuthorityId::Number(6326.0)
    );

    assert_eq!(
        result.map_projection.method.identifiers[0].authority_name,
        "EPSG"
    );
    assert_eq!(
        result.map_projection.method.identifiers[0].authority_unique_id,
        AuthorityId::Number(9807.0)
    );
}

#[test]
fn parse_identifier_with_version_and_uri() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2, ID["EPSG",4400,URI["urn:ogc:def:cs:EPSG::4400"]]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let id = &result.coordinate_system.identifiers[0];
    assert_eq!(id.authority_name, "EPSG");
    assert_eq!(id.authority_unique_id, AuthorityId::Number(4400.0));
    assert_eq!(id.uri.as_deref(), Some("urn:ogc:def:cs:EPSG::4400"));
}

#[test]
fn parse_identifier_text_id_with_version() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2, ID["Authority name","Abcd_Ef",7.1]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_projected_crs().unwrap();

    let id = &result.coordinate_system.identifiers[0];
    assert_eq!(id.authority_name, "Authority name");
    assert_eq!(
        id.authority_unique_id,
        AuthorityId::Text("Abcd_Ef".to_string())
    );
    assert_eq!(id.version, Some(AuthorityId::Number(7.1)));
}

// ---------------------------------------------------------------------------
// GEOGCRS parsing
// ---------------------------------------------------------------------------

#[test]
fn parse_geogcrs_static() {
    let wkt = r#"GEOGCRS["WGS 84",
        DATUM["World Geodetic System 1984",
            ELLIPSOID["WGS 84",6378137,298.257223563,LENGTHUNIT["metre",1]]],
        CS[ellipsoidal,2],
            AXIS["latitude",north,ORDER[1],ANGLEUNIT["degree",0.0174532925199433]],
            AXIS["longitude",east,ORDER[2],ANGLEUNIT["degree",0.0174532925199433]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_geog_crs().unwrap();

    assert_eq!(result.name, "WGS 84");
    assert!(result.dynamic.is_none());
    let Datum::ReferenceFrame(ref rf) = result.datum else {
        panic!("expected ReferenceFrame");
    };
    assert_eq!(rf.keyword, DatumKeyword::Datum);
    assert_eq!(rf.name, "World Geodetic System 1984");
    assert_eq!(rf.ellipsoid.semi_major_axis, 6378137.0);
    assert!(rf.prime_meridian.is_none());

    let cs = &result.coordinate_system;
    assert_eq!(cs.cs_type, CsType::Ellipsoidal);
    assert_eq!(cs.dimension, 2);
    assert_eq!(cs.axes.len(), 2);
    assert_eq!(cs.axes[0].direction, "north");
    assert_eq!(cs.axes[1].direction, "east");
}

#[test]
fn parse_geogcrs_with_prime_meridian() {
    let wkt = r#"GEOGCRS["NTF (Paris)",
        DATUM["Nouvelle Triangulation Francaise",
            ELLIPSOID["Clarke 1880 (IGN)",6378249.2,293.4660213]],
        PRIMEM["Paris",2.5969213,ANGLEUNIT["grad",0.015707963267949]],
        CS[ellipsoidal,2],
            AXIS["latitude",north],
            AXIS["longitude",east],
            ANGLEUNIT["grad",0.015707963267949]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_geog_crs().unwrap();

    assert_eq!(result.name, "NTF (Paris)");
    let Datum::ReferenceFrame(ref rf) = result.datum else {
        panic!("expected ReferenceFrame");
    };
    let pm = rf.prime_meridian.as_ref().unwrap();
    assert_eq!(pm.name, "Paris");
    assert_eq!(pm.irm_longitude, 2.5969213);
}

#[test]
fn parse_geogcrs_dynamic() {
    let wkt = r#"GEOGCRS["ITRF2014",
        DYNAMIC[FRAMEEPOCH[2010]],
        TRF["International Terrestrial Reference Frame 2014",
            ELLIPSOID["GRS 1980",6378137,298.257222101]],
        CS[ellipsoidal,3],
            AXIS["latitude",north],
            AXIS["longitude",east],
            AXIS["ellipsoidal height",up],
            ANGLEUNIT["degree",0.0174532925199433]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_geog_crs().unwrap();

    assert_eq!(result.name, "ITRF2014");
    let d = result.dynamic.as_ref().unwrap();
    assert_eq!(d.frame_reference_epoch, 2010.0);
    let Datum::ReferenceFrame(ref rf) = result.datum else {
        panic!("expected ReferenceFrame");
    };
    assert_eq!(rf.keyword, DatumKeyword::Trf);
    assert_eq!(result.coordinate_system.dimension, 3);
}

#[test]
fn parse_geogcrs_with_ensemble() {
    let wkt = r#"GEOGCRS["WGS 84",
        ENSEMBLE["WGS 84 ensemble",
            MEMBER["WGS 84 (TRANSIT)"],
            MEMBER["WGS 84 (G730)"],
            ELLIPSOID["WGS 84",6378137,298.257223563],
            ENSEMBLEACCURACY[2]],
        CS[ellipsoidal,2],
            AXIS["latitude",north],
            AXIS["longitude",east],
            ANGLEUNIT["degree",0.0174532925199433],
        ID["EPSG",4326]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_geog_crs().unwrap();

    let Datum::Ensemble(ref ens) = result.datum else {
        panic!("expected Ensemble");
    };
    assert_eq!(ens.members.len(), 2);
    assert_eq!(result.identifiers.len(), 1);
    assert_eq!(result.identifiers[0].authority_name, "EPSG");
}

#[test]
fn parse_geogcrs_via_parse_crs() {
    let wkt = r#"GEOGCRS["WGS 84",
        DATUM["WGS 1984",ELLIPSOID["WGS 84",6378137,298.257223563]],
        CS[ellipsoidal,2],
            AXIS["latitude",north],
            AXIS["longitude",east],
            ANGLEUNIT["degree",0.0174532925199433]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_crs().unwrap();
    assert!(matches!(result, crate::crs::Crs::GeogCrs(_)));
}

#[test]
fn parse_projcrs_via_parse_crs() {
    let wkt = r#"PROJCRS["test",
        BASEGEOGCRS["x", DATUM["d", ELLIPSOID["e",6378137,298.257]]],
        CONVERSION["y", METHOD["m"]],
        CS[Cartesian, 2]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_crs().unwrap();
    assert!(matches!(result, crate::crs::Crs::ProjectedCrs(_)));
}

#[test]
fn parse_geographiccrs_keyword() {
    let wkt = r#"GEOGRAPHICCRS["WGS 84",
        DATUM["WGS 1984",ELLIPSOID["WGS 84",6378137,298.257223563]],
        CS[ellipsoidal,2],
            AXIS["latitude",north],
            AXIS["longitude",east],
            ANGLEUNIT["degree",0.0174532925199433]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_geog_crs().unwrap();
    assert_eq!(result.keyword, crate::crs::GeogCrsKeyword::GeographicCrs);
}

// ---------------------------------------------------------------------------
// GEODCRS parsing
// ---------------------------------------------------------------------------

#[test]
fn parse_geodcrs_cartesian() {
    let wkt = r#"GEODCRS["WGS 84 (geocentric)",
        DATUM["World Geodetic System 1984",
            ELLIPSOID["WGS 84",6378137,298.257223563,LENGTHUNIT["metre",1]]],
        CS[Cartesian,3],
            AXIS["(X)",geocentricX],
            AXIS["(Y)",geocentricY],
            AXIS["(Z)",geocentricZ],
            LENGTHUNIT["metre",1]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_geod_crs().unwrap();

    assert_eq!(result.keyword, crate::crs::GeodCrsKeyword::GeodCrs);
    assert_eq!(result.name, "WGS 84 (geocentric)");
    let Datum::ReferenceFrame(ref rf) = result.datum else {
        panic!("expected ReferenceFrame");
    };
    assert_eq!(rf.name, "World Geodetic System 1984");

    let cs = &result.coordinate_system;
    assert_eq!(cs.cs_type, CsType::Cartesian);
    assert_eq!(cs.dimension, 3);
    assert_eq!(cs.axes.len(), 3);
}

#[test]
fn parse_geodeticcrs_keyword() {
    let wkt = r#"GEODETICCRS["WGS 84 (geocentric)",
        DATUM["WGS 1984",ELLIPSOID["WGS 84",6378137,298.257223563]],
        CS[Cartesian,3],
            AXIS["(X)",geocentricX],
            AXIS["(Y)",geocentricY],
            AXIS["(Z)",geocentricZ],
            LENGTHUNIT["metre",1]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_geod_crs().unwrap();
    assert_eq!(result.keyword, crate::crs::GeodCrsKeyword::GeodeticCrs);
}

#[test]
fn parse_geodcrs_via_parse_crs() {
    let wkt = r#"GEODCRS["WGS 84 (geocentric)",
        DATUM["WGS 1984",ELLIPSOID["WGS 84",6378137,298.257223563]],
        CS[Cartesian,3],
            AXIS["(X)",geocentricX],
            AXIS["(Y)",geocentricY],
            AXIS["(Z)",geocentricZ],
            LENGTHUNIT["metre",1]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_crs().unwrap();
    assert!(matches!(result, crate::crs::Crs::GeodCrs(_)));
}

#[test]
fn parse_geodcrs_dynamic() {
    let wkt = r#"GEODCRS["ITRF2014 (geocentric)",
        DYNAMIC[FRAMEEPOCH[2010]],
        TRF["International Terrestrial Reference Frame 2014",
            ELLIPSOID["GRS 1980",6378137,298.257222101]],
        CS[Cartesian,3],
            AXIS["(X)",geocentricX],
            AXIS["(Y)",geocentricY],
            AXIS["(Z)",geocentricZ],
            LENGTHUNIT["metre",1]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_geod_crs().unwrap();

    assert!(result.dynamic.is_some());
    assert_eq!(
        result.dynamic.as_ref().unwrap().frame_reference_epoch,
        2010.0
    );
}

// ---------------------------------------------------------------------------
// VERTCRS parsing
// ---------------------------------------------------------------------------

#[test]
fn parse_vertcrs_static() {
    let wkt = r#"VERTCRS["NAVD88",
        VDATUM["North American Vertical Datum 1988"],
        CS[vertical,1],
            AXIS["gravity-related height (H)",up],
            LENGTHUNIT["metre",1]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_vert_crs().unwrap();

    assert_eq!(result.keyword, crate::crs::VertCrsKeyword::VertCrs);
    assert_eq!(result.name, "NAVD88");
    assert!(result.dynamic.is_none());
    let crate::crs::VerticalDatum::ReferenceFrame(ref rf) = result.datum else {
        panic!("expected ReferenceFrame");
    };
    assert_eq!(
        rf.keyword,
        crate::crs::VerticalReferenceFrameKeyword::VDatum
    );
    assert_eq!(rf.name, "North American Vertical Datum 1988");
    assert_eq!(result.coordinate_system.cs_type, CsType::Vertical);
    assert_eq!(result.coordinate_system.dimension, 1);
    assert!(result.geoid_models.is_empty());
}

#[test]
fn parse_vertcrs_with_geoid_model() {
    let wkt = r#"VERTCRS["CGVD2013",
        VRF["Canadian Geodetic Vertical Datum of 2013"],
        CS[vertical,1],
            AXIS["gravity-related height (H)",up],
            LENGTHUNIT["metre",1],
        GEOIDMODEL["CGG2013",ID["EPSG",6648]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_vert_crs().unwrap();

    assert_eq!(result.geoid_models.len(), 1);
    assert_eq!(result.geoid_models[0].name, "CGG2013");
    assert_eq!(result.geoid_models[0].identifiers.len(), 1);
    let crate::crs::VerticalDatum::ReferenceFrame(ref rf) = result.datum else {
        panic!("expected ReferenceFrame");
    };
    assert_eq!(rf.keyword, crate::crs::VerticalReferenceFrameKeyword::Vrf);
}

#[test]
fn parse_vertcrs_dynamic() {
    let wkt = r#"VERTCRS["RH2000",
        DYNAMIC[FRAMEEPOCH[2000],MODEL["NKG2016LU"]],
        VDATUM["Rikets Hojdsystem 2000"],
        CS[vertical,1],
            AXIS["gravity-related height (H)",up],
            LENGTHUNIT["metre",1]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_vert_crs().unwrap();

    let d = result.dynamic.as_ref().unwrap();
    assert_eq!(d.frame_reference_epoch, 2000.0);
    assert_eq!(d.deformation_model.as_ref().unwrap().name, "NKG2016LU");
}

#[test]
fn parse_vertcrs_with_anchor() {
    let wkt = r#"VERTCRS["test",
        VERTICALDATUM["Newlyn",ANCHOR["Mean Sea Level 1915 to 1921."]],
        CS[vertical,1],
            AXIS["gravity-related height (H)",up],
            LENGTHUNIT["metre",1]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_vert_crs().unwrap();

    let crate::crs::VerticalDatum::ReferenceFrame(ref rf) = result.datum else {
        panic!("expected ReferenceFrame");
    };
    assert_eq!(
        rf.keyword,
        crate::crs::VerticalReferenceFrameKeyword::VerticalDatum
    );
    assert_eq!(rf.anchor.as_deref(), Some("Mean Sea Level 1915 to 1921."));
}

#[test]
fn parse_vertcrs_via_parse_crs() {
    let wkt = r#"VERTCRS["NAVD88",
        VDATUM["North American Vertical Datum 1988"],
        CS[vertical,1],
            AXIS["gravity-related height (H)",up],
            LENGTHUNIT["metre",1]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_crs().unwrap();
    assert!(matches!(result, crate::crs::Crs::VertCrs(_)));
}

// ---------------------------------------------------------------------------
// COMPOUNDCRS parsing
// ---------------------------------------------------------------------------

#[test]
fn parse_compoundcrs_geog_vert() {
    let wkt = r#"COMPOUNDCRS["NAD83 + NAVD88",
        GEOGCRS["NAD83",
            DATUM["North American Datum 1983",
                ELLIPSOID["GRS 1980",6378137,298.257222101,LENGTHUNIT["metre",1]]],
            PRIMEMERIDIAN["Greenwich",0],
            CS[ellipsoidal,2],
                AXIS["latitude",north,ORDER[1]],
                AXIS["longitude",east,ORDER[2]],
                ANGLEUNIT["degree",0.0174532925199433]],
        VERTCRS["NAVD88",
            VDATUM["North American Vertical Datum 1983"],
            CS[vertical,1],
                AXIS["gravity-related height (H)",up],
                LENGTHUNIT["metre",1]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_compound_crs().unwrap();

    assert_eq!(result.name, "NAD83 + NAVD88");
    assert_eq!(result.components.len(), 2);
    assert!(matches!(
        result.components[0],
        crate::crs::SingleCrs::GeogCrs(_)
    ));
    assert!(matches!(
        result.components[1],
        crate::crs::SingleCrs::VertCrs(_)
    ));
}

#[test]
fn parse_compoundcrs_with_unsupported_component() {
    let wkt = r#"COMPOUNDCRS["2D GPS position with civil time",
        GEOGCRS["WGS 84",
            DATUM["World Geodetic System 1984",
                ELLIPSOID["WGS 84",6378137,298.257223563]],
            CS[ellipsoidal,2],
                AXIS["(lat)",north],
                AXIS["(lon)",east],
                ANGLEUNIT["degree",0.0174532925199433]],
        TIMECRS["DateTime",
            TDATUM["Gregorian Calendar"],
            CS[TemporalDateTime,1],AXIS["Time (T)",future]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_compound_crs().unwrap();

    assert_eq!(result.components.len(), 2);
    assert!(matches!(
        result.components[0],
        crate::crs::SingleCrs::GeogCrs(_)
    ));
    assert!(matches!(
        result.components[1],
        crate::crs::SingleCrs::Other(_)
    ));
}

#[test]
fn parse_compoundcrs_via_parse_crs() {
    let wkt = r#"COMPOUNDCRS["test",
        GEOGCRS["WGS 84",
            DATUM["WGS 1984",ELLIPSOID["WGS 84",6378137,298.257223563]],
            CS[ellipsoidal,2],
                AXIS["latitude",north],
                AXIS["longitude",east],
                ANGLEUNIT["degree",0.0174532925199433]],
        VERTCRS["NAVD88",
            VDATUM["NAVD88"],
            CS[vertical,1],
                AXIS["gravity-related height (H)",up],
                LENGTHUNIT["metre",1]]]"#;

    let mut parser = Parser::new(wkt);
    let result = parser.parse_crs().unwrap();
    assert!(matches!(result, crate::crs::Crs::CompoundCrs(_)));
}
