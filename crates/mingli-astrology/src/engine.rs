//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, s, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query, SchoolItem};
use serde_json::Value;

/// 西洋占星本命盘叶（B 族）。仅 `astrology` feature 开启时编译（连带 VSOP87 星历）。
#[derive(Debug, Default)]
pub struct AstrologyEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(e: &AstrologyEngine, m: &Moment, q: &Query) -> crate::NatalChart {
    let geo = match (q.latitude, q.longitude) {
        (Some(latitude), Some(longitude)) => Some(crate::GeoLocation {
            latitude,
            longitude,
        }),
        _ => None,
    };
    let house_system = crate::HouseSystem::from_id(q.school_of(e.id(), "placidus"));crate::compute_at(m, geo, house_system)
}

impl CastingEngine for AstrologyEngine {
    fn id(&self) -> &'static str {
        "astrology"
    }
    fn name(&self) -> &'static str {
        "西洋占星"
    }
    fn family(&self) -> Family {
        Family::Angular
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn answers(&self) -> &'static [Intent] {
        // 「运」要行运（transit / 推运 / 太阳返照），「合」要两盘之间的几何关系，
        // 本叶两者都还没有——只有本命盘与盘内相位。「群/国」用的是立国盘，那是本命盘的一种用法，
        // 同一份计算即可，故答。
        &[Intent::Natal, Intent::Mundane]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 太阳所在星座。
        let c = chart(self, m, q);
        Some(Principal { label: "太阳星座", value: c.planets.first().map_or_else(String::new, |p| p.sign.clone()) })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d(
                "行运（transit / 推运 / 太阳返照，未实现）",
                Und,
                "🟡 这不是定不下，是还没做——星历层已能给任意时刻的行星位置，缺的是「与本命盘比对」\
                 这一步及各法的容许度约定。本叶只出本命盘与盘内相位，故不认领「运」这一类问局",
            ),
            d(
                "两盘比对（合盘几何相位，未实现）",
                Und,
                "🟡 同上，是还没做——两盘之间的相位算法与盘内相同，缺的是双盘接口与各家对\
                 「哪些相位入合盘」的取舍。故本叶不认领「合」这一类问局",
            ),
            d("行星落座·相位", Det, "VSOP87 视黄经，太阳校验 Meeus 0.02°"),
            d("月亮落座", Det, "ELP-2000/82 (astro crate)，校验 Meeus 47.a < 5″ 与 Diana(AA) < 0.2°"),
            d("Asc/MC", Det, "平恒星时+平交角，校验 Diana(AA) < 0.5°"),
            d("分宫制(Placidus/Koch/WholeSign/Equal/Porphyry)", Det, "Placidus/Koch 移植 swehouse.c+Diana 12 cusp<0.05°(pyswisseph oracle)；极区回落 Porphyry"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("placidus", "Placidus 半弧三分", true, "占星圈业界默认；移植 Swiss swehouse.c；极区(|φ|≥66.5°)回落 Porphyry"),
            s("koch", "Koch 等赤经四分", false, "Walter Koch 1962；移植 Swiss swehouse.c 'K' case；极区(|φ|≥66.5°)回落 Porphyry"),
            s("whole_sign", "整宫制 Whole Sign", false, "古典占星与希腊占星派常用；一宫=一星座；极区可用"),
            s("equal", "Equal 等宫", false, "从上升起每 30° 一宫；MC 不作 10 宫尖；极区可用"),
            s("porphyry", "Porphyry 黄道三分", false, "1/10/4/7=Asc/MC/IC/DC；中间宫尖在黄道弧上三分；极区可用"),
        ] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 坐标是本叶的可选原子：给了就出 Asc/MC 与宫位，没给就只出行星落座。
    /// 两条路都得走一遍——缺坐标时的降级路径与带坐标时的完整路径同样是契约的一部分。
    #[test]
    fn coordinates_are_optional_and_both_paths_hold() {
        let e = AstrologyEngine;
        let m = Moment::new(1961, 7, 1, 19, 45, 1.0);
        let mut q = Query::at(1961, 7, 1, 19, 45, 1.0);

        let without = e.cast(&m, &q);
        assert!(without["angles"].is_null(), "没有坐标就不该凭空出 Asc/MC");
        assert!(without["planets"].is_array(), "行星落座不依赖坐标");

        q.latitude = Some(52.833);
        q.longitude = Some(0.500);
        let with = e.cast(&m, &q);
        assert_eq!(with["angles"]["asc_sign"], "射手", "Diana 上升在射手");
        assert_eq!(with["cusp_system"], "placidus", "缺省分宫制");
        // 单给一边不算给：纬度经度必须成对。
        let mut half = Query::at(1961, 7, 1, 19, 45, 1.0);
        half.latitude = Some(52.833);
        assert!(e.cast(&m, &half)["angles"].is_null(), "只给纬度不成对，应走降级路径");
    }

    /// 适配器把本叶接到统一契约上：元数据齐备、能出盘、确定性谱已声明、
    /// 有流派时恰有一个默认。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = AstrologyEngine;
        assert!(!e.id().is_empty() && !e.name().is_empty());
        let m = Moment::new(1990, 6, 15, 14, 30, 8.0);
        let q = Query::at(1990, 6, 15, 14, 30, 8.0);
        assert!(!e.cast(&m, &q).is_null(), "每片叶都应产出非空盘面");
        assert!(!e.profile().is_empty(), "每片叶都要显式声明确定性谱");
        let defaults = e.schools().iter().filter(|s| s.default).count();
        assert!(e.schools().is_empty() || defaults == 1, "有流派的叶应恰有一个默认");
        assert!(!e.family().label().is_empty());
    }
}
