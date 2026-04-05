/// A top-level coordinate reference system.
///
/// This enum dispatches over the different CRS types that this parser can handle.
#[derive(Debug, Clone, PartialEq)]
pub enum Crs {
    /// A projected CRS (WKT2 keyword: `PROJCRS`).
    ProjectedCrs(Box<ProjectedCrs>),
    /// A geographic CRS (WKT2 keyword: `GEOGCRS`).
    GeogCrs(Box<GeogCrs>),
    /// A geodetic CRS (WKT2 keyword: `GEODCRS`).
    GeodCrs(Box<GeodCrs>),
    /// A vertical CRS (WKT2 keyword: `VERTCRS`).
    VertCrs(Box<VertCrs>),
    /// A compound CRS (WKT2 keyword: `COMPOUNDCRS`).
    CompoundCrs(Box<CompoundCrs>),
}

impl Crs {
    /// Serialize this CRS to a WKT2 string.
    ///
    /// This is an alias for [`ToString::to_string()`] provided for discoverability.
    pub fn to_wkt2(&self) -> String {
        self.to_string()
    }

    /// Extract the EPSG code from this CRS's identifiers, if present.
    pub fn to_epsg(&self) -> Option<i32> {
        match self {
            Crs::ProjectedCrs(crs) => crs.to_epsg(),
            Crs::GeogCrs(crs) => crs.to_epsg(),
            Crs::GeodCrs(crs) => crs.to_epsg(),
            Crs::VertCrs(crs) => crs.to_epsg(),
            Crs::CompoundCrs(crs) => crs.to_epsg(),
        }
    }
}

/// The keyword used for a geographic CRS.
#[derive(Debug, Clone, PartialEq)]
pub enum GeogCrsKeyword {
    /// `GEOGCRS` -- the preferred keyword.
    GeogCrs,
    /// `GEOGRAPHICCRS` -- the long form.
    GeographicCrs,
}

/// A geographic coordinate reference system (GEOGCRS).
///
/// A geographic CRS uses an ellipsoidal coordinate system with latitude and longitude.
///
/// WKT2 keywords: `GEOGCRS` (preferred), `GEOGRAPHICCRS`.
#[derive(Debug, Clone, PartialEq)]
pub struct GeogCrs {
    /// Which keyword was used in the WKT.
    pub keyword: GeogCrsKeyword,
    /// The name of the geographic CRS (e.g. "WGS 84").
    pub name: String,
    /// Present only if the CRS is dynamic (has a time-varying reference frame).
    pub dynamic: Option<DynamicCrs>,
    /// The datum: either a geodetic reference frame or a datum ensemble.
    pub datum: Datum,
    /// The coordinate system describing axes, their directions, and units.
    pub coordinate_system: CoordinateSystem,
    /// Zero or more scope-extent pairings describing the applicability of this CRS.
    pub usages: Vec<Usage>,
    /// Zero or more external identifiers referencing this CRS.
    pub identifiers: Vec<Identifier>,
    /// An optional free-text remark about this CRS.
    pub remark: Option<String>,
}

impl GeogCrs {
    /// Extract the EPSG code from this CRS's identifiers, if present.
    pub fn to_epsg(&self) -> Option<i32> {
        self.identifiers.iter().find_map(|id| {
            if id.authority_name == "EPSG"
                && let AuthorityId::Number(n) = id.authority_unique_id
            {
                return Some(n as i32);
            }
            None
        })
    }
}

/// The keyword used for a geodetic CRS.
#[derive(Debug, Clone, PartialEq)]
pub enum GeodCrsKeyword {
    /// `GEODCRS` -- the preferred keyword.
    GeodCrs,
    /// `GEODETICCRS` -- the long form.
    GeodeticCrs,
}

/// A geodetic coordinate reference system (GEODCRS).
///
/// A geodetic CRS uses a Cartesian or spherical coordinate system.
///
/// WKT2 keywords: `GEODCRS` (preferred), `GEODETICCRS`.
#[derive(Debug, Clone, PartialEq)]
pub struct GeodCrs {
    /// Which keyword was used in the WKT.
    pub keyword: GeodCrsKeyword,
    /// The name of the geodetic CRS (e.g. "WGS 84").
    pub name: String,
    /// Present only if the CRS is dynamic (has a time-varying reference frame).
    pub dynamic: Option<DynamicCrs>,
    /// The datum: either a geodetic reference frame or a datum ensemble.
    pub datum: Datum,
    /// The coordinate system describing axes, their directions, and units.
    pub coordinate_system: CoordinateSystem,
    /// Zero or more scope-extent pairings describing the applicability of this CRS.
    pub usages: Vec<Usage>,
    /// Zero or more external identifiers referencing this CRS.
    pub identifiers: Vec<Identifier>,
    /// An optional free-text remark about this CRS.
    pub remark: Option<String>,
}

impl GeodCrs {
    /// Extract the EPSG code from this CRS's identifiers, if present.
    pub fn to_epsg(&self) -> Option<i32> {
        self.identifiers.iter().find_map(|id| {
            if id.authority_name == "EPSG"
                && let AuthorityId::Number(n) = id.authority_unique_id
            {
                return Some(n as i32);
            }
            None
        })
    }
}

/// The keyword used for a vertical CRS.
#[derive(Debug, Clone, PartialEq)]
pub enum VertCrsKeyword {
    /// `VERTCRS` -- the preferred keyword.
    VertCrs,
    /// `VERTICALCRS` -- the long form.
    VerticalCrs,
}

/// A vertical coordinate reference system (VERTCRS).
///
/// A vertical CRS uses a vertical coordinate system (height or depth).
/// It may be a standalone CRS (with a datum) or a derived CRS (with a base
/// vertical CRS and a deriving conversion).
///
/// WKT2 keywords: `VERTCRS` (preferred), `VERTICALCRS`.
#[derive(Debug, Clone, PartialEq)]
pub struct VertCrs {
    /// Which keyword was used in the WKT.
    pub keyword: VertCrsKeyword,
    /// The name of the vertical CRS (e.g. "NAVD88").
    pub name: String,
    /// The source of this CRS: either a datum or a base vertical CRS with a
    /// deriving conversion.
    pub source: VertCrsSource,
    /// The coordinate system describing axes, their directions, and units.
    pub coordinate_system: CoordinateSystem,
    /// Zero or more geoid model references.
    pub geoid_models: Vec<GeoidModel>,
    /// Zero or more scope-extent pairings describing the applicability of this CRS.
    pub usages: Vec<Usage>,
    /// Zero or more external identifiers referencing this CRS.
    pub identifiers: Vec<Identifier>,
    /// An optional free-text remark about this CRS.
    pub remark: Option<String>,
}

/// Whether a vertical CRS is standalone (datum-based) or derived from a base
/// vertical CRS.
#[derive(Debug, Clone, PartialEq)]
pub enum VertCrsSource {
    /// A standalone vertical CRS with a datum.
    Datum {
        /// Present only if the CRS is dynamic.
        dynamic: Option<DynamicCrs>,
        /// The datum: either a vertical reference frame or a datum ensemble.
        datum: VerticalDatum,
    },
    /// A derived vertical CRS with a base CRS and a deriving conversion.
    Derived {
        /// The base vertical CRS from which this CRS is derived.
        base_vert_crs: BaseVertCrs,
        /// The conversion applied to the base CRS.
        deriving_conversion: MapProjection,
    },
}

/// The base vertical CRS of a derived vertical CRS.
///
/// WKT2 keyword: `BASEVERTCRS`.
#[derive(Debug, Clone, PartialEq)]
pub struct BaseVertCrs {
    /// The name of the base CRS.
    pub name: String,
    /// Present only if the base CRS is dynamic.
    pub dynamic: Option<DynamicCrs>,
    /// The datum: either a vertical reference frame or a datum ensemble.
    pub datum: VerticalDatum,
    /// Identifiers for this base CRS.
    pub identifiers: Vec<Identifier>,
}

impl VertCrs {
    /// Extract the EPSG code from this CRS's identifiers, if present.
    pub fn to_epsg(&self) -> Option<i32> {
        self.identifiers.iter().find_map(|id| {
            if id.authority_name == "EPSG"
                && let AuthorityId::Number(n) = id.authority_unique_id
            {
                return Some(n as i32);
            }
            None
        })
    }
}

/// A vertical datum is either a vertical reference frame or a datum ensemble.
#[derive(Debug, Clone, PartialEq)]
pub enum VerticalDatum {
    /// A single vertical reference frame.
    ReferenceFrame(VerticalReferenceFrame),
    /// An ensemble of vertical reference frames.
    Ensemble(Box<DatumEnsemble>),
}

/// The keyword used for a vertical reference frame.
#[derive(Debug, Clone, PartialEq)]
pub enum VerticalReferenceFrameKeyword {
    /// `VDATUM` -- the preferred keyword.
    VDatum,
    /// `VRF` -- vertical reference frame.
    Vrf,
    /// `VERTICALDATUM` -- the fully spelled-out form.
    VerticalDatum,
}

/// A vertical reference frame (vertical datum).
///
/// WKT2 keywords: `VDATUM` (preferred), `VRF`, `VERTICALDATUM`.
#[derive(Debug, Clone, PartialEq)]
pub struct VerticalReferenceFrame {
    /// Which keyword was used in the WKT.
    pub keyword: VerticalReferenceFrameKeyword,
    /// The datum name (e.g. "North American Vertical Datum 1988").
    pub name: String,
    /// A textual description of the datum anchor point.
    pub anchor: Option<String>,
    /// The epoch at which a derived static reference frame is aligned to its parent
    /// dynamic frame.
    pub anchor_epoch: Option<f64>,
    /// Identifiers for this datum.
    pub identifiers: Vec<Identifier>,
}

/// A reference to a geoid model associated with a vertical CRS.
///
/// WKT2 keyword: `GEOIDMODEL`.
#[derive(Debug, Clone, PartialEq)]
pub struct GeoidModel {
    /// The name of the geoid model.
    pub name: String,
    /// Identifiers for this geoid model.
    pub identifiers: Vec<Identifier>,
}

/// A compound coordinate reference system (COMPOUNDCRS).
///
/// A compound CRS is a non-repeating sequence of two or more independent CRSs.
///
/// WKT2 keyword: `COMPOUNDCRS`.
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundCrs {
    /// The name of the compound CRS (e.g. "NAD83 + NAVD88").
    pub name: String,
    /// The constituent single CRSs (at least two).
    pub components: Vec<SingleCrs>,
    /// Zero or more scope-extent pairings describing the applicability of this CRS.
    pub usages: Vec<Usage>,
    /// Zero or more external identifiers referencing this CRS.
    pub identifiers: Vec<Identifier>,
    /// An optional free-text remark about this CRS.
    pub remark: Option<String>,
}

impl CompoundCrs {
    /// Extract the EPSG code from this CRS's identifiers, if present.
    pub fn to_epsg(&self) -> Option<i32> {
        self.identifiers.iter().find_map(|id| {
            if id.authority_name == "EPSG"
                && let AuthorityId::Number(n) = id.authority_unique_id
            {
                return Some(n as i32);
            }
            None
        })
    }
}

/// A single (non-compound) CRS, used as a component of a compound CRS.
#[derive(Debug, Clone, PartialEq)]
pub enum SingleCrs {
    /// A projected CRS.
    ProjectedCrs(Box<ProjectedCrs>),
    /// A geographic CRS.
    GeogCrs(Box<GeogCrs>),
    /// A geodetic CRS.
    GeodCrs(Box<GeodCrs>),
    /// A vertical CRS.
    VertCrs(Box<VertCrs>),
    /// An unsupported CRS type, stored as raw WKT text.
    Other(String),
}

/// A projected coordinate reference system (PROJCRS).
///
/// A projected CRS is derived from a geographic CRS by applying a map projection.
/// It uses Cartesian coordinates (easting/northing) rather than angular coordinates.
///
/// WKT2 keywords: `PROJCRS` (preferred), `PROJECTEDCRS` (not supported by this parser).
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectedCrs {
    /// The name of the projected CRS (e.g. "WGS 84 / UTM zone 31N").
    pub name: String,
    /// The base geodetic or geographic CRS from which this CRS is derived.
    pub base_geodetic_crs: BaseGeodeticCrs,
    /// The map projection (conversion) applied to the base CRS.
    pub map_projection: MapProjection,
    /// The coordinate system describing axes, their directions, and units.
    pub coordinate_system: CoordinateSystem,
    /// Zero or more scope-extent pairings describing the applicability of this CRS.
    pub usages: Vec<Usage>,
    /// Zero or more external identifiers referencing this CRS.
    pub identifiers: Vec<Identifier>,
    /// An optional free-text remark about this CRS.
    pub remark: Option<String>,
}

impl ProjectedCrs {
    /// Extract the EPSG code from this CRS's identifiers, if present.
    ///
    /// Returns `Some(code)` if the CRS has an identifier with authority "EPSG"
    /// and a numeric code, or `None` otherwise.
    pub fn to_epsg(&self) -> Option<i32> {
        self.identifiers.iter().find_map(|id| {
            if id.authority_name == "EPSG"
                && let AuthorityId::Number(n) = id.authority_unique_id
            {
                return Some(n as i32);
            }
            None
        })
    }
}

/// A coordinate system definition, consisting of a type, dimension, axes, and an optional
/// shared unit.
///
/// WKT2 keyword: `CS`.
#[derive(Debug, Clone, PartialEq)]
pub struct CoordinateSystem {
    /// The type of coordinate system (e.g. Cartesian, ellipsoidal).
    pub cs_type: CsType,
    /// The number of dimensions (1, 2, or 3).
    pub dimension: u8,
    /// Identifiers for the coordinate system itself.
    pub identifiers: Vec<Identifier>,
    /// The axes of the coordinate system. These appear as siblings after the `CS[...]` node.
    pub axes: Vec<Axis>,
    /// An optional unit shared by all axes. If absent, each axis specifies its own unit.
    pub cs_unit: Option<Unit>,
}

/// The type of a coordinate system.
#[derive(Debug, Clone, PartialEq)]
pub enum CsType {
    /// An affine coordinate system.
    Affine,
    /// A Cartesian coordinate system with orthogonal straight axes.
    Cartesian,
    /// A 3D coordinate system with a polar and a longitudinal axis, plus a straight axis.
    Cylindrical,
    /// A coordinate system on the surface of an ellipsoid using latitude and longitude.
    Ellipsoidal,
    /// A 1D coordinate system along a straight line.
    Linear,
    /// A coordinate system for parametric values (e.g. pressure levels).
    Parametric,
    /// A 2D coordinate system with a straight radial axis and an angular axis.
    Polar,
    /// A 3D coordinate system on a sphere using two angular coordinates plus radius.
    Spherical,
    /// A 1D coordinate system for height or depth.
    Vertical,
    /// A temporal coordinate system using integer counts of a time unit.
    TemporalCount,
    /// A temporal coordinate system using real-valued measurements of a time unit.
    TemporalMeasure,
    /// A coordinate system using discrete ordinal values (ranks).
    Ordinal,
    /// A temporal coordinate system using date-time values.
    TemporalDateTime,
}

/// A single axis of a coordinate system.
///
/// WKT2 keyword: `AXIS`.
#[derive(Debug, Clone, PartialEq)]
pub struct Axis {
    /// The axis name and/or abbreviation (e.g. "easting (E)").
    pub name_abbrev: String,
    /// The axis direction (e.g. "north", "east", "up", "clockwise").
    pub direction: String,
    /// The meridian from which the axis direction is measured. Only used with
    /// certain directions like "north" or "south".
    pub meridian: Option<Meridian>,
    /// The bearing angle for "clockwise" or "counterClockwise" directions, in degrees.
    pub bearing: Option<f64>,
    /// The ordering of this axis within the coordinate system (1-based).
    pub order: Option<u32>,
    /// The unit for values along this axis. If absent, the coordinate system's shared unit applies.
    pub unit: Option<Unit>,
    /// The minimum value for this axis.
    pub axis_min_value: Option<f64>,
    /// The maximum value for this axis.
    pub axis_max_value: Option<f64>,
    /// Whether the axis range is exact or wraps around (e.g. longitude 0-360).
    pub range_meaning: Option<RangeMeaning>,
    /// Identifiers for this axis.
    pub identifiers: Vec<Identifier>,
}

/// A meridian from which an axis direction is measured.
///
/// WKT2 keyword: `MERIDIAN`.
#[derive(Debug, Clone, PartialEq)]
pub struct Meridian {
    /// The meridian value (typically longitude in the given angle unit).
    pub value: f64,
    /// The angle unit for the meridian value.
    pub unit: Unit,
}

/// How the axis range between min and max values is interpreted.
///
/// WKT2 keyword: `RANGEMEANING`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RangeMeaning {
    /// The range boundaries are exact limits.
    Exact,
    /// The range wraps around (e.g. longitude 0 to 360 degrees).
    Wraparound,
}

/// A map projection (coordinate conversion) applied to a base CRS to produce a projected CRS.
///
/// WKT2 keyword: `CONVERSION`.
#[derive(Debug, Clone, PartialEq)]
pub struct MapProjection {
    /// The name of the map projection (e.g. "UTM zone 31N").
    pub name: String,
    /// The projection method (e.g. "Transverse Mercator").
    pub method: MapProjectionMethod,
    /// The parameters of the projection (e.g. central meridian, scale factor).
    pub parameters: Vec<MapProjectionParameter>,
    /// Identifiers for this conversion as a whole.
    pub identifiers: Vec<Identifier>,
}

/// The method (algorithm) used by a map projection.
///
/// WKT2 keywords: `METHOD` (preferred), `PROJECTION` (backward compatibility).
#[derive(Debug, Clone, PartialEq)]
pub struct MapProjectionMethod {
    /// The name of the method (e.g. "Transverse Mercator").
    pub name: String,
    /// Identifiers for this method.
    pub identifiers: Vec<Identifier>,
}

/// A single parameter of a map projection.
///
/// WKT2 keyword: `PARAMETER`.
#[derive(Debug, Clone, PartialEq)]
pub struct MapProjectionParameter {
    /// The parameter name (e.g. "Latitude of natural origin").
    pub name: String,
    /// The numeric value of the parameter.
    pub value: f64,
    /// The unit for the parameter value (angle, length, or scale).
    pub unit: Option<Unit>,
    /// Identifiers for this parameter.
    pub identifiers: Vec<Identifier>,
}

/// The keyword used for a base geodetic CRS within a projected CRS.
#[derive(Debug, Clone, PartialEq)]
pub enum BaseGeodeticCrsKeyword {
    /// `BASEGEODCRS` -- a geodetic (geocentric) base CRS.
    BaseGeodCrs,
    /// `BASEGEOGCRS` -- a geographic base CRS (the more common form).
    BaseGeogCrs,
}

/// The base geodetic or geographic CRS from which a projected CRS is derived.
///
/// WKT2 keywords: `BASEGEOGCRS` (preferred), `BASEGEODCRS`.
#[derive(Debug, Clone, PartialEq)]
pub struct BaseGeodeticCrs {
    /// Which keyword was used in the WKT.
    pub keyword: BaseGeodeticCrsKeyword,
    /// The name of the base CRS.
    pub name: String,
    /// Present only if the CRS is dynamic (has a time-varying reference frame).
    pub dynamic: Option<DynamicCrs>,
    /// The datum: either a geodetic reference frame or a datum ensemble.
    pub datum: Datum,
    /// An optional ellipsoidal coordinate system unit for the base CRS.
    pub ellipsoidal_cs_unit: Option<Unit>,
    /// Identifiers for this base CRS.
    pub identifiers: Vec<Identifier>,
}

/// Dynamic CRS attributes for a CRS with a time-varying reference frame.
///
/// In a dynamic CRS, coordinate values of a point change with time.
///
/// WKT2 keyword: `DYNAMIC`.
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicCrs {
    /// The epoch at which the reference frame is defined (e.g. 2010.0).
    pub frame_reference_epoch: f64,
    /// An optional reference to a deformation model or velocity grid.
    pub deformation_model: Option<DeformationModel>,
}

/// A reference to a deformation model or velocity grid associated with a dynamic CRS.
///
/// WKT2 keywords: `MODEL` (preferred), `VELOCITYGRID`.
#[derive(Debug, Clone, PartialEq)]
pub struct DeformationModel {
    /// The name of the deformation model.
    pub name: String,
    /// Identifiers for this deformation model.
    pub identifiers: Vec<Identifier>,
}

/// A datum is either a geodetic reference frame or a datum ensemble.
#[derive(Debug, Clone, PartialEq)]
pub enum Datum {
    /// A single geodetic reference frame (classical datum or modern terrestrial reference frame).
    ReferenceFrame(GeodeticReferenceFrame),
    /// An ensemble of reference frames that are considered approximately equivalent.
    Ensemble(DatumEnsemble),
}

/// The keyword used for a geodetic reference frame.
#[derive(Debug, Clone, PartialEq)]
pub enum DatumKeyword {
    /// `DATUM` -- the preferred keyword for backward compatibility.
    Datum,
    /// `TRF` -- terrestrial reference frame.
    Trf,
    /// `GEODETICDATUM` -- the fully spelled-out form.
    GeodeticDatum,
}

/// A geodetic reference frame (datum), defining the relationship between a coordinate
/// system and the Earth.
///
/// WKT2 keywords: `DATUM` (preferred), `TRF`, `GEODETICDATUM`.
#[derive(Debug, Clone, PartialEq)]
pub struct GeodeticReferenceFrame {
    /// Which keyword was used in the WKT.
    pub keyword: DatumKeyword,
    /// The datum name (e.g. "World Geodetic System 1984").
    pub name: String,
    /// The reference ellipsoid.
    pub ellipsoid: Ellipsoid,
    /// A textual description of the datum anchor point.
    pub anchor: Option<String>,
    /// The epoch at which a derived static reference frame is aligned to its parent
    /// dynamic frame.
    pub anchor_epoch: Option<f64>,
    /// Identifiers for this datum.
    pub identifiers: Vec<Identifier>,
    /// The prime meridian. Appears as a sibling after the DATUM node in WKT2.
    /// If absent, the international reference meridian (Greenwich) is assumed.
    pub prime_meridian: Option<PrimeMeridian>,
}

/// The prime meridian defining zero longitude.
///
/// WKT2 keywords: `PRIMEM` (preferred), `PRIMEMERIDIAN`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimeMeridian {
    /// The name of the prime meridian (e.g. "Greenwich", "Paris").
    pub name: String,
    /// The longitude of this meridian measured from the international reference meridian,
    /// positive eastward.
    pub irm_longitude: f64,
    /// The angle unit for the longitude value. If absent, the value is in the CRS's
    /// angular unit (if available), otherwise in decimal degrees.
    pub unit: Option<Unit>,
    /// Identifiers for this prime meridian.
    pub identifiers: Vec<Identifier>,
}

/// A reference ellipsoid (the mathematical figure of the Earth).
///
/// WKT2 keywords: `ELLIPSOID` (preferred), `SPHEROID` (backward compatibility).
#[derive(Debug, Clone, PartialEq)]
pub struct Ellipsoid {
    /// The ellipsoid name (e.g. "WGS 84", "GRS 1980").
    pub name: String,
    /// The semi-major axis length.
    pub semi_major_axis: f64,
    /// The inverse flattening (0 for a sphere).
    pub inverse_flattening: f64,
    /// The length unit for the semi-major axis. If absent, metres are assumed.
    pub unit: Option<Unit>,
    /// Identifiers for this ellipsoid.
    pub identifiers: Vec<Identifier>,
}

/// A datum ensemble: a collection of reference frames considered approximately equivalent.
///
/// Use of datum ensembles comes with a caveat: it will not be possible to identify which
/// member the data is most accurately referenced to.
///
/// WKT2 keyword: `ENSEMBLE`.
#[derive(Debug, Clone, PartialEq)]
pub struct DatumEnsemble {
    /// The ensemble name (e.g. "WGS 84 ensemble").
    pub name: String,
    /// The member reference frames of this ensemble.
    pub members: Vec<EnsembleMember>,
    /// The reference ellipsoid. Present for geodetic datum ensembles, absent for vertical.
    pub ellipsoid: Option<Ellipsoid>,
    /// The positional accuracy of the ensemble in metres, indicating the difference in
    /// coordinate values between members.
    pub accuracy: f64,
    /// Identifiers for this ensemble.
    pub identifiers: Vec<Identifier>,
    /// The prime meridian. Present for geodetic datum ensembles, appears as a sibling
    /// after the ENSEMBLE node.
    pub prime_meridian: Option<PrimeMeridian>,
}

/// A member of a datum ensemble.
///
/// WKT2 keyword: `MEMBER`.
#[derive(Debug, Clone, PartialEq)]
pub struct EnsembleMember {
    /// The member name (e.g. "WGS 84 (G730)").
    pub name: String,
    /// Identifiers for this member.
    pub identifiers: Vec<Identifier>,
}

/// An external identifier referencing an authority's definition of an object.
///
/// WKT2 keyword: `ID`.
#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    /// The name of the authority (e.g. "EPSG").
    pub authority_name: String,
    /// The unique identifier within the authority (e.g. 4326 or "Abcd_Ef").
    pub authority_unique_id: AuthorityId,
    /// The version of the cited object or repository.
    pub version: Option<AuthorityId>,
    /// A citation giving further details of the authority.
    pub citation: Option<String>,
    /// A URI referencing an online resource.
    pub uri: Option<String>,
}

/// An authority identifier value, which can be either numeric or textual.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthorityId {
    /// A numeric identifier (e.g. `4326`).
    Number(f64),
    /// A textual identifier (e.g. `"Abcd_Ef"`).
    Text(String),
}

/// The keyword used for a unit.
#[derive(Debug, Clone, PartialEq)]
pub enum UnitKeyword {
    /// `ANGLEUNIT` -- for angular measurements.
    AngleUnit,
    /// `LENGTHUNIT` -- for linear measurements.
    LengthUnit,
    /// `PARAMETRICUNIT` -- for parametric values (e.g. pressure).
    ParametricUnit,
    /// `SCALEUNIT` -- for dimensionless scale factors.
    ScaleUnit,
    /// `TIMEUNIT` (or `TEMPORALQUANTITY`) -- for temporal measurements.
    TimeUnit,
    /// `UNIT` -- the generic backward-compatible keyword.
    Unit,
}

/// A scope-extent pairing describing the applicability of a CRS or coordinate operation.
///
/// WKT2 keyword: `USAGE`.
#[derive(Debug, Clone, PartialEq)]
pub struct Usage {
    /// A textual description of the scope (purpose) of the CRS.
    pub scope: String,
    /// A textual description of the geographic area of applicability.
    pub area: Option<String>,
    /// A geographic bounding box describing the area of applicability.
    pub bbox: Option<BBox>,
    /// A vertical height range of applicability.
    pub vertical_extent: Option<VerticalExtent>,
    /// A temporal range of applicability.
    pub temporal_extent: Option<TemporalExtent>,
}

/// A geographic bounding box in decimal degrees relative to the international reference meridian.
///
/// WKT2 keyword: `BBOX`.
#[derive(Debug, Clone, PartialEq)]
pub struct BBox {
    /// The southern latitude boundary (-90 to +90).
    pub lower_left_latitude: f64,
    /// The western longitude boundary (-180 to +180).
    pub lower_left_longitude: f64,
    /// The northern latitude boundary (-90 to +90).
    pub upper_right_latitude: f64,
    /// The eastern longitude boundary (-180 to +180). May be less than
    /// `lower_left_longitude` when the area crosses the 180 degree meridian.
    pub upper_right_longitude: f64,
}

/// A vertical height range of applicability. Depths have negative height values.
///
/// WKT2 keyword: `VERTICALEXTENT`.
#[derive(Debug, Clone, PartialEq)]
pub struct VerticalExtent {
    /// The minimum height (most negative = deepest).
    pub minimum_height: f64,
    /// The maximum height.
    pub maximum_height: f64,
    /// The unit for the height values. If absent, metres are assumed.
    pub unit: Option<Unit>,
}

/// A temporal range of applicability.
///
/// WKT2 keyword: `TIMEEXTENT`.
#[derive(Debug, Clone, PartialEq)]
pub struct TemporalExtent {
    /// The start of the temporal range. Either a date/time string (e.g. "2013-01-01")
    /// or descriptive text (e.g. "Jurassic").
    pub start: String,
    /// The end of the temporal range.
    pub end: String,
}

/// A unit of measurement with an optional conversion factor to the SI base unit.
///
/// WKT2 keywords: `ANGLEUNIT`, `LENGTHUNIT`, `SCALEUNIT`, `PARAMETRICUNIT`,
/// `TIMEUNIT`, `TEMPORALQUANTITY`, `UNIT`.
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    /// Which keyword was used in the WKT.
    pub keyword: UnitKeyword,
    /// The unit name (e.g. "metre", "degree").
    pub name: String,
    /// The conversion factor to the SI base unit (metres for length, radians for angle,
    /// seconds for time, unity for scale). Optional only for `TIMEUNIT` when the
    /// conversion is not a simple scaling (e.g. "calendar month").
    pub conversion_factor: Option<f64>,
    /// Identifiers for this unit.
    pub identifiers: Vec<Identifier>,
}
