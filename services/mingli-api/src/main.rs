//! mingli-api —— 承接层（axum）。把命理大树的排盘引擎暴露为 HTTP JSON 端点。
//!
//! 端点：
//!   POST /api/bazi   —— 四柱推命（精盘）
//!   POST /api/bazi/overlay-strength —— 岁运叠加旺衰（本命 + extras 大运/流年柱）
//!   POST /api/ziwei  —— 紫微斗数（精盘）
//!   POST /api/cast   —— 全叶并行排盘（engine `cast_all_detailed`，含 id/显示名/家族/盘）
//!   GET  /api/health —— 含已注册叶清单
//! 请求体（各端点共用）：
//!   { "year":1990, "month":6, "day":15, "hour":14, "minute":30, "tz":8.0, "gender":"male",
//!     "latitude":31.23, "longitude":121.47, "seed":2024, "name":"Ada Lovelace" }
//!   latitude/longitude（占星 Asc/MC）、seed（C 族起卦可复现）、name（D 族数字学）均可选。

use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use mingli_app::Birth;
use mingli_contract::{CastingEngine, Gender, Query, WordEngine, WordQuery};
use serde::Deserialize;
use std::sync::OnceLock;
use tower_http::cors::CorsLayer;

/// 装配根只装配一次，全进程复用（原先每个请求都重建一遍注册表）。
fn leaves() -> &'static [Box<dyn CastingEngine>] {
    static REG: OnceLock<Vec<Box<dyn CastingEngine>>> = OnceLock::new();
    REG.get_or_init(mingli_registry::registry)
}

/// 字词叶注册表，同样只装配一次。
fn word_leaves() -> &'static [Box<dyn WordEngine>] {
    static REG: OnceLock<Vec<Box<dyn WordEngine>>> = OnceLock::new();
    REG.get_or_init(mingli_registry::word_registry)
}

#[derive(Debug, Deserialize)]
struct ChartRequest {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    #[serde(default)]
    minute: u32,
    /// 时区偏移小时，缺省 +8（中国）。日本传 9。
    tz: Option<f64>,
    /// "male" / "female"，缺省不算大运。
    gender: Option<String>,
    /// 出生地纬度（占星 Asc/MC 用，可选）。
    latitude: Option<f64>,
    /// 出生地经度（占星 Asc/MC 用，可选）。
    longitude: Option<f64>,
    /// C 族起卦可复现种子（可选）。
    seed: Option<u64>,
    /// 姓名（D 族数字学用，可选）。
    name: Option<String>,
    /// 要释义的叶 id（仅 /api/interpret 用，可选）。
    leaf: Option<String>,
    /// 流派选择： key=叶 id， value=该叶的流派 id（可选）。各叶 default 由 schools() 给出。
    schools: Option<std::collections::BTreeMap<String, String>>,
    /// 真太阳时：true 则按 longitude + EoT 校正时柱。默认 false（钟表时）。
    #[serde(default)]
    true_solar_time: bool,
    /// 主体类型(`person`/`company`/`product`/`event`)：仅释义层(/api/interpret)生效。默认 person。
    subject: Option<String>,
}

/// claude CLI 释义后端（外部非确定 I/O，故置于承接层；实现 `mingli_interpret::Interpreter`，可替换）。
struct ClaudeCli;

impl mingli_interpret::Interpreter for ClaudeCli {
    fn interpret(&self, prompt: &str) -> std::io::Result<String> {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut child = Command::new("claude")
            .arg("-p")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .ok_or_else(|| std::io::Error::other("no stdin"))?
            .write_all(prompt.as_bytes())?;
        let out = child.wait_with_output()?;
        if !out.status.success() {
            return Err(std::io::Error::other(
                String::from_utf8_lossy(&out.stderr).to_string(),
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }
    fn backend(&self) -> &'static str {
        "claude-cli"
    }
}

/// HTTP 的性别字面 → 契约层性别。
fn parse_gender(g: &Option<String>) -> Option<Gender> {
    match g.as_deref() {
        Some("male" | "男") => Some(Gender::Male),
        Some("female" | "女") => Some(Gender::Female),
        _ => None,
    }
}

/// DTO → 用例层入参。
fn birth(req: &ChartRequest) -> Birth {
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
fn engine_query(req: &ChartRequest) -> Query {
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

fn validate(req: &ChartRequest) -> Result<(), String> {
    birth(req).validate()
}

async fn bazi_handler(Json(req): Json<ChartRequest>) -> impl IntoResponse {
    if let Err(e) = validate(&req) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response();
    }
    Json(mingli_app::bazi::natal(&birth(&req))).into_response()
}

/// 岁运叠加旺衰：本命 + extras（大运柱、流年柱等）。
/// extras 干支以字符串「癸未」形式传入，后端解析。
#[derive(Debug, Deserialize)]
struct OverlayRequest {
    #[serde(flatten)]
    natal: ChartRequest,
    /// 叠加干支字符串列表（顺序无关）。如 `["丁酉","丙午"]` = 大运丁酉柱 + 流年丙午柱。
    extras: Vec<String>,
}

async fn overlay_strength_handler(Json(req): Json<OverlayRequest>) -> impl IntoResponse {
    if let Err(e) = validate(&req.natal) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response();
    }
    match mingli_app::bazi::overlay_strength(&birth(&req.natal), &req.extras) {
        Ok(v) => Json(v).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response(),
    }
}

/// 团队合盘：N 人生辰 → N 张本命盘 + 团队五行画像 + 互补矩阵。
#[derive(Debug, Deserialize)]
struct TeamRequest {
    members: Vec<TeamMember>,
}

#[derive(Debug, Deserialize)]
struct TeamMember {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    #[serde(default)]
    minute: u32,
    tz: Option<f64>,
    gender: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

/// DTO 成员 → 用例层成员。
fn team_members(req: &TeamRequest) -> Vec<mingli_app::team::Member<'_>> {
    req.members
        .iter()
        .map(|m| mingli_app::team::Member {
            birth: Birth {
                year: m.year,
                month: m.month,
                day: m.day,
                hour: m.hour,
                minute: m.minute,
                tz: m.tz.unwrap_or(8.0),
                gender: parse_gender(&m.gender),
                true_solar_time: false,
                longitude: None,
            },
            name: m.name.as_deref(),
        })
        .collect()
}

async fn team_handler(Json(req): Json<TeamRequest>) -> impl IntoResponse {
    match mingli_app::team::compute(&team_members(&req)) {
        Ok(r) => Json(r.to_json()).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response(),
    }
}

/// 团队 LLM 释义：接受 `/api/team` 同形 body → 算团队结果 → 让 LLM 解读结构。
/// 与 `/api/team` 分开，因 LLM 是阻塞慢 I/O，不应污染纯计算端点。
async fn team_interpret_handler(Json(req): Json<TeamRequest>) -> impl IntoResponse {
    let team_json = match mingli_app::team::compute(&team_members(&req)) {
        Ok(r) => serde_json::to_string(&r.to_summary_json()).unwrap_or_default(),
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response()
        }
    };
    // 释义后端是阻塞慢 I/O → 移出异步执行器。
    let result = tokio::task::spawn_blocking(move || mingli_app::interpret::team(&ClaudeCli, &team_json)).await;
    match result {
        Ok(Ok(interp)) => Json(interp).into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "释义后端不可用" })),
        )
            .into_response(),
    }
}

async fn ziwei_handler(Json(req): Json<ChartRequest>) -> impl IntoResponse {
    if let Err(e) = validate(&req) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response();
    }
    Json(mingli_app::ziwei::natal(&birth(&req))).into_response()
}

/// 全叶并行排盘：一次输入 → engine 共享层算一次 → 并行 fan-out 所有叶 → 带元数据 JSON 数组。
async fn cast_handler(Json(req): Json<ChartRequest>) -> impl IntoResponse {
    if let Err(e) = validate(&req) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response();
    }
    let leaves = mingli_engine::cast_all_detailed(leaves(), &engine_query(&req));
    Json(serde_json::json!({ "leaves": leaves })).into_response()
}

/// D 族字/词模态请求：gematria/abjad 吃文字；wuge 吃笔画（康熙笔画表不内置，由调用方给）。
#[derive(Debug, Deserialize)]
struct WordRequest {
    /// "gematria" / "abjad" / "wuge"。
    system: String,
    /// gematria（希伯来）/abjad（阿拉伯） 的词。
    text: Option<String>,
    /// wuge：姓各字笔画。
    surname: Option<Vec<u32>>,
    /// wuge：名各字笔画。
    given: Option<Vec<u32>>,
}

/// D 族字/词模态入口（与 moment-based 排盘并列；这些术数不吃出生时刻）。
async fn word_handler(Json(req): Json<WordRequest>) -> impl IntoResponse {
    let q = WordQuery { text: req.text, surname: req.surname, given: req.given };
    match mingli_app::word::compute(word_leaves(), &req.system, &q) {
        Ok(v) => Json(v).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response(),
    }
}

/// LLM 释义（INT，与算分离）：算出该叶盘面 → 组装带护栏提示词 → claude CLI 释义；失败回退离线模板。
async fn interpret_handler(Json(req): Json<ChartRequest>) -> impl IntoResponse {
    if let Err(e) = validate(&req) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response();
    }
    let leaf_id = req.leaf.clone().unwrap_or_else(|| "bazi".to_string());
    let subject = req
        .subject
        .as_deref()
        .and_then(mingli_interpret::Subject::from_str_opt)
        .unwrap_or(mingli_interpret::Subject::Person);
    let q = engine_query(&req);
    // 释义后端是阻塞慢 I/O → 移出异步执行器；后端失败会回退离线模板（诚实标 backend）。
    let result =
        tokio::task::spawn_blocking(move || mingli_app::interpret::leaf(leaves(), &ClaudeCli, &leaf_id, &q, subject))
            .await;
    match result {
        Ok(Ok(interp)) => Json(interp).into_response(),
        Ok(Err(e)) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "释义后端不可用" })),
        )
            .into_response(),
    }
}

/// 跨叶相关性分析（信息论 NMI 矩阵）。网格固定→结果确定，首次算后缓存。
async fn analysis_handler() -> impl IntoResponse {
    Json(mingli_app::analysis::cross_leaf_cached(leaves()))
}

/// Fortune：t 时刻运势切片 + 100 年用神供给时间序列。
/// body = {natal: ChartRequest, t_target: AskTime(y/m/d/h/min/tz), timeline_max_age?: u32}
#[derive(Debug, Deserialize)]
struct FortuneRequest {
    /// 本命输入（全量 ChartRequest：出生 y/m/d/h/min/tz/性别等）。
    natal: ChartRequest,
    /// 目标时刻(year/month/day/hour/minute/tz)。
    t_target: TTime,
    /// 时间序列扫描上限年龄。默认 100。
    #[serde(default)]
    timeline_max_age: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TTime {
    year: i32,
    month: u32,
    day: u32,
    #[serde(default)]
    hour: u32,
    #[serde(default)]
    minute: u32,
    #[serde(default = "default_tz")]
    tz: f64,
}

fn default_tz() -> f64 { 8.0 }

async fn fortune_handler(Json(req): Json<FortuneRequest>) -> impl IntoResponse {
    if let Err(e) = validate(&req.natal) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response();
    }
    let t_target = mingli_contract::AskTime {
        year: req.t_target.year,
        month: req.t_target.month,
        day: req.t_target.day,
        hour: req.t_target.hour,
        minute: req.t_target.minute,
        tz: req.t_target.tz,
    };
    match mingli_app::bazi::fortune(&birth(&req.natal), &t_target, req.timeline_max_age) {
        Ok(v) => Json(v).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response(),
    }
}

/// 返回 8 类问事意图清单 + 当前注册叶集合（供 web 顶层「先选你要问什么」UI）。
async fn intents_handler() -> impl IntoResponse {
    let intents: Vec<_> = mingli_contract::intents()
        .iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "name_zh": s.name_zh,
                "atoms": s.atoms,
                "default_leaves": s.default_leaves,
                "output_shape": s.output_shape,
                "status": s.status,
                "status_label": s.status.label(),
                "note": s.note,
            })
        })
        .collect();
    let registered: Vec<_> = leaves()
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id(),
                "name": e.name(),
                "family": e.family(),
                "family_label": e.family().label(),
            })
        })
        .collect();
    Json(serde_json::json!({
        "intents": intents,
        "registered_leaves": registered,
    }))
}

/// 对给定 QueryKind 返回路由叶 id 列表（过滤当前 registry 实际启用）。
/// 请求体即 [`mingli_engine::QueryKind`] 的 JSON（内部标签 `{"kind":"natal", ...}`）。
async fn route_handler(Json(kind): Json<mingli_contract::QueryKind>) -> impl IntoResponse {
    let leaves = mingli_engine::route(leaves(), &kind);
    Json(serde_json::json!({
        "intent": kind.id(),
        "leaves": leaves,
    }))
}

async fn health() -> impl IntoResponse {
    // 列出已注册叶（id/显示名/家族），便于前端发现可用叶。
    let leaf_meta: Vec<_> = leaves()
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id(),
                "name": e.name(),
                "family": e.family(),
                "family_label": e.family().label(),
            })
        })
        .collect();
    Json(serde_json::json!({
        "status": "ok",
        "service": "mingli-api",
        "leaf_count": leaf_meta.len(),
        "leaves": leaf_meta,
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_target(false).init();

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/bazi", post(bazi_handler))
        .route("/api/bazi/overlay-strength", post(overlay_strength_handler))
        .route("/api/team", post(team_handler))
        .route("/api/team/interpret", post(team_interpret_handler))
        .route("/api/ziwei", post(ziwei_handler))
        .route("/api/cast", post(cast_handler))
        .route("/api/analysis", get(analysis_handler))
        .route("/api/interpret", post(interpret_handler))
        .route("/api/word", post(word_handler))
        .route("/api/intents", get(intents_handler))
        .route("/api/route", post(route_handler))
        .route("/api/fortune", post(fortune_handler))
        .layer(CorsLayer::permissive());

    // 端口由 port-registry 分配（lab32-mingli → 6027）；可用 MINGLI_API_BIND 覆盖。
    let addr = std::env::var("MINGLI_API_BIND").unwrap_or_else(|_| "127.0.0.1:6027".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("mingli-api listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
