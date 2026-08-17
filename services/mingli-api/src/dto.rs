//! 请求 DTO 与它们到领域类型的转换。
//!
//! 承接层的第一步只有这一件事：把线上的形状变成领域的形状。校验也在这里——
//! 领域类型不该为了迁就一个可能乱填的 HTTP body 而放宽自己的不变量。


use mingli_app::Birth;
use mingli_contract::{Gender, Query};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct ChartRequest {
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
    pub(crate) hour: u32,
    #[serde(default)]
    pub(crate) minute: u32,
    /// 时区偏移小时，缺省 +8（中国）。日本传 9。
    pub(crate) tz: Option<f64>,
    /// "male" / "female"，缺省不算大运。
    pub(crate) gender: Option<String>,
    /// 出生地纬度（占星 Asc/MC 用，可选）。
    pub(crate) latitude: Option<f64>,
    /// 出生地经度（占星 Asc/MC 用，可选）。
    pub(crate) longitude: Option<f64>,
    /// C 族起卦可复现种子（可选）。
    pub(crate) seed: Option<u64>,
    /// 姓名（D 族数字学用，可选）。
    pub(crate) name: Option<String>,
    /// 要释义的叶 id（仅 /api/interpret 用，可选）。
    pub(crate) leaf: Option<String>,
    /// 流派选择： key=叶 id， value=该叶的流派 id（可选）。各叶 default 由 schools() 给出。
    pub(crate) schools: Option<std::collections::BTreeMap<String, String>>,
    /// 真太阳时：true 则按 longitude + EoT 校正时柱。默认 false（钟表时）。
    #[serde(default)]
    pub(crate) true_solar_time: bool,
    /// 主体类型(`person`/`company`/`product`/`event`)：仅释义层(/api/interpret)生效。默认 person。
    pub(crate) subject: Option<String>,
}

/// HTTP 的性别字面 → 契约层性别。
pub(crate) fn parse_gender(g: &Option<String>) -> Option<Gender> {
    match g.as_deref() {
        Some("male" | "男") => Some(Gender::Male),
        Some("female" | "女") => Some(Gender::Female),
        _ => None,
    }
}

/// DTO → 用例层入参。
pub(crate) fn birth(req: &ChartRequest) -> Birth {
    Birth {
        year: req.year,
        month: req.month,
        day: req.day,
        hour: req.hour,
        minute: req.minute,
        tz: req.tz.unwrap_or(8.0),
        gender: parse_gender(&req.gender),
        true_solar_time: req.true_solar_time,
        longitude: req.longitude,
    }
}

/// DTO → 全叶排盘的共享查询。
pub(crate) fn engine_query(req: &ChartRequest) -> Query {
    Query {
        year: req.year,
        month: req.month,
        day: req.day,
        hour: req.hour,
        minute: req.minute,
        tz: req.tz.unwrap_or(8.0),
        gender: parse_gender(&req.gender),
        latitude: req.latitude,
        longitude: req.longitude,
        seed: req.seed,
        name: req.name.clone(),
        schools: req.schools.clone().unwrap_or_default(),
    }
}

pub(crate) fn validate(req: &ChartRequest) -> Result<(), String> {
    birth(req).validate()
}

/// 岁运叠加旺衰：本命 + extras（大运柱、流年柱等）。
/// extras 干支以字符串「癸未」形式传入，后端解析。
#[derive(Debug, Deserialize)]
pub(crate) struct OverlayRequest {
    #[serde(flatten)]
    pub(crate) natal: ChartRequest,
    /// 叠加干支字符串列表（顺序无关）。如 `["丁酉","丙午"]` = 大运丁酉柱 + 流年丙午柱。
    pub(crate) extras: Vec<String>,
}

/// 团队合盘：N 人生辰 → N 张本命盘 + 团队五行画像 + 互补矩阵。
#[derive(Debug, Deserialize)]
pub(crate) struct TeamRequest {
    pub(crate) members: Vec<TeamMember>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TeamMember {
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
    pub(crate) hour: u32,
    #[serde(default)]
    pub(crate) minute: u32,
    pub(crate) tz: Option<f64>,
    pub(crate) gender: Option<String>,
    #[serde(default)]
    pub(crate) name: Option<String>,
}

/// DTO 成员 → 用例层成员。
pub(crate) fn team_members(req: &TeamRequest) -> Vec<mingli_app::team::Member<'_>> {
    req.members
        .iter()
        .map(|m| mingli_app::team::Member { birth: member_birth(m), name: m.name.as_deref() })
        .collect()
}

/// D 族字/词模态请求：gematria/abjad 吃文字；wuge 吃笔画（康熙笔画表不内置，由调用方给）。
#[derive(Debug, Deserialize)]
pub(crate) struct WordRequest {
    /// "gematria" / "abjad" / "wuge"。
    pub(crate) system: String,
    /// gematria（希伯来）/abjad（阿拉伯） 的词。
    pub(crate) text: Option<String>,
    /// wuge：姓各字笔画。
    pub(crate) surname: Option<Vec<u32>>,
    /// wuge：名各字笔画。
    pub(crate) given: Option<Vec<u32>>,
}

/// Fortune：t 时刻运势切片 + 100 年用神供给时间序列。
/// body = {natal: ChartRequest, t_target: AskTime(y/m/d/h/min/tz), timeline_max_age?: u32}
#[derive(Debug, Deserialize)]
pub(crate) struct FortuneRequest {
    /// 本命输入（全量 ChartRequest：出生 y/m/d/h/min/tz/性别等）。
    pub(crate) natal: ChartRequest,
    /// 目标时刻(year/month/day/hour/minute/tz)。
    pub(crate) t_target: TTime,
    /// 时间序列扫描上限年龄。默认 100。
    #[serde(default)]
    pub(crate) timeline_max_age: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TTime {
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
    #[serde(default)]
    pub(crate) hour: u32,
    #[serde(default)]
    pub(crate) minute: u32,
    #[serde(default = "default_tz")]
    pub(crate) tz: f64,
}

pub(crate) fn default_tz() -> f64 { 8.0 }

/// 占事请求：问事此刻 + 取机 + 问句。
#[derive(Debug, Deserialize)]
pub(crate) struct EventRequest {
    /// 问事此刻（缺省字段按 0 / +8 处理）。
    pub(crate) t_ask: TTime,
    /// 取机种子；缺省表示未取机，各叶按问事时刻自行派生。
    pub(crate) seed: Option<u64>,
    /// 问句（只入释义，不参与计算）。
    pub(crate) question: Option<String>,
}

pub(crate) fn ask_time(t: &TTime) -> mingli_contract::AskTime {
    mingli_contract::AskTime {
        year: t.year,
        month: t.month,
        day: t.day,
        hour: t.hour,
        minute: t.minute,
        tz: t.tz,
    }
}

/// 择吉请求：时窗 + 事类。
#[derive(Debug, Deserialize)]
pub(crate) struct ElectionRequest {
    /// 时窗起（含）。
    pub(crate) window_start: TTime,
    /// 时窗止（含）。
    pub(crate) window_end: TTime,
    /// 事类（婚 / 葬 / 动土 / 行 / 开业…；只入释义，不参与排序）。
    pub(crate) category: Option<String>,
}

/// 寻方位请求：问事此刻 + 取机 + 所寻。
#[derive(Debug, Deserialize)]
pub(crate) struct LocativeRequest {
    /// 问事此刻。
    pub(crate) t_ask: TTime,
    /// 取机种子（可缺）。
    pub(crate) seed: Option<u64>,
    /// 所寻（人 / 物 / 向；只入释义）。
    pub(crate) category: Option<String>,
}

/// 合盘请求：甲乙两人。
#[derive(Debug, Deserialize)]
pub(crate) struct SynastryRequest {
    /// 甲方。
    pub(crate) a: TeamMember,
    /// 乙方。
    pub(crate) b: TeamMember,
}

pub(crate) fn member_birth(m: &TeamMember) -> Birth {
    Birth {
        year: m.year,
        month: m.month,
        day: m.day,
        hour: m.hour,
        minute: m.minute,
        tz: m.tz.unwrap_or(8.0),
        gender: parse_gender(&m.gender),
        true_solar_time: false,
        longitude: None,
    }
}

/// 国运请求：奠基时刻 + 坐标（占星立国盘用）+ 目标年 + 时间线年数。
#[derive(Debug, Deserialize)]
pub(crate) struct MundaneRequest {
    /// 政体奠基时刻。
    pub(crate) founded_at: TTime,
    /// 奠基地纬度（可缺）。
    pub(crate) latitude: Option<f64>,
    /// 奠基地经度（可缺）。
    pub(crate) longitude: Option<f64>,
    /// 目标年（年度盘所在年，缺省取立国年）。
    pub(crate) target_year: Option<i32>,
    /// 时间线年数（缺省 24，上限 72）。
    pub(crate) span: Option<u32>,
}
