use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;
async fn post(path: &str, body: &Value) -> (u16, Value) {
    let response = mingli_api::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.headers()["x-mingli-build-id"],
        env!("MINGLI_BUILD_ID")
    );
    let status = response.status().as_u16();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
#[tokio::test]
async fn report_preserves_real_pre_extension_chart_and_old_endpoint_shape() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/bazi-before-report.json")).unwrap();
    let input = &fixture["input"];
    let (s, old) = post("/api/bazi", input).await;
    assert_eq!(s, 200);
    assert_eq!(old, fixture["raw"]);
    let (s, report) = post("/api/bazi/report", input).await;
    assert_eq!(s, 200);
    assert_eq!(report.as_object().unwrap().len(), 2);
    let mut high = report["chart"].clone();
    let mut legacy = old.clone();
    high.as_object_mut().unwrap().remove("dayun");
    legacy.as_object_mut().unwrap().remove("dayun");
    assert_eq!(high, legacy); // away from boundaries only the cycle ages change
                              // The high-order roots can round to the same displayed ages away from a boundary.
    assert!(old.get("cycle_basis").is_none());
    let b = &report["cycle_basis"];
    assert_eq!(b["schema_version"], 1);
    assert_eq!(b["model_id"], "vsop87d-iau1980-delta-t-v1");
    assert_eq!(b["selected"], "previous");
    assert_eq!(b["days_per_year"], 3);
    if let Ok(path) = std::env::var("MINGLI_VSOP_REPORT_EVIDENCE_DUMP") {
        std::fs::write(path,serde_json::to_string_pretty(&json!({"input":input,"response":report,
            "build":{"build_id":env!("MINGLI_BUILD_ID"),"source_sha256":env!("MINGLI_SOURCE_SHA256")}})).unwrap()).unwrap();
    }
}
#[tokio::test]
async fn report_requires_gender_clock_time_and_valid_birth() {
    let i = json!({"year":1990,"month":6,"day":15,"hour":14,"minute":30,"tz":8,"gender":"male","true_solar_time":false});
    for (k, v) in [
        ("gender", Value::Null),
        ("gender", json!("unknown")),
        ("true_solar_time", json!(true)),
        ("year", json!(1899)),
        ("year", json!(2101)),
        ("month", json!(13)),
        ("day", json!(31)),
        ("hour", json!(24)),
        ("minute", json!(60)),
        ("tz", json!(15)),
    ] {
        let mut x = i.clone();
        x[k] = v;
        let (status, _) = post("/api/bazi/report", &x).await;
        assert!((400..500).contains(&status), "{k}: {status}");
    }
    for y in [1900, 2100] {
        for tz in [-12.0, 14.0, 5.75] {
            let mut x = i.clone();
            x["year"] = json!(y);
            x["tz"] = json!(tz);
            let (s, r) = post("/api/bazi/report", &x).await;
            assert_eq!(s, 200);
            assert_eq!(r["chart"]["input"]["tz"], json!(tz));
        }
    }
}

#[tokio::test]
async fn report_uses_new_model_for_month_and_cycle_boundary_together() {
    let input = json!({"year":2013,"month":5,"day":5,"hour":8,"minute":10,"tz":0,"gender":"male","true_solar_time":false});
    let (_, old) = post("/api/bazi", &input).await;
    let (status, new) = post("/api/bazi/report", &input).await;
    assert_eq!(status, 200);
    assert_eq!(old["month"]["branch"], "巳");
    assert_eq!(new["chart"]["month"]["branch"], "辰");
    assert_eq!(
        new["cycle_basis"]["previous_jie"]["target_longitude_deg"],
        15.0
    );
    assert_eq!(new["cycle_basis"]["next_jie"]["target_longitude_deg"], 45.0);
    assert_ne!(new["chart"]["dayun"]["pillars"], old["dayun"]["pillars"]);
}
