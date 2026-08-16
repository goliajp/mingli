//! 用例层：一次完整请求要做的事。
//!
//! 承接层（HTTP / wasm）只负责协议转换与错误映射，凡是「先算本命、再拼岁运、
//! 再组一份对外结构」这类编排都住在这里，于是同一份用例可以同时被 axum 服务与
//! wasm 绑定复用，且能脱离 HTTP 单独测。
//!
//! 依赖方向：用例可以认识具体的叶（[`mingli_bazi`] 等实体），但**注册表由调用方注入**
//! ——装配根是更外层的事，用例不去 `mingli-registry` 里拿。

pub mod analysis;
pub mod bazi;
pub mod interpret;
pub mod team;
pub mod word;
pub mod ziwei;

use mingli_contract::Gender;

/// 一次出生/占问输入——用例层的公共入参。
///
/// 与 HTTP DTO 的区别：这里的字段已经过校验与默认值填充，是领域可直接消费的形状。
#[derive(Debug, Clone, Copy)]
pub struct Birth {
    /// 公历年（1900–2100）。
    pub year: i32,
    /// 公历月 1..12。
    pub month: u32,
    /// 公历日 1..31。
    pub day: u32,
    /// 时 0..23。
    pub hour: u32,
    /// 分 0..59。
    pub minute: u32,
    /// 时区偏移小时。
    pub tz: f64,
    /// 性别（缺省则不排大运）。
    pub gender: Option<Gender>,
    /// 是否按真太阳时校正时柱。
    pub true_solar_time: bool,
    /// 出生地经度（真太阳时校正需要）。
    pub longitude: Option<f64>,
}

impl Birth {
    /// 输入域校验。
    ///
    /// # Errors
    ///
    /// 年份越界、月/日/时/分越界时返回面向调用方的中文说明。
    pub fn validate(&self) -> Result<(), String> {
        if !(1900..=2100).contains(&self.year) {
            return Err("year 仅支持 1900–2100".into());
        }
        if !(1..=12).contains(&self.month) {
            return Err("month 须 1–12".into());
        }
        if !(1..=31).contains(&self.day) {
            return Err("day 须 1–31".into());
        }
        if self.hour > 23 || self.minute > 59 {
            return Err("hour/minute 越界".into());
        }
        Ok(())
    }
}
