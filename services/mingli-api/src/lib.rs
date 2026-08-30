//! mingli-api —— 承接层（axum）。把排盘引擎暴露为 HTTP JSON 端点。
//!
//! 这一层只做三件事：把线上的形状变成领域的形状（[`dto`]）、调用例、把结果与错误
//! 映射回线上（[`error`]）。业务编排在 `mingli-app`，算法在各叶——这里一行都不该有。
//!
//! 端点按意图分在 [`routes`] 的各模块里，装配在 [`router`]；释义后端与它那段
//! 「阻塞慢 I/O + 失败回退离线模板」的机制在 [`backend`]。
//!
//! 请求体的公共形状（本命一路各端点共用）：
//!
//! ```json
//! { "year":1990, "month":6, "day":15, "hour":14, "minute":30, "tz":8.0, "gender":"male",
//!   "latitude":31.23, "longitude":121.47, "seed":2024, "name":"Ada Lovelace" }
//! ```
//!
//! latitude/longitude（占星 Asc/MC）、seed（起卦可复现）、name（数字学）均可选。

use axum::routing::{get, post};
use axum::Router;
use mingli_contract::{CastingEngine, WordEngine};
use std::sync::OnceLock;
use tower_http::cors::CorsLayer;

pub mod backend;
pub mod dto;
pub mod error;
pub mod routes;

/// 装配根只装配一次，全进程复用（原先每个请求都重建一遍注册表）。
pub(crate) fn leaves() -> &'static [Box<dyn CastingEngine>] {
    static REG: OnceLock<Vec<Box<dyn CastingEngine>>> = OnceLock::new();
    REG.get_or_init(mingli_registry::registry)
}

/// 字词叶注册表，同样只装配一次。
pub(crate) fn word_leaves() -> &'static [Box<dyn WordEngine>] {
    static REG: OnceLock<Vec<Box<dyn WordEngine>>> = OnceLock::new();
    REG.get_or_init(mingli_registry::word_registry)
}

/// 组装全部路由，用 claude CLI 做主释义后端。
///
/// 单独拿出来是为了让测试能**在进程内**打端点——承接层最要紧的性质是
/// 「它没有在用例层之外多做事」，那得把两侧摆在一起比才看得见，
/// 而不是起一个真服务再 curl。见 `tests/no_drift.rs`。
pub fn router() -> Router {
    router_with(backend::Interpret::Cli)
}

/// 组装全部路由，并指定主释义后端。
///
/// 「找谁来说」是交付层的选择，所以由这里注入而不是写死在 handler 里。测试传
/// [`backend::Interpret::Offline`]，测的才是这条路本身。
pub fn router_with(interpret: backend::Interpret) -> Router {
    Router::new()
        .route("/api/health", get(routes::meta::health))
        .route("/api/intents", get(routes::meta::intents_handler))
        .route("/api/route", post(routes::meta::route_handler))
        .route("/api/analysis", get(routes::meta::analysis_handler))
        .route("/api/bazi", post(routes::natal::bazi_handler))
        .route("/api/bazi/overlay-strength", post(routes::natal::overlay_strength_handler))
        .route("/api/ziwei", post(routes::natal::ziwei_handler))
        .route("/api/cast", post(routes::natal::cast_handler))
        .route("/api/fortune", post(routes::natal::fortune_handler))
        .route("/api/interpret", post(routes::natal::interpret_handler))
        .route("/api/team", post(routes::team::team_handler))
        .route("/api/team/interpret", post(routes::team::team_interpret_handler))
        .route("/api/word", post(routes::word::word_handler))
        .route("/api/event", post(routes::event::handler))
        .route("/api/event/interpret", post(routes::event::interpret_handler))
        .route("/api/election", post(routes::election::handler))
        .route("/api/election/interpret", post(routes::election::interpret_handler))
        .route("/api/locative", post(routes::locative::handler))
        .route("/api/locative/interpret", post(routes::locative::interpret_handler))
        .route("/api/synastry", post(routes::synastry::handler))
        .route("/api/synastry/interpret", post(routes::synastry::interpret_handler))
        .route("/api/mundane", post(routes::mundane::handler))
        .route("/api/mundane/interpret", post(routes::mundane::interpret_handler))
        .layer(CorsLayer::permissive())
        .with_state(interpret)
}
