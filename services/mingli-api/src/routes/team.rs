//! 团队合盘与其释义。

use crate::backend::{cast_then_interpret, Interpret};
use crate::dto::TeamRequest;
use crate::error::bad_request;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// 团队合盘：多人本命 → 互供矩阵与结构判读。
pub(crate) async fn team_handler(Json(req): Json<TeamRequest>) -> Response {
    match mingli_app::team::compute(&req.members()) {
        Ok(r) => Json(r.to_json()).into_response(),
        Err(e) => bad_request(e),
    }
}

/// 团队释义：接受 `/api/team` 同形 body → 算团队结果 → 让释义后端解读结构。
///
/// 与 `/api/team` 分开，因释义是阻塞慢 I/O，不该污染纯计算端点。
pub(crate) async fn team_interpret_handler(State(backend): State<Interpret>, Json(req): Json<TeamRequest>) -> Response {
    cast_then_interpret(
        || mingli_app::team::compute(&req.members()).map(|r| r.to_summary_json()),
        move |json| mingli_app::interpret::team(&backend, &json),
    )
    .await
}
