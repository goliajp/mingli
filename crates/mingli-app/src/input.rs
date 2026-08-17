//! 各用例的入参形状。
//!
//! 这些结构原先长在 HTTP 的 DTO 里。于是 wasm 想接同一批用例，就得把同样的字段名、
//! 同样的缺省、同样的时刻转换再写一遍——两条交付路从此各自漂，而谁也不会因为漂了报错。
//!
//! 形状属于用例：「这个用例要哪些原子」是它自己的事。交付层各自只加自己那点东西
//! （HTTP 还要 `leaf` / `subject` 这类与释义端点有关的字段，wasm 什么都不加）。

use crate::Birth;
use mingli_contract::AskTime;
use serde::Deserialize;

/// 时区缺省 +8（中国）。
#[must_use]
pub fn default_tz() -> f64 {
    8.0
}

/// 一个时刻。比 [`AskTime`] 宽容：时、分、时区都可缺省。
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Moment {
    /// 公历年。
    pub year: i32,
    /// 公历月 1..12。
    pub month: u32,
    /// 公历日 1..31。
    pub day: u32,
    /// 时 0..23，缺省 0。
    #[serde(default)]
    pub hour: u32,
    /// 分 0..59，缺省 0。
    #[serde(default)]
    pub minute: u32,
    /// 时区偏移小时，缺省 +8。
    #[serde(default = "default_tz")]
    pub tz: f64,
}

impl Moment {
    /// 转成契约层的占测时刻。
    #[must_use]
    pub fn ask_time(&self) -> AskTime {
        AskTime {
            year: self.year,
            month: self.month,
            day: self.day,
            hour: self.hour,
            minute: self.minute,
            tz: self.tz,
        }
    }
}

/// 合盘 / 团队里的一个人：出生输入 + 称呼。
#[derive(Debug, Clone, Deserialize)]
pub struct MemberInput {
    /// 出生输入（字段与单人排盘平铺在一起）。
    #[serde(flatten)]
    pub birth: Birth,
    /// 称呼（缺省时由用例用出生年占位）。
    #[serde(default)]
    pub name: Option<String>,
}

/// 运势：本命 + 目标时刻。
#[derive(Debug, Clone, Deserialize)]
pub struct FortuneRequest {
    /// 本命输入。
    pub natal: Birth,
    /// 目标时刻。
    pub t_target: Moment,
    /// 时序扫描的年龄上限，缺省 100。
    #[serde(default)]
    pub timeline_max_age: Option<u32>,
}

/// 团队合盘：N 人。
#[derive(Debug, Clone, Deserialize)]
pub struct TeamRequest {
    /// 成员。
    pub members: Vec<MemberInput>,
}

/// 占事：问事此刻 + 取机 + 问句。
#[derive(Debug, Clone, Deserialize)]
pub struct EventRequest {
    /// 问事此刻。
    pub t_ask: Moment,
    /// 取机种子；缺省表示未取机，各叶按问事时刻自行派生。
    #[serde(default)]
    pub seed: Option<u64>,
    /// 问句（只入释义，不参与计算）。
    #[serde(default)]
    pub question: Option<String>,
}

/// 择吉：时窗 + 事类。
#[derive(Debug, Clone, Deserialize)]
pub struct ElectionRequest {
    /// 时窗起（含）。
    pub window_start: Moment,
    /// 时窗止（含）。
    pub window_end: Moment,
    /// 事类（婚 / 葬 / 动土 / 行 / 开业…；只入释义，不参与排序）。
    #[serde(default)]
    pub category: Option<String>,
}

/// 寻方位：问事此刻 + 取机 + 所寻。
#[derive(Debug, Clone, Deserialize)]
pub struct LocativeRequest {
    /// 问事此刻。
    pub t_ask: Moment,
    /// 取机种子（可缺）。
    #[serde(default)]
    pub seed: Option<u64>,
    /// 所寻（人 / 物 / 向；只入释义）。
    #[serde(default)]
    pub category: Option<String>,
}

/// 合盘：甲乙两人。
#[derive(Debug, Clone, Deserialize)]
pub struct SynastryRequest {
    /// 甲方。
    pub a: MemberInput,
    /// 乙方。
    pub b: MemberInput,
}

/// 国运：奠基时刻 + 坐标 + 目标年 + 时间线年数。
#[derive(Debug, Clone, Deserialize)]
pub struct MundaneRequest {
    /// 政体奠基时刻。
    pub founded_at: Moment,
    /// 奠基地纬度（可缺）。
    #[serde(default)]
    pub latitude: Option<f64>,
    /// 奠基地经度（可缺）。
    #[serde(default)]
    pub longitude: Option<f64>,
    /// 目标年（年度盘所在年，缺省取立国年）。
    #[serde(default)]
    pub target_year: Option<i32>,
    /// 时间线年数（缺省 24，上限 72）。
    #[serde(default)]
    pub span: Option<u32>,
}

/// 字/词术数：与时刻无关的第二条契约。
#[derive(Debug, Clone, Deserialize)]
pub struct WordRequest {
    /// `"gematria"` / `"abjad"` / `"wuge"`。
    pub system: String,
    /// gematria（希伯来）/ abjad（阿拉伯）的词。
    #[serde(default)]
    pub text: Option<String>,
    /// 五格：姓各字笔画。
    #[serde(default)]
    pub surname: Option<Vec<u32>>,
    /// 五格：名各字笔画。
    #[serde(default)]
    pub given: Option<Vec<u32>>,
}

impl TeamRequest {
    /// 借出成员，交给 [`crate::team::compute`]。
    #[must_use]
    pub fn members(&self) -> Vec<crate::team::Member<'_>> {
        self.members
            .iter()
            .map(|m| crate::team::Member { birth: m.birth, name: m.name.as_deref() })
            .collect()
    }
}
