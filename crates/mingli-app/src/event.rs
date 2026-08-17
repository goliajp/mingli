//! 占事用例：问事此刻 + 取机 → 卜筮叶各出一盘。
//!
//! 与本命的分野在**时间的含义**：本命盘切的是出生那一刻，占事盘切的是「问的此刻」；
//! 而且卜筮叶还要一个**取机**动作（摇钱 / 抽牌 / 数蓍），这里以种子表达——同一时刻同一种子
//! 必得同一盘，所以事后可复现、可复核。
//!
//! 问句只入释义、不入算：它不改变任何一片叶的输出。

use mingli_contract::{AskTime, CastingEngine, LeafOutput, Query, QueryKind};
use serde::Serialize;
use std::collections::BTreeMap;

/// 一次占事的结果。
#[derive(Debug, Clone, Serialize)]
pub struct EventCast {
    /// 问事此刻。
    pub asked_at: AskTime,
    /// 取机种子（可复现的凭据）。缺省表示未取机，各叶按问事时刻自行派生。
    pub seed: Option<u64>,
    /// 问句（只入释义，不参与计算）。
    pub question: Option<String>,
    /// 参与本次占事的叶（由意图路由决定，与注册表取交集）。
    pub leaves: Vec<LeafOutput>,
}

/// 把问事此刻与取机种子铺成一次共享查询。
///
/// 占事不需要性别 / 坐标 / 姓名——那些是本命的输入原子。
fn event_query(t: &AskTime, seed: Option<u64>) -> Query {
    Query {
        year: t.year,
        month: t.month,
        day: t.day,
        hour: t.hour,
        minute: t.minute,
        tz: t.tz,
        gender: None,
        latitude: None,
        longitude: None,
        seed,
        name: None,
        schools: BTreeMap::new(),
    }
}

/// 占事：按 `Event` 意图路由到卜筮诸叶，在同一时刻同一种子上各排一盘。
///
/// # Errors
///
/// 当前注册表里一片可路由的叶都没有时返回错误（例如把卜筮叶全 feature-gate 掉了）。
pub fn cast(
    reg: &[Box<dyn CastingEngine>],
    t: &AskTime,
    seed: Option<u64>,
    question: Option<String>,
) -> Result<EventCast, String> {
    // 路由只看意图种类，与种子取值无关；未取机时以 0 占位参与路由。
    let kind = QueryKind::Event { t_ask: t.clone(), seed: seed.unwrap_or(0), q_text: question.clone() };
    let ids = mingli_engine::route(reg, &kind);
    if ids.is_empty() {
        return Err("当前注册表内没有可用于占事的叶".into());
    }
    let q = event_query(t, seed);
    let leaves = ids.iter().filter_map(|id| mingli_engine::cast_one(reg, id, &q)).collect();
    Ok(EventCast { asked_at: t.clone(), seed, question, leaves })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mingli_registry::registry;

    fn t() -> AskTime {
        AskTime { year: 2026, month: 8, day: 16, hour: 20, minute: 0, tz: 8.0 }
    }

    #[test]
    fn routes_to_the_divination_leaves_only() {
        let reg = registry();
        let ev = cast(&reg, &t(), Some(2024), None).expect("默认注册表应能占事");
        let ids: Vec<&str> = ev.leaves.iter().map(|l| l.id).collect();
        // 次序即注册表次序：从前这里是端口层一张手写表的次序，现在由各叶自己认领、
        // 按注册表列出。集合未变，次序变了——只有一处定次序，好过两处各写一份。
        assert_eq!(ids, ["yijing", "geomancy", "sikidy", "ifa", "tarot", "meihua", "liuren", "qimen"]);
        // 本命专属的叶不该出现在占事里
        assert!(!ids.contains(&"bazi") && !ids.contains(&"ziwei") && !ids.contains(&"astrology"));
    }

    #[test]
    fn same_moment_and_seed_replay_the_same_chart() {
        let reg = registry();
        let a = cast(&reg, &t(), Some(777), None).expect("应可占");
        let b = cast(&reg, &t(), Some(777), None).expect("应可占");
        for (x, y) in a.leaves.iter().zip(b.leaves.iter()) {
            assert_eq!(x.chart, y.chart, "{} 同时刻同种子应复现", x.id);
        }
        // 换种子，抽样类的叶应当变（确定性叶不变，故只要有一片变即可）
        let c = cast(&reg, &t(), Some(778), None).expect("应可占");
        assert!(
            a.leaves.iter().zip(c.leaves.iter()).any(|(x, y)| x.chart != y.chart),
            "换取机种子后抽样叶应出不同的盘"
        );
    }

    #[test]
    fn without_a_draw_the_moment_itself_seeds_the_chart() {
        let reg = registry();
        let a = cast(&reg, &t(), None, None).expect("不取机也应能占");
        let b = cast(&reg, &t(), None, None).expect("不取机也应能占");
        assert!(a.seed.is_none());
        for (x, y) in a.leaves.iter().zip(b.leaves.iter()) {
            assert_eq!(x.chart, y.chart, "{} 同一时刻应复现", x.id);
        }
    }

    #[test]
    fn question_rides_along_without_touching_the_computation() {
        let reg = registry();
        let plain = cast(&reg, &t(), Some(5), None).expect("应可占");
        let asked = cast(&reg, &t(), Some(5), Some("此事成否".into())).expect("应可占");
        assert_eq!(asked.question.as_deref(), Some("此事成否"));
        for (x, y) in plain.leaves.iter().zip(asked.leaves.iter()) {
            assert_eq!(x.chart, y.chart, "问句不得改变任何一片叶的输出");
        }
    }
}
