//! mingli-core —— 树之根 / 数学石（L0 元 crate）。
//!
//! 纯数学，零世界数据、零依赖。提供张成全世界术数「算」层的 6 块代数石：
//! - [`cyclic`]    S1 循环群 + CRT（区分完整乘积 vs 对角子群）—— 家族 A
//! - [`quantizer`] S2 圆 S¹ 角度 → Z_n 分段 —— 家族 B
//! - [`gf2`]       S3 二进制格 (Z₂)^k + GF(2) 线性（转置/XOR/奇偶校验）—— 家族 C
//! - [`ringhash`]  S4 字符串→环求和 + 数字根 —— 家族 D
//! - [`group`]     S5 有限集上的群作用/置换（飞布/安星/无放回洗牌）—— 横切 ⟂
//! - [`sampler`]   S6 可审计随机种子 → 均匀抽样（家族 C 的随机源，种子可复现）
//!
//! 每块石头都带校验/性质测试，证明其承载的数学事实
//! （如干支 60≠120 的对角子群、地占法官恒偶的 GF(2) 定理）。

pub mod cyclic;
pub mod gf2;
pub mod group;
pub mod quantizer;
pub mod ringhash;
pub mod sampler;
