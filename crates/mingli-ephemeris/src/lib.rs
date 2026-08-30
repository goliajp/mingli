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
        Body::Sun => sun_from(&vsop87d::earth(jde)),
        Body::Moon => moon_geocentric(jde).longitude,
        _ => planet_from(body, jde, &to_rect(vsop87d::earth(jde))),
    }
}

/// 一次算多颗星的地心黄经，地球的日心位置只求一次。
///
/// 逐颗调用 [`geocentric_ecliptic_longitude`] 会把地球那条 VSOP87D 级数重算一遍，
/// 九星就是八遍。实测（本机 release）：地球一次 5.93 µs，八次 47.45 µs，
/// 而整条路合计 264 µs——把它提出来省下 41.5 µs，占 15.7%，且每一位输出都不变。
///
/// `out` 与 `bodies` 一一对应，长度不足的部分不写。
pub fn geocentric_ecliptic_longitudes(bodies: &[Body], jde: f64, out: &mut [f64]) {
    // 只在真有行星要算时才求地球——只问月亮的调用方不该为它付钱。
    let needs_earth = bodies.iter().any(|b| !matches!(b, Body::Moon));
    let earth_sph = needs_earth.then(|| vsop87d::earth(jde));
    let earth_lon = earth_sph.as_ref().map(sun_from);
    let earth = earth_sph.map(to_rect);
    for (slot, &body) in out.iter_mut().zip(bodies.iter()) {
        *slot = match body {
            // `needs_earth` 为真才会走到这两支，故 earth 必已求出。
            Body::Sun => earth_lon.unwrap_or(0.0),
            Body::Moon => moon_geocentric(jde).longitude,
            _ => earth.as_ref().map_or(0.0, |e| planet_from(body, jde, e)),
        };
    }
}

/// 地心太阳 = 地球日心 + 180°。
fn sun_from(earth: &vsop87::SphericalCoordinates) -> f64 {
    (earth.longitude().to_degrees() + 180.0).rem_euclid(360.0)
}

/// 每颗行星的光行时迭代轮数。
///
/// 三轮不是随手取的数，也不该随手改——每一轮都要把该行星的整条 VSOP87D 级数
/// 重算一遍，而那是排一张盘里最贵的一项（七颗行星单轮合计 76.3 µs，三轮 217 µs，
/// 占九星总耗时 264 µs 的 82%）。
///
/// 1900–2100 每两年取一点、七颗行星逐轮量下来的最大位移：
///
/// | | 1→2 轮 | 2→3 轮 | 3→4 轮 |
/// |---|---|---|---|
/// | 水星 | 38.30″ | 0.0037″ | 0.0000018″ |
/// | 金星 | 24.14″ | 0.0014″ | 0.0000010″ |
/// | 火星 | 18.17″ | 0.0008″ | 0.0000011″ |
/// | 木星 |  9.40″ | 0.00009″ | 0 |
/// | 海王 |  3.80″ | 0.000002″ | 0 |
///
/// 也就是说：第二轮非要不可（几十角秒），第三轮值 3.7 毫角秒而要价 72 µs，
/// 第四轮什么也不改。降到两轮能省 27%，代价是每一张已发出去的盘末位都会变——
/// 那是对外契约变更，而 27% 关不上我们与截断级数实现之间三十倍的差距，
/// 所以现在不动它。真要动，先看 `the_third_light_time_pass_is_worth_this_much`
/// 钉住的那个数，再决定这笔交易划不划算。
const LIGHT_TIME_PASSES: usize = 3;

/// 一颗行星在给定地球位置下的地心黄经。
fn planet_from(body: Body, jde: f64, earth: &Rect) -> f64 {
    let mut tau = 0.0;
    let mut lambda = 0.0;
    // 光行时迭代：在「光离开行星的时刻」取其位置。
    for _ in 0..LIGHT_TIME_PASSES {
        let p = to_rect(heliocentric(body, jde - tau));
        let (dx, dy, dz) = (p.x - earth.x, p.y - earth.y, p.z - earth.z);
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        tau = LIGHT_TIME_PER_AU * dist;
        lambda = dy.atan2(dx).to_degrees().rem_euclid(360.0);
    }
    lambda
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

    const PLANETS: [Body; 7] = [
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
    ];

    /// 环形最短差，返回角秒。
    fn arcsec_apart(a: f64, b: f64) -> f64 {
        ((a - b + 180.0).rem_euclid(360.0) - 180.0) * 3600.0
    }

    /// 七颗行星的地心黄经，1950–2050 每 25 年一个历元，逐个对表。
    ///
    /// 此前这里只有一条「经度落在 `[0,360)`」——而产出它的那行末尾就是
    /// `rem_euclid(360.0)`，断言复述了刚刚做过的事，除了 NaN 没有任何东西能让它红。
    /// 也就是说占星／印占／七政四余三片叶吃的行星位置，一个外部锚点都没有。
    ///
    /// 取值两源相合，且两源背后是两套彼此独立的行星理论：
    ///
    /// 1. NASA JPL Horizons（DE441）<https://ssd.jpl.nasa.gov/horizons/>
    ///    `QUANTITIES=31` 的 ObsEcLon —— 当日平分点的**视**位置，含光行时、光线偏折、光行差
    /// 2. IMCCE Miriade（INPOP19）<https://ssp.imcce.fr/webservices/miriade/>
    ///    的 `ephemcc` 服务，J2000 黄道的**天测**位置 —— 含光行时，不含光行差
    ///
    /// 2000-01-01 两源逐颗相差 3.5″ 至 34.7″，正是章动（当日约 −14″）加光行差（至多 20″）
    /// 该有的量；火星在 2024-06-15 上两源差 0.3370°，与 24.5 年岁差 0.3415° 扣掉同一项相符。
    /// 框架关系对得上，所以两源确认的是同一件事。
    ///
    /// 本实现走 VSOP87D，当日平分点、含光行时、不含光行差与章动，因此与 JPL 的视位置
    /// 应差在光行差加章动之内。实测 35 个点最大 37.4″，容差取 60″（1 角分）。
    /// 下游最细的分度是二十七宿的 13°20′，1 角分比它紧三个数量级。
    #[test]
    fn every_planet_matches_the_positions_jpl_publishes_across_a_century() {
        // JPL Horizons ObsEcLon，历元 1950 / 1975 / 2000 / 2025 / 2050 的 01-01 00:00 UT。
        const PUBLISHED: [[f64; 5]; 7] = [
            [299.447_252_3, 287.043_471_0, 271.111_799_4, 259.869_967_7, 270.059_488_2],
            [316.979_477_5, 293.380_913_2, 240.961_401_7, 327.712_098_6, 281.248_037_7],
            [182.211_196_7, 254.959_265_2, 327.575_459_2, 121.917_909_4, 227.714_693_4],
            [306.505_324_9, 343.314_846_5, 25.233_108_6, 73.215_446_5, 121.691_548_0],
            [169.437_421_8, 105.874_436_3, 40.405_837_4, 344.524_052_5, 297.574_279_9],
            [92.682_720_5, 211.877_795_5, 314.784_051_9, 53.635_824_5, 170.732_610_0],
            [197.266_039_2, 250.433_396_1, 303.175_243_2, 357.297_808_0, 53.603_420_9],
        ];
        const YEARS: [i32; 5] = [1950, 1975, 2000, 2025, 2050];
        const TOLERANCE_ARCSEC: f64 = 60.0;

        let mut worst = 0.0f64;
        for (body, published) in PLANETS.iter().zip(PUBLISHED) {
            for (year, expected) in YEARS.iter().zip(published) {
                let ours = geocentric_ecliptic_longitude(*body, jde_of(*year, 1, 1.0));
                let off = arcsec_apart(ours, expected);
                assert!(
                    off.abs() < TOLERANCE_ARCSEC,
                    "{body:?} {year}-01-01：算出 {ours:.6}°，JPL 作 {expected:.6}°，差 {off:.1}″"
                );
                worst = worst.max(off.abs());
            }
        }
        // 实测（2026-08-23）最大 37.4″，出现在土星 1975。逼近容差说明模型变了而不只是抖动。
        assert!(worst < 45.0, "最大偏差 {worst:.1}″，已逼近容差");
    }

    /// 同一批位置的紧口径校验：2000-01-01 时当日平分点与 J2000 几乎重合，而 IMCCE 给的
    /// 是天测位置——跟本实现一样不含光行差。两边于是可以直接比，容差从 1 角分收到 3 角秒。
    ///
    /// 这一条与上一条问的不是同一件事：上一条问「一个世纪里位置都对不对」，这一条问
    /// 「理论本身是不是真跟一套独立星历同级」。实测七颗最大 0.62″（海王星，VSOP87D 截断
    /// 在外行星上最大），水星差 0.003″。
    #[test]
    fn at_j2000_every_planet_matches_an_independent_theory_to_the_arcsecond() {
        // IMCCE Miriade 的 ephemcc / INPOP19，2000-01-01T00:00:00，地心 J2000 黄道天测经度。
        // 原文为度分秒：水 271°07′17.12957″、金 240°58′11.30468″、火 327°34′59.72815″、
        // 木 25°14′07.75378″、土 40°24′24.52219″、天 314°47′33.73699″、海 303°11′04.04710″。
        const IMCCE: [f64; 7] = [
            271.121_424_9,
            240.969_806_9,
            327.583_257_8,
            25.235_487_2,
            40.406_811_7,
            314.792_704_7,
            303.184_457_5,
        ];
        let jde = jde_of(2000, 1, 1.0);
        for (body, expected) in PLANETS.iter().zip(IMCCE) {
            let ours = geocentric_ecliptic_longitude(*body, jde);
            let off = arcsec_apart(ours, expected);
            assert!(
                off.abs() < 3.0,
                "{body:?}：算出 {ours:.7}°，IMCCE 作 {expected:.7}°，差 {off:.3}″"
            );
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

    /// 第三轮光行时值多少角秒——把这个数钉住，好让想省掉它的人先看见代价。
    ///
    /// 本测试自己按两轮与三轮各算一遍，不调 `planet_from`：那个函数的轮数正是被测的东西，
    /// 用它来验它等于什么也没验。
    #[test]
    fn the_third_light_time_pass_is_worth_this_much() {
        fn lambda_with(body: Body, jde: f64, passes: usize) -> f64 {
            let earth = to_rect(vsop87d::earth(jde));
            let (mut tau, mut lambda) = (0.0_f64, 0.0_f64);
            for _ in 0..passes {
                let p = to_rect(heliocentric(body, jde - tau));
                let (dx, dy, dz) = (p.x - earth.x, p.y - earth.y, p.z - earth.z);
                tau = LIGHT_TIME_PER_AU * (dx * dx + dy * dy + dz * dz).sqrt();
                lambda = dy.atan2(dx).to_degrees().rem_euclid(360.0);
            }
            lambda
        }
        let short = |d: f64| {
            let d = d.rem_euclid(360.0);
            if d > 180.0 { d - 360.0 } else { d }
        };
        let bodies = [
            Body::Mercury, Body::Venus, Body::Mars, Body::Jupiter,
            Body::Saturn, Body::Uranus, Body::Neptune,
        ];
        let (mut second, mut third, mut n) = (0.0_f64, 0.0_f64, 0);
        let mut jde = 2_415_020.0_f64; // 1900-01-01
        while jde < 2_488_070.0 {
            // 到 2100
            for &b in &bodies {
                let (a, c, d) = (lambda_with(b, jde, 1), lambda_with(b, jde, 2), lambda_with(b, jde, 3));
                second = second.max(short(c - a).abs() * 3600.0);
                third = third.max(short(d - c).abs() * 3600.0);
                n += 1;
            }
            jde += 733.0;
        }
        assert!(n > 600, "只比了 {n} 组，取样太少");
        // 第二轮非要不可：几十角秒。
        assert!(second > 10.0, "第二轮只值 {second:.4}″——那它可以省，本注释要重写");
        // 第三轮值 3.7 毫角秒。这个上界是实测记录：涨了说明星历那一路变了，
        // 而不是说明这条测试该放宽。
        assert!(
            third < 0.005,
            "第三轮值 {third:.6}″，记录是 0.0037″——变大了就要重新算这笔账"
        );
    }
}
