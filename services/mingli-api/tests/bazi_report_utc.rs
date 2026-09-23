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

#[tokio::test]
async fn recorded_minute_endpoint_returns_complete_alternatives_and_keeps_original_record() {
    let mut dump=Vec::new();
    for (id,input,count) in [
        ("inside_jie_minute",json!({"year":2013,"month":5,"day":5,"hour":8,"minute":18,"tz":0,"gender":"female","true_solar_time":false}),2),
        ("normal",json!({"year":1990,"month":6,"day":15,"hour":14,"minute":30,"tz":8,"gender":"female","true_solar_time":false}),1),
        ("leap_minute",json!({"year":2016,"month":12,"day":31,"hour":23,"minute":59,"tz":0,"gender":"male","true_solar_time":false}),1),
        ("li_chun_minute",json!({"year":2013,"month":2,"day":3,"hour":16,"minute":13,"tz":0,"gender":"female","true_solar_time":false}),2),
    ] {
        let (status,response)=post("/api/bazi/report/utc/minute",&input).await;
        assert_eq!(status,200,"{id}: {response}");
        let candidates=response["candidates"].as_array().unwrap();
        assert_eq!(candidates.len(),count,"{id}");
        assert_eq!(response["recorded_input"]["minute"],input["minute"]);
        assert_eq!(response["minute"]["start_inclusive"],true);
        assert_eq!(response["minute"]["end_inclusive"],false);
        for c in candidates {
            assert_eq!(c["report"]["cycle_basis"]["birth_jde_tt"],c["representative"]["jde_tt"]);
            assert_eq!(c["report"]["chart"]["dayun"]["pillars"].as_array().unwrap().len(),10);
        }
        dump.push(json!({"id":id,"input":input,"response":response,"build":{"build_id":env!("MINGLI_BUILD_ID"),"source_sha256":env!("MINGLI_SOURCE_SHA256")}}));
    }
    if let Ok(path)=std::env::var("MINGLI_UTC_MINUTE_DUMP") {std::fs::write(path,serde_json::to_string_pretty(&dump).unwrap()).unwrap();}
    for input in [
        json!({"year":1961,"month":7,"day":31,"hour":23,"minute":59,"tz":0,"gender":"female"}),
        json!({"year":2027,"month":7,"day":1,"hour":0,"minute":0,"tz":0,"gender":"female"}),
        json!({"year":1990,"month":6,"day":15,"hour":14,"minute":30,"tz":8,"gender":"female","true_solar_time":true}),
    ] {assert_eq!(post("/api/bazi/report/utc/minute",&input).await.0,400);}
}

#[tokio::test]
async fn minute_boundary_matrix_uses_actual_api_roots_and_both_genders() {
    let mut dump=Vec::new();
    for year in [1900,1961,2013,2026] {
        for month in 1..=12 {
            let seed=json!({"year":year,"month":month,"day":15,"hour":0,"minute":0,"tz":0,"gender":"female","true_solar_time":false});
            let (status,r)=post("/api/bazi/report/utc",&seed).await;
            assert_eq!(status,200);
            let root=r["cycle_basis"]["previous_jie"]["jd_civil"].as_f64().unwrap();
            let mid=r["cycle_basis"]["birth_jd_civil"].as_f64().unwrap();
            let minute=((root+0.5)*1440.0).floor() as i64;
            let day=15+minute.div_euclid(1440)-(mid+0.5).floor() as i64;
            let within=minute.rem_euclid(1440);
            for gender in ["male","female"] {
                let input=json!({"year":year,"month":month,"day":day,"hour":within/60,"minute":within%60,"tz":0,"gender":gender,"true_solar_time":false});
                let (status,response)=post("/api/bazi/report/utc/minute",&input).await;
                assert_eq!(status,200,"{input}: {response}");
                assert_eq!(response["candidates"].as_array().unwrap().len(),2);
                dump.push(json!({"id":format!("{year}-{month:02}-{gender}"),"input":input,"response":response,"build":{"build_id":env!("MINGLI_BUILD_ID"),"source_sha256":env!("MINGLI_SOURCE_SHA256")}}));
            }
        }
    }
    assert_eq!(dump.len(),96);
    if let Ok(path)=std::env::var("MINGLI_UTC_MINUTE_BOUNDARIES_DUMP") {std::fs::write(path,serde_json::to_string_pretty(&dump).unwrap()).unwrap();}
}
