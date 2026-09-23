//! 真太阳时校正：经度差 + 简化均时差，以及所需的民用日历算术。

use super::*;

/// 是否闰年（公历）。
pub(crate) const fn is_leap_year(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

pub(crate) const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

pub(crate) const fn days_in_month(y: i32, m: u32) -> u32 {
    if m == 2 && is_leap_year(y) { 29 } else { DAYS_IN_MONTH[(m - 1) as usize] }
}

/// 公历日期 +Δ 天（可正可负，跨月/跨年自动处理）。
pub(crate) fn add_days_civil(y: i32, m: u32, d: u32, delta: i32) -> (i32, u32, u32) {
    let mut y = y;
    let mut m = m;
    let mut d = d as i32 + delta;
    // 这个 `<` 松成 `<=` 是等价变异：`d == 1` 时它会向前借一个月
    // （`m-1`，`d += 那个月的天数`），紧接着下面那个循环发现 `d` 超了本月天数，
    // 又原样还回来，绕一圈回到同一天。
    while d < 1 {
        m = if m == 1 { y -= 1; 12 } else { m - 1 };
        d += days_in_month(y, m) as i32;
    }
    while d > days_in_month(y, m) as i32 {
        d -= days_in_month(y, m) as i32;
        m = if m == 12 { y += 1; 1 } else { m + 1 };
    }
    (y, m, d as u32)
}

/// 公历年内日序 1..366。
pub(crate) fn day_of_year(y: i32, m: u32, d: u32) -> u32 {
    const CUMUL: [u32; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let mut n = CUMUL[(m - 1) as usize] + d;
    if is_leap_year(y) && m > 2 {
        n += 1;
    }
    n
}

/// 均时差（分钟，Spencer/Iqbal 简化公式，精度 ±0.5 min，足够时辰判定）。
///
/// 真太阳时与平太阳时的差(EoT) = 9.87 sin(2B) − 7.53 cos(B) − 1.5 sin(B)，
/// 其中 B = 2π(N−81)/365、N = 年内日序(1..366)。
#[must_use]
pub fn equation_of_time_minutes(year: i32, month: u32, day: u32) -> f64 {
    let n = f64::from(day_of_year(year, month, day));
    let b = 2.0 * std::f64::consts::PI * (n - 81.0) / 365.0;
    9.87 * (2.0 * b).sin() - 7.53 * b.cos() - 1.5 * b.sin()
}

/// 真太阳时相对钟表时的总偏移（分钟，正=真太阳时较钟表晚）。
///
/// 由两部分组成：① 经度差（出生地经度 − 时区标准经线）× 4 分钟/度；② [`equation_of_time_minutes`] 均时差。
#[must_use]
pub fn true_solar_offset_minutes(
    longitude: f64,
    tz_hours: f64,
    year: i32,
    month: u32,
    day: u32,
) -> f64 {
    let std_longitude = tz_hours * 15.0;
    let geo_correction = (longitude - std_longitude) * 4.0;
    geo_correction + equation_of_time_minutes(year, month, day)
}

/// 真太阳时排盘：按出生地经度 + EoT 校正钟表时，再排八字。
///
/// 钟表时 → 真太阳时差（±约 30 分钟内典型）；跨时辰边界时，时柱与钟表版排盘不同。
#[must_use]
pub fn compute_with_true_solar(input: BirthInput, longitude: f64) -> BaziChart {
    let offset = true_solar_offset_minutes(
        longitude, input.tz, input.year, input.month, input.day,
    );
    let offset_min = offset.round() as i32;
    let total = input.hour as i32 * 60 + input.minute as i32 + offset_min;
    let (day_delta, in_day_min) = if total < 0 {
        (-1, total + 24 * 60)
    } else if total >= 24 * 60 {
        (1, total - 24 * 60)
    } else {
        (0, total)
    };
    let (ny, nm, nd) = if day_delta == 0 {
        (input.year, input.month, input.day)
    } else {
        add_days_civil(input.year, input.month, input.day, day_delta)
    };
    let nh = (in_day_min / 60) as u32;
    let nmin = (in_day_min % 60) as u32;
    let moment = Moment::new(ny, nm, nd, nh, nmin, input.tz);
    compute_at(&moment, input.gender)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn century_years_follow_the_gregorian_rule() {
        assert!(is_leap_year(2024) && is_leap_year(2000), "普通四年闰与四百年闰");
        assert!(!is_leap_year(1900) && !is_leap_year(2100), "百年不闰");
        assert!(!is_leap_year(2023));
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(1900, 2), 28);
        assert_eq!(day_of_year(2024, 3, 1), 61, "闰年三月一日是第 61 天");
        assert_eq!(day_of_year(2023, 3, 1), 60);
    }

    #[test]
    fn adding_days_walks_backwards_across_a_year_boundary() {
        assert_eq!(add_days_civil(2026, 1, 1, -1), (2025, 12, 31));
        assert_eq!(add_days_civil(2026, 3, 1, -1), (2026, 2, 28));
        assert_eq!(add_days_civil(2024, 3, 1, -1), (2024, 2, 29), "闰年二月");
        assert_eq!(add_days_civil(2025, 12, 31, 1), (2026, 1, 1));
        assert_eq!(add_days_civil(2026, 1, 1, 0), (2026, 1, 1));
    }

    /// 均时差此前一处也没验过——`equation_of_time_minutes` 从来没被任何测试调用。
    ///
    /// 它决定真太阳时排盘要挪多少分钟，跨时辰边界时直接换一根时柱；而它是个
    /// 四项算术式，任何一项写反都无声。变异测试在这个函数上留下四个活口，
    /// 底下这八个点各能把它们拉开 14 分钟以上。
    ///
    /// 取值两源相合，且两源给的是同一套约定（视太阳时 − 平太阳时，正 = 日晷快）：
    ///
    /// 1. 维基「Equation of time」引美国海军天文台：2 月 11 日 −14 分 15 秒、
    ///    5 月 14 日 +3 分 41 秒、7 月 26 日 −6 分 30 秒、11 月 3 日 +16 分 25 秒；
    ///    零点在 4 月 15、6 月 13、9 月 1、12 月 25 日附近
    ///    <https://en.wikipedia.org/wiki/Equation_of_time>
    /// 2. Universal Workshop 的日晷条目：11 月 2 日 +16.49 分、2 月 11 日 −14.24 分、
    ///    5 月 13 日 +3.65 分、7 月 25 日 −6.55 分，零点同上四处
    ///    <https://www.universalworkshop.com/the-equation-of-time/>
    ///
    /// 本式是 Spencer/Iqbal 简化式，模块自述 ±0.5 分。实测四个极值最大差 0.33 分，
    /// 故容差取 0.5 分。
    #[test]
    fn the_equation_of_time_matches_two_published_tables_at_its_extremes() {
        for (month, day, published) in
            [(2u32, 11u32, -14.25f64), (5, 14, 3.68), (7, 26, -6.50), (11, 3, 16.42)]
        {
            let ours = equation_of_time_minutes(2026, month, day);
            assert!(
                (ours - published).abs() < 0.5,
                "{month:02}-{day:02}：算出 {ours:+.2} 分，两源作 {published:+.2} 分"
            );
        }
        // 方向本身也钉住：二月中日晷慢、十一月初日晷快。四个变异体里有两个是把号写反的。
        assert!(equation_of_time_minutes(2026, 2, 11) < -10.0, "二月中应为负");
        assert!(equation_of_time_minutes(2026, 11, 3) > 10.0, "十一月初应为正");
    }

    /// 四个零点用变号窗口夹，而不是断言「≈ 0」。
    ///
    /// 实测（2026-08-25）：闰年把 12 月 25 日那个零点推到 0.96 分——本式只吃年内日序，
    /// 闰年二月之后整条曲线错一天，而零点附近斜率最大，所以「≈ 0」的写法在闰年会红。
    /// 夹住两侧的号则在闰年平年都成立，且它说的是同一件事的更强形式。
    #[test]
    fn the_equation_of_time_crosses_zero_four_times_a_year() {
        for year in [2024i32, 2025, 2026] {
            for ((m1, d1), (m2, d2)) in
                [((4u32, 8u32), (4u32, 22u32)), ((6, 6), (6, 20)), ((8, 25), (9, 8))]
            {
                let before = equation_of_time_minutes(year, m1, d1);
                let after = equation_of_time_minutes(year, m2, d2);
                assert!(
                    before * after < 0.0,
                    "{year} 年 {m1:02}-{d1:02}（{before:+.2}）到 {m2:02}-{d2:02}（{after:+.2}）之间应变号"
                );
            }
            // 跨年的那个零点
            let before = equation_of_time_minutes(year, 12, 18);
            let after = equation_of_time_minutes(year + 1, 1, 3);
            assert!(before > 0.0 && after < 0.0, "岁末那个零点应由正转负");
        }
    }

    /// 经度那一项是定义式：一度四分钟，与日期无关。
    #[test]
    fn one_degree_of_longitude_is_exactly_four_minutes() {
        for (year, month, day) in [(2026i32, 1u32, 1u32), (2026, 7, 1), (2024, 11, 3)] {
            let at = |lon: f64| true_solar_offset_minutes(lon, 8.0, year, month, day);
            assert!((at(121.0) - at(120.0) - 4.0).abs() < 1e-9, "一度应差四分钟");
            assert!((at(120.0) - at(90.0) - 120.0).abs() < 1e-9, "三十度应差两小时");
            // 站在时区标准经线上，偏移只剩均时差。
            assert!(
                (at(120.0) - equation_of_time_minutes(year, month, day)).abs() < 1e-9,
                "120°E 是东八区的标准经线，此处偏移应恰为均时差"
            );
        }
    }

    /// 真太阳时校正把时刻推过午夜时，日期必须跟着走——否则日柱会错一整柱。
    #[test]
    fn true_solar_correction_can_carry_the_date_across_midnight() {
        let base = |year, month, day, hour, minute| BirthInput {
            year,
            month,
            day,
            hour,
            minute,
            tz: 8.0,
            gender: None,
        };
        // 东八区 +8 的标准经线是 120°E。取 175°E：经度差 +55° × 4 分钟 = +220 分钟，
        // 23:00 加上去越过午夜，日期须进一天。
        let late = compute_with_true_solar(base(2026, 6, 15, 23, 0), 175.0);
        let next = crate::compute(base(2026, 6, 16, 2, 40));
        assert_eq!(
            late.day.ganzhi, next.day.ganzhi,
            "越过午夜后应与次日同一日柱"
        );
        // 取 60°E：经度差 −60° × 4 分钟 = −240 分钟，00:30 减下去退回前一天。
        let early = compute_with_true_solar(base(2026, 6, 15, 0, 30), 60.0);
        let prev = crate::compute(base(2026, 6, 14, 20, 30));
        assert_eq!(
            early.day.ganzhi, prev.day.ganzhi,
            "退回午夜前应与前一日同一日柱"
        );
        // 同一时区标准经线上只剩均时差，日期不动。
        let same = compute_with_true_solar(base(2026, 6, 15, 12, 0), 120.0);
        assert_eq!(same.day.ganzhi, crate::compute(base(2026, 6, 15, 12, 0)).day.ganzhi);
    }
}
