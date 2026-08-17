//! 端口层：叶与编排之间的契约。
//!
//! 这里只有**抽象**——一片叶要长成什么样（[`CastingEngine`]）、一次排盘的输入是什么
//! （[`Query`]）、一片叶如何声明自己的确定性边界与流派（[`DetItem`] / [`SchoolItem`]）、
//! 需求侧有哪几类问局（[`IntentSpec`]）。
//!
//! 依赖方向：叶实现这里的 trait，编排层消费这里的 trait，**双方都不认识对方**。
//! 本 crate 除共享时刻 [`mingli_astro::Moment`] 外不依赖任何领域实现。
//!
//! 按关注点分四份：[`ports`] 两个 trait 本身、[`query`] 输入、[`declare`] 叶的自述、
//! [`intent`] 需求侧的问局分类。

pub use mingli_astro::Moment;

pub mod declare;
pub mod intent;
pub mod ports;
pub mod query;

pub use declare::{d, s, DetItem, Determinism, Family, SchoolItem};
pub use intent::{intents, Intent, IntentSpec, IntentStatus};
pub use ports::{effective_school_id, Bearing, CastingEngine, LeafOutput, Principal, WordEngine, WordQuery};
pub use query::{effective_seed, AskTime, Gender, Query, QueryKind, Subject};

#[cfg(test)]
mod tests;
