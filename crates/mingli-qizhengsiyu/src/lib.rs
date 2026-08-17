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

    /// 三套宿度表的 oracle：拿各书**自载的四方小计与周天**对账。
    ///
    /// 这是最硬的一种校验——表若在传抄中崩了一位，小计立刻对不上。
    /// 考据过程里就靠这条抓到两处讹误：维基文库《淮南子》作「東井三十」，
    /// 整表和只有 362¼ 而非 365¼，判为脱字；《宋史》崇天历作「氐十七度」，
    /// 与它自己的「東方七十五度」矛盾，改回十五才合。
    #[test]
    fn every_lodge_table_closes_on_the_totals_its_own_source_reports() {
        use xiudu::{table, Epoch};
        // (纪元, 东, 北, 西, 南, 周天)  单位：万分之一度
        const ORACLE: [(Epoch, u32, u32, u32, u32, u32); 3] = [
            // 《汉书·律历志下》自载「東七十五度／北九十八度／西八十度／南百一十二度」
            (Epoch::Han, 750_000, 980_000, 800_000, 1_120_000, 3_650_000),
            // 《新唐书》卷 028 上大衍历赤道；西 81 南 111（毕觜参鬼四宿改，净值为零）
            (Epoch::Dayan, 750_000, 980_000, 810_000, 1_110_000, 3_650_000),
            // 《元史》卷 054 自载「七十九度二十分／九十三度八十分太／八十三度八十五分／一百八度四十分」
            (Epoch::Shoushi, 792_000, 938_075, 838_500, 1_084_000, 3_652_575),
        ];
        for (epoch, e, n, w, s, total) in ORACLE {
            let t = table(epoch);
            let quad = |k: usize| t[k * 7..k * 7 + 7].iter().sum::<u32>();
            assert_eq!(quad(0), e, "{epoch:?} 东方七宿");
            assert_eq!(quad(1), n, "{epoch:?} 北方七宿");
            assert_eq!(quad(2), w, "{epoch:?} 西方七宿");
            assert_eq!(quad(3), s, "{epoch:?} 南方七宿");
            assert_eq!(t.iter().sum::<u32>(), total, "{epoch:?} 周天");
        }
    }

    /// 汉→唐只动了四宿，且原典自己点了名；汉→元则一宿不剩。
    #[test]
    fn the_tang_table_changes_exactly_the_four_lodges_its_source_names() {
        use xiudu::{degrees_of, table, Epoch, NAMES};
        let (han, dayan) = (table(Epoch::Han), table(Epoch::Dayan));
        let changed: Vec<&str> = (0..28).filter(|&i| han[i] != dayan[i]).map(|i| NAMES[i]).collect();
        assert_eq!(changed, ["毕", "觜", "参", "鬼"], "《新唐书》「其畢、觜觿、參、輿鬼四宿度數，與古不同」");
        // 净值为零，故周天整数部分不变
        assert_eq!(han.iter().sum::<u32>(), dayan.iter().sum::<u32>());
        // 到授时已无一宿与汉制相同
        let same = (0..28).filter(|&i| han[i] == table(Epoch::Shoushi)[i]).count();
        assert_eq!(same, 0, "授时历应无一宿与汉制相同");
        // 觜宿的塌缩：2 → 1 → 0.05
        assert_eq!(degrees_of(Epoch::Han, "觜"), Some(2.0));
        assert_eq!(degrees_of(Epoch::Dayan, "觜"), Some(1.0));
        assert_eq!(degrees_of(Epoch::Shoushi, "觜"), Some(0.05));
    }

    /// 汉代那 ¼ 度余分的两种归属，各自都让整表收在 365¼。
    #[test]
    fn either_home_for_the_quarter_degree_closes_the_circle() {
        use xiudu::{table, Epoch, QUARTER_REMAINDER, NAMES};
        for (label, idx, with_remainder) in QUARTER_REMAINDER {
            let mut t = *table(Epoch::Han);
            assert_eq!(with_remainder - t[idx], 2_500, "{label}：加的应恰是 ¼ 度");
            t[idx] = with_remainder;
            assert_eq!(t.iter().sum::<u32>(), 3_652_500, "{label}：整表应收在 365¼");
        }
        assert_eq!(NAMES[QUARTER_REMAINDER[0].1], "箕");
        assert_eq!(NAMES[QUARTER_REMAINDER[1].1], "斗");
    }

    /// 距度表与本叶的二十八宿名同序，取用接口自洽。
    #[test]
    fn the_lodge_table_lines_up_with_the_mansion_names() {
        use xiudu::{degrees, degrees_of, Epoch, NAMES};
        assert_eq!(NAMES, MANSIONS);
        for (i, name) in NAMES.iter().enumerate() {
            assert_eq!(degrees(Epoch::Han, i), degrees_of(Epoch::Han, name), "{name}");
            assert!(degrees(Epoch::Han, i).is_some_and(|d| d > 0.0), "{name} 的汉制距度应为正");
        }
        assert!(degrees(Epoch::Han, 28).is_none());
        assert!(degrees_of(Epoch::Han, "不存在之宿").is_none());
        assert_eq!(Epoch::default(), Epoch::Han);
    }

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
