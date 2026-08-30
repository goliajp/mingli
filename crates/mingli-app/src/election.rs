//! 择吉用例：扫一段时窗，逐日出择日要素，按通行粗筛排序。
//!
//! **排序依据只有一条**——建除十二神的通行分档（口诀「建满平收黑，除危定执黄，
//! 成开皆可用，破闭不可当」）。这是多源一致、可查可核的粗筛。
//!
//! 事类宜忌（婚 / 葬 / 动土 / 行 / 开业各宜什么神）各家出入很大，**本层不合成任何总分、
//! 不下宜忌断语**：候选带着彭祖百忌与天乙贵人等结构事实一并输出，由释义层结合所问之事去说。

use mingli_astro::civil_day_number;
use mingli_contract::{AskTime, Moment};
use mingli_zeri::DayGrade;
use serde::Serialize;

/// 一个候选日。
#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    /// 公历年。
    pub year: i32,
    /// 公历月。
    pub month: u32,
    /// 公历日。
    pub day: u32,
    /// 日干支。
    pub day_ganzhi: String,
    /// 建除十二神名。
    pub jianchu: &'static str,
    /// 建除分档。
    pub grade: DayGrade,
    /// 分档中文标签。
    pub grade_label: &'static str,
    /// 二十八宿值日。
    pub mansion: &'static str,
    /// 彭祖百忌·干句。
    pub pengzu_gan: &'static str,
    /// 彭祖百忌·支句。
    pub pengzu_zhi: &'static str,
    /// 天乙贵人所临地支。
    pub tianyi: [&'static str; 2],
}

/// 一次择吉扫描的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Election {
    /// 时窗起（含）。
    pub window_start: AskTime,
    /// 时窗止（含）。
    pub window_end: AskTime,
    /// 事类（只入释义，不参与排序）。
    pub category: Option<String>,
    /// 扫描到的天数。
    pub scanned_days: u32,
    /// 候选日，已按分档排序（黄道在前），同档保持时间先后。
    pub candidates: Vec<Candidate>,
}

/// 一次扫描最多覆盖的天数——时窗再长也不至于把服务拖垮。
const MAX_DAYS: u32 = 366;

/// 扫描时窗并排序。
///
/// 逐日取正午为代表时刻（择日以「日」为单位，日内时辰另属择时）。
///
/// # 为什么是单叶
///
/// 「择」这一类目前只有择日一片叶答得起，故这个用例不走 `route`——它直接扫择日的逐日要素。
/// 这是**有意的**，不是还没接多叶：
///
/// - 本用例的产物是「候选日按吉凶分档排名」，而分档的依据是建除十二神。
///   要多叶，得先有一个「每叶各出一档」的形状，且各叶的档次可比——而各家分档的粒度与判据都不同，
///   合成一个总排名等于替读者选边，这正是本项目不做的事
/// - 小六壬曾在这一类的名单上，但它给的是某一时辰落在六神的哪一位，不是分档的候选日；
///   它的声明已按「算得出该意图的 output_shape」这条标准撤下（见该叶的 `answers`）
///
/// 要变成多叶，先改的是 [`Election`] 的形状，不是这里的循环。
///
/// # Errors
///
/// 时窗起点晚于终点、或跨度超过一年时返回说明。
pub fn scan(start: &AskTime, end: &AskTime, category: Option<String>) -> Result<Election, String> {
    let from = civil_day_number(start.year, start.month, start.day);
    let to = civil_day_number(end.year, end.month, end.day);
    if to < from {
        return Err("时窗终点早于起点".into());
    }
    let days = u32::try_from(to - from + 1).unwrap_or(u32::MAX);
    if days > MAX_DAYS {
        return Err(format!("时窗最长 {MAX_DAYS} 天，当前 {days} 天"));
    }

    let mut candidates = Vec::with_capacity(days as usize);
    let (mut y, mut mo, mut d) = (start.year, start.month, start.day);
    for _ in 0..days {
        // 逐日取正午为代表时刻：择日以「日」为单位，日内择时另论。
        let c = mingli_zeri::compute_at(&Moment::new(y, mo, d, 12, 0, start.tz));
        candidates.push(Candidate {
            year: y,
            month: mo,
            day: d,
            day_ganzhi: c.day_ganzhi_name,
            jianchu: c.jianchu,
            grade: c.grade,
            grade_label: c.grade_label,
            mansion: c.mansion,
            pengzu_gan: c.pengzu_gan,
            pengzu_zhi: c.pengzu_zhi,
            tianyi: c.tianyi_names,
        });
        (y, mo, d) = next_civil_day(y, mo, d);
    }
    // 稳定排序：同档者保持时间先后，读者一眼看出「最近的黄道日是哪天」
    candidates.sort_by_key(|c| c.grade.rank());

    Ok(Election {
        window_start: start.clone(),
        window_end: end.clone(),
        category,
        scanned_days: days,
        candidates,
    })
}

/// 走到下一个公历日。
///
/// 月长取 [`crate::days_in_month`]——本模块原先自带一份私有副本连同 `is_leap_year`，
/// 与同 crate 里那份逐字重复，却没有守卫：当时的测试只走 2026 与 2028，
/// 百年闰的两支（`% 100`、`% 400`）从没执行过；又只查每月末日的进位，
/// 删掉「三十一天那几个月」的 match 臂后落到通配的二月天数，末日进位照样成立。
fn next_civil_day(y: i32, m: u32, d: u32) -> (i32, u32, u32) {
    if d < crate::days_in_month(y, m) {
        (y, m, d + 1)
    } else if m < 12 {
        (y, m + 1, 1)
    } else {
        (y + 1, 1, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(year: i32, month: u32, day: u32) -> AskTime {
        AskTime { year, month, day, hour: 12, minute: 0, tz: 8.0 }
    }

    #[test]
    fn scans_every_day_in_the_window_inclusive() {
        let e = scan(&at(2026, 9, 1), &at(2026, 9, 10), None).expect("十天时窗应可扫");
        assert_eq!(e.scanned_days, 10);
        assert_eq!(e.candidates.len(), 10);
        // 日干支在连续十天里各不相同
        let mut gz: Vec<&str> = e.candidates.iter().map(|c| c.day_ganzhi.as_str()).collect();
        gz.sort_unstable();
        gz.dedup();
        assert_eq!(gz.len(), 10);
    }

    #[test]
    fn candidates_come_back_ranked_by_grade() {
        let e = scan(&at(2026, 9, 1), &at(2026, 10, 15), None).expect("应可扫");
        let ranks: Vec<u8> = e.candidates.iter().map(|c| c.grade.rank()).collect();
        assert!(ranks.windows(2).all(|w| w[0] <= w[1]), "应按分档升序");
        assert_eq!(e.candidates[0].grade, DayGrade::Huang, "首位应是黄道日");
        // 同档内保持时间先后（稳定排序）
        let huang: Vec<u32> = e
            .candidates
            .iter()
            .filter(|c| c.grade == DayGrade::Huang)
            .map(|c| c.month * 100 + c.day)
            .collect();
        assert!(huang.windows(2).all(|w| w[0] < w[1]), "同档应保持时间先后");
    }

    #[test]
    fn day_stepping_crosses_months_and_leap_years() {
        assert_eq!(next_civil_day(2026, 1, 31), (2026, 2, 1));
        assert_eq!(next_civil_day(2026, 12, 31), (2027, 1, 1));
        assert_eq!(next_civil_day(2026, 2, 28), (2026, 3, 1), "平年二月");
        assert_eq!(next_civil_day(2028, 2, 28), (2028, 2, 29), "闰年二月");
        assert_eq!(next_civil_day(2028, 2, 29), (2028, 3, 1));
        // 百年闰：1900 不闰、2000 闰、2100 不闰——原先一年都没走到这两支。
        assert_eq!(next_civil_day(1900, 2, 28), (1900, 3, 1), "1900 能被 100 整除、不能被 400 整除");
        assert_eq!(next_civil_day(2000, 2, 28), (2000, 2, 29), "2000 能被 400 整除");
        assert_eq!(next_civil_day(2100, 2, 28), (2100, 3, 1), "2100 能被 100 整除、不能被 400 整除");
        // 月中步进：原先只查每月末日的进位，而末日进位在月长被算短时照样成立。
        for (y, m, len) in [
            (2026i32, 1u32, 31u32), (2026, 4, 30), (2026, 2, 28), (2028, 2, 29),
            (2026, 7, 31), (2026, 9, 30), (2026, 12, 31),
        ] {
            for d in 1..len {
                assert_eq!(next_civil_day(y, m, d), (y, m, d + 1), "{y}-{m:02}-{d:02} 应只进一天");
            }
            let rolled = next_civil_day(y, m, len);
            assert_ne!(rolled, (y, m, len + 1), "{y}-{m:02} 该在第 {len} 天翻月");
        }
        // 走满一整年，天数必与该年实际日数相同。
        for y in [1900i32, 2000, 2024, 2026, 2100] {
            let (mut cy, mut cm, mut cd) = (y, 1u32, 1u32);
            let mut n = 0u32;
            while (cy, cm) != (y + 1, 1) {
                (cy, cm, cd) = next_civil_day(cy, cm, cd);
                n += 1;
            }
            let want = if crate::days_in_month(y, 2) == 29 { 366 } else { 365 };
            assert_eq!(n, want, "{y} 年共 {want} 天");
        }
        // 跨年扫描的天数与日历一致
        let e = scan(&at(2027, 12, 28), &at(2028, 1, 3), None).expect("跨年应可扫");
        assert_eq!(e.scanned_days, 7);
    }

    #[test]
    fn window_bounds_are_checked() {
        assert!(scan(&at(2026, 9, 10), &at(2026, 9, 1), None).is_err(), "终点早于起点");
        assert!(scan(&at(2026, 1, 1), &at(2027, 6, 1), None).is_err(), "跨度超过一年");
        // 单日时窗合法
        let one = scan(&at(2026, 9, 1), &at(2026, 9, 1), None).expect("单日应可扫");
        assert_eq!((one.scanned_days, one.candidates.len()), (1, 1));

        // 上限那一天两边各站一次：恰好 MAX_DAYS 天要收，多一天要拒。
        // 原先只用了「一年半」这种远超上限的窗，边界上 `>` 与 `>=` 分不出来。
        // 2028 是闰年，1-1 到 12-31 恰 366 天。
        let exact = scan(&at(2028, 1, 1), &at(2028, 12, 31), None)
            .expect("恰好 366 天应可扫——文档说的是「最长 366 天」");
        assert_eq!(exact.scanned_days, MAX_DAYS);
        assert_eq!(exact.candidates.len(), MAX_DAYS as usize);
        let one_too_many = scan(&at(2028, 1, 1), &at(2029, 1, 1), None);
        assert!(one_too_many.is_err(), "367 天应被拒");
    }

    #[test]
    fn category_rides_along_without_touching_the_ranking() {
        let plain = scan(&at(2026, 9, 1), &at(2026, 9, 20), None).expect("应可扫");
        let wed = scan(&at(2026, 9, 1), &at(2026, 9, 20), Some("婚".into())).expect("应可扫");
        assert_eq!(wed.category.as_deref(), Some("婚"));
        let a: Vec<&str> = plain.candidates.iter().map(|c| c.day_ganzhi.as_str()).collect();
        let b: Vec<&str> = wed.candidates.iter().map(|c| c.day_ganzhi.as_str()).collect();
        assert_eq!(a, b, "事类只入释义，不得改变排序");
    }
}
