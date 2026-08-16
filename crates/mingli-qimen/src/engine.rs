//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// 奇门遁甲叶（⟂ 横切）。定局（阴阳遁+三元）+ 地盘三奇六仪。
#[derive(Debug, Default)]
pub struct QimenEngine;

impl CastingEngine for QimenEngine {
    fn id(&self) -> &'static str {
        "qimen"
    }
    fn name(&self) -> &'static str {
        "奇门遁甲"
    }
    fn family(&self) -> Family {
        Family::CrossCutting
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(crate::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("阴阳遁·三元局·地盘三奇六仪", Det, "72 局表（±3 不变量自检），校验阳遁一局"),
            d("时柱·旬首六仪·旬空·值符宫之根", Det, "60 甲子 6 旬穷举校验，1987-09-17 15：00 时柱壬申/甲子旬遁戊/旬空戌亥 oracle"),
            d("值符干·值符宫·值符星·九星原配", Det, "时干甲遁旬首六仪；值符宫 = 实际值符干在地盘的位置；值符星 = 旬首所在宫原配九星（蓬芮冲辅禽心柱任英）。1987 oracle 时干壬→艮8宫，值符星天冲"),
            d("天盘旋转（九星 + 三奇六仪）", Det, "值符自旬首宫走到时干宫，整盘沿后天八卦圆周 坎1-艮8-震3-巽4-离9-坤2-兑7-乾6 刚体旋转；中 5 寄坤 2，中宫之干随坤 2 同转。两则古例复现：阳三局丙寅时（震3→坎1）与阳一局庚午时（坎1→震3），天盘九星各 8 宫全中"),
            d("天禽寄宫", Und, "通行版寄坤 2（与天芮同宫）已固化；古本寄艮 8 一派无多源佐证，未开关"),
            d("人盘八门（值使旋转）", Det, "值使门 = 旬首宫本位门（休坎1 生艮8 伤震3 杜巽4 景离9 死坤2 惊兑7 开乾6）；自旬首宫按宫序号线性阳顺阴逆数过本旬时辰位次落宫（中 5 占位后寄坤 2），八门再沿圆周同步旋转。校验阳遁一局庚午时：休门数至兑 7，8 宫全中"),
            d("神盘八神（位序）", Det, "直符与值符同宫，其余七神自值符宫起沿圆周阳遁顺时针、阴遁逆时针依次落宫。校验起点坎 1 的阳遁盘 8 宫全中"),
            d("旺相休囚死", Det, "节气月令（两气一支，立春开寅月）对天盘九星五行（蓬水芮土冲木辅木禽土心金柱金任土英火）判五等级，取《五行大义》通行判法：当令旺 / 令生相 / 生令休 / 克令囚 / 令克死。校验白露酉月金旺木死"),
            d("八神第 5/6 位称谓", Und, "一系两遁通用「白虎/玄武」，一系阳遁作「勾陈/朱雀」——位序一致、仅名称相左，故两名并出（`spirits` 与 `spirits_alt`）"),
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
        let e = QimenEngine;
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
