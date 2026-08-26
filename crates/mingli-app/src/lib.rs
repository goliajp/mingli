//! 用例层：一次完整请求要做的事。
//!
//! 承接层（HTTP / wasm）只负责协议转换与错误映射，凡是「先算本命、再拼岁运、
//! 再组一份对外结构」这类编排都住在这里，于是同一份用例可以同时被 axum 服务与
//! wasm 绑定复用，且能脱离 HTTP 单独测。
//!
//! 依赖方向：用例可以认识具体的叶（[`mingli_bazi`] 等实体），但**注册表由调用方注入**
//! ——装配根是更外层的事，用例不去 `mingli-registry` 里拿。

pub mod analysis;
pub mod bazi;
pub mod election;
pub mod event;
pub mod input;
pub mod interpret;
pub mod locative;
pub mod mundane;
pub mod synastry;
pub mod team;
pub mod word;
pub mod ziwei;

use mingli_contract::Gender;

/// 一次出生/占问输入——用例层的公共入参。
///
/// 也是它的**线上形状**：`tz` 缺省 +8、`minute` 缺省 0、性别缺省不排大运。
/// 直接可反序列化是有意的——见 [`input`] 里为什么入参形状归用例层。
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
pub struct Birth {
    /// 公历年（1900–2100）。
    pub year: i32,
    /// 公历月 1..12。
    pub month: u32,
    /// 公历日 1..31。
    pub day: u32,
    /// 时 0..23。
    pub hour: u32,
    /// 分 0..59。
    #[serde(default)]
    pub minute: u32,
    /// 时区偏移小时。缺省 +8（中国）；日本传 9。
    #[serde(default = "input::default_tz")]
    pub tz: f64,
    /// 性别（缺省则不排大运）。
    #[serde(default)]
    pub gender: Option<Gender>,
    /// 是否按真太阳时校正时柱。
    #[serde(default)]
    pub true_solar_time: bool,
    /// 出生地经度（真太阳时校正需要）。
    #[serde(default)]
    pub longitude: Option<f64>,
}

impl Birth {
    /// 输入域校验。
    ///
    /// # Errors
    ///
    /// 年份、月、日、时、分、时区、经度中任一越界时返回面向调用方的中文说明。
    pub fn validate(&self) -> Result<(), String> {
        validate_instant(self.year, self.month, self.day, self.hour, self.minute, self.tz)?;
        validate_coords(None, self.longitude)
    }
}

/// 一个时刻的取值域。两条交付路共用——HTTP 走 [`Birth::validate`]，wasm 走
/// [`validate_query`]，两边落到的是同一段判断，不各写一份。
///
/// # Errors
///
/// 年、月、日、时、分、时区中任一越界时返回面向调用方的中文说明。
pub fn validate_instant(year: i32, month: u32, day: u32, hour: u32, minute: u32, tz: f64) -> Result<(), String> {
    if !(1900..=2100).contains(&year) {
        return Err("year 仅支持 1900–2100".into());
    }
    if !(1..=12).contains(&month) {
        return Err("month 须 1–12".into());
    }
    // 日要按当月实际长度收，不能一律放到 31：2 月 31 日会被历法换算悄悄挪成 3 月 3 日，
    // 于是打错一个数字的人拿到的是**另一天**的盘，而界面上没有任何迹象
    let last = days_in_month(year, month);
    if !(1..=last).contains(&day) {
        return Err(format!("{year} 年 {month} 月只有 {last} 天"));
    }
    if hour > 23 || minute > 59 {
        return Err("hour/minute 越界".into());
    }
    // 现实中的 UTC 偏移落在 −12 到 +14 之间（+14 是 Kiritimati）
    if !(-12.0..=14.0).contains(&tz) {
        return Err("tz 须在 −12 到 +14 之间".into());
    }
    Ok(())
}

/// 排盘入参的取值域。wasm 那扇门吃的是 [`mingli_contract::Query`] 而非 [`Birth`]，
/// 两者字段不同、该收的东西一样，故各有一个入口、共用同一段判断。
///
/// # Errors
///
/// 时刻或坐标越界时返回面向调用方的中文说明。
pub fn validate_query(q: &mingli_contract::Query) -> Result<(), String> {
    validate_instant(q.year, q.month, q.day, q.hour, q.minute, q.tz)?;
    validate_coords(q.latitude, q.longitude)
}

/// 公历某年某月有几天。`month` 须已在 1–12 内。
#[must_use]
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 0,
    }
}

/// 坐标的取值域。纬度不在 `Birth` 上（只有占星那一路要它，走 `Query`），故单独一条。
///
/// # Errors
///
/// 纬度不在 −90–90、或经度不在 −180–180 时返回说明。
pub fn validate_coords(latitude: Option<f64>, longitude: Option<f64>) -> Result<(), String> {
    if let Some(lat) = latitude
        && !(-90.0..=90.0).contains(&lat)
    {
        return Err("latitude 须在 −90 到 90 之间".into());
    }
    if let Some(lon) = longitude
        && !(-180.0..=180.0).contains(&lon)
    {
        return Err("longitude 须在 −180 到 180 之间".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(year: i32, month: u32, day: u32) -> Birth {
        Birth { year, month, day, hour: 12, minute: 0, tz: 8.0, gender: None, true_solar_time: false, longitude: None }
    }

    fn hm(hour: u32, minute: u32) -> Birth {
        Birth { year: 1990, month: 6, day: 15, hour, minute, tz: 8.0, gender: None, true_solar_time: false, longitude: None }
    }

    /// 时与分也要收，理由与「日」那一条一样：不收不是多算，而是**算成另一时刻**。
    ///
    /// 打成 25 点的人不会收到任何提示——`hour_branch` 把 25 点折成丑时，
    /// 儒略日把它滚进次日，返回的是一张别的盘。此前时与分一次没测过：
    /// 把 `hour > 23 || minute > 59` 的 `||` 换成 `&&`（于是只有两者同时越界才拦），
    /// 全量套件一条都不红。
    ///
    /// 两个边界各站两次，并且**分开**试——`||` 与 `&&` 只有在「一个越界一个不越界」
    /// 时才分得出来。
    #[test]
    fn an_hour_or_a_minute_out_of_range_is_refused_rather_than_rolled_over() {
        for hour in 0..=23u32 {
            assert!(hm(hour, 0).validate().is_ok(), "{hour} 点是合法的");
        }
        for minute in 0..=59u32 {
            assert!(hm(12, minute).validate().is_ok(), "12 点 {minute} 分是合法的");
        }
        // 只有时越界（分合法）——`&&` 会在这里放行。
        assert!(hm(24, 0).validate().is_err(), "24 点应被拒");
        assert!(hm(25, 30).validate().is_err(), "25 点 30 分应被拒");
        assert!(hm(99, 59).validate().is_err());
        // 只有分越界（时合法）——`&&` 同样会在这里放行。
        assert!(hm(12, 60).validate().is_err(), "60 分应被拒");
        assert!(hm(0, 100).validate().is_err());
        // 两者都越界当然也要拒。
        assert!(hm(24, 60).validate().is_err());

        // 越界的时刻若放过去会变成什么：25 点被折进次日丑时，与真正的次日 01:00 同盘。
        // 这正是必须在门口拦下的理由——放行不会报错，只会悄悄换一张盘。
        let rolled = mingli_astro::Moment::new(1990, 6, 15, 25, 0, 8.0);
        let next_day_1am = mingli_astro::Moment::new(1990, 6, 16, 1, 0, 8.0);
        let same_day = mingli_astro::Moment::new(1990, 6, 15, 12, 0, 8.0);
        // 儒略日把它当作次日凌晨一点……
        assert!(
            (rolled.jd_ut - next_day_1am.jd_ut).abs() < 1e-9,
            "25 点的瞬时等同于次日 01:00"
        );
        // ……而民用日序仍停在当日。同一个时刻对象内部就对不上：
        // 吃 `jd_ut` 的（节气、月柱、太阳黄经）看到十六日，吃 `civil_day` 的
        // （日柱、农历）看到十五日，拼出来的是两天合成的一张盘。
        assert_eq!(rolled.civil_day, same_day.civil_day, "民用日序仍是十五日");
        assert_ne!(rolled.civil_day, next_day_1am.civil_day, "两者对不上，正是要在门口拦下的理由");
    }

    /// 日要按当月实际长度收。
    ///
    /// 不收的后果不是「多算一天」而是**算成另一天**：历法换算会把 1990-02-31 悄悄挪到
    /// 1990-03-03，两者返回的日柱同为丁卯、农历同为二月初七，打错一个数字的人
    /// 拿到的是别人的盘，而界面上没有任何迹象。
    #[test]
    fn a_day_that_does_not_exist_is_refused_rather_than_slid_to_the_next_month() {
        assert!(at(1990, 2, 31).validate().is_err());
        assert!(at(1990, 2, 29).validate().is_err(), "1990 不是闰年");
        assert!(at(1990, 4, 31).validate().is_err(), "四月只有 30 天");
        assert!(at(1990, 2, 28).validate().is_ok());
        assert!(at(1990, 1, 31).validate().is_ok());
    }

    /// 百年闰规则在支持区间的两端各踩一次：1900 与 2100 都被 400 排除，2000 没有。
    #[test]
    fn the_century_leap_rule_holds_at_both_ends_of_the_supported_range() {
        assert_eq!(days_in_month(1900, 2), 28, "1900 能被 100 整除、不能被 400 整除");
        assert_eq!(days_in_month(2000, 2), 29, "2000 能被 400 整除");
        assert_eq!(days_in_month(2100, 2), 28);
        assert_eq!(days_in_month(2024, 2), 29);
        // 其余各月与年份无关
        for y in [1900, 2000, 2024, 2100] {
            let total: u32 = (1..=12).map(|m| days_in_month(y, m)).sum();
            assert_eq!(total, if days_in_month(y, 2) == 29 { 366 } else { 365 }, "{y} 年各月之和");
        }
    }

    /// 时区与坐标的取值域。+14 是 Kiritimati，真实存在，不能收得比它窄。
    #[test]
    fn the_offset_and_the_coordinates_stay_inside_the_real_world() {
        let tz = |t: f64| Birth { tz: t, ..at(1990, 6, 15) };
        assert!(tz(14.0).validate().is_ok(), "+14 是 Kiritimati 的真实偏移");
        assert!(tz(-12.0).validate().is_ok());
        assert!(tz(14.5).validate().is_err());
        assert!(tz(99.0).validate().is_err());

        assert!(validate_coords(Some(90.0), Some(180.0)).is_ok());
        assert!(validate_coords(Some(-90.0), Some(-180.0)).is_ok());
        assert!(validate_coords(Some(91.0), None).is_err());
        assert!(validate_coords(None, Some(181.0)).is_err());
        assert!(validate_coords(None, None).is_ok(), "两者都可缺");
    }
}
