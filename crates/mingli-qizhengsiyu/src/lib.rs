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
//! - **十二次**：名与顺序、次 ↔ 十二辰、整宿归次三层多源一致，见 [`erci`] — Det；
//!   **具体度界** 🟡 **Und 不实现**（三统历 / 费直 / 蔡邕 / 大衍历 / 明历五系并存，
//!   受岁差支配不可能统一）。因此本叶给出十二次的对照表，但**不由黄经推落次**。
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

/// 十二次：名、对应十二辰、整宿归属。
///
/// # 为什么只到这三层
///
/// 十二次在古籍里分「名与顺序」「对应辰」「整宿归属」「度界」四层，前三层多源一致、第四层
/// 从来没统一过。度界至少五系并存——《汉书·律历志》三统历、费直《周易分野》、蔡邕
/// 《月令章句》、两唐书的大衍历、《明史》另一套——《晋书·天文志》自己就把前三家并列，
/// 这不是后人拼凑而是原典层面的公开分歧，且受岁差支配，本就不可能收敛。
/// 所以本模块给对照表，**不由黄经推落次**：那一步必须先选一套度界。
///
/// 另有一条常见的坑：《汉书》「视其建而知其次」给的是次 ↔ **月建**，不是次 ↔ **辰位**，
/// 两者互为镜像（星纪之中为冬至、冬至建子，而星纪於辰在丑）。照那句直接推表会全盘错位。
pub mod erci {
    /// 一个次的对照条目。
    #[derive(Debug, Clone, Copy)]
    pub struct Ci {
        /// 次名。
        pub name: &'static str,
        /// 对应十二辰（地支序 0 = 子）。
        pub branch: u8,
        /// 整宿归属（通行表；每宿整归一次的近似，见模块说明）。
        pub mansions: &'static [&'static str],
    }

    /// 十二次对照表，按传统顺序（星纪起）。
    ///
    /// **名与顺序**：《汉书·律历志》《晋书·天文志》《旧唐书》《新唐书》《明史》五处一致。
    ///
    /// **次 ↔ 辰**：两条互相独立的证据链。《晋书·天文志上》逐条「於辰在 X」（陈卓 / 班固传统），
    /// 《旧唐书·天文志下》逐条「X 初起…」（一行《大衍历》传统）——两家的**度数彼此打架而地支全同**，
    /// 恰说明这一层与度界方案无关。第三条旁证：《淮南子·天文训》的太阴辰 → 岁星舍宿，经镜像后逐条吻合。
    ///
    /// **整宿归属**：《淮南子·天文训》（前 2 世纪，经辰镜像还原）与《新唐书》一行表头（8 世纪）
    /// 逐条一致，且等于今日通行表。这是**整宿近似**——四部原典的度界都让若干宿跨次
    /// （女、胃、氐、张、毕、井、柳、轸、尾、斗、危、奎每一个都落在次界上），整宿归谁只能是约定。
    ///
    /// 🟡 未入码：各家度界（见模块说明）；《汉书》与《晋书》在鹑火 / 鹑尾分界另有一度之差
    /// （张十七 / 十八 对 张十六 / 十七），两版求和都恰是 365 度，算术判不了，需点校本校勘记。
    /// 中气对应只见《汉书·律历志》一处（且用汉代节气次序，立春→惊蛰→雨水→春分），单源不写。
    pub const TWELVE_CI: [Ci; 12] = [
        Ci { name: "星纪", branch: 1, mansions: &["斗", "牛"] },
        Ci { name: "玄枵", branch: 0, mansions: &["女", "虚", "危"] },
        Ci { name: "娵訾", branch: 11, mansions: &["室", "壁"] },
        Ci { name: "降娄", branch: 10, mansions: &["奎", "娄"] },
        Ci { name: "大梁", branch: 9, mansions: &["胃", "昴", "毕"] },
        Ci { name: "实沈", branch: 8, mansions: &["觜", "参"] },
        Ci { name: "鹑首", branch: 7, mansions: &["井", "鬼"] },
        Ci { name: "鹑火", branch: 6, mansions: &["柳", "星", "张"] },
        Ci { name: "鹑尾", branch: 5, mansions: &["翼", "轸"] },
        Ci { name: "寿星", branch: 4, mansions: &["角", "亢"] },
        Ci { name: "大火", branch: 3, mansions: &["氐", "房", "心"] },
        Ci { name: "析木", branch: 2, mansions: &["尾", "箕"] },
    ];

    /// 某宿在整宿近似下属哪个次；宿名不在二十八宿内则 `None`。
    #[must_use]
    pub fn ci_of_mansion(mansion: &str) -> Option<&'static Ci> {
        TWELVE_CI.iter().find(|c| c.mansions.contains(&mansion))
    }
}

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

    /// 十二次对照表的 oracle：三层各有各的来源，逐层钉。
    #[test]
    fn the_twelve_ci_table_holds_on_all_three_layers() {
        use erci::{ci_of_mansion, TWELVE_CI};
        // 一、名与顺序：五部正史一致，星纪起、析木末
        let names: Vec<&str> = TWELVE_CI.iter().map(|c| c.name).collect();
        assert_eq!(
            names,
            ["星纪", "玄枵", "娵訾", "降娄", "大梁", "实沈", "鹑首", "鹑火", "鹑尾", "寿星", "大火", "析木"]
        );

        // 二、次 ↔ 辰：星纪丑起，**次序与辰序逆行**（次往前一格、辰往后一格）
        assert_eq!(TWELVE_CI[0].branch, 1, "星纪於辰在丑");
        for pair in TWELVE_CI.windows(2) {
            let (a, b) = (pair[0].branch, pair[1].branch);
            assert_eq!(b, (a + 11) % 12, "{} → {} 的辰应逆行一格", pair[0].name, pair[1].name);
        }
        // 十二辰各用一次，无重无漏
        let mut branches: Vec<u8> = TWELVE_CI.iter().map(|c| c.branch).collect();
        branches.sort_unstable();
        assert_eq!(branches, (0..12).collect::<Vec<u8>>());

        // 三、整宿归属：二十八宿恰好被十二次瓜分，不重不漏
        let mut all: Vec<&str> = TWELVE_CI.iter().flat_map(|c| c.mansions.iter().copied()).collect();
        assert_eq!(all.len(), 28, "整宿归次应覆盖全部二十八宿");
        all.sort_unstable();
        let mut uniq = all.clone();
        uniq.dedup();
        assert_eq!(uniq.len(), 28, "不该有宿被归进两个次");
        let mut canonical = MANSIONS.to_vec();
        canonical.sort_unstable();
        assert_eq!(all, canonical, "归次用的宿名应与本叶的二十八宿表一致");

        // 反查
        assert_eq!(ci_of_mansion("斗").map(|c| c.name), Some("星纪"));
        assert_eq!(ci_of_mansion("柳").map(|c| c.name), Some("鹑火"));
        assert_eq!(ci_of_mansion("觜").map(|c| c.name), Some("实沈"));
        assert!(ci_of_mansion("不存在之宿").is_none());
    }

    /// 《尔雅·释天》给的是标志宿而非次界，且**连十二个都没给全**——
    /// 实沈 / 鹑首 / 鹑尾三个次名在《尔雅》全书零出现，玄枵 / 大梁 / 鹑火只给单宿标志。
    /// 这条测试把「不能拿《尔雅》补全十二次」这个判断固定下来，防止日后有人照它改表。
    #[test]
    fn the_erya_marker_stars_are_a_subset_and_cannot_fill_the_table() {
        use erci::ci_of_mansion;
        // 《尔雅》能对上的（标志宿落在通行整宿表的同一个次里）
        for (mansion, ci) in [("角", "寿星"), ("斗", "星纪"), ("虚", "玄枵"), ("室", "娵訾"),
                              ("奎", "降娄"), ("昴", "大梁"), ("柳", "鹑火"), ("箕", "析木")] {
            assert_eq!(ci_of_mansion(mansion).map(|c| c.name), Some(ci), "《尔雅》{mansion} → {ci}");
        }
        // 《尔雅》「大辰 = 房心尾」与通行整宿表「大火 = 氐房心」不合：尾归析木
        assert_eq!(ci_of_mansion("尾").map(|c| c.name), Some("析木"), "尾在通行表归析木，非大火");
    }

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
