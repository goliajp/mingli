//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query};
use serde_json::Value;

/// 小六壬叶（A 族·时间起课，确定性）。月→日→时辰在 Z₆ 上掐指。
#[derive(Debug, Default)]
pub struct XiaoliurenEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(_e: &XiaoliurenEngine, m: &Moment, _q: &Query) -> crate::Cast {
crate::compute_at(m)
}

impl CastingEngine for XiaoliurenEngine {
    fn id(&self) -> &'static str {
        "xiaoliuren"
    }
    fn name(&self) -> &'static str {
        "小六壬"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn reading_notes(&self) -> Option<&'static str> {
        Some("\n【字段语义提示（小六壬，掐指三步）】\n\
            - 三步连掐：`lunar_month` → `month_pos` / `month_deity`，接着 `lunar_day` → `day_pos` / `day_deity`，\
              再 `hour_branch` → `hour_pos` / `hour_deity`。**位次自上一步的落点续数**，不是各自独立起算。\n\
            - 六神各义：大安主安稳不动、留连主迟滞反复、速喜主快得好音、赤口主口舌争执、\
              小吉主小有所得、空亡主落空无着。\n\
            - `hour_deity` 是末位，通行以它为断；`month_deity` / `day_deity` 是过程。\n\
            - `month_direction` / `day_direction` / `hour_direction`：该神所配之方。\
              大安东、留连北、速喜南、赤口西；**落在小吉或空亡时不给值**——\
              前者各家三说不一、后者配「中」而中宫不是可面向之方（见确定性谱）。\
              故本叶不答「寻」这一类问局，方位只作盘面事实随出。\n\
            - **读法**：直接说末位所落之神，再以前两位补一句过程即可，篇幅宜短。")
    }
    fn answers(&self) -> &'static [Intent] {
        // 只答「命」。「择」要的是按吉凶分档的候选日，本叶给的是某一时辰落在六神的哪一位，
        // 不是那个形态；「寻」要方位候选，而本叶没有实现 `bearings`——路由到它只会排一张盘、
        // 一个候选都不出。六神传统上确有方位之说，但那是「还没做」。
        &[Intent::Natal]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 时辰落在六神的哪一位。
        let c = chart(self, m, q);
        Some(Principal { label: "时神位", value: c.hour_pos.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const {
            &[
                d("六神掐指（月→日→时）", Det, "Z₆ 连续位移，六神为定义性有序环"),
                d(
                    "六神配方位（四定二不定）",
                    Und,
                    "🟡 查过了：四个定得下且随盘面出（`*_direction`）：大安木·东·青龙、留连水·北·玄武、\
                     速喜火·南·朱雀、赤口金·西·白虎——五行、方位、四象三者自洽且多源同述。\
                     两个定不下：**空亡**配「中」，而中宫不是可面向之方（本仓库在奇门那边同理处理过——\
                     值符落中五宫按「中 5 寄坤 2」归并，不出「朝中间」这种候选；小六壬无对应的寄宫之说可援）；\
                     **小吉**三说并存——一系作属水·北（与留连同方）、一系作属木而不给方位、\
                     其口诀又作「失物在坤方」（西南），且《小六壬理论知识详解》一路明记「不同的文献来源\
                     对小吉的方位属性描述有所不同，这反映了小六壬占卜法在民间传承中的多个版本」，\
                     三说无一取得两个独立源。**正因六分之二给不出方位，本叶不认领「寻」**：\
                     一次掐指落在哪个神不由人，那就不是「算得出这一类的 output_shape」",
                ),
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到统一契约上：元数据齐备、能出盘、确定性谱已声明、
    /// 有流派时恰有一个默认。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = XiaoliurenEngine;
        assert!(!e.id().is_empty() && !e.name().is_empty());
        let m = Moment::new(1990, 6, 15, 14, 30, 8.0);
        let q = Query::at(1990, 6, 15, 14, 30, 8.0);
        assert!(!e.cast(&m, &q).is_null(), "每片叶都应产出非空盘面");
        assert!(!e.profile().is_empty(), "每片叶都要显式声明确定性谱");
        let defaults = e.schools().iter().filter(|s| s.default).count();
        assert!(e.schools().is_empty() || defaults == 1, "有流派的叶应恰有一个默认");
    }
}
