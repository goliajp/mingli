//! 起课：从一个时刻算到一份完整的课。

use crate::plates::MONTH_GENERAL_NAMES;
use crate::transmission::derive_transmission;
use crate::{four_courses, heaven_plate, month_general_branch, plate_offset, Cast, SheHaiSchool};
use mingli_astro::Moment;

/// 在共享上下文 [`Moment`] 上起大六壬课（涉害取古法）。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    compute_at_with(m, SheHaiSchool::Classical)
}

/// 在共享上下文上起课，指定涉害流派。
#[must_use]
pub fn compute_at_with(m: &Moment, school: SheHaiSchool) -> Cast {
    let day = mingli_ganzhi::day_ganzhi(m.civil_day);
    let hb = mingli_ganzhi::hour_branch(m.hour, m.minute);
    let mg = month_general_branch(m.sun_longitude);
    let offset = plate_offset(mg, hb);
    let mut heaven = [0u8; 12];
    for (g, h) in heaven.iter_mut().enumerate() {
        #[allow(clippy::cast_possible_truncation, reason = "g∈0..12")]
        let gg = g as u8;
        *h = heaven_plate(gg, offset);
    }
    let courses = four_courses(day.stem, day.branch, offset);
    let (pattern, transmission) =
        derive_transmission(&courses, day.stem, day.branch, offset, school);
    Cast {
        day_stem: day.stem,
        day_branch: day.branch,
        hour_branch: hb,
        month_general: mg,
        month_general_name: MONTH_GENERAL_NAMES[mg as usize],
        offset,
        heaven,
        courses,
        pattern,
        pattern_label: pattern.label(),
        transmission,
    }
}

/// 由本地民用时刻起课（独立入口，构造 [`Moment`]）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, hour: u32, minute: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, hour, minute, tz))
}
