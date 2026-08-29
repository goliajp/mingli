//! 运势：t 时刻切片、吉凶判读与百年用神供给时序。

use super::*;

// ============================================================================
// Fortune：t 时刻运势切片（本命 + 大运 + 流年叠加旺衰 + 用神供给度） + 100 年时间序列
// 「拨杆 → 运势 → 用神供给」是旺衰 / 岁运叠加 / 用神在 t 时刻的统一聚合，
// 把「用神喜忌」从静态（出生即定的喜什么）升级为动态（t 时刻拿到多少 / 未来 100 年曲线）。
// ============================================================================

/// 吉凶判读（净增益分级）：由用神供给度计算 5 等级。
///
/// **算法**（基于命局所喜/所忌）：
/// `net = primary_pct + 0.5*secondary_pct − max_avoid_pct`
/// - 大吉：net ≥ +15（主用神远超忌神）
/// - 吉  ：+5 ≤ net < +15（主用神略胜）
/// - 平  ：-5 < net < +5（平衡）
/// - 凶  ：-15 < net ≤ -5（忌神略胜）
/// - 大凶：net ≤ -15（忌神远超）
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Judgment {
    /// 吉凶等级字面（大吉/吉/平/凶/大凶）。
    pub level: String,
    /// 净增益分（=主用神 + 0.5*副用神 − 最高忌神，可正可负）。
    pub score: i32,
    /// 一句话判读（基于结构事实给出有利/不利说明）。
    pub summary: String,
}

/// 判读核心：由 （主供给， 副供给， 最高忌神供给） → Judgment。
pub(crate) fn judge_from_supplies(primary: u32, secondary: Option<u32>, max_avoid: u32) -> Judgment {
    let p = i32::try_from(primary).unwrap_or(0);
    let s = secondary.and_then(|v| i32::try_from(v).ok()).unwrap_or(0);
    let a = i32::try_from(max_avoid).unwrap_or(0);
    let net = p + s / 2 - a;
    let (level, summary) = if net >= 15 {
        (
            "大吉",
            format!("主用神 {p}% 远超忌神 {a}%，流年大运对命局所喜五行供给充足，利于发展、决策、行动。"),
        )
    } else if net >= 5 {
        (
            "吉",
            format!("主用神 {p}% 略胜忌神 {a}%，流年大运扶持有力，顺势而为有利。"),
        )
    } else if net > -5 {
        (
            "平",
            format!("主用神 {p}% 与忌神 {a}% 相当，流年大运无明显加持也无明显损耗，守成之时。"),
        )
    } else if net > -15 {
        (
            "凶",
            format!("忌神 {a}% 略胜主用神 {p}%，流年大运对命局所忌五行偏强，谨慎决策、避免冒进。"),
        )
    } else {
        (
            "大凶",
            format!("忌神 {a}% 远超主用神 {p}%，流年大运对命局压制明显，宜守不宜攻、稳健渡过。"),
        )
    };
    Judgment {
        level: level.to_string(),
        score: net,
        summary,
    }
}

/// 五行名 → `WuxingPower` 字段查询。未知名返回 0（防御性 — 调用方应只用 5 标准名）。
pub(crate) fn wuxing_pct_by_name(w: &WuxingPower, name: &str) -> u32 {
    match name {
        "木" => w.wood,
        "火" => w.fire,
        "土" => w.earth,
        "金" => w.metal,
        "水" => w.water,
        _ => 0,
    }
}

/// 从大运 timeline 按年龄挑活动步。`pillars[i]` 在 `[start_age_i, start_age_{i+1})` 内活动；
/// 末步对未来无截断（传统大运十步即百年覆盖）。
pub(crate) fn active_dayun_step(dayun: Option<&DaYun>, age_years: f64) -> Option<(usize, String)> {
    let d = dayun?;
    let mut chosen: Option<(usize, &LuckPillar)> = None;
    for (i, p) in d.pillars.iter().enumerate() {
        if f64::from(p.start_age) <= age_years {
            chosen = Some((i, p));
        }
    }
    chosen.map(|(i, p)| (i, p.ganzhi.clone()))
}

/// Fortune：t 时刻运势切片。
///
/// 把本命旺衰 / 命格 / 岁运叠加 / 用神/喜忌 在 t 时刻一次给齐，
/// 供 web Fortune 视图直接渲染「拨杆动 → 运层动」的运势画面。
///
/// **算法**：
/// 1. 本命盘 = `compute(natal_input)`（见 [`compute`]）— 出生切片（固定底图）。
/// 2. t 时刻盘 = [`compute_at`] 在 `(t_year,..,t_tz)` 的 Moment — t 流年/流月/流日/流时四柱。
/// 3. 当前大运步 = 从本命大运 timeline 按年龄挑（三日折一年起运 / 每步十年）。
/// 4. 运层旺衰 = [`compute_strength_with_extras`]（本命四柱， extras=[当前大运柱， t 流年柱]）。
/// 5. 用神供给度 = `yun_strength.wuxing[本命主用神五行] / [副用神] / [忌神...]`。
///
/// 主用神供给度高 = t 时刻拿到喜用多 = **吉**；忌神供给度高 = **凶**。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct FortuneAt {
    /// 本命盘（出生切片，固定底图）。
    pub natal: BaziChart,
    /// t 时刻 bazi 盘（t 流年/流月/流日/流时四柱）。
    pub t_chart: BaziChart,
    /// t 时刻年龄（从 natal 出生时刻到 t_target 的浮点年）。
    pub age_years: f64,
    /// 当前活动大运步 index（本命大运 timeline `pillars[step]`）；无大运 → None。
    pub dayun_step: Option<usize>,
    /// 当前活动大运干支字面（同上）。
    pub dayun_ganzhi: Option<String>,
    /// t 时刻流年干支（取自 t_chart 年柱）。
    pub flow_year_ganzhi: String,
    /// 本命旺衰（等于 natal.strength）。
    pub ming_strength: Strength,
    /// 运层旺衰（本命 + 当前大运柱 + t 流年柱叠加）。
    pub yun_strength: Strength,
    /// 综合分差 = yun_strength.score − ming_strength.score（可正可负）。
    pub delta_score: i32,
    /// t 时刻主用神供给度 %（运层五行分布对本命主用神五行的占比）。
    pub primary_supply_pct: u32,
    /// t 时刻副用神供给度；调候法无副用神 → None。
    pub secondary_supply_pct: Option<u32>,
    /// t 时刻各忌神供给度，长度 = natal.yongshen.avoid_wuxing。
    pub avoid_supply_pcts: Vec<u32>,
    /// t 时刻吉凶判读（由 primary/secondary/avoid 供给度量化）。
    pub judgment: Judgment,
}

/// Fortune 入口：给定本命输入 + 目标时刻，聚合返回运势切片。
///
/// # Panics
///
/// 不会发生：本命四柱字面由内部 `compute()` 产出，必为合法干支，[`parse_ganzhi`] 解析永远成功。
#[must_use]
pub fn fortune_at(
    natal_input: BirthInput,
    t_year: i32,
    t_month: u32,
    t_day: u32,
    t_hour: u32,
    t_minute: u32,
    t_tz: f64,
) -> FortuneAt {
    let natal = compute(natal_input);
    let t_moment = Moment::new(t_year, t_month, t_day, t_hour, t_minute, t_tz);
    let t_chart = compute_at(&t_moment, natal_input.gender);

    // 年龄（浮点年）：简化用儒略日差 / 365.25。
    let birth_moment = Moment::new(
        natal_input.year, natal_input.month, natal_input.day,
        natal_input.hour, natal_input.minute, natal_input.tz,
    );
    let age_years = ((t_moment.jd_ut - birth_moment.jd_ut) / 365.25).max(0.0);

    let dayun_active = active_dayun_step(natal.dayun.as_ref(), age_years);
    let dayun_step = dayun_active.as_ref().map(|(i, _)| *i);
    let dayun_ganzhi = dayun_active.as_ref().map(|(_, gz)| gz.clone());

    let flow_year_ganzhi = t_chart.year.ganzhi.clone();

    // extras：本命四柱 + 当前大运柱 + t 流年柱（若解析成功）。
    let mut extras: Vec<GanZhi> = Vec::with_capacity(2);
    if let Some((_, ref gz_s)) = dayun_active
        && let Some(g) = parse_ganzhi(gz_s)
    {
        extras.push(g);
    }
    if let Some(g) = parse_ganzhi(&flow_year_ganzhi) { extras.push(g); }

    // 本命四柱 GanZhi（从 natal 重建，或用 t_chart 不行 — 必须用 natal 的）。
    let n_year = parse_ganzhi(&natal.year.ganzhi).expect("natal year_gz 应可解析");
    let n_month = parse_ganzhi(&natal.month.ganzhi).expect("natal month_gz 应可解析");
    let n_day = parse_ganzhi(&natal.day.ganzhi).expect("natal day_gz 应可解析");
    let n_hour = parse_ganzhi(&natal.hour.ganzhi).expect("natal hour_gz 应可解析");
    let yun_strength = compute_strength_with_extras(n_year, n_month, n_day, n_hour, &extras);

    let delta_score = i32::try_from(yun_strength.score).unwrap_or(0)
        - i32::try_from(natal.strength.score).unwrap_or(0);

    let primary_supply_pct = wuxing_pct_by_name(&yun_strength.wuxing, &natal.yongshen.primary_wuxing);
    let secondary_supply_pct = natal.yongshen.secondary_wuxing.as_ref()
        .map(|w| wuxing_pct_by_name(&yun_strength.wuxing, w));
    let avoid_supply_pcts: Vec<u32> = natal.yongshen.avoid_wuxing.iter()
        .map(|w| wuxing_pct_by_name(&yun_strength.wuxing, w))
        .collect();
    let max_avoid = avoid_supply_pcts.iter().copied().max().unwrap_or(0);
    let judgment = judge_from_supplies(primary_supply_pct, secondary_supply_pct, max_avoid);

    FortuneAt {
        ming_strength: natal.strength.clone(),
        natal,
        t_chart,
        age_years,
        dayun_step,
        dayun_ganzhi,
        flow_year_ganzhi,
        yun_strength,
        delta_score,
        primary_supply_pct,
        secondary_supply_pct,
        avoid_supply_pcts,
        judgment,
    }
}

/// 用神供给时间序列的一年点（供「100 年用神供给曲线」时序图）。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct FortuneTimelinePoint {
    /// 年龄（整数岁，0..=max_age）。
    pub age: u32,
    /// 对应公历年（出生年 + age，以正月初一近似不细分）。
    pub year: i32,
    /// 该年流年干支。
    pub flow_year_ganzhi: String,
    /// 当前大运步 index。
    pub dayun_step: Option<usize>,
    /// 当前大运干支。
    pub dayun_ganzhi: Option<String>,
    /// 该年运层综合分(0..=100)。
    pub yun_score: u32,
    /// 主用神供给度 %。
    pub primary_supply_pct: u32,
    /// 副用神供给度 %；调候法无副用神 → None。
    pub secondary_supply_pct: Option<u32>,
    /// 该年最高忌神供给度 %（各忌神中 max，作单线展示便利）。
    pub avoid_supply_pct: u32,
    /// 该年吉凶判读（由 supply 度量化）。
    pub judgment: Judgment,
}

/// 扫描 `[0..=max_age]` 每年点的运势供给（主用神/副用神/忌神最高）。
///
/// **简化**：每年生日时刻锚定该年流年干支（出生月日同年内 1 个流年柱）；不细分流月/流日。
/// 大运按 `start_age` 整数比对取活动步。
///
/// # Panics
///
/// 不会发生：本命四柱字面由内部 `compute()` 产出，必为合法干支，[`parse_ganzhi`] 解析永远成功。
#[must_use]
pub fn fortune_supply_timeline(natal_input: BirthInput, max_age: u32) -> Vec<FortuneTimelinePoint> {
    let natal = compute(natal_input);
    let n_year = parse_ganzhi(&natal.year.ganzhi).expect("natal year_gz 应可解析");
    let n_month = parse_ganzhi(&natal.month.ganzhi).expect("natal month_gz 应可解析");
    let n_day = parse_ganzhi(&natal.day.ganzhi).expect("natal day_gz 应可解析");
    let n_hour = parse_ganzhi(&natal.hour.ganzhi).expect("natal hour_gz 应可解析");

    let primary_w = natal.yongshen.primary_wuxing.clone();
    let secondary_w = natal.yongshen.secondary_wuxing.clone();
    let avoid_w = natal.yongshen.avoid_wuxing.clone();

    (0..=max_age)
        .map(|age| {
            // 流年：用「每年回到出生时刻」近似采样 — Moment 取出生月/日/时，只换公历年。
            let target_year = natal_input.year + i32::try_from(age).unwrap_or(0);
            let m = Moment::new(
                target_year,
                natal_input.month,
                natal_input.day,
                natal_input.hour,
                natal_input.minute,
                natal_input.tz,
            );
            let flow_chart = compute_at(&m, None);
            let flow_year_gz = flow_chart.year.ganzhi.clone();

            let dayun_active = active_dayun_step(natal.dayun.as_ref(), f64::from(age));
            let dayun_step = dayun_active.as_ref().map(|(i, _)| *i);
            let dayun_ganzhi = dayun_active.as_ref().map(|(_, gz)| gz.clone());

            let mut extras: Vec<GanZhi> = Vec::with_capacity(2);
            if let Some((_, ref gz_s)) = dayun_active
                && let Some(g) = parse_ganzhi(gz_s)
            {
                extras.push(g);
            }
            if let Some(g) = parse_ganzhi(&flow_year_gz) { extras.push(g); }

            let strength = compute_strength_with_extras(n_year, n_month, n_day, n_hour, &extras);
            let primary = wuxing_pct_by_name(&strength.wuxing, &primary_w);
            let secondary = secondary_w.as_ref().map(|w| wuxing_pct_by_name(&strength.wuxing, w));
            let avoid = avoid_w.iter()
                .map(|w| wuxing_pct_by_name(&strength.wuxing, w))
                .max().unwrap_or(0);

            let judgment = judge_from_supplies(primary, secondary, avoid);
            FortuneTimelinePoint {
                age,
                year: target_year,
                flow_year_ganzhi: flow_year_gz,
                dayun_step,
                dayun_ganzhi,
                yun_score: strength.score,
                primary_supply_pct: primary,
                secondary_supply_pct: secondary,
                avoid_supply_pct: avoid,
                judgment,
            }
        })
        .collect()
}
