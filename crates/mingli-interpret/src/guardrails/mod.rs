//! 各意图的护栏与读法提示，一意图一文件。
//!
//! 护栏的**正文**各意图确实不同——那是内容。重复的是**框**（起句、免责句、篇幅上限、
//! 「X 结果 JSON：」尾巴），那部分已抽到 [`crate::Prompt`]，不在这里重复六遍。

pub mod election;
pub mod event;
pub mod locative;
pub mod mundane;
pub mod natal;
pub mod synastry;
pub mod team;
