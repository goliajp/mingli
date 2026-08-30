//! 承接层不许在用例层之外多做事。
//!
//! 每个纯计算端点都该是「解 DTO → 调用例 → 序列化」三步，中间不加工。可它是不是真的没加工，
//! 光读代码看不出来——handler 里悄悄补一个字段、改个命名、把某段裁掉，都不会有任何东西报错，
//! 直到前端与用例层各自按不同的形状写下去，两边就此漂开。
//!
//! 这里把两侧摆在一起逐字节比：同一入参，直接调用例层得到的 JSON，与在进程内打端点拿回的
//! body，必须一模一样。不同就是承接层动了手脚——要么是有意的（那该写在别处并说明），
//! 要么是漏的（那正是这条测试要抓的）。

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

/// 在进程内打一次端点，返回 (状态码, body **原文**)。
///
/// 刻意返回原文而不是 `serde_json::Value`：`serde_json::to_value` 会把 f64 过一遍 `Number`，
/// 某些值上会掉一个 ULP。拿它做比较，会凭空造出「端点与用例层不一致」的假象——
/// 本测试第一版就是这么被自己的量具骗了一回。两侧一律比序列化后的文本。
async fn post(path: &str, body: serde_json::Value) -> (StatusCode, String) {
    let res = mingli_api::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("请求应可构造"),
        )
        .await
        .expect("路由应可响应");
    let status = res.status();
    let bytes = res.into_body().collect().await.expect("body 应可读").to_bytes();
    (status, String::from_utf8(bytes.to_vec()).expect("body 应是 UTF-8"))
}

async fn get(path: &str) -> (StatusCode, serde_json::Value) {
    let res = mingli_api::router()
        .oneshot(Request::builder().uri(path).body(Body::empty()).expect("请求应可构造"))
        .await
        .expect("路由应可响应");
    let status = res.status();
    let bytes = res.into_body().collect().await.expect("body 应可读").to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null))
}

fn birth() -> mingli_app::Birth {
    mingli_app::Birth {
        year: 1987,
        month: 9,
        day: 17,
        hour: 15,
        minute: 0,
        tz: 8.0,
        gender: Some(mingli_contract::Gender::Male),
        true_solar_time: false,
        longitude: None,
    }
}

fn birth_body() -> serde_json::Value {
    serde_json::json!({
        "year": 1987, "month": 9, "day": 17, "hour": 15, "minute": 0,
        "tz": 8.0, "gender": "male"
    })
}

fn ask() -> mingli_contract::AskTime {
    mingli_contract::AskTime { year: 2026, month: 8, day: 17, hour: 14, minute: 0, tz: 8.0 }
}

fn ask_body() -> serde_json::Value {
    serde_json::json!({ "year": 2026, "month": 8, "day": 17, "hour": 14, "minute": 0, "tz": 8.0 })
}

/// 逐字节比：用例层直接序列化的文本 ⟺ 端点返回的 body 原文。
fn same<T: serde::Serialize>(label: &str, from_use_case: &T, from_endpoint: &str) {
    let a = serde_json::to_string(from_use_case).expect("用例层结果应可序列化");
    assert_eq!(
        a, from_endpoint,
        "{label}：端点返回与用例层直出不一致——承接层只该做协议转换，不该加工结果。\n\
         用例层 {} 字节 / 端点 {} 字节",
        a.len(),
        from_endpoint.len()
    );
}

#[tokio::test]
async fn natal_endpoints_pass_the_use_case_through_untouched() {
    let (s, body) = post("/api/bazi", birth_body()).await;
    assert_eq!(s, StatusCode::OK);
    same("/api/bazi", &mingli_app::bazi::natal(&birth()), &body);

    let (s, body) = post("/api/ziwei", birth_body()).await;
    assert_eq!(s, StatusCode::OK);
    same("/api/ziwei", &mingli_app::ziwei::natal(&birth()), &body);
}

#[tokio::test]
async fn intent_endpoints_pass_the_use_case_through_untouched() {
    let reg = mingli_registry::registry();

    let (s, body) = post("/api/event", serde_json::json!({ "t_ask": ask_body(), "seed": 42 })).await;
    assert_eq!(s, StatusCode::OK);
    let ev = mingli_app::event::cast(&reg, &ask(), Some(42), None).expect("占事应可起");
    same("/api/event", &ev, &body);

    let (s, body) = post(
        "/api/locative",
        serde_json::json!({ "t_ask": ask_body(), "seed": 42, "category": "寻物" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let lo = mingli_app::locative::cast(&reg, &ask(), Some(42), Some("寻物".into())).expect("应可寻");
    same("/api/locative", &lo, &body);

    let (s, body) = post(
        "/api/mundane",
        serde_json::json!({ "founded_at": ask_body(), "target_year": 2030 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let mu = mingli_app::mundane::cast(&reg, &ask(), None, None, Some(2030), None).expect("应可推");
    same("/api/mundane", &mu, &body);

    let (s, body) = post(
        "/api/election",
        serde_json::json!({ "window_start": ask_body(), "window_end": { "year": 2026, "month": 9, "day": 1, "hour": 12, "minute": 0, "tz": 8.0 } }),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let end = mingli_contract::AskTime { year: 2026, month: 9, day: 1, hour: 12, minute: 0, tz: 8.0 };
    let el = mingli_app::election::scan(&ask(), &end, None).expect("应可择");
    same("/api/election", &el, &body);
}

/// 全叶排盘：21 片叶的输出，最容易在承接层被顺手加工。
///
/// 这个端点确实加了一层 `{"leaves": [...]}` 的外壳——那是协议成形（让响应是对象而非裸数组），
/// 属承接层本分。壳里的内容必须与编排层直出的一字不差。
#[tokio::test]
async fn the_full_cast_endpoint_only_adds_an_envelope() {
    let (s, body) = post("/api/cast", birth_body()).await;
    assert_eq!(s, StatusCode::OK);
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("body 应是 JSON");
    assert_eq!(
        parsed.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()),
        Some(vec!["leaves".to_string()]),
        "/api/cast 的外壳只该有 leaves 一个键"
    );
    let q = mingli_contract::Query {
        year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
        gender: Some(mingli_contract::Gender::Male),
        latitude: None, longitude: None, seed: None, name: None,
        schools: std::collections::BTreeMap::new(),
    };
    let want = mingli_engine::cast_all_detailed(&mingli_registry::registry(), &q);
    let enveloped = serde_json::to_string(&serde_json::json!({ "leaves": [] })).expect("壳可序列化");
    assert!(enveloped.contains("leaves"), "壳的形状");
    same("/api/cast", &serde_json::json!({ "leaves": want }), &body);
}

/// 元数据端点：叶数与注册表一致，意图清单与契约一致。
#[tokio::test]
async fn the_metadata_endpoints_report_the_registry_as_it_is() {
    let (s, health) = get("/api/health").await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        health["leaf_count"].as_u64().expect("应有 leaf_count"),
        mingli_registry::registry().len() as u64,
        "/api/health 报的叶数应等于注册表实际长度"
    );

    let (s, intents) = get("/api/intents").await;
    assert_eq!(s, StatusCode::OK);
    let declared: Vec<&str> = mingli_contract::intents().iter().map(|i| i.id.id()).collect();
    let served: Vec<String> = intents["intents"]
        .as_array()
        .expect("/api/intents 应有 intents 数组")
        .iter()
        .map(|i| i["id"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(served, declared, "/api/intents 应原样转述契约的意图清单");
}
