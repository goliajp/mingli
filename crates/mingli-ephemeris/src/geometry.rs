//! 球面天文几何：地理坐标与上升点 / 中天的闭式解。
//!
//! 这层只做几何，不含任何占星或术数语义——西洋占星与印度 Jyotish 都从这里取
//! Asc/MC，宫位制（Placidus / Koch 等）则属于各自叶的领域知识。

use serde::Serialize;

/// 地理坐标（地心算上升点/中天所需）。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct GeoLocation {
    /// 纬度（度，北纬为正）。
    pub latitude: f64,
    /// 经度（度，东经为正）。
    pub longitude: f64,
}

/// 由本地恒星时 RAMC、黄赤交角 ε、地理纬度 φ（皆度）算上升点 Asc 与中天 MC 黄经（度）。
///
/// 闭式三角（对齐 Swiss `swehouse.c`，象限由 `atan2` 直接给出）：
/// - `MC  = atan2(sin RAMC, cos RAMC · cos ε)`
/// - `Asc = atan2(cos RAMC, −(sin RAMC · cos ε + tan φ · sin ε))`
#[must_use]
pub fn asc_mc(ramc_deg: f64, obliquity_deg: f64, lat_deg: f64) -> (f64, f64) {
    let ramc = ramc_deg.to_radians();
    let eps = obliquity_deg.to_radians();
    let phi = lat_deg.to_radians();
    let mc = ramc
        .sin()
        .atan2(ramc.cos() * eps.cos())
        .to_degrees()
        .rem_euclid(360.0);
    let asc = ramc
        .cos()
        .atan2(-(ramc.sin() * eps.cos() + phi.tan() * eps.sin()))
        .to_degrees()
        .rem_euclid(360.0);
    (asc, mc)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 赤道上的闭式解：RAMC=0（春分点上中天）时 MC=0°、Asc=90°（东地平正是巨蟹 0°）。
    #[test]
    fn on_the_equator_the_closed_form_is_exact() {
        let (asc, mc) = asc_mc(0.0, 23.44, 0.0);
        assert!(mc.abs() < 1e-9 || (mc - 360.0).abs() < 1e-9, "MC={mc}");
        assert!((asc - 90.0).abs() < 1e-9, "Asc={asc}");
        // RAMC=90°（夏至点上中天）：MC=90°，Asc=180°（天秤 0°）。
        let (asc, mc) = asc_mc(90.0, 23.44, 0.0);
        assert!((mc - 90.0).abs() < 1e-9, "MC={mc}");
        assert!((asc - 180.0).abs() < 1e-9, "Asc={asc}");
    }

    /// 权威本命盘校验：Diana, Princess of Wales（Rodden AA）。
    ///
    /// 1961-07-01 19:45 BST（= UT 18:45），Sandringham 52°50′N 0°30′E。
    /// astrotheme 与 astro.com 两处独立给出 Asc = 射手 18°24′ = 258.40°、
    /// MC = 天秤 23°03′ = 203.05°。
    ///
    /// 这条 oracle 在 `mingli-astrology` 里也有一份（整条管线级）；此处是**几何本身**的
    /// 直接校验——`asc_mc` 住在本 crate，就要在本 crate 里独立立得住，不靠下游代验。
    #[test]
    fn ascendant_and_midheaven_match_diana() {
        let m = mingli_astro::Moment::new(1961, 7, 1, 19, 45, 1.0);
        let geo = GeoLocation { latitude: 52.833, longitude: 0.500 };
        // 本地恒星时 RAMC = 格林尼治平恒星时 + 东经。
        let ramc = (m.sidereal_time + geo.longitude).rem_euclid(360.0);
        let (asc, mc) = asc_mc(ramc, m.obliquity, geo.latitude);
        // 容差 0.05°（3′）。限制因素有两条，都比它小得多：oracle 只给到角分（±0.5′ = 0.008°），
        // 本算用平恒星时 / 平交角、不含章动（Δψ·cos ε ≤ 0.005°）。实测 Asc 差 +0.0072°、MC +0.0047°，
        // 余量约七倍。原先取 0.5°——比实测松近百倍，那样的容差验不出任何回归。
        assert!((asc - 258.40).abs() < 0.05, "Asc={asc:.3}°，应 ≈258.40°（射手 18°24′）");
        assert!((mc - 203.05).abs() < 0.05, "MC={mc:.3}°，应 ≈203.05°（天秤 23°03′）");
    }

    /// 南北半球对称：同一 RAMC 下 MC 与纬度无关，Asc 随纬度变。
    #[test]
    fn midheaven_ignores_latitude_but_the_ascendant_does_not() {
        let (asc_n, mc_n) = asc_mc(123.0, 23.44, 52.0);
        let (asc_s, mc_s) = asc_mc(123.0, 23.44, -52.0);
        assert!((mc_n - mc_s).abs() < 1e-9, "MC 只由 RAMC 与 ε 定");
        assert!((asc_n - asc_s).abs() > 1.0, "Asc 必随纬度变：{asc_n} vs {asc_s}");
        // 值域恒在 [0,360)
        for (a, c) in [(asc_n, mc_n), (asc_s, mc_s)] {
            assert!((0.0..360.0).contains(&a) && (0.0..360.0).contains(&c));
        }
    }
}
