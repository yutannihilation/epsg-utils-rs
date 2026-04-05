//! Deserialization of PROJJSON into parsed WKT2 types.

use serde_json::Value;

use crate::crs::*;
use crate::error::ParseError;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn err(msg: impl Into<String>) -> ParseError {
    ParseError::InvalidJson {
        message: msg.into(),
    }
}

fn get_str<'a>(obj: &'a Value, key: &str) -> Result<&'a str, ParseError> {
    obj.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| err(format!("missing or invalid string field '{key}'")))
}

fn get_f64(obj: &Value, key: &str) -> Result<f64, ParseError> {
    obj.get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| err(format!("missing or invalid number field '{key}'")))
}

fn opt_str<'a>(obj: &'a Value, key: &str) -> Option<&'a str> {
    obj.get(key).and_then(Value::as_str)
}

fn opt_f64(obj: &Value, key: &str) -> Option<f64> {
    obj.get(key).and_then(Value::as_f64)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse a PROJJSON string into a [`ProjectedCrs`].
pub fn parse_projjson(input: &str) -> Result<ProjectedCrs, ParseError> {
    let v: Value =
        serde_json::from_str(input).map_err(|e| err(format!("JSON syntax error: {e}")))?;
    parse_projected_crs_json(&v)
}

fn parse_projected_crs_json(v: &Value) -> Result<ProjectedCrs, ParseError> {
    let name = get_str(v, "name")?.to_string();

    let base_crs = v.get("base_crs").ok_or_else(|| err("missing 'base_crs'"))?;
    let base_geodetic_crs = parse_base_crs(base_crs)?;

    let conversion = v
        .get("conversion")
        .ok_or_else(|| err("missing 'conversion'"))?;
    let map_projection = parse_conversion(conversion)?;

    let cs = v
        .get("coordinate_system")
        .ok_or_else(|| err("missing 'coordinate_system'"))?;
    let coordinate_system = parse_coordinate_system(cs)?;

    let usages = parse_usages(v)?;
    let identifiers = parse_ids(v)?;
    let remark = opt_str(v, "remarks").map(String::from);

    Ok(ProjectedCrs {
        name,
        base_geodetic_crs,
        map_projection,
        coordinate_system,
        usages,
        identifiers,
        remark,
    })
}

// ---------------------------------------------------------------------------
// Base CRS
// ---------------------------------------------------------------------------

fn parse_base_crs(v: &Value) -> Result<BaseGeodeticCrs, ParseError> {
    let type_str = opt_str(v, "type").unwrap_or("GeographicCRS");
    let keyword = match type_str {
        "GeographicCRS" => BaseGeodeticCrsKeyword::BaseGeogCrs,
        "GeodeticCRS" => BaseGeodeticCrsKeyword::BaseGeodCrs,
        _ => return Err(err(format!("unknown base_crs type '{type_str}'"))),
    };

    let name = get_str(v, "name")?.to_string();

    let (datum, dynamic) = if let Some(d) = v.get("datum") {
        parse_datum_json(d)?
    } else if let Some(de) = v.get("datum_ensemble") {
        (Datum::Ensemble(parse_datum_ensemble(de)?), None)
    } else {
        return Err(err("base_crs must have either 'datum' or 'datum_ensemble'"));
    };

    // deformation_models on the CRS become part of dynamic
    let dynamic = if let Some(models) = v.get("deformation_models").and_then(Value::as_array) {
        let model = models.first().map(|m| {
            Ok::<_, ParseError>(DeformationModel {
                name: get_str(m, "name")?.to_string(),
                identifiers: parse_ids(m)?,
            })
        });
        let deformation_model = model.transpose()?;
        match dynamic {
            Some(mut d) => {
                d.deformation_model = deformation_model;
                Some(d)
            }
            None => deformation_model.map(|dm| DynamicCrs {
                frame_reference_epoch: 0.0,
                deformation_model: Some(dm),
            }),
        }
    } else {
        dynamic
    };

    let ellipsoidal_cs_unit = None; // Not typically present in PROJJSON base_crs
    let identifiers = parse_ids(v)?;

    Ok(BaseGeodeticCrs {
        keyword,
        name,
        dynamic,
        datum,
        ellipsoidal_cs_unit,
        identifiers,
    })
}

// ---------------------------------------------------------------------------
// Datum
// ---------------------------------------------------------------------------

fn parse_datum_json(v: &Value) -> Result<(Datum, Option<DynamicCrs>), ParseError> {
    let type_str = opt_str(v, "type").unwrap_or("GeodeticReferenceFrame");

    let is_dynamic = type_str == "DynamicGeodeticReferenceFrame";

    let name = get_str(v, "name")?.to_string();

    let ellipsoid_v = v
        .get("ellipsoid")
        .ok_or_else(|| err("datum missing 'ellipsoid'"))?;
    let ellipsoid = parse_ellipsoid(ellipsoid_v)?;

    let anchor = opt_str(v, "anchor").map(String::from);
    let anchor_epoch = opt_f64(v, "anchor_epoch");

    let prime_meridian = v
        .get("prime_meridian")
        .map(parse_prime_meridian)
        .transpose()?;

    let identifiers = parse_ids(v)?;

    let rf = GeodeticReferenceFrame {
        name,
        ellipsoid,
        anchor,
        anchor_epoch,
        identifiers,
        prime_meridian,
    };

    let dynamic = if is_dynamic {
        Some(DynamicCrs {
            frame_reference_epoch: get_f64(v, "frame_reference_epoch")?,
            deformation_model: None, // filled by caller from CRS-level
        })
    } else {
        None
    };

    Ok((Datum::ReferenceFrame(rf), dynamic))
}

fn parse_datum_ensemble(v: &Value) -> Result<DatumEnsemble, ParseError> {
    let name = get_str(v, "name")?.to_string();

    let members_arr = v
        .get("members")
        .and_then(Value::as_array)
        .ok_or_else(|| err("datum_ensemble missing 'members'"))?;
    let members: Result<Vec<_>, _> = members_arr
        .iter()
        .map(|m| {
            Ok(EnsembleMember {
                name: get_str(m, "name")?.to_string(),
                identifiers: parse_ids(m)?,
            })
        })
        .collect();
    let members = members?;

    let ellipsoid = v.get("ellipsoid").map(parse_ellipsoid).transpose()?;

    // Schema says accuracy is a string
    let accuracy_str = get_str(v, "accuracy")?;
    let accuracy = accuracy_str
        .parse::<f64>()
        .map_err(|_| err(format!("invalid accuracy '{accuracy_str}'")))?;

    let identifiers = parse_ids(v)?;

    Ok(DatumEnsemble {
        name,
        members,
        ellipsoid,
        accuracy,
        identifiers,
        prime_meridian: None, // PROJJSON puts prime_meridian inside the datum, not the ensemble
    })
}

// ---------------------------------------------------------------------------
// Ellipsoid & Prime Meridian
// ---------------------------------------------------------------------------

fn parse_ellipsoid(v: &Value) -> Result<Ellipsoid, ParseError> {
    let name = get_str(v, "name")?.to_string();

    let (semi_major_axis, unit) = parse_value_and_optional_unit(v, "semi_major_axis")?;
    let inverse_flattening = get_f64(v, "inverse_flattening")?;
    let identifiers = parse_ids(v)?;

    Ok(Ellipsoid {
        name,
        semi_major_axis,
        inverse_flattening,
        unit,
        identifiers,
    })
}

fn parse_prime_meridian(v: &Value) -> Result<PrimeMeridian, ParseError> {
    let name = get_str(v, "name")?.to_string();

    let (irm_longitude, unit) = parse_value_and_optional_unit(v, "longitude")?;
    let identifiers = parse_ids(v)?;

    Ok(PrimeMeridian {
        name,
        irm_longitude,
        unit,
        identifiers,
    })
}

/// Parse a field that can be either a bare number or `{ "value": N, "unit": ... }`.
fn parse_value_and_optional_unit(
    obj: &Value,
    key: &str,
) -> Result<(f64, Option<Unit>), ParseError> {
    let field = obj
        .get(key)
        .ok_or_else(|| err(format!("missing field '{key}'")))?;

    if let Some(n) = field.as_f64() {
        Ok((n, None))
    } else if field.is_object() {
        let value = get_f64(field, "value")?;
        let unit_v = field
            .get("unit")
            .ok_or_else(|| err(format!("value_and_unit '{key}' missing 'unit'")))?;
        let unit = parse_unit(unit_v)?;
        Ok((value, Some(unit)))
    } else {
        Err(err(format!("field '{key}' must be a number or object")))
    }
}

// ---------------------------------------------------------------------------
// Conversion (Map Projection)
// ---------------------------------------------------------------------------

fn parse_conversion(v: &Value) -> Result<MapProjection, ParseError> {
    let name = get_str(v, "name")?.to_string();

    let method_v = v
        .get("method")
        .ok_or_else(|| err("conversion missing 'method'"))?;
    let method = MapProjectionMethod {
        name: get_str(method_v, "name")?.to_string(),
        identifiers: parse_ids(method_v)?,
    };

    let parameters = v
        .get("parameters")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(parse_parameter)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();

    let identifiers = parse_ids(v)?;

    Ok(MapProjection {
        name,
        method,
        parameters,
        identifiers,
    })
}

fn parse_parameter(v: &Value) -> Result<MapProjectionParameter, ParseError> {
    let name = get_str(v, "name")?.to_string();
    let value = get_f64(v, "value")?;
    let unit = v.get("unit").map(parse_unit).transpose()?;
    let identifiers = parse_ids(v)?;

    Ok(MapProjectionParameter {
        name,
        value,
        unit,
        identifiers,
    })
}

// ---------------------------------------------------------------------------
// Coordinate System
// ---------------------------------------------------------------------------

fn parse_coordinate_system(v: &Value) -> Result<CoordinateSystem, ParseError> {
    let subtype = get_str(v, "subtype")?;
    let cs_type = match subtype {
        "affine" => CsType::Affine,
        "Cartesian" => CsType::Cartesian,
        "cylindrical" => CsType::Cylindrical,
        "ellipsoidal" => CsType::Ellipsoidal,
        "linear" => CsType::Linear,
        "parametric" => CsType::Parametric,
        "polar" => CsType::Polar,
        "spherical" => CsType::Spherical,
        "vertical" => CsType::Vertical,
        "TemporalCount" => CsType::TemporalCount,
        "TemporalMeasure" => CsType::TemporalMeasure,
        "ordinal" => CsType::Ordinal,
        "TemporalDateTime" => CsType::TemporalDateTime,
        _ => return Err(err(format!("unknown CS subtype '{subtype}'"))),
    };

    let axis_arr = v
        .get("axis")
        .and_then(Value::as_array)
        .ok_or_else(|| err("coordinate_system missing 'axis'"))?;
    let axes: Result<Vec<_>, _> = axis_arr.iter().map(parse_axis).collect();
    let axes = axes?;

    let dimension = axes.len() as u8;
    let identifiers = parse_ids(v)?;

    Ok(CoordinateSystem {
        cs_type,
        dimension,
        identifiers,
        axes,
        cs_unit: None, // PROJJSON puts unit on each axis, not shared
    })
}

fn parse_axis(v: &Value) -> Result<Axis, ParseError> {
    let name = get_str(v, "name")?;
    let abbreviation = opt_str(v, "abbreviation").unwrap_or("");
    let name_abbrev = if abbreviation.is_empty() {
        name.to_string()
    } else {
        format!("{name} ({abbreviation})")
    };

    let direction = get_str(v, "direction")?.to_string();

    let unit = v.get("unit").map(parse_unit).transpose()?;

    let meridian = v.get("meridian").map(parse_meridian).transpose()?;
    let bearing = None; // PROJJSON doesn't seem to have bearing in axis
    let order = None; // PROJJSON doesn't have ORDER; order is implicit from array position

    let axis_min_value = opt_f64(v, "minimum_value");
    let axis_max_value = opt_f64(v, "maximum_value");
    let range_meaning = opt_str(v, "range_meaning").map(|s| match s {
        "exact" => RangeMeaning::Exact,
        _ => RangeMeaning::Wraparound,
    });

    let identifiers = parse_ids(v)?;

    Ok(Axis {
        name_abbrev,
        direction,
        meridian,
        bearing,
        order,
        unit,
        axis_min_value,
        axis_max_value,
        range_meaning,
        identifiers,
    })
}

fn parse_meridian(v: &Value) -> Result<Meridian, ParseError> {
    let value = get_f64(v, "longitude")?;
    let unit_v = v
        .get("unit")
        .ok_or_else(|| err("meridian missing 'unit'"))?;
    let unit = parse_unit(unit_v)?;
    Ok(Meridian { value, unit })
}

// ---------------------------------------------------------------------------
// Unit
// ---------------------------------------------------------------------------

fn parse_unit(v: &Value) -> Result<Unit, ParseError> {
    // Shorthand strings
    if let Some(s) = v.as_str() {
        return match s {
            "metre" => Ok(Unit {
                keyword: UnitKeyword::LengthUnit,
                name: "metre".into(),
                conversion_factor: Some(1.0),
                identifiers: vec![],
            }),
            "degree" => Ok(Unit {
                keyword: UnitKeyword::AngleUnit,
                name: "degree".into(),
                conversion_factor: Some(0.0174532925199433),
                identifiers: vec![],
            }),
            "unity" => Ok(Unit {
                keyword: UnitKeyword::ScaleUnit,
                name: "unity".into(),
                conversion_factor: Some(1.0),
                identifiers: vec![],
            }),
            _ => Err(err(format!("unknown unit shorthand '{s}'"))),
        };
    }

    // Full object
    let type_str = get_str(v, "type")?;
    let keyword = match type_str {
        "AngularUnit" => UnitKeyword::AngleUnit,
        "LinearUnit" => UnitKeyword::LengthUnit,
        "ScaleUnit" => UnitKeyword::ScaleUnit,
        "TimeUnit" => UnitKeyword::TimeUnit,
        "ParametricUnit" => UnitKeyword::ParametricUnit,
        "Unit" => UnitKeyword::Unit,
        _ => return Err(err(format!("unknown unit type '{type_str}'"))),
    };

    let name = get_str(v, "name")?.to_string();
    let conversion_factor = opt_f64(v, "conversion_factor");
    let identifiers = parse_ids(v)?;

    Ok(Unit {
        keyword,
        name,
        conversion_factor,
        identifiers,
    })
}

// ---------------------------------------------------------------------------
// Identifier
// ---------------------------------------------------------------------------

fn parse_ids(obj: &Value) -> Result<Vec<Identifier>, ParseError> {
    if let Some(id) = obj.get("id") {
        Ok(vec![parse_id(id)?])
    } else if let Some(ids) = obj.get("ids").and_then(Value::as_array) {
        ids.iter().map(parse_id).collect()
    } else {
        Ok(vec![])
    }
}

fn parse_id(v: &Value) -> Result<Identifier, ParseError> {
    let authority_name = get_str(v, "authority")?.to_string();

    let code = v.get("code").ok_or_else(|| err("id missing 'code'"))?;
    let authority_unique_id = if let Some(s) = code.as_str() {
        AuthorityId::Text(s.to_string())
    } else if let Some(n) = code.as_i64() {
        AuthorityId::Number(n as f64)
    } else if let Some(n) = code.as_f64() {
        AuthorityId::Number(n)
    } else {
        return Err(err("id 'code' must be string or number"));
    };

    let version = v.get("version").map(|v| {
        if let Some(s) = v.as_str() {
            AuthorityId::Text(s.to_string())
        } else {
            AuthorityId::Number(v.as_f64().unwrap_or(0.0))
        }
    });

    let citation = opt_str(v, "authority_citation").map(String::from);
    let uri = opt_str(v, "uri").map(String::from);

    Ok(Identifier {
        authority_name,
        authority_unique_id,
        version,
        citation,
        uri,
    })
}

// ---------------------------------------------------------------------------
// Usage & Extent
// ---------------------------------------------------------------------------

fn parse_usages(obj: &Value) -> Result<Vec<Usage>, ParseError> {
    if let Some(arr) = obj.get("usages").and_then(Value::as_array) {
        arr.iter().map(parse_usage).collect()
    } else if obj.get("scope").is_some() || obj.get("area").is_some() || obj.get("bbox").is_some() {
        // Flat form
        Ok(vec![parse_usage(obj)?])
    } else {
        Ok(vec![])
    }
}

fn parse_usage(v: &Value) -> Result<Usage, ParseError> {
    let scope = opt_str(v, "scope").unwrap_or("").to_string();

    let area = opt_str(v, "area").map(String::from);

    let bbox = v.get("bbox").map(parse_bbox).transpose()?;
    let vertical_extent = v
        .get("vertical_extent")
        .map(parse_vertical_extent)
        .transpose()?;
    let temporal_extent = v
        .get("temporal_extent")
        .map(parse_temporal_extent)
        .transpose()?;

    Ok(Usage {
        scope,
        area,
        bbox,
        vertical_extent,
        temporal_extent,
    })
}

fn parse_bbox(v: &Value) -> Result<BBox, ParseError> {
    Ok(BBox {
        lower_left_latitude: get_f64(v, "south_latitude")?,
        lower_left_longitude: get_f64(v, "west_longitude")?,
        upper_right_latitude: get_f64(v, "north_latitude")?,
        upper_right_longitude: get_f64(v, "east_longitude")?,
    })
}

fn parse_vertical_extent(v: &Value) -> Result<VerticalExtent, ParseError> {
    Ok(VerticalExtent {
        minimum_height: get_f64(v, "minimum")?,
        maximum_height: get_f64(v, "maximum")?,
        unit: v.get("unit").map(parse_unit).transpose()?,
    })
}

fn parse_temporal_extent(v: &Value) -> Result<TemporalExtent, ParseError> {
    Ok(TemporalExtent {
        start: get_str(v, "start")?.to_string(),
        end: get_str(v, "end")?.to_string(),
    })
}
