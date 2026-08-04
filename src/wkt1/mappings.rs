//! Lookup logic over the WKT1 method/parameter mapping tables.
//!
//! The tables themselves live in [`super::mappings_data`] (transcribed from
//! PROJ). This module selects the right method entry for a parsed
//! `PROJECTION` + `PARAMETER` set and maps the parameter names to their EPSG
//! equivalents.

use crate::error::ParseError;

use super::esri_alias_data::{DATA_METHOD_NAME_CODES, DATA_PARAM_NAME_CODES};
use super::mappings_data::{
    EPSG_METHOD_NAME_CODES, EPSG_PARAM_NAME_CODES, ESRI_METHODS, GDAL_METHODS, PARAM_UNIT_TYPES,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum UnitType {
    Angular,
    Linear,
    Scale,
    /// No unit or an unknown unit type (e.g. polynomial coefficients).
    None_,
}

pub(crate) struct ParamMapping {
    pub wkt1_name: &'static str,
    pub epsg_name: &'static str,
    pub epsg_code: i32,
    pub unit_type: UnitType,
}

pub(crate) struct MethodMapping {
    pub wkt1_name: &'static str,
    pub epsg_name: &'static str,
    pub epsg_code: i32,
    pub params: &'static [ParamMapping],
}

pub(crate) struct EsriParamMapping {
    pub esri_name: &'static str,
    /// Empty when the parameter has no EPSG equivalent; such parameters are
    /// accepted only when their value equals `fixed_value` and are dropped.
    pub epsg_name: &'static str,
    pub epsg_code: i32,
    pub fixed_value: &'static str,
    #[allow(dead_code)] // kept to mirror PROJ's table shape
    pub is_fixed_value: bool,
}

pub(crate) struct EsriMethodMapping {
    pub esri_name: &'static str,
    pub epsg_name: &'static str,
    pub epsg_code: i32,
    pub params: &'static [EsriParamMapping],
}

/// A parameter resolved to its EPSG name.
pub(crate) struct ResolvedParam {
    pub epsg_name: String,
    pub epsg_code: i32,
    pub unit_type: UnitType,
    pub value: f64,
}

/// A projection method resolved to its EPSG name.
pub(crate) struct ResolvedMethod {
    pub epsg_name: String,
    /// 0 when the method has no EPSG code.
    pub epsg_code: i32,
    pub params: Vec<ResolvedParam>,
}

/// Properties of the enclosing CRS that pick between method variants sharing a
/// single WKT1 name. WKT1 records only the shared name, so the variant has to
/// be inferred from the CRS around it.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MethodContext {
    /// The base ellipsoid is a sphere (zero inverse flattening).
    pub is_sphere: bool,
    /// The coordinate system has a south- or west-pointing axis.
    pub south_west_oriented: bool,
}

impl MethodContext {
    /// How well an EPSG method name fits this context. EPSG marks the variants
    /// that WKT1 conflates in the method name itself, so the qualifiers are
    /// read from the name rather than enumerated as code pairs.
    fn score(self, epsg_name: &str) -> i32 {
        let mut score = 0;
        score += if epsg_name.contains("(Spherical)") == self.is_sphere {
            1
        } else {
            -1
        };
        score += if epsg_name.contains("(North Orientated)") == !self.south_west_oriented {
            1
        } else {
            -1
        };
        score
    }
}

/// Why a table entry could not interpret the input parameters. Kept so the
/// reported error names an actual cause instead of defaulting to a blank.
enum Reject {
    /// The entry does not define this WKT1 parameter name.
    UnknownName(String),
    /// The entry defines the name but cannot accept this value: either a
    /// fixed-value parameter set to something else, or a second spelling of an
    /// EPSG parameter the entry has already seen with a different value.
    BadValue { name: String, value: f64 },
}

impl Reject {
    fn into_error(self, method: &str) -> ParseError {
        match self {
            Reject::UnknownName(name) => ParseError::UnknownParameter {
                method: method.to_string(),
                name,
            },
            Reject::BadValue { name, value } => ParseError::UnsupportedParameterValue {
                method: method.to_string(),
                name,
                value,
            },
        }
    }
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-10 * a.abs().max(b.abs()).max(1.0)
}

/// Whether dropping a parameter carrying this value leaves the projection
/// unchanged: unity for a scale factor, zero for everything else.
fn is_identity_value(unit_type: UnitType, value: f64) -> bool {
    match unit_type {
        UnitType::Scale => approx_eq(value, 1.0),
        _ => approx_eq(value, 0.0),
    }
}

fn unit_type_for_code(code: i32) -> Option<UnitType> {
    PARAM_UNIT_TYPES
        .binary_search_by_key(&code, |(c, _)| *c)
        .ok()
        .map(|i| PARAM_UNIT_TYPES[i].1)
}

/// Guess a unit type from an EPSG parameter name (used when the parameter has
/// no EPSG code in the table).
fn unit_type_from_name(epsg_name: &str) -> UnitType {
    let lower = epsg_name.to_ascii_lowercase();
    let is_coefficient = {
        // Polynomial coefficients: "C1" .. "C10", "A0" .. "B3", etc.
        let mut chars = epsg_name.chars();
        matches!(chars.next(), Some('A'..='C')) && chars.all(|c| c.is_ascii_digit())
    };
    if lower.contains("latitude")
        || lower.contains("longitude")
        || lower.contains("azimuth")
        || lower.contains("angle")
        || lower.contains("rotation")
        || lower == "zone width"
    {
        UnitType::Angular
    } else if lower.contains("scale") || lower.contains("scaling") || is_coefficient {
        UnitType::Scale
    } else {
        UnitType::Linear
    }
}

/// Resolve a WKT1 `PROJECTION` name and its `PARAMETER`s to EPSG names.
///
/// Tries the GDAL table, then the ESRI table, then EPSG names directly. Within
/// a table, entries sharing a WKT1 name are filtered by the parameter set (an
/// entry matches when it defines every input parameter and every "fixed value"
/// parameter carries its expected value) and the survivors are ranked by how
/// well they fit `context`.
pub(crate) fn resolve_method(
    method_name: &str,
    input_params: &[(String, f64)],
    context: MethodContext,
) -> Result<ResolvedMethod, ParseError> {
    // A method name can appear in several tables (e.g. "Azimuthal_Equidistant"
    // is both a GDAL and an ESRI name with different parameter names), so a
    // parameter-set mismatch in one table falls through to the next. The
    // resolvers are tried lazily: each is a full table scan.
    type Resolver = fn(&str, &[(String, f64)], MethodContext) -> Option<ResolvedAttempt>;
    let mut first_err: Option<ParseError> = None;
    for resolve in [resolve_gdal as Resolver, resolve_esri, resolve_epsg_names] {
        match resolve(method_name, input_params, context) {
            Some(Ok(resolved)) => return Ok(disambiguate_stereographic(resolved)),
            Some(Err(e)) => {
                first_err.get_or_insert(e);
            }
            None => {}
        }
    }
    Err(
        first_err.unwrap_or_else(|| ParseError::UnknownProjectionMethod {
            name: method_name.to_string(),
        }),
    )
}

type ResolvedAttempt = Result<ResolvedMethod, ParseError>;

/// Compare two names ignoring case and every non-alphanumeric character, so
/// that `"Lambert_Conic_Conformal_(1SP)"` matches `"Lambert Conic Conformal
/// (1SP)"`. Allocation-free: both sides are compared as character streams.
pub(super) fn eq_normalized(a: &str, b: &str) -> bool {
    let normalize = |s: &str| {
        s.chars()
            .filter(char::is_ascii_alphanumeric)
            .map(|c| c.to_ascii_lowercase())
            .collect::<Vec<_>>()
    };
    normalize(a) == normalize(b)
}

/// EPSG constrains some variants to parameter values WKT1 does not enforce; a
/// candidate that contradicts its own definition is not a valid reading of the
/// input, even though its parameter *names* all matched.
fn violates_variant_domain(method: &ResolvedMethod) -> bool {
    // Polar Stereographic variant A puts the natural origin at a pole;
    // anywhere else the latitude is a standard parallel, i.e. variant B.
    if method.epsg_code == 9810
        && let Some(lat) = method
            .params
            .iter()
            .find(|p| p.epsg_name == "Latitude of natural origin")
    {
        return !approx_eq(lat.value.abs(), 90.0);
    }
    false
}

/// Choose among the entries that accepted the input: prefer ones whose own
/// EPSG definition the values satisfy, then the best contextual fit, then
/// table order.
///
/// `name_is_ambiguous` says whether the WKT1 name maps to more than one table
/// entry. When it does not, the string named its variant explicitly and is
/// taken at its word even if the values sit outside the variant's domain; when
/// it does, the domain is a real discriminator and violating every candidate
/// means the input is self-contradictory.
fn pick(
    matches: Vec<(ResolvedMethod, &'static str)>,
    context: MethodContext,
    name_is_ambiguous: bool,
) -> Option<ResolvedMethod> {
    let (in_domain, rest): (Vec<_>, Vec<_>) = matches
        .into_iter()
        .partition(|(m, _)| !violates_variant_domain(m));
    let pool = if in_domain.is_empty() && !name_is_ambiguous {
        rest
    } else {
        in_domain
    };

    let mut best: Option<(ResolvedMethod, i32)> = None;
    for (candidate, epsg_name) in pool {
        let score = context.score(epsg_name);
        if best.as_ref().is_none_or(|(_, b)| score > *b) {
            best = Some((candidate, score));
        }
    }
    best.map(|(c, _)| c)
}

/// Fallback for WKT1 strings carrying (possibly morphed) EPSG names instead
/// of traditional WKT1 names: PROJ exports methods missing from its WKT1
/// table as e.g. `PROJECTION["Lambert_Conic_Conformal_(West_Orientated)"],
/// PARAMETER["Latitude of natural origin",...]`.
fn resolve_epsg_names(
    method_name: &str,
    input_params: &[(String, f64)],
    _context: MethodContext,
) -> Option<ResolvedAttempt> {
    let (epsg_name, epsg_code) = EPSG_METHOD_NAME_CODES
        .iter()
        .chain(DATA_METHOD_NAME_CODES)
        .find(|(name, _)| eq_normalized(name, method_name))?;

    let mut resolved = Vec::with_capacity(input_params.len());
    for (name, value) in input_params {
        let Some((pname, pcode)) = EPSG_PARAM_NAME_CODES
            .iter()
            .chain(DATA_PARAM_NAME_CODES)
            .find(|(pname, _)| eq_normalized(pname, name))
        else {
            return Some(Err(ParseError::UnknownParameter {
                method: method_name.to_string(),
                name: name.clone(),
            }));
        };
        resolved.push(ResolvedParam {
            epsg_name: pname.to_string(),
            epsg_code: *pcode,
            unit_type: unit_type_for_code(*pcode).unwrap_or_else(|| unit_type_from_name(pname)),
            value: *value,
        });
    }
    Some(Ok(ResolvedMethod {
        epsg_name: epsg_name.to_string(),
        epsg_code: *epsg_code,
        params: resolved,
    }))
}

/// Plain "Stereographic" (GDAL and ESRI) is polar at the poles and oblique
/// elsewhere; the tables map it to a placeholder name with no EPSG code.
fn disambiguate_stereographic(mut resolved: ResolvedMethod) -> ResolvedMethod {
    if resolved.epsg_name != "Stereographic" || resolved.epsg_code != 0 {
        return resolved;
    }
    let lat = resolved
        .params
        .iter()
        .find(|p| p.epsg_name == "Latitude of natural origin")
        .map(|p| p.value)
        .unwrap_or(0.0);
    if approx_eq(lat.abs(), 90.0) {
        resolved.epsg_name = "Polar Stereographic (variant A)".to_string();
        resolved.epsg_code = 9810;
    } else {
        resolved.epsg_name = "Oblique Stereographic".to_string();
        resolved.epsg_code = 9809;
    }
    resolved
}

fn resolve_gdal(
    method_name: &str,
    input_params: &[(String, f64)],
    context: MethodContext,
) -> Option<ResolvedAttempt> {
    let entries: Vec<&MethodMapping> = GDAL_METHODS
        .iter()
        .filter(|m| m.wkt1_name.eq_ignore_ascii_case(method_name))
        .collect();
    if entries.is_empty() {
        return None;
    }

    let mut matches = Vec::new();
    let mut reject: Option<Reject> = None;
    for candidate in &entries {
        match match_gdal(candidate, &entries, input_params) {
            Ok(params) => matches.push((
                ResolvedMethod {
                    epsg_name: candidate.epsg_name.to_string(),
                    epsg_code: candidate.epsg_code,
                    params,
                },
                candidate.epsg_name,
            )),
            Err(r) => {
                reject.get_or_insert(r);
            }
        }
    }

    match pick(matches, context, entries.len() > 1) {
        Some(resolved) => Some(Ok(resolved)),
        // The method name is known but no entry accepts this parameter set.
        None => Some(Err(reject
            .expect("a table entry exists, so it either matched or rejected")
            .into_error(method_name))),
    }
}

/// Match `input_params` against one GDAL table entry.
///
/// A parameter that a *sibling* variant of the same WKT1 method defines may be
/// omitted here when it carries its identity value: WKT1 producers write e.g.
/// `scale_factor` unconditionally, and Polar Stereographic variant B has no
/// scale factor, so a unity value is redundant rather than contradictory.
fn match_gdal(
    candidate: &'static MethodMapping,
    siblings: &[&'static MethodMapping],
    input_params: &[(String, f64)],
) -> Result<Vec<ResolvedParam>, Reject> {
    let find = |entry: &'static MethodMapping, name: &str| {
        entry
            .params
            .iter()
            .find(|p| p.wkt1_name.eq_ignore_ascii_case(name) || eq_normalized(p.epsg_name, name))
    };

    let mut resolved = Vec::with_capacity(input_params.len());
    for (name, value) in input_params {
        if let Some(p) = find(candidate, name) {
            resolved.push(ResolvedParam {
                epsg_name: p.epsg_name.to_string(),
                epsg_code: p.epsg_code,
                unit_type: if p.unit_type == UnitType::None_ {
                    unit_type_from_name(p.epsg_name)
                } else {
                    p.unit_type
                },
                value: *value,
            });
            continue;
        }
        match siblings.iter().find_map(|s| find(s, name)) {
            Some(p) if is_identity_value(p.unit_type, *value) => continue, // redundant
            Some(p) => {
                return Err(Reject::BadValue {
                    name: p.wkt1_name.to_string(),
                    value: *value,
                });
            }
            None => return Err(Reject::UnknownName(name.clone())),
        }
    }
    Ok(resolved)
}

fn resolve_esri(
    method_name: &str,
    input_params: &[(String, f64)],
    context: MethodContext,
) -> Option<ResolvedAttempt> {
    let entries: Vec<&EsriMethodMapping> = ESRI_METHODS
        .iter()
        .filter(|m| m.esri_name.eq_ignore_ascii_case(method_name))
        .collect();
    if entries.is_empty() {
        return None;
    }

    let mut matches = Vec::new();
    let mut reject: Option<Reject> = None;
    for candidate in &entries {
        match match_esri(candidate, input_params) {
            Ok(params) => matches.push((
                ResolvedMethod {
                    epsg_name: candidate.epsg_name.to_string(),
                    epsg_code: candidate.epsg_code,
                    params,
                },
                candidate.epsg_name,
            )),
            Err(r) => {
                reject.get_or_insert(r);
            }
        }
    }

    match pick(matches, context, entries.len() > 1) {
        Some(resolved) => Some(Ok(resolved)),
        None => Some(Err(reject
            .expect("a table entry exists, so it either matched or rejected")
            .into_error(method_name))),
    }
}

/// Match `input_params` against one ESRI table entry.
fn match_esri(
    candidate: &EsriMethodMapping,
    input_params: &[(String, f64)],
) -> Result<Vec<ResolvedParam>, Reject> {
    let mut resolved: Vec<ResolvedParam> = Vec::with_capacity(input_params.len());
    for (name, value) in input_params {
        let Some(p) = candidate
            .params
            .iter()
            .find(|p| p.esri_name.eq_ignore_ascii_case(name))
        else {
            return Err(Reject::UnknownName(name.clone()));
        };
        if p.epsg_name.is_empty() {
            // No EPSG equivalent: only acceptable at its expected value. A
            // malformed table literal must fail loudly rather than default.
            let expected: f64 = p
                .fixed_value
                .parse()
                .expect("generated table holds numeric fixed values");
            if !approx_eq(*value, expected) {
                return Err(Reject::BadValue {
                    name: p.esri_name.to_string(),
                    value: *value,
                });
            }
            continue; // validated, dropped
        }
        // ESRI sometimes encodes the same EPSG parameter under two names
        // (e.g. LCC 1SP: Standard_Parallel_1 and Latitude_Of_Origin are both
        // the latitude of natural origin). Equal values collapse; differing
        // ones mean this entry is the wrong interpretation.
        if let Some(prev) = resolved.iter().find(|r| r.epsg_name == p.epsg_name) {
            if approx_eq(prev.value, *value) {
                continue;
            }
            return Err(Reject::BadValue {
                name: p.esri_name.to_string(),
                value: *value,
            });
        }
        resolved.push(ResolvedParam {
            epsg_name: p.epsg_name.to_string(),
            epsg_code: p.epsg_code,
            unit_type: unit_type_for_code(p.epsg_code)
                .unwrap_or_else(|| unit_type_from_name(p.epsg_name)),
            value: *value,
        });
    }
    Ok(resolved)
}
