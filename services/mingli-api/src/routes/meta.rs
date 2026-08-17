//! 元数据：健康检查、意图清单、路由查询、跨叶相关性。
//!
//! 这几个端点的内联 `json!` 是**协议成形**（拼响应外壳），不是加工结果——
//! 壳里的内容一律原样转述注册表与契约，`tests/no_drift.rs` 盯着这一点。

use crate::leaves;
use axum::response::IntoResponse;
use axum::Json;

/// 跨叶相关性分析（信息论 NMI 矩阵）。网格固定→结果确定，首次算后缓存。
pub(crate) async fn analysis_handler() -> impl IntoResponse {
    Json(mingli_app::analysis::cross_leaf_cached(leaves()))
}

/// 返回 8 类问事意图清单 + 当前注册叶集合（供 web 顶层「先选你要问什么」UI）。
pub(crate) async fn intents_handler() -> impl IntoResponse {
    // 「这一类问局是什么」在端口层，「谁来答」要问注册表里的叶——两者在编排层合成。
    let intents: Vec<_> = mingli_engine::intent_catalog(leaves())
        .into_iter()
        .map(|(s, default_leaves)| {
            serde_json::json!({
                "id": s.id,
                "name_zh": s.name_zh,
                "atoms": s.atoms,
                "default_leaves": default_leaves,
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
/// 请求体即 [`mingli_contract::QueryKind`] 的 JSON（内部标签 `{"kind":"natal", ...}`）。
pub(crate) async fn route_handler(Json(kind): Json<mingli_contract::QueryKind>) -> impl IntoResponse {
    let leaves = mingli_engine::route(leaves(), &kind);
    Json(serde_json::json!({
        "intent": kind.id(),
        "leaves": leaves,
    }))
}

/// 健康检查：顺带列出已注册叶（id / 显示名 / 家族），便于前端发现可用叶。
pub(crate) async fn health() -> impl IntoResponse {
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
