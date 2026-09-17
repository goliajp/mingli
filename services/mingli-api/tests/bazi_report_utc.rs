use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;
async fn post(path: &str, input: &Value) -> (u16, Value) {
    let r = mingli_api::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(input.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.headers()["x-mingli-build-id"], env!("MINGLI_BUILD_ID"));
    let status = r.status().as_u16();
    (
        status,
        serde_json::from_slice(&r.into_body().collect().await.unwrap().to_bytes()).unwrap(),
    )
}
#[tokio::test]
async fn new_utc_report_preserves_existing_v1_and_has_separate_evidence() {
    let f: Value =
        serde_json::from_str(include_str!("fixtures/bazi-report-before-utc.json")).unwrap();
    let (s, v1) = post("/api/bazi/report", &f["input"]).await;
    assert_eq!(s, 200);
    assert_eq!(v1, f["response"]);
    let (s, v2) = post("/api/bazi/report/utc", &f["input"]).await;
    assert_eq!(s, 200);
    let b = &v2["cycle_basis"];
    assert_eq!(b["schema_version"], 2);
    assert_eq!(b["model_id"], "vsop87d-iau1980-utc-v2");
    assert!(b.get("birth_jd_ut").is_none());
    assert_eq!(b["interval_scale"], "tt_elapsed");
    assert_eq!(b["birth_time_scale"]["kind"], "utc_leap_table");
    assert_ne!(b["birth_jde_tt"], v1["cycle_basis"]["birth_jde_tt"]);
    if let Ok(path) = std::env::var("MINGLI_UTC_REPORT_DUMP") {
        std::fs::write(path,serde_json::to_string_pretty(&json!({"input":f["input"],"response":v2,"build":{"build_id":env!("MINGLI_BUILD_ID"),"source_sha256":env!("MINGLI_SOURCE_SHA256")}})).unwrap()).unwrap();
    }
}
#[tokio::test]
async fn coverage_of_required_roots_is_checked_without_clamping_or_panicking() {
    for (y, m, d, ok) in [
        (1900, 1, 1, true),
        (1960, 1, 1, true),
        (1972, 1, 1, true),
        (2017, 1, 1, true),
        (2026, 12, 31, true),
        (2027, 1, 1, true),
        (2027, 6, 1, true),
        (2027, 6, 20, false),
        (2027, 7, 1, false),
        (2100, 12, 31, false),
    ] {
        let i = json!({"year":y,"month":m,"day":d,"hour":0,"minute":0,"tz":14,"gender":"female","true_solar_time":false});
        let (s, r) = post("/api/bazi/report/utc", &i).await;
        assert_eq!(s, if ok { 200 } else { 400 }, "{y}-{m}-{d}: {r}");
        if !ok {
            assert!(r["error"].as_str().unwrap().contains("时间数据范围"));
        }
    }
    let f: Value =
        serde_json::from_str(include_str!("fixtures/bazi-report-before-utc.json")).unwrap();
    for (k, v) in [
        ("gender", Value::Null),
        ("true_solar_time", json!(true)),
        ("hour", json!(24)),
        ("tz", json!(15)),
    ] {
        let mut i = f["input"].clone();
        i[k] = v;
        assert_eq!(post("/api/bazi/report/utc", &i).await.0, 400);
    }
}

#[tokio::test]
async fn actual_api_design_cases_preserve_jie_and_rounding_minute_differences() {
    let inputs: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/utc-design-inputs.json")).unwrap();
    let mut cases = Vec::new();
    for case in inputs {
        let (s, response) = post("/api/bazi/report/utc", &case["input"]).await;
        assert_eq!(s, 200);
        cases.push(json!({"name":case["name"],"input":case["input"],"response":response,"build":{"build_id":env!("MINGLI_BUILD_ID"),"source_sha256":env!("MINGLI_SOURCE_SHA256")}}));
    }
    assert_eq!(cases[0]["response"]["chart"]["month"]["branch"], "辰");
    assert_eq!(cases[1]["response"]["chart"]["month"]["branch"], "巳");
    assert_eq!(
        cases[2]["response"]["chart"]["dayun"]["pillars"][0]["start_age"],
        2
    );
    assert_eq!(
        cases[3]["response"]["chart"]["dayun"]["pillars"][0]["start_age"],
        1
    );
    if let Ok(path) = std::env::var("MINGLI_UTC_DESIGN_SOURCE_DUMP") {
        let source:Vec<Value>=cases.iter().map(|c|json!({"id":c["name"],"input":c["input"],"response":c["response"],"build":c["build"]})).collect();
        std::fs::write(path, serde_json::to_string_pretty(&source).unwrap()).unwrap();
    }
    if let Ok(path) = std::env::var("MINGLI_UTC_DESIGN_API_DUMP") {
        std::fs::write(path, serde_json::to_string_pretty(&cases).unwrap()).unwrap();
    }
}
