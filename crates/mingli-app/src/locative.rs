//! 寻（方位）用例：问事此刻起课，把奇门与六壬盘上的「宫 / 支」翻成**方位候选**。
//!
//! 只出结构：奇门各要素落在哪一方（值符 / 值使 / 三吉门 / 三奇 / 各宫旺衰），六壬发用之支
//! 指向哪一方。**所寻之事取哪一宫为用、哪一方为吉**是判读，各家不同，交释义层。
//!
//! 方位表复用主干层的后天八卦九宫方位（[`mingli_luoshu::PALACE_DIR`]）与十二支方位。

#![allow(
    clippy::cast_possible_truncation,
    reason = "宫号 1..=9 与地支 0..=11 都是个位数下标，u64 → usize 不会截断"
)]

use mingli_contract::{AskTime, CastingEngine, LeafOutput, Query, QueryKind};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// 十二地支所指方位（子北起顺时针）：子北 · 丑寅东北 · 卯东 · 辰巳东南 · 午南 · 未申西南 · 酉西 · 戌亥西北。
pub const BRANCH_DIR: [&str; 12] =
    ["北", "东北", "东北", "东", "东南", "东南", "南", "西南", "西南", "西", "西北", "西北"];

/// 一个方位候选：来源要素 → 落宫 → 方位。
#[derive(Debug, Clone, Serialize)]
pub struct Bearing {
    /// 来源叶。
    pub leaf: &'static str,
    /// 要素名（值符 / 值使 / 开门 / 乙奇 / 发用 …）。
    pub element: String,
    /// 落宫（奇门 1..=9）或地支（六壬 0..=11）的字面。
    pub at: String,
    /// 方位。
    pub direction: &'static str,
    /// 附注（旺衰 / 门 / 神等同宫结构，供判读）。
    pub note: String,
}

/// 一次寻方位的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Locative {
    /// 问事此刻。
    pub asked_at: AskTime,
    /// 取机种子。
    pub seed: Option<u64>,
    /// 所寻（人 / 物 / 向；只入释义）。
    pub category: Option<String>,
    /// 方位候选（奇门在前、六壬在后，皆按各自盘面顺序）。
    pub bearings: Vec<Bearing>,
    /// 参与的叶原盘（供释义层与前端）。
    pub leaves: Vec<LeafOutput>,
}

fn s(v: &Value) -> String {
    v.as_str().unwrap_or("").to_string()
}

fn palace_dir(p: u64) -> &'static str {
    mingli_luoshu::PALACE_DIR.get(p as usize).copied().unwrap_or("")
}

fn palace_name(chart: &Value, p: u64) -> String {
    chart["palace"]
        .get(p as usize - 1)
        .and_then(Value::as_str)
        .map_or_else(String::new, |n| format!("{n}{p}"))
}

/// 从奇门盘抽方位候选：值符宫、值使宫、三吉门所在宫、三奇所在宫。
fn qimen_bearings(chart: &Value) -> Vec<Bearing> {
    let mut out = Vec::new();
    let leaf = "qimen";
    let note_of = |p: u64| -> String {
        let k = p as usize - 1;
        let star = s(&chart["sky"]["stars"][k]);
        let vigor = s(&chart["star_vigor"][k]);
        let gate = s(&chart["gates"]["gates"][k]);
        let spirit = s(&chart["spirits"]["spirits"][k]);
        let stem = s(&chart["sky"]["stems"][k]);
        format!("{star}{vigor} · {gate} · {spirit} · 天盘{stem}")
    };
    if let Some(p) = chart["zhi_fu_palace"].as_u64() {
        out.push(Bearing { leaf, element: "值符".into(), at: palace_name(chart, p), direction: palace_dir(p), note: note_of(p) });
    }
    if let Some(p) = chart["gates"]["zhi_shi_palace"].as_u64() {
        out.push(Bearing { leaf, element: "值使".into(), at: palace_name(chart, p), direction: palace_dir(p), note: note_of(p) });
    }
    if let Some(gates) = chart["gates"]["gates"].as_array() {
        for (k, g) in gates.iter().enumerate() {
            let g = s(g);
            if matches!(g.as_str(), "开门" | "休门" | "生门") {
                let p = k as u64 + 1;
                out.push(Bearing { leaf, element: g, at: palace_name(chart, p), direction: palace_dir(p), note: note_of(p) });
            }
        }
    }
    if let Some(stems) = chart["sky"]["stems"].as_array() {
        for (k, st) in stems.iter().enumerate() {
            let st = s(st);
            if matches!(st.as_str(), "乙" | "丙" | "丁") {
                let p = k as u64 + 1;
                out.push(Bearing { leaf, element: format!("{st}奇"), at: palace_name(chart, p), direction: palace_dir(p), note: note_of(p) });
            }
        }
    }
    // 中宫之干寄坤 2 随转，不在 sky.stems 里；若它恰是三奇，方位取其实际落宫
    let center = s(&chart["sky"]["center_stem"]);
    if matches!(center.as_str(), "乙" | "丙" | "丁")
        && let Some(p) = chart["sky"]["center_palace"].as_u64()
    {
        out.push(Bearing {
            leaf,
            element: format!("{center}奇（中宫寄）"),
            at: palace_name(chart, p),
            direction: palace_dir(p),
            note: note_of(p),
        });
    }
    out
}

const BRANCHES: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];

/// 从六壬盘抽方位候选：三传（有则）各支所指之方；无传时给四课上神。
fn liuren_bearings(chart: &Value) -> Vec<Bearing> {
    let mut out = Vec::new();
    let leaf = "liuren";
    if let Some(tr) = chart["transmission"].as_array() {
        for (i, b) in tr.iter().enumerate() {
            if let Some(b) = b.as_u64() {
                let name = ["初传", "中传", "末传"][i.min(2)];
                out.push(Bearing {
                    leaf,
                    element: name.into(),
                    at: BRANCHES[b as usize % 12].into(),
                    direction: BRANCH_DIR[b as usize % 12],
                    note: format!("课式 {}", s(&chart["pattern_label"])),
                });
            }
        }
    } else if let Some(cs) = chart["courses"].as_array() {
        for (i, c) in cs.iter().enumerate() {
            if let Some(up) = c["up"].as_u64() {
                out.push(Bearing {
                    leaf,
                    element: format!("第{}课上神", i + 1),
                    at: BRANCHES[up as usize % 12].into(),
                    direction: BRANCH_DIR[up as usize % 12],
                    note: format!("课式 {} · 🟡 取传流派分歧，未出三传", s(&chart["pattern_label"])),
                });
            }
        }
    }
    out
}

/// 寻方位：按 `Locative` 意图路由（六壬 / 奇门 / 小六壬），在问事此刻起课并抽方位候选。
///
/// # Errors
///
/// 注册表内没有可路由的叶时返回说明。
pub fn cast(
    reg: &[Box<dyn CastingEngine>],
    t: &AskTime,
    seed: Option<u64>,
    category: Option<String>,
) -> Result<Locative, String> {
    let kind = QueryKind::Locative { t_ask: t.clone(), seed: seed.unwrap_or(0), category: category.clone().unwrap_or_default() };
    let ids = mingli_engine::route(reg, &kind);
    if ids.is_empty() {
        return Err("当前注册表内没有可用于寻方位的叶".into());
    }
    let q = Query {
        year: t.year, month: t.month, day: t.day, hour: t.hour, minute: t.minute, tz: t.tz,
        gender: None, latitude: None, longitude: None, seed, name: None, schools: BTreeMap::new(),
    };
    let leaves: Vec<LeafOutput> = ids.iter().filter_map(|id| mingli_engine::cast_one(reg, id, &q)).collect();
    let mut bearings = Vec::new();
    for l in &leaves {
        match l.id {
            "qimen" => bearings.extend(qimen_bearings(&l.chart)),
            "liuren" => bearings.extend(liuren_bearings(&l.chart)),
            _ => {}
        }
    }
    Ok(Locative { asked_at: t.clone(), seed, category, bearings, leaves })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mingli_registry::registry;

    fn t() -> AskTime {
        AskTime { year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0 }
    }

    #[test]
    fn branch_directions_follow_the_compass() {
        assert_eq!(BRANCH_DIR[0], "北");
        assert_eq!(BRANCH_DIR[3], "东");
        assert_eq!(BRANCH_DIR[6], "南");
        assert_eq!(BRANCH_DIR[9], "西");
        // 四维各由两支共享
        assert_eq!((BRANCH_DIR[1], BRANCH_DIR[2]), ("东北", "东北"));
        assert_eq!((BRANCH_DIR[10], BRANCH_DIR[11]), ("西北", "西北"));
    }

    #[test]
    fn routes_to_liuren_qimen_xiaoliuren_only() {
        let l = cast(&registry(), &t(), None, None).expect("应可寻");
        let ids: Vec<&str> = l.leaves.iter().map(|x| x.id).collect();
        assert_eq!(ids, ["liuren", "qimen", "xiaoliuren"]);
    }

    #[test]
    fn qimen_bearings_agree_with_the_reference_chart() {
        // 1987-09-17 15:00：值符宫艮 8 = 东北，值使伤门落巽 4 = 东南（见 qimen 叶测试）
        let l = cast(&registry(), &t(), None, None).expect("应可寻");
        let zf = l.bearings.iter().find(|b| b.element == "值符").expect("应有值符");
        assert_eq!((zf.at.as_str(), zf.direction), ("艮8", "东北"));
        let zs = l.bearings.iter().find(|b| b.element == "值使").expect("应有值使");
        assert_eq!((zs.at.as_str(), zs.direction), ("巽4", "东南"));
        // 三吉门恰三条；三奇恰三条（含寄中宫那一奇）
        assert_eq!(l.bearings.iter().filter(|b| b.element.ends_with('门')).count(), 3);
        assert_eq!(l.bearings.iter().filter(|b| b.element.contains('奇')).count(), 3);
        // 该盘中宫之干丙寄坤 2 随转落离 9 = 南
        let bing = l.bearings.iter().find(|b| b.element.starts_with("丙奇")).expect("应有丙奇");
        assert_eq!((bing.at.as_str(), bing.direction), ("离9", "南"));
        // 乙奇在震 3 = 东，附注里能看到同宫的生门
        let yi = l.bearings.iter().find(|b| b.element == "乙奇").expect("应有乙奇");
        assert_eq!((yi.at.as_str(), yi.direction), ("震3", "东"));
        assert!(yi.note.contains("生门"));
    }

    #[test]
    fn liuren_gives_directions_even_when_transmission_is_undetermined() {
        let l = cast(&registry(), &t(), None, None).expect("应可寻");
        let lr: Vec<&Bearing> = l.bearings.iter().filter(|b| b.leaf == "liuren").collect();
        assert!(!lr.is_empty(), "六壬至少给出四课上神或三传");
        assert!(lr.iter().all(|b| !b.direction.is_empty()));
    }

    #[test]
    fn category_is_echoed_and_does_not_change_the_bearings() {
        let a = cast(&registry(), &t(), Some(3), None).expect("应可寻");
        let b = cast(&registry(), &t(), Some(3), Some("寻物".into())).expect("应可寻");
        assert_eq!(b.category.as_deref(), Some("寻物"));
        let da: Vec<_> = a.bearings.iter().map(|x| (&x.element, x.direction)).collect();
        let db: Vec<_> = b.bearings.iter().map(|x| (&x.element, x.direction)).collect();
        assert_eq!(da, db);
    }
}
