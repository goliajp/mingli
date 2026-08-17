//! 端点按意图分文件。每个 handler 只做三步：解 DTO、调用例、映射错误。
//!
//! 「它有没有多做第四步」不靠人读——`tests/no_drift.rs` 把用例层直出与端点 body 逐字节比。

pub mod election;
pub mod event;
pub mod locative;
pub mod meta;
pub mod mundane;
pub mod natal;
pub mod synastry;
pub mod team;
pub mod word;
