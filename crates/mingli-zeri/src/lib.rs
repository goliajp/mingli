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
//!    丙丁猪鸡位，六辛逢虎马，壬癸兔蛇藏」通行版口诀。🟡 另有「甲戊兼牛羊，庚辛逢虎马」古版分歧
//!    （《珞琭子赋》系），无多源校验工具，故只取通行版，变体在 [`tianyi`] doc 注明，不入码。
//!
//! 前两者「时间 → 模运算 → 值日」对同一日完全确定；后两者「干支 → 查表 → 禁忌/贵人」对同一干支
//! 完全确定。

use mingli_astro::Moment;
use serde::Serialize;

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

/// 天乙贵人（《三命通会》卷六口诀通行版）。
///
/// 口诀：「甲戊庚牛羊，乙己鼠猴乡，丙丁猪鸡位，六辛逢虎马，壬癸兔蛇藏，此是贵人方。」
/// 由日干推得双贵人地支（每日干对应两地支）。十干 10 行 × 2 地支固定表。
///
/// 🟡 流派分歧：另有《珞琭子赋》系古版「甲戊兼牛羊（无庚），庚辛逢虎马（辛归）」与此通行版分歧——
/// 庚归丑未还是寅午、辛位置不变。本 crate **不实现该变体**：无多源稳定校验源时不臆造（诚实范式）。
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
mod tests {
    use super::*;

    #[test]
    fn jianchu_is_a_twelve_cycle() {
        let set: std::collections::HashSet<_> = JIANCHU.iter().collect();
        assert_eq!(set.len(), 12);
        assert_eq!(JIANCHU[0], "建");
        assert_eq!(JIANCHU[11], "闭");
    }

    #[test]
    fn jian_falls_on_month_branch_day() {
        // 建日：日支 == 月建支 → 建除位 0 = 建。
        for mb in 0u8..12 {
            assert_eq!(jianchu::position(mb, mb), 0);
            // 逐日顺行：日支比月建支大 k → 第 k 神。
            for k in 0u8..12 {
                let db = (mb + k) % 12;
                assert_eq!(jianchu::position(mb, db), k);
            }
        }
    }

    #[test]
    fn month_branch_from_solar_terms() {
        // λ=315°（立春）→寅(2)；λ=345°（惊蛰）→卯(3)；λ=0°（春分附近，已过惊蛰）→卯(3)；
        // λ=45°（立夏）→巳(5)；λ=285°（小寒）→丑(1)。
        assert_eq!(month_branch(315.0), 2); // 寅
        assert_eq!(month_branch(345.0), 3); // 卯
        assert_eq!(month_branch(0.0), 3); // 仍卯月（惊蛰至清明）
        assert_eq!(month_branch(45.0), 5); // 巳
        assert_eq!(month_branch(285.0), 1); // 丑
        // 全 360° 扫描：月支恒在 0..12。
        let mut i = 0.0;
        while i < 360.0 {
            assert!(month_branch(i) < 12);
            i += 7.5;
        }
    }

    #[test]
    fn mansions_are_28_distinct() {
        let set: std::collections::HashSet<_> = mansion::MANSIONS.iter().collect();
        assert_eq!(set.len(), 28);
        assert_eq!(mansion::MANSIONS[0], "角");
        assert_eq!(mansion::MANSIONS[27], "轸");
    }

    #[test]
    fn mansion_anchors_cross_verified() {
        // 多锚点（跨 341 年、独立来源）校验：index = (JDN+11) mod 28，角=0。
        // 公历日 → 民用日序 → 值日宿。
        let cases = [
            (2026, 6, 14, "昴"),  // 三源一致，最强锚（实时日历）
            (2026, 6, 1, "心"),   // 两源
            (2026, 1, 1, "井"),   // rekichu 月历
            (2024, 1, 5, "鬼"),   // hotdoglab 2024 鬼宿日列表
        ];
        for (y, mo, d, want) in cases {
            let jdn = mingli_astro::civil_day_number(y, mo, d);
            let idx = mansion::index_for_jdn(jdn);
            assert_eq!(mansion::MANSIONS[idx], want, "{y}-{mo}-{d} 值日宿");
        }
        // 1685-02-04 贞享改历历元：正月朔 = 星宿（JDN 2336529）。
        assert_eq!(mansion::MANSIONS[mansion::index_for_jdn(2_336_529)], "星");
        // 连续性：逐日 +1（mod 28）。
        let j = mingli_astro::civil_day_number(2024, 1, 5);
        for k in 0..56 {
            let i0 = mansion::index_for_jdn(j + k);
            let i1 = mansion::index_for_jdn(j + k + 1);
            assert_eq!(i1, (i0 + 1) % 28);
        }
    }

    #[test]
    fn mansion_weekday_phase_lock() {
        // 28=4×7 → 值日宿与星期严格同相位：房/虚/星/昴 恒为星期日（JDN%7==... 一致）。
        // 取四个该日，验证它们的 JDN mod 7 全相同。
        let sundays = ["房", "虚", "星", "昴"];
        let mut weekday = None;
        for k in 0..(28 * 6) {
            let jdn = 2_460_311 + k;
            let name = mansion::MANSIONS[mansion::index_for_jdn(jdn)];
            if sundays.contains(&name) {
                let w = jdn.rem_euclid(7);
                assert_eq!(*weekday.get_or_insert(w), w, "宿 {name} 应恒同一星期");
            }
        }
        assert!(weekday.is_some());
    }

    #[test]
    fn compute_is_deterministic_and_well_formed() {
        let a = compute(2024, 6, 15, 14, 30, 8.0);
        let b = compute(2024, 6, 15, 14, 30, 8.0);
        assert_eq!(a.jianchu, b.jianchu);
        assert!((a.jianchu_pos as usize) < 12);
        assert!(a.day_branch < 12 && a.month_branch < 12);
        // 名与位一致。
        assert_eq!(a.jianchu, JIANCHU[a.jianchu_pos as usize]);
        assert!((a.mansion_index as usize) < 28);
        assert_eq!(a.mansion, mansion::MANSIONS[a.mansion_index as usize]);
        // 彭祖/天乙字段一致性。
        assert_eq!(a.pengzu_gan, pengzu::GAN[a.day_stem as usize]);
        assert_eq!(a.pengzu_zhi, pengzu::ZHI[a.day_branch as usize]);
        assert_eq!(a.tianyi_branches, tianyi::TIANYI[a.day_stem as usize]);
        assert!(a.day_stem < 10);
    }

    #[test]
    fn pengzu_tables_well_formed() {
        // 22 句皆非空、不重复、首字与干支顺序对应。
        assert_eq!(pengzu::GAN.len(), 10);
        assert_eq!(pengzu::ZHI.len(), 12);
        let stems = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
        let branches = [
            "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
        ];
        for (i, s) in stems.iter().enumerate() {
            assert!(
                pengzu::GAN[i].starts_with(s),
                "干句 {i} 应以 {s} 起，实为 {}",
                pengzu::GAN[i]
            );
            assert!(pengzu::GAN[i].contains("不"), "干句 {i} 缺『不』");
        }
        for (i, b) in branches.iter().enumerate() {
            assert!(
                pengzu::ZHI[i].starts_with(b),
                "支句 {i} 应以 {b} 起，实为 {}",
                pengzu::ZHI[i]
            );
            assert!(pengzu::ZHI[i].contains("不"), "支句 {i} 缺『不』");
        }
        // 22 句两两不重（无录入失误）。
        let all: std::collections::HashSet<_> =
            pengzu::GAN.iter().chain(pengzu::ZHI.iter()).collect();
        assert_eq!(all.len(), 22);
    }

    #[test]
    fn pengzu_oracle_lines() {
        // 通胜/钦定协纪辨方书通行版逐句校验：抽 4 句 + 1 句关键（辛不合酱）避免录入错。
        assert_eq!(pengzu::gan(0), "甲不开仓 财物耗散");
        assert_eq!(pengzu::gan(7), "辛不合酱 主人不尝");
        assert_eq!(pengzu::gan(9), "癸不词讼 理弱敌强");
        assert_eq!(pengzu::zhi(0), "子不问卜 自惹祸殃");
        assert_eq!(pengzu::zhi(7), "未不服药 毒气入肠");
        assert_eq!(pengzu::zhi(11), "亥不嫁娶 不利新郎");
    }

    #[test]
    fn tianyi_table_classical_couplet() {
        // 《三命通会》「甲戊庚牛羊，乙己鼠猴乡，丙丁猪鸡位，六辛逢虎马，壬癸兔蛇藏」
        // 双地支恒不等、与日干 mod 群结构一致（甲戊庚同组、乙己同组、丙丁同组、壬癸同组、辛独）。
        for stem in 0..10u8 {
            let [a, b] = tianyi::branches_for(stem);
            assert_ne!(a, b, "stem {stem} 双贵人应不同支");
            assert!(a < 12 && b < 12);
        }
        // 甲(0)/戊(4)/庚(6) → 牛(1)、未(7)
        assert_eq!(tianyi::branches_for(0), [1, 7]);
        assert_eq!(tianyi::branches_for(4), [1, 7]);
        assert_eq!(tianyi::branches_for(6), [1, 7]);
        // 乙(1)/己(5) → 子(0)、申(8)
        assert_eq!(tianyi::branches_for(1), [0, 8]);
        assert_eq!(tianyi::branches_for(5), [0, 8]);
        // 丙(2)/丁(3) → 亥(11)、酉(9)
        assert_eq!(tianyi::branches_for(2), [11, 9]);
        assert_eq!(tianyi::branches_for(3), [11, 9]);
        // 辛(7) 独 → 寅(2)、午(6)
        assert_eq!(tianyi::branches_for(7), [2, 6]);
        // 壬(8)/癸(9) → 卯(3)、巳(5)
        assert_eq!(tianyi::branches_for(8), [3, 5]);
        assert_eq!(tianyi::branches_for(9), [3, 5]);
    }

    #[test]
    fn cast_couplet_for_1990_06_15() {
        // 1990-06-15 14：30 CST 八字日柱 = 辛亥（见 ziwei/bazi oracle）。
        // → 干句 = 辛不合酱 主人不尝；支句 = 亥不嫁娶 不利新郎；
        //   天乙贵人（辛） = 寅、午。日干支名 = 辛亥。
        let c = compute(1990, 6, 15, 14, 30, 8.0);
        assert_eq!(c.day_ganzhi_name, "辛亥");
        assert_eq!(c.pengzu_gan, "辛不合酱 主人不尝");
        assert_eq!(c.pengzu_zhi, "亥不嫁娶 不利新郎");
        assert_eq!(c.tianyi_names, ["寅", "午"]);
    }
}
