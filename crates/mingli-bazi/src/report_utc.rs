//! Versioned civil UTC/TT report. Ordinary civil JD is NOT SOFA UTC quasi-JD.
//! Inputs omit second 60. TT intervals retain elapsed leap seconds.
use super::*;
use crate::report_utc_data::SEGMENTS;
use mingli_astro::julian_day;

/// UTC-aware report model, separate from both legacy models.
pub const REPORT_UTC_MODEL: &str = "vsop87d-iau1980-utc-v2";
/// Fixed source-data and coverage-policy identity.
pub const UTC_TABLE_ID: &str = "erfa-2.0.1-iers-c72-utc-v2";
/// Typed failures are returned by the API as client errors, never clamped roots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UtcReportError {
    /// Missing gender, invalid civil input or nonfinite coordinate.
    InvalidInput,
    /// Birth or required root lies outside the published-data coverage.
    UnsupportedCoverage,
    /// TT falls into a leap/offset gap not expressible by ordinary civil JD.
    UnrepresentableCivilInstant,
    /// Backward UTC adjustment yields multiple civil-coordinate solutions.
    AmbiguousCivilInstant,
    /// The requested target is not bracketed by the solar solver.
    UnbracketedSolarRoot,
}
impl std::fmt::Display for UtcReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidInput => "UTC报告需要有效生日、明确计算性别和钟面时间",
            Self::UnsupportedCoverage => {
                "出生或相邻交节超出已确认的时间数据范围（截至2027年6月30日）；当前不能生成UTC报告"
            }
            Self::UnrepresentableCivilInstant => "交节位于无法用普通民用时刻表示的时间调整区间",
            Self::AmbiguousCivilInstant => "交节位于历史UTC回调重叠区间，无法唯一确定民用时刻",
            Self::UnbracketedSolarRoot => "未能确认太阳交节区间",
        })
    }
}
impl std::error::Error for UtcReportError {}
/// Per-instant conversion evidence.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct UtcTimeScale {
    /// historical_ut_approx / utc_drift_table / utc_leap_table.
    pub kind: &'static str,
    /// Applied TT minus ordinary civil-coordinate seconds.
    pub tt_minus_civil_seconds: f64,
    /// Concrete offset segment, or historical approximation identifier.
    pub source_segment: String,
}
/// Civil-coordinate instant and corresponding TT instant.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct UtcInstant {
    /// Uniform 86400-coordinate-seconds calendar JD; not UTC quasi-JD.
    pub jd_civil: f64,
    /// TT Julian ephemeris day.
    pub jde_tt: f64,
    /// Per-instant table classification and offset.
    pub time_scale: UtcTimeScale,
}
fn coverage() -> (f64, f64) {
    (julian_day(1899, 1, 1.0), julian_day(2027, 7, 1.0))
}
/// Convert a covered ordinary civil JD, rejecting dates beyond the fixed bulletin policy.
pub fn report_utc_instant(jd: f64) -> Result<UtcInstant, UtcReportError> {
    let (start, end) = coverage();
    if !jd.is_finite() {
        return Err(UtcReportError::InvalidInput);
    }
    if jd < start || jd >= end {
        return Err(UtcReportError::UnsupportedCoverage);
    }
    if jd < julian_day(1960, 1, 1.0) {
        let tt = report_jd_ut_to_jde(jd);
        return Ok(UtcInstant {
            jd_civil: jd,
            jde_tt: tt,
            time_scale: UtcTimeScale {
                kind: "historical_ut_approx",
                tt_minus_civil_seconds: (tt - jd) * 86400.0,
                source_segment: "espenak-meeus-historical-ut".into(),
            },
        });
    }
    let &(y, m, base, ref_mjd, rate) = SEGMENTS
        .iter()
        .rev()
        .find(|(y, m, _, _, _)| jd >= julian_day(*y, *m, 1.0))
        .ok_or(UtcReportError::UnsupportedCoverage)?;
    let seconds = base + (jd - 2400000.5 - ref_mjd) * rate + 32.184;
    Ok(UtcInstant {
        jd_civil: jd,
        jde_tt: jd + seconds / 86400.0,
        time_scale: UtcTimeScale {
            kind: if y < 1972 {
                "utc_drift_table"
            } else {
                "utc_leap_table"
            },
            tt_minus_civil_seconds: seconds,
            source_segment: format!("{y:04}-{m:02}-01"),
        },
    })
}
/// Invert TT over explicit UTC segments. Leap-second gaps and historical backward
/// steps have zero or multiple ordinary-civil solutions and receive typed errors.
pub fn report_tt_to_civil(tt: f64) -> Result<UtcInstant, UtcReportError> {
    if !tt.is_finite() {
        return Err(UtcReportError::InvalidInput);
    }
    let (start, end) = coverage();
    let mut found = Vec::new();
    // Historical model has no leap steps in this interval.
    let mut lo = start;
    let mut hi = julian_day(1960, 1, 1.0);
    if tt >= report_jd_ut_to_jde(lo) && tt < report_jd_ut_to_jde(hi) {
        for _ in 0..48 {
            let mid = (lo + hi) * 0.5;
            if mid == lo || mid == hi {
                break;
            }
            if report_jd_ut_to_jde(mid) >= tt {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let x = (lo + hi) * 0.5;
        if x < julian_day(1960, 1, 1.0) {
            found.push(report_utc_instant(x)?);
        }
    }
    for (n, &(y, m, base, ref_mjd, rate)) in SEGMENTS.iter().enumerate() {
        let a = julian_day(y, m, 1.0);
        let b = SEGMENTS
            .get(n + 1)
            .map_or(end, |(y, m, _, _, _)| julian_day(*y, *m, 1.0));
        // Select the mathematical TT half-open interval before floating-point
        // inversion, so rounding near a continuous step cannot admit an old segment.
        let segment_tt =
            |civil: f64| civil + (base + (civil - 2400000.5 - ref_mjd) * rate + 32.184) / 86400.0;
        let a_tt = segment_tt(a);
        if tt < a_tt || tt >= segment_tt(b) {
            continue;
        }
        let reference = if rate == 0.0 { a } else { 2400000.5 + ref_mjd };
        let ref_tt =
            reference + (base + (reference - 2400000.5 - ref_mjd) * rate + 32.184) / 86400.0;
        let mut civil = reference + (tt - ref_tt) / (1.0 + rate / 86400.0);
        // Algebraic inversion around a distant drift reference can lose one JD
        // ULP at an exact effective midnight. Anchor only exact forward equality;
        // do not clamp arbitrary gap instants into a neighboring segment.
        if tt == a_tt {
            civil = a;
        }
        if civil >= a && civil < b {
            let instant = report_utc_instant(civil)?;
            if (instant.jde_tt - tt).abs() < 1e-8 {
                found.push(instant);
            }
        }
    }
    match found.len() {
        1 => Ok(found.remove(0)),
        n if n > 1 => Err(UtcReportError::AmbiguousCivilInstant),
        _ => {
            let upper = end + 69.184 / 86400.0; // known left-hand limit; does not predict the July leap
            if tt < report_jd_ut_to_jde(start) || tt >= upper {
                Err(UtcReportError::UnsupportedCoverage)
            } else {
                Err(UtcReportError::UnrepresentableCivilInstant)
            }
        }
    }
}
/// Solve solely in TT; trial guesses are not restricted by UTC table coverage.
pub fn report_utc_solar_term_tt_near(guess: f64, target: f64) -> Result<f64, UtcReportError> {
    if !guess.is_finite() || !target.is_finite() {
        return Err(UtcReportError::InvalidInput);
    }
    let residual = |tt| (report_solar_longitude(tt) - target + 180.0).rem_euclid(360.0) - 180.0;
    let (mut lo, mut hi) = (guess - 8.0, guess + 8.0);
    if residual(lo) >= 0.0 || residual(hi) <= 0.0 {
        return Err(UtcReportError::UnbracketedSolarRoot);
    }
    for _ in 0..48 {
        let mid = (lo + hi) * 0.5;
        if mid == lo || mid == hi {
            break;
        }
        if residual(mid) >= 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Ok((lo + hi) * 0.5)
}
/// TT solar-term root seeded in the Gregorian year, without assuming future UTC.
pub fn report_utc_solar_term_tt(year: i32, target: f64) -> Result<f64, UtcReportError> {
    let jan1 = julian_day(year, 1, 1.0);
    report_utc_solar_term_tt_near(
        jan1 + (target - report_solar_longitude(jan1)).rem_euclid(360.0) / 0.98565,
        target,
    )
}
/// Versioned, immutable data provenance and explicit coverage decision.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct UtcPolicy {
    /// Fixed identity for numeric table plus coverage policy.
    pub table_id: &'static str,
    /// SHA256 of the retained ERFA v2.0.1 dat.c.
    pub erfa_dat_sha256: &'static str,
    /// SHA256 of the retained IERS leap list.
    pub leap_list_sha256: &'static str,
    /// SHA256 of the retained C72 announcement.
    pub bulletin_sha256: &'static str,
    /// Announcement date, not offset effective date.
    pub bulletin_published: &'static str,
    /// Most recent TAI-UTC step's effective date.
    pub last_tai_utc_effective: &'static str,
    /// File maintenance expiration, not a leap-second announcement.
    pub leap_list_expires: &'static str,
    /// Lower bound for adjacent roots of supported births.
    pub coverage_start: &'static str,
    /// Exclusive civil-coordinate coverage end.
    pub coverage_end_exclusive: &'static str,
    /// Why the exclusive end follows from the announcement.
    pub coverage_basis: &'static str,
}
/// Fixed policy; expiry, publication and offset effective dates are separate.
#[must_use]
pub fn report_utc_policy() -> UtcPolicy {
    UtcPolicy{
    table_id:UTC_TABLE_ID,
    erfa_dat_sha256:"09e3377ff0c770372c759dc16af12aadc70975d3c549d9524e5c3dd44e24fa88",
    leap_list_sha256:"db5a895f16853b03bfc865e8d68f9fc8710ef1740e3400c701cd46a5bbbc3433",
    bulletin_sha256:"310e172eadacacca3adf92cb9bd646fcc0d6320e35e996a106d2644ede3b52f0",
    bulletin_published:"2026-07-06",last_tai_utc_effective:"2017-01-01",leap_list_expires:"2027-06-28",
    coverage_start:"1899-01-01",coverage_end_exclusive:"2027-07-01",
    coverage_basis:"IERS-C72:no-leap-end-2026-12;possible-only-June-or-December;exclusive-end-inferred-before-next-possible-adjustment",
}
}
/// One root and its TT/civil correspondence.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct UtcJieBasis {
    /// Normalized target longitude.
    pub target_longitude_deg: f64,
    /// Ordinary civil JD (not quasi-JD).
    pub jd_civil: f64,
    /// Solved TT root.
    pub jde_tt: f64,
    /// Applied conversion classification and offset.
    pub time_scale: UtcTimeScale,
}
fn jie(target: f64, tt: f64) -> Result<UtcJieBasis, UtcReportError> {
    let civil = report_tt_to_civil(tt)?;
    Ok(UtcJieBasis {
        target_longitude_deg: target.rem_euclid(360.0),
        jd_civil: civil.jd_civil,
        jde_tt: tt,
        time_scale: civil.time_scale,
    })
}
/// UTC-aware cycle evidence, intentionally not serialized as the v1 basis shape.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct UtcCycleBasis {
    /// Always 2.
    pub schema_version: u32,
    /// Independent model identity.
    pub model_id: &'static str,
    /// Frozen source hashes and coverage.
    pub time_scale_policy: UtcPolicy,
    /// Uniform-coordinate civil birth JD.
    pub birth_jd_civil: f64,
    /// Birth TT.
    pub birth_jde_tt: f64,
    /// Birth apparent longitude.
    pub birth_longitude_deg: f64,
    /// Birth conversion evidence.
    pub birth_time_scale: UtcTimeScale,
    /// Li Chun root used to select the year pillar.
    pub li_chun: UtcJieBasis,
    /// Adjacent previous jie.
    pub previous_jie: UtcJieBasis,
    /// Adjacent next jie.
    pub next_jie: UtcJieBasis,
    /// previous or next.
    pub selected: &'static str,
    /// Always tt_elapsed; never ordinary civil-JD subtraction.
    pub interval_scale: &'static str,
    /// Nonnegative TT elapsed days.
    pub interval_days: f64,
    /// Full-precision cycle age before public rounding.
    pub unrounded_start_age_years: f64,
    /// Three physical elapsed days per traditional year.
    pub days_per_year: u32,
}
/// V2 chart and UTC-aware evidence.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct BaziUtcReport {
    /// Existing chart wire shape, using the v2 model consistently.
    pub chart: BaziChart,
    /// Separate v2 evidence shape.
    pub cycle_basis: UtcCycleBasis,
}
/// Default-school UTC/historical-UT report with bounded published time data.
pub fn compute_report_utc(input: BirthInput) -> Result<BaziUtcReport, UtcReportError> {
    let leap = input.year % 4 == 0 && (input.year % 100 != 0 || input.year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if input.gender.is_none()
        || !(1900..=2100).contains(&input.year)
        || !(1..=12).contains(&input.month)
        || input.day < 1
        || input.day > month_days[(input.month - 1) as usize]
        || input.hour > 23
        || input.minute > 59
        || !input.tz.is_finite()
        || !(-12.0..=14.0).contains(&input.tz)
    {
        return Err(UtcReportError::InvalidInput);
    }
    let jd = mingli_astro::jd_from_local(
        input.year,
        input.month,
        input.day,
        input.hour,
        input.minute,
        0.0,
        input.tz,
    );
    let birth = report_utc_instant(jd)?;
    let lam = report_solar_longitude(birth.jde_tt);
    let k = ((lam - 15.0) / 30.0).floor();
    let previous = 15.0 + 30.0 * k;
    let next = previous + 30.0;
    let pt = report_utc_solar_term_tt_near(
        birth.jde_tt - (lam - previous).rem_euclid(360.0) / 0.98565,
        previous,
    )?;
    let nt = report_utc_solar_term_tt_near(
        birth.jde_tt + (next - lam).rem_euclid(360.0) / 0.98565,
        next,
    )?;
    let lt = report_utc_solar_term_tt(input.year, 315.0)?;
    // Validate coverage/invertibility BEFORE entering the infallible chart assembler.
    let previous_jie = jie(previous, pt)?;
    let next_jie = jie(next, nt)?;
    let li_chun = jie(315.0, lt)?;
    if pt > birth.jde_tt || nt < birth.jde_tt {
        return Err(UtcReportError::UnbracketedSolarRoot);
    }
    let mut m = Moment::new(
        input.year,
        input.month,
        input.day,
        input.hour,
        input.minute,
        input.tz,
    );
    m.jde = birth.jde_tt;
    m.sun_longitude = lam;
    let model = crate::report::SolarModel::PreparedUtc {
        li_chun_tt: lt,
        previous_tt: pt,
        next_tt: nt,
        next_target: next.rem_euclid(360.0),
    };
    let (chart, b) = crate::chart::compute_at_model(&m, input.gender, BaziSchool::default(), model);
    let b = b.ok_or(UtcReportError::InvalidInput)?;
    Ok(BaziUtcReport {
        chart,
        cycle_basis: UtcCycleBasis {
            schema_version: 2,
            model_id: REPORT_UTC_MODEL,
            time_scale_policy: report_utc_policy(),
            birth_jd_civil: jd,
            birth_jde_tt: birth.jde_tt,
            birth_longitude_deg: lam,
            birth_time_scale: birth.time_scale,
            li_chun,
            previous_jie,
            next_jie,
            selected: b.selected,
            interval_scale: "tt_elapsed",
            interval_days: b.interval_days,
            unrounded_start_age_years: b.unrounded_start_age_years,
            days_per_year: 3,
        },
    })
}
