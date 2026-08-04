//! Tests for the WKT1 (GDAL/ESRI dialect) parser.
//!
//! Fixtures were generated with `projinfo -o WKT1_GDAL|WKT1_ESRI -q
//! --single-line EPSG:<code>` (PROJ 9.x with its database), then committed
//! here so the tests run without PROJ.

use crate::crs::*;
use crate::{ParseError, parse_wkt1, parse_wkt1_lossy};

// ---------------------------------------------------------------------------
// GDAL-dialect fixtures
// ---------------------------------------------------------------------------

const GDAL_32631: &str = r#"PROJCS["WGS 84 / UTM zone 31N",GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563,AUTHORITY["EPSG","7030"]],AUTHORITY["EPSG","6326"]],PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],AUTHORITY["EPSG","4326"]],PROJECTION["Transverse_Mercator"],PARAMETER["latitude_of_origin",0],PARAMETER["central_meridian",3],PARAMETER["scale_factor",0.9996],PARAMETER["false_easting",500000],PARAMETER["false_northing",0],UNIT["metre",1,AUTHORITY["EPSG","9001"]],AXIS["Easting",EAST],AXIS["Northing",NORTH],AUTHORITY["EPSG","32631"]]"#;

const GDAL_2222: &str = r#"PROJCS["NAD83 / Arizona East (ft)",GEOGCS["NAD83",DATUM["North_American_Datum_1983",SPHEROID["GRS 1980",6378137,298.257222101,AUTHORITY["EPSG","7019"]],AUTHORITY["EPSG","6269"]],PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],AUTHORITY["EPSG","4269"]],PROJECTION["Transverse_Mercator"],PARAMETER["latitude_of_origin",31],PARAMETER["central_meridian",-110.166666666667],PARAMETER["scale_factor",0.9999],PARAMETER["false_easting",700000],PARAMETER["false_northing",0],UNIT["foot",0.3048,AUTHORITY["EPSG","9002"]],AXIS["Easting",EAST],AXIS["Northing",NORTH],AUTHORITY["EPSG","2222"]]"#;

const GDAL_27561: &str = r#"PROJCS["NTF (Paris) / Lambert Nord France",GEOGCS["NTF (Paris)",DATUM["Nouvelle_Triangulation_Francaise_Paris",SPHEROID["Clarke 1880 (IGN)",6378249.2,293.466021293627,AUTHORITY["EPSG","7011"]],AUTHORITY["EPSG","6807"]],PRIMEM["Paris",2.33722917,AUTHORITY["EPSG","8903"]],UNIT["grad",0.0157079632679489,AUTHORITY["EPSG","9105"]],AUTHORITY["EPSG","4807"]],PROJECTION["Lambert_Conformal_Conic_1SP"],PARAMETER["latitude_of_origin",55],PARAMETER["central_meridian",0],PARAMETER["scale_factor",0.999877341],PARAMETER["false_easting",600000],PARAMETER["false_northing",200000],UNIT["metre",1,AUTHORITY["EPSG","9001"]],AXIS["Easting",EAST],AXIS["Northing",NORTH],AUTHORITY["EPSG","27561"]]"#;

const GDAL_4326: &str = r#"GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563,AUTHORITY["EPSG","7030"]],AUTHORITY["EPSG","6326"]],PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],AUTHORITY["EPSG","4326"]]"#;

const GDAL_4978: &str = r#"GEOCCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563,AUTHORITY["EPSG","7030"]],AUTHORITY["EPSG","6326"]],PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],UNIT["metre",1,AUTHORITY["EPSG","9001"]],AXIS["Geocentric X",OTHER],AXIS["Geocentric Y",OTHER],AXIS["Geocentric Z",NORTH],AUTHORITY["EPSG","4978"]]"#;

const GDAL_5714: &str = r#"VERT_CS["MSL height",VERT_DATUM["Mean Sea Level",2005,AUTHORITY["EPSG","5100"]],UNIT["metre",1,AUTHORITY["EPSG","9001"]],AXIS["Gravity-related height",UP],AUTHORITY["EPSG","5714"]]"#;

const GDAL_9518: &str = r#"COMPD_CS["WGS 84 + EGM2008 height",GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563,AUTHORITY["EPSG","7030"]],AUTHORITY["EPSG","6326"]],PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],AUTHORITY["EPSG","4326"]],VERT_CS["EGM2008 height",VERT_DATUM["EGM2008 geoid",2005,AUTHORITY["EPSG","1027"]],UNIT["metre",1,AUTHORITY["EPSG","9001"]],AXIS["Gravity-related height",UP],AUTHORITY["EPSG","3855"]],AUTHORITY["EPSG","9518"]]"#;

const GDAL_7405: &str = r#"COMPD_CS["OSGB36 / British National Grid + ODN height",PROJCS["OSGB36 / British National Grid",GEOGCS["OSGB36",DATUM["Ordnance_Survey_of_Great_Britain_1936",SPHEROID["Airy 1830",6377563.396,299.3249646,AUTHORITY["EPSG","7001"]],AUTHORITY["EPSG","6277"]],PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],AUTHORITY["EPSG","4277"]],PROJECTION["Transverse_Mercator"],PARAMETER["latitude_of_origin",49],PARAMETER["central_meridian",-2],PARAMETER["scale_factor",0.9996012717],PARAMETER["false_easting",400000],PARAMETER["false_northing",-100000],UNIT["metre",1,AUTHORITY["EPSG","9001"]],AXIS["Easting",EAST],AXIS["Northing",NORTH],AUTHORITY["EPSG","27700"]],VERT_CS["ODN height",VERT_DATUM["Ordnance Datum Newlyn",2005,AUTHORITY["EPSG","5101"]],UNIT["metre",1,AUTHORITY["EPSG","9001"]],AXIS["Gravity-related height",UP],AUTHORITY["EPSG","5701"]],AUTHORITY["EPSG","7405"]]"#;

// ---------------------------------------------------------------------------
// ESRI-dialect fixtures
// ---------------------------------------------------------------------------

const ESRI_4326: &str = r#"GEOGCS["GCS_WGS_1984",DATUM["D_WGS_1984",SPHEROID["WGS_1984",6378137.0,298.257223563]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]]"#;

const ESRI_3857: &str = r#"PROJCS["WGS_1984_Web_Mercator_Auxiliary_Sphere",GEOGCS["GCS_WGS_1984",DATUM["D_WGS_1984",SPHEROID["WGS_1984",6378137.0,298.257223563]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]],PROJECTION["Mercator_Auxiliary_Sphere"],PARAMETER["False_Easting",0.0],PARAMETER["False_Northing",0.0],PARAMETER["Central_Meridian",0.0],PARAMETER["Standard_Parallel_1",0.0],PARAMETER["Auxiliary_Sphere_Type",0.0],UNIT["Meter",1.0]]"#;

const ESRI_6592: &str = r#"PROJCS["NAD_1983_2011_StatePlane_Virginia_North_FIPS_4501",GEOGCS["GCS_NAD_1983_2011",DATUM["D_NAD_1983_2011",SPHEROID["GRS_1980",6378137.0,298.257222101]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]],PROJECTION["Lambert_Conformal_Conic"],PARAMETER["False_Easting",3500000.0],PARAMETER["False_Northing",2000000.0],PARAMETER["Central_Meridian",-78.5],PARAMETER["Standard_Parallel_1",39.2],PARAMETER["Standard_Parallel_2",38.0333333333333],PARAMETER["Latitude_Of_Origin",37.6666666666667],UNIT["Meter",1.0]]"#;

const ESRI_2230: &str = r#"PROJCS["NAD_1983_StatePlane_California_VI_FIPS_0406_Feet",GEOGCS["GCS_North_American_1983",DATUM["D_North_American_1983",SPHEROID["GRS_1980",6378137.0,298.257222101]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]],PROJECTION["Lambert_Conformal_Conic"],PARAMETER["False_Easting",6561666.667],PARAMETER["False_Northing",1640416.667],PARAMETER["Central_Meridian",-116.25],PARAMETER["Standard_Parallel_1",33.8833333333333],PARAMETER["Standard_Parallel_2",32.7833333333333],PARAMETER["Latitude_Of_Origin",32.1666666666667],UNIT["US survey foot",0.304800609601219]]"#;

const ESRI_27561: &str = r#"PROJCS["NTF_Paris_Lambert_Nord_France",GEOGCS["GCS_NTF_Paris",DATUM["Nouvelle_Triangulation_Francaise_(Paris)",SPHEROID["Clarke_1880_IGN",6378249.2,293.466021293627]],PRIMEM["Paris",2.33722917],UNIT["Grad",0.0157079632679489]],PROJECTION["Lambert_Conformal_Conic"],PARAMETER["False_Easting",600000.0],PARAMETER["False_Northing",200000.0],PARAMETER["Central_Meridian",0.0],PARAMETER["Standard_Parallel_1",55.0],PARAMETER["Scale_Factor",0.999877341],PARAMETER["Latitude_Of_Origin",55.0],UNIT["Meter",1.0]]"#;

const ESRI_32661: &str = r#"PROJCS["UPS_North",GEOGCS["GCS_WGS_1984",DATUM["D_WGS_1984",SPHEROID["WGS_1984",6378137.0,298.257223563]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]],PROJECTION["Stereographic"],PARAMETER["False_Easting",2000000.0],PARAMETER["False_Northing",2000000.0],PARAMETER["Central_Meridian",0.0],PARAMETER["Scale_Factor",0.994],PARAMETER["Latitude_Of_Origin",90.0],UNIT["Meter",1.0]]"#;

const ESRI_28992: &str = r#"PROJCS["RD_New",GEOGCS["GCS_Amersfoort",DATUM["D_Amersfoort",SPHEROID["Bessel_1841",6377397.155,299.1528128]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]],PROJECTION["Double_Stereographic"],PARAMETER["False_Easting",155000.0],PARAMETER["False_Northing",463000.0],PARAMETER["Central_Meridian",5.38763888888889],PARAMETER["Scale_Factor",0.9999079],PARAMETER["Latitude_Of_Origin",52.1561605555556],UNIT["Meter",1.0]]"#;

const ESRI_31468: &str = r#"PROJCS["DHDN_3_Degree_Gauss_Zone_4",GEOGCS["GCS_Deutsches_Hauptdreiecksnetz",DATUM["D_Deutsches_Hauptdreiecksnetz",SPHEROID["Bessel_1841",6377397.155,299.1528128]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]],PROJECTION["Gauss_Kruger"],PARAMETER["False_Easting",4500000.0],PARAMETER["False_Northing",0.0],PARAMETER["Central_Meridian",12.0],PARAMETER["Scale_Factor",1.0],PARAMETER["Latitude_Of_Origin",0.0],UNIT["Meter",1.0]]"#;

const ESRI_VERTCS_UP: &str = r#"VERTCS["NAVD_1988",VDATUM["North_American_Vertical_Datum_1988"],PARAMETER["Vertical_Shift",0.0],PARAMETER["Direction",1.0],UNIT["Meter",1.0]]"#;

const ESRI_VERTCS_DOWN: &str = r#"VERTCS["MSL_Depth",VDATUM["Mean_Sea_Level"],PARAMETER["Vertical_Shift",0.0],PARAMETER["Direction",-1.0],UNIT["Meter",1.0]]"#;

// ---------------------------------------------------------------------------
// Hand-crafted fixtures for lossy nodes / errors
// ---------------------------------------------------------------------------

const GDAL_TOWGS84: &str = r#"GEOGCS["Amersfoort",DATUM["Amersfoort",SPHEROID["Bessel 1841",6377397.155,299.1528128],TOWGS84[565.4171,50.3319,465.5524,1.9342,-1.6677,9.1019,4.0725]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433],AUTHORITY["EPSG","4289"]]"#;

const GDAL_EXTENSION: &str = r#"PROJCS["WGS 84 / Pseudo-Mercator",GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Mercator_1SP"],PARAMETER["central_meridian",0],PARAMETER["scale_factor",1],PARAMETER["false_easting",0],PARAMETER["false_northing",0],UNIT["metre",1],EXTENSION["PROJ4","+proj=merc +a=6378137 +b=6378137 +lat_ts=0 +lon_0=0 +x_0=0 +y_0=0 +k=1 +units=m +nadgrids=@null +wktext +no_defs"],AUTHORITY["EPSG","3857"]]"#;

const LOCAL_CS: &str =
    r#"LOCAL_CS["Engineering",LOCAL_DATUM["Local",0],UNIT["metre",1],AXIS["X",EAST]]"#;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_projected(input: &str) -> ProjectedCrs {
    match parse_wkt1(input).unwrap() {
        Crs::ProjectedCrs(crs) => *crs,
        other => panic!("expected ProjectedCrs, got {other:?}"),
    }
}

fn param<'a>(crs: &'a ProjectedCrs, name: &str) -> &'a MapProjectionParameter {
    crs.map_projection
        .parameters
        .iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| {
            panic!(
                "parameter {name:?} not found in {:?}",
                crs.map_projection
                    .parameters
                    .iter()
                    .map(|p| &p.name)
                    .collect::<Vec<_>>()
            )
        })
}

// ---------------------------------------------------------------------------
// GDAL dialect
// ---------------------------------------------------------------------------

#[test]
fn gdal_projcs_utm() {
    let crs = parse_projected(GDAL_32631);
    assert_eq!(crs.name, "WGS 84 / UTM zone 31N");
    assert_eq!(crs.to_epsg(), Some(32631));

    // Method resolved to the EPSG name with its EPSG code attached.
    assert_eq!(crs.map_projection.name, "unnamed");
    assert_eq!(crs.map_projection.method.name, "Transverse Mercator");
    assert_eq!(
        crs.map_projection.method.identifiers[0].authority_unique_id,
        AuthorityId::Number(9807.0)
    );

    // Parameters carry EPSG names and contextual units.
    let lat = param(&crs, "Latitude of natural origin");
    assert_eq!(lat.value, 0.0);
    let lat_unit = lat.unit.as_ref().unwrap();
    assert_eq!(lat_unit.keyword, UnitKeyword::AngleUnit);
    assert_eq!(lat_unit.name, "degree");
    let fe = param(&crs, "False easting");
    assert_eq!(fe.value, 500000.0);
    let fe_unit = fe.unit.as_ref().unwrap();
    assert_eq!(fe_unit.keyword, UnitKeyword::LengthUnit);
    assert_eq!(fe_unit.name, "metre");
    let k = param(&crs, "Scale factor at natural origin");
    assert_eq!(k.unit.as_ref().unwrap().keyword, UnitKeyword::ScaleUnit);

    // Base CRS.
    assert_eq!(crs.base_geodetic_crs.name, "WGS 84");
    let Datum::ReferenceFrame(rf) = &crs.base_geodetic_crs.datum else {
        panic!("expected reference frame");
    };
    // GDAL datum name is de-aliased to the official EPSG name, without the
    // EPSG ensemble suffix (WKT1 has no ensemble concept).
    assert_eq!(rf.name, "World Geodetic System 1984");
    assert_eq!(rf.ellipsoid.name, "WGS 84");
    assert_eq!(rf.ellipsoid.semi_major_axis, 6378137.0);

    // Axes from the AXIS nodes; unit is the PROJCS unit.
    let cs = &crs.coordinate_system;
    assert_eq!(cs.cs_type, CsType::Cartesian);
    assert_eq!(cs.dimension, 2);
    assert_eq!(cs.axes.len(), 2);
    assert_eq!(cs.axes[0].direction, "east");
    assert_eq!(cs.axes[1].direction, "north");
    assert_eq!(cs.cs_unit.as_ref().unwrap().name, "metre");

    // The result serializes to parseable WKT2.
    let wkt2 = Crs::ProjectedCrs(Box::new(crs)).to_wkt2();
    crate::parse_wkt2(&wkt2).unwrap();
}

#[test]
fn gdal_projcs_foot_unit() {
    let crs = parse_projected(GDAL_2222);
    assert_eq!(crs.to_epsg(), Some(2222));
    // Linear parameters are in the PROJCS unit (foot), value untouched.
    let fe = param(&crs, "False easting");
    assert_eq!(fe.value, 700000.0);
    let unit = fe.unit.as_ref().unwrap();
    assert_eq!(unit.name, "foot");
    assert_eq!(unit.conversion_factor, Some(0.3048));
    assert_eq!(crs.coordinate_system.cs_unit.as_ref().unwrap().name, "foot");
}

#[test]
fn gdal_projcs_grad_and_paris() {
    let crs = parse_projected(GDAL_27561);
    assert_eq!(crs.to_epsg(), Some(27561));
    assert_eq!(
        crs.map_projection.method.name,
        "Lambert Conic Conformal (1SP)"
    );

    // Angular parameters get the GEOGCS unit (grad), value untouched.
    let lat = param(&crs, "Latitude of natural origin");
    assert_eq!(lat.value, 55.0);
    let unit = lat.unit.as_ref().unwrap();
    assert_eq!(unit.name, "grad");
    assert_eq!(unit.conversion_factor, Some(0.0157079632679489));

    // PRIMEM value stays in degrees with an explicit degree unit.
    let Datum::ReferenceFrame(rf) = &crs.base_geodetic_crs.datum else {
        panic!("expected reference frame");
    };
    let pm = rf.prime_meridian.as_ref().unwrap();
    assert_eq!(pm.name, "Paris");
    assert_eq!(pm.irm_longitude, 2.33722917);
    assert_eq!(pm.unit.as_ref().unwrap().name, "degree");

    // The GEOGCS unit lands on the base CRS.
    assert_eq!(
        crs.base_geodetic_crs
            .ellipsoidal_cs_unit
            .as_ref()
            .unwrap()
            .name,
        "grad"
    );
}

#[test]
fn gdal_geogcs() {
    let Crs::GeogCrs(crs) = parse_wkt1(GDAL_4326).unwrap() else {
        panic!("expected GeogCrs");
    };
    assert_eq!(crs.name, "WGS 84");
    assert_eq!(crs.to_epsg(), Some(4326));

    // Synthesized axes in EPSG order (lat, lon).
    let cs = &crs.coordinate_system;
    assert_eq!(cs.cs_type, CsType::Ellipsoidal);
    assert_eq!(cs.dimension, 2);
    assert_eq!(cs.axes[0].direction, "north");
    assert_eq!(cs.axes[1].direction, "east");
    assert_eq!(cs.cs_unit.as_ref().unwrap().name, "degree");

    let wkt2 = Crs::GeogCrs(crs).to_wkt2();
    crate::parse_wkt2(&wkt2).unwrap();
}

#[test]
fn gdal_geoccs() {
    let Crs::GeodCrs(crs) = parse_wkt1(GDAL_4978).unwrap() else {
        panic!("expected GeodCrs");
    };
    assert_eq!(crs.name, "WGS 84");
    assert_eq!(crs.to_epsg(), Some(4978));
    let cs = &crs.coordinate_system;
    assert_eq!(cs.cs_type, CsType::Cartesian);
    assert_eq!(cs.dimension, 3);
    assert_eq!(cs.axes[0].direction, "geocentricX");
    assert_eq!(cs.axes[1].direction, "geocentricY");
    assert_eq!(cs.axes[2].direction, "geocentricZ");
}

#[test]
fn gdal_vert_cs() {
    let Crs::VertCrs(crs) = parse_wkt1(GDAL_5714).unwrap() else {
        panic!("expected VertCrs");
    };
    assert_eq!(crs.name, "MSL height");
    assert_eq!(crs.to_epsg(), Some(5714));
    let VertCrsSource::Datum { dynamic, datum } = &crs.source else {
        panic!("expected datum source");
    };
    assert!(dynamic.is_none());
    let VerticalDatum::ReferenceFrame(rf) = datum else {
        panic!("expected reference frame");
    };
    assert_eq!(rf.name, "Mean Sea Level");
    assert_eq!(crs.coordinate_system.cs_type, CsType::Vertical);
    assert_eq!(crs.coordinate_system.dimension, 1);
    assert_eq!(crs.coordinate_system.axes[0].direction, "up");
}

#[test]
fn gdal_compd_cs_geog_vert() {
    let Crs::CompoundCrs(crs) = parse_wkt1(GDAL_9518).unwrap() else {
        panic!("expected CompoundCrs");
    };
    assert_eq!(crs.name, "WGS 84 + EGM2008 height");
    assert_eq!(crs.to_epsg(), Some(9518));
    assert_eq!(crs.components.len(), 2);
    assert!(matches!(crs.components[0], SingleCrs::GeogCrs(_)));
    assert!(matches!(crs.components[1], SingleCrs::VertCrs(_)));
}

#[test]
fn gdal_compd_cs_proj_vert() {
    let Crs::CompoundCrs(crs) = parse_wkt1(GDAL_7405).unwrap() else {
        panic!("expected CompoundCrs");
    };
    assert_eq!(crs.to_epsg(), Some(7405));
    let SingleCrs::ProjectedCrs(proj) = &crs.components[0] else {
        panic!("expected ProjectedCrs component");
    };
    assert_eq!(proj.to_epsg(), Some(27700));
    let SingleCrs::VertCrs(vert) = &crs.components[1] else {
        panic!("expected VertCrs component");
    };
    assert_eq!(vert.to_epsg(), Some(5701));
}

// ---------------------------------------------------------------------------
// ESRI dialect
// ---------------------------------------------------------------------------

#[test]
fn esri_geogcs() {
    let Crs::GeogCrs(crs) = parse_wkt1(ESRI_4326).unwrap() else {
        panic!("expected GeogCrs");
    };
    // GCS_ prefix stripped, underscores to spaces.
    assert_eq!(crs.name, "WGS 1984");
    let Datum::ReferenceFrame(rf) = &crs.datum else {
        panic!("expected reference frame");
    };
    // Alias tables restore official names, with the datum's EPSG id attached.
    assert_eq!(rf.name, "World Geodetic System 1984");
    assert_eq!(
        rf.identifiers[0].authority_unique_id,
        AuthorityId::Number(6326.0)
    );
    assert_eq!(rf.ellipsoid.name, "WGS 84");
    // Unit name normalized, input conversion factor preserved.
    let unit = crs.coordinate_system.cs_unit.as_ref().unwrap();
    assert_eq!(unit.name, "degree");
    assert_eq!(unit.conversion_factor, Some(0.0174532925199433));
}

#[test]
fn esri_web_mercator() {
    let crs = parse_projected(ESRI_3857);
    assert_eq!(
        crs.map_projection.method.name,
        "Popular Visualisation Pseudo Mercator"
    );
    // Auxiliary_Sphere_Type=0 is a fixed/ignored parameter, not an error,
    // and must not leak into the parameter list.
    assert!(
        crs.map_projection
            .parameters
            .iter()
            .all(|p| p.name != "Auxiliary_Sphere_Type")
    );
}

#[test]
fn esri_lcc_2sp() {
    let crs = parse_projected(ESRI_6592);
    assert_eq!(
        crs.map_projection.method.name,
        "Lambert Conic Conformal (2SP)"
    );
    let Datum::ReferenceFrame(rf) = &crs.base_geodetic_crs.datum else {
        panic!("expected reference frame");
    };
    assert_eq!(rf.name, "NAD83 (National Spatial Reference System 2011)");
    assert_eq!(
        rf.identifiers[0].authority_unique_id,
        AuthorityId::Number(1116.0)
    );
    // 2SP parameter names.
    param(&crs, "Latitude of 1st standard parallel");
    param(&crs, "Latitude of 2nd standard parallel");
    param(&crs, "Latitude of false origin");
    param(&crs, "Easting at false origin");
}

#[test]
fn esri_lcc_1sp() {
    let crs = parse_projected(ESRI_27561);
    assert_eq!(
        crs.map_projection.method.name,
        "Lambert Conic Conformal (1SP)"
    );
    param(&crs, "Scale factor at natural origin");
    // Angular params in the GEOGCS unit (grad here).
    let lat = param(&crs, "Latitude of natural origin");
    assert_eq!(lat.unit.as_ref().unwrap().name, "grad");
}

#[test]
fn esri_ftus_unit() {
    let crs = parse_projected(ESRI_2230);
    let unit = crs.coordinate_system.cs_unit.as_ref().unwrap();
    assert_eq!(unit.name, "US survey foot");
    assert_eq!(unit.conversion_factor, Some(0.304800609601219));
    let fe = param(&crs, "Easting at false origin");
    assert_eq!(fe.value, 6561666.667);
    assert_eq!(fe.unit.as_ref().unwrap().name, "US survey foot");
}

#[test]
fn esri_stereographic_polar() {
    let crs = parse_projected(ESRI_32661);
    // Latitude_Of_Origin = 90 selects the polar variant.
    assert_eq!(
        crs.map_projection.method.name,
        "Polar Stereographic (variant A)"
    );
}

#[test]
fn esri_stereographic_oblique() {
    let crs = parse_projected(ESRI_28992);
    assert_eq!(crs.map_projection.method.name, "Oblique Stereographic");
}

#[test]
fn esri_gauss_kruger() {
    let crs = parse_projected(ESRI_31468);
    assert_eq!(crs.map_projection.method.name, "Transverse Mercator");
}

#[test]
fn esri_vertcs_up() {
    let Crs::VertCrs(crs) = parse_wkt1(ESRI_VERTCS_UP).unwrap() else {
        panic!("expected VertCrs");
    };
    let VertCrsSource::Datum { datum, .. } = &crs.source else {
        panic!("expected datum source");
    };
    let VerticalDatum::ReferenceFrame(rf) = datum else {
        panic!("expected reference frame");
    };
    // Alias table restores the official name.
    assert_eq!(rf.name, "North American Vertical Datum 1988");
    assert_eq!(crs.coordinate_system.axes[0].direction, "up");
}

#[test]
fn esri_vertcs_down() {
    let Crs::VertCrs(crs) = parse_wkt1(ESRI_VERTCS_DOWN).unwrap() else {
        panic!("expected VertCrs");
    };
    assert_eq!(crs.coordinate_system.axes[0].direction, "down");
}

// ---------------------------------------------------------------------------
// Strict vs lossy
// ---------------------------------------------------------------------------

#[test]
fn strict_rejects_towgs84() {
    match parse_wkt1(GDAL_TOWGS84) {
        Err(ParseError::LossyWkt1Node { keyword, .. }) => assert_eq!(keyword, "TOWGS84"),
        other => panic!("expected LossyWkt1Node, got {other:?}"),
    }
}

#[test]
fn lossy_ignores_towgs84() {
    let Crs::GeogCrs(crs) = parse_wkt1_lossy(GDAL_TOWGS84).unwrap() else {
        panic!("expected GeogCrs");
    };
    assert_eq!(crs.to_epsg(), Some(4289));
    // No trace of the shift parameters anywhere.
    assert!(!Crs::GeogCrs(crs).to_wkt2().contains("565.4171"));
}

#[test]
fn strict_rejects_extension() {
    match parse_wkt1(GDAL_EXTENSION) {
        Err(ParseError::LossyWkt1Node { keyword, .. }) => assert_eq!(keyword, "EXTENSION"),
        other => panic!("expected LossyWkt1Node, got {other:?}"),
    }
}

#[test]
fn lossy_ignores_extension() {
    let Crs::ProjectedCrs(crs) = parse_wkt1_lossy(GDAL_EXTENSION).unwrap() else {
        panic!("expected ProjectedCrs");
    };
    assert_eq!(crs.to_epsg(), Some(3857));
    assert_eq!(crs.map_projection.method.name, "Mercator (variant A)");
}

#[test]
fn lossy_rejects_nonzero_vertical_shift_strictly() {
    let input = r#"VERTCS["X",VDATUM["Y"],PARAMETER["Vertical_Shift",1.5],PARAMETER["Direction",1.0],UNIT["Meter",1.0]]"#;
    assert!(matches!(
        parse_wkt1(input),
        Err(ParseError::LossyWkt1Node { .. })
    ));
    assert!(parse_wkt1_lossy(input).is_ok());
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[test]
fn local_cs_unsupported() {
    assert!(matches!(
        parse_wkt1(LOCAL_CS),
        Err(ParseError::UnsupportedWkt1Node { .. })
    ));
    // Also unsupported in lossy mode: this is a whole CRS type, not a node.
    assert!(matches!(
        parse_wkt1_lossy(LOCAL_CS),
        Err(ParseError::UnsupportedWkt1Node { .. })
    ));
}

#[test]
fn geogcs_with_three_axes_unsupported() {
    let input = r#"GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433],AXIS["Latitude",NORTH],AXIS["Longitude",EAST],AXIS["Up",UP]]"#;
    assert!(matches!(
        parse_wkt1(input),
        Err(ParseError::UnsupportedWkt1Node { .. })
    ));
}

#[test]
fn unknown_projection_method() {
    let input = r#"PROJCS["X",GEOGCS["Y",DATUM["Z",SPHEROID["S",6378137,298.3]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Frobnicator"],PARAMETER["false_easting",0],UNIT["metre",1]]"#;
    match parse_wkt1(input) {
        Err(ParseError::UnknownProjectionMethod { name }) => assert_eq!(name, "Frobnicator"),
        other => panic!("expected UnknownProjectionMethod, got {other:?}"),
    }
}

#[test]
fn unknown_parameter() {
    let input = r#"PROJCS["X",GEOGCS["Y",DATUM["Z",SPHEROID["S",6378137,298.3]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Transverse_Mercator"],PARAMETER["banana",1],UNIT["metre",1]]"#;
    match parse_wkt1(input) {
        Err(ParseError::UnknownParameter { name, .. }) => assert_eq!(name, "banana"),
        other => panic!("expected UnknownParameter, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Details
// ---------------------------------------------------------------------------

#[test]
fn projcs_without_axes_synthesizes_easting_northing() {
    let input = r#"PROJCS["X",GEOGCS["Y",DATUM["Z",SPHEROID["S",6378137,298.3]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Transverse_Mercator"],PARAMETER["false_easting",0],UNIT["metre",1]]"#;
    let crs = parse_projected(input);
    let cs = &crs.coordinate_system;
    assert_eq!(cs.axes.len(), 2);
    assert_eq!(cs.axes[0].direction, "east");
    assert_eq!(cs.axes[1].direction, "north");
}

#[test]
fn authority_with_non_numeric_code() {
    let input = r#"GEOGCS["Y",DATUM["Z",SPHEROID["S",6378137,298.3]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433],AUTHORITY["FOO","BAR_1"]]"#;
    let Crs::GeogCrs(crs) = parse_wkt1(input).unwrap() else {
        panic!("expected GeogCrs");
    };
    assert_eq!(crs.identifiers[0].authority_name, "FOO");
    assert_eq!(
        crs.identifiers[0].authority_unique_id,
        AuthorityId::Text("BAR_1".to_string())
    );
}

#[test]
fn unit_factor_from_input_wins_over_table() {
    // Deliberately wrong factor for "Meter": the parsed value must keep it.
    let input = r#"GEOGCS["Y",DATUM["Z",SPHEROID["S",6378137,298.3]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]]"#;
    let Crs::GeogCrs(crs) = parse_wkt1(input).unwrap() else {
        panic!("expected GeogCrs");
    };
    assert_eq!(
        crs.coordinate_system
            .cs_unit
            .as_ref()
            .unwrap()
            .conversion_factor,
        Some(0.0174532925199433)
    );

    let input2 = r#"VERTCS["X",VDATUM["Y"],PARAMETER["Vertical_Shift",0.0],PARAMETER["Direction",1.0],UNIT["Meter",1.5]]"#;
    let Crs::VertCrs(crs) = parse_wkt1(input2).unwrap() else {
        panic!("expected VertCrs");
    };
    let unit = crs.coordinate_system.cs_unit.as_ref().unwrap();
    assert_eq!(unit.name, "metre");
    assert_eq!(unit.conversion_factor, Some(1.5));
}

#[test]
fn datum_name_heuristic_without_alias_hit() {
    // A datum name that is not in the alias table falls back to
    // underscore-to-space normalization.
    let input = r#"GEOGCS["Y",DATUM["My_Custom_Datum_2049",SPHEROID["S",6378137,298.3]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]]"#;
    let Crs::GeogCrs(crs) = parse_wkt1(input).unwrap() else {
        panic!("expected GeogCrs");
    };
    let Datum::ReferenceFrame(rf) = &crs.datum else {
        panic!("expected reference frame");
    };
    assert_eq!(rf.name, "My Custom Datum 2049");
    assert!(rf.identifiers.is_empty());
}

#[test]
fn wkt2_roundtrip_all_fixtures() {
    for (name, fixture) in [
        ("GDAL_32631", GDAL_32631),
        ("GDAL_2222", GDAL_2222),
        ("GDAL_27561", GDAL_27561),
        ("GDAL_4326", GDAL_4326),
        ("GDAL_4978", GDAL_4978),
        ("GDAL_5714", GDAL_5714),
        ("GDAL_9518", GDAL_9518),
        ("GDAL_7405", GDAL_7405),
        ("ESRI_4326", ESRI_4326),
        ("ESRI_3857", ESRI_3857),
        ("ESRI_6592", ESRI_6592),
        ("ESRI_2230", ESRI_2230),
        ("ESRI_27561", ESRI_27561),
        ("ESRI_32661", ESRI_32661),
        ("ESRI_28992", ESRI_28992),
        ("ESRI_31468", ESRI_31468),
        ("ESRI_VERTCS_UP", ESRI_VERTCS_UP),
        ("ESRI_VERTCS_DOWN", ESRI_VERTCS_DOWN),
    ] {
        let crs = parse_wkt1(fixture).unwrap_or_else(|e| panic!("{name}: parse failed: {e}"));
        let wkt2 = crs.to_wkt2();
        crate::parse_wkt2(&wkt2)
            .unwrap_or_else(|e| panic!("{name}: WKT2 roundtrip failed: {e}\n{wkt2}"));
    }
}

// ---------------------------------------------------------------------------
// Method variant selection
// ---------------------------------------------------------------------------

/// A GDAL-written polar stereographic CRS (EPSG:3031-shaped) carries the
/// redundant `scale_factor` of 1 that variant B does not define; the unity
/// value must be dropped rather than rejecting the string.
#[test]
fn gdal_polar_stereographic_variant_b() {
    let input = r#"PROJCS["WGS 84 / Antarctic Polar Stereographic",GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Polar_Stereographic"],PARAMETER["latitude_of_origin",-71],PARAMETER["central_meridian",0],PARAMETER["scale_factor",1],PARAMETER["false_easting",0],PARAMETER["false_northing",0],UNIT["metre",1]]"#;
    let crs = parse_projected(input);
    assert_eq!(
        crs.map_projection.method.name,
        "Polar Stereographic (variant B)"
    );
    // The latitude becomes the standard parallel, and no scale factor remains.
    let sp = param(&crs, "Latitude of standard parallel");
    assert_eq!(sp.value, -71.0);
    assert!(
        crs.map_projection
            .parameters
            .iter()
            .all(|p| !p.name.contains("Scale factor"))
    );
}

/// At the pole the same WKT1 name means variant A, which keeps the scale factor.
#[test]
fn gdal_polar_stereographic_variant_a() {
    let input = r#"PROJCS["UPS North",GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Polar_Stereographic"],PARAMETER["latitude_of_origin",90],PARAMETER["central_meridian",0],PARAMETER["scale_factor",0.994],PARAMETER["false_easting",2000000],PARAMETER["false_northing",2000000],UNIT["metre",1]]"#;
    let crs = parse_projected(input);
    assert_eq!(
        crs.map_projection.method.name,
        "Polar Stereographic (variant A)"
    );
    assert_eq!(param(&crs, "Scale factor at natural origin").value, 0.994);
}

/// A non-unity scale factor away from the pole fits neither variant, and the
/// error must name the offending parameter rather than an empty string.
#[test]
fn gdal_polar_stereographic_contradictory() {
    let input = r#"PROJCS["X",GEOGCS["Y",DATUM["Z",SPHEROID["S",6378137,298.3]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Polar_Stereographic"],PARAMETER["latitude_of_origin",-71],PARAMETER["scale_factor",0.97],UNIT["metre",1]]"#;
    match parse_wkt1(input) {
        Err(ParseError::UnsupportedParameterValue { name, value, .. }) => {
            assert_eq!(name, "scale_factor");
            assert_eq!(value, 0.97);
        }
        other => panic!("expected UnsupportedParameterValue, got {other:?}"),
    }
}

/// A spherical ellipsoid selects the EPSG spherical method variant. This is
/// derived from the method name, not a hardcoded code pair, so it covers the
/// whole family (here Equirectangular, which has no bespoke rule).
#[test]
fn spherical_ellipsoid_selects_spherical_variant() {
    let sphere = r#"PROJCS["X",GEOGCS["Y",DATUM["Z",SPHEROID["Sphere",6371007,0]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Equirectangular"],PARAMETER["latitude_of_origin",0],PARAMETER["central_meridian",0],PARAMETER["false_easting",0],PARAMETER["false_northing",0],UNIT["metre",1]]"#;
    assert_eq!(
        parse_projected(sphere).map_projection.method.name,
        "Equidistant Cylindrical (Spherical)"
    );

    let ellipsoid = sphere.replace(
        r#"SPHEROID["Sphere",6371007,0]"#,
        r#"SPHEROID["WGS 84",6378137,298.257223563]"#,
    );
    assert_eq!(
        parse_projected(&ellipsoid).map_projection.method.name,
        "Equidistant Cylindrical"
    );
}

/// Krovak's orientation comes from the axis directions by the same mechanism.
#[test]
fn south_west_axes_select_non_north_orientated_krovak() {
    let base = r#"PROJCS["X",GEOGCS["Y",DATUM["Z",SPHEROID["Bessel 1841",6377397.155,299.1528128]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Krovak"],PARAMETER["latitude_of_center",49.5],PARAMETER["longitude_of_center",42.5],PARAMETER["azimuth",30.2881397527778],PARAMETER["pseudo_standard_parallel_1",78.5],PARAMETER["scale_factor",0.9999],PARAMETER["false_easting",0],PARAMETER["false_northing",0],UNIT["metre",1]"#;
    let north = parse_projected(&format!("{base}]"));
    assert_eq!(
        north.map_projection.method.name,
        "Krovak (North Orientated)"
    );

    let south = parse_projected(&format!(r#"{base},AXIS["X",SOUTH],AXIS["Y",WEST]]"#));
    assert_eq!(south.map_projection.method.name, "Krovak");
}

// ---------------------------------------------------------------------------
// Required nodes
// ---------------------------------------------------------------------------

#[test]
fn missing_unit_is_an_error() {
    // OGC 01-009 requires UNIT; without it there is no way to know what the
    // parameter values and coordinates mean, so guessing is refused.
    let projcs = r#"PROJCS["X",GEOGCS["Y",DATUM["Z",SPHEROID["S",6378137,298.3]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Transverse_Mercator"],PARAMETER["false_easting",500000]]"#;
    match parse_wkt1(projcs) {
        Err(ParseError::MissingWkt1Node {
            keyword, parent, ..
        }) => {
            assert_eq!(keyword, "UNIT");
            assert_eq!(parent, "PROJCS");
        }
        other => panic!("expected MissingWkt1Node, got {other:?}"),
    }

    let geogcs = r#"GEOGCS["Y",DATUM["Z",SPHEROID["S",6378137,298.3]],PRIMEM["Greenwich",0]]"#;
    assert!(matches!(
        parse_wkt1(geogcs),
        Err(ParseError::MissingWkt1Node { .. })
    ));
    // Lossy mode is about discardable nodes, not about inventing missing ones.
    assert!(matches!(
        parse_wkt1_lossy(geogcs),
        Err(ParseError::MissingWkt1Node { .. })
    ));
}

// ---------------------------------------------------------------------------
// ESRI geographic 3D (LINUNIT)
// ---------------------------------------------------------------------------

#[test]
fn esri_geographic_3d_synthesized_axes() {
    let input = r#"GEOGCS["IGRS_3D",DATUM["D_Iraqi_Geospatial_Reference_System",SPHEROID["GRS_1980",6378137.0,298.257222101]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433],LINUNIT["Meter",1.0]]"#;
    let Crs::GeogCrs(crs) = parse_wkt1(input).unwrap() else {
        panic!("expected GeogCrs");
    };
    let cs = &crs.coordinate_system;
    assert_eq!(cs.cs_type, CsType::Ellipsoidal);
    assert_eq!(cs.dimension, 3);
    assert_eq!(cs.cs_unit, None); // axes carry their own units
    let dirs: Vec<&str> = cs.axes.iter().map(|a| a.direction.as_str()).collect();
    assert_eq!(dirs, ["north", "east", "up"]);
    assert_eq!(cs.axes[0].unit.as_ref().unwrap().name, "degree");
    assert_eq!(cs.axes[2].unit.as_ref().unwrap().name, "metre");
    crate::parse_wkt2(&Crs::GeogCrs(crs).to_wkt2()).unwrap();
}

/// Explicit 3D axes must be preserved in their input order: silently
/// reordering them would swap coordinates without any error.
#[test]
fn esri_geographic_3d_preserves_explicit_axis_order() {
    let input = r#"GEOGCS["X",DATUM["D_WGS_1984",SPHEROID["WGS_1984",6378137.0,298.257223563]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433],LINUNIT["Meter",1.0],AXIS["Longitude",EAST],AXIS["Latitude",NORTH],AXIS["Height",UP]]"#;
    let Crs::GeogCrs(crs) = parse_wkt1(input).unwrap() else {
        panic!("expected GeogCrs");
    };
    let axes = &crs.coordinate_system.axes;
    let dirs: Vec<&str> = axes.iter().map(|a| a.direction.as_str()).collect();
    assert_eq!(dirs, ["east", "north", "up"]);
    assert_eq!(axes[0].name_abbrev, "Longitude");
    assert_eq!(axes[0].order, Some(1));
    // The height axis still gets LINUNIT even though it is last.
    assert_eq!(axes[2].unit.as_ref().unwrap().name, "metre");
    assert_eq!(axes[0].unit.as_ref().unwrap().name, "degree");
}

// ---------------------------------------------------------------------------
// ESRI VERTCS on a geodetic datum
// ---------------------------------------------------------------------------

/// An ESRI ellipsoidal-height VERTCS names a geodetic datum. Its EPSG code
/// identifies a geodetic object and must not be attached to the vertical
/// reference frame derived from it.
#[test]
fn esri_vertcs_on_geodetic_datum_drops_geodetic_id() {
    let input = r#"VERTCS["WGS_1984",DATUM["D_WGS_1984",SPHEROID["WGS_1984",6378137.0,298.257223563]],PARAMETER["Vertical_Shift",0.0],PARAMETER["Direction",1.0],UNIT["Meter",1.0]]"#;
    let Crs::VertCrs(crs) = parse_wkt1(input).unwrap() else {
        panic!("expected VertCrs");
    };
    let VertCrsSource::Datum { datum, .. } = &crs.source else {
        panic!("expected datum source");
    };
    let VerticalDatum::ReferenceFrame(rf) = datum else {
        panic!("expected reference frame");
    };
    assert_eq!(rf.name, "World Geodetic System 1984");
    assert!(
        rf.identifiers.is_empty(),
        "geodetic datum id must not be reused as a vertical datum id, got {:?}",
        rf.identifiers
    );
    // The height is ellipsoidal, not gravity-related.
    assert_eq!(
        crs.coordinate_system.axes[0].name_abbrev,
        "Ellipsoidal height (h)"
    );
}

#[test]
fn esri_vertcs_rejects_undefined_direction() {
    for bad in ["0.0", "2.0", "-0.5"] {
        let input = format!(
            r#"VERTCS["X",VDATUM["Y"],PARAMETER["Vertical_Shift",0.0],PARAMETER["Direction",{bad}],UNIT["Meter",1.0]]"#
        );
        match parse_wkt1(&input) {
            Err(ParseError::UnsupportedParameterValue { name, .. }) => {
                assert_eq!(name, "Direction");
            }
            other => panic!("expected UnsupportedParameterValue for {bad}, got {other:?}"),
        }
        // A value that would silently invert the axis is not "lossy", so the
        // lossy variant rejects it too.
        assert!(parse_wkt1_lossy(&input).is_err());
    }
}

// ---------------------------------------------------------------------------
// Error quality
// ---------------------------------------------------------------------------

/// An ESRI fixed-value parameter set to an unsupported value must report that
/// value, not an empty parameter name.
#[test]
fn esri_unsupported_fixed_value_names_the_parameter() {
    let input = ESRI_3857.replace(
        r#"PARAMETER["Auxiliary_Sphere_Type",0.0]"#,
        r#"PARAMETER["Auxiliary_Sphere_Type",1.0]"#,
    );
    match parse_wkt1(&input) {
        Err(ParseError::UnsupportedParameterValue { name, value, .. }) => {
            assert_eq!(name, "Auxiliary_Sphere_Type");
            assert_eq!(value, 1.0);
        }
        other => panic!("expected UnsupportedParameterValue, got {other:?}"),
    }
}

/// ESRI's `Lambert_Conformal_Conic` spells the 1SP reading with
/// `Standard_Parallel_1` and `Latitude_Of_Origin` both meaning the latitude of
/// natural origin. When the two disagree, the 1SP reading is contradictory and
/// must be rejected in favour of the 2SP reading, where they are distinct
/// parameters.
#[test]
fn esri_lcc_differing_parallel_and_origin_selects_2sp() {
    let input = r#"PROJCS["X",GEOGCS["Y",DATUM["D_North_American_1983",SPHEROID["GRS_1980",6378137.0,298.257222101]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]],PROJECTION["Lambert_Conformal_Conic"],PARAMETER["False_Easting",0.0],PARAMETER["False_Northing",0.0],PARAMETER["Central_Meridian",-120.5],PARAMETER["Standard_Parallel_1",38.0],PARAMETER["Scale_Factor",1.0],PARAMETER["Latitude_Of_Origin",36.5],UNIT["Meter",1.0]]"#;
    let crs = parse_projected(input);
    assert_eq!(
        crs.map_projection.method.name,
        "Lambert Conic Conformal (2SP)"
    );
    assert_eq!(param(&crs, "Latitude of 1st standard parallel").value, 38.0);
    assert_eq!(param(&crs, "Latitude of false origin").value, 36.5);
}

/// The same two names carrying the same value collapse into one parameter and
/// the 1SP reading stands (as in the real EPSG:27561 fixture).
#[test]
fn esri_lcc_equal_parallel_and_origin_selects_1sp() {
    let crs = parse_projected(ESRI_27561);
    assert_eq!(
        crs.map_projection.method.name,
        "Lambert Conic Conformal (1SP)"
    );
    assert_eq!(param(&crs, "Latitude of natural origin").value, 55.0);
}

// ---------------------------------------------------------------------------
// Malformed input
// ---------------------------------------------------------------------------

#[test]
fn unterminated_string_in_skipped_node() {
    // The lossy path skips EXTENSION without interpreting it; an unterminated
    // quoted string inside must not run the cursor past the end of the input.
    let input = r#"GEOGCS["Y",DATUM["Z",SPHEROID["S",6378137,298.3]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433],EXTENSION["PROJ4","+proj=merc"#;
    assert!(matches!(
        parse_wkt1_lossy(input),
        Err(ParseError::UnterminatedString { .. } | ParseError::UnexpectedEnd)
    ));
}

// ---------------------------------------------------------------------------
// Alias table integrity
// ---------------------------------------------------------------------------

#[test]
fn alias_tables_are_sorted_and_unique() {
    for table in [
        super::esri_alias_data::GEODETIC_DATUM_ALIASES,
        super::esri_alias_data::VERTICAL_DATUM_ALIASES,
        super::esri_alias_data::ELLIPSOID_ALIASES,
        super::esri_alias_data::UNIT_ALIASES,
    ] {
        for w in table.windows(2) {
            assert!(
                w[0].0 < w[1].0,
                "not sorted/unique: {:?} vs {:?}",
                w[0].0,
                w[1].0
            );
        }
    }
}
