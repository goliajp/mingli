//! 用神 / 喜忌：由旺衰与五行分布推出该补什么、该忌什么。

use super::*;

/// 用神 / 喜忌：把旺衰 + 五行分布合起来给出「该补什么 / 该忌什么」。
///
/// **算法（扶抑学派 + 调候辅助，显式权重）**：
/// - **身强(score ≥ 60)** → 取耗身五行（官杀/财/食伤）；三类候选取**当前盘中分布最弱**者
///   为主用神（补缺最有效），次弱者为副用神；忌神 = 印（生身）+ 比劫（帮身）。
/// - **身弱(score ≤ 40)** → 取助身五行（印星/比劫）；**印星优先**（双重作用 — 生身 + 化杀生官），
///   比劫副选；忌神 = 官杀（克身） + 财（损印）。
/// - **中和(40 < score < 60)** → 走调候为主：寒月（亥子丑）取火、燥月（巳午未）取水、
///   春木月（寅卯）取金修剪、秋金月（申酉）取火炼；辰戌杂气取日主同行扶身。
///
/// 「同党」+「耗身」分类沿用 [`is_friendly_to_day_master`]。
///
/// 用神是命格 + 旺衰的**自然推论** — 命局所喜五行 = 用神 = 补之则吉；
/// 命局所忌五行 = 忌神 = 加强则凶。这是命理体系给出的吉凶判断方向。
///
/// **🟡 流派分歧**：取用神有扶抑/调候/通关/病药/格局用神五法，各家先后顺序不同；
/// 「从格 / 化格」反扶抑（扶其太过、抑其不及）本算法不覆盖。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct YongShen {
    /// 取用法（扶抑·身强 / 扶抑·身弱 / 调候为主）。
    pub method: String,
    /// 主用神五行（命局最该补的）。
    pub primary_wuxing: String,
    /// 主用神对日主的角色（印星/比劫/官杀/财/食伤/调候）。
    pub primary_role: String,
    /// 副用神五行（次要扶抑）；调候法无副用神。
    pub secondary_wuxing: Option<String>,
    /// 副用神对日主的角色。
    pub secondary_role: Option<String>,
    /// 忌神五行（命局最该避的；调候法暂留空）。
    pub avoid_wuxing: Vec<String>,
    /// 推理链（短句解释为什么取这个用神）。
    pub reasoning: String,
}

/// 反查「生我」的五行（印星五行 = X 满足 X.generates() == dm）。
pub(crate) const fn yin_xing_of(dm: Element) -> Element {
    match dm {
        Element::Wood => Element::Water,
        Element::Fire => Element::Wood,
        Element::Earth => Element::Fire,
        Element::Metal => Element::Earth,
        Element::Water => Element::Metal,
    }
}

/// 反查「克我」的五行（官杀五行 = X 满足 X.controls() == dm）。
pub(crate) const fn guan_sha_of(dm: Element) -> Element {
    match dm {
        Element::Wood => Element::Metal,
        Element::Fire => Element::Water,
        Element::Earth => Element::Wood,
        Element::Metal => Element::Fire,
        Element::Water => Element::Earth,
    }
}

/// 取用神：旺衰 + 五行分布 → 主/副用神五行 + 忌神。
///
/// 见 [`YongShen`] 文档说明算法与流派分歧。
#[must_use]
pub fn determine_yongshen(
    day_master_stem: u8,
    month_branch: u8,
    strength: &Strength,
) -> YongShen {
    let dm_e = stem_element(day_master_stem);
    let bijie = dm_e;
    let yin = yin_xing_of(dm_e);
    let guan = guan_sha_of(dm_e);
    let cai = dm_e.controls();
    let shishang = dm_e.generates();
    let score = strength.score;
    let dm_name = STEMS[day_master_stem as usize];

    if score >= 60 {
        // 身强：取耗身。三候选按盘中分布升序排，最弱者为主用（补缺最有效）。
        let mut candidates: [(Element, &'static str); 3] = [
            (guan, "官杀"),
            (cai, "财"),
            (shishang, "食伤"),
        ];
        candidates.sort_by_key(|&(e, _)| wx_pct(&strength.wuxing, e));
        let (p_e, p_r) = candidates[0];
        let (s_e, s_r) = candidates[1];
        YongShen {
            method: "扶抑 · 身强宜耗".to_string(),
            primary_wuxing: p_e.name().to_string(),
            primary_role: p_r.to_string(),
            secondary_wuxing: Some(s_e.name().to_string()),
            secondary_role: Some(s_r.to_string()),
            avoid_wuxing: vec![yin.name().to_string(), bijie.name().to_string()],
            reasoning: format!(
                "日主{}{}（综合 {}），宜以耗身五行抑之；三候选（官杀{}/财{}/食伤{}）中{}{}最缺({}%)，补之最有效。忌印星{}生身、比劫{}帮身。",
                dm_name, strength.level, score,
                guan.name(), cai.name(), shishang.name(),
                p_r, p_e.name(), wx_pct(&strength.wuxing, p_e),
                yin.name(), bijie.name(),
            ),
        }
    } else if score <= 40 {
        // 身弱：取助身。印星优先（生身+化杀），比劫副选。
        YongShen {
            method: "扶抑 · 身弱宜扶".to_string(),
            primary_wuxing: yin.name().to_string(),
            primary_role: "印星".to_string(),
            secondary_wuxing: Some(bijie.name().to_string()),
            secondary_role: Some("比劫".to_string()),
            avoid_wuxing: vec![guan.name().to_string(), cai.name().to_string()],
            reasoning: format!(
                "日主{}{}（综合 {}），宜以助身五行扶之；印星{}双重作用（生身+化杀）优先，比劫{}副选。忌官杀{}克身、财{}损印。",
                dm_name, strength.level, score,
                yin.name(), bijie.name(), guan.name(), cai.name(),
            ),
        }
    } else {
        // 中和：调候为主，按月支寒燥取。
        let (target, note) = match month_branch {
            0..=1 | 11 => (Element::Fire, "亥子丑寒月 — 取火暖局"),
            5..=7 => (Element::Water, "巳午未燥月 — 取水润局"),
            2..=3 => (Element::Metal, "寅卯春木月 — 取金修剪"),
            8..=9 => (Element::Fire, "申酉秋金月 — 取火炼金"),
            _ => (dm_e, "辰戌杂气月 — 取日主同行扶身"),
        };
        YongShen {
            method: "调候为主".to_string(),
            primary_wuxing: target.name().to_string(),
            primary_role: "调候".to_string(),
            secondary_wuxing: None,
            secondary_role: None,
            avoid_wuxing: vec![],
            reasoning: format!(
                "日主{dm_name}中和（综合 {score}），扶抑余地小 → 看调候。{note}。",
            ),
        }
    }
}
