//! Recorded-minute alternatives. Representative seconds are assumptions, never new birth data.
use super::*;

/// Half-open interval in both ordinary civil coordinates and physical TT.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct UtcMinuteInterval {
    /// Included lower endpoint.
    pub start: UtcInstant,
    /// Excluded upper endpoint.
    pub end: UtcInstant,
    /// Always [start,end).
    pub interval: &'static str,
    /// Lower endpoint is included.
    pub start_inclusive: bool,
    /// Upper endpoint is excluded.
    pub end_inclusive: bool,
    /// Physical elapsed duration, including a positive leap second if present.
    pub elapsed_tt_seconds: f64,
}
/// Exact unrounded age range, with endpoint inclusion retained.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct UtcMinuteAgeRange {
    /// Lower age bound.
    pub min_years: f64,
    /// Upper age bound.
    pub max_years: f64,
    /// Reverse cycles include their lower age endpoint.
    pub min_inclusive: bool,
    /// Forward cycles include their upper age endpoint.
    pub max_inclusive: bool,
}
/// A complete chart for one homogeneous pillar interval.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct UtcMinuteCandidate {
    /// Ordered stable identifier within the recorded minute.
    pub id: String,
    /// Assumed birth interval, not a correction of the supplied birth record.
    pub interval: UtcMinuteInterval,
    /// Interior civil midpoint used for this complete representative chart.
    pub representative: UtcInstant,
    /// Complete freshly computed chart and evidence at the representative instant.
    pub report: BaziUtcReport,
    /// Cycle ages vary continuously within this interval.
    pub start_age_range: UtcMinuteAgeRange,
}
/// Independent recorded-minute contract; existing UTC report semantics are unchanged.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct BaziUtcMinuteReport {
    /// Version of this interval envelope, not of nested cycle evidence.
    pub schema_version: u32,
    /// Interval algorithm identity.
    pub model_id: &'static str,
    /// Original minute precision input, unchanged.
    pub recorded_input: BirthInput,
    /// Entire recorded minute.
    pub minute: UtcMinuteInterval,
    /// Solar boundaries strictly inside this minute.
    pub boundaries: Vec<UtcJieBasis>,
    /// Ordered alternatives covering the minute without overlap.
    pub candidates: Vec<UtcMinuteCandidate>,
}
fn interval(start: UtcInstant, end: UtcInstant) -> UtcMinuteInterval {
    UtcMinuteInterval {
        elapsed_tt_seconds: (end.jde_tt - start.jde_tt) * 86400.0,
        start,
        end,
        interval: "[start,end)",
        start_inclusive: true,
        end_inclusive: false,
    }
}
/// Compute all complete chart alternatives within one recorded local civil minute.
/// Positive leap seconds contribute physical duration; historical backward steps
/// are refused because civil intervals then do not order physical instants uniquely.
pub fn compute_report_utc_minute(input: BirthInput) -> Result<BaziUtcMinuteReport, UtcReportError> {
    let initial = compute_report_utc(input)?; // canonical validation and covered adjacent roots
    let start = report_utc_instant(initial.cycle_basis.birth_jd_civil)?;
    let end = report_utc_instant(start.jd_civil + 1.0 / 1440.0)?;
    // A backward offset adjustment at the end overlaps the previous minute in TT.
    if end.time_scale.tt_minus_civil_seconds + 1e-5 < start.time_scale.tt_minus_civil_seconds {
        return Err(UtcReportError::AmbiguousCivilInstant);
    }
    // Birth itself must also have one civil representation (historical overlap).
    report_tt_to_civil(start.jde_tt)?;
    let mut boundaries = Vec::new();
    let next = &initial.cycle_basis.next_jie;
    if next.jde_tt > start.jde_tt && next.jde_tt < end.jde_tt {
        boundaries.push(next.clone());
    }
    let mut points = vec![start.clone()];
    for root in &boundaries {
        points.push(UtcInstant {
            jd_civil: root.jd_civil,
            jde_tt: root.jde_tt,
            time_scale: root.time_scale.clone(),
        });
    }
    points.push(end.clone());
    let mut candidates = Vec::new();
    for (index, pair) in points.windows(2).enumerate() {
        let (a, b) = (&pair[0], &pair[1]);
        if a.jd_civil >= b.jd_civil || a.jde_tt >= b.jde_tt {
            return Err(UtcReportError::AmbiguousCivilInstant);
        }
        let representative = report_utc_instant((a.jd_civil + b.jd_civil) * 0.5)?;
        if representative.jde_tt <= a.jde_tt || representative.jde_tt >= b.jde_tt {
            return Err(UtcReportError::UnrepresentableCivilInstant);
        }
        report_tt_to_civil(representative.jde_tt)?;
        let report = crate::report_utc::compute_report_utc_at(input, representative.clone())?;
        let basis = &report.cycle_basis;
        let forward = basis.selected == "next";
        let root = if forward {
            basis.next_jie.jde_tt
        } else {
            basis.previous_jie.jde_tt
        };
        let (min, max) = if forward {
            ((root - b.jde_tt) / 3.0, (root - a.jde_tt) / 3.0)
        } else {
            ((a.jde_tt - root) / 3.0, (b.jde_tt - root) / 3.0)
        };
        if min < 0.0 || max < min {
            return Err(UtcReportError::UnbracketedSolarRoot);
        }
        candidates.push(UtcMinuteCandidate {
            id: if boundaries.is_empty() {
                "single"
            } else if index == 0 {
                "before"
            } else {
                "after"
            }
            .into(),
            interval: interval(a.clone(), b.clone()),
            representative,
            report,
            start_age_range: UtcMinuteAgeRange {
                min_years: min,
                max_years: max,
                min_inclusive: !forward,
                max_inclusive: forward,
            },
        });
    }
    Ok(BaziUtcMinuteReport {
        schema_version: 1,
        model_id: "vsop87d-iau1980-utc-minute-v1",
        recorded_input: input,
        minute: interval(start, end),
        boundaries,
        candidates,
    })
}

#[cfg(all(test, feature = "port"))]
mod tests {
    use super::*;
    use mingli_astro::julian_day;
    #[test]
    fn all_modern_jie_minutes_have_two_complete_consistent_alternatives() {
        let mut count = 0;
        for year in 1900..=2026 {
            for target in (15..360).step_by(30) {
                let tt = report_utc_solar_term_tt(year, target as f64).unwrap();
                let root = report_tt_to_civil(tt).unwrap();
                let minute = ((root.jd_civil + 0.5) * 1440.0).floor() as i64;
                let jd = minute.div_euclid(1440) as f64 - 0.5;
                let within = minute.rem_euclid(1440);
                let month = (1..=12)
                    .rev()
                    .find(|m| julian_day(year, *m, 1.0) <= jd)
                    .unwrap();
                let day = (jd - julian_day(year, month, 1.0)).round() as u32 + 1;
                for gender in [Gender::Male, Gender::Female] {
                    let input = BirthInput {
                        year,
                        month,
                        day,
                        hour: (within / 60) as u32,
                        minute: (within % 60) as u32,
                        tz: 0.0,
                        gender: Some(gender),
                    };
                    let r = compute_report_utc_minute(input).unwrap();
                    assert_eq!(r.candidates.len(), 2, "{year}/{target}");
                    let a = &r.candidates[0];
                    let b = &r.candidates[1];
                    assert_ne!(a.report.chart.month.ganzhi, b.report.chart.month.ganzhi);
                    assert_eq!(a.report.chart.day.ganzhi, b.report.chart.day.ganzhi);
                    assert_eq!(a.report.chart.hour.ganzhi, b.report.chart.hour.ganzhi);
                    if target == 315 {
                        assert_ne!(a.report.chart.year.ganzhi, b.report.chart.year.ganzhi);
                        assert_ne!(a.report.cycle_basis.selected, b.report.cycle_basis.selected);
                    } else {
                        assert_eq!(a.report.chart.year.ganzhi, b.report.chart.year.ganzhi);
                    }
                    assert_eq!(a.interval.end.jde_tt, b.interval.start.jde_tt);
                    assert_eq!(a.interval.start.jde_tt, r.minute.start.jde_tt);
                    assert_eq!(b.interval.end.jde_tt, r.minute.end.jde_tt);
                    for c in &r.candidates {
                        let p = c.representative.jde_tt;
                        assert!(p > c.interval.start.jde_tt && p < c.interval.end.jde_tt);
                        assert_eq!(p, c.report.cycle_basis.birth_jde_tt);
                        let age = c.report.cycle_basis.unrounded_start_age_years;
                        assert!(
                            age >= c.start_age_range.min_years
                                && age <= c.start_age_range.max_years
                        );
                        assert_eq!(c.report.chart.dayun.as_ref().unwrap().pillars.len(), 10);
                    }
                    count += 1;
                }
            }
        }
        assert_eq!(count, 3048);
    }
    #[test]
    fn leap_duration_historical_overlap_and_civil_rollover_are_explicit() {
        let mut i = BirthInput {
            year: 2016,
            month: 12,
            day: 31,
            hour: 23,
            minute: 59,
            tz: 0.0,
            gender: Some(Gender::Female),
        };
        let r = compute_report_utc_minute(i).unwrap();
        assert!((r.minute.elapsed_tt_seconds - 61.0).abs() < 0.0001);
        assert_eq!(r.candidates.len(), 1);
        let c = &r.candidates[0];
        assert_eq!(
            c.report.chart.day.ganzhi,
            compute_report_utc(i).unwrap().chart.day.ganzhi
        );
        assert!(
            ((c.start_age_range.max_years - c.start_age_range.min_years) * 3.0 * 86400.0 - 61.0)
                .abs()
                < 0.0001
        );
        i.year = 1961;
        i.month = 7;
        i.day = 31;
        assert_eq!(
            compute_report_utc_minute(i).unwrap_err(),
            UtcReportError::AmbiguousCivilInstant
        );
        i.month = 8;
        i.day = 1;
        i.hour = 0;
        i.minute = 0;
        assert_eq!(
            compute_report_utc_minute(i).unwrap_err(),
            UtcReportError::AmbiguousCivilInstant
        );
        i.year = 2027;
        i.month = 7;
        i.day = 1;
        assert_eq!(
            compute_report_utc_minute(i).unwrap_err(),
            UtcReportError::UnsupportedCoverage
        );
    }
}
