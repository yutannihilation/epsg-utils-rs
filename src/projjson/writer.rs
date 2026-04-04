//! Serialization of parsed WKT2 types to PROJJSON (JSON encoding of WKT2:2019).
//!
//! See <https://proj.org/en/stable/specifications/projjson.html> for the specification
//! and <https://proj.org/en/latest/schemas/v0.7/projjson.schema.json> for the schema.

use serde_json::{Map, Value, json};

use crate::crs::*;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

impl Crs {
    /// Serialize this CRS to a PROJJSON `serde_json::Value`.
    pub fn to_projjson(&self) -> Value {
        match self {
            Crs::ProjectedCrs(crs) => crs.to_projjson(),
            Crs::GeogCrs(crs) => crs.to_projjson(),
        }
    }
}

impl ProjectedCrs {
    /// Serialize this projected CRS to a PROJJSON `serde_json::Value`.
    pub fn to_projjson(&self) -> Value {
        let mut obj = Map::new();
        insert_schema(&mut obj);
        obj.insert("type".into(), json!("ProjectedCRS"));
        obj.insert("name".into(), json!(self.name));
        obj.insert("base_crs".into(), base_crs_to_json(&self.base_geodetic_crs));
        obj.insert(
            "conversion".into(),
            conversion_to_json(&self.map_projection),
        );
        obj.insert(
            "coordinate_system".into(),
            cs_to_json(&self.coordinate_system),
        );

        insert_usages(&mut obj, &self.usages);
        if let Some(ref remark) = self.remark {
            obj.insert("remarks".into(), json!(remark));
        }
        insert_ids(&mut obj, &self.identifiers);

        Value::Object(obj)
    }
}

impl GeogCrs {
    /// Serialize this geographic CRS to a PROJJSON `serde_json::Value`.
    pub fn to_projjson(&self) -> Value {
        let mut obj = Map::new();
        insert_schema(&mut obj);
        obj.insert("type".into(), json!("GeographicCRS"));
        obj.insert("name".into(), json!(self.name));

        match &self.datum {
            Datum::ReferenceFrame(rf) => {
                obj.insert("datum".into(), datum_to_json(rf, self.dynamic.as_ref()));
            }
            Datum::Ensemble(ens) => {
                obj.insert("datum_ensemble".into(), ensemble_to_json(ens));
            }
        }

        if let Some(ref dynamic) = self.dynamic
            && let Some(ref model) = dynamic.deformation_model
        {
            let mut m = Map::new();
            m.insert("name".into(), json!(model.name));
            insert_ids(&mut m, &model.identifiers);
            obj.insert("deformation_models".into(), json!([Value::Object(m)]));
        }

        obj.insert(
            "coordinate_system".into(),
            cs_to_json(&self.coordinate_system),
        );

        insert_usages(&mut obj, &self.usages);
        if let Some(ref remark) = self.remark {
            obj.insert("remarks".into(), json!(remark));
        }
        insert_ids(&mut obj, &self.identifiers);

        Value::Object(obj)
    }
}

fn insert_schema(obj: &mut Map<String, Value>) {
    obj.insert(
        "$schema".into(),
        json!("https://proj.org/schemas/v0.7/projjson.schema.json"),
    );
}

// ---------------------------------------------------------------------------
// Base CRS
// ---------------------------------------------------------------------------

fn base_crs_to_json(base: &BaseGeodeticCrs) -> Value {
    let mut obj = Map::new();

    let crs_type = match base.keyword {
        BaseGeodeticCrsKeyword::BaseGeogCrs => "GeographicCRS",
        BaseGeodeticCrsKeyword::BaseGeodCrs => "GeodeticCRS",
    };
    obj.insert("type".into(), json!(crs_type));
    obj.insert("name".into(), json!(base.name));

    match &base.datum {
        Datum::ReferenceFrame(rf) => {
            obj.insert("datum".into(), datum_to_json(rf, base.dynamic.as_ref()));
        }
        Datum::Ensemble(ens) => {
            obj.insert("datum_ensemble".into(), ensemble_to_json(ens));
        }
    }

    // deformation_models live on the CRS in PROJJSON, not inside the datum
    if let Some(ref dynamic) = base.dynamic
        && let Some(ref model) = dynamic.deformation_model
    {
        let mut m = Map::new();
        m.insert("name".into(), json!(model.name));
        insert_ids(&mut m, &model.identifiers);
        obj.insert("deformation_models".into(), json!([Value::Object(m)]));
    }

    // base_crs in a ProjectedCRS typically doesn't have its own coordinate_system
    // in PROJJSON output (it's implied). But if we have ellipsoidal_cs_unit, we could
    // emit one. For now, omit it as PROJ itself does for projected CRS base_crs.

    insert_ids(&mut obj, &base.identifiers);

    Value::Object(obj)
}

// ---------------------------------------------------------------------------
// Datum
// ---------------------------------------------------------------------------

fn datum_to_json(rf: &GeodeticReferenceFrame, dynamic: Option<&DynamicCrs>) -> Value {
    let mut obj = Map::new();

    if let Some(dyn_crs) = dynamic {
        obj.insert("type".into(), json!("DynamicGeodeticReferenceFrame"));
        obj.insert(
            "frame_reference_epoch".into(),
            json!(dyn_crs.frame_reference_epoch),
        );
    } else {
        obj.insert("type".into(), json!("GeodeticReferenceFrame"));
    }

    obj.insert("name".into(), json!(rf.name));

    if let Some(ref anchor) = rf.anchor {
        obj.insert("anchor".into(), json!(anchor));
    }
    if let Some(epoch) = rf.anchor_epoch {
        obj.insert("anchor_epoch".into(), json!(epoch));
    }

    obj.insert("ellipsoid".into(), ellipsoid_to_json(&rf.ellipsoid));

    if let Some(ref pm) = rf.prime_meridian {
        obj.insert("prime_meridian".into(), prime_meridian_to_json(pm));
    }

    insert_ids(&mut obj, &rf.identifiers);

    Value::Object(obj)
}

fn ensemble_to_json(ens: &DatumEnsemble) -> Value {
    let mut obj = Map::new();
    obj.insert("type".into(), json!("DatumEnsemble"));
    obj.insert("name".into(), json!(ens.name));

    let members: Vec<Value> = ens
        .members
        .iter()
        .map(|m| {
            let mut mobj = Map::new();
            mobj.insert("name".into(), json!(m.name));
            insert_ids(&mut mobj, &m.identifiers);
            Value::Object(mobj)
        })
        .collect();
    obj.insert("members".into(), Value::Array(members));

    if let Some(ref ellipsoid) = ens.ellipsoid {
        obj.insert("ellipsoid".into(), ellipsoid_to_json(ellipsoid));
    }

    // PROJJSON schema says accuracy is a string
    obj.insert("accuracy".into(), json!(ens.accuracy.to_string()));

    insert_ids(&mut obj, &ens.identifiers);

    Value::Object(obj)
}

// ---------------------------------------------------------------------------
// Ellipsoid & Prime Meridian
// ---------------------------------------------------------------------------

fn ellipsoid_to_json(e: &Ellipsoid) -> Value {
    let mut obj = Map::new();
    obj.insert("name".into(), json!(e.name));
    obj.insert(
        "semi_major_axis".into(),
        value_with_optional_unit(e.semi_major_axis, e.unit.as_ref()),
    );
    obj.insert("inverse_flattening".into(), json!(e.inverse_flattening));
    insert_ids(&mut obj, &e.identifiers);
    Value::Object(obj)
}

fn prime_meridian_to_json(pm: &PrimeMeridian) -> Value {
    let mut obj = Map::new();
    obj.insert("name".into(), json!(pm.name));
    obj.insert(
        "longitude".into(),
        value_with_optional_unit(pm.irm_longitude, pm.unit.as_ref()),
    );
    insert_ids(&mut obj, &pm.identifiers);
    Value::Object(obj)
}

/// If the unit is present and not the default for the context, emit `{"value": n, "unit": ...}`.
/// Otherwise emit just the number.
fn value_with_optional_unit(value: f64, unit: Option<&Unit>) -> Value {
    match unit {
        Some(u) => {
            json!({
                "value": value,
                "unit": unit_to_json(u),
            })
        }
        None => json!(value),
    }
}

// ---------------------------------------------------------------------------
// Map Projection (Conversion)
// ---------------------------------------------------------------------------

fn conversion_to_json(mp: &MapProjection) -> Value {
    let mut obj = Map::new();
    obj.insert("name".into(), json!(mp.name));
    obj.insert("method".into(), method_to_json(&mp.method));

    if !mp.parameters.is_empty() {
        let params: Vec<Value> = mp.parameters.iter().map(parameter_to_json).collect();
        obj.insert("parameters".into(), Value::Array(params));
    }

    insert_ids(&mut obj, &mp.identifiers);
    Value::Object(obj)
}

fn method_to_json(m: &MapProjectionMethod) -> Value {
    let mut obj = Map::new();
    obj.insert("name".into(), json!(m.name));
    insert_ids(&mut obj, &m.identifiers);
    Value::Object(obj)
}

fn parameter_to_json(p: &MapProjectionParameter) -> Value {
    let mut obj = Map::new();
    obj.insert("name".into(), json!(p.name));
    obj.insert("value".into(), json!(p.value));
    if let Some(ref unit) = p.unit {
        obj.insert("unit".into(), unit_to_json(unit));
    }
    insert_ids(&mut obj, &p.identifiers);
    Value::Object(obj)
}

// ---------------------------------------------------------------------------
// Coordinate System
// ---------------------------------------------------------------------------

fn cs_to_json(cs: &CoordinateSystem) -> Value {
    let mut obj = Map::new();
    obj.insert("subtype".into(), json!(cs.cs_type.to_string()));

    let axes: Vec<Value> = cs
        .axes
        .iter()
        .map(|a| axis_to_json(a, cs.cs_unit.as_ref()))
        .collect();
    obj.insert("axis".into(), Value::Array(axes));

    insert_ids(&mut obj, &cs.identifiers);
    Value::Object(obj)
}

fn axis_to_json(axis: &Axis, cs_unit: Option<&Unit>) -> Value {
    let mut obj = Map::new();

    let (name, abbreviation) = split_axis_name_abbrev(&axis.name_abbrev);
    obj.insert("name".into(), json!(name));
    obj.insert("abbreviation".into(), json!(abbreviation));
    obj.insert("direction".into(), json!(axis.direction));

    // Unit: prefer axis-level unit, fall back to CS-level unit
    let effective_unit = axis.unit.as_ref().or(cs_unit);
    if let Some(unit) = effective_unit {
        obj.insert("unit".into(), unit_to_json(unit));
    }

    if let Some(ref meridian) = axis.meridian {
        obj.insert("meridian".into(), meridian_to_json(meridian));
    }

    if let Some(min) = axis.axis_min_value {
        obj.insert("minimum_value".into(), json!(min));
    }
    if let Some(max) = axis.axis_max_value {
        obj.insert("maximum_value".into(), json!(max));
    }
    if let Some(rm) = axis.range_meaning {
        obj.insert(
            "range_meaning".into(),
            json!(match rm {
                RangeMeaning::Exact => "exact",
                RangeMeaning::Wraparound => "wraparound",
            }),
        );
    }

    insert_ids(&mut obj, &axis.identifiers);
    Value::Object(obj)
}

fn meridian_to_json(m: &Meridian) -> Value {
    json!({
        "longitude": m.value,
        "unit": unit_to_json(&m.unit),
    })
}

/// Split "easting (E)" into ("easting", "E").
/// If no parenthesized abbreviation, use the full string as name and empty abbreviation.
fn split_axis_name_abbrev(name_abbrev: &str) -> (&str, &str) {
    if let Some(paren_start) = name_abbrev.rfind('(')
        && let Some(paren_end) = name_abbrev[paren_start..].find(')')
    {
        let name = name_abbrev[..paren_start].trim();
        let abbrev = &name_abbrev[paren_start + 1..paren_start + paren_end];
        return (name, abbrev);
    }
    (name_abbrev, "")
}

// ---------------------------------------------------------------------------
// Unit
// ---------------------------------------------------------------------------

fn unit_to_json(unit: &Unit) -> Value {
    // Use shorthand for well-known units
    match (unit.name.as_str(), unit.conversion_factor) {
        ("metre", Some(1.0)) => return json!("metre"),
        ("degree", Some(f)) if (f - 0.0174532925199433).abs() < 1e-15 => {
            return json!("degree");
        }
        ("unity", Some(1.0)) => return json!("unity"),
        _ => {}
    }

    let mut obj = Map::new();
    let type_str = match unit.keyword {
        UnitKeyword::AngleUnit => "AngularUnit",
        UnitKeyword::LengthUnit => "LinearUnit",
        UnitKeyword::ScaleUnit => "ScaleUnit",
        UnitKeyword::TimeUnit => "TimeUnit",
        UnitKeyword::ParametricUnit => "ParametricUnit",
        UnitKeyword::Unit => "Unit",
    };
    obj.insert("type".into(), json!(type_str));
    obj.insert("name".into(), json!(unit.name));
    if let Some(factor) = unit.conversion_factor {
        obj.insert("conversion_factor".into(), json!(factor));
    }
    insert_ids(&mut obj, &unit.identifiers);
    Value::Object(obj)
}

// ---------------------------------------------------------------------------
// Identifier
// ---------------------------------------------------------------------------

fn id_to_json(id: &Identifier) -> Value {
    let mut obj = Map::new();
    obj.insert("authority".into(), json!(id.authority_name));
    match &id.authority_unique_id {
        AuthorityId::Number(n) => {
            // Emit as integer if it's a whole number
            if *n == (*n as i64) as f64 {
                obj.insert("code".into(), json!(*n as i64));
            } else {
                obj.insert("code".into(), json!(n));
            }
        }
        AuthorityId::Text(s) => {
            obj.insert("code".into(), json!(s));
        }
    }
    if let Some(ref version) = id.version {
        match version {
            AuthorityId::Number(n) => obj.insert("version".into(), json!(n)),
            AuthorityId::Text(s) => obj.insert("version".into(), json!(s)),
        };
    }
    if let Some(ref citation) = id.citation {
        obj.insert("authority_citation".into(), json!(citation));
    }
    if let Some(ref uri) = id.uri {
        obj.insert("uri".into(), json!(uri));
    }
    Value::Object(obj)
}

/// Insert `id` (single) or `ids` (multiple) into a JSON object map.
fn insert_ids(obj: &mut Map<String, Value>, identifiers: &[Identifier]) {
    match identifiers.len() {
        0 => {}
        1 => {
            obj.insert("id".into(), id_to_json(&identifiers[0]));
        }
        _ => {
            let ids: Vec<Value> = identifiers.iter().map(id_to_json).collect();
            obj.insert("ids".into(), Value::Array(ids));
        }
    }
}

// ---------------------------------------------------------------------------
// Usage & Extent
// ---------------------------------------------------------------------------

fn insert_usages(obj: &mut Map<String, Value>, usages: &[Usage]) {
    if usages.is_empty() {
        return;
    }

    // Use the structured `usages` array form
    let arr: Vec<Value> = usages.iter().map(usage_to_json).collect();
    obj.insert("usages".into(), Value::Array(arr));
}

fn usage_to_json(u: &Usage) -> Value {
    let mut obj = Map::new();
    obj.insert("scope".into(), json!(u.scope));
    if let Some(ref area) = u.area {
        obj.insert("area".into(), json!(area));
    }
    if let Some(ref bbox) = u.bbox {
        obj.insert("bbox".into(), bbox_to_json(bbox));
    }
    if let Some(ref ve) = u.vertical_extent {
        obj.insert("vertical_extent".into(), vertical_extent_to_json(ve));
    }
    if let Some(ref te) = u.temporal_extent {
        obj.insert("temporal_extent".into(), temporal_extent_to_json(te));
    }
    Value::Object(obj)
}

fn bbox_to_json(bbox: &BBox) -> Value {
    json!({
        "south_latitude": bbox.lower_left_latitude,
        "west_longitude": bbox.lower_left_longitude,
        "north_latitude": bbox.upper_right_latitude,
        "east_longitude": bbox.upper_right_longitude,
    })
}

fn vertical_extent_to_json(ve: &VerticalExtent) -> Value {
    let mut obj = Map::new();
    obj.insert("minimum".into(), json!(ve.minimum_height));
    obj.insert("maximum".into(), json!(ve.maximum_height));
    if let Some(ref unit) = ve.unit {
        obj.insert("unit".into(), unit_to_json(unit));
    }
    Value::Object(obj)
}

fn temporal_extent_to_json(te: &TemporalExtent) -> Value {
    json!({
        "start": te.start,
        "end": te.end,
    })
}
