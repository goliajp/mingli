//! L3 叶（⟂ 横切 / 确定性）：奇门遁甲（时家转盘法）的可计算结构。
//!
//! 一次排盘按「定局 → 四盘 → 判读」推进，每一步各住一个模块：
//!
//! | 模块 | 管什么 | 关键出口 |
//! |---|---|---|
//! | [`setup`] | 定局：节气定阴阳遁、符头定三元、查 72 局表 | [`solar_term_setup`] · [`yuan_of_branch`] |
//! | [`earth`] | 地盘：三奇六仪按局数布九宫 | [`earth_plate`] |
//! | [`sky`] | 天盘：九星与三奇六仪随值符旋转 | [`sky_rotation`] |
//! | [`gates`] | 人盘：值使门数落宫，八门同步旋转 | [`gate_plate`] |
//! | [`spirits`] | 神盘：直符与值符同宫，七神阳顺阴逆 | [`spirit_plate`] |
//! | [`vigor`] | 旺相休囚死：以月令衡量九星八门五行 | [`vigor_of`] |
//! | [`mod@patterns`] | 格局：伏吟反吟、三奇得使等结构判定 | [`patterns()`] |
//! | [`cast`] | 起局：把上面这些拼成一份完整盘面 | [`compute_at`] |
//!
//! 三处容易混淆、也是校验重点的地方：
//!
//! - **地盘走宫是宫序号 1→9 线性**，不是九宫飞星的斜线。阳遁六仪顺布、三奇逆布
//!   （实排序列 `戊己庚辛壬癸丁丙乙`），阴遁镜像。阳遁一局对古法
//!   「坎1戊·坤2己·震3庚·巽4辛·中5壬·乾6癸·兑7丁·艮8丙·离9乙」。
//! - **天盘是沿后天八卦圆周的一次刚体旋转**，不是逐星各算。中宫不在圆周上，寄宫另论。
//! - **72 局常数表**六源零冲突，且自带结构不变量：阳遁「中元 = 上元 + 6、下元 = 上元 + 3」，
//!   阴遁「−6 / −3」——表与不变量互为校验。
//!
//! 诚实边界（🟡）：八神第 5 / 6 位的**称谓**两系不一（白虎 / 玄武 与 勾陈 / 朱雀），位序则一致，
//! 故两名并出；天禽与中宫寄宫取通行的坤 2，古本「阳遁寄艮 8」一派未开关；
//! 定局的「拆补法 / 置闰法」差异只在交节临界数日的元 / 局对齐，本 crate 用**主流拆补法**（符头定元）。

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "节气/局数/宫位均落 0..24 / 1..9 小范围，与 i64/usize 间换算受控安全"
)]
#![allow(
    clippy::wildcard_imports,
    reason = "叶内各模块以 `use super::*` 共享 crate 顶层的领域 import——这是把一张大盘拆成多文件的常规手法"
)]

mod engine;
pub use engine::QimenEngine;

// 各模块经 `use super::*` 共享这三个顶层 import。
use mingli_astro::Moment;
use mingli_ganzhi::Element;
use serde::Serialize;

pub mod bearings;
pub mod cast;
pub mod earth;
pub mod gates;
pub mod patterns;
pub mod setup;
pub mod sky;
pub mod spirits;
pub mod vigor;
#[cfg(test)]
mod tests;

// 全部出口在 crate 根平铺——拆成多文件是内部组织，对外仍是一片叶。
pub use bearings::*;
pub use cast::*;
pub use earth::*;
pub use gates::*;
pub use patterns::*;
pub use setup::*;
pub use sky::*;
pub use spirits::*;
pub use vigor::*;
