//! L1 物理石：行星星历——日月五星的地心黄道经度。
//!
//! 行星位置基于经审计的 [`vsop87`] crate（其日心 L/B/R 已对官方 VSOP87 参考文件验证）。
//! 本 crate 在其上做标准几何：日心直角坐标差得地心矢量，含**光行时**迭代校正，
//! 输出**当日平分点**的地心黄道经度（度，mean equinox of date）。占星（回归）直接用之；
//! Jyotish（恒星）再减 ayanamsa。
//!
//! 月亮位置基于经审计的 [`astro`] crate（Saurav Sachidanand 实现 Chapront ELP-2000/82
//! 主项 + Meeus 章动），返回**视位置**（apparent，含 Δψ·cos ε 章动）的地心黄道经/纬/距。
//! 行星(VSOP87)给 mean、月亮给 apparent，黄经差异 ~17″（章动），远小于占星默认容许度 6°。
//!
//! 正确性：太阳经度（地球日心经度 +180°）与 `mingli-astro` 独立的 Meeus 太阳模型自洽校验；
//! 月亮位置与 Meeus 第 47 章教科书算例(1992-04-12.0 TD， JDE 2448724.5)校验：
//! λ=133.1613° / β=−3.2292° / Δ=368 409.7 km（均见测试）。
//!
//! 缺口（🟡）：行星与 JPL Horizons 的逐点校验留作后续（上游 crate 已对 VSOP87 参考验证）；
//! 行星 apparent 化（加章动）未实现。

mod geometry;
pub use geometry::{asc_mc, GeoLocation};

use vsop87::vsop87d;

/// 光行时常数：每天文单位约 0.005 775 518 3 天。
const LIGHT_TIME_PER_AU: f64 = 0.005_775_518_3;

/// 可计算地心黄经的天体。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Body {
    /// 太阳。
    Sun,
    /// 月亮（apparent 位置，经 Chapront ELP-2000/82 + Meeus 章动）。
    Moon,
    /// 水星。
    Mercury,
    /// 金星。
    Venus,
    /// 火星。
    Mars,
    /// 木星。
    Jupiter,
    /// 土星。
    Saturn,
    /// 天王星。
    Uranus,
    /// 海王星。
    Neptune,
}

/// 月亮的地心位置。
#[derive(Debug, Clone, Copy)]
pub struct MoonPosition {
    /// 黄道经度（度，`[0,360)`，apparent，含章动）。
    pub longitude: f64,
    /// 黄道纬度（度，apparent）。
    pub latitude: f64,
    /// 地月距离（千米）。
    pub distance_km: f64,
}

struct Rect {
    x: f64,
    y: f64,
    z: f64,
}

/// 日心球面 （L，B，R 弧度/AU） → 日心直角坐标。
fn to_rect(s: vsop87::SphericalCoordinates) -> Rect {
    let (l, b, r) = (s.longitude(), s.latitude(), s.distance());
    Rect {
        x: r * b.cos() * l.cos(),
        y: r * b.cos() * l.sin(),
        z: r * b.sin(),
    }
}

/// 天体的日心球面坐标（7 颗行星各取自家 VSOP87D 级数；太阳取地球，月亮不走此路）。
fn heliocentric(body: Body, jde: f64) -> vsop87::SphericalCoordinates {
    match body {
        Body::Mercury => vsop87d::mercury(jde),
        Body::Venus => vsop87d::venus(jde),
        Body::Mars => vsop87d::mars(jde),
        Body::Jupiter => vsop87d::jupiter(jde),
        Body::Saturn => vsop87d::saturn(jde),
        Body::Uranus => vsop87d::uranus(jde),
        Body::Neptune => vsop87d::neptune(jde),
        // 太阳走地球的日心位置——地心太阳 = 地球日心 + 180°，见下方 `Body::Sun` 分支。
        // 月亮不用日心坐标（ELP-2000 直接给地心位置），上层已特判，不到此。
        Body::Sun | Body::Moon => vsop87d::earth(jde),
    }
}

/// 月亮在 `jde`（力学时儒略日）的地心 apparent 位置。
///
/// 经 Chapront ELP-2000/82 + Meeus 章动，校验 Meeus 第 47 章教科书算例。
#[must_use]
pub fn moon_geocentric(jde: f64) -> MoonPosition {
    let (p, dist_km) = astro::lunar::geocent_ecl_pos(jde);
    MoonPosition {
        longitude: p.long.to_degrees().rem_euclid(360.0),
        latitude: p.lat.to_degrees(),
        distance_km: dist_km,
    }
}

/// 月亮平升交点(ascending node)黄经 Ω（度，`[0, 360)`）。
///
/// 公式：Meeus *Astronomical Algorithms* 2nd ed 第 47 章 eq 47.7（精确五项）
/// `Ω = 125°.0445479 − 1934°.1362891·T + 0°.0020754·T² + T³/467441 − T⁴/60616000`,
/// 其中 `T = (JDE − J2000) / 36525`（儒略千年世纪）。
/// 18.6 年逆行周期（角速度 ≈ −1934°/世纪 ≈ −19.34°/yr）。
/// 系数与 soniakeys/meeus（Go 移植）`moonposition.go` 字符级一致；
/// SOFA `iauFaom03` (IERS 2003) J2000 给 125.04455501°，差 0.14″ 在公式精度内。
///
/// 中国传统四余之一**罗㬋**(Luohou) = `mean_lunar_node`（通行近代/印度对位）；
/// **计都**(Jidu) = `mean_lunar_node + 180°`（月降交点，同上派别）。
/// 注：沈括《梦溪笔谈》古法计都 = 月远地点（=月孛），清初汤若望后改用印度对位；
/// 本算法采通行近代（印度对位）。
#[must_use]
pub fn mean_lunar_node(jde: f64) -> f64 {
    let t = (jde - 2_451_545.0) / 36_525.0;
    let omega = 125.044_547_9 - 1_934.136_289_1 * t + 0.002_075_4 * t * t
        + t.powi(3) / 467_441.0
        - t.powi(4) / 60_616_000.0;
    omega.rem_euclid(360.0)
}

/// 月亮平近地点(perigee)黄经 Π（度，`[0, 360)`）。
///
/// 公式：Meeus *Astronomical Algorithms* 2nd ed **p.343**（未编号，与 Chapront ELP 同源）：
/// `Π = 83°.3532465 + 4069°.0137287·T − 0°.0103200·T² − T³/80053 + T⁴/18999000`,
/// `T = (JDE − J2000) / 36525`。8.85 年顺行周期（角速度 ≈ +4069°/世纪）。
/// 系数与 PyMeeus `Moon.longitude_mean_perigee` / NASA GSFC `lpteop_fortran.txt` /
/// soniakeys/meeus `moonposition.go` 三源字符级一致。
///
/// 注：T³ 项系数为 **`−1/80053`**（注意负号），T⁴ 项为 `+1/18999000`，与
/// 由 `L′ − M′` 派生的近似式（全正号）略有差异（~0.7″ 级别，系 Meeus 各章独立解算）。
#[must_use]
pub fn mean_lunar_perigee(jde: f64) -> f64 {
    let t = (jde - 2_451_545.0) / 36_525.0;
    let pi = 83.353_246_5 + 4_069.013_728_7 * t - 0.010_320_0 * t * t
        - t.powi(3) / 80_053.0
        + t.powi(4) / 18_999_000.0;
    pi.rem_euclid(360.0)
}

/// 月亮平远地点(apogee)黄经（度，`[0, 360)`）= **月孛**（Yuebo，中国传统四余之一）。
///
/// 几何上 = `mean_lunar_perigee + 180°`。
#[must_use]
pub fn mean_lunar_apogee(jde: f64) -> f64 {
    (mean_lunar_perigee(jde) + 180.0).rem_euclid(360.0)
}

/// 天体在 `jde`（力学时儒略日）的地心黄道经度（度，`[0,360)`，当日平分点）。
///
/// 月亮返回 apparent 经度（含章动）；太阳/行星返回 mean（VSOP87 当日平分点）。差异 ~17″。
#[must_use]
pub fn geocentric_ecliptic_longitude(body: Body, jde: f64) -> f64 {
    match body {
        Body::Sun => {
            // 太阳地心经度 = 地球日心经度 + 180°。
            let e = heliocentric(Body::Sun, jde);
            (e.longitude().to_degrees() + 180.0).rem_euclid(360.0)
        }
        Body::Moon => moon_geocentric(jde).longitude,
        _ => {
            let earth = to_rect(vsop87d::earth(jde));
            let mut tau = 0.0;
            let mut lambda = 0.0;
            // 光行时迭代：在「光离开行星的时刻」取其位置。三次足以收敛。
            for _ in 0..3 {
                let p = to_rect(heliocentric(body, jde - tau));
                let (dx, dy, dz) = (p.x - earth.x, p.y - earth.y, p.z - earth.z);
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                tau = LIGHT_TIME_PER_AU * dist;
                lambda = dy.atan2(dx).to_degrees().rem_euclid(360.0);
            }
            lambda
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jde_of(y: i32, m: u32, d: f64) -> f64 {
        mingli_astro::jd_ut_to_jde(mingli_astro::julian_day(y, m, d))
    }

    #[test]
    fn sun_matches_meeus_model() {
        // VSOP87（地球+180°） 与 astro 的 Meeus 太阳模型应在 ~arcmin 内一致。
        for (y, m, d) in [(2024, 3, 20.5), (2024, 6, 21.0), (1990, 6, 15.0), (2000, 1, 1.0)] {
            let jde = jde_of(y, m, d);
            let v = geocentric_ecliptic_longitude(Body::Sun, jde);
            let meeus = mingli_astro::sun_apparent_longitude(jde);
            // 环形最短差
            let diff = ((v - meeus + 180.0).rem_euclid(360.0) - 180.0).abs();
            assert!(diff < 0.02, "{y}-{m}： VSOP {v} vs Meeus {meeus}， 差 {diff}°");
        }
    }

    #[test]
    fn sun_near_zero_at_vernal_equinox() {
        // 2024 春分太阳黄经 ≈ 0°。
        let v = geocentric_ecliptic_longitude(Body::Sun, jde_of(2024, 3, 20.0));
        let d = (v.rem_euclid(360.0) - 360.0).abs().min(v.rem_euclid(360.0));
        assert!(d < 1.0, "春分太阳经度应近 0°，实得 {v}");
    }

    #[test]
    fn planets_in_range() {
        let jde = jde_of(2024, 6, 15.0);
        for body in [
            Body::Mercury,
            Body::Venus,
            Body::Mars,
            Body::Jupiter,
            Body::Saturn,
            Body::Uranus,
            Body::Neptune,
        ] {
            let lon = geocentric_ecliptic_longitude(body, jde);
            assert!((0.0..360.0).contains(&lon), "{body:?} 经度越界： {lon}");
        }
    }

    /// Meeus《Astronomical Algorithms》第 47 章教科书算例：1992-04-12.0 TD（JDE 2448724.5）
    /// 月亮 apparent 位置 λ=133°09′40.6″ ≈ 133.1613°，β=−3°13′45″ ≈ −3.2292°，Δ=368 409.7 km。
    /// 我们的 `astro` crate 实现给出 λ=133.1627° / β=−3.2291° / Δ=368 409.7 km，
    /// 经度差 ~5″（高阶 ELP 项截断 + 短周期章动残差），可接受。
    #[test]
    fn moon_matches_meeus_47a() {
        let m = moon_geocentric(2_448_724.5);
        assert!(
            (m.longitude - 133.1613).abs() < 0.01,
            "Moon λ={:.4}°，应 ≈133.1613°",
            m.longitude
        );
        assert!(
            (m.latitude - (-3.2292)).abs() < 0.01,
            "Moon β={:.4}°，应 ≈−3.2292°",
            m.latitude
        );
        assert!(
            (m.distance_km - 368_409.7).abs() < 1.0,
            "Moon Δ={:.1} km，应 ≈368409.7",
            m.distance_km
        );
    }

    #[test]
    fn moon_path_returns_apparent_longitude() {
        // 走 enum 分发路径（覆盖 Body::Moon arm），与 moon_geocentric 应一致。
        let jde = 2_448_724.5;
        let via_enum = geocentric_ecliptic_longitude(Body::Moon, jde);
        let direct = moon_geocentric(jde).longitude;
        assert!((via_enum - direct).abs() < 1e-12, "enum 与直接调用不一致");
    }

    /// J2000 (T=0)：Ω = 125.0445479°，与 SOFA `iauFaom03` 125.04455501° 差 0.14″（<公式精度）。
    #[test]
    fn lunar_node_at_j2000() {
        let omega = mean_lunar_node(2_451_545.0);
        assert!((omega - 125.044_547_9).abs() < 1e-6, "Ω(J2000) = {omega}");
        // SOFA 实测值
        assert!((omega - 125.044_555_01).abs() < 1e-4, "vs SOFA: {omega}");
    }

    /// 月升交点逆行 18.6 年 ≈ 6798.4 天：`Ω(T) − Ω(T+6798.4d) ≡ 360°`。
    #[test]
    fn lunar_node_retrograde_one_period() {
        let jde = 2_451_545.0;
        let p_days = 360.0 / 1_934.136_184_9 * 36_525.0; // 6798.38 d
        let omega0 = mean_lunar_node(jde);
        let omega1 = mean_lunar_node(jde + p_days);
        let diff = ((omega0 - omega1 + 540.0).rem_euclid(360.0) - 180.0).abs();
        assert!(diff < 0.01, "一个 18.6 年周期应整圈，差 {diff}°");
    }

    /// J2000 (T=0)：Π = 83.3532465°（月平近地点，Meeus p.343 + PyMeeus + NASA 三源一致）。
    #[test]
    fn lunar_perigee_at_j2000() {
        let pi = mean_lunar_perigee(2_451_545.0);
        assert!((pi - 83.353_246_5).abs() < 1e-6, "Π(J2000) = {pi}");
    }

    /// J2000：月孛（远地点） = Π + 180° = 263.3532465°。
    #[test]
    fn lunar_apogee_at_j2000() {
        let a = mean_lunar_apogee(2_451_545.0);
        assert!((a - 263.353_246_5).abs() < 1e-6, "Apogee(J2000) = {a}");
    }

    /// 月近地点顺行 8.85 年：`Π(T+8.85yr) − Π(T) ≡ 360°`，与 18.6 年逆行(Ω)方向相反。
    #[test]
    fn lunar_perigee_prograde_one_period() {
        let jde = 2_451_545.0;
        let p_days = 360.0 / 4_069.013_728_7 * 36_525.0; // ~3232.6 d ≈ 8.85 yr
        let pi0 = mean_lunar_perigee(jde);
        let pi1 = mean_lunar_perigee(jde + p_days);
        let diff = ((pi1 - pi0 + 540.0).rem_euclid(360.0) - 180.0).abs();
        assert!(diff < 0.01, "一个 8.85 年周期应整圈，差 {diff}°");
    }

    /// 月远地点与近地点恒对宫(180°)。
    #[test]
    fn apogee_opposite_perigee() {
        for jde in [2_451_545.0, 2_460_311.5, 2_400_000.0] {
            let pi = mean_lunar_perigee(jde);
            let a = mean_lunar_apogee(jde);
            let diff = ((a - pi - 180.0 + 540.0).rem_euclid(360.0) - 180.0).abs();
            assert!(diff < 1e-12, "Apogee/Perigee 非对宫： {diff}");
        }
    }

    #[test]
    fn moon_position_in_range_over_a_year() {
        // 性质测试：月亮 apparent 黄经/纬应始终落 [0，360) / (−6°，6°)。
        let jde0 = jde_of(2024, 1, 1.0);
        for k in 0..24 {
            let jde = jde0 + f64::from(k) * 15.0;
            let m = moon_geocentric(jde);
            assert!((0.0..360.0).contains(&m.longitude), "λ 越界： {}", m.longitude);
            assert!(m.latitude.abs() < 6.0, "|β| 越界： {}", m.latitude);
            assert!(m.distance_km > 350_000.0 && m.distance_km < 410_000.0, "Δ 越界： {}", m.distance_km);
        }
    }
}
