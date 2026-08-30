//! 日主旺衰量化：得令 / 得地 / 得势三栏与五行力量分布。

use super::*;

/// 在 [`WuxingPower`] 上按 [`Element`] 取百分比。
pub(crate) fn wx_pct(wx: &WuxingPower, e: Element) -> u32 {
    match e {
        Element::Wood => wx.wood,
        Element::Fire => wx.fire,
        Element::Earth => wx.earth,
        Element::Metal => wx.metal,
        Element::Water => wx.water,
    }
}

/// 日主旺衰量化（扶抑学派范式）。
///
/// **算法（显式声明权重，🟡 权重值无统一标准）**：
/// - **得令(0–30)** = 日主在月支的十二长生分(`LING_TABLE`)+ 月支藏干同党加成（本气+5/中气+3/余气+1）
/// - **得地(0–30)** = 年/日/时三支藏干中同党（比劫/印）按本气×9+中气×5+余气×3 加权，封顶 30
/// - **得势(0–30)** = 年/月/时三干头（非日主）中每个同党 +10
/// - **总分** = 三栏之和(0..90)× 100 ÷ 90，封顶 100。
///
/// 「同党」= 比劫（同五行）+ 印星（生我五行），见 [`is_friendly_to_day_master`]。
/// 「耗身」= 食伤/财/官杀，本算法仅做正向加分，不做反向扣分(三栏直观可读；
///  扣分制等同净值，但黑盒化)。
///
/// **🟡 流派分歧**：权重值传统命理书未给统一表(《滴天髓》《子平真诠》《盲派》各家月令
///  权重 30%–60% 不一)。用神/扶抑会以本结果为输入。
///
/// **诚实**：量化是辅助判断，非定论；阈值（强/偏强/中和/偏弱/弱）亦为常见区分。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Strength {
    /// 综合强弱 0–100。
    pub score: u32,
    /// 等级（强/偏强/中和/偏弱/弱）。
    pub level: String,
    /// 得令（月支）0–30。
    pub got_ling: u32,
    /// 得地（通根）0–30。
    pub got_di: u32,
    /// 得势（干头）0–30。
    pub got_shi: u32,
    /// 五行力量分布（百分比，合 100）。
    pub wuxing: WuxingPower,
}

/// 五行力量分布（百分比，合 100）。
///
/// 权重：天干 10、地支本气 12、中气 6、余气 3；月支×1.5（得令加成）。
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct WuxingPower {
    /// 木 %。
    pub wood: u32,
    /// 火 %。
    pub fire: u32,
    /// 土 %。
    pub earth: u32,
    /// 金 %。
    pub metal: u32,
    /// 水 %。
    pub water: u32,
}

/// 日主在月支的十二长生阶段「得令」分(0..30)。
/// 帝旺/临官最旺，死/绝最弱。索引同 [`mingli_ganzhi::TWELVE_STAGES`]。
pub(crate) const LING_TABLE: [u32; 12] = [
    20, // 0 长生
    8,  // 1 沐浴
    18, // 2 冠带
    27, // 3 临官
    30, // 4 帝旺
    10, // 5 衰
    6,  // 6 病
    3,  // 7 死
    8,  // 8 墓
    1,  // 9 绝
    10, // 10 胎
    15, // 11 养
];

/// 在本命四柱基础上叠加额外干支（岁运叠加旺衰：大运柱、流年柱等）算 t 时刻的实际旺衰。
///
/// 「得令」固定取本命月支（月令是出生即定的天时，大运流年不改）；「得地」「得势」「五行分布」
/// 把 extras 拼入加权；封顶 30 不变（外力把得地/得势推得更接近满分，但不超）。
/// 综合分仍以 90 为基线归一到 100，可解释「外力让本命旺衰偏向更强还是更弱」。
///
/// **🟡 流派分歧**：岁运叠加权重无统一标准，本算法把 extras 当作与本命三干/三支同质量级的「外援」（每干10、每支按本/中/余 9/5/3）；
/// 盲派/新派对岁运折扣不一（有给 ½ 的、有完全等同的）。
#[must_use]
pub fn compute_strength_with_extras(
    year_gz: GanZhi,
    month_gz: GanZhi,
    day_gz: GanZhi,
    hour_gz: GanZhi,
    extras: &[GanZhi],
) -> Strength {
    compute_strength_inner(year_gz, month_gz, day_gz, hour_gz, extras)
}

pub(crate) fn compute_strength(year_gz: GanZhi, month_gz: GanZhi, day_gz: GanZhi, hour_gz: GanZhi) -> Strength {
    compute_strength_inner(year_gz, month_gz, day_gz, hour_gz, &[])
}

pub(crate) fn compute_strength_inner(
    year_gz: GanZhi,
    month_gz: GanZhi,
    day_gz: GanZhi,
    hour_gz: GanZhi,
    extras: &[GanZhi],
) -> Strength {
    let dm = day_gz.stem;

    // 得令 = 月支十二长生分 + 月支藏干同党加成
    let stage = twelve_stage(dm, month_gz.branch) as usize;
    let mut got_ling = LING_TABLE[stage];
    for (i, &s) in hidden_stems(month_gz.branch).iter().enumerate() {
        if is_friendly_to_day_master(dm, s) {
            got_ling += [5, 3, 1][i.min(2)];
        }
    }
    got_ling = got_ling.min(30);

    // 得地 = 年/日/时三支 + extras 地支 藏干同党加权（本=9/中=5/余=3），月支已计入得令不重算。
    let mut got_di = 0u32;
    let di_branches = [year_gz.branch, day_gz.branch, hour_gz.branch];
    for &br in di_branches.iter().chain(extras.iter().map(|g| &g.branch)) {
        for (i, &s) in hidden_stems(br).iter().enumerate() {
            if is_friendly_to_day_master(dm, s) {
                got_di += [9, 5, 3][i.min(2)];
            }
        }
    }
    got_di = got_di.min(30);

    // 得势 = 年/月/时三干头 + extras 天干 同党各 +10（日主本身不算）。
    let mut got_shi = 0u32;
    let shi_stems = [year_gz.stem, month_gz.stem, hour_gz.stem];
    for &st in shi_stems.iter().chain(extras.iter().map(|g| &g.stem)) {
        if is_friendly_to_day_master(dm, st) {
            got_shi += 10;
        }
    }
    got_shi = got_shi.min(30);

    let raw = got_ling + got_di + got_shi;
    let score = (raw * 100 / 90).min(100);
    let level = if score >= 75 {
        "强"
    } else if score >= 60 {
        "偏强"
    } else if score >= 40 {
        "中和"
    } else if score >= 25 {
        "偏弱"
    } else {
        "弱"
    }
    .to_string();

    // 五行力量分布：天干 10、地支本气 12/中 6/余 3，月支 ×1.5。extras 干支与本命同质量级入。
    let mut wx = [0u32; 5];
    let stems = [year_gz.stem, month_gz.stem, day_gz.stem, hour_gz.stem];
    for &st in stems.iter().chain(extras.iter().map(|g| &g.stem)) {
        wx[stem_element(st).index()] += 10;
    }
    let branches = [year_gz.branch, month_gz.branch, day_gz.branch, hour_gz.branch];
    for (j, &br) in branches.iter().enumerate() {
        let is_month = j == 1;
        for (i, &s) in hidden_stems(br).iter().enumerate() {
            let base = [12u32, 6, 3][i.min(2)];
            let w = if is_month { base * 3 / 2 } else { base };
            wx[stem_element(s).index()] += w;
        }
    }
    for g in extras {
        for (i, &s) in hidden_stems(g.branch).iter().enumerate() {
            wx[stem_element(s).index()] += [12u32, 6, 3][i.min(2)];
        }
    }
    // 总权重必 ≥ 4*10=40（四个天干本身），整数除安全。
    let total: u32 = wx.iter().sum();
    let norm = |v: u32| (v * 100 + total / 2) / total;
    let wuxing = WuxingPower {
        wood: norm(wx[0]),
        fire: norm(wx[1]),
        earth: norm(wx[2]),
        metal: norm(wx[3]),
        water: norm(wx[4]),
    };

    Strength {
        score,
        level,
        got_ling,
        got_di,
        got_shi,
        wuxing,
    }
}
