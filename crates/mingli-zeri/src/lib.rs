//! L3 叶（A 族 / 确定性）：择日学的**循环要素**。
//!
//! 本 crate 实现择日里**纯周期 / 纯查表**的四类要素，不碰随流派分歧的神煞宜忌（那属 🟡 释义层）：
//!
//! 1. **建除十二神**（[`jianchu`]）——`建·除·满·平·定·执·破·危·成·收·开·闭` 固定成环，
//!    其相位由「日支 − 月建支」在 `Z₁₂` 上定：正月（寅月）建寅、二月（卯月）建卯……即
//!    *月建之日为「建」，逐日顺行十二神*。月建支由太阳过「节」确定（与八字月柱同源）。
//! 2. **二十八宿值日**（[`mansion`]）——二十八宿逐日轮值，是与节气/朔望无关的连续 `Z₂₈` 计数，
//!    相位由经多源交叉验证的历日偏移确定（见 [`mansion::OFFSET`]）。
//! 3. **彭祖百忌**（[`pengzu`]）——10 干句 + 12 支句的传世口诀（《彭祖百忌歌》），由日柱
//!    干支拼出双句行为禁忌。表为多源完全一致的中文古文，纯查表，无流派分歧。
//! 4. **天乙贵人**（[`tianyi`]）——10 日干→双贵人地支，《三命通会》「甲戊庚牛羊，乙己鼠猴乡，
//!    丙丁猪鸡位，六辛逢虎马，壬癸兔蛇藏」通行版口诀，五源一致。🟡 另有「甲戊兼牛羊，庚辛逢虎马」
//!    一系，出处与实情见 [`tianyi`] doc；本 crate 只取通行版。
//!
//! 前两者「时间 → 模运算 → 值日」对同一日完全确定；后两者「干支 → 查表 → 禁忌/贵人」对同一干支
//! 完全确定。


mod engine;
pub use engine::ZeriEngine;

use mingli_astro::Moment;
use serde::Serialize;

/// 建除十二神的通行分档，出自口诀「**建满平收黑，除危定执黄，成开皆可用，破闭不可当**」。
///
/// 🟡 另有一说把「成 · 开」并入黄道合称**六黄道**（除危定执成开）；本 crate 依口诀原文分四档，
/// 两说只在「成 · 开」算不算黄道上有别，其余一致。**分档只是通行的粗筛，不是断语**——
/// 具体某事宜忌还要看事类，那部分各家出入大，交释义层。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum DayGrade {
    /// 黄道：除 · 危 · 定 · 执。
    Huang,
    /// 可用：成 · 开。
    Usable,
    /// 黑道：建 · 满 · 平 · 收。
    Hei,
    /// 不可当：破 · 闭。
    Avoid,
}

impl DayGrade {
    /// 中文标签。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            DayGrade::Huang => "黄道",
            DayGrade::Usable => "可用",
            DayGrade::Hei => "黑道",
            DayGrade::Avoid => "不可当",
        }
    }

    /// 排序权重：数值越小越优先（黄道 0 → 不可当 3）。
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            DayGrade::Huang => 0,
            DayGrade::Usable => 1,
            DayGrade::Hei => 2,
            DayGrade::Avoid => 3,
        }
    }
}

/// 由建除环位 `0..12`（0=建）取其分档。
#[must_use]
pub const fn day_grade(jianchu_pos: u8) -> DayGrade {
    match jianchu_pos % 12 {
        // 除(1) 危(7) 定(4) 执(5)
        1 | 4 | 5 | 7 => DayGrade::Huang,
        // 成(8) 开(10)
        8 | 10 => DayGrade::Usable,
        // 破(6) 闭(11)
        6 | 11 => DayGrade::Avoid,
        // 建(0) 满(2) 平(3) 收(9)
        _ => DayGrade::Hei,
    }
}

/// 建除十二神（次序即逐日顺行方向）。`建` 落在「日支 == 月建支」之日。
pub const JIANCHU: [&str; 12] = [
    "建", "除", "满", "平", "定", "执", "破", "危", "成", "收", "开", "闭",
];

/// 由太阳视黄经定**月建地支**（0..11，子=0）。以「节」换月：立春(λ=315°)起寅月。
/// 与八字月柱同源（`mingli_bazi` 亦用此式）。
#[must_use]
pub fn month_branch(sun_longitude: f64) -> u8 {
    let s = ((sun_longitude - 315.0).rem_euclid(360.0) / 30.0).floor();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "s ∈ 0..12，窄化到 u8 受控安全"
    )]
    let s = s as u8;
    (2 + s) % 12
}

/// 建除十二神模块下的纯函数与说明。
pub mod jianchu {
    /// 给定月建支与日支（皆 0..11，子=0），返回建除十二神的环位 `0..12`（0=建）。
    #[must_use]
    pub fn position(month_branch: u8, day_branch: u8) -> u8 {
        (12 + day_branch - month_branch) % 12
    }
}

/// 二十八宿值日模块。
pub mod mansion {
    /// 二十八宿值日轮转有序名（角起、轸末，周而复始）。
    ///
    /// 七曜配宿（角木、亢金……）是各宿的固定属性，与此**值日轮转**相互独立；本表给的是值日顺序。
    pub const MANSIONS: [&str; 28] = [
        "角", "亢", "氐", "房", "心", "尾", "箕", // 东方苍龙
        "斗", "牛", "女", "虚", "危", "室", "壁", // 北方玄武
        "奎", "娄", "胃", "昴", "毕", "觜", "参", // 西方白虎
        "井", "鬼", "柳", "星", "张", "翼", "轸", // 南方朱雀
    ];

    /// 值日相位偏移：`index = (JDN + OFFSET) mod 28`（角=0），其中 `OFFSET = 11`。
    ///
    /// 该偏移由多个历日锚点反解并交叉验证（跨 341 年、5 个独立来源，含 1685 贞享改历历元
    /// 「正月朔=星宿」与现代历注），绑定**天文 JDN（正午约定）**——与本项目民用日序一致。
    /// 28 = 4×7 故值日宿与七曜/星期严格同相位咬合（房虚星昴恒为日曜等）。
    pub const OFFSET: i64 = 11;

    /// 给定民用日序（JDN），返回二十八宿值日的下标 `0..28`（角=0）。
    #[must_use]
    pub fn index_for_jdn(jdn: i64) -> usize {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "rem_euclid(28) ∈ 0..28，窄化安全"
        )]
        let i = (jdn + OFFSET).rem_euclid(28) as usize;
        i
    }
}

/// 彭祖百忌（《彭祖百忌歌》传世口诀，多源完全一致）。
///
/// 来源：《钦定协纪辨方书》（乾隆官修历书） / 《玉匣记》 / 港台《通胜》 / 中国万年历通用版。
/// 全 22 句皆中文古文口诀（非数值表/非吉凶判断），由日柱干支拼出「干句 + 支句」两条行为禁忌。
/// 无流派分歧——多源版本仅极少处用字差异（如「子不问卜 自惹祸殃」一作「神鬼不安」），取通胜流传最广本。
pub mod pengzu {
    /// 10 天干句（甲..癸，顺序与 [`mingli_ganzhi::GanZhi::stem`] 一致）。
    pub const GAN: [&str; 10] = [
        "甲不开仓 财物耗散",
        "乙不栽植 千株不长",
        "丙不修灶 必见灾殃",
        "丁不剃头 头必生疮",
        "戊不受田 田主不祥",
        "己不破券 二比并亡",
        "庚不经络 织机虚张",
        "辛不合酱 主人不尝",
        "壬不汲水 更难提防",
        "癸不词讼 理弱敌强",
    ];

    /// 12 地支句（子..亥，顺序与 [`mingli_ganzhi::GanZhi::branch`] 一致）。
    pub const ZHI: [&str; 12] = [
        "子不问卜 自惹祸殃",
        "丑不冠带 主不还乡",
        "寅不祭祀 神鬼不尝",
        "卯不穿井 水泉不香",
        "辰不哭泣 必主重丧",
        "巳不远行 财物伏藏",
        "午不苫盖 屋主更张",
        "未不服药 毒气入肠",
        "申不安床 鬼祟入房",
        "酉不会客 醉坐颠狂",
        "戌不吃犬 作怪上床",
        "亥不嫁娶 不利新郎",
    ];

    /// 查干句（stem 0..10，甲=0）。
    #[must_use]
    pub fn gan(stem: u8) -> &'static str {
        GAN[stem as usize]
    }

    /// 查支句（branch 0..12，子=0）。
    #[must_use]
    pub fn zhi(branch: u8) -> &'static str {
        ZHI[branch as usize]
    }
}

/// 天乙贵人（通行版）。
///
/// 口诀：「甲戊庚牛羊，乙己鼠猴乡，丙丁猪鸡位，六辛逢虎马，壬癸兔蛇藏，此是贵人方。」
/// 由日干推得双贵人地支（每日干对应两地支）。十干 10 行 × 2 地支固定表。
///
/// 五源一致：《三命通会》卷三·论天乙贵人、《五行精纪》卷十三与卷十四（后者逐干列位）、
/// 《珞琭子三命消息赋注》（徐子平，「丙寅丁卯贵在猪鸡，壬戌癸亥贵于兔蛇」）、《渊海子平·论日贵》。
///
/// 🟡 另有一系把庚辛同归寅午（「甲戊兼牛羊……庚辛逢虎马」）。查证结果与坊间说法不符，记在这里：
///
/// - 这一系**不出自《珞琭子赋》**。查过《珞琭子三命消息赋注》（徐子平注）与释昙莹《珞琭子赋注》
///   两部四库本全文，均无天乙贵人口诀；徐注里出现的贵人用例（丙丁猪鸡、壬癸兔蛇）恰是通行版。
///   《三命通会·论天乙贵人》遍引《壶中子》《三车一览》《烛神经》等七种，也未引珞琭子、未记此异说。
/// - 可查的原始出处只有一处：唐·李筌《神机制敌太白阴经》卷十「推天乙所理法」——
///   「庚辛之日，旦理胜光，暮理功曹」，胜光为午、功曹为寅。这是**六壬的旦暮贵人体系**，不是禄命体系。
/// - 坊间归给《渊海子平》的说法不成立：该书全文只有一处「甲戊兼牛羊」，无「虎马」「鼠猴」。
/// - 更要紧的是，《太白阴经》那套**不是在通行版上只改庚一格**——它把甲戊合并为「旦丑暮未」，
///   整体是另一套排法。只把庚改成寅午、其余照抄通行版，等于造一套古籍里不存在的第三种。
///
/// 单源，且不可局部移植，故**不实现**。
pub mod tianyi {
    /// 天乙贵人地支表：`TIANYI[stem] = [zhi_a, zhi_b]`（地支序 0..12，子=0）。
    ///
    /// 索引（stem 顺序甲乙丙丁戊己庚辛壬癸）：
    /// - 甲(0)→[丑1， 未7] · 乙(1)→[子0， 申8] · 丙(2)→[亥11， 酉9] · 丁(3)→[亥11， 酉9]
    /// - 戊(4)→[丑1， 未7] · 己(5)→[子0， 申8] · 庚(6)→[丑1， 未7] · 辛(7)→[寅2， 午6]
    /// - 壬(8)→[卯3， 巳5] · 癸(9)→[卯3， 巳5]
    pub const TIANYI: [[u8; 2]; 10] = [
        [1, 7],  // 甲 → 丑、未
        [0, 8],  // 乙 → 子、申
        [11, 9], // 丙 → 亥、酉
        [11, 9], // 丁 → 亥、酉
        [1, 7],  // 戊 → 丑、未
        [0, 8],  // 己 → 子、申
        [1, 7],  // 庚 → 丑、未
        [2, 6],  // 辛 → 寅、午
        [3, 5],  // 壬 → 卯、巳
        [3, 5],  // 癸 → 卯、巳
    ];

    /// 查天乙贵人双地支（stem 0..10，甲=0）。
    #[must_use]
    pub fn branches_for(stem: u8) -> [u8; 2] {
        TIANYI[stem as usize]
    }
}

/// 一日择日循环要素的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Cast {
    /// 月建地支序（0..11，子=0）。
    pub month_branch: u8,
    /// 日支序（0..11，子=0）。
    pub day_branch: u8,
    /// 日干序（0..10，甲=0）。
    pub day_stem: u8,
    /// 日干支组合名（如「甲子」）。
    pub day_ganzhi_name: String,
    /// 建除十二神环位 `0..12`（0=建）。
    pub jianchu_pos: u8,
    /// 建除十二神名。
    pub jianchu: &'static str,
    /// 建除分档（黄道 / 可用 / 黑道 / 不可当）。
    pub grade: DayGrade,
    /// 分档中文标签。
    pub grade_label: &'static str,
    /// 二十八宿值日下标 `0..28`（角=0）。
    pub mansion_index: u8,
    /// 二十八宿值日名。
    pub mansion: &'static str,
    /// 彭祖百忌·干句（由日干查 [`pengzu::GAN`]）。
    pub pengzu_gan: &'static str,
    /// 彭祖百忌·支句（由日支查 [`pengzu::ZHI`]）。
    pub pengzu_zhi: &'static str,
    /// 天乙贵人地支序双值（由日干查 [`tianyi::TIANYI`]）。
    pub tianyi_branches: [u8; 2],
    /// 天乙贵人地支名双值（如 `["丑", "未"]`）。
    pub tianyi_names: [&'static str; 2],
}

/// 12 地支名（子..亥），与 [`mingli_ganzhi::GanZhi::branch`] 同顺序。
const BRANCH_NAMES: [&str; 12] = [
    "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
];

/// 在共享上下文 [`Moment`] 上算择日循环要素（确定性）。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    let mb = month_branch(m.sun_longitude);
    let gz = mingli_ganzhi::day_ganzhi(m.civil_day);
    let db = gz.branch;
    let ds = gz.stem;
    let pos = jianchu::position(mb, db);
    let mi = mansion::index_for_jdn(m.civil_day);
    let ty = tianyi::branches_for(ds);
    Cast {
        month_branch: mb,
        day_branch: db,
        day_stem: ds,
        day_ganzhi_name: format!("{}{}", gz.stem_str(), gz.branch_str()),
        jianchu_pos: pos,
        jianchu: JIANCHU[pos as usize],
        grade: day_grade(pos),
        grade_label: day_grade(pos).label(),
        #[allow(clippy::cast_possible_truncation, reason = "下标 0..28，窄化 u8 安全")]
        mansion_index: mi as u8,
        mansion: mansion::MANSIONS[mi],
        pengzu_gan: pengzu::gan(ds),
        pengzu_zhi: pengzu::zhi(db),
        tianyi_branches: ty,
        tianyi_names: [BRANCH_NAMES[ty[0] as usize], BRANCH_NAMES[ty[1] as usize]],
    }
}

/// 由本地民用时刻算择日要素（独立入口，构造 [`Moment`]）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, hour: u32, minute: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, hour, minute, tz))
}

#[cfg(test)]
mod tests;
