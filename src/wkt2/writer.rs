//! `Display` implementations for emitting WKT2 text from parsed types.
//!
//! All types use the preferred WKT2 keywords (e.g. `PROJCRS` not `PROJECTEDCRS`,
//! `ELLIPSOID` not `SPHEROID`, `METHOD` not `PROJECTION`).

use std::fmt::{self, Display, Formatter, Write};

use crate::crs::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write a comma-separated sequence of items, each preceded by `,`.
fn write_comma_items<T: Display>(f: &mut Formatter<'_>, items: &[T]) -> fmt::Result {
    for item in items {
        write!(f, ",{item}")?;
    }
    Ok(())
}

/// Write a quoted string.
fn write_quoted(f: &mut Formatter<'_>, s: &str) -> fmt::Result {
    write!(f, "\"{s}\"")
}

// ---------------------------------------------------------------------------
// Top-level CRS
// ---------------------------------------------------------------------------

impl Display for Crs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Crs::ProjectedCrs(crs) => crs.fmt(f),
            Crs::GeogCrs(crs) => crs.fmt(f),
            Crs::GeodCrs(crs) => crs.fmt(f),
            Crs::VertCrs(crs) => crs.fmt(f),
            Crs::CompoundCrs(crs) => crs.fmt(f),
        }
    }
}

impl Display for GeogCrsKeyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            GeogCrsKeyword::GeogCrs => "GEOGCRS",
            GeogCrsKeyword::GeographicCrs => "GEOGRAPHICCRS",
        })
    }
}

impl Display for GeogCrs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}[", self.keyword)?;
        write_quoted(f, &self.name)?;

        if let Some(ref dynamic) = self.dynamic {
            write!(f, ",{dynamic}")?;
        }

        match &self.datum {
            Datum::ReferenceFrame(rf) => {
                write!(f, ",{rf}")?;
                if let Some(ref pm) = rf.prime_meridian {
                    write!(f, ",{pm}")?;
                }
            }
            Datum::Ensemble(ens) => {
                write!(f, ",{ens}")?;
                if let Some(ref pm) = ens.prime_meridian {
                    write!(f, ",{pm}")?;
                }
            }
        }

        write!(f, ",{}", self.coordinate_system)?;
        write_comma_items(f, &self.usages)?;
        write_comma_items(f, &self.identifiers)?;
        if let Some(ref remark) = self.remark {
            f.write_str(",REMARK[")?;
            write_quoted(f, remark)?;
            f.write_char(']')?;
        }
        f.write_char(']')
    }
}

impl Display for GeodCrsKeyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            GeodCrsKeyword::GeodCrs => "GEODCRS",
            GeodCrsKeyword::GeodeticCrs => "GEODETICCRS",
        })
    }
}

impl Display for GeodCrs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}[", self.keyword)?;
        write_quoted(f, &self.name)?;

        if let Some(ref dynamic) = self.dynamic {
            write!(f, ",{dynamic}")?;
        }

        match &self.datum {
            Datum::ReferenceFrame(rf) => {
                write!(f, ",{rf}")?;
                if let Some(ref pm) = rf.prime_meridian {
                    write!(f, ",{pm}")?;
                }
            }
            Datum::Ensemble(ens) => {
                write!(f, ",{ens}")?;
                if let Some(ref pm) = ens.prime_meridian {
                    write!(f, ",{pm}")?;
                }
            }
        }

        write!(f, ",{}", self.coordinate_system)?;
        write_comma_items(f, &self.usages)?;
        write_comma_items(f, &self.identifiers)?;
        if let Some(ref remark) = self.remark {
            f.write_str(",REMARK[")?;
            write_quoted(f, remark)?;
            f.write_char(']')?;
        }
        f.write_char(']')
    }
}

impl Display for VertCrsKeyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            VertCrsKeyword::VertCrs => "VERTCRS",
            VertCrsKeyword::VerticalCrs => "VERTICALCRS",
        })
    }
}

impl Display for VertCrs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}[", self.keyword)?;
        write_quoted(f, &self.name)?;

        match &self.source {
            VertCrsSource::Datum { dynamic, datum } => {
                if let Some(dynamic) = dynamic {
                    write!(f, ",{dynamic}")?;
                }
                match datum {
                    VerticalDatum::ReferenceFrame(rf) => write!(f, ",{rf}")?,
                    VerticalDatum::Ensemble(ens) => write!(f, ",{ens}")?,
                }
            }
            VertCrsSource::Derived {
                base_vert_crs,
                deriving_conversion,
            } => {
                write!(f, ",{base_vert_crs}")?;
                f.write_str(",DERIVINGCONVERSION[")?;
                write_quoted(f, &deriving_conversion.name)?;
                write!(f, ",{}", deriving_conversion.method)?;
                write_comma_items(f, &deriving_conversion.parameters)?;
                write_comma_items(f, &deriving_conversion.identifiers)?;
                f.write_char(']')?;
            }
        }

        write!(f, ",{}", self.coordinate_system)?;
        write_comma_items(f, &self.geoid_models)?;
        write_comma_items(f, &self.usages)?;
        write_comma_items(f, &self.identifiers)?;
        if let Some(ref remark) = self.remark {
            f.write_str(",REMARK[")?;
            write_quoted(f, remark)?;
            f.write_char(']')?;
        }
        f.write_char(']')
    }
}

impl Display for BaseVertCrs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("BASEVERTCRS[")?;
        write_quoted(f, &self.name)?;

        if let Some(ref dynamic) = self.dynamic {
            write!(f, ",{dynamic}")?;
        }

        match &self.datum {
            VerticalDatum::ReferenceFrame(rf) => write!(f, ",{rf}")?,
            VerticalDatum::Ensemble(ens) => write!(f, ",{ens}")?,
        }

        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')
    }
}

impl Display for VerticalReferenceFrameKeyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            VerticalReferenceFrameKeyword::VDatum => "VDATUM",
            VerticalReferenceFrameKeyword::Vrf => "VRF",
            VerticalReferenceFrameKeyword::VerticalDatum => "VERTICALDATUM",
        })
    }
}

impl Display for VerticalReferenceFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}[", self.keyword)?;
        write_quoted(f, &self.name)?;
        if let Some(ref anchor) = self.anchor {
            f.write_str(",ANCHOR[")?;
            write_quoted(f, anchor)?;
            f.write_char(']')?;
        }
        if let Some(epoch) = self.anchor_epoch {
            write!(f, ",ANCHOREPOCH[{epoch}]")?;
        }
        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')
    }
}

impl Display for GeoidModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("GEOIDMODEL[")?;
        write_quoted(f, &self.name)?;
        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')
    }
}

impl Display for CompoundCrs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("COMPOUNDCRS[")?;
        write_quoted(f, &self.name)?;
        for component in &self.components {
            write!(f, ",{component}")?;
        }
        write_comma_items(f, &self.usages)?;
        write_comma_items(f, &self.identifiers)?;
        if let Some(ref remark) = self.remark {
            f.write_str(",REMARK[")?;
            write_quoted(f, remark)?;
            f.write_char(']')?;
        }
        f.write_char(']')
    }
}

impl Display for SingleCrs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SingleCrs::ProjectedCrs(crs) => crs.fmt(f),
            SingleCrs::GeogCrs(crs) => crs.fmt(f),
            SingleCrs::GeodCrs(crs) => crs.fmt(f),
            SingleCrs::VertCrs(crs) => crs.fmt(f),
            SingleCrs::Other(raw) => f.write_str(raw),
        }
    }
}

impl Display for ProjectedCrs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("PROJCRS[")?;
        write_quoted(f, &self.name)?;
        write!(f, ",{}", self.base_geodetic_crs)?;
        write!(f, ",{}", self.map_projection)?;
        write!(f, ",{}", self.coordinate_system)?;
        write_comma_items(f, &self.usages)?;
        write_comma_items(f, &self.identifiers)?;
        if let Some(ref remark) = self.remark {
            f.write_str(",REMARK[")?;
            write_quoted(f, remark)?;
            f.write_char(']')?;
        }
        f.write_char(']')
    }
}

// ---------------------------------------------------------------------------
// Base geodetic CRS
// ---------------------------------------------------------------------------

impl Display for BaseGeodeticCrsKeyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            BaseGeodeticCrsKeyword::BaseGeodCrs => "BASEGEODCRS",
            BaseGeodeticCrsKeyword::BaseGeogCrs => "BASEGEOGCRS",
        })
    }
}

impl Display for BaseGeodeticCrs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}[", self.keyword)?;
        write_quoted(f, &self.name)?;

        if let Some(ref dynamic) = self.dynamic {
            write!(f, ",{dynamic}")?;
        }

        match &self.datum {
            Datum::ReferenceFrame(rf) => {
                write!(f, ",{rf}")?;
                if let Some(ref pm) = rf.prime_meridian {
                    write!(f, ",{pm}")?;
                }
            }
            Datum::Ensemble(ens) => {
                write!(f, ",{ens}")?;
                if let Some(ref pm) = ens.prime_meridian {
                    write!(f, ",{pm}")?;
                }
            }
        }

        if let Some(ref unit) = self.ellipsoidal_cs_unit {
            write!(f, ",{unit}")?;
        }
        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')
    }
}

// ---------------------------------------------------------------------------
// Dynamic CRS
// ---------------------------------------------------------------------------

impl Display for DynamicCrs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DYNAMIC[FRAMEEPOCH[{}]", self.frame_reference_epoch)?;
        if let Some(ref model) = self.deformation_model {
            write!(f, ",{model}")?;
        }
        f.write_char(']')
    }
}

impl Display for DeformationModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("MODEL[")?;
        write_quoted(f, &self.name)?;
        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')
    }
}

// ---------------------------------------------------------------------------
// Datum
// ---------------------------------------------------------------------------

impl Display for DatumKeyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            DatumKeyword::Datum => "DATUM",
            DatumKeyword::Trf => "TRF",
            DatumKeyword::GeodeticDatum => "GEODETICDATUM",
        })
    }
}

impl Display for GeodeticReferenceFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}[", self.keyword)?;
        write_quoted(f, &self.name)?;
        write!(f, ",{}", self.ellipsoid)?;
        if let Some(ref anchor) = self.anchor {
            f.write_str(",ANCHOR[")?;
            write_quoted(f, anchor)?;
            f.write_char(']')?;
        }
        if let Some(epoch) = self.anchor_epoch {
            write!(f, ",ANCHOREPOCH[{epoch}]")?;
        }
        write_comma_items(f, &self.identifiers)?;
        // prime_meridian is written by the parent (BaseGeodeticCrs) as a sibling
        f.write_char(']')
    }
}

impl Display for DatumEnsemble {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("ENSEMBLE[")?;
        write_quoted(f, &self.name)?;
        write_comma_items(f, &self.members)?;
        if let Some(ref ellipsoid) = self.ellipsoid {
            write!(f, ",{ellipsoid}")?;
        }
        write!(f, ",ENSEMBLEACCURACY[{}]", self.accuracy)?;
        write_comma_items(f, &self.identifiers)?;
        // prime_meridian is written by the parent (BaseGeodeticCrs) as a sibling
        f.write_char(']')
    }
}

impl Display for EnsembleMember {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("MEMBER[")?;
        write_quoted(f, &self.name)?;
        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')
    }
}

// ---------------------------------------------------------------------------
// Ellipsoid & Prime Meridian
// ---------------------------------------------------------------------------

impl Display for Ellipsoid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("ELLIPSOID[")?;
        write_quoted(f, &self.name)?;
        write!(f, ",{},{}", self.semi_major_axis, self.inverse_flattening)?;
        if let Some(ref unit) = self.unit {
            write!(f, ",{unit}")?;
        }
        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')
    }
}

impl Display for PrimeMeridian {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("PRIMEM[")?;
        write_quoted(f, &self.name)?;
        write!(f, ",{}", self.irm_longitude)?;
        if let Some(ref unit) = self.unit {
            write!(f, ",{unit}")?;
        }
        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')
    }
}

// ---------------------------------------------------------------------------
// Map Projection
// ---------------------------------------------------------------------------

impl Display for MapProjection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("CONVERSION[")?;
        write_quoted(f, &self.name)?;
        write!(f, ",{}", self.method)?;
        write_comma_items(f, &self.parameters)?;
        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')
    }
}

impl Display for MapProjectionMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("METHOD[")?;
        write_quoted(f, &self.name)?;
        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')
    }
}

impl Display for MapProjectionParameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("PARAMETER[")?;
        write_quoted(f, &self.name)?;
        write!(f, ",{}", self.value)?;
        if let Some(ref unit) = self.unit {
            write!(f, ",{unit}")?;
        }
        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')
    }
}

// ---------------------------------------------------------------------------
// Coordinate System
// ---------------------------------------------------------------------------

impl Display for CoordinateSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "CS[{},{}", self.cs_type, self.dimension)?;
        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')?;
        // axes and cs_unit are siblings after CS[...]
        write_comma_items(f, &self.axes)?;
        if let Some(ref unit) = self.cs_unit {
            write!(f, ",{unit}")?;
        }
        Ok(())
    }
}

impl Display for CsType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CsType::Affine => "affine",
            CsType::Cartesian => "Cartesian",
            CsType::Cylindrical => "cylindrical",
            CsType::Ellipsoidal => "ellipsoidal",
            CsType::Linear => "linear",
            CsType::Parametric => "parametric",
            CsType::Polar => "polar",
            CsType::Spherical => "spherical",
            CsType::Vertical => "vertical",
            CsType::TemporalCount => "temporalCount",
            CsType::TemporalMeasure => "temporalMeasure",
            CsType::Ordinal => "ordinal",
            CsType::TemporalDateTime => "temporalDateTime",
        })
    }
}

impl Display for Axis {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("AXIS[")?;
        write_quoted(f, &self.name_abbrev)?;
        write!(f, ",{}", self.direction)?;
        if let Some(ref meridian) = self.meridian {
            write!(f, ",{meridian}")?;
        }
        if let Some(bearing) = self.bearing {
            write!(f, ",BEARING[{bearing}]")?;
        }
        if let Some(order) = self.order {
            write!(f, ",ORDER[{order}]")?;
        }
        if let Some(ref unit) = self.unit {
            write!(f, ",{unit}")?;
        }
        if let Some(min) = self.axis_min_value {
            write!(f, ",AXISMINVALUE[{min}]")?;
        }
        if let Some(max) = self.axis_max_value {
            write!(f, ",AXISMAXVALUE[{max}]")?;
        }
        if let Some(rm) = self.range_meaning {
            write!(f, ",{rm}")?;
        }
        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')
    }
}

impl Display for Meridian {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "MERIDIAN[{},{}]", self.value, self.unit)
    }
}

impl Display for RangeMeaning {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            RangeMeaning::Exact => "RANGEMEANING[exact]",
            RangeMeaning::Wraparound => "RANGEMEANING[wraparound]",
        })
    }
}

// ---------------------------------------------------------------------------
// Unit
// ---------------------------------------------------------------------------

impl Display for UnitKeyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            UnitKeyword::AngleUnit => "ANGLEUNIT",
            UnitKeyword::LengthUnit => "LENGTHUNIT",
            UnitKeyword::ParametricUnit => "PARAMETRICUNIT",
            UnitKeyword::ScaleUnit => "SCALEUNIT",
            UnitKeyword::TimeUnit => "TIMEUNIT",
            UnitKeyword::Unit => "UNIT",
        })
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}[", self.keyword)?;
        write_quoted(f, &self.name)?;
        if let Some(factor) = self.conversion_factor {
            write!(f, ",{factor}")?;
        }
        write_comma_items(f, &self.identifiers)?;
        f.write_char(']')
    }
}

// ---------------------------------------------------------------------------
// Identifier
// ---------------------------------------------------------------------------

impl Display for AuthorityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AuthorityId::Number(n) => write!(f, "{n}"),
            AuthorityId::Text(s) => write_quoted(f, s),
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("ID[")?;
        write_quoted(f, &self.authority_name)?;
        write!(f, ",{}", self.authority_unique_id)?;
        if let Some(ref version) = self.version {
            write!(f, ",{version}")?;
        }
        if let Some(ref citation) = self.citation {
            f.write_str(",CITATION[")?;
            write_quoted(f, citation)?;
            f.write_char(']')?;
        }
        if let Some(ref uri) = self.uri {
            f.write_str(",URI[")?;
            write_quoted(f, uri)?;
            f.write_char(']')?;
        }
        f.write_char(']')
    }
}

// ---------------------------------------------------------------------------
// Usage & Extent
// ---------------------------------------------------------------------------

impl Display for Usage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("USAGE[SCOPE[")?;
        write_quoted(f, &self.scope)?;
        f.write_char(']')?;
        if let Some(ref area) = self.area {
            f.write_str(",AREA[")?;
            write_quoted(f, area)?;
            f.write_char(']')?;
        }
        if let Some(ref bbox) = self.bbox {
            write!(f, ",{bbox}")?;
        }
        if let Some(ref ve) = self.vertical_extent {
            write!(f, ",{ve}")?;
        }
        if let Some(ref te) = self.temporal_extent {
            write!(f, ",{te}")?;
        }
        f.write_char(']')
    }
}

impl Display for BBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BBOX[{},{},{},{}]",
            self.lower_left_latitude,
            self.lower_left_longitude,
            self.upper_right_latitude,
            self.upper_right_longitude
        )
    }
}

impl Display for VerticalExtent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VERTICALEXTENT[{},{}",
            self.minimum_height, self.maximum_height
        )?;
        if let Some(ref unit) = self.unit {
            write!(f, ",{unit}")?;
        }
        f.write_char(']')
    }
}

impl Display for TemporalExtent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // If the values look like dates (contain digits/hyphens), write unquoted;
        // otherwise write quoted.
        write!(f, "TIMEEXTENT[")?;
        write_temporal_value(f, &self.start)?;
        f.write_char(',')?;
        write_temporal_value(f, &self.end)?;
        f.write_char(']')
    }
}

fn write_temporal_value(f: &mut Formatter<'_>, value: &str) -> fmt::Result {
    // If it looks like a date/time (starts with a digit), write unquoted
    if value.starts_with(|c: char| c.is_ascii_digit()) {
        f.write_str(value)
    } else {
        write_quoted(f, value)
    }
}
