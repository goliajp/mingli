//! L3 叶：四柱推命（八字）排盘。
//!
//! 确定性「排盘」：用 `mingli-astro` 的天文/历法 + `mingli-ganzhi` 的干支符号，
//! 算出年/月/日/时四柱、十神、五行、农历、大运。年柱以立春为界，月柱以「节」换月，
//! 日柱由民用日序递推，时柱五鼠遁。不含「释义/文案」（那是表达层/LLM 的事）。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    reason = "干支/大运均为小循环群上的有界模运算与天数折算，整数窄化安全"
)]
#![allow(
    clippy::wildcard_imports,
    reason = "叶内各模块以 `use super::*` 共享 crate 顶层的领域 import——这是把一张大盘拆成多文件的常规手法"
)]

#[cfg(feature = "port")]
mod engine;
#[cfg(feature = "port")]
pub use engine::BaziEngine;

use mingli_astro::{solar_term_jd, solar_term_time_near, Moment};
use mingli_ganzhi::{
    branch_element, day_ganzhi, hidden_stems, hour_branch, is_friendly_to_day_master,
    is_kuigang_day, month_pillar_stem, nayin_element, shensha_by_branch_anchor,
    shensha_by_day_stem, stem_element, ten_god, twelve_stage, year_ganzhi, Element, GanZhi,
    BRANCHES, STEMS, TWELVE_STAGES,
};
pub use mingli_ganzhi::parse_ganzhi;
#[cfg(feature = "serde")]
use serde::Serialize;

mod types;
mod chart;
mod pattern;
mod strength;
mod yongshen;
mod houses;
mod solar;
mod team;
mod fortune;

pub use types::*;
pub use chart::*;
pub use pattern::*;
pub use strength::*;
pub use yongshen::*;
pub use houses::*;
pub use solar::*;
pub use team::*;
pub use fortune::*;

#[cfg(test)]
mod tests;
