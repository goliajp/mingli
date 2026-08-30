//! L3 叶（⟂ 横切 / 确定性）：大六壬起课。
//!
//! 大六壬把「时间」折成一个 `Z₁₂` 上的盘旋转，再由日干支取四课、发三传：
//!
//! 1. **天地盘**：地盘十二支固定顺布；天盘 = 地盘整体平移，偏移 `offset = (月将支 − 时支) mod 12`
//!    （「月将加占时」）。地盘第 `g` 宫之上神 = `(g + offset) mod 12`（[`heaven_plate`]）。
//! 2. **月将**：太阳过宫，每过一中气换将，随黄经递减（[`month_general_branch`]）；雨水后日躔亥=登明。
//! 3. **四课**：用天干寄宫（[`STEM_LODGING`]）取一课，层层取天盘上神得四课（[`four_courses`]）。
//! 4. **三传**：先判课式（九宗门），再取传。
//!
//! 验证：天地盘 + 四课校验古法工作例「亥将子时甲子日 → 四课 丑/子/亥/戌」。
//!
//! 九宗门取传已全部实现：贼克 / 比用 / 遥克 / 伏吟 / 返吟（有克与无克两路）/ 昴星 / 别责 / 八专；
//! 涉害亦取传，但**取用法两派**（数不数「受克深浅」），见 [`SheHaiSchool`]。
//! 九宗门取传已全部实现。
//!
//! 三门的取传各自有一张**全表课数**可对账，这是本叶最硬的校验面：
//! 昴星恰 16 课（刚 4 柔 12）、别责恰 9 课（刚 3 柔 6）、八专恰 16 课（刚 6 柔 10），
//! 且八专里三传三字全同的「独足课」有且仅有一课。这些数字在《六壬大全》与《六壬粹言》
//! 两部彼此独立的书里各自被自报过。

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::trivially_copy_pass_by_ref,
    reason = "盘位全在 Z₁₂（0..12）小范围内换算；Course/数组按引用传是为可读性，受控安全"
)]

#[cfg(feature = "port")]
mod bearings;
mod compute;
mod courses;
#[cfg(feature = "port")]
mod engine;
mod plates;
mod transmission;
mod types;

#[cfg(feature = "port")]
pub use bearings::{bearings_of, BRANCH_DIR, BRANCH_NAMES};
pub use compute::{compute, compute_at, compute_at_with};
pub use courses::{four_courses, Course};
#[cfg(feature = "port")]
pub use engine::LiurenEngine;
pub use plates::{heaven_plate, month_general_branch, plate_offset, MONTH_GENERAL_NAMES, STEM_LODGING};

pub use types::{Cast, Pattern, SheHaiSchool};

#[cfg(test)]
mod tests;
