#![allow(clippy::float_cmp, reason = "report evidence must be bit-identical across paths, not approximately equal")]
#![allow(clippy::cast_precision_loss, reason = "day numbers near 2.4e6 are exact in f64")]
use super::*;
use mingli_astro::{jd_from_local, julian_day};
#[test]
fn utc_steps_drift_and_inverse_do_not_invent_civil_times() {
    for &(y, m, _, _, _) in &crate::report_utc_data::SEGMENTS {
        let jd = julian_day(y, m, 1.0);
        for offset in [-1.0 / 1440.0, 0.0, 0.5, 1.0] {
            let instant = report_utc_instant(jd + offset).unwrap();
            match report_tt_to_civil(instant.jde_tt) {
                Ok(back) => assert!((back.jd_civil - instant.jd_civil).abs() < 1e-8),
                Err(UtcReportError::AmbiguousCivilInstant) => {
                    assert!(offset == 0.0 && [(1961, 8), (1968, 2)].contains(&(y, m)));
                }
                Err(e) => panic!("unexpected inverse {y}-{m}/{offset}: {e}"),
            }
        }
    }
    let last = report_utc_instant(jd_from_local(2016, 12, 31, 23, 59, 0.0, 0.0)).unwrap();
    let next = report_utc_instant(julian_day(2017, 1, 1.0)).unwrap();
    assert!(((next.jde_tt - last.jde_tt) * 86400.0 - 61.0).abs() < 0.0001);
    assert_eq!(
        report_tt_to_civil(next.jde_tt - 0.5 / 86400.0).unwrap_err(),
        UtcReportError::UnrepresentableCivilInstant
    );
    let boundary = julian_day(1961, 8, 1.0);
    let after = report_utc_instant(boundary).unwrap();
    assert_eq!(
        report_tt_to_civil(after.jde_tt + 0.02 / 86400.0).unwrap_err(),
        UtcReportError::AmbiguousCivilInstant
    );
    for jd in [
        julian_day(1898, 12, 31.0),
        julian_day(2027, 7, 1.0),
        julian_day(2100, 1, 1.0),
    ] {
        assert_eq!(
            report_utc_instant(jd).unwrap_err(),
            UtcReportError::UnsupportedCoverage
        );
    }
}
fn check(input: BirthInput) -> BaziUtcReport {
    let r = compute_report_utc(input).unwrap();
    let b = &r.cycle_basis;
    assert_eq!(b.schema_version, 2);
    assert_eq!(b.model_id, REPORT_UTC_MODEL);
    assert!(b.previous_jie.jde_tt <= b.birth_jde_tt && b.next_jie.jde_tt >= b.birth_jde_tt);
    let year = if b.birth_jde_tt < b.li_chun.jde_tt {
        input.year - 1
    } else {
        input.year
    };
    assert_eq!(r.chart.year.ganzhi, year_ganzhi(year).to_string());
    let branch =
        (2 + ((b.birth_longitude_deg - 315.0).rem_euclid(360.0) / 30.0).floor() as usize) % 12;
    assert_eq!(r.chart.month.branch, BRANCHES[branch]);
    for j in [&b.previous_jie, &b.next_jie, &b.li_chun] {
        let t = report_utc_instant(j.jd_civil).unwrap();
        assert!((t.jde_tt - j.jde_tt).abs() < 1e-8);
        let residual = (report_solar_longitude(j.jde_tt) - j.target_longitude_deg + 180.0)
            .rem_euclid(360.0)
            - 180.0;
        assert!(residual.abs() < 1e-7);
    }
    let d = r.chart.dayun.as_ref().unwrap();
    assert_eq!(
        b.interval_days,
        if d.forward {
            b.next_jie.jde_tt - b.birth_jde_tt
        } else {
            b.birth_jde_tt - b.previous_jie.jde_tt
        }
    );
    assert_eq!(
        d.pillars[0].start_age,
        (b.interval_days / 3.0).round() as u32
    );
    assert_eq!(
        d.start_age_years,
        (b.interval_days / 3.0 * 100.0).round() / 100.0
    );
    r
}
#[test]
fn utc_reports_boundary_matrix_and_all_covered_jie() {
    let mut dump = Vec::new();
    for (y, m, d, h, min) in [
        (1900, 1, 1, 0, 0),
        (1959, 12, 31, 23, 59),
        (1960, 1, 1, 0, 0),
        (1961, 8, 1, 0, 0),
        (1968, 2, 1, 0, 0),
        (1972, 1, 1, 0, 0),
        (1972, 7, 1, 0, 0),
        (2016, 12, 31, 23, 59),
        (2017, 1, 1, 0, 0),
        (2026, 12, 31, 23, 59),
        (2027, 6, 1, 0, 0),
        (2013, 5, 5, 8, 10),
    ] {
        for tz in [-12.0, 5.75, 14.0] {
            for gender in [Gender::Male, Gender::Female] {
                let i = BirthInput {
                    year: y,
                    month: m,
                    day: d,
                    hour: h,
                    minute: min,
                    tz,
                    gender: Some(gender),
                };
                let response = check(i);
                dump.push(serde_json::json!({"input":{"year":y,"month":m,"day":d,"hour":h,"minute":min,"tz":tz,"gender":gender,"true_solar_time":false},"response":response}));
            }
        }
    }
    for y in 1900..=2026 {
        for target in (15..360).step_by(30) {
            let tt = report_utc_solar_term_tt(y, f64::from(target)).unwrap();
            let root = report_tt_to_civil(tt).unwrap();
            let minute = ((root.jd_civil + 0.5) * 1440.0).floor() as i64;
            for extra in [0, 1] {
                for g in [Gender::Male, Gender::Female] {
                    let n = minute + extra;
                    let jd = n.div_euclid(1440) as f64 - 0.5;
                    let within = n.rem_euclid(1440);
                    let month = (1..=12)
                        .rev()
                        .find(|m| julian_day(y, *m, 1.0) <= jd)
                        .unwrap();
                    let day = (jd - julian_day(y, month, 1.0)).round() as u32 + 1;
                    check(BirthInput {
                        year: y,
                        month,
                        day,
                        hour: (within / 60) as u32,
                        minute: (within % 60) as u32,
                        tz: 0.0,
                        gender: Some(g),
                    });
                }
            }
        }
    }
    if let Ok(path) = std::env::var("MINGLI_UTC_BOUNDARIES_DUMP") {
        std::fs::write(path, serde_json::to_string_pretty(&dump).unwrap()).unwrap();
    }
}

#[test]
fn utc_design_samples_include_jie_minute_and_integer_rounding_threshold() {
    let mut samples = Vec::new();
    let mut push = |name: &str, i: BirthInput| {
        let response = check(i);
        samples.push(serde_json::json!({"name":name,"input":{"year":i.year,"month":i.month,"day":i.day,"hour":i.hour,"minute":i.minute,"tz":i.tz,"gender":i.gender,"true_solar_time":false},"response":response}));
    };
    for (name, min) in [("jie_recorded_minute", 18), ("jie_following_minute", 19)] {
        push(
            name,
            BirthInput {
                year: 2013,
                month: 5,
                day: 5,
                hour: 8,
                minute: min,
                tz: 0.0,
                gender: Some(Gender::Female),
            },
        );
    }
    let jie = report_utc_solar_term_tt(2013, 45.0).unwrap();
    let half = report_tt_to_civil(jie - 1.5 * 3.0).unwrap();
    let minute = ((half.jd_civil + 0.5) * 1440.0).floor() as i64;
    for (name, extra) in [
        ("rounding_recorded_minute", 0),
        ("rounding_following_minute", 1),
    ] {
        let n = minute + extra;
        let jd = n.div_euclid(1440) as f64 - 0.5;
        let within = n.rem_euclid(1440);
        let month = (1..=12)
            .rev()
            .find(|m| julian_day(2013, *m, 1.0) <= jd)
            .unwrap();
        let day = (jd - julian_day(2013, month, 1.0)).round() as u32 + 1;
        push(
            name,
            BirthInput {
                year: 2013,
                month,
                day,
                hour: (within / 60) as u32,
                minute: (within % 60) as u32,
                tz: 0.0,
                gender: Some(Gender::Female),
            },
        );
    }
    assert_eq!(
        samples[2]["response"]["chart"]["dayun"]["pillars"][0]["start_age"],
        2
    );
    assert_eq!(
        samples[3]["response"]["chart"]["dayun"]["pillars"][0]["start_age"],
        1
    );
    if let Ok(path) = std::env::var("MINGLI_UTC_DESIGN_DUMP") {
        std::fs::write(path, serde_json::to_string_pretty(&samples).unwrap()).unwrap();
    }
}
