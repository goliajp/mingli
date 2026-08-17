//! L2 主干：六十干支（sexagenary cycle）符号系统。
//!
//! 天干（10）与地支（12）并行推进，因 `gcd(10,12)=2`，其联合不是完整乘积 `Z₁₀×Z₁₂`
//! 而是阶为 `lcm(10,12)=60` 的对角子群——即六十甲子恰有 60 个组合（同阴阳配对），而非 120。
//! 这一结构由 [`mingli_core::cyclic`] 提供；本 crate 在其上构建干支的领域语义
//! （五行、纳音、五虎遁、时辰、日柱递推）。对天文/历法零依赖：日柱以民用日序（JDN）为输入。

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "小循环群上的模运算：结果恒被约束在 [0，n) 内（n≤60），整数窄化安全"
)]

#![allow(
    clippy::wildcard_imports,
    reason = "各模块以 `use super::*` 共享 crate 顶层的领域 import——这是把一张大表拆成多文件的常规手法"
)]

use serde::Serialize;

pub mod cycle;
pub mod wuxing;
pub mod xun;
pub mod shensha;
#[cfg(test)]
mod tests;

// 全部出口在 crate 根平铺——拆成多文件是内部组织，对外仍是一片叶。
pub use cycle::*;
pub use wuxing::*;
pub use xun::*;
pub use shensha::*;
