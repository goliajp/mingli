//! 占事：起盘与释义。
//!
//! 两个 handler 都只声明自己不同的那一点——调哪个用例、交哪个释义函数；
//! 「阻塞慢 I/O + 失败回退离线模板」的机制在 [`crate::backend`]。

use crate::backend::{cast_then_interpret, Interpret};
use crate::dto::{ask_time, EventRequest};
use crate::error::bad_request;
use crate::leaves;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// 占事：算盘。
pub(crate) async fn handler(Json(req): Json<EventRequest>) -> Response {
    match mingli_app::event::cast(leaves(), &ask_time(&req.t_ask), req.seed, req.question) {
        Ok(v) => Json(v).into_response(),
        Err(e) => bad_request(e),
    }
}

/// 占事：算盘后交释义。
pub(crate) async fn interpret_handler(State(backend): State<Interpret>, Json(req): Json<EventRequest>) -> Response {
    cast_then_interpret(
        || mingli_app::event::cast(leaves(), &ask_time(&req.t_ask), req.seed, req.question),
        move |json| {
            mingli_interpret::interpret_event(&backend, &json)
                .or_else(|_| mingli_interpret::interpret_event(&mingli_interpret::Template, &json))
        },
    )
    .await
}
