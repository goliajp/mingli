//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, s, CastingEngine, DetItem, Determinism, Family, Moment, Query, SchoolItem};
use serde_json::Value;

/// 印度占星(Jyotish)叶（B 族）。仅 `jyotish` feature 开启时编译（依赖 astrology + ephemeris）。
/// 9 行星（含 Rahu/Ketu）+ 27 nakshatra + 12 rasi + Lagna，4 ayanamsa 流派。
#[derive(Debug, Default)]
pub struct JyotishEngine;

impl CastingEngine for JyotishEngine {
    fn id(&self) -> &'static str {
        "jyotish"
    }
    fn name(&self) -> &'static str {
        "印度占星"
    }
    fn family(&self) -> Family {
        Family::Angular
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        let geo = match (q.latitude, q.longitude) {
            (Some(latitude), Some(longitude)) => Some(mingli_ephemeris::GeoLocation { latitude, longitude }),
            _ => None,
        };
        let mode = crate::Ayanamsa::from_id(q.school_of(self.id(), "lahiri"))
            .unwrap_or_default();
        serde_json::to_value(crate::compute_at(m, geo, mode)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("9 行星 navagraha 恒星黄经", Det, "7 真行星走 VSOP87/ELP-2000/82（Lahiri J2000 容差<0.005°）；Rahu/Ketu 走 Meeus 22.4 月升交点公式"),
            d("27 nakshatra + Vimshottari 主星", Det, "Wikipedia/GrahaGuru/Vedicka 3 源完全一致"),
            d("12 rasi（白羊..双鱼）", Det, "与西洋 12 sign 一一对应，恒星黄道下"),
            d("Lagna（上升）", Det, "Asc(tropical) − ayanamsa；复用 astrology 平恒星时+平交角"),
            d("Ayanamsa（4 派）", Det, "Lahiri 用 SE 1956-01-01 anchor + 平岁差线性，容差 ±0.05° vs Swiss Ephemeris；KP/Raman/Fagan 用 J2000 静态偏移"),
            d("Vimshottari mahadasha timeline", Det, "9 主星固定年表（7/20/6/10/7/18/16/19/17 总 120），月亮 nakshatra 残余比例算 birth dasha 起止；9 段完整 timeline"),
            d("D-9 navamsa 分盘", Det, "公式 floor(lon×0.3)%12；校验三类(Movable/Fixed/Dual)起算 sign 与古典分类完全一致"),
            d("Antardasha/Pratyantar 子细分", Und, "本叶给完整 mahadasha 9 段；antardasha （mahadasha 内 9 步子细分） 留后续"),
            d("其它分盘(D-10/D-12/...)", Und, "本叶给 D-1(rasi) + D-9(navamsa) 两核心；其余 vargas 留后续"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("lahiri", "Lahiri （印度官方）", true, "Indian Astronomical Ephemeris 1955；SE_SIDM=1；1956-01-01 anchor 强权威"),
            s("krishnamurti", "Krishnamurti (KP)", false, "K. S. Krishnamurti；与 Lahiri 差 ~−6′"),
            s("raman", "Raman", false, "B. V. Raman；与 Lahiri 差 ~−1°26′46″"),
            s("fagan_bradley", "Fagan-Bradley", false, "西方 sidereal 学派；与 Lahiri 差 ~+0°53′01″"),
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
        let e = JyotishEngine;
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
