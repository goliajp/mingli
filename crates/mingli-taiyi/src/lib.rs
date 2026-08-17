//! L3 叶（⟂ 横切 / 确定性）：太乙神数的可计算结构。
//!
//! 太乙数（三式之首）把一个**积年**（自上元累积的年数）折成太乙在八宫上的位置：
//!
//! - **太乙积年**：自「上元混沌甲子」累积。历元锚用《太乙金镜式经》：唐开元十二年（724 CE）=
//!   积年 `1_937_281`（[`accumulated_years`]）。
//! - **太乙行宫**：太乙**三年居一宫、不入中宫、二十四年转一周（八宫×3）、七十二年游三期**，
//!   阳遁顺行（一宫起）、阴遁逆行（九宫起）。每宫三年依次「理天 / 理地 / 理人」（三才）。
//!   宫由积年定：`r = 积年 mod 24`，宫序 `= ⌊r/3⌋`，三才 `= r mod 3`（[`taiyi_palace`]）。
//! - **阴阳遁**：冬至后阳遁、夏至后阴遁（由太阳视黄经定）。
//! - **十六神**：八正宫（八卦方位）+ 八间神（十二支中非四正者）共十六方位，是太乙盘的方位框架。
//!
//! 八宫复用洛书九宫（[`mingli_luoshu`]）的宫数↔八卦映射。
//!
//! - **诸将**：文昌（主目）→ 主算 → 主大将 / 主参将；始击（客目）→ 客算 → 客大将 / 客参将。
//!   算法与十六神名皆两部原典明载，见 [`wenchang`] / [`shiji`] / [`suan`]。
//!
//! # 一个词三义的坑：天目
//!
//! 「天目」在太乙文献里一词三义，不分清必然算错：
//!
//! 1. **种子义**：由积年推出的那个十六神位，它**就是文昌**（《太乙统宗宝鉴》卷一
//!    「求四計天目文昌所在」「天目者，主目上将是也，名曰文昌」）
//! 2. **配对义**：在「上目 / 下目」对举里，**天目 = 上目 = 始击**（客），地目 = 下目 = 文昌（主）
//! 3. **总名义**：上下二目合称天目
//!
//! 同书两处看似矛盾，实为一词二义。《统宗》卷二：「文昌名地目，亦名下目，属主人之計。
//! 始擊名天目，亦名上目，属客之計。……先無始擊，故曰天目。因生始擊，相反而變，故為地目也。」
//! 现代通俗读物普遍用「文昌 = 天目」而与原典第 2 义相反。
//! **本 crate 一律弃用「天目 / 地目」，只用「文昌」「始击」**，从命名上绕开这个坑。
//!
//! 诚实边界（🟡）：**定计目**（及其定算 / 定大将 / 定参将）只见《太乙统宗宝鉴》一书，
//! 《太乙金镜式经》的「運式之儀有八」里没有这一条，单源不实现。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "积年 mod 24 ∈ 0..24、step∈0..8、三才∈0..3，窄化到 u8/usize 受控安全"
)]

mod engine;
pub use engine::TaiyiEngine;

use mingli_astro::Moment;
use serde::Serialize;

/// 太乙积年历元锚：唐开元十二年 = 公元 724 年。
pub const TAIYI_EPOCH_YEAR: i64 = 724;
/// 该锚年的太乙积年（《太乙金镜式经》：上元混沌甲子 → 724 CE）。
pub const TAIYI_EPOCH_JINIAN: i64 = 1_937_281;

/// 太乙行宫的八宫（洛书宫数，顺行序，不入中五）：一宫起 1→2→3→4→6→7→8→9。
pub const PALACES_8: [u8; 8] = [1, 2, 3, 4, 6, 7, 8, 9];

/// 三才（太乙居一宫三年，依次理天/理地/理人）。
pub const SANCAI: [&str; 3] = ["理天", "理地", "理人"];

/// 十六方位（八正宫 + 八间神）：十二地支 + 四维卦，按罗盘自子顺时针。**indisputable 结构**。
pub const SIXTEEN_DIRECTIONS: [&str; 16] = [
    "子", "丑", "艮", "寅", "卯", "辰", "巽", "巳", "午", "未", "坤", "申", "酉", "戌", "乾", "亥",
];

/// 十六神名（与 [`SIXTEEN_DIRECTIONS`] 同序）。
///
/// 《太乙金镜式经》卷二「推十六神所主法」与《太乙统宗宝鉴》卷二「明太乙十六宫間之神術」
/// 逐条对照，名、支、序全同，仅异体字之差（太炅 / 大炅、大神 / 太神、太蔟 / 太簇）。
///
/// **偶数位是正宫（配八卦与宫数），奇数位是间神（不配宫）**——这条分野是主客算的基础。
pub const SIXTEEN_GODS: [&str; 16] = [
    "地主", "阳德", "和德", "吕申", "高丛", "太阳", "大炅", "大神", "大威", "天道", "大武", "武德",
    "太簇", "阴主", "阴德", "大义",
];

/// 十六神里正宫位对应的**太乙九宫**宫数；间神位为 `None`。
///
/// ⚠ 太乙的九宫配法是**乾 1 · 离 2 · 艮 3 · 震 4 · 中 5 · 兑 6 · 坤 7 · 坎 8 · 巽 9**，
/// 与洛书（坎 1 · 坤 2 · 震 3 · 巽 4 · 中 5 · 乾 6 · 兑 7 · 艮 8 · 离 9）**不是一回事**。
/// 三源一致，且由主客算的算例反证：局 11 客算得 4，只有在太乙配法下才走得通。
pub const RING_PALACE: [Option<u8>; 16] = [
    Some(8), None, Some(3), None, Some(4), None, Some(9), None, // 子艮卯巽
    Some(2), None, Some(7), None, Some(6), None, Some(1), None, // 午坤酉乾
];

/// 太乙九宫的宫数 → 卦名（`PALACE_GUA[n]`，index 0 占位）。
pub const PALACE_GUA: [&str; 10] = ["", "乾", "离", "艮", "震", "中", "兑", "坤", "坎", "巽"];

/// 文昌（主目上将）所在的十六神位。
///
/// 《太乙金镜式经》卷一「推天目所在法」：「置上元積年……以天目周法十八去之，
/// 不滿者命起武徳，順行十六神，遇隂徳、大武重留一筭，外即天目所在」。
/// 阴遁则「命起吕申，過大炅、和德……重留一算」（《太乙统宗宝鉴》卷一）。
///
/// 「重留一算」的所以然《统宗》卷二给了：「隂徳属乾，乾為天門；大武属坤，坤為地户。
/// 天目之神行至天門地戸之方，以伺其命，故重留一算以待之故也。」
/// 两个双计位各占两算，一周恰 18 算，故周法十八。
#[must_use]
pub fn wenchang(jinian: i64, yang_dun: bool) -> usize {
    let (start, doubled): (usize, [usize; 2]) =
        if yang_dun { (11, [10, 14]) } else { (3, [2, 6]) };
    let target = {
        let r = jinian.rem_euclid(18);
        if r == 0 { 18 } else { r }
    };
    let mut k = start;
    let mut count = 0_i64;
    loop {
        count += if doubled.contains(&k) { 2 } else { 1 };
        if count >= target {
            return k;
        }
        k = (k + 1) % 16;
    }
}

/// 计神所在的十六神位。阳遁起寅、阴遁起申，**逆行**十二辰（四维不入）。
#[must_use]
pub fn jishen(ju: i64, yang_dun: bool) -> usize {
    const ZHI_RING: [usize; 12] = [0, 1, 3, 4, 5, 7, 8, 9, 11, 12, 13, 15];
    let start = if yang_dun { 3 } else { 11 }; // 寅 / 申
    let si = ZHI_RING.iter().position(|&z| z == start).unwrap_or(0);
    let steps = usize::try_from((ju - 1).rem_euclid(12)).unwrap_or(0);
    ZHI_RING[(si + 12 - steps) % 12]
}

/// 始击（客目上将）所在的十六神位。
///
/// 《太乙金镜式经》卷二第五条：「以計神加和徳宫，求文昌所臨宫」；
/// 《太乙统宗宝鉴》卷二说得更全：「計神既加和徳之宫，却視天上文昌所臨之下，而為始擊神也。
/// 文昌為主目，始擊為客目，因主而生客之義也。」
///
/// 即把天盘转到「计神压和德（艮）」，读文昌此时落在地盘的哪一神位：
/// `始击 = (文昌 + 和德 − 计神) mod 16`，和德位 = 2。
///
/// ⚠ 有把它做成「计神加文昌、看和德下」的，二者只在 `文昌 − 计神 ≡ 0 或 8 (mod 16)`
/// 时才碰巧相等，方向反了会算错。
#[must_use]
pub fn shiji(wenchang_pos: usize, jishen_pos: usize) -> usize {
    (wenchang_pos + 2 + 16 - jishen_pos) % 16
}

/// 主算 / 客算：自 `from`（文昌或始击）顺行累加沿途**正宫**的宫数，至太乙宫止。
///
/// 《太乙金镜式经》卷二第八条：「各視天目所在宫而行筭，**若天目在正宫則按本數，
/// 若天目間神則加一數**，而行筭**至太乙宫止矣**。」
/// 《太乙统宗宝鉴》卷二第七条补上方向与终点：「自左順行，依宫数筭……**故算至太乙前一宫而止**」。
///
/// 三条边界：起点若为正宫则计其宫数、为间神则计 1；间神一律不累加；**终点太乙宫不计入**。
#[must_use]
pub fn suan(from: usize, taiyi_palace: u8) -> u32 {
    if RING_PALACE[from] == Some(taiyi_palace) {
        return u32::from(taiyi_palace);
    }
    let mut total = RING_PALACE[from].map_or(1, u32::from);
    let mut k = from;
    loop {
        k = (k + 1) % 16;
        match RING_PALACE[k] {
            Some(p) if p == taiyi_palace => return total,
            Some(p) => total += u32::from(p),
            None => {}
        }
    }
}

/// 「去十用零」：《太乙统宗宝鉴》卷二第八条——余数即大将所迁宫次；
/// 「如得一十、或二十、或三十、或四十，**以九去之**」。
#[must_use]
pub fn qu_shi(n: u32) -> u8 {
    let r = if n.is_multiple_of(10) { n % 9 } else { n % 10 };
    #[allow(clippy::cast_possible_truncation, reason = "取模后恒在 0..10")]
    let r = r as u8;
    if r == 0 { 9 } else { r }
}

/// 参将宫：大将宫「**三因**」后再去十用零（《太乙统宗宝鉴》卷二第八条）。
#[must_use]
pub fn can_jiang(da_jiang: u8) -> u8 {
    qu_shi(u32::from(da_jiang) * 3)
}

/// 一组「目 → 算 → 大将 → 参将」。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Mu {
    /// 所在十六神位序 0..16。
    pub position: usize,
    /// 神名。
    pub name: &'static str,
    /// 所在方位（十二支或四维）。
    pub direction: &'static str,
    /// 算数。
    pub suan: u32,
    /// 大将所迁宫次 1..=9。
    pub da_jiang: u8,
    /// 参将所迁宫次 1..=9。
    pub can_jiang: u8,
}

impl Mu {
    fn at(position: usize, taiyi_palace: u8) -> Self {
        let s = suan(position, taiyi_palace);
        let da = qu_shi(s);
        Self {
            position,
            name: SIXTEEN_GODS[position],
            direction: SIXTEEN_DIRECTIONS[position],
            suan: s,
            da_jiang: da,
            can_jiang: can_jiang(da),
        }
    }
}

/// 由公历年算太乙积年（积年 = 历元积年 + （年 − 历元年））。
#[must_use]
pub fn accumulated_years(year: i64) -> i64 {
    TAIYI_EPOCH_JINIAN + (year - TAIYI_EPOCH_YEAR)
}

/// 是否阳遁：冬至后（太阳黄经 `λ∈[270,360)∪[0,90)`）阳遁，夏至后阴遁。
#[must_use]
pub fn is_yang_dun(sun_longitude: f64) -> bool {
    let l = sun_longitude.rem_euclid(360.0);
    // 阳遁段 = 冬至(270)→夏至(90)，即不在 [90，270)（夏至→冬至 阴遁段）内。
    !(90.0..270.0).contains(&l)
}

/// 太乙行宫结果。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct TaiyiPalace {
    /// 八宫序 `0..8`（沿阴阳遁方向的第几步）。
    pub step: u8,
    /// 洛书宫数（1..9，必非 5）。
    pub palace: u8,
    /// 该宫八卦名（复用洛书九宫）。
    pub gua: &'static str,
    /// 入宫年数 `1..=3`（该宫已居第几年）。
    pub year_in_palace: u8,
    /// 三才（理天/理地/理人）。
    pub sancai: &'static str,
}

/// 由积年与阴阳遁定太乙宫。`r = 积年 mod 24`，宫序 `⌊r/3⌋`，三才 `r mod 3`；阳顺阴逆。
#[must_use]
pub fn taiyi_palace(jinian: i64, yang_dun: bool) -> TaiyiPalace {
    let r = jinian.rem_euclid(24);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "r∈0..24，step∈0..8，sancai∈0..3，窄化受控"
    )]
    let step = (r / 3) as usize;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "见上")]
    let sancai = (r % 3) as usize;
    // 阳遁顺行（一宫起），阴遁逆行（九宫起）= 顺行序的镜像。
    let palace = if yang_dun {
        PALACES_8[step]
    } else {
        PALACES_8[7 - step]
    };
    TaiyiPalace {
        step: step as u8,
        palace,
        gua: PALACE_GUA[palace as usize],
        year_in_palace: sancai as u8 + 1,
        sancai: SANCAI[sancai],
    }
}

/// 一次太乙起局（确定部分）的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Cast {
    /// 公历年。
    pub year: i64,
    /// 太乙积年。
    pub jinian: i64,
    /// 是否阳遁。
    pub yang_dun: bool,
    /// 太乙行宫。
    pub taiyi: TaiyiPalace,
    /// 入局数 1..=72（积年 mod 72，零则七十二）。
    pub ju: i64,
    /// 计神所在方位（阳遁起寅、阴遁起申，逆行十二辰）。
    pub jishen: &'static str,
    /// 文昌（主目）与主算 / 主大将 / 主参将。
    pub wenchang: Mu,
    /// 始击（客目）与客算 / 客大将 / 客参将。
    pub shiji: Mu,
}

/// 在共享上下文 [`Moment`] 上起太乙局（确定部分）。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    let year = i64::from(m.year);
    let jinian = accumulated_years(year);
    let yang = is_yang_dun(m.sun_longitude);
    let taiyi = taiyi_palace(jinian, yang);
    let ju = {
        let r = jinian.rem_euclid(72);
        if r == 0 { 72 } else { r }
    };
    let wc = wenchang(jinian, yang);
    let js = jishen(ju, yang);
    Cast {
        year,
        jinian,
        yang_dun: yang,
        ju,
        jishen: SIXTEEN_DIRECTIONS[js],
        wenchang: Mu::at(wc, taiyi.palace),
        shiji: Mu::at(shiji(wc, js), taiyi.palace),
        taiyi,
    }
}

/// 由本地民用日期起局（独立入口）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, 12, 0, tz))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jinian_epoch_anchor() {
        // 《太乙金镜式经》：724 CE = 积年 1_937_281。
        assert_eq!(accumulated_years(724), 1_937_281);
        // 线性外推：相邻年差 1。
        assert_eq!(accumulated_years(725), 1_937_282);
        assert_eq!(accumulated_years(723), 1_937_280);
    }

    #[test]
    fn palaces_skip_center_and_are_eight() {
        assert_eq!(PALACES_8.len(), 8);
        assert!(!PALACES_8.contains(&5), "太乙不入中五");
        let set: std::collections::HashSet<u8> = PALACES_8.iter().copied().collect();
        assert_eq!(set.len(), 8);
        // 顺行序：一宫起，1→2→3→4→6→7→8→9。
        assert_eq!(PALACES_8[0], 1);
        assert_eq!(PALACES_8[7], 9);
    }

    #[test]
    fn taiyi_dwells_three_years_per_palace_cycle_24() {
        // 三年居一宫、廿四年转一周：同一阳遁下，每 3 年宫序 +1，24 年回到原宫。
        for base in 0..24i64 {
            let p0 = taiyi_palace(base, true);
            // 入宫年数 1..3，三才随之。
            assert!((1..=3).contains(&p0.year_in_palace));
            assert_eq!(p0.sancai, SANCAI[(p0.year_in_palace - 1) as usize]);
            // 同宫连居三年。
            let same = taiyi_palace(base - (base % 3), true);
            let same2 = taiyi_palace(base - (base % 3) + 2, true);
            assert_eq!(same.palace, same2.palace, "同宫三年应同宫");
            // 24 年后复位。
            assert_eq!(taiyi_palace(base + 24, true).palace, p0.palace);
        }
        // 八宫在 24 年内恰好各被走到一次。
        let set: std::collections::HashSet<u8> = (0..24).step_by(3).map(|r| taiyi_palace(r, true).palace).collect();
        assert_eq!(set.len(), 8);
    }

    #[test]
    fn yin_dun_is_mirror_of_yang() {
        // 阴遁逆行：九宫起，与阳遁同步序号镜像。
        assert_eq!(taiyi_palace(0, false).palace, 9); // 阴遁一步起九宫
        for r in 0..24i64 {
            let y = taiyi_palace(r, true);
            let n = taiyi_palace(r, false);
            assert_eq!(n.palace, PALACES_8[7 - y.step as usize]);
            // 太乙恒不入中五。
            assert_ne!(y.palace, 5);
            assert_ne!(n.palace, 5);
        }
    }

    #[test]
    fn yang_yin_dun_by_solar_term() {
        // 冬至(λ=270)后阳遁、夏至(λ=90)后阴遁。
        assert!(is_yang_dun(270.0)); // 冬至
        assert!(is_yang_dun(0.0)); // 春分仍阳遁段
        assert!(!is_yang_dun(90.0)); // 夏至
        assert!(!is_yang_dun(180.0)); // 秋分阴遁段
        // 全 360° 各半。
        let yang = (0..360).filter(|&d| is_yang_dun(f64::from(d))).count();
        assert_eq!(yang, 180);
    }

    #[test]
    fn sixteen_gods_framework() {
        assert_eq!(SIXTEEN_DIRECTIONS.len(), 16);
        assert_eq!(SIXTEEN_GODS.len(), 16);
        // 十六方位含十二地支 + 四维卦。
        for s in ["子", "午", "卯", "酉", "艮", "巽", "坤", "乾"] {
            assert!(SIXTEEN_DIRECTIONS.contains(&s), "缺方位 {s}");
        }
        assert_eq!(SIXTEEN_GODS[0], "地主"); // 子神
    }

    #[test]
    fn compute_is_deterministic_and_palace_valid() {
        let c = compute(2024, 6, 15, 8.0);
        assert_eq!(c.jinian, accumulated_years(2024));
        assert!((1..=9).contains(&c.taiyi.palace) && c.taiyi.palace != 5);
        // 卦名走太乙自家的九宫配法，不是洛书——见 `taiyi_numbers_its_palaces_differently_from_the_luoshu`
        assert_eq!(c.taiyi.gua, PALACE_GUA[c.taiyi.palace as usize]);
        let c2 = compute(2024, 6, 15, 8.0);
        assert_eq!(c.taiyi.palace, c2.taiyi.palace);
        assert_eq!(c.taiyi.sancai, c2.taiyi.sancai);
    }
}

#[cfg(test)]
mod generals {
    use super::*;

    /// 两则原典纪年实例，相隔一千一百年、出自两部不同的书，全字段吻合。
    ///
    /// 《太乙统宗宝鉴》卷四：「唐昭宗天復二年壬戍嵗……壬子元第十一局，太乙在四宫，
    /// 文昌在髙叢……始擊在陽徳，客筭得四」；
    /// 「秦二世二年甲午歲……庚子元第五十五局，太乙在三宫，始擊臨和徳……文昌在武徳」。
    /// 两者与《太乙金镜式经》卷三的立成表逐字吻合，是两部书之间的交叉见证。
    ///
    /// 这一条同时钉住：积年锚点、入局数、太乙行宫、文昌周法十八、计神逆行十二辰、
    /// 「计神加和德看文昌」的方向、以及主客算的三条边界。任一处错都会红。
    #[test]
    fn two_dated_examples_from_the_sources_agree_on_every_field() {
        /// 一则纪年实例：年 / 局 / 太乙宫 / 文昌 / 计神 / 始击 / 客算（原文未载则 `None`）。
        struct Example {
            year: i64,
            ju: i64,
            palace: u8,
            wenchang: &'static str,
            jishen: &'static str,
            shiji: &'static str,
            ke_suan: Option<u32>,
        }
        const ORACLE: [Example; 2] = [
            Example { year: 902, ju: 11, palace: 4, wenchang: "高丛", jishen: "辰", shiji: "阳德", ke_suan: Some(4) },
            Example { year: -206, ju: 55, palace: 3, wenchang: "武德", jishen: "申", shiji: "和德", ke_suan: None },
        ];
        for Example { year, ju: want_ju, palace: want_palace, wenchang: want_wc, jishen: want_js, shiji: want_sj, ke_suan: want_ke } in ORACLE {
            let j = accumulated_years(year);
            let ju = {
                let r = j.rem_euclid(72);
                if r == 0 { 72 } else { r }
            };
            assert_eq!(ju, want_ju, "{year} 年应是第 {want_ju} 局");
            let t = taiyi_palace(j, true);
            assert_eq!(t.palace, want_palace, "{year} 年太乙应在 {want_palace} 宫");
            let wc = wenchang(j, true);
            assert_eq!(SIXTEEN_GODS[wc], want_wc, "{year} 年文昌");
            let js = jishen(ju, true);
            assert_eq!(SIXTEEN_DIRECTIONS[js], want_js, "{year} 年计神");
            let sj = shiji(wc, js);
            assert_eq!(SIXTEEN_GODS[sj], want_sj, "{year} 年始击");
            if let Some(k) = want_ke {
                assert_eq!(suan(sj, t.palace), k, "{year} 年客算");
            }
        }
    }

    /// 三个纪年锚点落在同一个 72 局环上、互相自洽。
    ///
    /// 《金镜式经》「開元十二年甲子＝局 49」、《统宗》「天復二年＝局 11」「秦二世二年＝局 55」。
    /// 724 → 902 相隔 178 年，49 + 178 ≡ 11 (mod 72)。
    #[test]
    fn the_three_anchors_land_on_one_consistent_cycle() {
        let ju = |y: i64| {
            let r = accumulated_years(y).rem_euclid(72);
            if r == 0 { 72 } else { r }
        };
        assert_eq!(ju(724), 49, "开元十二年应是第 49 局");
        assert_eq!(ju(902), 11);
        assert_eq!(ju(-206), 55);
        assert_eq!((49 + 178) % 72, 11, "724→902 相隔 178 年，局序自洽");
    }

    /// 文昌周法十八：两个双计位各占两算，绕一圈恰 18 算。
    #[test]
    fn the_wenchang_cycle_is_eighteen_because_two_positions_count_twice() {
        for yang in [true, false] {
            let seen: Vec<usize> = (1..=18).map(|j| wenchang(j, yang)).collect();
            // 18 算走遍 16 神位，其中两位各占两算
            let mut uniq = seen.clone();
            uniq.sort_unstable();
            uniq.dedup();
            assert_eq!(uniq.len(), 16, "十八算应走遍十六神位");
            // 第 19 算回到起点
            assert_eq!(wenchang(19, yang), wenchang(1, yang), "周法十八，第十九算回起点");
            let doubled: Vec<usize> = (0..16)
                .filter(|k| seen.iter().filter(|s| *s == k).count() == 2)
                .collect();
            let want = if yang { vec![10, 14] } else { vec![2, 6] };
            assert_eq!(doubled, want, "阳遁双计大武阴德，阴遁双计和德大炅");
        }
        // 阳遁起武德、阴遁起吕申
        assert_eq!(SIXTEEN_GODS[wenchang(1, true)], "武德");
        assert_eq!(SIXTEEN_GODS[wenchang(1, false)], "吕申");
    }

    /// 计神逆行十二辰，不入四维；阳起寅、阴起申。
    #[test]
    fn the_reckoning_spirit_runs_backwards_through_the_twelve_branches() {
        assert_eq!(SIXTEEN_DIRECTIONS[jishen(1, true)], "寅");
        assert_eq!(SIXTEEN_DIRECTIONS[jishen(1, false)], "申");
        // 逆行：第二局退到丑
        assert_eq!(SIXTEEN_DIRECTIONS[jishen(2, true)], "丑");
        // 十二局走遍十二辰，绝不落四维
        for yang in [true, false] {
            let mut seen: Vec<&str> =
                (1..=12).map(|j| SIXTEEN_DIRECTIONS[jishen(j, yang)]).collect();
            for d in &seen {
                assert!(!["艮", "巽", "坤", "乾"].contains(d), "计神不入四维，实得 {d}");
            }
            seen.sort_unstable();
            seen.dedup();
            assert_eq!(seen.len(), 12);
            assert_eq!(jishen(13, yang), jishen(1, yang), "十二局一周");
        }
    }

    /// 主客算的三条边界：起点为间神计 1、间神不累加、终点太乙宫不计入。
    #[test]
    fn the_reckoning_counts_only_proper_palaces_and_stops_before_taiyi() {
        // 起点即压太乙宫 → 径取该宫数
        assert_eq!(suan(4, 4), 4, "高丛正宫 4，恰压太乙 4 宫");
        // 局 11 的客算：阳德(间神,计1) → 和德(宫3) → 吕申(间神,不计) → 高丛=太乙宫，止
        assert_eq!(suan(1, 4), 4);
        // 无论起点与太乙宫如何，算数恒为正且有限
        for from in 0..16 {
            for t in [1, 2, 3, 4, 6, 7, 8, 9] {
                assert!(suan(from, t) > 0);
            }
        }
    }

    /// 去十用零与三因取参将。整十者以九去之，是《统宗》明写的特例。
    #[test]
    fn dropping_the_tens_treats_exact_tens_by_nine() {
        assert_eq!(qu_shi(4), 4);
        assert_eq!(qu_shi(16), 6);
        assert_eq!(qu_shi(25), 5);
        assert_eq!(qu_shi(10), 1, "整十以九去之：10 % 9 = 1");
        assert_eq!(qu_shi(20), 2);
        assert_eq!(qu_shi(30), 3);
        assert_eq!(qu_shi(90), 9, "90 % 9 = 0 → 归 9");
        // 参将 = 大将三因后再去十
        assert_eq!(can_jiang(4), 2, "4×3=12 → 2");
        assert_eq!(can_jiang(5), 5, "5×3=15 → 5，中五特例：诸将可落中五");
        for d in 1..=9u8 {
            assert!((1..=9).contains(&can_jiang(d)));
        }
    }

    /// 太乙的九宫配法与洛书不同——宫 1 是乾不是坎。
    ///
    /// 这不是命名口味问题：主客算沿环累加宫数，配法错了算数就错。
    /// 局 11 客算得 4 只在太乙配法下成立（阳德计 1 + 和德艮 3 = 4，止于高丛卯 = 4 宫）。
    #[test]
    fn taiyi_numbers_its_palaces_differently_from_the_luoshu() {
        assert_eq!(PALACE_GUA[1], "乾");
        assert_eq!(mingli_luoshu::PALACE_NAME[1], "坎", "洛书的一宫是坎，两套不可混用");
        assert_eq!(PALACE_GUA[4], "震");
        assert_eq!(PALACE_GUA[9], "巽");
        assert_eq!(PALACE_GUA[5], "中");
        // 十六神环上八正宫恰配八卦宫数，八间神无宫
        let with_palace: Vec<u8> = RING_PALACE.iter().flatten().copied().collect();
        let mut sorted = with_palace.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![1, 2, 3, 4, 6, 7, 8, 9], "八正宫恰是不含中五的八宫");
        for (k, p) in RING_PALACE.iter().enumerate() {
            assert_eq!(p.is_some(), k.is_multiple_of(2), "偶数位正宫、奇数位间神");
        }
    }
}
