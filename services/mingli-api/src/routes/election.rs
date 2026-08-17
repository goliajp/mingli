//! 择吉：起盘与释义。
//!
//! 两个 handler 都只声明自己不同的那一点——调哪个用例、交哪个释义函数；
//! 「阻塞慢 I/O + 失败回退离线模板」的机制在 [`crate::backend`]。

use crate::backend::{cast_then_interpret, Interpret};
use crate::dto::{ask_time, ElectionRequest};
use crate::error::bad_request;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// 择吉：算盘。
pub(crate) async fn handler(Json(req): Json<ElectionRequest>) -> Response {
    match mingli_app::election::scan(&ask_time(&req.window_start), &ask_time(&req.window_end), req.category) {
        Ok(v) => Json(v).into_response(),
        Err(e) => bad_request(e),
    }
}

/// 择吉：算盘后交释义。
pub(crate) async fn interpret_handler(State(backend): State<Interpret>, Json(req): Json<ElectionRequest>) -> Response {
    cast_then_interpret(
        || mingli_app::election::scan(&ask_time(&req.window_start), &ask_time(&req.window_end), req.category),
        move |json| {
            mingli_interpret::interpret_election(&backend, &json)
                .or_else(|_| mingli_interpret::interpret_election(&mingli_interpret::Template, &json))
        },
    )
    .await
}
