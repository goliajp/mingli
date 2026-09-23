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

/// Annual Tibetan cycle attributes, without a date conversion or personal chart.
///
/// `year` labels the cycle year; it does not assert that a particular civil date
/// falls after Losar. There are intentionally no day-parkha fields: the leaf's
/// year-only calculation fills those with a placeholder, not a computed day.
#[derive(Debug, Serialize)]
pub struct TibetanYear {
    /// Public response schema version.
    pub schema_version: u32,
    /// Distinguishes this record from a natal chart or complete calendar.
    pub kind: &'static str,
    /// Cycle-year label, not an inferred birth year.
    pub year: i32,
    /// Interpretation of the requested year.
    pub year_basis: &'static str,
    /// Annual animal name from the leaf.
    pub animal: &'static str,
    /// Annual power element (dbang thang), not all personal elements.
    pub element: &'static str,
    /// Traditional annual polarity, not the user's gender.
    pub male: bool,
    /// Position in the 60-year cycle beginning with Wood-Male-Rat.
    pub sexagenary: i64,
    /// Rabjung number, with the first beginning in 1027.
    pub rabjung: i64,
    /// Position within that rabjung (a different origin from `sexagenary`).
    pub year_in_rabjung: i64,
    /// Annual nine-number cycle, moving backward each year.
    pub mewa: i64,
    /// Traditional colour associated with the annual number.
    pub mewa_color: &'static str,
    /// No Losar boundary or civil-to-Tibetan date conversion was performed.
    pub calendar_conversion: &'static str,
    /// Frozen method/projection version for consumers storing snapshots.
    pub method_version: &'static str,
    /// Source identifiers for the annual formulas.
    pub source_ids: [&'static str; 1],
}

/// Query annual cycle attributes without inventing a birthday or a day trigram.
///
/// The 1900–2099 window is the reviewed delivery range, not a mathematical limit
/// of the underlying cycles. Formula reference: Svante Janson, *Tibetan Calendar
/// Mathematics*, Appendix E.1; see the leaf for cycle origins and source notes.
///
/// # Errors
/// Rejects years outside the reviewed delivery range; never substitutes a year.
pub fn tibetan_year(year: i32) -> Result<TibetanYear, &'static str> {
    if !(1900..=2099).contains(&year) {
        return Err("tibetan annual cycle year must be between 1900 and 2099");
    }
    let c = mingli_tibetan::compute_year(i64::from(year));
    Ok(TibetanYear {
        schema_version: 1,
        kind: "tibetan_annual_cycle",
        year,
        year_basis: "cycle_year_label",
        animal: c.animal,
        element: c.element,
        male: c.male,
        sexagenary: c.sexagenary,
        rabjung: c.rabjung,
        year_in_rabjung: c.year_in_rabjung,
        mewa: c.mewa,
        mewa_color: c.mewa_color,
        calendar_conversion: "not_computed",
        method_version: "tibetan-annual-v1",
        source_ids: ["janson-tibetan-calendar-E1"],
    })
}
