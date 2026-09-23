//! 读法提示点名的字段，盘上必须真有。
//!
//! `reading_notes()` 是写给释义层的说明书：它用反引号点名盘面字段，告诉读者每个字段是什么。
//! 说明书提到一个盘上没有的字段，释义层就会去找一个不存在的东西——
//! 找不到时它不会报错，只会自己补一个。
//!
//! 这里只查「这个名字在盘上某处出现过」，不查它挂在哪一层。七政四余的说明书曾写
//! 「每颗星带 `mansion` 所值之宿」，而 `mansion` 只在盘面顶层、不在星的条目里——
//! 那一类错这条抓不到，说明书里的层级是散文，没法机械解析。

use mingli_contract::{CastingEngine, Gender, Moment, Query};
use mingli_registry::registry;
use serde_json::Value;
use std::collections::BTreeSet;

/// 收齐一片叶在各种输入下可能出现的全部键名。
///
/// 有些字段只在带了性别、坐标、种子或选了某个流派时才出现（八字的大运、占星的四轴），
/// 所以要把这些输入都走一遍，取并集。
fn every_key(e: &dyn CastingEngine) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let mut full = Query::at(1990, 6, 15, 14, 30, 8.0);
    full.gender = Some(Gender::Female);
    full.latitude = Some(31.23);
    full.longitude = Some(121.47);
    full.seed = Some(123);
    full.name = Some("Ada Lovelace".to_string());
    let mut queries = vec![Query::at(1990, 6, 15, 14, 30, 8.0), full.clone()];
    for s in e.schools() {
        let mut q = full.clone();
        q.schools.insert(e.id().to_string(), s.id.to_string());
        queries.push(q);
    }
    for q in &queries {
        let m = Moment::new(q.year, q.month, q.day, q.hour, q.minute, q.tz);
        collect_keys(&e.cast(&m, q), &mut keys);
    }
    keys
}

fn collect_keys(v: &Value, keys: &mut BTreeSet<String>) {
    match v {
        Value::Object(map) => {
            for (k, child) in map {
                keys.insert(k.clone());
                collect_keys(child, keys);
            }
        }
        Value::Array(items) => items.iter().for_each(|child| collect_keys(child, keys)),
        _ => {}
    }
}

/// 说明书里用反引号点名的、长得像字段名的记号。
///
/// 只收小写蛇形名（可带 `[]` 表示数组）：宫名、星名、公式这类反引号内容不是字段，不收。
fn named_fields(notes: &str) -> Vec<String> {
    notes
        .split('`')
        .skip(1)
        .step_by(2)
        .map(|s| s.trim_end_matches("[]"))
        .filter(|s| {
            !s.is_empty()
                && s.starts_with(|c: char| c.is_ascii_lowercase())
                && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        })
        .map(str::to_string)
        .collect()
}

#[test]
fn every_field_a_reading_note_names_is_on_the_chart() {
    let mut checked = 0;
    let mut missing = Vec::new();
    for e in &registry() {
        let Some(notes) = e.reading_notes() else { continue };
        let keys = every_key(e.as_ref());
        for f in named_fields(notes) {
            checked += 1;
            if !keys.contains(&f) {
                missing.push(format!("{}：`{f}`", e.id()));
            }
        }
    }
    // 量具自己得能红：扫到的名字少了，说明提取规则或注册表坏了，不是「全都对」。
    // 实测（2026-09-23）21 片叶的说明书共点名 232 个字段。
    assert!(checked >= 200, "只扫到 {checked} 个字段名，提取规则或注册表出了问题");
    assert!(
        missing.is_empty(),
        "读法提示点名了盘上没有的字段：\n  {}",
        missing.join("\n  ")
    );
}
