//! 二十四种术数的算法内核，一个名字取用。
//!
//! ```toml
//! mingli = { version = "1", default-features = false, features = ["bazi", "yijing"] }
//! ```
//!
//! 每片叶也可以单独取用（`mingli-bazi = "1"`），那样连本 crate 都不必要；
//! 加上 `default-features = false` 之后，一片叶的依赖链上不会出现 serde——
//! 类型化出口本来就不经过 JSON，端口那一层在 `port` feature 之后。
//!
//! # 两种用法
//!
//! **要一张盘**：走 [`leaves`] 里那片叶的类型化 API，拿到的是结构体不是 JSON。
//!
//! **要全树**：把 [`registry`] 交给 [`engine`] 的 `cast_all`，一次输入排出所有已开的叶。

#![forbid(unsafe_code)]

/// 契约层：[`Query`](contract::Query)、`CastingEngine` 与确定性谱。
pub use mingli_contract as contract;
/// 编排层：共享时刻算一次，然后 fan-out 到每片叶。
pub use mingli_engine as engine;
/// 各叶的类型化 API，按 feature 转发（`mingli::leaves::bazi::compute(..)`）。
pub use mingli_registry::leaves;
pub use mingli_registry::{registry, word_registry};
