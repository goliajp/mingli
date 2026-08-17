//! 天地盘：月将、天干寄宫，以及「月将加占时」那一次平移。
//!
//! 大六壬把时间折成 `Z₁₂` 上的一次旋转——地盘十二支不动，天盘整体平移 `offset` 位。
//! 这一步之后的一切（四课、三传、方位）都读这张盘。

/// 十二月将名，按地支序索引（子=0…亥=11）。
pub const MONTH_GENERAL_NAMES: [&str; 12] = [
    "神后", "大吉", "功曹", "太冲", "天罡", "太乙", "胜光", "小吉", "传送", "从魁", "河魁", "登明",
];

/// 天干寄宫：天干（甲=0…癸=9）寄于某地支宫。四正（子午卯酉）不作寄宫，故丙戊同寄巳、丁己同寄未。
pub const STEM_LODGING: [u8; 10] = [2, 4, 5, 7, 5, 7, 8, 10, 11, 1];

/// 由太阳视黄经定月将地支（0..11）。每过一中气（黄经每 30°）月将递减；
/// `λ∈[0,30)`→戌(10)、`λ∈[330,360)`→亥(11)（雨水后日躔亥=登明）。
#[must_use]
pub fn month_general_branch(sun_longitude: f64) -> u8 {
    let s = (sun_longitude.rem_euclid(360.0) / 30.0).floor();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "s∈0..12"
    )]
    let s = s as i64;
    ((10 - s).rem_euclid(12)) as u8
}

/// 天地盘偏移：`(月将支 − 时支) mod 12`（月将加占时）。
#[must_use]
pub fn plate_offset(month_general: u8, hour_branch: u8) -> u8 {
    (12 + month_general - hour_branch) % 12
}

/// 地盘第 `ground` 宫之上的天盘地支：`(ground + offset) mod 12`。
#[must_use]
pub fn heaven_plate(ground: u8, offset: u8) -> u8 {
    (ground + offset) % 12
}
