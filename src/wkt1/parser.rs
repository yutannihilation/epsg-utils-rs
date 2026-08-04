//! WKT1 (GDAL/ESRI dialect) parser producing [`crate::crs::Crs`].
//!
//! The two dialects share one parser; ESRI-isms (`VERTCS`, `D_` datum names,
//! `GCS_` CRS names, title-case parameter names) are absorbed per node.
//!
//! Deviations from OGC 01-009, matching GDAL/ESRI practice:
//! - `PRIMEM` longitudes are always interpreted as degrees, regardless of the
//!   GEOGCS angular unit.
//! - Missing axes are synthesized in EPSG order (lat, lon for geographic;
//!   easting, northing for projected).

use crate::crs::*;
use crate::error::ParseError;
use crate::wkt2::Parser;

use super::esri_alias_data as aliases;
use super::mappings::{self, MethodContext, UnitType, eq_normalized};

/// Parse a complete WKT1 string.
pub(crate) fn parse(input: &str, lossy: bool) -> Result<Crs, ParseError> {
    let mut p = Parser::new_wkt1(input);
    p.skip_whitespace();
    let head_pos = p.pos();
    let crs = parse_crs(&mut p, lossy)?;
    p.skip_whitespace();

    // ESRI writes compound CRSs without a COMPD_CS wrapper, as a horizontal
    // CRS followed by a vertical one: `PROJCS[...],VERTCS[...]`.
    if p.peek_char() == Some(',') {
        let head = into_single_crs(crs, head_pos)?;
        let mut components = vec![head];
        while p.peek_char() == Some(',') {
            let pos = p.pos();
            let next = p.comma_then(|p| parse_crs(p, lossy))?;
            components.push(into_single_crs(next, pos)?);
            p.skip_whitespace();
        }
        if p.peek_char().is_some() {
            return Err(ParseError::TrailingInput { pos: p.pos() });
        }
        let name = components
            .iter()
            .map(single_crs_name)
            .collect::<Vec<_>>()
            .join(" + ");
        return Ok(Crs::CompoundCrs(Box::new(CompoundCrs {
            name,
            components,
            usages: vec![],
            identifiers: vec![],
            remark: None,
        })));
    }

    if p.peek_char().is_some() {
        return Err(ParseError::TrailingInput { pos: p.pos() });
    }
    Ok(crs)
}

fn single_crs_name(crs: &SingleCrs) -> &str {
    match crs {
        SingleCrs::ProjectedCrs(c) => &c.name,
        SingleCrs::GeogCrs(c) => &c.name,
        SingleCrs::GeodCrs(c) => &c.name,
        SingleCrs::VertCrs(c) => &c.name,
        SingleCrs::Other(raw) => raw,
    }
}

/// Demote a parsed CRS to a compound-CRS component. Nested compound CRSs are
/// not permitted by ISO 19111.
fn into_single_crs(crs: Crs, pos: usize) -> Result<SingleCrs, ParseError> {
    match crs {
        Crs::ProjectedCrs(c) => Ok(SingleCrs::ProjectedCrs(c)),
        Crs::GeogCrs(c) => Ok(SingleCrs::GeogCrs(c)),
        Crs::GeodCrs(c) => Ok(SingleCrs::GeodCrs(c)),
        Crs::VertCrs(c) => Ok(SingleCrs::VertCrs(c)),
        Crs::CompoundCrs(_) => Err(ParseError::UnsupportedWkt1Node {
            keyword: "COMPD_CS".to_string(),
            pos,
        }),
    }
}

fn parse_crs(p: &mut Parser, lossy: bool) -> Result<Crs, ParseError> {
    p.skip_whitespace();
    let keyword = p.peek_keyword().unwrap_or_default();
    match keyword {
        "PROJCS" => Ok(Crs::ProjectedCrs(Box::new(parse_projcs(p, lossy)?))),
        "GEOGCS" => Ok(Crs::GeogCrs(Box::new(parse_top_level_geogcs(p, lossy)?))),
        "GEOCCS" => Ok(Crs::GeodCrs(Box::new(parse_geoccs(p, lossy)?))),
        "VERT_CS" | "VERTCS" => Ok(Crs::VertCrs(Box::new(parse_vert_cs(p, lossy)?))),
        "COMPD_CS" => Ok(Crs::CompoundCrs(Box::new(parse_compd_cs(p, lossy)?))),
        "LOCAL_CS" | "FITTED_CS" | "PARAM_MT" | "CONCAT_MT" | "INVERSE_MT" | "PASSTHROUGH_MT" => {
            Err(ParseError::UnsupportedWkt1Node {
                keyword: keyword.to_string(),
                pos: p.pos(),
            })
        }
        _ => Err(ParseError::ExpectedKeyword { pos: p.pos() }),
    }
}

// ---------------------------------------------------------------------------
// Name normalization
// ---------------------------------------------------------------------------

fn underscores_to_spaces(s: &str) -> String {
    s.replace('_', " ")
}

/// EPSG names a datum ensemble `"<name> ensemble"`, but WKT1 has no ensemble
/// concept: the value lands in a `DATUM` / `VDATUM` node, where the suffix
/// would read as part of the datum name. Drop it, keeping the EPSG code that
/// records which object the input referred to.
fn strip_ensemble_suffix(name: &str) -> &str {
    name.strip_suffix(" ensemble").unwrap_or(name)
}

fn alias_lookup(table: &[(&'static str, &'static str, i32)], name: &str) -> Option<(String, i32)> {
    table
        .binary_search_by_key(&name, |(esri, _, _)| esri)
        .ok()
        .map(|i| (strip_ensemble_suffix(table[i].1).to_string(), table[i].2))
}

/// Find a unique official name whose normalized form matches `name`, comparing
/// against both the official name and its ensemble-suffix-free form.
fn normalized_scan(
    table: &[(&'static str, &'static str, i32)],
    name: &str,
) -> Option<(String, i32)> {
    let mut hit: Option<(String, i32)> = None;
    for (_, official, code) in table {
        let base = strip_ensemble_suffix(official);
        if eq_normalized(official, name) || eq_normalized(base, name) {
            match &hit {
                Some((_, prev)) if *prev != *code => return None, // ambiguous
                _ => hit = Some((base.to_string(), *code)),
            }
        }
    }
    hit
}

/// Normalize a datum-like name via an alias table, falling back to
/// underscore-to-space cleanup. Returns the name and, when the alias table
/// identified it, the EPSG code.
fn normalize_datum_name(
    table: &[(&'static str, &'static str, i32)],
    name: &str,
) -> (String, Option<i32>) {
    if let Some((official, code)) = alias_lookup(table, name) {
        return (official, Some(code));
    }
    // GDAL writes ESRI-style names without the D_ prefix (e.g. "WGS_1984").
    if let Some((official, code)) = alias_lookup(table, &format!("D_{name}")) {
        return (official, Some(code));
    }
    let stripped = name.strip_prefix("D_").unwrap_or(name);
    if let Some((official, code)) = normalized_scan(table, stripped) {
        return (official, Some(code));
    }
    (underscores_to_spaces(stripped), None)
}

fn normalize_ellipsoid_name(name: &str) -> (String, Option<i32>) {
    if let Some((official, code)) = alias_lookup(aliases::ELLIPSOID_ALIASES, name) {
        return (official, Some(code));
    }
    if let Some((official, code)) = normalized_scan(aliases::ELLIPSOID_ALIASES, name) {
        return (official, Some(code));
    }
    (underscores_to_spaces(name), None)
}

/// Well-known unit spellings not covered by the ESRI alias table
/// (the table only contains linear units).
static BUILTIN_UNITS: &[(&str, &str)] = &[
    ("degree", "degree"),
    ("foot", "foot"),
    ("grad", "grad"),
    ("gradian", "grad"),
    ("kilometer", "kilometre"),
    ("kilometre", "kilometre"),
    ("link", "link"),
    ("meter", "metre"),
    ("metre", "metre"),
    ("radian", "radian"),
    ("unity", "unity"),
    ("us survey foot", "US survey foot"),
];

fn normalize_unit_name(name: &str) -> String {
    if let Some((official, _)) = alias_lookup(aliases::UNIT_ALIASES, name) {
        return official;
    }
    let lower = name.to_ascii_lowercase();
    for (from, to) in BUILTIN_UNITS {
        if lower == *from {
            return to.to_string();
        }
    }
    name.to_string()
}

/// Clean up an ESRI geographic CRS name: `GCS_WGS_1984` -> `WGS 1984`.
fn normalize_geogcs_name(name: &str) -> String {
    let stripped = name.strip_prefix("GCS_").unwrap_or(name);
    underscores_to_spaces(stripped)
}

// ---------------------------------------------------------------------------
// Small constructors
// ---------------------------------------------------------------------------

fn degree_unit() -> Unit {
    Unit {
        keyword: UnitKeyword::AngleUnit,
        name: "degree".to_string(),
        conversion_factor: Some(0.0174532925199433),
        identifiers: vec![],
    }
}

fn scale_unity_unit() -> Unit {
    Unit {
        keyword: UnitKeyword::ScaleUnit,
        name: "unity".to_string(),
        conversion_factor: Some(1.0),
        identifiers: vec![],
    }
}

fn epsg_identifier(code: i32) -> Identifier {
    Identifier {
        authority_name: "EPSG".to_string(),
        authority_unique_id: AuthorityId::Number(code as f64),
        version: None,
        citation: None,
        uri: None,
    }
}

fn make_axis(name: &str, direction: &str, order: Option<u32>) -> Axis {
    Axis {
        name_abbrev: name.to_string(),
        direction: direction.to_string(),
        meridian: None,
        bearing: None,
        order,
        unit: None,
        axis_min_value: None,
        axis_max_value: None,
        range_meaning: None,
        identifiers: vec![],
    }
}

/// Number axes from 1 in their WKT1 order.
fn numbered(axes: Vec<Axis>) -> Vec<Axis> {
    axes.into_iter()
        .enumerate()
        .map(|(i, mut a)| {
            a.order = Some(i as u32 + 1);
            a
        })
        .collect()
}

/// Number the explicit axes, or fall back to `defaults` when the WKT1 string
/// omitted them (OGC 01-009 leaves them optional).
fn axes_or_default(
    axes: Vec<Axis>,
    defaults: [Axis; 2],
    pos: usize,
) -> Result<Vec<Axis>, ParseError> {
    match axes.len() {
        0 => Ok(numbered(defaults.to_vec())),
        2 => Ok(numbered(axes)),
        _ => Err(ParseError::UnsupportedWkt1Node {
            keyword: "AXIS".to_string(),
            pos,
        }),
    }
}

// ---------------------------------------------------------------------------
// Shared node parsers
// ---------------------------------------------------------------------------

/// Skip a node in lossy mode; error in strict mode.
fn lossy_node(p: &mut Parser, lossy: bool, keyword: &str) -> Result<(), ParseError> {
    if lossy {
        p.skip_node()?;
        Ok(())
    } else {
        Err(ParseError::LossyWkt1Node {
            keyword: keyword.to_string(),
            pos: p.pos(),
        })
    }
}

/// Parse trailing `AUTHORITY[...]` nodes, the only trailing node WKT1 permits
/// on leaf objects.
fn trailing_authorities(p: &mut Parser) -> Result<Vec<Identifier>, ParseError> {
    let mut identifiers = Vec::new();
    p.trailing_items(|p, kw| {
        if kw == "AUTHORITY" {
            identifiers.push(parse_authority(p)?);
            Ok(())
        } else {
            Err(ParseError::UnexpectedKeyword {
                keyword: kw.to_string(),
                pos: p.pos(),
            })
        }
    })?;
    Ok(identifiers)
}

/// A `UNIT` node that this crate requires in order to produce a complete CRS.
fn require_unit(unit: Option<Unit>, parent: &str, pos: usize) -> Result<Unit, ParseError> {
    unit.ok_or_else(|| ParseError::MissingWkt1Node {
        keyword: "UNIT".to_string(),
        parent: parent.to_string(),
        pos,
    })
}

/// `AUTHORITY["EPSG","4326"]` (the code is quoted text in WKT1).
fn parse_authority(p: &mut Parser) -> Result<Identifier, ParseError> {
    let (_, id) = p.bracketed(&["AUTHORITY"], |p| {
        let authority_name = p.parse_quoted_string()?;
        let code = p.comma_then(|p| p.parse_quoted_string())?;
        let authority_unique_id = match code.parse::<f64>() {
            Ok(n) => AuthorityId::Number(n),
            Err(_) => AuthorityId::Text(code),
        };
        Ok(Identifier {
            authority_name,
            authority_unique_id,
            version: None,
            citation: None,
            uri: None,
        })
    })?;
    Ok(id)
}

/// `UNIT["name",factor {,AUTHORITY[...]}]`. The name is normalized; the
/// conversion factor always comes from the input.
fn parse_unit(p: &mut Parser, keyword: UnitKeyword) -> Result<Unit, ParseError> {
    let (_, unit) = p.bracketed(&["UNIT"], |p| {
        let name = p.parse_quoted_string()?;
        let factor = p.comma_then(|p| p.parse_number())?;
        let identifiers = trailing_authorities(p)?;
        Ok(Unit {
            keyword,
            name: normalize_unit_name(&name),
            conversion_factor: Some(factor),
            identifiers,
        })
    })?;
    Ok(unit)
}

/// `AXIS["name",DIRECTION]` where DIRECTION is a bare keyword.
fn parse_axis(p: &mut Parser) -> Result<Axis, ParseError> {
    let (_, axis) = p.bracketed(&["AXIS"], |p| {
        let name = p.parse_quoted_string()?;
        let direction = p.comma_then(|p| p.parse_identifier())?;
        let direction = match direction.to_ascii_uppercase().as_str() {
            "NORTH" => "north",
            "SOUTH" => "south",
            "EAST" => "east",
            "WEST" => "west",
            "UP" => "up",
            "DOWN" => "down",
            _ => "unspecified",
        };
        Ok(make_axis(&name, direction, None))
    })?;
    Ok(axis)
}

/// `SPHEROID|ELLIPSOID["name",a,rf {,AUTHORITY}]`.
fn parse_spheroid(p: &mut Parser) -> Result<Ellipsoid, ParseError> {
    let (_, ellipsoid) = p.bracketed(&["SPHEROID", "ELLIPSOID"], |p| {
        let raw_name = p.parse_quoted_string()?;
        let semi_major_axis = p.comma_then(|p| p.parse_number())?;
        let inverse_flattening = p.comma_then(|p| p.parse_number())?;
        let mut identifiers = trailing_authorities(p)?;
        let (name, code) = normalize_ellipsoid_name(&raw_name);
        if identifiers.is_empty()
            && let Some(code) = code
        {
            identifiers.push(epsg_identifier(code));
        }
        Ok(Ellipsoid {
            name,
            semi_major_axis,
            inverse_flattening,
            // WKT1 semi-major axes are always in metres, which is also the
            // WKT2 default when the unit is omitted.
            unit: None,
            identifiers,
        })
    })?;
    Ok(ellipsoid)
}

/// `PRIMEM["name",longitude {,AUTHORITY}]`. The longitude is always
/// interpreted as degrees (GDAL/ESRI convention).
fn parse_primem(p: &mut Parser) -> Result<PrimeMeridian, ParseError> {
    let (_, pm) = p.bracketed(&["PRIMEM"], |p| {
        let name = p.parse_quoted_string()?;
        let irm_longitude = p.comma_then(|p| p.parse_number())?;
        let identifiers = trailing_authorities(p)?;
        Ok(PrimeMeridian {
            name,
            irm_longitude,
            unit: Some(degree_unit()),
            identifiers,
        })
    })?;
    Ok(pm)
}

/// `DATUM["name",SPHEROID[...] {,TOWGS84[...]} {,AUTHORITY}]`.
fn parse_datum(p: &mut Parser, lossy: bool) -> Result<GeodeticReferenceFrame, ParseError> {
    let (_, datum) = p.bracketed(&["DATUM"], |p| {
        let raw_name = p.parse_quoted_string()?;
        let ellipsoid = p.comma_then(parse_spheroid)?;
        let mut identifiers = Vec::new();
        p.trailing_items(|p, kw| match kw {
            "AUTHORITY" => {
                identifiers.push(parse_authority(p)?);
                Ok(())
            }
            "TOWGS84" => lossy_node(p, lossy, "TOWGS84"),
            other => Err(ParseError::UnexpectedKeyword {
                keyword: other.to_string(),
                pos: p.pos(),
            }),
        })?;
        let (name, code) = normalize_datum_name(aliases::GEODETIC_DATUM_ALIASES, &raw_name);
        if identifiers.is_empty()
            && let Some(code) = code
        {
            identifiers.push(epsg_identifier(code));
        }
        Ok(GeodeticReferenceFrame {
            name,
            ellipsoid,
            anchor: None,
            anchor_epoch: None,
            identifiers,
            prime_meridian: None, // attached by the caller
        })
    })?;
    Ok(datum)
}

// ---------------------------------------------------------------------------
// GEOGCS
// ---------------------------------------------------------------------------

/// The parsed contents of a `GEOGCS[...]` node.
struct Geogcs {
    name: String,
    datum: GeodeticReferenceFrame,
    angle_unit: Option<Unit>,
    /// ESRI `LINUNIT`: present only for geographic 3D CRSs, giving the unit
    /// of the ellipsoidal height axis.
    linear_unit: Option<Unit>,
    axes: Vec<Axis>,
    identifiers: Vec<Identifier>,
}

fn parse_geogcs(p: &mut Parser, lossy: bool) -> Result<Geogcs, ParseError> {
    let (_, geogcs) = p.bracketed(&["GEOGCS"], |p| {
        let raw_name = p.parse_quoted_string()?;
        let mut datum = p.comma_then(|p| parse_datum(p, lossy))?;
        let mut angle_unit = None;
        let mut linear_unit = None;
        let mut axes = Vec::new();
        let mut identifiers = Vec::new();
        p.trailing_items(|p, kw| match kw {
            "PRIMEM" => {
                datum.prime_meridian = Some(parse_primem(p)?);
                Ok(())
            }
            "UNIT" => {
                angle_unit = Some(parse_unit(p, UnitKeyword::AngleUnit)?);
                Ok(())
            }
            // ESRI geographic 3D CRS: the unit of the ellipsoidal height axis.
            "LINUNIT" => {
                let (_, unit) = p.bracketed(&["LINUNIT"], |p| {
                    let name = p.parse_quoted_string()?;
                    let factor = p.comma_then(|p| p.parse_number())?;
                    Ok(Unit {
                        keyword: UnitKeyword::LengthUnit,
                        name: normalize_unit_name(&name),
                        conversion_factor: Some(factor),
                        identifiers: vec![],
                    })
                })?;
                linear_unit = Some(unit);
                Ok(())
            }
            "AXIS" => {
                axes.push(parse_axis(p)?);
                Ok(())
            }
            "AUTHORITY" => {
                identifiers.push(parse_authority(p)?);
                Ok(())
            }
            "TOWGS84" | "EXTENSION" | "METADATA" => lossy_node(p, lossy, kw),
            other => Err(ParseError::UnexpectedKeyword {
                keyword: other.to_string(),
                pos: p.pos(),
            }),
        })?;
        Ok(Geogcs {
            name: normalize_geogcs_name(&raw_name),
            datum,
            angle_unit,
            linear_unit,
            axes,
            identifiers,
        })
    })?;
    Ok(geogcs)
}

fn geographic_cs(geogcs: &Geogcs, pos: usize) -> Result<CoordinateSystem, ParseError> {
    let angle_unit = require_unit(geogcs.angle_unit.clone(), "GEOGCS", pos)?;

    // An ESRI LINUNIT marks a geographic 3D CRS (ellipsoidal height axis).
    // The height axis carries LINUNIT and the angular axes carry UNIT, so all
    // three axes are unit-annotated individually rather than via `cs_unit`.
    if let Some(linear_unit) = &geogcs.linear_unit {
        let axes = match geogcs.axes.len() {
            0 => vec![
                make_axis("Latitude (lat)", "north", None),
                make_axis("Longitude (lon)", "east", None),
                make_axis("Ellipsoidal height (h)", "up", None),
            ],
            3 => geogcs.axes.clone(),
            _ => {
                return Err(ParseError::UnsupportedWkt1Node {
                    keyword: "AXIS".to_string(),
                    pos,
                });
            }
        };
        let axes = numbered(axes)
            .into_iter()
            .map(|mut a| {
                a.unit = Some(if a.direction == "up" || a.direction == "down" {
                    linear_unit.clone()
                } else {
                    angle_unit.clone()
                });
                a
            })
            .collect();
        return Ok(CoordinateSystem {
            cs_type: CsType::Ellipsoidal,
            dimension: 3,
            identifiers: vec![],
            axes,
            cs_unit: None,
        });
    }

    let axes = axes_or_default(
        geogcs.axes.clone(),
        [
            make_axis("Latitude (lat)", "north", None),
            make_axis("Longitude (lon)", "east", None),
        ],
        pos,
    )?;
    Ok(CoordinateSystem {
        cs_type: CsType::Ellipsoidal,
        dimension: 2,
        identifiers: vec![],
        axes,
        cs_unit: Some(angle_unit),
    })
}

fn parse_top_level_geogcs(p: &mut Parser, lossy: bool) -> Result<GeogCrs, ParseError> {
    let pos = p.pos();
    let geogcs = parse_geogcs(p, lossy)?;
    let coordinate_system = geographic_cs(&geogcs, pos)?;
    Ok(GeogCrs {
        name: geogcs.name,
        dynamic: None,
        datum: Datum::ReferenceFrame(geogcs.datum),
        coordinate_system,
        usages: vec![],
        identifiers: geogcs.identifiers,
        remark: None,
    })
}

// ---------------------------------------------------------------------------
// GEOCCS
// ---------------------------------------------------------------------------

fn parse_geoccs(p: &mut Parser, lossy: bool) -> Result<GeodCrs, ParseError> {
    let start = p.pos();
    let (_, crs) = p.bracketed(&["GEOCCS"], |p| {
        let name = p.parse_quoted_string()?;
        let mut datum = p.comma_then(|p| parse_datum(p, lossy))?;
        let mut length_unit = None;
        let mut axes = Vec::new();
        let mut identifiers = Vec::new();
        p.trailing_items(|p, kw| match kw {
            "PRIMEM" => {
                datum.prime_meridian = Some(parse_primem(p)?);
                Ok(())
            }
            "UNIT" => {
                length_unit = Some(parse_unit(p, UnitKeyword::LengthUnit)?);
                Ok(())
            }
            "AXIS" => {
                axes.push(parse_axis(p)?);
                Ok(())
            }
            "AUTHORITY" => {
                identifiers.push(parse_authority(p)?);
                Ok(())
            }
            "TOWGS84" | "EXTENSION" | "METADATA" => lossy_node(p, lossy, kw),
            other => Err(ParseError::UnexpectedKeyword {
                keyword: other.to_string(),
                pos: p.pos(),
            }),
        })?;

        // OGC 01-009 axis directions for geocentric CRSs were incorrectly
        // specified (X and Y are written as OTHER), so the directions come
        // from the axis position rather than from the input.
        let geocentric = ["geocentricX", "geocentricY", "geocentricZ"];
        let names = ["Geocentric X (X)", "Geocentric Y (Y)", "Geocentric Z (Z)"];
        if !axes.is_empty() && axes.len() != 3 {
            return Err(ParseError::UnsupportedWkt1Node {
                keyword: "AXIS".to_string(),
                pos: start,
            });
        }
        let axes = (0..3)
            .map(|i| {
                let name = axes.get(i).map(|a| a.name_abbrev.clone());
                make_axis(
                    name.as_deref().unwrap_or(names[i]),
                    geocentric[i],
                    Some(i as u32 + 1),
                )
            })
            .collect();

        Ok(GeodCrs {
            name,
            dynamic: None,
            datum: Datum::ReferenceFrame(datum),
            coordinate_system: CoordinateSystem {
                cs_type: CsType::Cartesian,
                dimension: 3,
                identifiers: vec![],
                axes,
                cs_unit: Some(require_unit(length_unit, "GEOCCS", start)?),
            },
            usages: vec![],
            identifiers,
            remark: None,
        })
    })?;
    Ok(crs)
}

// ---------------------------------------------------------------------------
// PROJCS
// ---------------------------------------------------------------------------

fn parse_projcs(p: &mut Parser, lossy: bool) -> Result<ProjectedCrs, ParseError> {
    let start = p.pos();
    let (_, crs) = p.bracketed(&["PROJCS"], |p| {
        let name = p.parse_quoted_string()?;
        let geogcs = p.comma_then(|p| parse_geogcs(p, lossy))?;

        let mut projection_name: Option<String> = None;
        let mut raw_params: Vec<(String, f64)> = Vec::new();
        let mut length_unit: Option<Unit> = None;
        let mut axes: Vec<Axis> = Vec::new();
        let mut identifiers = Vec::new();

        p.trailing_items(|p, kw| match kw {
            "PROJECTION" => {
                let (_, name) = p.bracketed(&["PROJECTION"], |p| {
                    let name = p.parse_quoted_string()?;
                    // Any AUTHORITY is discarded: the mapping tables, not the
                    // WKT1 string, define the method identity.
                    trailing_authorities(p)?;
                    Ok(name)
                })?;
                projection_name = Some(name);
                Ok(())
            }
            "PARAMETER" => {
                let (_, pair) = p.bracketed(&["PARAMETER"], |p| {
                    let name = p.parse_quoted_string()?;
                    let value = p.comma_then(|p| p.parse_number())?;
                    trailing_authorities(p)?;
                    Ok((name, value))
                })?;
                raw_params.push(pair);
                Ok(())
            }
            "UNIT" => {
                length_unit = Some(parse_unit(p, UnitKeyword::LengthUnit)?);
                Ok(())
            }
            "AXIS" => {
                axes.push(parse_axis(p)?);
                Ok(())
            }
            "AUTHORITY" => {
                identifiers.push(parse_authority(p)?);
                Ok(())
            }
            "TOWGS84" | "EXTENSION" | "METADATA" => lossy_node(p, lossy, kw),
            other => Err(ParseError::UnexpectedKeyword {
                keyword: other.to_string(),
                pos: p.pos(),
            }),
        })?;

        let axes = axes_or_default(
            axes,
            [
                make_axis("Easting (E)", "east", None),
                make_axis("Northing (N)", "north", None),
            ],
            start,
        )?;

        let projection_name = projection_name.ok_or(ParseError::MissingWkt1Node {
            keyword: "PROJECTION".to_string(),
            parent: "PROJCS".to_string(),
            pos: start,
        })?;
        // WKT1 cannot express which variant of a method a CRS uses, so the
        // enclosing CRS supplies the discriminators.
        let context = MethodContext {
            is_sphere: geogcs.datum.ellipsoid.inverse_flattening == 0.0,
            south_west_oriented: axes
                .iter()
                .any(|a| a.direction == "south" || a.direction == "west"),
        };
        let resolved = mappings::resolve_method(&projection_name, &raw_params, context)?;

        let angle_unit = require_unit(geogcs.angle_unit.clone(), "GEOGCS", start)?;
        let linear_unit = require_unit(length_unit, "PROJCS", start)?;

        let parameters = resolved
            .params
            .into_iter()
            .map(|param| {
                let unit = match param.unit_type {
                    UnitType::Angular => Some(angle_unit.clone()),
                    UnitType::Linear => Some(linear_unit.clone()),
                    UnitType::Scale | UnitType::None_ => Some(scale_unity_unit()),
                };
                let identifiers = if param.epsg_code != 0 {
                    vec![epsg_identifier(param.epsg_code)]
                } else {
                    vec![]
                };
                MapProjectionParameter {
                    name: param.epsg_name,
                    value: param.value,
                    unit,
                    identifiers,
                }
            })
            .collect();

        let method_identifiers = if resolved.epsg_code != 0 {
            vec![epsg_identifier(resolved.epsg_code)]
        } else {
            vec![]
        };

        let map_projection = MapProjection {
            name: "unnamed".to_string(),
            method: MapProjectionMethod {
                name: resolved.epsg_name,
                identifiers: method_identifiers,
            },
            parameters,
            identifiers: vec![],
        };

        Ok(ProjectedCrs {
            name,
            base_geodetic_crs: BaseGeodeticCrs {
                keyword: BaseGeodeticCrsKeyword::BaseGeogCrs,
                name: geogcs.name,
                dynamic: None,
                datum: Datum::ReferenceFrame(geogcs.datum),
                ellipsoidal_cs_unit: Some(angle_unit),
                identifiers: geogcs.identifiers,
            },
            map_projection,
            coordinate_system: CoordinateSystem {
                cs_type: CsType::Cartesian,
                dimension: 2,
                identifiers: vec![],
                axes,
                cs_unit: Some(linear_unit),
            },
            usages: vec![],
            identifiers,
            remark: None,
        })
    })?;
    Ok(crs)
}

// ---------------------------------------------------------------------------
// VERT_CS / VERTCS
// ---------------------------------------------------------------------------

fn parse_vert_cs(p: &mut Parser, lossy: bool) -> Result<VertCrs, ParseError> {
    let start = p.pos();
    let (_, crs) = p.bracketed(&["VERT_CS", "VERTCS"], |p| {
        let name = p.parse_quoted_string()?;

        let mut datum_name: Option<String> = None;
        let mut datum_identifiers: Vec<Identifier> = Vec::new();
        let mut from_geodetic_datum = false;
        let mut unit: Option<Unit> = None;
        let mut axes: Vec<Axis> = Vec::new();
        let mut identifiers = Vec::new();
        let mut direction_param: Option<f64> = None;

        p.trailing_items(|p, kw| match kw {
            // GDAL: VERT_DATUM["name",datum-type {,AUTHORITY}]
            // ESRI: VDATUM["name"]
            "VERT_DATUM" | "VDATUM" | "VERTICALDATUM" => {
                let (_, (name, ids)) =
                    p.bracketed(&["VERT_DATUM", "VDATUM", "VERTICALDATUM"], |p| {
                        let name = p.parse_quoted_string()?;
                        let mut ids = Vec::new();
                        p.trailing_items(|p, kw| {
                            if kw == "AUTHORITY" {
                                ids.push(parse_authority(p)?);
                                Ok(())
                            } else if kw.is_empty() {
                                // The datum-type number (OGC 01-009); read and discard.
                                p.parse_number()?;
                                Ok(())
                            } else {
                                Err(ParseError::UnexpectedKeyword {
                                    keyword: kw.to_string(),
                                    pos: p.pos(),
                                })
                            }
                        })?;
                        Ok((name, ids))
                    })?;
                datum_name = Some(name);
                datum_identifiers = ids;
                Ok(())
            }
            // ESRI ellipsoidal-height VERTCS references a geodetic datum.
            // Its name carries over, but its identifiers must not: an EPSG
            // geodetic-datum code does not identify a vertical datum.
            "DATUM" => {
                let datum = parse_datum(p, lossy)?;
                datum_name = Some(datum.name.clone());
                from_geodetic_datum = true;
                Ok(())
            }
            "PARAMETER" => {
                let (_, (pname, value)) = p.bracketed(&["PARAMETER"], |p| {
                    let pname = p.parse_quoted_string()?;
                    let value = p.comma_then(|p| p.parse_number())?;
                    Ok((pname, value))
                })?;
                match pname.as_str() {
                    "Vertical_Shift" => {
                        if value != 0.0 && !lossy {
                            return Err(ParseError::LossyWkt1Node {
                                keyword: "PARAMETER[\"Vertical_Shift\"]".to_string(),
                                pos: p.pos(),
                            });
                        }
                        Ok(())
                    }
                    // ESRI defines only +1 (up) and -1 (down); anything else
                    // would silently invert the axis.
                    "Direction" => {
                        if value != 1.0 && value != -1.0 {
                            return Err(ParseError::UnsupportedParameterValue {
                                method: "VERTCS".to_string(),
                                name: "Direction".to_string(),
                                value,
                            });
                        }
                        direction_param = Some(value);
                        Ok(())
                    }
                    other => Err(ParseError::UnknownParameter {
                        method: "VERTCS".to_string(),
                        name: other.to_string(),
                    }),
                }
            }
            "UNIT" => {
                unit = Some(parse_unit(p, UnitKeyword::LengthUnit)?);
                Ok(())
            }
            "AXIS" => {
                axes.push(parse_axis(p)?);
                Ok(())
            }
            "AUTHORITY" => {
                identifiers.push(parse_authority(p)?);
                Ok(())
            }
            "EXTENSION" | "METADATA" => lossy_node(p, lossy, kw),
            other => Err(ParseError::UnexpectedKeyword {
                keyword: other.to_string(),
                pos: p.pos(),
            }),
        })?;

        let raw_datum_name = datum_name.ok_or(ParseError::ExpectedKeyword { pos: p.pos() })?;
        let (datum_official, datum_code) = if from_geodetic_datum {
            // Already normalized by parse_datum; no vertical-datum identity.
            (raw_datum_name, None)
        } else {
            normalize_datum_name(aliases::VERTICAL_DATUM_ALIASES, &raw_datum_name)
        };
        if datum_identifiers.is_empty()
            && let Some(code) = datum_code
        {
            datum_identifiers.push(epsg_identifier(code));
        }

        // Axis direction: explicit AXIS node (GDAL), or ESRI's
        // PARAMETER["Direction",±1], defaulting to up.
        let axis = match (axes.into_iter().next(), direction_param) {
            (Some(mut a), _) => {
                a.order = None;
                a
            }
            (None, Some(-1.0)) => make_axis("Depth (D)", "down", None),
            // An ESRI VERTCS built on a geodetic datum measures ellipsoidal
            // height, not a gravity-related one.
            (None, _) if from_geodetic_datum => make_axis("Ellipsoidal height (h)", "up", None),
            (None, _) => make_axis("Gravity-related height (H)", "up", None),
        };

        Ok(VertCrs {
            name,
            source: VertCrsSource::Datum {
                dynamic: None,
                datum: VerticalDatum::ReferenceFrame(VerticalReferenceFrame {
                    name: datum_official,
                    anchor: None,
                    anchor_epoch: None,
                    identifiers: datum_identifiers,
                }),
            },
            coordinate_system: CoordinateSystem {
                cs_type: CsType::Vertical,
                dimension: 1,
                identifiers: vec![],
                axes: vec![axis],
                cs_unit: Some(require_unit(unit, "VERT_CS", start)?),
            },
            geoid_models: vec![],
            usages: vec![],
            identifiers,
            remark: None,
        })
    })?;
    Ok(crs)
}

// ---------------------------------------------------------------------------
// COMPD_CS
// ---------------------------------------------------------------------------

fn parse_compd_cs(p: &mut Parser, lossy: bool) -> Result<CompoundCrs, ParseError> {
    let (_, crs) = p.bracketed(&["COMPD_CS"], |p| {
        let name = p.parse_quoted_string()?;
        let mut components = Vec::new();
        let mut identifiers = Vec::new();
        p.trailing_items(|p, kw| match kw {
            "AUTHORITY" => {
                identifiers.push(parse_authority(p)?);
                Ok(())
            }
            _ => {
                let pos = p.pos();
                let component = parse_crs(p, lossy)?;
                components.push(into_single_crs(component, pos)?);
                Ok(())
            }
        })?;
        Ok(CompoundCrs {
            name,
            components,
            usages: vec![],
            identifiers,
            remark: None,
        })
    })?;
    Ok(crs)
}
