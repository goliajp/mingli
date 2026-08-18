//! 请求 DTO 与它们到领域类型的转换。
//!
//! 承接层的第一步只有这一件事：把线上的形状变成领域的形状。校验也在这里——
//! 领域类型不该为了迁就一个可能乱填的 HTTP body 而放宽自己的不变量。


// 各意图的入参形状住在用例层（见 `mingli_app::input`）——同一批用例还要给 wasm 用，
// 形状写在承接层就等于让两条交付路各写一遍。这里只留 HTTP 自己多出来的那些字段。
pub(crate) use mingli_app::input::{
    ElectionRequest, EventRequest, FortuneRequest, LocativeRequest, MundaneRequest, SynastryRequest, TeamRequest,
    WordRequest,
};
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
    /// `"male"` / `"female"`（也收 `"男"` / `"女"` 与首字母大写），缺省不算大运。
    #[serde(default)]
    pub(crate) gender: Option<Gender>,
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

/// DTO → 用例层入参。
pub(crate) fn birth(req: &ChartRequest) -> Birth {
    Birth {
        year: req.year,
        month: req.month,
        day: req.day,
        hour: req.hour,
        minute: req.minute,
        tz: req.tz.unwrap_or(8.0),
        gender: req.gender,
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
        gender: req.gender,
        latitude: req.latitude,
        longitude: req.longitude,
        seed: req.seed,
        name: req.name.clone(),
        schools: req.schools.clone().unwrap_or_default(),
    }
}

pub(crate) fn validate(req: &ChartRequest) -> Result<(), String> {
    birth(req).validate()?;
    // 纬度只在这一层的形状上（`Birth` 没有它），故在这里补一条；经度两处都收，重复无害
    mingli_app::validate_coords(req.latitude, req.longitude)
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
