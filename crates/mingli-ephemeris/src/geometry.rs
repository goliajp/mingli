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
