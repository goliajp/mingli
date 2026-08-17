//! 国运：起盘与释义。
//!
//! 两个 handler 都只声明自己不同的那一点——调哪个用例、交哪个释义函数；
//! 「阻塞慢 I/O + 失败回退离线模板」的机制在 [`crate::backend`]。

use crate::backend::{cast_then_interpret, Interpret};
use crate::dto::{MundaneRequest};
use crate::error::bad_request;
use crate::leaves;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// 国运：算盘。
pub(crate) async fn handler(Json(req): Json<MundaneRequest>) -> Response {
    match mingli_app::mundane::cast(leaves(), &req.founded_at.ask_time(), req.latitude, req.longitude, req.target_year, req.span) {
        Ok(v) => Json(v).into_response(),
        Err(e) => bad_request(e),
    }
}

/// 国运：算盘后交释义。
pub(crate) async fn interpret_handler(State(backend): State<Interpret>, Json(req): Json<MundaneRequest>) -> Response {
    cast_then_interpret(
        || mingli_app::mundane::cast(leaves(), &req.founded_at.ask_time(), req.latitude, req.longitude, req.target_year, req.span),
        move |json| {
            mingli_interpret::interpret_mundane(&backend, &json)
                .or_else(|_| mingli_interpret::interpret_mundane(&mingli_interpret::Template, &json))
        },
    )
    .await
}
