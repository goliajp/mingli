//! 字词一路：gematria / abjad / 五格。与时刻无关，走第二条契约。

use crate::dto::WordRequest;
use crate::error::bad_request;
use crate::word_leaves;
use axum::response::{IntoResponse, Response};
use axum::Json;
use mingli_contract::WordQuery;

/// D 族字/词模态入口（与 moment-based 排盘并列；这些术数不吃出生时刻）。
pub(crate) async fn word_handler(Json(req): Json<WordRequest>) -> Response {
    let q = WordQuery { text: req.text, surname: req.surname, given: req.given };
    match mingli_app::word::compute(word_leaves(), &req.system, &q) {
        Ok(v) => Json(v).into_response(),
        Err(e) => bad_request(e),
    }
}
