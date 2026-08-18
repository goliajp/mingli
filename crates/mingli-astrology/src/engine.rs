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
    fn reading_notes(&self) -> Option<&'static str> {
        Some("\n【字段语义提示（西洋占星本命盘，读懂后给出有据的性格 / 领域倾向判读）】\n\
            - `planets[]`：每颗行星的 `longitude`（黄经）、`sign`（所落星座）、`degree`（座内度数）、\
              `house`（所落宫位 1..12）。太阳主自我与生命力、月亮主情绪与内在需求、水星主思维沟通、\
              金星主关系与审美、火星主行动与冲劲、木星主扩张与机遇、土星主限制与责任，\
              天王 / 海王 / 冥王三颗外行星走得慢，主世代性主题而非个人特质。\n\
            - `houses[12]`：十二宫及其内行星。一宫自我、二宫资源、三宫沟通、四宫家宅、五宫创造、\
              六宫日常与健康、七宫伴侣、八宫共有与转化、九宫远行与学问、十宫事业、\
              十一宫社群、十二宫潜隐。**行星落宫说的是这份能量用在哪个领域**。\n\
            - `angles`：`ascendant` / `asc_sign` 上升（呈现于外的样子）、`midheaven` / `mc_sign` 中天（事业与公众面）。\
              两者依出生地与时刻而定，缺坐标时不出。\n\
            - `aspects[]`：行星间的角度关系，`kind` 为合 / 冲 / 拱 / 刑 / 六分。\
              合主融合、拱与六分主顺畅、冲与刑主张力——**张力不等于坏**，是需要处理的动力。\n\
            - `cusp_system`：宫位制（本盘所用）。各制分宫线不同，同一行星可能落在不同宫，\
              这是流派差异不是误差，见确定性谱。\n\
            - **读法**：先看太阳 / 月亮 / 上升三者的星座，再看最紧的两三个相位，\
              最后看行星集中在哪几宫；挑最值得一说的几处，不必逐颗铺陈。")
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
                "两盘之间的相位（几何）",
                Det,
                "与盘内相位同一套判定：两个黄经的夹角落在相位角的容许度内即成。\
                 不同的只是两个黄经来自两张盘，故每条都带主宾（甲的某星对乙的某星），全矩阵而非上三角",
            ),
            d(
                "合盘取哪些相位",
                Und,
                "🟡 各家出入很大：有只取日月金火土的、有把外行星一律排除的、有按星体分别定容许度的。\
                 本叶只出几何（默认容许度下的全量），选哪些交释义层，不代为取舍。\
                 也因此本叶仍不认领「合」这一类问局——出几何不等于出「配」这个形态",
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
