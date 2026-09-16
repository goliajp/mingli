//! 公共年历接口。请求没有人物或出生输入。
use crate::error::{bad_request, server_error};
use axum::{
    Json,
    extract::Query,
    response::{IntoResponse, Response},
};
use mingli_app::calendar::CalendarError;
use serde::Deserialize;

/// 完整农历年的查询参数。不接受可变时区或额外出生字段。
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChineseYearQuery {
    /// 农历年，一九〇〇至二〇九九。
    pub year: i32,
}
/// 读取固定东八区的完整中国农历年。
pub async fn chinese_year(Query(q): Query<ChineseYearQuery>) -> Response {
    match mingli_app::calendar::chinese_year(q.year) {
        Ok(year) => Json(year).into_response(),
        Err(e @ CalendarError::UnsupportedYear) => bad_request(e.to_string()),
        Err(e @ CalendarError::InconsistentSequence) => server_error(e.to_string()),
    }
}
