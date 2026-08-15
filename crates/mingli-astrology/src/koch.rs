//! Koch 分宫制：等赤经象限四分（移植自 Swiss Ephemeris `swehouse.c` 'K' case）。
//!
//! Koch（Walter Koch， 1895–1970）以**出生地的赤经**为基准在每象限作等增分，
//! 与 Placidus 的「半弧三分」不同——Koch 的中间宫尖由 MC 处一次性几何量 `ad3` 给出，
//! 不需迭代。
//!
//! **算法**（`swehouse.c` 第 1250–1272 行，共享 [`crate::placidus`] 模块的内部 `asc1`/`pack`
//! 工具函数，以保持与 Placidus 的赤道→黄道几何完全一致）：
//!
//! ```text
//!   sina = sin(MC) · sin ε / cos φ
//!   cosa = √(1 − sina²)              // 恒 ≥ 0
//!   c    = atan(tan φ / cosa)
//!   ad3  = asin(sin c · sina) / 3    // 度
//!   cusp[11] = Asc1(RAMC + 30  − 2·ad3, φ, sin ε, cos ε)
//!   cusp[12] = Asc1(RAMC + 60  −   ad3, ...)
//!   cusp[ 2] = Asc1(RAMC + 120 +   ad3, ...)
//!   cusp[ 3] = Asc1(RAMC + 150 + 2·ad3, ...)
//! ```
//!
//! 1/10/4/7 = Asc/MC/IC/DC 闭式；5/6/8/9 由对宫等同性 `cusp[k+6] = cusp[k] + 180°` 派生。
//!
//! **极区**：`|φ| ≥ 90° − ε`（约 |φ| ≥ 66.5°）失效——`swehouse.c` 在此回退 Porphyry，
//! 本模块返回 [`None`]，上层应改用 [`crate::placidus::porphyry_cusps`]。

use crate::placidus::{asc1, pack, PlacidusCusps};

/// 计算 12 个 Koch 宫尖（度，`[0, 360)`）。`asc` 与 `mc` 由调用方算好。
///
/// 极区（`|φ| ≥ 90° − ε`）返回 [`None`]。
#[must_use]
pub fn cusps(
    ramc_deg: f64,
    obliquity_deg: f64,
    lat_deg: f64,
    asc: f64,
    mc: f64,
) -> Option<PlacidusCusps> {
    if lat_deg.abs() >= 90.0 - obliquity_deg {
        return None;
    }
    let phi_rad = lat_deg.to_radians();
    let sine = obliquity_deg.to_radians().sin();
    let cose = obliquity_deg.to_radians().cos();

    // 定义角 a （swehouse.c 同款）： sin a = sin(MC) · sin ε / cos φ。
    // 数值上恒在 [-1, 1] 内（合法 MC/ε/φ），浮点边界以 clamp 防御。
    let sin_a = (mc.to_radians().sin() * sine / phi_rad.cos()).clamp(-1.0, 1.0);
    let cos_a = (1.0 - sin_a * sin_a).sqrt(); // 恒 ≥ 0

    // c_aux = atan(tan φ / cos a)；ad3 = asin(sin c_aux · sin a) / 3 （度）。
    let c_aux = (phi_rad.tan() / cos_a).atan();
    let ad3 = (c_aux.sin() * sin_a).asin().to_degrees() / 3.0;

    let c11 = asc1(ramc_deg + 30.0 - 2.0 * ad3, lat_deg, sine, cose);
    let c12 = asc1(ramc_deg + 60.0 - ad3, lat_deg, sine, cose);
    let c2 = asc1(ramc_deg + 120.0 + ad3, lat_deg, sine, cose);
    let c3 = asc1(ramc_deg + 150.0 + 2.0 * ad3, lat_deg, sine, cose);

    Some(pack(asc, mc, c11, c12, c2, c3))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::placidus::signed_diff_deg;

    // —— Diana, Princess of Wales (Rodden AA) Koch oracle ——
    // 由 Swiss Ephemeris (pyswisseph) 对同一输入直接算出：
    //   1961-07-01 19:45 BST (UT 18:45), Sandringham 52°50′N 0°30′E
    //   JD UT = 2437482.28125; lat=52.8333..°, lon=0.5°
    //   swe.houses(jd, lat, lon, b'K') →
    //     1=258.4085  2=282.4609  3=320.9800  4=23.0521  5=40.9067  6=58.9750
    //     7=78.4085   8=102.4609  9=140.9800  10=203.0521 11=220.9067 12=238.9750
    // 容差 0.05°（角分级，与 Placidus 同口径——平恒星时 + 平交角的截断精度）。
    #[test]
    fn diana_koch_cusps() {
        let m = mingli_astro::Moment::new(1961, 7, 1, 19, 45, 1.0);
        let geo_lat = 52.833;
        let geo_lon = 0.5;
        let ramc = (m.sidereal_time + geo_lon).rem_euclid(360.0);
        let eps = m.obliquity;
        let (asc, mc) = crate::asc_mc(ramc, eps, geo_lat);
        let cs = cusps(ramc, eps, geo_lat, asc, mc).expect("Diana 非极区");
        let expected: [(usize, f64); 12] = [
            (1, 258.4085),
            (2, 282.4609),
            (3, 320.9800),
            (4, 23.0521),
            (5, 40.9067),
            (6, 58.9750),
            (7, 78.4085),
            (8, 102.4609),
            (9, 140.9800),
            (10, 203.0521),
            (11, 220.9067),
            (12, 238.9750),
        ];
        for (k, want) in expected {
            let got = cs.cusps[k];
            let diff = signed_diff_deg(got, want).abs();
            assert!(
                diff < 0.05,
                "Koch cusp {k}: got {got:.4}°, want {want:.4}°, diff {diff:.4}°"
            );
        }
    }

    #[test]
    fn polar_region_returns_none() {
        // |φ| ≥ 90° − ε ≈ 66.56° → Koch 失效
        assert!(cusps(0.0, 23.44, 70.0, 90.0, 0.0).is_none());
        assert!(cusps(0.0, 23.44, -80.0, 90.0, 0.0).is_none());
    }

    #[test]
    fn cusp_opposites_hold() {
        // 任一 cusp k 与 k+6 应严格 180° 对宫（模 360°）。
        let m = mingli_astro::Moment::new(1961, 7, 1, 19, 45, 1.0);
        let ramc = (m.sidereal_time + 0.5).rem_euclid(360.0);
        let (asc, mc) = crate::asc_mc(ramc, m.obliquity, 52.833);
        let cs = cusps(ramc, m.obliquity, 52.833, asc, mc).unwrap();
        for k in 1..=6 {
            let diff = signed_diff_deg(cs.cusps[k] + 180.0, cs.cusps[k + 6]).abs();
            assert!(diff < 1e-9, "Koch cusp {k} vs {}: diff {diff}", k + 6);
        }
    }

    #[test]
    fn equator_phi_zero_implies_ad3_zero() {
        // φ=0 → tan φ = 0 → c = 0 → sin c = 0 → ad3 = 0;
        // 11/12/2/3 退化为 Asc1(RAMC + 30/60/120/150， φ=0， ...) 与赤经偏移直接给。
        let cs = cusps(45.0, 23.44, 0.0, 0.0, 0.0).unwrap();
        // φ=0 时 Koch cusp k 与 RAMC + offset 的 Asc1 应一致——
        // 直接调用对比验证 ad3=0：
        let sin_eps = 23.44_f64.to_radians().sin();
        let cos_eps = 23.44_f64.to_radians().cos();
        for (k, off) in [(11usize, 30.0_f64), (12, 60.0), (2usize, 120.0), (3, 150.0)] {
            let want = asc1(45.0 + off, 0.0, sin_eps, cos_eps);
            assert!(
                (cs.cusps[k] - want).abs() < 1e-9,
                "phi=0 cusp {k}: got {} want {want}",
                cs.cusps[k]
            );
        }
    }

    #[test]
    fn mc_at_zero_implies_ad3_zero() {
        // MC = 0° → sin MC = 0 → sina = 0 → ad3 = 0
        // Koch 中间宫尖退化为 Asc1(RAMC + offset， φ)，不含赤纬偏移。
        let ramc = 0.0;
        let eps = 23.44;
        let lat = 30.0;
        let cs = cusps(ramc, eps, lat, 90.0, 0.0).unwrap();
        let s = eps.to_radians().sin();
        let co = eps.to_radians().cos();
        for (k, off) in [(11usize, 30.0), (12, 60.0), (2usize, 120.0), (3, 150.0)] {
            let exp = asc1(ramc + off, lat, s, co);
            assert!(
                (cs.cusps[k] - exp).abs() < 1e-9,
                "k={k} got {} exp {exp}",
                cs.cusps[k]
            );
        }
    }

    #[test]
    fn near_polar_limit_still_returns() {
        // |φ| 略小于 90 − ε，Koch 还应能算出非 None。
        let eps = 23.44;
        let lat = 90.0 - eps - 0.5; // 略低于极区
        let cs = cusps(0.0, eps, lat, 90.0, 0.0);
        assert!(cs.is_some());
    }
}
