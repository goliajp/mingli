//! 同一份入参，走 HTTP 与走 wasm，收到的该是同一份东西。
//!
//! 这套引擎有两扇门：服务端的 axum 端点，与浏览器里整库跑的 wasm 包。两边共用一套入参类型
//! （`mingli_app::input`），所以请求形状不会各写一遍；出参却是各自序列化的，谁多包一层、谁少
//! 取一段，都不会有任何东西报错。wasm 包的模块说明写着「同一份 body 打服务端还是喂 wasm，
//! 收到的东西一样」——那是一句承诺，本文件是它的凭据。
//!
//! 比的是**解析回来的 JSON**，唯独放过键的先后：HTTP 那侧的 handler 交给 axum 的是
//! `serde_json::Value`，它的对象底是 BTreeMap，键因此按字典序落盘；wasm 直接序列化结构体，
//! 键按声明序。同一份数据的两种排法，谁也不比谁对，而 JSON 的对象本就无序。
//!
//! 放过的只有这一件事。数值仍然逐个比对——`Value` 的数字相等比的是解析出的 f64 本身，
//! 一侧多过一遍 `to_value` 而掉了一个 ULP，这里照样会红（同仓 `no_drift.rs` 有过这一课）。

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

const B: &str = r#"{"year":1990,"month":6,"day":15,"hour":14,"minute":30,"tz":8.0,"gender":"male"}"#;
const B2: &str = r#"{"year":1987,"month":3,"day":2,"hour":9,"tz":8.0,"gender":"female","name":"B"}"#;
const T: &str = r#"{"year":2026,"month":8,"day":16,"hour":10,"minute":0,"tz":8.0}"#;

/// 在进程内打一次端点，返回 body 原文。非 200 直接失败——两扇门连成败都该一致。
async fn http(path: &str, body: &str) -> String {
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
        .expect("路由应可调用");
    let status = res.status();
    let bytes = res.into_body().collect().await.expect("body 应可读").to_bytes();
    let text = String::from_utf8(bytes.to_vec()).expect("body 应是 UTF-8");
    assert_eq!(status, StatusCode::OK, "{path} 未返回 200：{text}");
    text
}

fn same(label: &str, from_http: &str, from_wasm: &str) {
    let h: serde_json::Value = serde_json::from_str(from_http).expect("HTTP body 应是 JSON");
    let w: serde_json::Value = serde_json::from_str(from_wasm).expect("wasm 出参应是 JSON");
    assert_eq!(h, w, "{label}：两扇门给出的不是同一份\n  HTTP: {from_http}\n  wasm: {from_wasm}");
}

#[tokio::test]
async fn the_two_natal_doors_agree() {
    same("/api/bazi", &http("/api/bazi", B).await, &mingli_wasm::bazi(B).expect("wasm 排盘"));
    same("/api/ziwei", &http("/api/ziwei", B).await, &mingli_wasm::ziwei(B).expect("wasm 排盘"));
}

#[tokio::test]
async fn the_two_intent_doors_agree() {
    let fortune = format!(r#"{{"natal":{B},"t_target":{T}}}"#);
    same("/api/fortune", &http("/api/fortune", &fortune).await, &mingli_wasm::fortune(&fortune).expect("wasm 运势"));

    let team = format!(r#"{{"members":[{B},{B2}]}}"#);
    same("/api/team", &http("/api/team", &team).await, &mingli_wasm::team(&team).expect("wasm 团队"));

    let syn = format!(r#"{{"a":{B},"b":{B2}}}"#);
    same("/api/synastry", &http("/api/synastry", &syn).await, &mingli_wasm::synastry(&syn).expect("wasm 合盘"));

    let event = format!(r#"{{"t_ask":{T},"seed":42}}"#);
    same("/api/event", &http("/api/event", &event).await, &mingli_wasm::event(&event).expect("wasm 占事"));

    let election = format!(r#"{{"window_start":{T},"window_end":{{"year":2026,"month":8,"day":20}}}}"#);
    same(
        "/api/election",
        &http("/api/election", &election).await,
        &mingli_wasm::election(&election).expect("wasm 择吉"),
    );

    let locative = format!(r#"{{"t_ask":{T},"seed":42}}"#);
    same(
        "/api/locative",
        &http("/api/locative", &locative).await,
        &mingli_wasm::locative(&locative).expect("wasm 寻方位"),
    );

    let mundane = r#"{"founded_at":{"year":1949,"month":10,"day":1,"hour":15},"target_year":2026,"span":3}"#;
    same("/api/mundane", &http("/api/mundane", mundane).await, &mingli_wasm::mundane(mundane).expect("wasm 国运"));
}

#[tokio::test]
async fn the_two_word_doors_agree() {
    let hebrew = r#"{"system":"gematria","text":"שלום"}"#;
    same("/api/word gematria", &http("/api/word", hebrew).await, &mingli_wasm::gematria("שלום"));

    let arabic = r#"{"system":"abjad","text":"الله"}"#;
    same("/api/word abjad", &http("/api/word", arabic).await, &mingli_wasm::abjad("الله"));

    let name = r#"{"system":"wuge","surname":[7],"given":[16,9]}"#;
    same("/api/word wuge", &http("/api/word", name).await, &mingli_wasm::wuge("[7]", "[16,9]").expect("wasm 五格"));
}

/// 全叶排盘是唯一一处两边刻意不同：HTTP 多一层 `{"leaves": …}` 外壳。
///
/// 壳是给 HTTP 的——顶层直接回数组是 JSON 响应上一个老问题，端点一律回对象；wasm 这边
/// 函数返回的是一个字符串，没有这层顾虑，故直接给内容。壳里装的东西必须一字不差，
/// 这条测试守的就是「差别只有那层壳」。
#[tokio::test]
async fn the_full_cast_differs_only_by_the_envelope() {
    let from_http = http("/api/cast", B).await;
    let from_wasm = mingli_wasm::cast(B).expect("wasm 排盘");
    let enveloped = format!(r#"{{"leaves":{from_wasm}}}"#);
    same("/api/cast", &from_http, &enveloped);
}

/// 两扇门要拒绝同样的东西，不只是对同样的东西给出同样的答案。
///
/// 上面几条比的都是**成功路径**。可「拒不拒」同样是契约的一半：HTTP 在承接层收一次，
/// wasm 吃的是裸 `Query`——本仓库给日加上「按当月实际长度收」之后，一度出现过
/// 同一个 2 月 31 日在服务端被拒、在浏览器里照常出盘的局面。
///
/// 这里不直接调 wasm：它的错误路径返回 `JsValue`，在非 wasm32 目标上构造不出来，
/// 一碰就 panic（`crates/mingli-wasm` 的测试注释里写了这件事）。故改为核**两边落到的
/// 是不是同一段判断**：HTTP 给 400 的入参，用例层的 `validate_query` 也必须给 Err，
/// 而 wasm 的 `parse_query` 调的正是它。
#[tokio::test]
async fn the_two_doors_refuse_the_same_things() {
    // (给端点的 body, 给 Query 的等价 body)——Query 的字段没有 serde 缺省，要写全
    let cases = [
        (
            r#"{"year":1990,"month":2,"day":31,"hour":14,"tz":8}"#,
            r#"{"year":1990,"month":2,"day":31,"hour":14,"minute":0,"tz":8.0,"gender":"male","latitude":null,"longitude":null,"seed":null,"name":null,"schools":{}}"#,
            "不存在的日期",
        ),
        (
            r#"{"year":1990,"month":6,"day":15,"hour":14,"tz":99}"#,
            r#"{"year":1990,"month":6,"day":15,"hour":14,"minute":0,"tz":99.0,"gender":"male","latitude":null,"longitude":null,"seed":null,"name":null,"schools":{}}"#,
            "现实中不存在的时区",
        ),
        (
            r#"{"year":1990,"month":6,"day":15,"hour":14,"tz":8,"latitude":91}"#,
            r#"{"year":1990,"month":6,"day":15,"hour":14,"minute":0,"tz":8.0,"gender":"male","latitude":91.0,"longitude":null,"seed":null,"name":null,"schools":{}}"#,
            "球面外的纬度",
        ),
        (
            r#"{"year":1899,"month":6,"day":15,"hour":14,"tz":8}"#,
            r#"{"year":1899,"month":6,"day":15,"hour":14,"minute":0,"tz":8.0,"gender":"male","latitude":null,"longitude":null,"seed":null,"name":null,"schools":{}}"#,
            "支持区间之外的年份",
        ),
    ];
    for (http_body, query_body, what) in cases {
        let res = mingli_api::router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/cast")
                    .header("content-type", "application/json")
                    .body(Body::from(http_body.to_string()))
                    .expect("请求应可构造"),
            )
            .await
            .expect("路由应可调用");
        assert_eq!(res.status(), StatusCode::BAD_REQUEST, "/api/cast 应拒绝{what}");

        let q: mingli_contract::Query = serde_json::from_str(query_body).expect("Query 应可解析");
        assert!(
            mingli_app::validate_query(&q).is_err(),
            "用例层应拒绝{what}——wasm 那扇门只经由它收口，这里放过就是浏览器里放过"
        );
    }
}
