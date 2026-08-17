//! 合盘：两人本命 → 互供两数 + 团队结构，以及释义。
//!
//! 与另外四个意图的差别只在多一步「先把两人的 body 各转成 Birth」，
//! 机制仍走 [`crate::backend`]。

use crate::backend::{cast_then_interpret, Interpret};
use crate::dto::{member_birth, SynastryRequest};
use crate::error::bad_request;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// 合盘：算两人的互供度。
pub(crate) async fn handler(Json(req): Json<SynastryRequest>) -> Response {
    let (a, b) = (member_birth(&req.a), member_birth(&req.b));
    match mingli_app::synastry::compute((&a, req.a.name.as_deref()), (&b, req.b.name.as_deref())) {
        Ok(s) => Json(s.to_json()).into_response(),
        Err(e) => bad_request(e),
    }
}

/// 合盘释义：算完交释义后端出「配」。
pub(crate) async fn interpret_handler(State(backend): State<Interpret>, Json(req): Json<SynastryRequest>) -> Response {
    let (a, b) = (member_birth(&req.a), member_birth(&req.b));
    let names = (req.a.name.clone(), req.b.name.clone());
    cast_then_interpret(
        move || {
            mingli_app::synastry::compute((&a, names.0.as_deref()), (&b, names.1.as_deref()))
                .map(|s| s.to_json())
        },
        move |json| {
            mingli_interpret::interpret_synastry(&backend, &json)
                .or_else(|_| mingli_interpret::interpret_synastry(&mingli_interpret::Template, &json))
        },
    )
    .await
}
