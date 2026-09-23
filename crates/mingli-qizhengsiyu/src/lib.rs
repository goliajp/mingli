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
//! - **28 宿距度（古制不等长）**：三套原典表已入码，见 [`xiudu`] — Det。
//!   **宿度不是常数**，随纪元而变（见该模块说明），故按纪元分表而非单表。


#[cfg(feature = "port")]
mod engine;
#[cfg(feature = "port")]
pub use engine::QizhengsiyuEngine;

use mingli_astro::Moment;
use mingli_ephemeris::{
    geocentric_ecliptic_longitude, mean_lunar_apogee, mean_lunar_node, Body,
};
use mingli_ganzhi::{day_ganzhi, GanZhi};
#[cfg(feature = "serde")]
use serde::Serialize;

/// 七政四余十体标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
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

/// 二十八宿的**距度**（相邻两距星的赤经差），按纪元分表。
///
/// # 宿度会变，而且变得很厉害
///
/// 距度定义为**相邻两颗距星的赤经差**。赤经在岁差下的变化率 `dα/dt = m + n·sinα·tanδ`
/// 含赤纬项，每颗星速率不同，于是赤经**差**必然漂移——赤纬相差 60° 的两星可差到约
/// 6 度／千年。（黄经的变化率对所有恒星是同一常数，所以黄道距度才近似恒定。）
///
/// 这不是我们的推断，是原典自己的规定。三部正史的历法各自写着同一句话：
///
/// - 《新唐书》大衍历：「當據歲差，每移一度，各依術算，使得當時度分」
/// - 《宋史》纪元历：「如考唐，用唐所測；考古，用古所測：即各得當時宿度」
/// - 《元史》授时历：「若考往古，即用當時宿度為準」
///
/// 最极端的例子是觜宿：汉 2 度 → 唐 1 度 → 宋崇宁半度 → 元至元 **0.05 度** →
/// 明崇祯**变负**（《明史》卷三十一：「今測之，不啻無分，且侵入參宿二十四分」），
/// 觜参两宿的先后次序整个翻转，乾隆十七年靠**改换距星**才恢复传统次序。
/// 任何假设「宿度恒正」或「宿序恒定」的算法都会在这两宿上崩。
///
/// # 未入码
///
/// 🟡 明清改 360 度制（《明史》卷二十五「赤道宿度周天三百六十度，每度六十分」，
/// 崇祯历书新法起、清顺治元年定案），与古度制不可直接逐行对齐，且觜参次序已翻转，
/// 本模块只收古度制三表。🟡 黄道距度另有一套完全不同的数（《后汉书》贾逵黄道铜仪、
/// 大衍黄道、授时黄道），同代赤黄两套数差异极大，混用会静默出错，本模块只收赤道。
pub mod xiudu {
    /// 距度的纪元。宿度随岁差而变，取哪一套要看算的是哪个时代。
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum Epoch {
        /// 汉代赤道宿度。**两个独立原典 27/28 逐宿相同**：
        /// 《淮南子·天文训》（约前 139）与《汉书·律历志下》（约公元 100）。
        /// 唯一差异是那 ¼ 度余分挂在谁身上——淮南子挂箕（箕 11¼，整表和 365¼），
        /// 汉书不挂（整数和 365，余分 385/1539 循四分历「斗分」传统留给斗）。
        /// 本表取整数读法；¼ 度余分的归属见 [`QUARTER_REMAINDER`]。
        #[default]
        Han,
        /// 唐开元十二年（724）大衍历赤道宿度，《新唐书》卷 028 上。
        ///
        /// 原典自己点名了汉→唐变的是哪四宿：「其畢、觜觿、參、輿鬼四宿度數，與古不同。
        /// 依天以儀測定，用為常數」——毕 +1、觜 −1、参 +1、鬼 −1，净值为零，其余 24 宿逐字未动。
        /// 《宋史》卷 072 崇天历独立复述同一句话，是第二个见证。
        Dayan,
        /// 元至元十七年（1280）授时历赤道宿度，《元史》卷 054（百分制，1 度 = 100 分）。
        ///
        /// 郭守敬新制浑仪实测。四方小计与周天 365.2575 零误差；
        /// 《元史》卷 052 历议「至元所測」栏是另一份文献，逐条互证。
        /// **此时已无一宿与汉制相同。**
        Shoushi,
    }

    /// 二十八宿名，与 [`super::MANSIONS`] 同序（角起、轸末，按四象分组）。
    pub const NAMES: [&str; 28] = super::MANSIONS;

    /// 距度，单位 **万分之一度**（整数存放，四方小计与周天可零误差对账）。
    ///
    /// 索引与 [`NAMES`] 一致。取用请走 [`degrees`] 或 [`table`]。
    const HAN: [u32; 28] = [
        120_000, 90_000, 150_000, 50_000, 50_000, 180_000, 110_000, // 角亢氐房心尾箕 东 75
        260_000, 80_000, 120_000, 100_000, 170_000, 160_000, 90_000, // 斗牛女虚危室壁 北 98
        160_000, 120_000, 140_000, 110_000, 160_000, 20_000, 90_000, // 奎娄胃昴毕觜参 西 80
        330_000, 40_000, 150_000, 70_000, 180_000, 180_000, 170_000, // 井鬼柳星张翼轸 南 112
    ];
    const DAYAN: [u32; 28] = [
        120_000, 90_000, 150_000, 50_000, 50_000, 180_000, 110_000, // 东 75
        260_000, 80_000, 120_000, 100_000, 170_000, 160_000, 90_000, // 北 98
        160_000, 120_000, 140_000, 110_000, 170_000, 10_000, 100_000, // 毕觜参改 西 81
        330_000, 30_000, 150_000, 70_000, 180_000, 180_000, 170_000, // 鬼改 南 111
    ];
    const SHOUSHI: [u32; 28] = [
        121_000, 92_000, 163_000, 56_000, 65_000, 191_000, 104_000, // 东 79.20
        252_000, 72_000, 113_500, 89_575, 154_000, 171_000, 86_000, // 北 93.8075（虚带「太」= ¾ 分）
        166_000, 118_000, 156_000, 113_000, 174_000, 500, 111_000, // 西 83.85（觜只剩 0.05 度）
        333_000, 22_000, 133_000, 63_000, 172_500, 187_500, 173_000, // 南 108.40
    ];

    /// 汉代那 ¼ 度余分的两种归属——这是真实的原典分歧，不替调用方选边。
    ///
    /// `(纪元读法, 宿索引, 该宿的距度含余分后的万分度)`。
    pub const QUARTER_REMAINDER: [(&str, usize, u32); 2] = [
        ("淮南子：归箕", 6, 112_500),  // 箕 11¼，整表和 365¼
        ("四库/开元占经：归斗（斗分）", 7, 262_500), // 斗 26¼，北方 98¼
    ];

    /// 某纪元的整张距度表（万分之一度）。
    #[must_use]
    pub const fn table(epoch: Epoch) -> &'static [u32; 28] {
        match epoch {
            Epoch::Han => &HAN,
            Epoch::Dayan => &DAYAN,
            Epoch::Shoushi => &SHOUSHI,
        }
    }

    /// 某宿在某纪元的距度（度）。宿索引 0..28，越界返回 `None`。
    #[must_use]
    pub fn degrees(epoch: Epoch, mansion: usize) -> Option<f64> {
        table(epoch).get(mansion).map(|&v| f64::from(v) / 10_000.0)
    }

    /// 某宿名在某纪元的距度（度）；宿名不在二十八宿内返回 `None`。
    #[must_use]
    pub fn degrees_of(epoch: Epoch, name: &str) -> Option<f64> {
        NAMES.iter().position(|&n| n == name).and_then(|i| degrees(epoch, i))
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
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
        // `+ 180` 改成 `- 180` 是等价变异：模 360 下两者相同。
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
mod tests;
