//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, s, CastingEngine, DetItem, Determinism, Family, Moment, Query, SchoolItem};
use serde_json::Value;

/// 契约层性别 → 本叶性别。
fn leaf_gender(g: Option<mingli_contract::Gender>) -> Option<crate::Gender> {
    g.map(|x| match x {
        mingli_contract::Gender::Male => crate::Gender::Male,
        mingli_contract::Gender::Female => crate::Gender::Female,
    })
}

/// 四柱八字叶。
#[derive(Debug, Default)]
pub struct BaziEngine;

impl CastingEngine for BaziEngine {
    fn id(&self) -> &'static str {
        "bazi"
    }
    fn name(&self) -> &'static str {
        "四柱八字"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        use crate::{BaziSchool, YearBreakMethod, ZiHourMethod};
        let school = match q.school_of(self.id(), "late_lichun") {
            "late_sf" => BaziSchool { zi_hour: ZiHourMethod::Late, year_break: YearBreakMethod::SpringFestival },
            "early_lichun" => BaziSchool { zi_hour: ZiHourMethod::Early, year_break: YearBreakMethod::LiChun },
            "early_sf" => BaziSchool { zi_hour: ZiHourMethod::Early, year_break: YearBreakMethod::SpringFestival },
            _ => BaziSchool::default(),
        };
        serde_json::to_value(crate::compute_at_school(m, leaf_gender(q.gender), school))
            .unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("四柱·五行·十神", Det, "干支递推+节气界，校验 庚午/壬午/辛亥/乙未"),
            d("大运", Det, "节气距÷3 起运，阳男阴女顺、阴男阳女逆"),
            d("子时归属（可选）", Det, "晚子（主流）23-24→次日；早子（少数）23-24→当日"),
            d("年柱换岁（可选）", Det, "立春（主流）315°黄经；春节（民间）农历正月初一"),
            d("纳音·空亡", Det, "纳音=干支分组求和，空亡=日柱旬末两支(DET)"),
            d("地支藏干（人元）", Det, "通行「不分日固定表」，多源校验四源一致；🟡 巳/申异说取主流"),
            d("支藏十神", Det, "藏干对日主复用 ten_god；oracle 丁卯己酉己巳壬申 全验"),
            d("十二长生", Det, "传统派：阳干顺、阴干逆；校验各干临官=禄位；🟡 新派阴干顺行未开关"),
            d("旺衰量化（得令/得地/得势）", Und, "🟡 扶抑学派范式：三栏 0-30+ 综合 0-100；权重表无统一标准（各家月令权重 30%-60% 不一），本算法显式声明默认"),
            d("五行力量分布", Und, "🟡 天干 10/地支本 12/中 6/余 3/月支×1.5；权重同旺衰算法，流派分歧"),
            d("岁运叠加旺衰", Und, "🟡 本命基础上拼大运柱+流年柱，得令固定取本命月支；岁运折扣权重流派分歧"),
            d("格局（八正格/禄刃/暗格）", Det, "月令藏干透干：本→中→余，先透先定；月令本气=日主同五行 → 建禄/月刃；7 oracle 含 1987 暗食神/1990 暗七杀+5 构造透干分支"),
            d("从格/化格/专旺格/杂气", Und, "🟡 成立条件流派分歧大（身极弱+无救助、化神有无、月令分日用事），不机械化，留 INT 释义层"),
            d("用神/喜忌（扶抑+调候）", Und, "🟡 身强宜耗（官杀/财/食伤，取盘中最缺者）、身弱宜扶（印星优先）、中和走调候（寒月火/燥月水/春木金/秋金火/杂气日主同）；忌神=反方向。流派（扶抑/调候/通关/病药）无统一先后，本算法显式默认"),
            d("真太阳时校正（可选）", Det, "出生地经度差(±4 min/°)+ Spencer 均时差(±0.5 min) → 校正钟表时；跨时辰边界时时柱变。校验长沙 1987-09-17 15：00 钟表壬申 → 真太阳辛未"),
            d("三宫（命宫/身宫/胎元）", Det, "命宫=（月支−时支）mod12+五虎遁干；身宫=（月支+时支）mod12+五虎遁干；胎元=月柱干+1、支+3。校验 1987 → 癸丑/乙巳/庚子；🟡 命宫紫微版基于农历月与子平节气月支在节交日有差"),
            d("神煞（11 个常用）", Det, "日干锚 6 个（羊刃/禄/文昌/红艳/学堂/词馆，通行版固化） + 年支锚 4 个（桃花/驿马/华盖/将星，三合派生） + 日柱锚 1 个（魁罡严格 4 日）。多源校验（《三命通会》《渊海子平》+ 3 中文源）。校验 1987 → 年柱将星/月柱文昌+学堂/日柱驿马"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("late_lichun", "晚子·立春换年（主流）", true, "23-24 点归次日子时，立春为新年界。子平命理主流。"),
            s("late_sf", "晚子·春节换年", false, "夜子归次日，但年柱以农历正月初一为界。少数民间用法。"),
            s("early_lichun", "早子·立春换年", false, "夜子归当日（传统少数）；年柱仍立春。早期古法残留。"),
            s("early_sf", "早子·春节换年", false, "夜子归当日 + 春节换年。两少数派合用，极小众。"),
        ] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到统一契约上：元数据齐备、能出盘、确定性谱已声明、
    /// 有流派时恰有一个默认。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = BaziEngine;
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
