use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn identity_is_compiled_and_every_calculation_response_is_stamped() {
    let app = mingli_api::router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/build")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let header = response.headers()["x-mingli-build-id"]
        .to_str()
        .unwrap()
        .to_owned();
    let value: serde_json::Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["build_id"], header);
    assert_eq!(value["source_sha256"], env!("MINGLI_SOURCE_SHA256"));
    assert_eq!(header, env!("MINGLI_BUILD_ID"));
    for path in ["/api/bazi", "/api/bazi/report", "/api/bazi/report/utc", "/api/ziwei"] {
        let response=app.clone().oneshot(Request::builder().method("POST").uri(path).header("content-type","application/json").body(Body::from(r#"{"year":1990,"month":6,"day":15,"hour":14,"minute":30,"tz":8,"gender":"male"}"#)).unwrap()).await.unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers()["x-mingli-build-id"], header);
    }
}
