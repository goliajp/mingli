//! L3 叶（B 族 / 中国本土星占）：七政四余(Qīzhèng Sìyú)。
//!
//! 「七政」= 日、月、水、金、火、木、土（七大可观测天体）；
//! 「四余」= 罗㬋（月升交点）、计都（月降交点）、月孛（月平远地点）、紫炁（虚星）。
//!
//! 本叶承接 [`mingli_ephemeris`] 算出十体地心黄经，落 30° 等分十二宫(西洋 sign 名，
//! 30° 等分天文公认无歧义)，配 28 宿值日（JDN 周期，沿用 `mingli_zeri::mansion` 范式）。
//!
//! # 诚实边界（强权威可入码）
//!
//! - **罗㬋** = 月平升交点 Ω（Meeus AA 第 47 章 eq 47.7，精度 ~0.5″） — Det
//! - **计都** = Ω + 180°(汤若望《时宪历》之后通行近代/印度对位；沈括《梦溪笔谈》古法
//!   计都=月远地点，本算法不采，doc 仅注) — Det
//! - **月孛** = 月平远地点 = Π_perigee + 180°(Meeus AA p.343，与 PyMeeus/NASA GSFC/
//!   soniakeys 三源系数字符级一致) — Det
//! - **紫炁** = 🟡 **Und 不实现**。中文维基明文「找不著对应的天文现象」；五种互不兼容定义
//!   （28 年闰余虚星/月近地点/月轨中点/木余气/天狼星）无任何来源给可代入时间的公式，
//!   swisseph 等主流星历库均不提供。诚实标 Und。
//! - **十二次落宫**（星纪/玄枵/娵訾...）： 🟡 **Und 不实现**。《尔雅》（标志宿） /
//!   《汉书·律历志》（度数，多宿跨次） / 通行表（每宿整归一次） 三种性质不同，源间分歧实质
//!   （尤其斗/牛/女归属、大火 = 房心尾 vs 氐房心），不强编。
//! - **28 宿分黄道**（每宿不等长古制）： 🟡 **Und 不实现**。古制每宿距度由观测得，
//!   有岁差需校正，涉大查表；本叶只做 28 宿**值日**（JDN 周期，无歧义）。


mod engine;
pub use engine::QizhengsiyuEngine;

use mingli_astro::Moment;
use mingli_ephemeris::{
    geocentric_ecliptic_longitude, mean_lunar_apogee, mean_lunar_node, Body,
};
use mingli_ganzhi::{day_ganzhi, GanZhi};
use serde::Serialize;

/// 七政四余十体标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Star {
    /// 太阳 (Sun) — 七政之首。
    Sun,
    /// 太阴/月亮 (Moon) — 七政。
    Moon,
    /// 辰星/水星 (Mercury) — 七政。
    Mercury,
    /// 太白/金星 (Venus) — 七政。
    Venus,
    /// 荧惑/火星 (Mars) — 七政。
    Mars,
    /// 岁星/木星 (Jupiter) — 七政。
    Jupiter,
    /// 镇星/土星 (Saturn) — 七政。
    Saturn,
    /// 罗㬋 （Luohou，月平升交点） — 四余。
    Luohou,
    /// 计都 （Jidu，月平降交点 = 罗㬋 + 180°，通行近代/印度对位） — 四余。
    Jidu,
    /// 月孛 （Yuebo，月平远地点 = 月平近地点 + 180°） — 四余。
    Yuebo,
}

impl Star {
    /// 中文本名（七政沿汉以来本名，四余沿通行）。
    #[must_use]
    pub fn chinese_name(self) -> &'static str {
        match self {
            Self::Sun => "太阳",
            Self::Moon => "太阴",
            Self::Mercury => "辰星",
            Self::Venus => "太白",
            Self::Mars => "荧惑",
            Self::Jupiter => "岁星",
            Self::Saturn => "镇星",
            Self::Luohou => "罗㬋",
            Self::Jidu => "计都",
            Self::Yuebo => "月孛",
        }
    }

    /// 类别：`true` = 七政（可观测），`false` = 四余（虚/计算位）。
    #[must_use]
    pub fn is_qizheng(self) -> bool {
        matches!(
            self,
            Self::Sun
                | Self::Moon
                | Self::Mercury
                | Self::Venus
                | Self::Mars
                | Self::Jupiter
                | Self::Saturn
        )
    }
}

/// 十体顺序（七政前置、四余后置）。
pub const STARS: [Star; 10] = [
    Star::Sun,
    Star::Moon,
    Star::Mercury,
    Star::Venus,
    Star::Mars,
    Star::Jupiter,
    Star::Saturn,
    Star::Luohou,
    Star::Jidu,
    Star::Yuebo,
];

/// 黄道十二宫（回归 sign，30° 等分，与西洋占星共用语义）。索引 0 = 白羊。
pub const SIGNS: [&str; 12] = [
    "白羊", "金牛", "双子", "巨蟹", "狮子", "处女", "天秤", "天蝎", "射手", "摩羯", "水瓶",
    "双鱼",
];

/// 二十八宿值日轮转有序名（角起、轸末；与 `mingli_zeri::mansion::MANSIONS` 同）。
pub const MANSIONS: [&str; 28] = [
    "角", "亢", "氐", "房", "心", "尾", "箕", // 东方苍龙
    "斗", "牛", "女", "虚", "危", "室", "壁", // 北方玄武
    "奎", "娄", "胃", "昴", "毕", "觜", "参", // 西方白虎
    "井", "鬼", "柳", "星", "张", "翼", "轸", // 南方朱雀
];

/// 二十八宿值日相位偏移：`index = (JDN + 11) mod 28`（角=0）。
///
/// 与 `mingli_zeri::mansion::OFFSET` 同，跨 5 个独立锚点（341 年）交叉验证。
pub const MANSION_OFFSET: i64 = 11;

/// 给定黄经返回 12 宫索引 0..12（白羊=0）。
#[must_use]
pub fn sign_of(longitude: f64) -> usize {
    let i = (longitude.rem_euclid(360.0) / 30.0).floor();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "i ∈ [0，12)，窄化到 usize 安全"
    )]
    let i = i as usize;
    i % 12
}

/// 给定民用日 JDN 返回 28 宿值日索引 0..28（角=0）。
#[must_use]
pub fn mansion_for_jdn(jdn: i64) -> usize {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "rem_euclid(28) ∈ [0，28)，窄化安全"
    )]
    let i = (jdn + MANSION_OFFSET).rem_euclid(28) as usize;
    i
}

/// 单颗星的排盘条目。
#[derive(Debug, Clone, Serialize)]
pub struct StarPosition {
    /// 标识（序列化为小写名）。
    pub star: Star,
    /// 中文本名。
    pub name: &'static str,
    /// 是否七政(`true`)/ 四余(`false`)。
    pub is_qizheng: bool,
    /// 地心黄道经度（度，`[0, 360)`；太阳/行星 mean，月亮 apparent，余三星 mean）。
    pub longitude: f64,
    /// 12 宫索引 0..12（白羊=0）。
    pub sign: usize,
    /// 12 宫中文名（白羊/金牛/...）。
    pub sign_name: &'static str,
    /// 宫内度数 0..30(`longitude − sign × 30°`)。
    pub degree_in_sign: f64,
}

/// 七政四余完整排盘。
#[derive(Debug, Clone, Serialize)]
pub struct QizhengsiyuChart {
    /// 十体位置（七政前置 7 颗 + 四余后置 3 颗；紫炁 🟡 不输出）。
    pub stars: Vec<StarPosition>,
    /// 当日 28 宿值日索引 0..28。
    pub mansion: usize,
    /// 当日 28 宿值日中文名。
    pub mansion_name: &'static str,
    /// 日柱干支（便于跟其余中国术数关联）。
    pub day_ganzhi: String,
}

/// 由共享时刻 [`Moment`] 计算七政四余排盘。
#[must_use]
pub fn compute_at(m: &Moment) -> QizhengsiyuChart {
    let stars = STARS.iter().map(|&s| star_position(s, m.jde)).collect();
    let mansion = mansion_for_jdn(m.civil_day);
    let gz: GanZhi = day_ganzhi(m.civil_day);
    QizhengsiyuChart {
        stars,
        mansion,
        mansion_name: MANSIONS[mansion],
        day_ganzhi: gz.to_string(),
    }
}

fn star_position(star: Star, jde: f64) -> StarPosition {
    let lon = match star {
        Star::Sun => geocentric_ecliptic_longitude(Body::Sun, jde),
        Star::Moon => geocentric_ecliptic_longitude(Body::Moon, jde),
        Star::Mercury => geocentric_ecliptic_longitude(Body::Mercury, jde),
        Star::Venus => geocentric_ecliptic_longitude(Body::Venus, jde),
        Star::Mars => geocentric_ecliptic_longitude(Body::Mars, jde),
        Star::Jupiter => geocentric_ecliptic_longitude(Body::Jupiter, jde),
        Star::Saturn => geocentric_ecliptic_longitude(Body::Saturn, jde),
        Star::Luohou => mean_lunar_node(jde),
        Star::Jidu => (mean_lunar_node(jde) + 180.0).rem_euclid(360.0),
        Star::Yuebo => mean_lunar_apogee(jde),
    };
    let sign = sign_of(lon);
    #[allow(
        clippy::cast_precision_loss,
        reason = "sign ∈ [0，12)，f64 精确表示 0..12"
    )]
    let degree_in_sign = lon - (sign as f64) * 30.0;
    StarPosition {
        star,
        name: star.chinese_name(),
        is_qizheng: star.is_qizheng(),
        longitude: lon,
        sign,
        sign_name: SIGNS[sign],
        degree_in_sign,
    }
}

/// 由本地民用时刻入参排盘（构造 [`Moment`] 的薄壳）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, hour: u32, minute: u32, tz: f64) -> QizhengsiyuChart {
    compute_at(&Moment::new(year, month, day, hour, minute, tz))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moment(y: i32, mo: u32, d: u32, h: u32, mi: u32, tz: f64) -> Moment {
        Moment::new(y, mo, d, h, mi, tz)
    }

    /// 十体顺序固定：7 七政在前 + 3 四余在后（紫炁 🟡 不入）。
    #[test]
    fn star_list_well_formed() {
        assert_eq!(STARS.len(), 10);
        assert!(STARS[..7].iter().all(|s| s.is_qizheng()));
        assert!(STARS[7..].iter().all(|s| !s.is_qizheng()));
        // 不含紫炁（诚实标 🟡）。Star enum 也无 Ziqi 变体。
    }

    /// 名表长度与 const。
    #[test]
    fn name_tables_len() {
        assert_eq!(SIGNS.len(), 12);
        assert_eq!(MANSIONS.len(), 28);
        assert_eq!(MANSION_OFFSET, 11);
    }

    /// `sign_of` 边界：0° = 白羊、30° = 金牛、359.999° = 双鱼，wrap 360 → 白羊。
    #[test]
    fn sign_of_boundaries() {
        assert_eq!(sign_of(0.0), 0);
        assert_eq!(sign_of(29.999), 0);
        assert_eq!(sign_of(30.0), 1);
        assert_eq!(sign_of(359.99), 11);
        assert_eq!(sign_of(360.0), 0);
        assert_eq!(sign_of(720.0), 0);
        assert_eq!(sign_of(-30.0), 11);
    }

    /// 28 宿值日：2026-06-14 = 昴（idx 17，与 zeri 校验值一致）。
    #[test]
    fn mansion_2026_06_14() {
        let jdn = mingli_astro::civil_day_number(2026, 6, 14);
        let i = mansion_for_jdn(jdn);
        assert_eq!(MANSIONS[i], "昴");
    }

    /// 性质：罗㬋/计都恒 180° 对宫。
    #[test]
    fn luohou_opposite_jidu() {
        for (y, mo, d) in [(2024, 1, 1), (1990, 6, 15), (2000, 1, 1)] {
            let m = moment(y, mo, d, 12, 0, 0.0);
            let c = compute_at(&m);
            let lo = c.stars.iter().find(|s| s.star == Star::Luohou).unwrap();
            let ji = c.stars.iter().find(|s| s.star == Star::Jidu).unwrap();
            let diff = ((ji.longitude - lo.longitude - 180.0 + 540.0).rem_euclid(360.0) - 180.0)
                .abs();
            assert!(diff < 1e-9, "{y}-{mo}-{d}： 罗/计非 180° 对宫 (diff={diff})");
        }
    }

    /// 性质：月孛黄经在 [0， 360) 且与月平近地点恒差 180°（已在 ephemeris 测过，这里走完整路径）。
    #[test]
    fn yuebo_in_range() {
        let m = moment(2024, 6, 15, 14, 30, 8.0);
        let c = compute_at(&m);
        let y = c.stars.iter().find(|s| s.star == Star::Yuebo).unwrap();
        assert!((0.0..360.0).contains(&y.longitude), "月孛越界： {}", y.longitude);
    }

    /// 1990-06-15 14：30 CST 七政四余完整排盘 oracle（全字段类型 + 值范围 + 关键值）。
    ///
    /// - 太阳：在双子座（已由 astrology 校验）
    /// - 日柱：辛亥（与 bazi/zeri 同源）
    /// - 罗㬋 ≈ Ω(1990-06-15) ≈ 月升交点位置（逆行约 250°/yr 自 J2000 起 ~10 年）
    /// - 月孛 ≈ Π(1990-06-15)+180° ≈ 月远地点位置（顺行约 40°/yr）
    #[test]
    fn sample_1990_06_15_full() {
        let m = moment(1990, 6, 15, 14, 30, 8.0);
        let c = compute_at(&m);

        assert_eq!(c.stars.len(), 10);
        for sp in &c.stars {
            assert!((0.0..360.0).contains(&sp.longitude));
            assert!(sp.sign < 12);
            assert!((0.0..30.0).contains(&sp.degree_in_sign));
            assert_eq!(sp.sign_name, SIGNS[sp.sign]);
        }

        // 太阳在双子（与 astrology 校验值一致）。
        let sun = c.stars.iter().find(|s| s.star == Star::Sun).unwrap();
        assert_eq!(sun.sign_name, "双子", "太阳座 {} @ {:.2}°", sun.sign_name, sun.longitude);

        // 日柱辛亥（与 bazi 校验一致）。
        assert_eq!(c.day_ganzhi, "辛亥");
        // 28 宿值日落在某一宿。
        assert!(MANSIONS.contains(&c.mansion_name));
    }

    /// `compute` 入口与 `compute_at` 等价。
    #[test]
    fn compute_equals_compute_at() {
        let a = compute(1990, 6, 15, 14, 30, 8.0);
        let m = moment(1990, 6, 15, 14, 30, 8.0);
        let b = compute_at(&m);
        assert_eq!(a.stars.len(), b.stars.len());
        for (x, y) in a.stars.iter().zip(b.stars.iter()) {
            assert!((x.longitude - y.longitude).abs() < 1e-12);
        }
        assert_eq!(a.mansion, b.mansion);
        assert_eq!(a.day_ganzhi, b.day_ganzhi);
    }

    /// `Star::chinese_name` 全 10 变体唯一且非空。
    #[test]
    fn chinese_names_unique() {
        let mut names: Vec<&str> = STARS.iter().map(|s| s.chinese_name()).collect();
        names.sort_unstable();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "中文名有重复");
        assert!(names.iter().all(|n| !n.is_empty()));
    }

    /// `is_qizheng` 与 STARS 切片划分一致。
    #[test]
    fn is_qizheng_partition() {
        for &s in &STARS[..7] {
            assert!(s.is_qizheng(), "{s:?} 应属七政");
        }
        for &s in &STARS[7..] {
            assert!(!s.is_qizheng(), "{s:?} 应属四余");
        }
    }
}
