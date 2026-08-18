//! 每个端点至少打一次。
//!
//! 拆分前这一层有二十三个 handler、零条测试。「编译过了」只说明类型对得上，说明不了
//! 路由挂没挂、DTO 的 `#[serde(default)]` 缺省成什么、错误路径答的是 400 还是 500。
//! 这些都是承接层自己的性质，用例层的测试看不见。
//!
//! 释义端点一律用离线模板后端（[`mingli_api::backend::Interpret::Offline`]）：这里要验的是
//! 「这条路走得通、出来的东西标着 INT」，不是 LLM 今天怎么说。

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use mingli_api::backend::Interpret;
use serde_json::{json, Value};
use tower::ServiceExt;

/// 在进程内打一次端点。释义走离线模板，故全程不出进程、不碰网络。
async fn hit(method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let req = Request::builder().method(method).uri(path);
    let req = match body {
        Some(v) => req
            .header("content-type", "application/json")
            .body(Body::from(v.to_string())),
        None => req.body(Body::empty()),
    }
    .expect("请求应可构造");
    let res = mingli_api::router_with(Interpret::Offline)
        .oneshot(req)
        .await
        .expect("路由应可响应");
    let status = res.status();
    let bytes = res.into_body().collect().await.expect("body 应可读").to_bytes();
    // 422 是 axum 的 Json 抽取器自己回的，body 是纯文本；不是 JSON 就原样带回，
    // 免得量具在这里 panic，把「响应形状不是我以为的」变成「测试挂了」。
    let v = serde_json::from_slice(&bytes).unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into()));
    (status, v)
}

async fn get(path: &str) -> (StatusCode, Value) {
    hit("GET", path, None).await
}

async fn post(path: &str, body: Value) -> (StatusCode, Value) {
    hit("POST", path, Some(body)).await
}

/// 本命一路各端点共用的入参。
fn natal_body() -> Value {
    json!({
        "year": 1990, "month": 6, "day": 15, "hour": 14, "minute": 30, "tz": 8.0,
        "gender": "male", "latitude": 31.23, "longitude": 121.47, "seed": 2024,
        "name": "Ada Lovelace"
    })
}

/// 岁运叠加的入参：本命字段平铺（`#[serde(flatten)]`）+ extras。
fn overlay_body(extras: &[&str]) -> Value {
    let mut b = natal_body();
    b["extras"] = json!(extras);
    b
}

fn t(year: i32, month: u32, day: u32) -> Value {
    json!({ "year": year, "month": month, "day": day, "hour": 10, "minute": 0, "tz": 8.0 })
}

fn two_people() -> (Value, Value) {
    (
        json!({"year":1990,"month":6,"day":15,"hour":14,"tz":8.0,"gender":"male","name":"A"}),
        json!({"year":1987,"month":3,"day":2,"hour":9,"tz":8.0,"gender":"female","name":"B"}),
    )
}

/// 释义响应的共同形状：标着 INT，说了是谁说的，且真有话。
fn is_interpretation(v: &Value) -> bool {
    v["kind"] == "INT" && v["backend"].is_string() && !v["text"].as_str().unwrap_or("").is_empty()
}

// ===================== 元数据 =====================

#[tokio::test]
async fn health_lists_what_is_registered() {
    let (s, v) = get("/api/health").await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v["status"], "ok");
    let n = v["leaves"].as_array().expect("leaves 应是数组").len();
    assert_eq!(v["leaf_count"].as_u64(), Some(n as u64), "自报的数目要和列出的一致");
    assert!(n > 0);
}

#[tokio::test]
async fn intents_come_with_the_leaves_they_route_to() {
    let (s, v) = get("/api/intents").await;
    assert_eq!(s, StatusCode::OK);
    let intents = v["intents"].as_array().expect("intents 应是数组");
    assert_eq!(intents.len(), 8, "契约里是 8 类问事意图");
    for i in intents {
        assert!(i["id"].is_string() && i["name_zh"].is_string());
        assert!(i["default_leaves"].is_array());
        assert!(i["status_label"].is_string());
    }
    assert!(!v["registered_leaves"].as_array().expect("应是数组").is_empty());
}

#[tokio::test]
async fn analysis_returns_a_square_matrix() {
    let (s, v) = get("/api/analysis").await;
    assert_eq!(s, StatusCode::OK);
    let k = v["leaves"].as_array().expect("leaves 应是数组").len();
    let m = v["nmi"].as_array().expect("nmi 应是数组");
    assert_eq!(m.len(), k);
    for row in m {
        assert_eq!(row.as_array().expect("每行应是数组").len(), k);
    }
}

#[tokio::test]
async fn route_answers_with_the_leaves_for_that_kind() {
    // QueryKind 是内部标签枚举，Natal 是 newtype variant → kind 与 Query 的字段平铺在一起。
    let mut body = natal_body();
    body["kind"] = json!("natal");
    let (s, v) = post("/api/route", body).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(v["intent"], "natal");
    assert!(!v["leaves"].as_array().expect("leaves 应是数组").is_empty());
}

// ===================== 本命一路 =====================

#[tokio::test]
async fn bazi_gives_the_four_pillars() {
    let (s, v) = post("/api/bazi", natal_body()).await;
    assert_eq!(s, StatusCode::OK);
    // 1990-06-15 14:30 +8 → 庚午 壬午 辛亥 乙未
    assert_eq!(v["year"]["ganzhi"], "庚午");
    assert_eq!(v["month"]["ganzhi"], "壬午");
    assert_eq!(v["day"]["ganzhi"], "辛亥");
    assert_eq!(v["hour"]["ganzhi"], "乙未");
}

#[tokio::test]
async fn bazi_refuses_a_year_outside_the_supported_range() {
    let mut b = natal_body();
    b["year"] = json!(1800);
    let (s, v) = post("/api/bazi", b).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert!(v["error"].is_string());
}

#[tokio::test]
async fn a_body_missing_a_required_field_is_rejected_before_the_handler() {
    // serde 的拒绝走 422，不是 handler 的 400——两者形状不同，写下来免得日后当成回归。
    let (s, _) = post("/api/bazi", json!({ "year": 1990 })).await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn ziwei_places_twelve_palaces() {
    let (s, v) = post("/api/ziwei", natal_body()).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v["palaces"].as_array().expect("palaces 应是数组").len(), 12);
    assert!(v["ming_branch"].is_string());
}

#[tokio::test]
async fn cast_fans_out_to_every_registered_leaf() {
    let (s, v) = post("/api/cast", natal_body()).await;
    assert_eq!(s, StatusCode::OK);
    let leaves = v["leaves"].as_array().expect("leaves 应是数组");
    let (hs, hv) = get("/api/health").await;
    assert_eq!(hs, StatusCode::OK);
    assert_eq!(leaves.len(), hv["leaves"].as_array().expect("应是数组").len(), "排的叶要和注册的叶一样多");
    for l in leaves {
        assert!(l["id"].is_string() && l["chart"].is_object());
    }
}

#[tokio::test]
async fn overlay_recomputes_strength_with_the_extra_pillars() {
    let (s, v) = post("/api/bazi/overlay-strength", overlay_body(&["丙午", "庚申"])).await;
    assert_eq!(s, StatusCode::OK);
    assert!(v["ming"]["score"].is_number(), "{v}");
    assert!(v["yun"]["score"].is_number(), "本命与叠加后两份旺衰都要在");
    assert!(v["delta_score"].is_number(), "以及两者之差");
}

#[tokio::test]
async fn overlay_refuses_a_pillar_that_is_not_a_pillar() {
    let (s, v) = post("/api/bazi/overlay-strength", overlay_body(&["不是干支"])).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert!(v["error"].is_string());
}

#[tokio::test]
async fn fortune_gives_a_slice_and_a_timeline() {
    let (s, v) = post("/api/fortune", json!({ "natal": natal_body(), "t_target": t(2026, 8, 16) })).await;
    assert_eq!(s, StatusCode::OK);
    assert!(v.is_object() && !v.as_object().expect("应是对象").is_empty());
}

#[tokio::test]
async fn interpreting_one_leaf_is_marked_int() {
    let mut b = natal_body();
    b["leaf"] = json!("bazi");
    let (s, v) = post("/api/interpret", b).await;
    assert_eq!(s, StatusCode::OK);
    assert!(is_interpretation(&v), "{v}");
    assert_eq!(v["leaf"], "bazi");
}

#[tokio::test]
async fn interpreting_a_leaf_that_does_not_exist_is_the_callers_problem() {
    let mut b = natal_body();
    b["leaf"] = json!("没有这片叶");
    let (s, v) = post("/api/interpret", b).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "未知叶是 400 而非 500");
    assert!(v["error"].is_string());
}

// ===================== 团队 / 合盘 =====================

#[tokio::test]
async fn team_needs_more_than_nobody() {
    let (s, v) = post("/api/team", json!({ "members": [] })).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert!(v["error"].is_string());
}

#[tokio::test]
async fn team_reads_a_group() {
    let (a, b) = two_people();
    let (s, v) = post("/api/team", json!({ "members": [a, b] })).await;
    assert_eq!(s, StatusCode::OK);
    assert!(v.is_object() && !v.as_object().expect("应是对象").is_empty());
}

#[tokio::test]
async fn team_interpretation_is_marked_int() {
    let (a, b) = two_people();
    let (s, v) = post("/api/team/interpret", json!({ "members": [a, b] })).await;
    assert_eq!(s, StatusCode::OK);
    assert!(is_interpretation(&v), "{v}");
}

#[tokio::test]
async fn synastry_measures_both_directions() {
    let (a, b) = two_people();
    let (s, v) = post("/api/synastry", json!({ "a": a, "b": b })).await;
    assert_eq!(s, StatusCode::OK);
    assert!(v.is_object() && !v.as_object().expect("应是对象").is_empty());
}

#[tokio::test]
async fn synastry_interpretation_is_marked_int() {
    let (a, b) = two_people();
    let (s, v) = post("/api/synastry/interpret", json!({ "a": a, "b": b })).await;
    assert_eq!(s, StatusCode::OK);
    assert!(is_interpretation(&v), "{v}");
}

// ===================== 字词 =====================

#[tokio::test]
async fn word_takes_text_instead_of_a_moment() {
    let (s, v) = post("/api/word", json!({ "system": "gematria", "text": "chai" })).await;
    assert_eq!(s, StatusCode::OK);
    assert!(v.is_object() && !v.as_object().expect("应是对象").is_empty());
}

#[tokio::test]
async fn word_refuses_a_system_it_does_not_have() {
    let (s, v) = post("/api/word", json!({ "system": "没有这个系统", "text": "x" })).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert!(v["error"].is_string());
}

// ===================== 占事 =====================

#[tokio::test]
async fn event_casts_the_divination_leaves() {
    let (s, v) = post("/api/event", json!({ "t_ask": t(2026, 8, 16), "seed": 7, "question": "能成吗" })).await;
    assert_eq!(s, StatusCode::OK);
    assert!(v.is_object() && !v.as_object().expect("应是对象").is_empty());
}

#[tokio::test]
async fn event_interpretation_is_marked_int() {
    let (s, v) = post("/api/event/interpret", json!({ "t_ask": t(2026, 8, 16), "seed": 7 })).await;
    assert_eq!(s, StatusCode::OK);
    assert!(is_interpretation(&v), "{v}");
}

// ===================== 择吉 =====================

#[tokio::test]
async fn election_scans_the_window() {
    let (s, v) = post(
        "/api/election",
        json!({ "window_start": t(2026, 8, 16), "window_end": t(2026, 8, 20), "category": "婚" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(v.is_object() && !v.as_object().expect("应是对象").is_empty());
}

#[tokio::test]
async fn election_refuses_a_window_that_runs_backwards() {
    let (s, v) = post(
        "/api/election",
        json!({ "window_start": t(2026, 8, 20), "window_end": t(2026, 8, 16) }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert!(v["error"].is_string());
}

#[tokio::test]
async fn election_interpretation_is_marked_int() {
    let (s, v) = post(
        "/api/election/interpret",
        json!({ "window_start": t(2026, 8, 16), "window_end": t(2026, 8, 18), "category": "婚" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(is_interpretation(&v), "{v}");
}

// ===================== 寻方位 =====================

#[tokio::test]
async fn locative_points_somewhere() {
    let (s, v) = post("/api/locative", json!({ "t_ask": t(2026, 8, 16), "seed": 7, "category": "财" })).await;
    assert_eq!(s, StatusCode::OK);
    assert!(v.is_object() && !v.as_object().expect("应是对象").is_empty());
}

#[tokio::test]
async fn locative_interpretation_is_marked_int() {
    let (s, v) = post("/api/locative/interpret", json!({ "t_ask": t(2026, 8, 16), "seed": 7 })).await;
    assert_eq!(s, StatusCode::OK);
    assert!(is_interpretation(&v), "{v}");
}

// ===================== 国运 =====================

fn mundane_body() -> Value {
    json!({
        "founded_at": { "year": 1949, "month": 10, "day": 1, "hour": 15, "minute": 0, "tz": 8.0 },
        "latitude": 39.9, "longitude": 116.4, "target_year": 2026, "span": 3
    })
}

#[tokio::test]
async fn mundane_walks_the_years() {
    let (s, v) = post("/api/mundane", mundane_body()).await;
    assert_eq!(s, StatusCode::OK);
    assert!(v.is_object() && !v.as_object().expect("应是对象").is_empty());
}

#[tokio::test]
async fn mundane_interpretation_is_marked_int() {
    let (s, v) = post("/api/mundane/interpret", mundane_body()).await;
    assert_eq!(s, StatusCode::OK);
    assert!(is_interpretation(&v), "{v}");
}

/// `/api/route` 直接吃 [`mingli_contract::Query`]，其余端点吃 DTO——两条路的性别拼法必须一样。
///
/// 曾经不一样：契约枚举按 Rust 的拼法收 `"Male"`，DTO 层与 web 说 `"male"`，
/// 于是同一个 body 打 `/api/bazi` 是 200、打 `/api/route` 是 422。
#[tokio::test]
async fn the_two_ways_in_spell_gender_the_same() {
    let mut body = natal_body();
    body["kind"] = json!("natal");
    assert_eq!(body["gender"], json!("male"));
    let (s, v) = post("/api/route", body).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    let (bs, _) = post("/api/bazi", natal_body()).await;
    assert_eq!(bs, StatusCode::OK);
}

/// 旧拼法仍然收，别名不是摆设。
#[tokio::test]
async fn the_old_spelling_of_gender_still_works() {
    let mut body = natal_body();
    body["kind"] = json!("natal");
    body["gender"] = json!("Male");
    let (s, v) = post("/api/route", body).await;
    assert_eq!(s, StatusCode::OK, "{v}");
}


/// 500 那条路也要走一遍。
///
/// 覆盖率上 `error::server_error` 从没被执行过——端点测试用的是离线模板后端，
/// 它不会失败，于是「释义后端不可用」这条分支一次都没跑。
///
/// 不去起真后端来制造失败：这台机器上恰好装着那个外部进程，测试会真的把它跑起来，
/// 一次六十秒且结果不确定。直接验错误响应本身即可——要钉的是**形状**，
/// 而形状与它由哪条路径触发无关。
#[tokio::test]
async fn the_five_hundred_shape_matches_the_four_hundred_shape() {
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;

    let read = |r: axum::response::Response| async {
        let status = r.status();
        let b = r.into_body().collect().await.expect("body 应可读").to_bytes();
        (status, serde_json::from_slice::<Value>(&b).expect("body 应是 JSON"))
    };

    let (s4, v4) = read(mingli_api::error::bad_request("时窗终点早于起点").into_response()).await;
    let (s5, v5) = read(mingli_api::error::server_error("释义后端不可用").into_response()).await;

    assert_eq!(s4, StatusCode::BAD_REQUEST);
    assert_eq!(s5, StatusCode::INTERNAL_SERVER_ERROR);
    for v in [&v4, &v5] {
        let o = v.as_object().expect("应是对象");
        assert_eq!(o.len(), 1, "错误响应只有一个字段");
        assert!(o["error"].is_string(), "那个字段叫 error 且是字符串");
    }
    assert_eq!(v4["error"], "时窗终点早于起点");
    assert_eq!(v5["error"], "释义后端不可用");
}

/// 认不出的 `subject` 要被拒，不能当成没写。
///
/// 缺省是人盘，这没问题；写了个认不出的值却也落到人盘，就是「默默忽略」——
/// 拼错 `company` 的人拿到的是人盘的读法，却以为看的是公司盘，而响应里没有任何迹象。
/// 同一份请求里的 `gender` 早已改成拒绝拼错值，`subject` 一直没有。
///
/// 别名与 `gender` 对齐：首字母大写与中文都收。两处宽严不一只会让人踩坑——
/// `gender` 收 `"Male"` 而 `subject` 不收 `"Person"`，没有道理。
#[tokio::test]
async fn an_unrecognised_subject_is_refused_rather_than_read_as_a_person() {
    let ask = |subject: Option<&str>| {
        let mut body = json!({ "year": 1990, "month": 6, "day": 15, "hour": 14, "tz": 8, "leaf": "bazi" });
        if let Some(s) = subject {
            body["subject"] = json!(s);
        }
        post("/api/interpret", body)
    };

    // 缺省与四种主体的英文、大写、中文写法都收
    let (s, _) = ask(None).await;
    assert_eq!(s, StatusCode::OK, "不写 subject 应落人盘");
    for good in ["person", "Person", "人", "company", "Company", "公司", "product", "Product", "物", "产品", "object", "Object", "event", "Event", "事"] {
        let (s, v) = ask(Some(good)).await;
        assert_eq!(s, StatusCode::OK, "`{good}` 应被认出，实得 {v}");
    }

    // 认不出的一律拒，且说清收哪些
    for bad in ["compnay", "PERSON", "组织", "", "human"] {
        let (s, v) = ask(Some(bad)).await;
        assert_eq!(s, StatusCode::BAD_REQUEST, "`{bad}` 应被拒，实得 {v}");
        let msg = v["error"].as_str().unwrap_or_default();
        assert!(msg.contains(bad) && msg.contains("company"), "错误里要带上原值与可选值，实为「{msg}」");
    }
}
