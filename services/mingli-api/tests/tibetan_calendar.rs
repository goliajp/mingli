//! Annual delivery contract: external annual anchors, strict scope, no fake day.
use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

async fn query(query: &str) -> (axum::http::StatusCode, Vec<u8>) {
    let r = mingli_api::router()
        .oneshot(
            Request::builder()
                .uri(format!("/api/calendar/tibetan-year?{query}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.headers()["x-mingli-build-id"], env!("MINGLI_BUILD_ID"));
    let status = r.status();
    (
        status,
        r.into_body().collect().await.unwrap().to_bytes().to_vec(),
    )
}

#[tokio::test]
async fn annual_anchors_and_projection_are_explicit() {
    // Janson Appendix E.1 / cycle tables; these expected values are independent
    // of serialization equivalence, which by itself would not test correctness.
    for (year, element, animal, male, sex, rabjung, within, mewa, colour) in [
        (1984, "Wood", "Rat", true, 1, 16, 58, 7, "Red"),
        (1987, "Fire", "Hare", false, 4, 17, 1, 4, "Green"),
        (2020, "Iron", "Rat", true, 37, 17, 34, 7, "Red"),
        (2024, "Wood", "Dragon", true, 41, 17, 38, 3, "Blue"),
        (2026, "Fire", "Horse", true, 43, 17, 40, 1, "White"),
        (2027, "Fire", "Sheep", false, 44, 17, 41, 9, "Maroon"),
    ] {
        let (status, body) = query(&format!("year={year}")).await;
        assert_eq!(status, 200);
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["year"], year);
        assert_eq!(v["element"], element);
        assert_eq!(v["animal"], animal);
        assert_eq!(v["male"], male);
        assert_eq!(v["sexagenary"], sex);
        assert_eq!(v["rabjung"], rabjung);
        assert_eq!(v["year_in_rabjung"], within);
        assert_eq!(v["mewa"], mewa);
        assert_eq!(v["mewa_color"], colour);
        assert_eq!(v["schema_version"], 1);
        assert_eq!(v["kind"], "tibetan_annual_cycle");
        assert_eq!(v["year_basis"], "cycle_year_label");
        assert_eq!(v["calendar_conversion"], "not_computed");
        assert_eq!(v["method_version"], "tibetan-annual-v1");
        assert_eq!(
            v["source_ids"],
            serde_json::json!(["janson-tibetan-calendar-E1"])
        );
        let mut keys: Vec<_> = v.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort_unstable();
        let mut expected = vec![
            "schema_version",
            "kind",
            "year",
            "year_basis",
            "animal",
            "element",
            "male",
            "sexagenary",
            "rabjung",
            "year_in_rabjung",
            "mewa",
            "mewa_color",
            "calendar_conversion",
            "method_version",
            "source_ids",
        ];
        expected.sort_unstable();
        assert_eq!(
            keys, expected,
            "No day placeholder, date conversion, or personal forecast may leak into annual output"
        );
        assert_eq!(
            body,
            serde_json::to_vec(&mingli_app::calendar::tibetan_year(year).unwrap()).unwrap()
        );
    }
}

#[tokio::test]
async fn reviewed_limits_are_inclusive_and_no_birth_fields_are_accepted() {
    for year in [1900, 2099] {
        assert_eq!(query(&format!("year={year}")).await.0, 200);
    }
    for q in [
        "",
        "year=1899",
        "year=2100",
        "year=-1",
        "year=0",
        "year=2147483648",
        "year=2026.5",
        "year=no",
        "year=2026&year=2027",
        "year=2026&month=1",
        "year=2026&day=1",
        "year=2026&hour=12",
        "year=2026&tz=8",
        "year=2026&gender=male",
        "year=2026&seed=1",
        "year=2026&leaf=tibetan",
    ] {
        assert_eq!(query(q).await.0, 400, "{q}");
    }
    for y in [i32::MIN, 1899, 2100, i32::MAX] {
        assert!(mingli_app::calendar::tibetan_year(y).is_err());
    }
}
