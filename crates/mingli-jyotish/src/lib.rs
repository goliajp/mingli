//! L3 叶（B 族）：印度占星（Jyotish）。
//!
//! 与西洋占星（B 族）复用同一行星地心经度，但采用**恒星黄道**(sidereal zodiac)
//! 而非回归黄道(tropical zodiac)。两者差 = `ayanamsa`（春分点岁差累积位移）。
//!
//! - 9 行星（Surya/Chandra/Mangala/Budha/Guru/Shukra/Shani + Rahu/Ketu 月升降交点）；
//! - 27 nakshatra（月宿，每 13°20'）；名表 + Vimshottari mahadasha 主星 9 行星序列；
//! - 12 rasi（白羊..双鱼，与西洋占星 12 sign 同）；
//! - Lagna（上升） = Asc(tropical， [`mingli_ephemeris::asc_mc`]) − ayanamsa。
//!
//! # Ayanamsa 流派
//! [`Ayanamsa::Lahiri`] （默认）： 印度政府 1955 历改采用，N. C. Lahiri 提案。
//! [`Ayanamsa::Krishnamurti`]： KP 派 K. S. Krishnamurti， 与 Lahiri 差 ~6′。
//! [`Ayanamsa::Raman`] / [`Ayanamsa::FaganBradley`]:
//! 余两派，本叶按 J2000 静态偏移取值（强权威：Swiss Ephemeris 源码 SE_SIDM 表）。
//!
//! # 算法注
//! Lahiri ayanamsa 在 1956-01-01 TT(JD 2435553.5)= 23.245524743°（Swiss Ephemeris 源 `sweph.h` anchor），
//! 以平岁差速率（IAU 1976 简化）`50.290966″/yr` 线性外推。1900–2050 间容差 ~±0.05°(月宿 13°20'
//! 跨度 800'，此精度足以唯一确定 nakshatra/rasi)。更严格的 Vondrák/SE 实现可作 [`mingli_ephemeris`]
//! 本叶诚实标注容差，不写出超过证据的精度。
//!
//! # 校验 oracle
//! - Lahiri @ J2000.0：23°51'11" ≈ 23.85306°（Jagannath Hora / Wikipedia，本算误差 < 6′）。
//! - Lahiri @ 1956-01-01 TT：23.245524743°（Swiss Ephemeris 源精确 anchor，本算精度 < 0.001°）。
//! - 27 nakshatra 名表 + Vimshottari 主星序列：Wikipedia + GrahaGuru + Vedicka 3 源完全一致。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "黄经/nakshatra index 全为有界小整数(< 27)，f64 mantissa 充足"
)]

#![allow(
    clippy::wildcard_imports,
    reason = "叶内各模块以 `use super::*` 共享 crate 顶层的领域 import——这是把一张大盘拆成多文件的常规手法"
)]

pub mod varga;
#[cfg(feature = "port")]
mod engine;
pub mod kuta;
pub use varga::{all_vargas, varga_rasi, Varga, VargaPositions};
#[cfg(feature = "port")]
pub use engine::JyotishEngine;

use mingli_astro::Moment;
use mingli_ephemeris::{asc_mc, GeoLocation};
use mingli_ephemeris::{geocentric_ecliptic_longitude, Body};
#[cfg(feature = "serde")]
use serde::Serialize;

pub use mingli_ephemeris::mean_lunar_node;

pub mod ayanamsa;
pub mod nakshatra;
pub mod graha;
pub mod dasha;
pub mod chart;
#[cfg(test)]
mod tests;

// 全部出口在 crate 根平铺——拆成多文件是内部组织，对外仍是一片叶。
pub use ayanamsa::*;
pub use nakshatra::*;
pub use graha::*;
pub use dasha::*;
pub use chart::*;
