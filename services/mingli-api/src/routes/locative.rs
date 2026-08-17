//! 寻方位：起盘与释义。
//!
//! 两个 handler 都只声明自己不同的那一点——调哪个用例、交哪个释义函数；
//! 「阻塞慢 I/O + 失败回退离线模板」的机制在 [`crate::backend`]。

use crate::backend::{cast_then_interpret, Interpret};
use crate::dto::{ask_time, LocativeRequest};
use crate::error::bad_request;
use crate::leaves;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// 寻方位：算盘。
pub(crate) async fn handler(Json(req): Json<LocativeRequest>) -> Response {
    match mingli_app::locative::cast(leaves(), &ask_time(&req.t_ask), req.seed, req.category) {
        Ok(v) => Json(v).into_response(),
        Err(e) => bad_request(e),
    }
}

/// 寻方位：算盘后交释义。
pub(crate) async fn interpret_handler(State(backend): State<Interpret>, Json(req): Json<LocativeRequest>) -> Response {
    cast_then_interpret(
        || mingli_app::locative::cast(leaves(), &ask_time(&req.t_ask), req.seed, req.category),
        move |json| {
            mingli_interpret::interpret_locative(&backend, &json)
                .or_else(|_| mingli_interpret::interpret_locative(&mingli_interpret::Template, &json))
        },
    )
    .await
}
