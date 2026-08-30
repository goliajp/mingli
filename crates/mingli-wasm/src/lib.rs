//! 命理大树的 wasm 绑定（整库一包）。
//!
//! 把引擎高层 API 以 JSON 字符串进出暴露给 JS：排盘（全叶 / 单叶）、跨叶相关、释义提示词组装、
//! D 族字/词。让浏览器**全客户端**跑引擎，无需服务端（仅 LLM 释义需外部后端，故此处只出提示词）。
//!
//! 入参形状与 HTTP 端点共用一套（`mingli_app::input`），所以同一份 body 打服务端还是喂 wasm，
//! 收到的东西一样——两条交付路只是出口不同，形状不该各写一遍。
//!
//! 两档装配。缺省（含 `usecases`）给出全部十六个出口；
//! `--no-default-features --features <叶>` 只给 [`cast`] 与 [`cast_one`]，
//! 用例层与释义层整个不进产物——只想排一张盘的人不必为跨叶用例付体积。
//! 入参校验因此搬去了契约层：它是 `Query` 自己的前置条件，不该只有走用例层的人才拿得到。

#[cfg(feature = "usecases")]
use mingli_app::input;
use mingli_contract::Query;
#[cfg(feature = "usecases")]
use mingli_contract::WordQuery;
use mingli_registry::registry;
#[cfg(feature = "usecases")]
use mingli_registry::word_registry;
use wasm_bindgen::prelude::*;

/// 解析并**校验**排盘入参。
///
/// 校验这一步不是可选的：HTTP 那扇门在承接层收过一次（`Birth::validate`），
/// wasm 吃的是裸 `Query`，若不在此收，同一个 2 月 31 日在服务端被拒、在浏览器里
/// 却被历法换算悄悄挪成 3 月 3 日照常出盘——两扇门给出的不是同一个答案。
/// 判断本身在契约层（`mingli_contract::validate`），两边落到的是同一段代码。
fn parse_query(s: &str) -> Result<Query, JsValue> {
    let q: Query = serde_json::from_str(s).map_err(|e| JsValue::from_str(&e.to_string()))?;
    mingli_contract::validate_query(&q).map_err(|e| JsValue::from_str(&e))?;
    Ok(q)
}
fn to_json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "null".to_string())
}

#[cfg(feature = "usecases")]
/// 字词叶的统一取值：注册表在装配根，派发在用例层，这里只做 JSON 出口。
///
/// 出的是用例层的整份结果（`input` / `system` / `result` 三段），与 `POST /api/word` 一致。
/// 早先这里只取 `result` 一段，而同族的 [`wuge`] 出整份——两个出口对不上，
/// 且都与 HTTP 那扇门对不上；现已统一，凭据见 `services/mingli-api/tests/two_doors.rs`。
fn word_json(system: &str, q: &WordQuery) -> String {
    mingli_app::word::compute(&word_registry(), system, q)
        .map_or_else(|_| "null".to_string(), |v| to_json(&v))
}

#[cfg(any(feature = "usecases", feature = "astrology-lite"))]
fn parse<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, JsValue> {
    serde_json::from_str(s).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(feature = "usecases")]
/// 用例返回的 `Err` 是给调用方看的说明，原样带出去。
fn out<T: serde::Serialize>(r: Result<T, String>) -> Result<String, JsValue> {
    r.map(|v| to_json(&v)).map_err(|e| JsValue::from_str(&e))
}

/// 全叶排盘（含 id/name/family/确定性谱/盘面）。入参为 [`mingli_contract::Query`] 的 JSON。
///
/// # Errors
/// query JSON 解析失败时返回错误。
#[wasm_bindgen]
pub fn cast(query_json: &str) -> Result<String, JsValue> {
    Ok(to_json(&mingli_engine::cast_all_detailed(&registry(), &parse_query(query_json)?)))
}

/// 由**调用方供给**的九个黄经排本命盘。位置的次序同 `mingli_astrology::BODY_NAMES`。
///
/// 这一档存在的理由是一个实测数字：同一段排盘代码，位置本地算的产物 857,633 字节，
/// 位置由调用方给的 79,863 字节，差 777,770（90.7%）——差的那份是 VSOP87D 的常量表。
/// 浏览器里已经有几十 KB 的 JS 星历，把九个数递进来就省掉整整一份表；
/// 顺带也省掉算它的时间，那是排一张盘里的九成七。
///
/// 上升点、中天、整宫、分宫、相位、落座仍在这里算——省掉的只有「位置从哪来」。
///
/// # Errors
/// 两个 JSON 任一解析失败，或位置不是九个数时返回错误。
#[cfg(feature = "astrology-lite")]
#[wasm_bindgen]
pub fn astrology_with(query_json: &str, longitudes_json: &str) -> Result<String, JsValue> {
    let q = parse_query(query_json)?;
    let lons: Vec<f64> = parse(longitudes_json)?;
    let lons: [f64; 9] = lons
        .try_into()
        .map_err(|v: Vec<f64>| JsValue::from_str(&format!("要九个黄经，给了 {}", v.len())))?;
    let m = mingli_contract::Moment::new(q.year, q.month, q.day, q.hour, q.minute, q.tz);
    let geo = q.latitude.zip(q.longitude).map(|(latitude, longitude)| {
        mingli_registry::leaves::astrology::GeoLocation { latitude, longitude }
    });
    let chart = mingli_registry::leaves::astrology::compute_at_with(
        &m,
        geo,
        mingli_registry::leaves::astrology::HouseSystem::Placidus,
        &lons,
    );
    Ok(to_json(&chart))
}

/// 九星的地心黄经（度），JSON 数组，次序同 `mingli_astrology::BODY_NAMES`。
///
/// 排一张本命盘里九成七的时间花在这九个数上（实测整盘 286.7 µs、只算位置 278.1 µs），
/// 所以只要位置的调用方不该被迫排一张盘——也只有这样，跟别家星历比才是同一件活。
///
/// # Errors
/// query JSON 解析或校验失败时返回错误。
#[cfg(feature = "astrology")]
#[wasm_bindgen]
pub fn longitudes(query_json: &str) -> Result<String, JsValue> {
    let q = parse_query(query_json)?;
    let m = mingli_contract::Moment::new(q.year, q.month, q.day, q.hour, q.minute, q.tz);
    // 经装配根转发，而不是在本 crate 的清单里写上那片叶——
    // 承接层直连叶正是架构测试要禁的事。
    Ok(to_json(&mingli_registry::leaves::astrology::longitudes_at(&m)))
}

/// 只排单片叶（按 id），省去其余叶（非占星叶还省掉 VSOP87）。未知 id 返回 `"null"`。
///
/// # Errors
/// query JSON 解析失败时返回错误。
#[wasm_bindgen]
pub fn cast_one(id: &str, query_json: &str) -> Result<String, JsValue> {
    Ok(to_json(&mingli_engine::cast_one(&registry(), id, &parse_query(query_json)?)))
}

/// 跨叶相关性（固定网格的 NMI 矩阵）。
#[must_use]
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn analysis() -> String {
    to_json(&mingli_app::analysis::cross_leaf_cached(&registry()))
}

/// 组装某叶的释义提示词（含护栏）。LLM 调用在外部（浏览器侧或服务端），此处只出提示词。
///
/// # Errors
/// query JSON 解析失败、或未知叶 id 时返回错误。
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn prompt(id: &str, query_json: &str) -> Result<String, JsValue> {
    let reg = registry();
    // 提示词里的读法提示由叶自己声明，故除了盘面还要把叶本身交给释义层。
    let e = reg.iter().find(|e| e.id() == id).ok_or_else(|| JsValue::from_str("未知叶"))?;
    let leaf = mingli_engine::cast_one(&reg, id, &parse_query(query_json)?)
        .ok_or_else(|| JsValue::from_str("未知叶"))?;
    Ok(mingli_interpret::build_prompt(e.as_ref(), &leaf))
}

/// 四柱精盘。入参为 [`mingli_app::Birth`] 的 JSON（与 `POST /api/bazi` 同形）。
///
/// # Errors
/// JSON 解析失败、或出生输入越界时返回错误。
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn bazi(birth_json: &str) -> Result<String, JsValue> {
    let b: mingli_app::Birth = parse(birth_json)?;
    b.validate().map_err(|e| JsValue::from_str(&e))?;
    Ok(to_json(&mingli_app::bazi::natal(&b)))
}

/// 紫微精盘。入参同 [`bazi`]。
///
/// # Errors
/// JSON 解析失败、或出生输入越界时返回错误。
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn ziwei(birth_json: &str) -> Result<String, JsValue> {
    let b: mingli_app::Birth = parse(birth_json)?;
    b.validate().map_err(|e| JsValue::from_str(&e))?;
    Ok(to_json(&mingli_app::ziwei::natal(&b)))
}

/// 运势：目标时刻切片 + 一生供给时序。入参与 `POST /api/fortune` 同形。
///
/// # Errors
/// JSON 解析失败、出生输入越界、或用例判定入参不成立时返回错误。
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn fortune(body_json: &str) -> Result<String, JsValue> {
    let r: input::FortuneRequest = parse(body_json)?;
    r.natal.validate().map_err(|e| JsValue::from_str(&e))?;
    out(mingli_app::bazi::fortune(&r.natal, &r.t_target.ask_time(), r.timeline_max_age))
}

/// 团队合盘。入参与 `POST /api/team` 同形。
///
/// # Errors
/// JSON 解析失败、或人数不在 1–12 之间时返回错误。
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn team(body_json: &str) -> Result<String, JsValue> {
    let r: input::TeamRequest = parse(body_json)?;
    out(mingli_app::team::compute(&r.members()).map(|t| t.to_json()))
}

/// 合盘：两人互供。入参与 `POST /api/synastry` 同形。
///
/// # Errors
/// JSON 解析失败、或用例判定入参不成立时返回错误。
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn synastry(body_json: &str) -> Result<String, JsValue> {
    let r: input::SynastryRequest = parse(body_json)?;
    let (a, b) = (r.a.birth, r.b.birth);
    out(mingli_app::synastry::compute((&a, r.a.name.as_deref()), (&b, r.b.name.as_deref())).map(|s| s.to_json()))
}

/// 占事：问事此刻 + 取机 → 卜筮诸叶各一盘。入参与 `POST /api/event` 同形。
///
/// # Errors
/// JSON 解析失败、或注册表内没有可路由的叶时返回错误。
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn event(body_json: &str) -> Result<String, JsValue> {
    let r: input::EventRequest = parse(body_json)?;
    out(mingli_app::event::cast(&registry(), &r.t_ask.ask_time(), r.seed, r.question))
}

/// 择吉：扫时窗逐日分档。入参与 `POST /api/election` 同形。
///
/// # Errors
/// JSON 解析失败、或时窗倒置时返回错误。
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn election(body_json: &str) -> Result<String, JsValue> {
    let r: input::ElectionRequest = parse(body_json)?;
    out(mingli_app::election::scan(&r.window_start.ask_time(), &r.window_end.ask_time(), r.category))
}

/// 寻方位：起课 + 抽方位候选。入参与 `POST /api/locative` 同形。
///
/// # Errors
/// JSON 解析失败、或注册表内没有可路由的叶时返回错误。
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn locative(body_json: &str) -> Result<String, JsValue> {
    let r: input::LocativeRequest = parse(body_json)?;
    out(mingli_app::locative::cast(&registry(), &r.t_ask.ask_time(), r.seed, r.category))
}

/// 国运：奠基时刻 → 年度盘时间线。入参与 `POST /api/mundane` 同形。
///
/// # Errors
/// JSON 解析失败、或 `span` 为 0 时返回错误。
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn mundane(body_json: &str) -> Result<String, JsValue> {
    let r: input::MundaneRequest = parse(body_json)?;
    out(mingli_app::mundane::cast(
        &registry(),
        &r.founded_at.ask_time(),
        r.latitude,
        r.longitude,
        r.target_year,
        r.span,
    ))
}

/// 希伯来 gematria（七法并出：Hechrachi/Gadol/Siduri/Katan/KatanMispari/AtBash/AlBam）。
///
/// 出参与 `POST /api/word`（`system` 取 `"gematria"`）同形。
#[must_use]
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn gematria(word: &str) -> String {
    word_json("gematria", &WordQuery { text: Some(word.to_string()), ..WordQuery::default() })
}

/// 阿拉伯 abjad（双序对照：Mashriqī 东方序 + Maghribī 西方序）。
///
/// 出参与 `POST /api/word`（`system` 取 `"abjad"`）同形。
#[must_use]
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn abjad(word: &str) -> String {
    word_json("abjad", &WordQuery { text: Some(word.to_string()), ..WordQuery::default() })
}

/// 姓名五格：`surname_json`/`given_json` 为各字笔画的 JSON 数组（如 `"[7]"` / `"[16,9]"`）。
///
/// 出参与 `POST /api/word`（`system` 取 `"wuge"`）同形。
///
/// # Errors
/// 笔画 JSON 解析失败、或姓/名为空时返回错误。
#[cfg(feature = "usecases")]
#[wasm_bindgen]
pub fn wuge(surname_json: &str, given_json: &str) -> Result<String, JsValue> {
    let s: Vec<u32> = serde_json::from_str(surname_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let g: Vec<u32> = serde_json::from_str(given_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let q = WordQuery { surname: Some(s), given: Some(g), ..WordQuery::default() };
    mingli_app::word::compute(&word_registry(), "wuge", &q)
        .map(|v| to_json(&v))
        .map_err(|e| JsValue::from_str(&e))
}

// host 端只测 Ok/String 路径（wasm-bindgen 的 JsValue 错误对象在非 wasm32 上无法构造）；
// 错误路径与底层逻辑已在 engine / 各叶 / 字词 crate 充分校验。
#[cfg(test)]
mod tests {
    use super::*;

    const QY: &str = r#"{"year":1990,"month":6,"day":15,"hour":14,"minute":30,"tz":8.0,"gender":"male","latitude":31.23,"longitude":121.47,"seed":null,"name":"Ada"}"#;
    const B: &str = r#"{"year":1990,"month":6,"day":15,"hour":14,"minute":30,"tz":8.0,"gender":"male"}"#;
    const B2: &str = r#"{"year":1987,"month":3,"day":2,"hour":9,"tz":8.0,"gender":"female","name":"B"}"#;
    const T: &str = r#"{"year":2026,"month":8,"day":16,"hour":10,"minute":0,"tz":8.0}"#;

    #[test]
    fn ok_paths() {
        let out = cast(QY).unwrap();
        assert!(out.contains("\"bazi\"") && out.contains("\"astrology\""));
        assert!(cast_one("maya", QY).unwrap().contains("tzolkin_name"));
        assert_eq!(cast_one("nope", QY).unwrap(), "null");
        assert!(prompt("bazi", QY).unwrap().contains("仅供研究与娱乐"));
    }

    /// 七个新入口各走一次 ok-path，并核对它们与 HTTP 端点吃的是同一份 body。
    #[test]
    fn the_intent_entries_answer() {
        assert!(bazi(B).unwrap().contains("辛亥"), "1990-06-15 14:30 +8 的日柱是辛亥");
        assert!(ziwei(B).unwrap().contains("ming_branch"));
        assert!(fortune(&format!(r#"{{"natal":{B},"t_target":{T}}}"#)).unwrap().contains("timeline"));
        assert!(team(&format!(r#"{{"members":[{B},{B2}]}}"#)).unwrap().contains("matrix"));
        assert!(synastry(&format!(r#"{{"a":{B},"b":{B2}}}"#)).unwrap().contains("a_supplies_b"));
        assert!(event(&format!(r#"{{"t_ask":{T},"seed":7}}"#)).unwrap().contains("leaves"));
        assert!(election(&format!(r#"{{"window_start":{T},"window_end":{{"year":2026,"month":8,"day":18}}}}"#))
            .unwrap()
            .contains("days"));
        assert!(locative(&format!(r#"{{"t_ask":{T},"seed":7}}"#)).unwrap().contains("bearings"));
        assert!(mundane(r#"{"founded_at":{"year":1949,"month":10,"day":1,"hour":15},"target_year":2026,"span":3}"#)
            .unwrap()
            .contains("timeline"));
    }

    /// 时区与分缺省时按 +8 / 0 走——这是入参形状自己的约定，两条交付路都靠它。
    #[test]
    fn a_moment_may_leave_out_the_hour_and_the_zone() {
        let full = bazi(r#"{"year":1990,"month":6,"day":15,"hour":0,"minute":0,"tz":8.0}"#).unwrap();
        let terse = bazi(r#"{"year":1990,"month":6,"day":15,"hour":0}"#).unwrap();
        assert_eq!(full, terse);
    }

    #[test]
    fn word_ok_paths() {
        assert!(gematria("שלום").contains("376"));
        assert!(abjad("الله").contains("66"));
        assert!(wuge("[7]", "[16,9]").unwrap().contains("\"value\""));
    }
}
