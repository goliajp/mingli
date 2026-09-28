#![allow(clippy::float_cmp, reason = "report evidence must be bit-identical across paths, not approximately equal")]
#![allow(clippy::cast_precision_loss, reason = "day numbers near 2.4e6 are exact in f64")]
use super::*;
use mingli_astro::{jd_ut_to_jde, julian_day, sun_apparent_longitude};
fn verify(input: BirthInput) {
    let r = compute_report(input).unwrap();
    let b = &r.cycle_basis;
    assert_eq!(
        serde_json::to_value(&r.chart).unwrap(),
        serde_json::to_value(compute(input)).unwrap()
    );
    assert_eq!(b.birth_jde_tt, jd_ut_to_jde(b.birth_jd_ut));
    assert_eq!(
        b.birth_longitude_deg,
        sun_apparent_longitude(b.birth_jde_tt)
    );
    assert!(b.previous_jie.jd_ut <= b.birth_jd_ut + 1e-8);
    assert!(b.next_jie.jd_ut >= b.birth_jd_ut - 1e-8);
    assert!((29.0..33.0).contains(&(b.next_jie.jd_ut - b.previous_jie.jd_ut)));
    assert_eq!(
        (b.next_jie.target_longitude_deg - b.previous_jie.target_longitude_deg).rem_euclid(360.0),
        30.0
    );
    for j in [&b.previous_jie, &b.next_jie] {
        assert!((0.0..360.0).contains(&j.target_longitude_deg));
        assert_eq!((j.target_longitude_deg - 15.0).rem_euclid(30.0), 0.0);
        let residual = (sun_apparent_longitude(jd_ut_to_jde(j.jd_ut)) - j.target_longitude_deg
            + 180.0)
            .rem_euclid(360.0)
            - 180.0;
        assert!(residual.abs() < 1e-5, "residual {residual}");
    }
    let d = r.chart.dayun.unwrap();
    assert_eq!(b.selected, if d.forward { "next" } else { "previous" });
    assert_eq!(
        b.interval_days,
        if d.forward {
            b.next_jie.jd_ut - b.birth_jd_ut
        } else {
            b.birth_jd_ut - b.previous_jie.jd_ut
        }
    );
    assert_eq!(b.unrounded_start_age_years, b.interval_days / 3.0);
    assert_eq!(
        d.start_age_years,
        (b.unrounded_start_age_years * 100.0).round() / 100.0
    );
    assert_eq!(
        d.pillars[0].start_age,
        b.unrounded_start_age_years.round() as u32
    );
}
// Convert an integer local minute near an in-year jie without rounding seconds
// into a fictional input: report API accepts real civil minutes only.
fn input_at_minute(y: i32, minute: i64, tz: f64, g: Gender) -> BirthInput {
    let day_number = minute.div_euclid(1440);
    let within = minute.rem_euclid(1440);
    let jd = day_number as f64 - 0.5;
    let m = (1..=12)
        .rev()
        .find(|m| julian_day(y, *m, 1.0) <= jd)
        .unwrap();
    let day = (jd - julian_day(y, m, 1.0)).round() as u32 + 1;
    BirthInput {
        year: y,
        month: m,
        day,
        hour: (within / 60) as u32,
        minute: (within % 60) as u32,
        tz,
        gender: Some(g),
    }
}
#[test]
fn report_evidence_all_supported_years_jie_both_sides_and_extreme_offsets() {
    for y in 1900..=2100 {
        for target in (15..360).step_by(30) {
            let term = solar_term_jd(y, f64::from(target));
            for tz in [-12.0, 5.75, 14.0] {
                let minute = ((term + tz / 24.0 + 0.5) * 1440.0).floor() as i64;
                for offset in [0, 1] {
                    for gender in [Gender::Male, Gender::Female] {
                        verify(input_at_minute(y, minute + offset, tz, gender));
                    }
                }
            }
        }
    }
}
#[test]
fn report_evidence_civil_year_wrap_and_missing_gender() {
    for y in [1900, 1901, 1999, 2000, 2099, 2100] {
        for (m, d, h, min) in [(1, 1, 0, 0), (12, 31, 23, 59)] {
            for tz in [-12.0, 14.0] {
                for gender in [Gender::Male, Gender::Female] {
                    verify(BirthInput {
                        year: y,
                        month: m,
                        day: d,
                        hour: h,
                        minute: min,
                        tz,
                        gender: Some(gender),
                    });
                }
            }
        }
    }
    assert!(compute_report(BirthInput {
        year: 2000,
        month: 1,
        day: 1,
        hour: 0,
        minute: 0,
        tz: 8.0,
        gender: None
    })
    .is_none());
}

#[cfg(feature = "report-vsop")]
fn verify_vsop(input: BirthInput) -> BaziReport {
    let r = compute_report_vsop(input).unwrap();
    let b = &r.cycle_basis;
    assert_eq!(b.model_id, REPORT_SOLAR_MODEL);
    assert_eq!(b.birth_jde_tt, report_jd_ut_to_jde(b.birth_jd_ut));
    assert_eq!(
        b.birth_longitude_deg,
        report_solar_longitude(b.birth_jde_tt)
    );
    assert!(b.previous_jie.jd_ut <= b.birth_jd_ut + 1e-8);
    assert!(b.next_jie.jd_ut >= b.birth_jd_ut - 1e-8);
    let year = if b.birth_jd_ut < report_solar_term_jd(input.year, 315.0) {
        input.year - 1
    } else {
        input.year
    };
    assert_eq!(r.chart.year.ganzhi, year_ganzhi(year).to_string());
    let branch =
        (2 + ((b.birth_longitude_deg - 315.0).rem_euclid(360.0) / 30.0).floor() as usize) % 12;
    assert_eq!(r.chart.month.branch, BRANCHES[branch]);
    assert_eq!(
        (b.next_jie.target_longitude_deg - b.previous_jie.target_longitude_deg).rem_euclid(360.0),
        30.0
    );
    for j in [&b.previous_jie, &b.next_jie] {
        let residual =
            (report_solar_longitude(report_jd_ut_to_jde(j.jd_ut)) - j.target_longitude_deg + 180.0)
                .rem_euclid(360.0)
                - 180.0;
        assert!(residual.abs() < 1e-7, "{residual}");
    }
    let d = r.chart.dayun.as_ref().unwrap();
    assert_eq!(d.forward, b.selected == "next");
    assert_eq!(
        b.interval_days,
        if d.forward {
            b.next_jie.jd_ut - b.birth_jd_ut
        } else {
            b.birth_jd_ut - b.previous_jie.jd_ut
        }
    );
    assert_eq!(b.unrounded_start_age_years, b.interval_days / 3.0);
    assert_eq!(
        d.start_age_years,
        (b.unrounded_start_age_years * 100.0).round() / 100.0
    );
    assert_eq!(
        d.pillars[0].start_age,
        b.unrounded_start_age_years.round() as u32
    );
    r
}
/// The 1900–2100 sweep below is split into year ranges so its shards run on separate
/// test threads; together they must cover every supported year exactly once.
#[cfg(feature = "report-vsop")]
const HIGH_MODEL_YEARS: [(i32, i32); 4] = [(1900, 1950), (1951, 2000), (2001, 2050), (2051, 2100)];

/// Every supported jie in `from..=to`: both real minute inputs and both cycle directions.
/// Returns the largest omitted general FK5 latitude contribution seen, in arcseconds.
#[cfg(feature = "report-vsop")]
fn high_model_years(from: i32, to: i32) -> f64 {
    let mut max_lat_correction = 0.0f64;
    for y in from..=to {
        for target in (15..360).step_by(30) {
            let term = report_solar_term_jd(y, f64::from(target));
            let tt = report_jd_ut_to_jde(term);
            let earth = vsop87::vsop87d::earth(tt);
            let t = (tt - 2_451_545.0) / 36525.0;
            let p = earth.longitude() + std::f64::consts::PI
                - (1.397 * t + 0.00031 * t * t).to_radians();
            let omitted = (0.03916 * (p.cos() + p.sin()) * (-earth.latitude()).tan()).abs();
            max_lat_correction = max_lat_correction.max(omitted);
            let minute = ((term + 0.5) * 1440.0).floor() as i64;
            for offset in [0, 1] {
                for gender in [Gender::Male, Gender::Female] {
                    verify_vsop(input_at_minute(y, minute + offset, 0.0, gender));
                }
            }
        }
    }
    println!("Maximum omitted general FK5 latitude contribution, {from}–{to}: {max_lat_correction:.12} arcsec");
    max_lat_correction
}

#[cfg(feature = "report-vsop")]
#[test]
fn high_model_years_1900_1950() {
    let (from, to) = HIGH_MODEL_YEARS[0];
    assert!(high_model_years(from, to) < 0.000_001);
}
#[cfg(feature = "report-vsop")]
#[test]
fn high_model_years_1951_2000() {
    let (from, to) = HIGH_MODEL_YEARS[1];
    assert!(high_model_years(from, to) < 0.000_001);
}
#[cfg(feature = "report-vsop")]
#[test]
fn high_model_years_2001_2050() {
    let (from, to) = HIGH_MODEL_YEARS[2];
    assert!(high_model_years(from, to) < 0.000_001);
}
#[cfg(feature = "report-vsop")]
#[test]
fn high_model_years_2051_2100() {
    let (from, to) = HIGH_MODEL_YEARS[3];
    assert!(high_model_years(from, to) < 0.000_001);
}
#[cfg(feature = "report-vsop")]
#[test]
fn high_model_shards_cover_1900_to_2100_once() {
    assert_eq!(HIGH_MODEL_YEARS[0].0, 1900);
    assert_eq!(HIGH_MODEL_YEARS[HIGH_MODEL_YEARS.len() - 1].1, 2100);
    for w in HIGH_MODEL_YEARS.windows(2) {
        assert_eq!(w[1].0, w[0].1 + 1, "shards must be contiguous: {w:?}");
    }
}

#[cfg(feature = "report-vsop")]
#[test]
fn high_model_boundaries_use_one_model() {
    let mut dump = Vec::new();
    for y in [1900, 2013, 2100] {
        for target in [45, 315] {
            let term = report_solar_term_jd(y, f64::from(target));
            let minute = ((term + 0.5) * 1440.0).floor() as i64;
            for offset in [0, 1] {
                for gender in [Gender::Male, Gender::Female] {
                    let input = input_at_minute(y, minute + offset, 0.0, gender);
                    let response = verify_vsop(input);
                    dump.push(serde_json::json!({"input":{ "year":input.year,"month":input.month,"day":input.day,"hour":input.hour,"minute":input.minute,"tz":input.tz,"gender":input.gender,"true_solar_time":false},"response":response}));
                }
            }
        }
    }
    for y in [1900, 2013, 2100] {
        for (month, day, hour, minute) in [
            (1, 1, 0, 0),
            (12, 31, 23, 59),
            (5, 5, 8, 10),
            (6, 15, 23, 0),
        ] {
            for tz in [-12.0, 5.75, 14.0] {
                for gender in [Gender::Male, Gender::Female] {
                    let input = BirthInput {
                        year: y,
                        month,
                        day,
                        hour,
                        minute,
                        tz,
                        gender: Some(gender),
                    };
                    let response = verify_vsop(input);
                    dump.push(serde_json::json!({"input":{"year":y,"month":month,"day":day,"hour":hour,"minute":minute,"tz":tz,"gender":gender,"true_solar_time":false},"response":response}));
                }
            }
        }
    }
    if let Ok(path) = std::env::var("MINGLI_VSOP_BOUNDARIES_DUMP") {
        std::fs::write(path, serde_json::to_string_pretty(&dump).unwrap()).unwrap();
    }
}
