//! 寻（方位）用例：问事此刻起课，收齐各叶给出的**方位候选**。
//!
//! 只出结构：哪个要素落在哪一方。**所寻之事取哪一宫为用、哪一方为吉**是判读，各家不同，交释义层。
//!
//! 本模块**不解析任何叶的盘面**——候选由各叶经端口方法 [`CastingEngine::bearings`] 自己给出。
//! 从前这里把奇门与六壬的整张盘从 JSON 里重新解析一遍（三十九处按字符串键取值），
//! 那样叶改个字段名不会编译报错、只会静默少出候选，而且把三吉门与三奇这两组
//! 奇门自家的判据在用例层又抄了一份字面量。现在判据留在叶里，用例层只做编排。

use mingli_contract::{AskTime, Bearing, CastingEngine, LeafOutput, Query, QueryKind};
use serde::Serialize;
use std::collections::BTreeMap;

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
    // 候选由各叶自报——本层不认识任何一种盘。
    let m = mingli_astro::Moment::new(t.year, t.month, t.day, t.hour, t.minute, t.tz);
    let bearings: Vec<Bearing> = ids
        .iter()
        .filter_map(|id| reg.iter().find(|e| e.id() == *id))
        .flat_map(|e| e.bearings(&m, &q))
        .collect();
    Ok(Locative { asked_at: t.clone(), seed, category, bearings, leaves })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mingli_registry::registry;

    fn t() -> AskTime {
        AskTime { year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0 }
    }

    /// 用例层不再自带方位表——十二支的方位归六壬叶所有，这里只确认取到的是同一张。
    #[test]
    fn the_branch_directions_come_from_the_leaf_that_owns_them() {
        use mingli_liuren::BRANCH_DIR;
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
        // 次序即注册表次序（从前是端口层手写表的次序）
        assert_eq!(ids, ["xiaoliuren", "liuren", "qimen"]);
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
