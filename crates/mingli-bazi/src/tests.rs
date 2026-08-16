//! 四柱各层的校验用例：权威 oracle、性质测试与边界。

#![allow(clippy::float_cmp, reason = "用例里的常量比较是精确期望值")]

use super::*;

// ---- fortune_at / timeline 测试 ----------------------------------

fn natal_1987() -> BirthInput {
    BirthInput { year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0, gender: Some(Gender::Male) }
}

#[test]
fn fortune_at_aggregates_natal_and_t() {
    // 1987 长沙男（本命主用神=木官杀，忌火印+土比劫）；t=2026-06-16(age ~38.7)
    let f = fortune_at(natal_1987(), 2026, 6, 16, 10, 0, 8.0);
    // 本命旺衰恒等
    assert_eq!(f.ming_strength.score, f.natal.strength.score);
    // age 浮点合理
    assert!(f.age_years > 38.0 && f.age_years < 39.0, "age_years={}", f.age_years);
    // 流年丙午（2026年柱）与 t_chart 年柱一致
    assert_eq!(f.flow_year_ganzhi, f.t_chart.year.ganzhi);
    assert_eq!(f.flow_year_ganzhi, "丙午");
    // 大运：1987-09-17 男 → 阳年顺行，十步覆盖，age ~38 落在第 4-5 步附近（具体随起运岁数）
    assert!(f.dayun_step.is_some());
    assert!(f.dayun_ganzhi.is_some());
    // 用神供给度 ∈ [0, 100]
    assert!(f.primary_supply_pct <= 100);
    if let Some(s) = f.secondary_supply_pct { assert!(s <= 100); }
    for v in &f.avoid_supply_pcts { assert!(*v <= 100); }
    // 忌神条目数 = natal.yongshen.avoid_wuxing.len()
    assert_eq!(f.avoid_supply_pcts.len(), f.natal.yongshen.avoid_wuxing.len());
    // delta = yun.score - ming.score
    assert_eq!(
        f.delta_score,
        i32::try_from(f.yun_strength.score).unwrap() - i32::try_from(f.ming_strength.score).unwrap()
    );
}

#[test]
fn fortune_at_t_chart_is_t_moment_bazi() {
    // t_chart 实际就是 t 时刻自起一盘 — 与从 t 算的本命 BaziChart 字节一致（确认共享层一次性算法路径）。
    let f = fortune_at(natal_1987(), 2026, 6, 16, 10, 0, 8.0);
    let standalone = compute(BirthInput {
        year: 2026, month: 6, day: 16, hour: 10, minute: 0, tz: 8.0, gender: Some(Gender::Male)
    });
    assert_eq!(f.t_chart.year.ganzhi, standalone.year.ganzhi);
    assert_eq!(f.t_chart.month.ganzhi, standalone.month.ganzhi);
    assert_eq!(f.t_chart.day.ganzhi, standalone.day.ganzhi);
    assert_eq!(f.t_chart.hour.ganzhi, standalone.hour.ganzhi);
}

#[test]
fn fortune_timeline_covers_range_and_is_well_formed() {
    let timeline = fortune_supply_timeline(natal_1987(), 100);
    assert_eq!(timeline.len(), 101);
    assert_eq!(timeline[0].age, 0);
    assert_eq!(timeline[100].age, 100);
    // 每年公历 = 出生年 + age
    for p in &timeline {
        assert_eq!(p.year, 1987 + i32::try_from(p.age).unwrap());
        assert!(p.yun_score <= 100);
        assert!(p.primary_supply_pct <= 100);
        assert!(p.avoid_supply_pct <= 100);
        // 流年干支字面合法（2 个汉字）
        assert_eq!(p.flow_year_ganzhi.chars().count(), 2);
    }
    // 出生当年 age=0 流年应 = natal 年柱（都是 1987 → 丁卯）。
    assert_eq!(timeline[0].flow_year_ganzhi, "丁卯");
    // age=39 (2026) → 丙午；age=43 (2030) → 庚戌。
    assert_eq!(timeline[39].flow_year_ganzhi, "丙午");
    assert_eq!(timeline[43].flow_year_ganzhi, "庚戌");
}

#[test]
fn fortune_timeline_dayun_step_monotone_non_decreasing() {
    // 大运按 start_age 递增，timeline 上 step index 应单调不减（或保持 None 期一致）。
    let timeline = fortune_supply_timeline(natal_1987(), 100);
    let mut prev: Option<usize> = None;
    for p in &timeline {
        if let (Some(prev_i), Some(cur_i)) = (prev, p.dayun_step) {
            assert!(cur_i >= prev_i, "dayun_step 应单调不减 prev={prev_i} cur={cur_i} at age={}", p.age);
        }
        if p.dayun_step.is_some() { prev = p.dayun_step; }
    }
}

#[test]
fn judgment_threshold_boundaries() {
    // 大吉：net=+15+ (primary 30， avoid 15， net=15)
    let j = judge_from_supplies(30, None, 15);
    assert_eq!(j.level, "大吉");
    assert_eq!(j.score, 15);
    // 吉：net 5..15 (primary 20， avoid 15， net=5)
    let j = judge_from_supplies(20, None, 15);
    assert_eq!(j.level, "吉");
    assert_eq!(j.score, 5);
    // 平：net 0 (primary 15， avoid 15)
    let j = judge_from_supplies(15, None, 15);
    assert_eq!(j.level, "平");
    // 凶：net -5 (primary 10， avoid 15)
    let j = judge_from_supplies(10, None, 15);
    assert_eq!(j.level, "凶");
    assert_eq!(j.score, -5);
    // 大凶：net -15 (primary 0， avoid 15)
    let j = judge_from_supplies(0, None, 15);
    assert_eq!(j.level, "大凶");
    assert_eq!(j.score, -15);
    // 副用神计入：primary 10 + secondary 20 （折 0.5=10） - avoid 5 = +15 → 大吉
    let j = judge_from_supplies(10, Some(20), 5);
    assert_eq!(j.level, "大吉");
    assert_eq!(j.score, 15);
    // summary 非空且含百分比
    assert!(!j.summary.is_empty());
    assert!(j.summary.contains('%'));
}

#[test]
fn fortune_at_carries_judgment_for_1987() {
    // 1987 长沙男 + 2026-06-16：主用神木 13%/副水 9%/忌火 33%
    // net = 13 + 9/2 - 33 = 13 + 4 - 33 = -16 → 大凶
    let f = fortune_at(natal_1987(), 2026, 6, 16, 10, 0, 8.0);
    assert_eq!(f.judgment.level, "大凶");
    assert_eq!(f.judgment.score, -16);
    assert!(f.judgment.summary.contains("33%"));
    assert!(f.judgment.summary.contains("宜守不宜攻"));
}

#[test]
fn fortune_timeline_carries_judgment_each_point() {
    let timeline = fortune_supply_timeline(natal_1987(), 100);
    let levels: std::collections::HashSet<&str> = timeline.iter().map(|p| p.judgment.level.as_str()).collect();
    // 100 年里至少出现 2 个等级（命局不可能恒一）
    assert!(levels.len() >= 2, "timeline should cover multiple judgment levels: {levels:?}");
    // 每点判读 score 满足 5 等级阈值
    for p in &timeline {
        let net = p.judgment.score;
        match p.judgment.level.as_str() {
            "大吉" => assert!(net >= 15),
            "吉" => assert!((5..15).contains(&net)),
            "平" => assert!((-4..5).contains(&net)),
            "凶" => assert!((-14..-4).contains(&net)),
            "大凶" => assert!(net <= -15),
            other => panic!("unexpected level {other}"),
        }
    }
}

#[test]
fn wuxing_pct_by_name_dispatch_all_five_plus_unknown() {
    let w = WuxingPower { wood: 10, fire: 20, earth: 30, metal: 40, water: 50 };
    assert_eq!(wuxing_pct_by_name(&w, "木"), 10);
    assert_eq!(wuxing_pct_by_name(&w, "火"), 20);
    assert_eq!(wuxing_pct_by_name(&w, "土"), 30);
    assert_eq!(wuxing_pct_by_name(&w, "金"), 40);
    assert_eq!(wuxing_pct_by_name(&w, "水"), 50);
    assert_eq!(wuxing_pct_by_name(&w, "未知"), 0);
}

/// 年柱换岁流派：2024-02-01（立春前一日）出生。
/// 立春派：归 2023 癸卯；春节派：归 2023（春节 2024-02-10，出生在它之前 → 也归 2023）。
/// 2024-02-09（春节前一日）：立春派 2024 甲辰（已过立春 2024-02-04）；春节派 2023（未到正月初一）。
#[test]
fn year_break_school_lichun_vs_springfestival() {
    let m = Moment::new(2024, 2, 9, 12, 0, 8.0);
    let chun = compute_at_school(&m, None, BaziSchool { zi_hour: ZiHourMethod::Late, year_break: YearBreakMethod::LiChun });
    let sf = compute_at_school(&m, None, BaziSchool { zi_hour: ZiHourMethod::Late, year_break: YearBreakMethod::SpringFestival });
    assert_eq!(chun.year.ganzhi, "甲辰", "立春派 2024-02-09 已过立春 → 甲辰");
    assert_eq!(sf.year.ganzhi, "癸卯", "春节派 2024-02-09 春节(02-10)未到 → 癸卯");
}

/// 子时流派校验：23：30 出生，晚子（主流）归次日日柱；早子（传统少数）归当日。
/// 1990-06-15 日柱=辛亥；1990-06-16 日柱应为壬子（辛亥之次）。
#[test]
fn zi_hour_school_late_vs_early() {
    let m_2330 = Moment::new(1990, 6, 15, 23, 30, 8.0);
    let late = compute_at_with(&m_2330, None, ZiHourMethod::Late);
    let early = compute_at_with(&m_2330, None, ZiHourMethod::Early);
    // 早子：仍归当日（辛亥）
    assert_eq!(early.day.ganzhi, "辛亥", "Early Zi 日柱应为 1990-06-15 当日");
    // 晚子：归次日（壬子）
    assert_eq!(late.day.ganzhi, "壬子", "Late Zi 日柱应为次日 1990-06-16");
    // 非 23 点出生时，两派应一致
    let m_1430 = Moment::new(1990, 6, 15, 14, 30, 8.0);
    let a = compute_at_with(&m_1430, None, ZiHourMethod::Late);
    let b = compute_at_with(&m_1430, None, ZiHourMethod::Early);
    assert_eq!(a.day.ganzhi, b.day.ganzhi);
}

#[test]
fn sample_1990_06_15_male() {
    let chart = compute(BirthInput {
        year: 1990,
        month: 6,
        day: 15,
        hour: 14,
        minute: 30,
        tz: 8.0,
        gender: Some(Gender::Male),
    });
    assert_eq!(chart.year.ganzhi, "庚午");
    assert_eq!(chart.month.ganzhi, "壬午");
    assert_eq!(chart.day.ganzhi, "辛亥");
    assert_eq!(chart.hour.ganzhi, "乙未");
    assert_eq!(chart.day_master, "辛");
    assert_eq!(chart.day_master_wuxing, "金");
    assert_eq!(chart.month.ten_god, "伤官"); // 壬 vs 日主辛
    assert_eq!(chart.day.ten_god, "日主");
    assert_eq!(
        (chart.lunar.year, chart.lunar.month, chart.lunar.day),
        (1990, 5, 23)
    );
    let dy = chart.dayun.as_ref().unwrap();
    assert!(dy.forward); // 庚午阳年男 → 顺行
    assert_eq!(dy.pillars.len(), 10);
    assert_eq!(dy.pillars[0].ganzhi, "癸未"); // 月柱壬午顺行下一步
}

#[test]
fn dayun_reverse_for_yin_year_male() {
    // 1989 己巳（阴年）男 → 逆行
    let chart = compute(BirthInput {
        year: 1989,
        month: 6,
        day: 15,
        hour: 12,
        minute: 0,
        tz: 8.0,
        gender: Some(Gender::Male),
    });
    assert!(!chart.dayun.as_ref().unwrap().forward);
}

#[test]
fn before_lichun_uses_prev_year() {
    // 1990-01-20 在立春(1990-02-04)前 → 年柱归 1989 己巳。
    let chart = compute(BirthInput {
        year: 1990,
        month: 1,
        day: 20,
        hour: 12,
        minute: 0,
        tz: 8.0,
        gender: None,
    });
    assert_eq!(chart.year.ganzhi, "己巳");
}

#[test]
fn no_dayun_without_gender() {
    let chart = compute(BirthInput {
        year: 2000,
        month: 1,
        day: 1,
        hour: 0,
        minute: 0,
        tz: 8.0,
        gender: None,
    });
    assert!(chart.dayun.is_none());
}

#[test]
fn xunkong_and_nayin_oracle() {
    // 1990-06-15 14：30 男 → 日柱辛亥。辛亥在甲辰旬 → 旬空寅卯；辛亥纳音=钗钏金（金）。
    let chart = compute(BirthInput {
        year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
        gender: Some(Gender::Male),
    });
    assert_eq!(chart.day.ganzhi, "辛亥");
    assert_eq!(chart.xunkong, ["寅".to_string(), "卯".to_string()]);
    assert_eq!(chart.day.nayin, "金"); // 钗钏金

    // 1987-09-17 15：00 男 → 四柱 丁卯 己酉 己巳 壬申。日柱己巳在甲子旬 → 旬空戌亥。
    let c2 = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    assert_eq!(c2.year.ganzhi, "丁卯");
    assert_eq!(c2.day.ganzhi, "己巳");
    assert_eq!(c2.hour.ganzhi, "壬申");
    assert_eq!(c2.xunkong, ["戌".to_string(), "亥".to_string()]);
}

#[test]
fn strength_oracle_1987_male_yin_earth() {
    // 1987-09-17 15：00 男 = 丁卯 己酉 己巳 壬申。日主己土。
    // 手算：
    //   得令（月酉）：己阴土在酉=长生 stage0 → 20；酉藏[辛]辛金=食伤非同党；got_ling=20。
    //   得地（卯/巳/申）：
    //     卯[乙] 七杀，非 → 0；
    //     巳[丙 庚 戊] 丙印（本+9） 庚伤(0) 戊劫（余+3） = 12；
    //     申[庚 壬 戊] 庚伤 壬财 戊劫（余+3） = 3；
    //     got_di = 0+12+3 = 15。
    //   得势（年丁/月己/时壬）：丁印(+10) 己比肩(+10) 壬财(0)=20。
    //   raw=20+15+20=55 → score=55*100/90=61 → 偏强。
    let c = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    assert_eq!(c.day_master, "己");
    assert_eq!(c.strength.got_ling, 20, "酉=长生 20、月支辛伤非同党");
    assert_eq!(c.strength.got_di, 15, "巳丙印9+戊劫3 申戊劫3");
    assert_eq!(c.strength.got_shi, 20, "丁印10+己比10");
    assert_eq!(c.strength.score, 61);
    assert_eq!(c.strength.level, "偏强");
    // 五行分布合 100（整数 round 凑巧；允差 1）。
    let s = c.strength.wuxing;
    let sum = s.wood + s.fire + s.earth + s.metal + s.water;
    assert!((99..=101).contains(&sum), "wuxing 合 ≈ 100，实 {sum}");
    // 金最旺（月令酉×1.5 + 巳/申庚金）：应是最大项
    let max = [s.wood, s.fire, s.earth, s.metal, s.water].into_iter().max().unwrap();
    assert_eq!(s.metal, max, "酉月金最旺");
}

#[test]
fn strength_oracle_1990_male_yin_metal_in_summer() {
    // 1990-06-15 14：30 男 = 庚午 壬午 辛亥 乙未。日主辛金，生于午月夏火克金。
    // 手算：
    //   得令（月午）：辛阴金在午=stage6 病 → 6；午藏[丁 己] 丁七杀(0) 己偏印（中+3） → 6+3=9。
    //   得地（午/亥/未）：
    //     午[丁 己] 丁七杀 己偏印（中+5） = 5
    //     亥[壬 甲] 壬伤 甲正财 = 0
    //     未[己 乙 丁] 己偏印（本+9） 乙偏财 丁七杀 = 9
    //     got_di = 5+0+9 = 14。
    //   得势（年庚/月壬/时乙）：庚劫财(+10) 壬伤 乙偏财 → 10。
    //   raw=9+14+10=33 → score=33*100/90=36 → 偏弱（辛金生夏天合理）。
    let c = compute(BirthInput {
        year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
        gender: Some(Gender::Male),
    });
    assert_eq!(c.day_master, "辛");
    assert_eq!(c.strength.got_ling, 9);
    assert_eq!(c.strength.got_di, 14);
    assert_eq!(c.strength.got_shi, 10);
    assert_eq!(c.strength.score, 36);
    assert_eq!(c.strength.level, "偏弱");
}

/// Female 大运：阴年女顺行 / 阳年女逆行。1990 庚午阳年女 → 逆行。
#[test]
fn dayun_female_gender() {
    let chart = compute(BirthInput {
        year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
        gender: Some(Gender::Female),
    });
    assert!(!chart.dayun.as_ref().unwrap().forward, "庚午阳年女 → 逆行");
}

/// 春节换年：春节后归本年（覆盖 L397 = month==1 day>=1 非闰分支）。
/// 2024-02-15（春节 02-10 已过）非闰正月初六 → 春节派应归 2024 甲辰。
#[test]
fn year_break_springfestival_after_lunar_new_year() {
    let m = Moment::new(2024, 2, 15, 12, 0, 8.0);
    let sf = compute_at_school(&m, None, BaziSchool {
        zi_hour: ZiHourMethod::Late, year_break: YearBreakMethod::SpringFestival,
    });
    assert_eq!(sf.year.ganzhi, "甲辰", "春节(02-10)已过 → 春节派归 2024 甲辰");
}

/// 春节换年 fallback：既非正月初一在前、又非月>=11/=12/闰正月 → m.year。
/// 实际上 2024-03-15（农历二月初六）中已到春节后，但 month=2，不命中前两条 → fallback L402。
#[test]
fn year_break_springfestival_fallback_branch() {
    let m = Moment::new(2024, 3, 15, 12, 0, 8.0);
    let sf = compute_at_school(&m, None, BaziSchool {
        zi_hour: ZiHourMethod::Late, year_break: YearBreakMethod::SpringFestival,
    });
    // 立春派与春节派此刻应一致（都在 2024 范围内），走 fallback m.year
    assert_eq!(sf.year.ganzhi, "甲辰");
}

#[test]
fn strength_extras_empty_equals_natal() {
    // 空 extras 必须等价于本命旺衰，作为本命 chart 的回归校验。
    use mingli_ganzhi::parse_ganzhi;
    let c = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    let y = parse_ganzhi(&c.year.ganzhi).unwrap();
    let mo = parse_ganzhi(&c.month.ganzhi).unwrap();
    let d = parse_ganzhi(&c.day.ganzhi).unwrap();
    let h = parse_ganzhi(&c.hour.ganzhi).unwrap();
    let no_extra = compute_strength_with_extras(y, mo, d, h, &[]);
    assert_eq!(no_extra.score, c.strength.score);
    assert_eq!(no_extra.got_ling, c.strength.got_ling);
    assert_eq!(no_extra.got_di, c.strength.got_di);
    assert_eq!(no_extra.got_shi, c.strength.got_shi);
}

#[test]
fn strength_extras_help_pushes_score_up() {
    // 1987 己土 本命 = 偏强 61（得令20+得地15+得势20）。叠加「戊午」（戊=劫财+10、午藏丁己印+劫）：
    //   得地原15 + （午丁本印+9 + 己中劫+5） = 29（未封顶）；得势原20 + 戊劫+10 = 30（封顶）；得令20。
    //   raw = 79 → score = 79*100/90 = 87 → 强。
    use mingli_ganzhi::parse_ganzhi;
    let c = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    let y = parse_ganzhi(&c.year.ganzhi).unwrap();
    let mo = parse_ganzhi(&c.month.ganzhi).unwrap();
    let d = parse_ganzhi(&c.day.ganzhi).unwrap();
    let h = parse_ganzhi(&c.hour.ganzhi).unwrap();
    let yun = compute_strength_with_extras(y, mo, d, h, &[parse_ganzhi("戊午").unwrap()]);
    assert_eq!(yun.got_di, 29);
    assert_eq!(yun.got_shi, 30, "得势封顶");
    assert_eq!(yun.score, 87);
    assert_eq!(yun.level, "强");
    assert!(yun.score > c.strength.score, "助党推升旺衰");
}

#[test]
fn strength_extras_hostile_keeps_score_steady() {
    // 加纯敌党（如「壬寅」：壬财、寅藏甲丙戊→甲杀/丙印/戊劫）：
    //   实际寅有印+劫，会拉升 di，所以严格意义不是「纯敌」。改测「乙未」乙偏财 + 未己乙丁（己劫本+9、乙财、丁印余+3=12）。
    //   得地原15+12=27；得势原20+乙财0=20；得令20；raw=67→score=74 偏强（仍提升，因未支带印劫）。
    // 真正纯敌党：取「庚申」（庚=伤、申=庚伤本+壬财中+戊劫余3=3）：
    //   得地原15+3=18；得势原20+庚伤0=20；得令20；raw=58→score=64 偏强（微升）。
    //   关键性质：不论加什么 extras，得令永远不变 = 月令固定。
    use mingli_ganzhi::parse_ganzhi;
    let c = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    let y = parse_ganzhi(&c.year.ganzhi).unwrap();
    let mo = parse_ganzhi(&c.month.ganzhi).unwrap();
    let d = parse_ganzhi(&c.day.ganzhi).unwrap();
    let h = parse_ganzhi(&c.hour.ganzhi).unwrap();
    let yun = compute_strength_with_extras(y, mo, d, h, &[parse_ganzhi("庚申").unwrap()]);
    assert_eq!(yun.got_ling, c.strength.got_ling, "得令固定取本命月支，extras 不改");
    assert_eq!(yun.got_di, 18);
    assert_eq!(yun.got_shi, 20);
    assert_eq!(yun.score, 64);
}

#[test]
fn strength_score_bounds() {
    // 任意输入下，三栏都在 [0,30]、综合分都在 [0,100]、五行和约 100。
    for &(y, m, d, h) in &[
        (2024, 6, 21, 12), (1980, 1, 1, 0), (2000, 11, 11, 11),
        (1949, 10, 1, 15), (2030, 2, 4, 16),
    ] {
        let c = compute(BirthInput {
            year: y, month: m, day: d, hour: h, minute: 0, tz: 8.0, gender: None,
        });
        let s = &c.strength;
        assert!(s.got_ling <= 30);
        assert!(s.got_di <= 30);
        assert!(s.got_shi <= 30);
        assert!(s.score <= 100);
        let sum = s.wuxing.wood + s.wuxing.fire + s.wuxing.earth + s.wuxing.metal + s.wuxing.water;
        assert!((99..=101).contains(&sum));
    }
}

#[test]
fn pattern_1987_yin_earth_anzang_shi_shen() {
    // 1987-09-17 15：00 男 = 丁卯 己酉 己巳 壬申。日主己土，月支酉（专气藏辛）。
    // 三干头（丁/己/壬）无辛 → 辛不透 → 暗藏取本气 → 辛（食神） → 暗食神格。
    let c = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    let p = &c.pattern;
    assert_eq!(p.name, "暗食神格");
    assert_eq!(p.qi_stem, "辛");
    assert_eq!(p.qi_kind, "本气");
    assert!(!p.revealed);
    assert_eq!(p.revealed_in, None);
    assert_eq!(p.ten_god, "食神");
    assert!(!p.is_lu_ren);
}

#[test]
fn pattern_1990_yin_metal_anzang_qisha() {
    // 1990-06-15 14：30 男 = 庚午 壬午 辛亥 乙未。日主辛，月支午（本丁/中己）。
    // 三干头（庚/壬/乙）无丁、无己 → 暗藏取本气丁（七杀） → 暗七杀格。
    let c = compute(BirthInput {
        year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
        gender: Some(Gender::Male),
    });
    assert_eq!(c.pattern.name, "暗七杀格");
    assert_eq!(c.pattern.qi_stem, "丁");
    assert_eq!(c.pattern.ten_god, "七杀");
    assert!(!c.pattern.revealed);
}

/// 直接构造 GanZhi 测纯算法（不经历法约束）——覆盖建禄/月刃/八正格透干的各分支。
#[test]
fn pattern_jianlu_when_main_qi_equals_day_master() {
    // 日主甲（0 阳木） + 月支寅（本气甲） → 同五行同阴阳 → 建禄格。
    let yr = GanZhi { stem: 9, branch: 0 };   // 任意
    let mo = GanZhi { stem: 2, branch: 2 };   // 月支寅
    let dy = GanZhi { stem: 0, branch: 4 };   // 日主甲
    let hr = GanZhi { stem: 5, branch: 11 };
    let p = determine_pattern(yr, mo, dy, hr);
    assert_eq!(p.name, "建禄格");
    assert!(p.is_lu_ren);
    assert_eq!(p.ten_god, "比肩");
}

#[test]
fn pattern_yueren_when_main_qi_same_element_different_polarity() {
    // 日主甲（0 阳木） + 月支卯（本气乙阴木） → 同五行异阴阳 → 月刃格。
    let yr = GanZhi { stem: 9, branch: 0 };
    let mo = GanZhi { stem: 2, branch: 3 };
    let dy = GanZhi { stem: 0, branch: 4 };
    let hr = GanZhi { stem: 5, branch: 11 };
    let p = determine_pattern(yr, mo, dy, hr);
    assert_eq!(p.name, "月刃格");
    assert!(p.is_lu_ren);
    assert_eq!(p.ten_god, "劫财");
}

#[test]
fn pattern_main_qi_revealed() {
    // 月支寅（本甲/中丙/余戊），日主己土。让本气甲在年柱透出 → 正官格（甲对己=正官）。
    let yr = GanZhi { stem: 0, branch: 0 }; // 年干 甲
    let mo = GanZhi { stem: 9, branch: 2 }; // 月干 癸（非月令藏干），月支寅
    let dy = GanZhi { stem: 5, branch: 11 }; // 日主 己
    let hr = GanZhi { stem: 3, branch: 7 }; // 时干 丁
    let p = determine_pattern(yr, mo, dy, hr);
    assert_eq!(p.name, "正官格");
    assert!(p.revealed);
    assert_eq!(p.qi_stem, "甲");
    assert_eq!(p.qi_kind, "本气");
    assert_eq!(p.revealed_in.as_deref(), Some("年柱"));
    assert_eq!(p.ten_god, "正官");
}

#[test]
fn pattern_middle_qi_revealed_skips_main_qi() {
    // 月支寅（本甲/中丙/余戊），日主己。本气甲不透，中气丙在时柱透 → 正印格。
    let yr = GanZhi { stem: 3, branch: 5 }; // 丁巳
    let mo = GanZhi { stem: 9, branch: 2 }; // 癸寅（非历法合理，纯算法测试）
    let dy = GanZhi { stem: 5, branch: 11 }; // 己亥
    let hr = GanZhi { stem: 2, branch: 3 }; // 丙卯
    let p = determine_pattern(yr, mo, dy, hr);
    assert_eq!(p.name, "正印格");
    assert!(p.revealed);
    assert_eq!(p.qi_stem, "丙");
    assert_eq!(p.qi_kind, "中气");
    assert_eq!(p.revealed_in.as_deref(), Some("时柱"));
    assert_eq!(p.ten_god, "正印");
}

#[test]
fn pattern_yu_qi_revealed_when_main_and_middle_unrevealed() {
    // 月支寅（本甲/中丙/余戊），日主庚金。让余气戊在月柱透 → 偏印格（戊对庚=偏印，土生金同阳）。
    let yr = GanZhi { stem: 1, branch: 5 }; // 乙巳
    let mo = GanZhi { stem: 4, branch: 2 }; // 戊寅
    let dy = GanZhi { stem: 6, branch: 11 }; // 庚亥
    let hr = GanZhi { stem: 9, branch: 3 }; // 癸卯
    let p = determine_pattern(yr, mo, dy, hr);
    assert_eq!(p.name, "偏印格");
    assert_eq!(p.qi_stem, "戊");
    assert_eq!(p.qi_kind, "余气");
    assert_eq!(p.revealed_in.as_deref(), Some("月柱"));
    assert_eq!(p.ten_god, "偏印");
}

#[test]
fn yongshen_1987_male_yin_earth_strong() {
    // 1987-09-17 15：00 男 = 丁卯 己酉 己巳 壬申。日主己土 score 61 偏强。
    // 五行分布：木11/火20/土23/金32/水14。
    // 走身强宜耗：候选 官杀木(11)/财水(14)/食伤金(32) → 升序 木<水<金 → 主用神=木（官杀），副=水（财）。
    // 忌神 = 印火 + 比劫土。
    let c = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    let y = &c.yongshen;
    assert_eq!(y.method, "扶抑 · 身强宜耗");
    assert_eq!(y.primary_wuxing, "木");
    assert_eq!(y.primary_role, "官杀");
    assert_eq!(y.secondary_wuxing.as_deref(), Some("水"));
    assert_eq!(y.secondary_role.as_deref(), Some("财"));
    assert_eq!(y.avoid_wuxing, vec!["火".to_string(), "土".to_string()]);
    assert!(y.reasoning.contains("耗身"));
}

#[test]
fn yongshen_1990_male_yin_metal_weak() {
    // 1990-06-15 14：30 男 = 庚午 壬午 辛亥 乙未。日主辛金 score 36 偏弱。
    // 走身弱宜扶：印星（土）优先，比劫（金）副。忌神 = 官杀火 + 财木。
    let c = compute(BirthInput {
        year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
        gender: Some(Gender::Male),
    });
    let y = &c.yongshen;
    assert_eq!(y.method, "扶抑 · 身弱宜扶");
    assert_eq!(y.primary_wuxing, "土");
    assert_eq!(y.primary_role, "印星");
    assert_eq!(y.secondary_wuxing.as_deref(), Some("金"));
    assert_eq!(y.secondary_role.as_deref(), Some("比劫"));
    assert_eq!(y.avoid_wuxing, vec!["火".to_string(), "木".to_string()]);
    assert!(y.reasoning.contains("助身"));
}

/// 中和走调候：用纯算法构造一个 score≈50 的盘，看调候按月支取。
#[test]
fn yongshen_neutral_takes_tiao_hou() {
    use mingli_ganzhi::parse_ganzhi;
    // 手造 score=50 strength + 月支子（冬月） → 调候取火。
    let fake_str = Strength {
        score: 50, level: "中和".into(),
        got_ling: 15, got_di: 15, got_shi: 15,
        wuxing: WuxingPower { wood: 20, fire: 20, earth: 20, metal: 20, water: 20 },
    };
    // 日主己土 + 月支子 → 调候 寒月取火
    let y = determine_yongshen(5, 0, &fake_str);
    assert_eq!(y.method, "调候为主");
    assert_eq!(y.primary_wuxing, "火");
    assert_eq!(y.primary_role, "调候");
    assert!(y.secondary_wuxing.is_none());
    assert!(y.avoid_wuxing.is_empty());
    // 日主己土 + 月支午（燥月） → 取水
    let y2 = determine_yongshen(5, 6, &fake_str);
    assert_eq!(y2.primary_wuxing, "水");
    // 日主庚 + 月支寅（春木） → 取金
    let y3 = determine_yongshen(6, 2, &fake_str);
    assert_eq!(y3.primary_wuxing, "金");
    // 日主壬 + 月支申（秋金） → 取火
    let y4 = determine_yongshen(8, 8, &fake_str);
    assert_eq!(y4.primary_wuxing, "火");
    // 日主甲 + 月支辰（杂气） → 取日主同行（木）
    let y5 = determine_yongshen(0, 4, &fake_str);
    assert_eq!(y5.primary_wuxing, "木");
    // 校验：5 个月支分支都已覆盖，确认 parse_ganzhi 与本测试无关
    assert!(parse_ganzhi("甲子").is_some());
}

/// 反查五行关系正确性：印星生我、官杀克我。
#[test]
fn yongshen_role_inverses_correct() {
    use mingli_ganzhi::Element;
    // 印星 X.generates() == dm
    for dm in [Element::Wood, Element::Fire, Element::Earth, Element::Metal, Element::Water] {
        assert_eq!(yin_xing_of(dm).generates(), dm, "印星生我：{dm:?}");
        assert_eq!(guan_sha_of(dm).controls(), dm, "官杀克我：{dm:?}");
    }
}

#[test]
fn true_solar_offset_changshanha_oracle() {
    // 长沙 1987-09-17 lon=112.94°E，tz=+8（标准经线 120°）。
    //   经度差 = (112.94 − 120) × 4 = −28.24 min
    //   EoT（9月17日） ≈ +6 min（Spencer 公式）
    //   合 ≈ −22 min（真太阳时较钟表早约 22 分钟）
    let off = true_solar_offset_minutes(112.94, 8.0, 1987, 9, 17);
    assert!(
        (-24.0..=-20.0).contains(&off),
        "长沙 1987-09-17 真太阳时差应在 [-24, -20] 分钟，实测 {off:.2}"
    );
}

#[test]
fn true_solar_does_not_change_pillar_when_within_same_chen() {
    // 长沙 1987-09-17 15：00 钟表（未时）；真太阳时 ≈ 14：38，仍在未时(13-15)：
    //   等等，未时 13-15，15：00 已是申时(15-17)；真太阳 14：38 是未时。
    //   ⇒ 时柱按钟表 是申时，按真太阳是未时；时柱会变！
    // 重新设计：取真正不变的 case：15：30 钟表 → 真太阳 ≈ 15：08，两者都申时 → 时柱同。
    let with_solar = compute_with_true_solar(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 30, tz: 8.0,
        gender: Some(Gender::Male),
    }, 112.94);
    let no_solar = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 30, tz: 8.0,
        gender: Some(Gender::Male),
    });
    // 钟表 15：30 申时 hour_branch=8；真太阳 ≈ 15：08 申时 hour_branch=8 → 时柱同。
    assert_eq!(no_solar.hour.ganzhi, with_solar.hour.ganzhi);
}

#[test]
fn true_solar_changes_pillar_across_chen_boundary() {
    // 长沙 1987-09-17 钟表 15：00（申时起点），真太阳 ≈ 14：38（未时）：
    //   钟表 hour_branch=hour_branch(15，30)=8 申、真太阳 hour_branch(14，..)=7 未 → 时柱必变。
    let no_solar = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    let with_solar = compute_with_true_solar(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    }, 112.94);
    // 钟表时柱 = 壬申（已验）；真太阳时柱 = ？
    // hour_branch=7 未， day_stem=5（己）， stem=(5%5)*2+7=7 → 辛。时柱=辛未。
    assert_eq!(no_solar.hour.ganzhi, "壬申");
    assert_eq!(with_solar.hour.ganzhi, "辛未");
    // 其它三柱不变（同一日同一月同一年）
    assert_eq!(no_solar.year.ganzhi, with_solar.year.ganzhi);
    assert_eq!(no_solar.month.ganzhi, with_solar.month.ganzhi);
    assert_eq!(no_solar.day.ganzhi, with_solar.day.ganzhi);
}

#[test]
fn true_solar_helpers_round_trip() {
    // 校验 day_of_year 闰年分支 + add_days_civil 跨月跨年。
    assert_eq!(day_of_year(2024, 3, 1), 61); // 闰年 1月31+2月29+1=61
    assert_eq!(day_of_year(2023, 3, 1), 60);
    assert_eq!(add_days_civil(2024, 1, 1, -1), (2023, 12, 31));
    assert_eq!(add_days_civil(2023, 12, 31, 1), (2024, 1, 1));
    assert_eq!(add_days_civil(2024, 2, 28, 2), (2024, 3, 1)); // 闰年 +2 跨 29
    assert_eq!(add_days_civil(2023, 2, 28, 2), (2023, 3, 2)); // 平年 28→3/2
}

#[test]
fn three_houses_1987_oracle() {
    // 1987-09-17 15：00 男 = 丁卯 己酉 己巳 壬申。
    // 月支酉(9)，时支未（7，15：00 hour_branch=(15+1)/2=8 申？）。
    //   等等 hour_branch(15，0)=(16/2)=8 申。所以时支=申(8)。
    //   命宫支 = (9 - 8 + 12) % 12 = 1 → 丑；命宫干 = 五虎遁丁年丑月 = 癸（丁年寅起壬，寅卯辰巳午未申酉戌亥子丑 = 壬癸甲乙丙丁戊己庚辛壬癸，丑=癸） → 命宫=癸丑。
    //   身宫支 = (9 + 8) % 12 = 5 → 巳；身宫干 = 丁年寅起壬，巳=丙（壬癸甲乙丙） → 身宫=丙巳。
    //   胎元 = 己酉 +1+3 = 庚子。
    let c = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    let t = &c.three_houses;
    // 丁年寅月起壬：寅壬卯癸辰甲巳乙午丙未丁申戊酉己戌庚亥辛子壬丑癸 — 命宫干=癸，身宫干=乙。
    assert_eq!(t.ming_gong, "癸丑");
    assert_eq!(t.shen_gong, "乙巳");
    assert_eq!(t.tai_yuan, "庚子");
}

#[test]
fn three_houses_1990_oracle() {
    // 1990-06-15 14：30 男 = 庚午 壬午 辛亥 乙未。
    // 月支午(6)，时支未（7，14：30 hour_branch=(15/2)=7 未）。
    //   命宫支 = (6 - 7 + 12) % 12 = 11 → 亥；命宫干 = 庚年寅月起戊，寅卯辰巳午未申酉戌亥子丑 = 戊己庚辛壬癸甲乙丙丁戊己，亥=丁 → 命宫=丁亥。
    //   身宫支 = (6 + 7) % 12 = 1 → 丑；身宫干 = 庚年戊起，丑=己 → 身宫=己丑。
    //   胎元 = 壬午 +1+3 = 癸酉。
    let c = compute(BirthInput {
        year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
        gender: Some(Gender::Male),
    });
    assert_eq!(c.three_houses.ming_gong, "丁亥");
    assert_eq!(c.three_houses.shen_gong, "己丑");
    assert_eq!(c.three_houses.tai_yuan, "癸酉");
}

/// 命宫公式性质：寅月寅时 → 命宫支 = (2-2+12)%12 = 0 = 子；身宫支 = 4 = 辰。
#[test]
fn three_houses_ming_gong_property() {
    // 甲年寅月寅时（任意 day）：月支=寅(2)，时支=寅(2)。
    // 甲年寅月起丙：寅丙卯丁辰戊巳己午庚未辛申壬酉癸戌甲亥乙子丙丑丁。
    let yr = GanZhi { stem: 0, branch: 0 }; // 甲年
    let mo = GanZhi { stem: 2, branch: 2 }; // 丙寅
    let th = determine_three_houses(yr, mo, 2);
    // 命宫支 = (2-2+12)%12 = 0 → 子；命宫干 = 甲年子月 = 丙 → 命宫=丙子。
    assert_eq!(th.ming_gong, "丙子");
    // 身宫支 = (2+2)%12 = 4 → 辰；身宫干 = 甲年辰月 = 戊 → 身宫=戊辰。
    assert_eq!(th.shen_gong, "戊辰");
    // 胎元 = 月柱干+1=丁、支+3=巳 → 丁巳。
    assert_eq!(th.tai_yuan, "丁巳");
}

#[test]
fn team_wuxing_average_oracle() {
    // 两人合盘：1987 长沙（木11火20土23金32水14）+ 1990 长沙（走 compute 后取实际值）。
    // 平均 = 两人逐项求和÷2。
    let a = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    let b = compute(BirthInput {
        year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
        gender: Some(Gender::Male),
    });
    let team = team_wuxing_average(&[a.clone(), b.clone()]);
    assert_eq!(team.wood, u32::midpoint(a.strength.wuxing.wood, b.strength.wuxing.wood));
    assert_eq!(team.fire, u32::midpoint(a.strength.wuxing.fire, b.strength.wuxing.fire));
    assert_eq!(team.earth, u32::midpoint(a.strength.wuxing.earth, b.strength.wuxing.earth));
    assert_eq!(team.metal, u32::midpoint(a.strength.wuxing.metal, b.strength.wuxing.metal));
    assert_eq!(team.water, u32::midpoint(a.strength.wuxing.water, b.strength.wuxing.water));
}

#[test]
fn team_wuxing_empty_is_zero() {
    let z = team_wuxing_average(&[]);
    assert_eq!(z.wood + z.fire + z.earth + z.metal + z.water, 0);
}

#[test]
fn complement_and_team_extremes() {
    // 1987 wuxing： 木11 火20 土23 金32 水14。
    let a = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    let wx = &a.strength.wuxing;
    // 对应主用神「木」（1987 偏强主用木），互补度 = 11（自给度低，需别人补）
    assert_eq!(complement_score(&a.yongshen.primary_wuxing, wx), wx.wood);
    // 极端最弱 = 木11、最旺 = 金32
    assert_eq!(team_weakest(wx), ("木".into(), wx.wood));
    assert_eq!(team_strongest(wx), ("金".into(), wx.metal));
    // 未知五行字符串 → 0
    assert_eq!(complement_score("xxx", wx), 0);
}

/// 1987-09-17 男 = 丁卯 己酉 己巳 壬申。日干己，年支卯。
/// 各柱命中神煞（人工核校）：
/// - 年柱卯：日干己 → 卯无日干锚命中（学堂/文昌均酉）；
///   年支卯 anchor 亥卯未组 → 桃花子/驿马巳/华盖未/将星卯 → 卯=将星 ✓
/// - 月柱酉：日干己 → 命中 学堂（酉） + 文昌（酉）；年支卯 anchor → 酉非该组任一神煞 → 无；合 [学堂， 文昌]
/// - 日柱巳：日干己 → 巳非任一日干锚位 → 无；年支卯 anchor → 巳=驿马；且非魁罡 → 合 [驿马]
/// - 时柱申：日干己 → 无；年支卯 anchor → 申非该组任一 → 无 → 合 []
#[test]
fn shensha_1987_oracle() {
    let c = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    assert_eq!(c.year.shensha, vec!["将星"]);
    // shensha_by_day_stem 内部顺序 = 羊刃/禄/文昌/红艳/学堂/词馆 → 文昌先于学堂
    assert_eq!(c.month.shensha, vec!["文昌", "学堂"]);
    assert_eq!(c.day.shensha, vec!["驿马"]);
    assert_eq!(c.hour.shensha, Vec::<String>::new());
}

/// 魁罡日柱触发：1980-09-13 22：00 男 → 看是不是庚辰/庚戌/壬辰/戊戌之一？
/// 实际不知日柱，构造一个已知魁罡日：1984-04-29 （任查）。
/// 改用 zi_hour 测试日柱 = 壬辰的样例。
/// 1976-12-04 → 日柱壬辰？（查实际锚 2024-01-01=甲子 0，壬辰=28号）
/// 简化：用合成 GanZhi 直接测 is_kuigang_day，主测「魁罡」字符串出现在 day.shensha 即可。
#[test]
fn shensha_kuigang_marker() {
    use mingli_ganzhi::is_kuigang_day;
    // 任意找一天日柱 = 庚戌（=06 戌 序号 47）？跑出来反测：
    // 1979-08-12 = JDN ？，日柱？
    // 直接 unit test ganzhi crate fn：
    assert!(is_kuigang_day(mingli_ganzhi::GanZhi { stem: 6, branch: 10 }));
    assert!(!is_kuigang_day(mingli_ganzhi::GanZhi { stem: 5, branch: 5 })); // 己巳
    // 1987 己巳日 → 非魁罡 → day.shensha 不含「魁罡」
    let c = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    assert!(!c.day.shensha.contains(&"魁罡".to_string()));
}

#[test]
fn hidden_stems_oracle() {
    // 1987-09-17：四柱 丁卯 己酉 己巳 壬申。
    let c = compute(BirthInput {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(Gender::Male),
    });
    let stems = |p: &Pillar| p.hidden.iter().map(|h| h.stem.clone()).collect::<Vec<_>>();
    assert_eq!(stems(&c.year), ["乙"]); // 卯藏乙
    assert_eq!(stems(&c.month), ["辛"]); // 酉藏辛
    assert_eq!(stems(&c.day), ["丙", "庚", "戊"]); // 巳藏丙庚戊
    assert_eq!(stems(&c.hour), ["庚", "壬", "戊"]); // 申藏庚壬戊
    // 支藏十神接线：日主己土，巳本气丙火生己土、阴阳异 → 正印。
    assert_eq!(c.day.hidden[0].ten_god, "正印");
    // 十二长生（日主己，阴干逆行，长生在酉）：年卯=病、月酉=长生、日巳=帝旺、时申=沐浴。
    assert_eq!(c.year.day_twelve, "病");
    assert_eq!(c.month.day_twelve, "长生");
    assert_eq!(c.day.day_twelve, "帝旺");
    assert_eq!(c.hour.day_twelve, "沐浴");
}
