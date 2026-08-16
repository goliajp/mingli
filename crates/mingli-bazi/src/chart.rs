//! 排盘主链：由共享时刻定四柱，再挂上藏干、十神、神煞、大运与各层结论。

use super::*;

pub(crate) fn pillar(gz: GanZhi, day_master: u8, is_day: bool, year_branch: u8, day_gz: GanZhi) -> Pillar {
    // 神煞落到该柱（日干锚 + 年支锚 + 日柱魁罡）
    let mut shensha: Vec<String> = Vec::new();
    for &name in &shensha_by_day_stem(day_master, gz.branch) {
        shensha.push(name.to_string());
    }
    for &name in &shensha_by_branch_anchor(year_branch, gz.branch) {
        // 避免日干锚和年支锚同支重复
        let s = name.to_string();
        if !shensha.contains(&s) {
            shensha.push(s);
        }
    }
    if is_day && is_kuigang_day(day_gz) {
        shensha.push("魁罡".to_string());
    }
    Pillar {
        ganzhi: gz.to_string(),
        stem: STEMS[gz.stem as usize].to_string(),
        branch: BRANCHES[gz.branch as usize].to_string(),
        stem_wuxing: stem_element(gz.stem).name().to_string(),
        branch_wuxing: branch_element(gz.branch).name().to_string(),
        nayin: nayin_element(gz).name().to_string(),
        ten_god: if is_day {
            "日主".to_string()
        } else {
            ten_god(day_master, gz.stem).to_string()
        },
        hidden: hidden_stems(gz.branch)
            .iter()
            .map(|&hs| HiddenStem {
                stem: STEMS[hs as usize].to_string(),
                ten_god: ten_god(day_master, hs).to_string(),
            })
            .collect(),
        day_twelve: TWELVE_STAGES[twelve_stage(day_master, gz.branch) as usize].to_string(),
        shensha,
    }
}

/// 排八字（独立入口：自行构造共享上下文 [`Moment`]）。
#[must_use]
pub fn compute(input: BirthInput) -> BaziChart {
    let m = Moment::new(
        input.year,
        input.month,
        input.day,
        input.hour,
        input.minute,
        input.tz,
    );
    compute_at(&m, input.gender)
}

/// 子时归属流派（影响 23：00–23：59 出生的日柱）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ZiHourMethod {
    /// **晚子（Late，主流）**：子时整体属次日，23-24 点 → 次日日柱。
    Late,
    /// **早子（Early，传统少数派）**：23-24 点仍属当日，称为「夜子」；0-1 点称「正子」次日。
    Early,
}

/// 年柱换岁流派（影响立春前/正月初一前出生的年柱）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum YearBreakMethod {
    /// **立春换年（主流）**：节气立春（太阳黄经 315°）为新年界。子平命理主流。
    LiChun,
    /// **春节换年（民间少数派）**：农历正月初一为新年界。民俗/择吉派偶用。
    SpringFestival,
}

/// 八字流派全集。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BaziSchool {
    /// 子时归属流派。
    pub zi_hour: ZiHourMethod,
    /// 年柱换岁流派。
    pub year_break: YearBreakMethod,
}

impl Default for BaziSchool {
    fn default() -> Self {
        Self { zi_hour: ZiHourMethod::Late, year_break: YearBreakMethod::LiChun }
    }
}

/// 在已算好的共享上下文上排八字（指定子时流派，年柱仍用主流立春）。向后兼容。
#[must_use]
pub fn compute_at_with(m: &Moment, gender: Option<Gender>, zi: ZiHourMethod) -> BaziChart {
    compute_at_school(m, gender, BaziSchool { zi_hour: zi, year_break: YearBreakMethod::LiChun })
}

/// 在已算好的共享上下文上排八字（完整流派指定）。
#[must_use]
pub fn compute_at_school(m: &Moment, gender: Option<Gender>, school: BaziSchool) -> BaziChart {
    compute_at_impl(m, gender, school)
}

/// 在已算好的共享上下文 [`Moment`] 上排八字——供 DAG 引擎复用同一 `Moment`、零重算天文（默认 [`BaziSchool::default`]）。
#[must_use]
pub fn compute_at(m: &Moment, gender: Option<Gender>) -> BaziChart {
    compute_at_impl(m, gender, BaziSchool::default())
}

pub(crate) fn compute_at_impl(m: &Moment, gender: Option<Gender>, school: BaziSchool) -> BaziChart {
    let zi = school.zi_hour;
    let (jd, lam) = (m.jd_ut, m.sun_longitude);

    // 年柱：换岁流派——主流立春（节气黄经 315°）；少数派春节（农历正月初一）。
    let solar_year = match school.year_break {
        YearBreakMethod::LiChun => {
            let lichun = solar_term_jd(m.year, 315.0);
            if jd < lichun { m.year - 1 } else { m.year }
        }
        YearBreakMethod::SpringFestival => {
            // 农历正月初一（非闰）之前归前一年；之后（含）归本年。
            // m.lunar.month=1 且 m.lunar.day>=1 且 leap=false → 已到正月，本公历年成立。
            // 若 m.lunar.month=12 或 （month=1 day>=1 但 leap=true 跨闰），则尚未到正月初一，归前一年。
            let l = &m.lunar;
            if l.month == 1 && !l.leap && l.day >= 1 {
                m.year
            } else if l.month >= 11 || (l.month == 12) || (l.month == 1 && l.leap) {
                // 公历 1 月 1 日到农历正月初一之间（必落在公历 1-2 月）
                m.year - 1
            } else {
                m.year
            }
        }
    };
    let year_gz = year_ganzhi(solar_year);

    // 月柱：以「节」换月。s=0 → 寅月（立春起）。
    let s = ((lam - 315.0).rem_euclid(360.0) / 30.0).floor() as u8;
    let month_branch = (2 + s) % 12;
    let month_gz = GanZhi {
        stem: month_pillar_stem(year_gz.stem, month_branch),
        branch: month_branch,
    };

    // 日柱：共享上下文的民用日序 → 干支锚点递推
    // 子时流派：晚子（默认）= 23-24 点出生归次日；早子（传统少数派）= 仍归当日。
    let day_jdn = match zi {
        ZiHourMethod::Late if m.hour == 23 => m.civil_day + 1,
        _ => m.civil_day,
    };
    let day_gz = day_ganzhi(day_jdn);

    // 时柱：五鼠遁
    let hb = hour_branch(m.hour, m.minute);
    let hour_gz = GanZhi {
        stem: ((day_gz.stem % 5) * 2 + hb) % 10,
        branch: hb,
    };

    let dm = day_gz.stem;
    let lunar = m.lunar;
    let dayun = gender.map(|g| compute_dayun(jd, lam, year_gz.stem, g, month_gz));
    let strength = compute_strength(year_gz, month_gz, day_gz, hour_gz);
    let pattern = determine_pattern(year_gz, month_gz, day_gz, hour_gz);
    let yongshen = determine_yongshen(day_gz.stem, month_gz.branch, &strength);
    let three_houses = determine_three_houses(year_gz, month_gz, hb);

    BaziChart {
        input: BirthInput {
            year: m.year,
            month: m.month,
            day: m.day,
            hour: m.hour,
            minute: m.minute,
            tz: m.tz,
            gender,
        },
        lunar: LunarChart {
            year: lunar.year,
            month: lunar.month,
            leap: lunar.leap,
            day: lunar.day,
        },
        year: pillar(year_gz, dm, false, year_gz.branch, day_gz),
        month: pillar(month_gz, dm, false, year_gz.branch, day_gz),
        day: pillar(day_gz, dm, true, year_gz.branch, day_gz),
        hour: pillar(hour_gz, dm, false, year_gz.branch, day_gz),
        day_master: STEMS[dm as usize].to_string(),
        day_master_wuxing: stem_element(dm).name().to_string(),
        xunkong: {
            // 旬空：日柱所在 6 旬中，10 干配 12 支余下的 2 支。沿用 ganzhi 主干层 helper。
            let kong = mingli_ganzhi::xunkong(day_gz);
            [BRANCHES[kong[0] as usize].to_string(), BRANCHES[kong[1] as usize].to_string()]
        },
        strength,
        pattern,
        yongshen,
        three_houses,
        dayun,
    }
}

/// 大运：阳男阴女顺行、阴男阳女逆行；起运 = 到前/后一「节」的天数 ÷ 3 年。
pub(crate) fn compute_dayun(jd: f64, lam: f64, year_stem: u8, gender: Gender, month_gz: GanZhi) -> DaYun {
    let year_yang = year_stem.is_multiple_of(2); // 甲丙戊庚壬 为阳年
    let forward = match gender {
        Gender::Male => year_yang,
        Gender::Female => !year_yang,
    };

    // 「节」黄经 ≡ 15 (mod 30)。求紧邻的前/后一个节。
    let k = ((lam - 15.0) / 30.0).floor();
    let next_target = 15.0 + 30.0 * (k + 1.0);
    let prev_target = 15.0 + 30.0 * k;
    let next_jd =
        solar_term_time_near(jd + (next_target - lam).rem_euclid(360.0) / 0.98565, next_target);
    let prev_jd =
        solar_term_time_near(jd - (lam - prev_target).rem_euclid(360.0) / 0.98565, prev_target);

    let days = if forward { next_jd - jd } else { jd - prev_jd };
    let start_age_years = (days / 3.0).max(0.0);
    let start_age0 = start_age_years.round() as u32;

    let mut pillars = Vec::with_capacity(10);
    let m_idx = i32::from(month_gz.index());
    for i in 1..=10i32 {
        let idx = (if forward { m_idx + i } else { m_idx - i }).rem_euclid(60) as u8;
        pillars.push(LuckPillar {
            start_age: start_age0 + (i as u32 - 1) * 10,
            ganzhi: GanZhi::from_index(idx).to_string(),
        });
    }

    DaYun {
        forward,
        start_age_years: (start_age_years * 100.0).round() / 100.0,
        pillars,
    }
}
