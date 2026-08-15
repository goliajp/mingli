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
use serde::Deserialize;
use tower_http::cors::CorsLayer;

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

fn parse_gender_bazi(g: &Option<String>) -> Option<mingli_bazi::Gender> {
    match g.as_deref() {
        Some("male") | Some("男") => Some(mingli_bazi::Gender::Male),
        Some("female") | Some("女") => Some(mingli_bazi::Gender::Female),
        _ => None,
    }
}

fn parse_gender_ziwei(g: &Option<String>) -> Option<mingli_ziwei::Gender> {
    match g.as_deref() {
        Some("male") | Some("男") => Some(mingli_ziwei::Gender::Male),
        Some("female") | Some("女") => Some(mingli_ziwei::Gender::Female),
        _ => None,
    }
}

fn parse_gender_engine(g: &Option<String>) -> Option<mingli_engine::Gender> {
    match g.as_deref() {
        Some("male") | Some("男") => Some(mingli_engine::Gender::Male),
        Some("female") | Some("女") => Some(mingli_engine::Gender::Female),
        _ => None,
    }
}

/// 由请求构造 engine 查询（共享输入）。
fn engine_query(req: &ChartRequest) -> mingli_engine::Query {
    mingli_engine::Query {
        year: req.year,
        month: req.month,
        day: req.day,
        hour: req.hour,
        minute: req.minute,
        tz: req.tz.unwrap_or(8.0),
        gender: parse_gender_engine(&req.gender),
        latitude: req.latitude,
        longitude: req.longitude,
        seed: req.seed,
        name: req.name.clone(),
        schools: req.schools.clone().unwrap_or_default(),
    }
}

fn validate(req: &ChartRequest) -> Result<(), String> {
    if !(1900..=2100).contains(&req.year) {
        return Err("year 仅支持 1900–2100".into());
    }
    if !(1..=12).contains(&req.month) {
        return Err("month 须 1–12".into());
    }
    if !(1..=31).contains(&req.day) {
        return Err("day 须 1–31".into());
    }
    if req.hour > 23 || req.minute > 59 {
        return Err("hour/minute 越界".into());
    }
    Ok(())
}

async fn bazi_handler(Json(req): Json<ChartRequest>) -> impl IntoResponse {
    if let Err(e) = validate(&req) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response();
    }
    let input = mingli_bazi::BirthInput {
        year: req.year,
        month: req.month,
        day: req.day,
        hour: req.hour,
        minute: req.minute,
        tz: req.tz.unwrap_or(8.0),
        gender: parse_gender_bazi(&req.gender),
    };
    let chart = match (req.true_solar_time, req.longitude) {
        (true, Some(lon)) => mingli_bazi::compute_with_true_solar(input, lon),
        _ => mingli_bazi::compute(input),
    };
    Json(chart).into_response()
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
    let input = mingli_bazi::BirthInput {
        year: req.natal.year,
        month: req.natal.month,
        day: req.natal.day,
        hour: req.natal.hour,
        minute: req.natal.minute,
        tz: req.natal.tz.unwrap_or(8.0),
        gender: parse_gender_bazi(&req.natal.gender),
    };
    let chart = match (req.natal.true_solar_time, req.natal.longitude) {
        (true, Some(lon)) => mingli_bazi::compute_with_true_solar(input, lon),
        _ => mingli_bazi::compute(input),
    };
    let parsed: Vec<_> = req
        .extras
        .iter()
        .filter_map(|s| mingli_bazi::parse_ganzhi(s))
        .collect();
    if parsed.len() != req.extras.len() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "extras 含无法解析的干支字符串" })),
        )
            .into_response();
    }
    let year_gz = mingli_bazi::parse_ganzhi(&chart.year.ganzhi).unwrap();
    let month_gz = mingli_bazi::parse_ganzhi(&chart.month.ganzhi).unwrap();
    let day_gz = mingli_bazi::parse_ganzhi(&chart.day.ganzhi).unwrap();
    let hour_gz = mingli_bazi::parse_ganzhi(&chart.hour.ganzhi).unwrap();
    let yun = mingli_bazi::compute_strength_with_extras(year_gz, month_gz, day_gz, hour_gz, &parsed);
    let delta = i32::try_from(yun.score).unwrap_or(0) - i32::try_from(chart.strength.score).unwrap_or(0);
    Json(serde_json::json!({
        "ming": chart.strength,
        "yun": yun,
        "delta_score": delta,
        "extras": req.extras,
    }))
    .into_response()
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

async fn team_handler(Json(req): Json<TeamRequest>) -> impl IntoResponse {
    if req.members.is_empty() || req.members.len() > 12 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "members 须 1-12 人" })),
        )
            .into_response();
    }
    // 排每张盘
    let charts: Vec<_> = req
        .members
        .iter()
        .map(|m| {
            mingli_bazi::compute(mingli_bazi::BirthInput {
                year: m.year, month: m.month, day: m.day, hour: m.hour, minute: m.minute,
                tz: m.tz.unwrap_or(8.0),
                gender: parse_gender_bazi(&m.gender),
            })
        })
        .collect();
    let team_wx = mingli_bazi::team_wuxing_average(&charts);
    let weakest = mingli_bazi::team_weakest(&team_wx);
    let strongest = mingli_bazi::team_strongest(&team_wx);
    // 互补矩阵 N×N：M[i][j] = j 对 i 用神（主）的供给度 = j.wuxing[i.yongshen.primary_wuxing]
    let n = charts.len();
    let mut matrix: Vec<Vec<u32>> = vec![vec![0; n]; n];
    for i in 0..n {
        for j in 0..n {
            matrix[i][j] = mingli_bazi::complement_score(
                &charts[i].yongshen.primary_wuxing,
                &charts[j].strength.wuxing,
            );
        }
    }
    let members_out: Vec<serde_json::Value> = req
        .members
        .iter()
        .zip(charts.iter())
        .map(|(m, c)| serde_json::json!({
            "name": m.name.clone().unwrap_or_else(|| format!("成员 {}", m.year)),
            "day_master": c.day_master,
            "day_master_wuxing": c.day_master_wuxing,
            "year_gz": c.year.ganzhi,
            "month_gz": c.month.ganzhi,
            "day_gz": c.day.ganzhi,
            "hour_gz": c.hour.ganzhi,
            "strength": c.strength,
            "yongshen": c.yongshen,
        }))
        .collect();
    Json(serde_json::json!({
        "members": members_out,
        "team_wuxing": team_wx,
        "team_weakest": { "wuxing": weakest.0, "pct": weakest.1 },
        "team_strongest": { "wuxing": strongest.0, "pct": strongest.1 },
        "complement_matrix": matrix,
    }))
    .into_response()
}

/// 团队 LLM 释义：接受 `/api/team` 同形 body → 算团队结果 → 让 LLM 解读结构。
/// 与 `/api/team` 分开，因 LLM 是阻塞慢 I/O，不应污染纯计算端点。
async fn team_interpret_handler(Json(req): Json<TeamRequest>) -> impl IntoResponse {
    if req.members.is_empty() || req.members.len() > 12 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "members 须 1-12 人" })),
        )
            .into_response();
    }
    let charts: Vec<_> = req
        .members
        .iter()
        .map(|m| {
            mingli_bazi::compute(mingli_bazi::BirthInput {
                year: m.year, month: m.month, day: m.day, hour: m.hour, minute: m.minute,
                tz: m.tz.unwrap_or(8.0),
                gender: parse_gender_bazi(&m.gender),
            })
        })
        .collect();
    let team_wx = mingli_bazi::team_wuxing_average(&charts);
    let weakest = mingli_bazi::team_weakest(&team_wx);
    let strongest = mingli_bazi::team_strongest(&team_wx);
    let n = charts.len();
    let mut matrix: Vec<Vec<u32>> = vec![vec![0; n]; n];
    for i in 0..n {
        for j in 0..n {
            matrix[i][j] = mingli_bazi::complement_score(
                &charts[i].yongshen.primary_wuxing,
                &charts[j].strength.wuxing,
            );
        }
    }
    let members_out: Vec<serde_json::Value> = req
        .members
        .iter()
        .zip(charts.iter())
        .map(|(m, c)| serde_json::json!({
            "name": m.name.clone().unwrap_or_else(|| format!("成员 {}", m.year)),
            "day_master": c.day_master,
            "day_master_wuxing": c.day_master_wuxing,
            "strength": c.strength,
            "yongshen": c.yongshen,
        }))
        .collect();
    let team_json = serde_json::to_string(&serde_json::json!({
        "members": members_out,
        "team_wuxing": team_wx,
        "team_weakest": { "wuxing": weakest.0, "pct": weakest.1 },
        "team_strongest": { "wuxing": strongest.0, "pct": strongest.1 },
        "complement_matrix": matrix,
    })).unwrap_or_default();
    let result = tokio::task::spawn_blocking(move || {
        mingli_interpret::interpret_team(&ClaudeCli, &team_json)
            .or_else(|_| mingli_interpret::interpret_team(&mingli_interpret::Template, &team_json))
    })
    .await;
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
    let chart = mingli_ziwei::compute(mingli_ziwei::BirthInput {
        year: req.year,
        month: req.month,
        day: req.day,
        hour: req.hour,
        minute: req.minute,
        tz: req.tz.unwrap_or(8.0),
        gender: parse_gender_ziwei(&req.gender),
    });
    Json(chart).into_response()
}

/// 全叶并行排盘：一次输入 → engine 共享层算一次 → 并行 fan-out 所有叶 → 带元数据 JSON 数组。
async fn cast_handler(Json(req): Json<ChartRequest>) -> impl IntoResponse {
    if let Err(e) = validate(&req) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response();
    }
    let leaves = mingli_engine::cast_all_detailed(&engine_query(&req));
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
    let bad = |m: &str| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": m }))).into_response();
    match req.system.as_str() {
        "gematria" => {
            let w = req.text.unwrap_or_default();
            Json(serde_json::json!({ "system": "gematria", "input": w, "result": mingli_gematria::compute(&w) })).into_response()
        }
        "abjad" => {
            let w = req.text.unwrap_or_default();
            Json(serde_json::json!({ "system": "abjad", "input": w, "result": mingli_abjad::compute(&w) })).into_response()
        }
        "wuge" => {
            let s = req.surname.unwrap_or_default();
            let g = req.given.unwrap_or_default();
            if s.is_empty() || g.is_empty() {
                return bad("姓与名笔画至少各一字");
            }
            Json(serde_json::json!({ "system": "wuge", "surname": s, "given": g, "result": mingli_wuge::five_grids(&s, &g) })).into_response()
        }
        other => bad(&format!("未知字词系统 {other}")),
    }
}

/// LLM 释义（INT，与算分离）：算出该叶盘面 → 组装带护栏提示词 → claude CLI 释义；失败回退离线模板。
async fn interpret_handler(Json(req): Json<ChartRequest>) -> impl IntoResponse {
    if let Err(e) = validate(&req) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response();
    }
    let leaf_id = req.leaf.clone().unwrap_or_else(|| "bazi".to_string());
    // 只算该叶（省去其余 18 叶，非占星叶还省掉 VSOP87）。
    let Some(leaf) = mingli_engine::cast_one(&leaf_id, &engine_query(&req)) else {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": format!("未知叶 {leaf_id}") }))).into_response();
    };
    let subject = req.subject.as_deref()
        .and_then(mingli_interpret::Subject::from_str_opt)
        .unwrap_or(mingli_interpret::Subject::Person);
    // claude 调用阻塞且慢 → 移出异步执行器；失败回退离线模板（诚实标 backend）。
    let result = tokio::task::spawn_blocking(move || {
        mingli_interpret::interpret_leaf_with_subject(&ClaudeCli, &leaf, subject)
            .or_else(|_| mingli_interpret::interpret_leaf_with_subject(&mingli_interpret::Template, &leaf, subject))
    })
    .await;
    match result {
        Ok(Ok(interp)) => Json(interp).into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "释义后端不可用" })),
        )
            .into_response(),
    }
}

/// 跨叶相关性分析（信息论 NMI 矩阵）。网格固定→结果确定，首次算后缓存。
async fn analysis_handler() -> impl IntoResponse {
    use std::sync::OnceLock;
    static CACHE: OnceLock<serde_json::Value> = OnceLock::new();
    let v = CACHE.get_or_init(|| {
        let a = mingli_analysis::cross_leaf(&mingli_analysis::sample_grid(1980, 2009));
        serde_json::to_value(a).unwrap_or(serde_json::Value::Null)
    });
    Json(v.clone())
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
    if req.natal.gender.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "fortune 需性别（决定大运顺逆），缺 gender" })),
        )
            .into_response();
    }
    let input = mingli_bazi::BirthInput {
        year: req.natal.year,
        month: req.natal.month,
        day: req.natal.day,
        hour: req.natal.hour,
        minute: req.natal.minute,
        tz: req.natal.tz.unwrap_or(8.0),
        gender: parse_gender_bazi(&req.natal.gender),
    };
    let max_age = req.timeline_max_age.unwrap_or(100).min(120);
    let at = mingli_bazi::fortune_at(
        input,
        req.t_target.year, req.t_target.month, req.t_target.day,
        req.t_target.hour, req.t_target.minute, req.t_target.tz,
    );
    let timeline = mingli_bazi::fortune_supply_timeline(input, max_age);
    Json(serde_json::json!({
        "at": at,
        "timeline": timeline,
        "max_age": max_age,
    }))
    .into_response()
}

/// 返回 8 类问事意图清单 + 当前注册叶集合（供 web 顶层「先选你要问什么」UI）。
async fn intents_handler() -> impl IntoResponse {
    let intents: Vec<_> = mingli_engine::intents()
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
    let registered: Vec<_> = mingli_engine::registry()
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
async fn route_handler(Json(kind): Json<mingli_engine::QueryKind>) -> impl IntoResponse {
    let leaves = mingli_engine::route(&kind);
    Json(serde_json::json!({
        "intent": kind.id(),
        "leaves": leaves,
    }))
}

async fn health() -> impl IntoResponse {
    // 列出已注册叶（id/显示名/家族），便于前端发现可用叶。
    let leaves: Vec<_> = mingli_engine::registry()
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
        "leaf_count": leaves.len(),
        "leaves": leaves,
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
