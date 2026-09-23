//! 年历端点与用例层逐字节一致；不把结构测试当成历法正确性证明。
use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use tower::ServiceExt;
#[tokio::test]
async fn chinese_year_has_no_delivery_drift_and_a_compiled_identity() {
    let app = mingli_api::router();
    for year in [1900, 2023, 2025, 2026, 2033, 2034, 2099] {
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/calendar/chinese-year?year={year}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), 200, "year {year}");
        assert_eq!(r.headers()["x-mingli-build-id"], env!("MINGLI_BUILD_ID"));
        let bytes = r.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            bytes.as_ref(),
            serde_json::to_vec(&mingli_app::calendar::chinese_year(year).unwrap()).unwrap()
        );
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["schema_version"], 1);
        assert_eq!(v["calendar"], "chinese_lunisolar");
        assert_eq!(v["time_zone_offset_minutes"], 480);
    }
}
#[tokio::test]
async fn calendar_refuses_partial_boundary_years_and_extra_parameters() {
    let app = mingli_api::router();
    for query in [
        "year=1899",
        "year=2100",
        "year=0",
        "year=oops",
        "",
        "year=2026&tz=9",
        "year=2026&month=1",
    ] {
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/calendar/chinese-year?{query}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), 400, "{query}");
    }
}
