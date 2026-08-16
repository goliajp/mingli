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
            d("天盘九星旋转·八门·八神", Und, "中宫寄宫法/八门数法/八神序 3 处流派开关，无权威排盘软件 oracle，暂缺"),
        ] }
    }
}
