//! 紫微斗数用例：本命盘。

use crate::Birth;
use mingli_contract::Gender;
use mingli_ziwei::{BirthInput, ZiweiChart};

/// 契约层性别 → 紫微叶性别。
fn leaf_gender(g: Option<Gender>) -> Option<mingli_ziwei::Gender> {
    g.map(|x| match x {
        Gender::Male => mingli_ziwei::Gender::Male,
        Gender::Female => mingli_ziwei::Gender::Female,
    })
}

/// 本命盘。
#[must_use]
pub fn natal(b: &Birth) -> ZiweiChart {
    mingli_ziwei::compute(BirthInput {
        year: b.year,
        month: b.month,
        day: b.day,
        hour: b.hour,
        minute: b.minute,
        tz: b.tz,
        gender: leaf_gender(b.gender),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn natal_matches_the_known_oracle() {
        let c = natal(&Birth {
            year: 1990,
            month: 6,
            day: 15,
            hour: 14,
            minute: 30,
            tz: 8.0,
            gender: Some(Gender::Male),
            true_solar_time: false,
            longitude: None,
        });
        assert_eq!(c.ming_ganzhi, "丁亥");
    }
}
