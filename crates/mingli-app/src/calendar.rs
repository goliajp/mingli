//! 中国农历年历用例。年份有界，历法日界固定，不借用本命排盘。
pub use mingli_astro::ChineseYearError as CalendarError;
use serde::Serialize;

/// 公开月表的一月。
#[derive(Debug, Serialize)]
pub struct ChineseMonth {
    /// 月序，一至十二。
    pub month: u32,
    /// 是否闰月。
    pub leap: bool,
    /// 初一的公历日期，年月日字符串。
    pub starts_on: String,
    /// 二十九或三十天。
    pub days: u32,
}
/// 完整中国农历年的公开投影。
#[derive(Debug, Serialize)]
pub struct ChineseYear {
    /// 当前契约版本。
    pub schema_version: u32,
    /// 历法标识。
    pub calendar: &'static str,
    /// 农历日界使用东八区，与提醒时区无关。
    pub time_zone_offset_minutes: i32,
    /// 农历年。
    pub year: i32,
    /// 顺序排列的月份。
    pub months: Vec<ChineseMonth>,
    /// 次年正月初一，不包含在本年。
    pub ends_before: String,
}
fn date(d: mingli_astro::CivilDate) -> String {
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}
/// 构造无需生日、时辰、性别或经纬度的公共年历。
///
/// # Errors
/// 完整年份越界、天文搜索异常或月份结构不一致时返回错误，不回退其他算法。
pub fn chinese_year(year: i32) -> Result<ChineseYear, CalendarError> {
    let y = std::panic::catch_unwind(|| mingli_astro::chinese_year(year))
        .map_err(|_| CalendarError::InconsistentSequence)??;
    Ok(ChineseYear {
        schema_version: 1,
        calendar: "chinese_lunisolar",
        time_zone_offset_minutes: 480,
        year: y.year,
        months: y
            .months
            .into_iter()
            .map(|m| ChineseMonth {
                month: m.month,
                leap: m.leap,
                starts_on: date(m.starts_on),
                days: m.days,
            })
            .collect(),
        ends_before: date(y.ends_before),
    })
}
