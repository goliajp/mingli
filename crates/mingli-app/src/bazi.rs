//! 四柱相关用例：本命盘、岁运叠加旺衰、运势切片与百年时序。

use crate::Birth;
use mingli_bazi::{BaziChart, BirthInput};
use mingli_contract::{AskTime, Gender};
use serde_json::{json, Value};

/// 契约层性别 → 八字叶性别。
fn leaf_gender(g: Option<Gender>) -> Option<mingli_bazi::Gender> {
    g.map(|x| match x {
        Gender::Male => mingli_bazi::Gender::Male,
        Gender::Female => mingli_bazi::Gender::Female,
    })
}

/// 把用例入参转成八字叶的出生输入。
#[must_use]
pub fn birth_input(b: &Birth) -> BirthInput {
    BirthInput {
        year: b.year,
        month: b.month,
        day: b.day,
        hour: b.hour,
        minute: b.minute,
        tz: b.tz,
        gender: leaf_gender(b.gender),
    }
}

/// 本命盘。开启真太阳时且给了经度时走校正排法，否则走钟表时。
#[must_use]
pub fn natal(b: &Birth) -> BaziChart {
    let input = birth_input(b);
    match (b.true_solar_time, b.longitude) {
        (true, Some(lon)) => mingli_bazi::compute_with_true_solar(input, lon),
        _ => mingli_bazi::compute(input),
    }
}

/// 岁运叠加旺衰：本命 + extras（大运柱、流年柱等，干支字符串形式）。
///
/// # Errors
///
/// `extras` 含无法解析的干支字符串时返回错误说明。
pub fn overlay_strength(b: &Birth, extras: &[String]) -> Result<Value, String> {
    let chart = natal(b);
    let parsed: Vec<_> = extras.iter().filter_map(|s| mingli_bazi::parse_ganzhi(s)).collect();
    if parsed.len() != extras.len() {
        return Err("extras 含无法解析的干支字符串".into());
    }
    // 本命四柱由引擎自产，必然可解析。
    let year_gz = mingli_bazi::parse_ganzhi(&chart.year.ganzhi).ok_or("本命年柱解析失败")?;
    let month_gz = mingli_bazi::parse_ganzhi(&chart.month.ganzhi).ok_or("本命月柱解析失败")?;
    let day_gz = mingli_bazi::parse_ganzhi(&chart.day.ganzhi).ok_or("本命日柱解析失败")?;
    let hour_gz = mingli_bazi::parse_ganzhi(&chart.hour.ganzhi).ok_or("本命时柱解析失败")?;
    let yun = mingli_bazi::compute_strength_with_extras(year_gz, month_gz, day_gz, hour_gz, &parsed);
    let delta = i32::try_from(yun.score).unwrap_or(0) - i32::try_from(chart.strength.score).unwrap_or(0);
    Ok(json!({
        "ming": chart.strength,
        "yun": yun,
        "delta_score": delta,
        "extras": extras,
    }))
}

/// 时序扫描的年龄上限（对外默认 100，硬顶 120）。
const MAX_AGE_CAP: u32 = 120;

/// 运势：t 时刻切片 + 百年用神供给时序。
///
/// # Errors
///
/// 缺性别时返回错误——大运顺逆由性别决定，没有它算不出运。
pub fn fortune(b: &Birth, t: &AskTime, timeline_max_age: Option<u32>) -> Result<Value, String> {
    if b.gender.is_none() {
        return Err("fortune 需性别（决定大运顺逆），缺 gender".into());
    }
    let input = birth_input(b);
    let max_age = timeline_max_age.unwrap_or(100).min(MAX_AGE_CAP);
    let at = mingli_bazi::fortune_at(input, t.year, t.month, t.day, t.hour, t.minute, t.tz);
    let timeline = mingli_bazi::fortune_supply_timeline(input, max_age);
    Ok(json!({ "at": at, "timeline": timeline, "max_age": max_age }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Birth {
        Birth {
            year: 1990,
            month: 6,
            day: 15,
            hour: 14,
            minute: 30,
            tz: 8.0,
            gender: Some(Gender::Male),
            true_solar_time: false,
            longitude: None,
        }
    }

    #[test]
    fn natal_matches_the_known_oracle() {
        let c = natal(&sample());
        assert_eq!(
            [c.year.ganzhi.as_str(), c.month.ganzhi.as_str(), c.day.ganzhi.as_str(), c.hour.ganzhi.as_str()],
            ["庚午", "壬午", "辛亥", "乙未"]
        );
    }

    #[test]
    fn overlay_rejects_unparsable_extras() {
        assert!(overlay_strength(&sample(), &["不是干支".to_string()]).is_err());
        let ok = overlay_strength(&sample(), &["戊午".to_string()]).expect("合法干支应通过");
        assert!(ok["delta_score"].is_i64() && ok["ming"].is_object() && ok["yun"].is_object());
    }

    #[test]
    fn fortune_needs_gender_and_caps_timeline() {
        let t = AskTime { year: 2026, month: 8, day: 16, hour: 12, minute: 0, tz: 8.0 };
        let mut no_sex = sample();
        no_sex.gender = None;
        assert!(fortune(&no_sex, &t, None).is_err());

        let v = fortune(&sample(), &t, Some(999)).expect("有性别应可算");
        assert_eq!(v["max_age"], MAX_AGE_CAP);
        assert_eq!(v["timeline"].as_array().map(Vec::len), Some(MAX_AGE_CAP as usize + 1));
    }

    #[test]
    fn true_solar_time_only_applies_with_longitude() {
        let mut b = sample();
        b.true_solar_time = true;
        // 没给经度 → 静默回退钟表时（数据完整性优先于校正信仰）
        assert_eq!(natal(&b).hour.ganzhi, natal(&sample()).hour.ganzhi);
    }
}
