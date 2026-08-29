//! 编排层：把命理大树当作一张记忆化计算 DAG 来跑。
//!
//! - 共享层：一个输入 → 用 [`Moment`] 把公共天文/历法子计算**算一次**。
//! - fan-out：注册表里每片叶（[`CastingEngine`]）在该共享上下文上排盘。native 走 rayon 并行；
//!   wasm32 串行，那里没有可用线程，把 rayon 链进去只是白付 37 KB。
//! - 统一输出：各叶输出 `serde_json::Value`，便于跨叶对齐比较。
//!
//! 本层**不认识任何具体叶**——注册表由调用方注入（见 `mingli-registry`）。
//! 加一片新叶不需要改动这里的任何一行。

use mingli_contract::{effective_school_id, intents, CastingEngine, IntentSpec, LeafOutput, Moment, Query, QueryKind};
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use serde_json::Value;
use std::collections::BTreeMap;

/// 注册表：一组待 fan-out 的叶。
pub type Leaves = [Box<dyn CastingEngine>];

/// 把每片叶映射成一个结果。native 上并行，wasm 上串行。
///
/// wasm32 没有可用线程，rayon 在那里只是把串行遍历绕过一层调度器，
/// 并且**实测在排盘档里占 37 KB**——易经单叶档一共才 197 KB。
/// native 那边它是真的：整棵树并行 445 µs，其中最慢的一片占 297 µs。
#[cfg(not(target_arch = "wasm32"))]
fn map_leaves<T: Send>(reg: &Leaves, f: impl Fn(&Box<dyn CastingEngine>) -> T + Send + Sync) -> Vec<T> {
    reg.par_iter().map(f).collect()
}

/// 同上，wasm32 一侧：串行，不引 rayon。
#[cfg(target_arch = "wasm32")]
fn map_leaves<T>(reg: &Leaves, f: impl Fn(&Box<dyn CastingEngine>) -> T) -> Vec<T> {
    reg.iter().map(f).collect()
}

/// 一个输入 → 共享层算一次 → **并行**排所有叶 → `id → 盘(JSON)`。
#[must_use]
pub fn cast_all(reg: &Leaves, q: &Query) -> BTreeMap<String, Value> {
    let m = shared_moment(q);
    map_leaves(reg, |e| (e.id().to_string(), e.cast(&m, q))).into_iter().collect()
}

/// 只算**单片**叶（按 id）——共享层仍只算一次，但仅排该叶（释义/单叶请求用，省去其余叶）。
/// 未知 id 返回 `None`。
#[must_use]
pub fn cast_one(reg: &Leaves, id: &str, q: &Query) -> Option<LeafOutput> {
    let e = reg.iter().find(|e| e.id() == id)?;
    let m = shared_moment(q);
    Some(leaf_output(e.as_ref(), &m, q))
}

/// 同 [`cast_all`]，但保留注册表**顺序**并附带每叶元数据（id/name/family/确定性谱/流派）。
#[must_use]
pub fn cast_all_detailed(reg: &Leaves, q: &Query) -> Vec<LeafOutput> {
    let m = shared_moment(q);
    map_leaves(reg, |e| leaf_output(e.as_ref(), &m, q))
}

/// 把一个问局意图路由到具体的叶 id 列表。
///
/// 问的是每片叶自己：谁在 [`CastingEngine::answers`] 里认领了这一类，谁就入选。
/// 从前这里读的是端口层写死的一张「意图 → 叶 id」表，那张表让端口层知道了树上有哪些叶，
/// 也让「加一片叶只动装配根」在字符串层面不成立——漏改那张表，新叶不入任何路由且不报错。
///
/// 次序即注册表次序。feature flag 关掉的叶不在注册表里，自然也不在结果里。
#[must_use]
pub fn route(reg: &Leaves, kind: &QueryKind) -> Vec<&'static str> {
    let want = kind.intent();
    reg.iter().filter(|e| e.answers().contains(&want)).map(|e| e.id()).collect()
}

/// 意图清单与各意图当前实际路由到的叶。
///
/// [`intents`] 只说这一类问局是什么、要哪些输入原子；「谁来答」要问注册表里的叶。
/// 两者在这里合成一份，供承接层直接展示。
#[must_use]
pub fn intent_catalog(reg: &Leaves) -> Vec<(&'static IntentSpec, Vec<&'static str>)> {
    intents()
        .iter()
        .map(|spec| {
            let leaves = reg.iter().filter(|e| e.answers().contains(&spec.id)).map(|e| e.id()).collect();
            (spec, leaves)
        })
        .collect()
}

/// 共享上下文：一次输入只构造一个 [`Moment`]，全叶复用（记忆化的落点）。
fn shared_moment(q: &Query) -> Moment {
    Moment::new(q.year, q.month, q.day, q.hour, q.minute, q.tz)
}

fn leaf_output(e: &dyn CastingEngine, m: &Moment, q: &Query) -> LeafOutput {
    LeafOutput {
        id: e.id(),
        name: e.name(),
        family: e.family(),
        family_label: e.family().label(),
        profile: e.profile(),
        schools: e.schools(),
        effective_school: effective_school_id(e, q),
        chart: e.cast(m, q),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mingli_contract::{d, s, DetItem, Determinism, Family, SchoolItem};

    /// 假叶甲：带确定性谱与两个流派，用来验元数据透传与流派落默认。
    #[derive(Debug, Default)]
    struct Alpha;
    impl CastingEngine for Alpha {
        fn id(&self) -> &'static str {
            "alpha"
        }
        fn name(&self) -> &'static str {
            "假叶甲"
        }
        fn family(&self) -> Family {
            Family::Cyclic
        }
        fn cast(&self, m: &Moment, q: &Query) -> Value {
            serde_json::json!({ "jdn": m.civil_day, "year": q.year })
        }
        fn profile(&self) -> &'static [DetItem] {
            const { &[d("假谱", Determinism::Det, "测试用")] }
        }
        fn schools(&self) -> &'static [SchoolItem] {
            const { &[s("one", "甲流派", true, "默认"), s("two", "乙流派", false, "备选")] }
        }
    }

    /// 假叶乙：只实现必需项，用来验 trait 默认（空谱、空流派）。
    #[derive(Debug, Default)]
    struct Beta;
    impl CastingEngine for Beta {
        fn id(&self) -> &'static str {
            "beta"
        }
        fn name(&self) -> &'static str {
            "假叶乙"
        }
        fn family(&self) -> Family {
            Family::Sampling
        }
        fn cast(&self, _m: &Moment, _q: &Query) -> Value {
            Value::Null
        }
    }

    fn fake_registry() -> Vec<Box<dyn CastingEngine>> {
        vec![Box::new(Alpha), Box::new(Beta)]
    }

    fn sample() -> Query {
        Query {
            year: 1990,
            month: 6,
            day: 15,
            hour: 14,
            minute: 30,
            tz: 8.0,
            gender: None,
            latitude: None,
            longitude: None,
            seed: None,
            name: None,
            schools: BTreeMap::new(),
        }
    }

    #[test]
    fn cast_all_covers_the_injected_registry() {
        let out = cast_all(&fake_registry(), &sample());
        assert_eq!(out.len(), 2);
        assert_eq!(out["alpha"]["year"], 1990);
        assert_eq!(out["beta"], Value::Null);
    }

    #[test]
    fn cast_one_selects_and_rejects_unknown() {
        let reg = fake_registry();
        let q = sample();
        let one = cast_one(&reg, "alpha", &q).expect("alpha 应在注册表内");
        assert_eq!(one.name, "假叶甲");
        assert_eq!(one.chart, cast_all(&reg, &q)["alpha"]);
        assert!(cast_one(&reg, "nope", &q).is_none());
    }

    #[test]
    fn detailed_preserves_order_and_carries_metadata() {
        let out = cast_all_detailed(&fake_registry(), &sample());
        assert_eq!(out.iter().map(|l| l.id).collect::<Vec<_>>(), ["alpha", "beta"]);
        assert_eq!(out[0].family_label, "循环群/CRT");
        assert_eq!(out[0].profile.len(), 1);
        // 未指定流派 → 落到该叶 default；无流派的叶 → 空串
        assert_eq!(out[0].effective_school, "one");
        assert_eq!(out[1].effective_school, "");
        assert!(out[1].profile.is_empty() && out[1].schools.is_empty());
    }

    #[test]
    fn explicit_school_overrides_default() {
        let mut q = sample();
        q.schools.insert("alpha".to_string(), "two".to_string());
        let out = cast_all_detailed(&fake_registry(), &q);
        assert_eq!(out[0].effective_school, "two");
    }

    #[test]
    fn shared_moment_is_computed_once_per_call() {
        // 同一输入下两次 fan-out 结果一致 —— 共享层是纯函数，可安全复用。
        let reg = fake_registry();
        assert_eq!(cast_all(&reg, &sample()), cast_all(&reg, &sample()));
    }

    #[test]
    fn route_natal_returns_whole_registry_in_order() {
        let reg = fake_registry();
        let ids = route(&reg, &QueryKind::Natal(sample()));
        assert_eq!(ids, ["alpha", "beta"]);
    }

    #[test]
    fn route_non_natal_intersects_with_registry() {
        // 假注册表里没有真叶，所以任何非 Natal 意图都路由到空集——
        // 「声明的默认叶 ∩ 实际装配的叶」这条规则本身即被验证。
        let reg = fake_registry();
        let kind = QueryKind::Election {
            window_start: AskTime { year: 2026, month: 1, day: 1, hour: 0, minute: 0, tz: 8.0 },
            window_end: AskTime { year: 2026, month: 1, day: 8, hour: 0, minute: 0, tz: 8.0 },
            category: "婚".to_string(),
        };
        assert!(route(&reg, &kind).is_empty());
    }

    use mingli_contract::AskTime;
}
