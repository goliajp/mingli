//! 中国农历年历。月名与置闰只取自现有公历转农历，不另立历法模型。
use crate::{LunarDate, civil_day_number, solar_to_lunar};

/// 公历民用日期，不带时刻。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CivilDate {
    /// 公历年。
    pub year: i32,
    /// 公历月。
    pub month: u32,
    /// 公历日。
    pub day: u32,
}
impl CivilDate {
    fn month_days(self) -> u32 {
        match self.month {
            4 | 6 | 9 | 11 => 30,
            2 => {
                28 + u32::from(self.year % 4 == 0 && (self.year % 100 != 0 || self.year % 400 == 0))
            }
            _ => 31,
        }
    }
    // 仅作最多三十二天的公历日期加减，不逐日调用农历或天文算法。
    fn shifted(mut self, days: i32) -> Self {
        for _ in 0..days.unsigned_abs() {
            if days > 0 {
                self.day += 1;
                if self.day > self.month_days() {
                    self.day = 1;
                    self.month += 1;
                    if self.month == 13 {
                        self.month = 1;
                        self.year += 1;
                    }
                }
            } else if self.day > 1 {
                self.day -= 1;
            } else {
                if self.month == 1 {
                    self.month = 12;
                    self.year -= 1;
                } else {
                    self.month -= 1;
                }
                self.day = self.month_days();
            }
        }
        self
    }
    fn number(self) -> i64 {
        civil_day_number(self.year, self.month, self.day)
    }
    fn lunar(self) -> LunarDate {
        solar_to_lunar(self.year, self.month, self.day, 8.0)
    }
}
/// 农历月，闰月保留本月月序与独立标记。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChineseMonth {
    /// 月序，一至十二。
    pub month: u32,
    /// 是否闰月。
    pub leap: bool,
    /// 初一对应的公历日期。
    pub starts_on: CivilDate,
    /// 本月天数，只能为二十九或三十。
    pub days: u32,
}
/// 从正月初一到次年正月初一的完整中国农历年。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChineseYear {
    /// 农历年。
    pub year: i32,
    /// 顺序排列的十二或十三个月。
    pub months: Vec<ChineseMonth>,
    /// 次年正月初一，不属于本年。
    pub ends_before: CivilDate,
}
/// 年历不可计算的原因。结构校验失败不能伪装为没有闰月。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChineseYearError {
    /// 完整农历年仅接受一九〇〇至二〇九九年。
    UnsupportedYear,
    /// 原有换算结果未通过连续性校验。
    InconsistentSequence,
}
impl std::fmt::Display for ChineseYearError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::UnsupportedYear => "完整农历年仅支持1900至2099年",
            Self::InconsistentSequence => "农历月份边界未通过校验",
        })
    }
}
impl std::error::Error for ChineseYearError {}

/// 完整中国农历年历，固定以东八区划分民用日。
///
/// 每次从月初向后三十二日探测，再减去探测日的农历日序，求下一月初。
/// 月长、月名顺序、闰月次数和次年正月边界均显式验证。最多十六次推进，
/// 覆盖公历年初之前的残月以及本农历年，不逐日排盘。
///
/// # Errors
/// 年份越界或换算结果不连续时返回错误。
///
/// # Panics
/// 继承 [`solar_to_lunar`] 的天文朔搜索收敛要求。
pub fn chinese_year(year: i32) -> Result<ChineseYear, ChineseYearError> {
    use ChineseYearError::{InconsistentSequence as invalid, UnsupportedYear};
    if !(1900..=2099).contains(&year) {
        return Err(UnsupportedYear);
    }
    let january = CivilDate {
        year,
        month: 1,
        day: 1,
    };
    let mut current = january.lunar();
    if !(1..=30).contains(&current.day) || !(1..=12).contains(&current.month) {
        return Err(invalid);
    }
    let mut start = january.shifted(1 - i32::try_from(current.day).map_err(|_| invalid)?);
    current.day = 1;
    let mut months: Vec<ChineseMonth> = Vec::with_capacity(13);
    for _ in 0..16 {
        let probe = start.shifted(32);
        let after = probe.lunar();
        if !(1..=30).contains(&after.day) {
            return Err(invalid);
        }
        let next_start = probe.shifted(1 - i32::try_from(after.day).map_err(|_| invalid)?);
        let next = next_start.lunar();
        let days = next_start.number() - start.number();
        if !(29..=30).contains(&days)
            || next.day != 1
            || (next.year, next.month, next.leap) != (after.year, after.month, after.leap)
        {
            return Err(invalid);
        }
        let last = next_start.shifted(-1).lunar();
        if last.year != current.year
            || last.month != current.month
            || last.leap != current.leap
            || i64::from(last.day) != days
        {
            return Err(invalid);
        }
        if current.year == year {
            if months.is_empty() && (current.month != 1 || current.leap) {
                return Err(invalid);
            }
            if let Some(previous) = months.last() {
                let expected = if current.leap {
                    previous.month
                } else {
                    previous.month + 1
                };
                if current.month != expected || (current.leap && previous.leap) {
                    return Err(invalid);
                }
            }
            months.push(ChineseMonth {
                month: current.month,
                leap: current.leap,
                starts_on: start,
                days: u32::try_from(days).map_err(|_| invalid)?,
            });
            if next.year == year + 1 && next.month == 1 && !next.leap {
                if !matches!(months.len(), 12 | 13)
                    || months.last().is_none_or(|m| m.month != 12)
                    || months.iter().filter(|m| m.leap).count() != months.len() - 12
                {
                    return Err(invalid);
                }
                if !(353..=385).contains(&(next_start.number() - months[0].starts_on.number())) {
                    return Err(invalid);
                }
                return Ok(ChineseYear {
                    year,
                    months,
                    ends_before: next_start,
                });
            }
        } else if !months.is_empty() || current.year != year - 1 {
            return Err(invalid);
        }
        start = next_start;
        current = next;
    }
    Err(invalid)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn leap_second_month_and_new_year_have_known_anchors() {
        // 外部日期锚沿用 lib.rs 的 HKO 春节与闰月算例；不是另一份置闰算法。
        let y = chinese_year(2023).unwrap();
        let m = y.months.iter().find(|m| m.leap).unwrap();
        assert_eq!(
            (m.month, m.starts_on, m.days),
            (
                2,
                CivilDate {
                    year: 2023,
                    month: 3,
                    day: 22
                },
                29
            )
        );
        assert_eq!(
            y.ends_before,
            CivilDate {
                year: 2024,
                month: 2,
                day: 10
            }
        );
    }
    #[test]
    fn recent_month_boundaries_match_hko_and_cwa() {
        // 两个独立发布机构的历表，预期日期不从待测算法生成。
        // HKO：https://www.hko.gov.hk/tc/gts/time/calendar/text/files/T2023c.txt
        // HKO：https://www.hko.gov.hk/tc/gts/time/calendar/text/files/T2025c.txt
        // HKO：https://www.hko.gov.hk/tc/gts/time/calendar/text/files/T2026c.txt
        // CWA：https://www.cwa.gov.tw/Data/service/notice/download/Publish_20221202095901.pdf
        // CWA：https://www.cwa.gov.tw/Data/service/notice/download/Publish_20241209150048.pdf
        // 2023 年历与 2025 年历的物理第二页，后者第三页为 2026 年。
        for (year, month, leap, solar_month, solar_day, days) in [
            (2023, 2, false, 2, 20, 30),
            (2023, 2, true, 3, 22, 29),
            (2025, 6, false, 6, 25, 30),
            (2025, 6, true, 7, 25, 29),
        ] {
            let y = chinese_year(year).unwrap();
            let m = y
                .months
                .iter()
                .find(|m| m.month == month && m.leap == leap)
                .unwrap();
            assert_eq!(
                m.starts_on,
                CivilDate {
                    year,
                    month: solar_month,
                    day: solar_day
                }
            );
            assert_eq!(m.days, days);
        }
        let y = chinese_year(2025).unwrap();
        assert_eq!(y.months.last().unwrap().days, 29);
        assert_eq!(
            y.ends_before,
            CivilDate {
                year: 2026,
                month: 2,
                day: 17
            }
        );
        assert_eq!(
            chinese_year(2026).unwrap().months[0].starts_on,
            y.ends_before
        );
    }

    #[test]
    fn winter_intercalation_matches_hko_and_taipei_independent_ephemeris() {
        // 两源分别为香港天文台历表和台北市立天文馆《台北星空》九十四期。
        // HKO：https://www.hko.gov.hk/tc/gts/time/calendar/text/files/T2033c.txt
        // HKO：https://www.hko.gov.hk/tc/gts/time/calendar/text/files/T2034c.txt
        // 台北：https://www-ws.gov.taipei/001/Upload/439/ebook/ebook_1820531/pdf/full.pdf
        // 物理第十九页表一用 MICA 2.2.2 地心计算与东八区，第二十页表二逐日核验。
        // 表一十二月范围栏把一月二十误印为十九；朔日栏和表二均确认二十日，
        // 表二十九日为廿九、二十日为大寒、二十一日为初二。此处不沿用范围栏笔误。
        for (year, month, leap, sy, sm, sd, days) in [
            (2033, 8, false, 2033, 8, 25, 29),
            (2033, 9, false, 2033, 9, 23, 30),
            (2033, 10, false, 2033, 10, 23, 30),
            (2033, 11, false, 2033, 11, 22, 30),
            (2033, 11, true, 2033, 12, 22, 29),
            (2033, 12, false, 2034, 1, 20, 30),
            (2034, 1, false, 2034, 2, 19, 29),
        ] {
            let y = chinese_year(year).unwrap();
            let m = y
                .months
                .iter()
                .find(|m| m.month == month && m.leap == leap)
                .unwrap();
            assert_eq!(
                m.starts_on,
                CivilDate {
                    year: sy,
                    month: sm,
                    day: sd
                }
            );
            assert_eq!(m.days, days);
        }
    }

    #[test]
    fn complete_supported_years_are_structurally_consistent() {
        // 结构遍历不能代替全区间的外部历表逐日校验。
        for year in 1900..=2099 {
            let y = chinese_year(year).unwrap_or_else(|e| panic!("{year}：{e}"));
            assert_eq!(
                y.months
                    .iter()
                    .filter(|m| !m.leap)
                    .map(|m| m.month)
                    .collect::<Vec<_>>(),
                (1..=12).collect::<Vec<_>>()
            );
            assert_eq!(y.months[0].starts_on.year, year);
            assert_eq!(y.ends_before.year, year + 1);
        }
        assert_eq!(chinese_year(1899), Err(ChineseYearError::UnsupportedYear));
        assert_eq!(chinese_year(2100), Err(ChineseYearError::UnsupportedYear));
    }
    #[test]
    fn civil_shifts_cover_century_leap_rules_and_cross_year() {
        assert_eq!(
            CivilDate {
                year: 1900,
                month: 2,
                day: 28
            }
            .shifted(1),
            CivilDate {
                year: 1900,
                month: 3,
                day: 1
            }
        );
        assert_eq!(
            CivilDate {
                year: 2000,
                month: 2,
                day: 28
            }
            .shifted(1),
            CivilDate {
                year: 2000,
                month: 2,
                day: 29
            }
        );
        assert_eq!(
            CivilDate {
                year: 2100,
                month: 1,
                day: 1
            }
            .shifted(-1),
            CivilDate {
                year: 2099,
                month: 12,
                day: 31
            }
        );
    }
}
