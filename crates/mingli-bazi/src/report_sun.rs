//! Report-only full VSOP87D Earth apparent solar longitude, equinox of date.
//! FK5 and approximate aberration follow Meeus ch25; IAU1980 nutation from
//! astro 2.0.0. Reference implementation: astronomia v4.2.0 solar.trueVSOP87 /
//! apparentVSOP87. This is an analytical model, not measured UT1 or a JPL ephemeris.
use mingli_astro::{delta_t_seconds, julian_day};
/// UT to TT for report calculations, including adjacent 1899 roots of 1900 inputs.
/// Decimal-year mapping matches the existing engine; this is not observed UT1.
#[must_use]
pub fn report_jd_ut_to_jde(jd: f64) -> f64 {
    let year = 2000.0 + (jd - 2451545.0) / 365.25;
    let dt = if year < 1900.0 {
        let t = year - 1860.0;
        7.62 + 0.5737 * t - 0.251754 * t.powi(2) + 0.01680668 * t.powi(3) - 0.0004473624 * t.powi(4)
            + t.powi(5) / 233174.0
    } else {
        delta_t_seconds(year)
    };
    jd + dt / 86400.0
}
/// High-order report model identifier; never used by the legacy chart endpoint.
pub const REPORT_SOLAR_MODEL: &str = "vsop87d-iau1980-delta-t-v1";
/// Apparent geocentric solar longitude in degrees, given TT Julian ephemeris day.
#[must_use]
pub fn report_solar_longitude(jde: f64) -> f64 {
    let earth = vsop87::vsop87d::earth(jde);
    let lon = earth.longitude().to_degrees() + 180.0;
    let nutation = astro::nutation::nutation(jde).0.to_degrees();
    (lon - 0.09033 / 3600.0 + nutation - 20.4898 / (3600.0 * earth.distance())).rem_euclid(360.0)
}
fn residual(jd: f64, target: f64) -> f64 {
    (report_solar_longitude(report_jd_ut_to_jde(jd)) - target + 180.0).rem_euclid(360.0) - 180.0
}
/// Solve a nearby solar crossing in UT using the report model. Caller supplies
/// a guess within eight days, satisfied by the mean-motion year/jie guesses.
#[must_use]
pub fn report_solar_term_time_near(guess: f64, target: f64) -> f64 {
    let mut lo = guess - 8.0;
    let mut hi = guess + 8.0;
    assert!(
        residual(lo, target) < 0.0 && residual(hi, target) > 0.0,
        "unbracketed report solar root"
    );
    // ~40 microseconds is the f64 JD grid near this epoch. Stop on adjacency.
    for _ in 0..48 {
        let mid = (lo + hi) * 0.5;
        if mid == lo || mid == hi {
            break;
        }
        if residual(mid, target) >= 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    (lo + hi) * 0.5
}
/// Solar-term root in the requested Gregorian year (15-degree term targets).
#[must_use]
pub fn report_solar_term_jd(year: i32, target: f64) -> f64 {
    let jan1 = julian_day(year, 1, 1.0);
    let lon = report_solar_longitude(report_jd_ut_to_jde(jan1));
    report_solar_term_time_near(jan1 + (target - lon).rem_euclid(360.0) / 0.98565, target)
}
