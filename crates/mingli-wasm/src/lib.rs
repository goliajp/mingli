//! 命理大树的 wasm 绑定（整库一包）。
//!
//! 把引擎高层 API 以 JSON 字符串进出暴露给 JS：排盘（全叶 / 单叶）、跨叶相关、释义提示词组装、
//! D 族字/词。让浏览器**全客户端**跑引擎，无需服务端（仅 LLM 释义需外部后端，故此处只出提示词）。
//!
//! 注：本包拉入 `mingli-engine` 全 19 叶（含占星 VSOP87 星历），是「整体」测量基准；

use mingli_engine::Query;
use wasm_bindgen::prelude::*;

fn parse_query(s: &str) -> Result<Query, JsValue> {
    serde_json::from_str(s).map_err(|e| JsValue::from_str(&e.to_string()))
}
fn to_json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "null".to_string())
}

/// 全叶排盘（含 id/name/family/确定性谱/盘面）。入参为 [`mingli_engine::Query`] 的 JSON。
///
/// # Errors
/// query JSON 解析失败时返回错误。
#[wasm_bindgen]
pub fn cast(query_json: &str) -> Result<String, JsValue> {
    Ok(to_json(&mingli_engine::cast_all_detailed(&parse_query(query_json)?)))
}

/// 只排单片叶（按 id），省去其余叶（非占星叶还省掉 VSOP87）。未知 id 返回 `"null"`。
///
/// # Errors
/// query JSON 解析失败时返回错误。
#[wasm_bindgen]
pub fn cast_one(id: &str, query_json: &str) -> Result<String, JsValue> {
    Ok(to_json(&mingli_engine::cast_one(id, &parse_query(query_json)?)))
}

/// 跨叶相关性（固定网格的 NMI 矩阵）。
#[must_use]
#[wasm_bindgen]
pub fn analysis() -> String {
    to_json(&mingli_analysis::cross_leaf(&mingli_analysis::sample_grid(1980, 2009)))
}

/// 组装某叶的释义提示词（含护栏）。LLM 调用在外部（浏览器侧或服务端），此处只出提示词。
///
/// # Errors
/// query JSON 解析失败、或未知叶 id 时返回错误。
#[wasm_bindgen]
pub fn prompt(id: &str, query_json: &str) -> Result<String, JsValue> {
    let leaf = mingli_engine::cast_one(id, &parse_query(query_json)?)
        .ok_or_else(|| JsValue::from_str("未知叶"))?;
    Ok(mingli_interpret::build_prompt(&leaf))
}

/// 希伯来 gematria（七法并出：Hechrachi/Gadol/Siduri/Katan/KatanMispari/AtBash/AlBam）。
#[must_use]
#[wasm_bindgen]
pub fn gematria(word: &str) -> String {
    to_json(&mingli_gematria::compute(word))
}

/// 阿拉伯 abjad（双序对照：Mashriqī 东方序 + Maghribī 西方序）。
#[must_use]
#[wasm_bindgen]
pub fn abjad(word: &str) -> String {
    to_json(&mingli_abjad::compute(word))
}

/// 姓名五格：`surname_json`/`given_json` 为各字笔画的 JSON 数组（如 `"[7]"` / `"[16,9]"`）。
///
/// # Errors
/// 笔画 JSON 解析失败、或姓/名为空时返回错误。
#[wasm_bindgen]
pub fn wuge(surname_json: &str, given_json: &str) -> Result<String, JsValue> {
    let s: Vec<u32> = serde_json::from_str(surname_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let g: Vec<u32> = serde_json::from_str(given_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    if s.is_empty() || g.is_empty() {
        return Err(JsValue::from_str("姓与名笔画至少各一字"));
    }
    Ok(to_json(&mingli_wuge::five_grids(&s, &g)))
}

// host 端只测 Ok/String 路径（wasm-bindgen 的 JsValue 错误对象在非 wasm32 上无法构造）；
// 错误路径与底层逻辑已在 engine / 各叶 / 字词 crate 充分校验。
#[cfg(test)]
mod tests {
    use super::*;

    const QY: &str = r#"{"year":1990,"month":6,"day":15,"hour":14,"minute":30,"tz":8.0,"gender":"Male","latitude":31.23,"longitude":121.47,"seed":null,"name":"Ada"}"#;

    #[test]
    fn ok_paths() {
        let out = cast(QY).unwrap();
        assert!(out.contains("\"bazi\"") && out.contains("\"astrology\""));
        assert!(cast_one("maya", QY).unwrap().contains("tzolkin_name"));
        assert_eq!(cast_one("nope", QY).unwrap(), "null");
        assert!(prompt("bazi", QY).unwrap().contains("仅供研究与娱乐"));
    }

    #[test]
    fn word_ok_paths() {
        assert!(gematria("שלום").contains("376"));
        assert!(abjad("الله").contains("66"));
        assert!(wuge("[7]", "[16,9]").unwrap().contains("\"value\""));
    }
}
