//! 四课：由日干支在天地盘上层层取上神。
//!
//! 一课取日干的寄宫，二课取一课上神之上，三课取日支，四课取三课上神之上。

use crate::plates::{heaven_plate, STEM_LODGING};
#[cfg(feature = "serde")]
use serde::Serialize;

/// 一课：下（地盘支）与上（天盘上神）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Course {
    /// 下神（地盘地支序 0..11）。
    pub down: u8,
    /// 上神（天盘地支序 0..11）。
    pub up: u8,
}

/// 起四课：一课=日干寄宫之上神；二课=一课上神之上神；三课=日支之上神；四课=三课上神之上神。
#[must_use]
pub fn four_courses(day_stem: u8, day_branch: u8, offset: u8) -> [Course; 4] {
    let c1d = STEM_LODGING[day_stem as usize];
    let c1u = heaven_plate(c1d, offset);
    let c2u = heaven_plate(c1u, offset);
    let c3u = heaven_plate(day_branch, offset);
    let c4u = heaven_plate(c3u, offset);
    [
        Course { down: c1d, up: c1u },
        Course { down: c1u, up: c2u },
        Course { down: day_branch, up: c3u },
        Course { down: c3u, up: c4u },
    ]
}
