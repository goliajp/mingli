//! 本命一路：四柱 / 紫微精盘、全叶排盘、岁运叠加、运势时序、单叶释义。

use crate::dto::{birth, engine_query, validate, ChartRequest, FortuneRequest, OverlayRequest};
use crate::backend::Interpret;
use crate::error::{bad_request, server_error};
use crate::leaves;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// 四柱精盘。
pub(crate) async fn bazi_handler(Json(req): Json<ChartRequest>) -> Response {
    if let Err(e) = validate(&req) {
        return bad_request(e);
    }
    Json(mingli_app::bazi::natal(&birth(&req))).into_response()
}

/// 紫微精盘。
pub(crate) async fn ziwei_handler(Json(req): Json<ChartRequest>) -> Response {
    if let Err(e) = validate(&req) {
        return bad_request(e);
    }
    Json(mingli_app::ziwei::natal(&birth(&req))).into_response()
}

/// 全叶并行排盘：一次输入 → engine 共享层算一次 → 并行 fan-out 所有叶 → 带元数据 JSON 数组。
pub(crate) async fn cast_handler(Json(req): Json<ChartRequest>) -> Response {
    if let Err(e) = validate(&req) {
        return bad_request(e);
    }
    let leaves = mingli_engine::cast_all_detailed(leaves(), &engine_query(&req));
    Json(serde_json::json!({ "leaves": leaves })).into_response()
}

/// 岁运叠加：本命 + 若干外来干支 → 重算旺衰。
pub(crate) async fn overlay_strength_handler(Json(req): Json<OverlayRequest>) -> Response {
    if let Err(e) = validate(&req.natal) {
        return bad_request(e);
    }
    match mingli_app::bazi::overlay_strength(&birth(&req.natal), &req.extras) {
        Ok(v) => Json(v).into_response(),
        Err(e) => bad_request(e),
    }
}

/// 运势时序：本命 + 目标时刻 → 当下切片与一生曲线。
pub(crate) async fn fortune_handler(Json(req): Json<FortuneRequest>) -> Response {
    if let Err(e) = req.natal.validate() {
        return bad_request(e);
    }
    match mingli_app::bazi::fortune(&req.natal, &req.t_target.ask_time(), req.timeline_max_age) {
        Ok(v) => Json(v).into_response(),
        Err(e) => bad_request(e),
    }
}

/// 单叶释义（INT，与算分离）：算出该叶盘面 → 组装带护栏提示词 → 交释义后端；失败回退离线模板。
///
/// 这个 handler 没有走 [`crate::backend::cast_then_interpret`]：那条路是「算不出来 400、
/// 释义不出来 500」，而这里算与释义在用例层是**一次调用**，它的 `Err` 说的是「没有这片叶」——
/// 那是调用方的问题，得是 400。差异是真的，不合并。
pub(crate) async fn interpret_handler(State(backend): State<Interpret>, Json(req): Json<ChartRequest>) -> Response {
    if let Err(e) = validate(&req) {
        return bad_request(e);
    }
    let leaf_id = req.leaf.clone().unwrap_or_else(|| "bazi".to_string());
    // 缺省是人盘；但**写了个认不出的值不能当成没写**——那正是「默默忽略」，
    // 而拼错的人拿到的是人盘的读法却以为看的是公司盘。性别那一处修过同样的毛病
    let subject = match req.subject.as_deref() {
        None => mingli_interpret::Subject::Person,
        Some(s) => match mingli_interpret::Subject::from_str_opt(s) {
            Some(v) => v,
            None => return bad_request(format!("subject 认不出「{s}」，须为 person / company / product / event（也收 人 / 公司 / 物 / 事）")),
        },
    };
    let q = engine_query(&req);
    // 释义后端是阻塞慢 I/O → 移出异步执行器；后端失败会回退离线模板（诚实标 backend）。
    let result =
        tokio::task::spawn_blocking(move || mingli_app::interpret::leaf(leaves(), &backend, &leaf_id, &q, subject))
            .await;
    match result {
        Ok(Ok(interp)) => Json(interp).into_response(),
        Ok(Err(e)) => bad_request(e),
        Err(_) => server_error("释义后端不可用"),
    }
}
