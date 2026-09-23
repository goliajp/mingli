//! Clock-time report evidence from the very same computation as its chart.
use super::*;

/// A neighboring solar jie, expressed in UT and normalized apparent longitude.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct JieBasis {
    /// Target apparent solar longitude in [0,360).
    pub target_longitude_deg: f64,
    /// The solved instant in UT Julian days.
    pub jd_ut: f64,
}
/// Exact intermediate values used to obtain the published, rounded cycle ages.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct CycleBasis {
    /// Wire schema version.
    pub schema_version: u32,
    /// Solar-position and time-scale model identifier, not a claim of exact astronomy.
    pub model_id: &'static str,
    /// Birth instant in UT Julian days.
    pub birth_jd_ut: f64,
    /// Same birth instant in TT Julian days using the engine's delta-T model.
    pub birth_jde_tt: f64,
    /// Apparent solar longitude at birth.
    pub birth_longitude_deg: f64,
    /// Previous jie used by reverse cycles.
    pub previous_jie: JieBasis,
    /// Next jie used by forward cycles.
    pub next_jie: JieBasis,
    /// `previous` or `next`, selected by the chart's direction.
    pub selected: &'static str,
    /// Nonnegative birth-to-selected-jie interval.
    pub interval_days: f64,
    /// Interval divided by three, before either public rounding operation.
    pub unrounded_start_age_years: f64,
    /// Traditional conversion: three days per age year.
    pub days_per_year: u32,
}
/// Existing chart shape plus separate calculation evidence; no fields added to BaziChart.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct BaziReport {
    /// Clock-time chart whose solar model is identified by `cycle_basis.model_id`.
    pub chart: BaziChart,
    /// Basis from that chart's cycle calculation.
    pub cycle_basis: CycleBasis,
}
/// Compute a default-school clock-time chart and its cycle evidence.
/// Returns None when no calculation gender was supplied. Input range validation
/// belongs to the app boundary, as it does for the existing `compute` function.
#[must_use]
pub fn compute_report(input: BirthInput) -> Option<BaziReport> {
    input.gender?;
    let m = Moment::new(
        input.year,
        input.month,
        input.day,
        input.hour,
        input.minute,
        input.tz,
    );
    let (chart, basis) =
        crate::chart::compute_at_with_evidence(&m, input.gender, BaziSchool::default());
    Some(BaziReport {
        chart,
        cycle_basis: basis?,
    })
}

#[derive(Clone, Copy)]
pub(crate) enum SolarModel {
    Meeus,
    #[cfg(feature = "report-vsop")]
    Vsop,
    #[cfg(feature = "report-vsop")]
    PreparedUtc {
        li_chun_tt: f64,
        previous_tt: f64,
        next_tt: f64,
        next_target: f64,
    },
}
impl SolarModel {
    pub(crate) fn coordinate(self, civil: f64, tt: f64) -> f64 {
        match self {
            #[cfg(feature = "report-vsop")]
            Self::PreparedUtc { .. } => tt,
            _ => {
                let _ = tt;
                civil
            }
        }
    }
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Meeus => "meeus25-delta-t-v1",
            #[cfg(feature = "report-vsop")]
            Self::Vsop => REPORT_SOLAR_MODEL,
            #[cfg(feature = "report-vsop")]
            Self::PreparedUtc { .. } => REPORT_UTC_MODEL,
        }
    }
    pub(crate) fn term(self, year: i32, target: f64) -> f64 {
        match self {
            Self::Meeus => solar_term_jd(year, target),
            #[cfg(feature = "report-vsop")]
            Self::Vsop => report_solar_term_jd(year, target),
            #[cfg(feature = "report-vsop")]
            Self::PreparedUtc { li_chun_tt, .. } => li_chun_tt,
        }
    }
    #[allow(
        clippy::float_cmp,
        reason = "targets are whole multiples of 15 degrees, exact in f64; picks the prepared root"
    )]
    pub(crate) fn near(self, jd: f64, target: f64) -> f64 {
        match self {
            Self::Meeus => solar_term_time_near(jd, target),
            #[cfg(feature = "report-vsop")]
            Self::Vsop => report_solar_term_time_near(jd, target),
            #[cfg(feature = "report-vsop")]
            Self::PreparedUtc {
                previous_tt,
                next_tt,
                next_target,
                ..
            } => {
                if target.rem_euclid(360.0) == next_target {
                    next_tt
                } else {
                    previous_tt
                }
            }
        }
    }
}
/// High-order clock-time report: year, month and cycle roots use one solar model.
/// The civil date/day/hour and lunar-date conventions remain the existing ones.
#[cfg(feature = "report-vsop")]
#[must_use]
pub fn compute_report_vsop(input: BirthInput) -> Option<BaziReport> {
    input.gender?;
    let mut m = Moment::new(
        input.year,
        input.month,
        input.day,
        input.hour,
        input.minute,
        input.tz,
    );
    m.jde = report_jd_ut_to_jde(m.jd_ut);
    m.sun_longitude = report_solar_longitude(m.jde);
    let (chart, basis) =
        crate::chart::compute_at_model(&m, input.gender, BaziSchool::default(), SolarModel::Vsop);
    Some(BaziReport {
        chart,
        cycle_basis: basis?,
    })
}
