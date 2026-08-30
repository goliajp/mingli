//! `Query` 与出生时刻的取值域。
//!
//! 校验属于契约而不属于用例：它说的是「什么样的入参算数」，与谁来用这份入参无关。
//! 从前它住在用例层，于是任何一扇只想排一张盘的门要么把用例层整个链进来，
//! 要么不校验——后者会让同一个 2 月 31 日在服务端被拒、在浏览器里被历法换算悄悄
//! 挪成 3 月 3 日，两扇门给出的不是同一个答案。搬到这里之后，拿得到 `Query` 的人
//! 就一定拿得到校验。
//!
//! 用例层按原路径 re-export 这几个函数，对外签名一字未动。

/// 一个时刻的取值域。两条交付路共用——HTTP 走 `Birth::validate`（承接层），wasm 走
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

/// 排盘入参的取值域。wasm 那扇门吃的是 [`crate::Query`] 而非 `Birth`，
/// 两者字段不同、该收的东西一样，故各有一个入口、共用同一段判断。
///
/// # Errors
///
/// 时刻或坐标越界时返回面向调用方的中文说明。
pub fn validate_query(q: &crate::Query) -> Result<(), String> {
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
