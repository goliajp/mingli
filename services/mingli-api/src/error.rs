//! 错误响应：全库只有两种形状，固化在这里。
//!
//! 审计时特意查过：23 个 handler 的错误响应本来就一致，都是 `{"error": "…"}`，
//! 状态码只用 400 与 500 两个。这里不是重新设计，只是把已经一致的东西收成一处，
//! 免得下一个 handler 顺手写出第三种。

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// 400：调用方给的东西不对（校验不过、时窗倒置、未知叶 id 等）。
pub fn bad_request(msg: impl Into<String>) -> Response {
    (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg.into() }))).into_response()
}

/// 500：我们这边的事（释义后端起不来、任务 panic）。
pub fn server_error(msg: impl Into<String>) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": msg.into() }))).into_response()
}
