//! 本叶对 [`mingli_contract::WordEngine`] 的实现——字/词模态不吃出生时刻，
//! 只吃文字或笔画，因此走与 `CastingEngine` 平行的第二条契约。

use mingli_contract::{d, DetItem, Determinism, WordEngine, WordQuery};
use serde_json::Value;

/// 姓名五格叶的字词入口。
#[derive(Debug, Default)]
pub struct WugeEngine;

impl WordEngine for WugeEngine {
    fn id(&self) -> &'static str {
        "wuge"
    }
    fn name(&self) -> &'static str {
        "姓名五格"
    }
    fn compute(&self, q: &WordQuery) -> Result<Value, String> {
        let s = q.surname.clone().unwrap_or_default();
        let g = q.given.clone().unwrap_or_default();
        if s.is_empty() || g.is_empty() {
            return Err("姓与名笔画至少各一字".to_string());
        }
        Ok(serde_json::json!({ "system": "wuge", "surname": s, "given": g, "result": crate::five_grids(&s, &g) }))
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d(
                "天 / 人 / 地 / 外 / 总五格公式",
                Det,
                "熊崎式：天格 = 姓和（单姓补虚位一）、人格 = 姓末 + 名首、地格 = 名和（单名补虚位一）、\
                 总格 = 全字和（不含虚位）、**外格 = 天 + 地 − 人**。\
                 与另一份独立实现（Getabako/SeimeiHandan，日文 `js/gokaku.js`）在四种姓名长度 × 笔画 1..20 \
                 共 1600 组上逐点相同。外格从前写作「总 − 人 + 1」，把虚位当成了通式的一部分，\
                 复姓双名一路多出 1，就是这次比对抓出来的。\
                 ★ 中文通行的两种口语化表述在**复姓单名**这一格上彼此矛盾，也都与本式不同：\
                 一种作「单姓单名外格为 2，其余取名末字加一」（复姓单名下给名末字 + 1），\
                 另一种作「单姓：总 − 人 + 1；复姓：总 − 人」（复姓单名下给姓首字，少一）。\
                 本式按虚位原则给姓首 + 1——单姓补天格、单名补地格，各补各的，与上述独立实现一致。\
                 那两种表述是把四种情形压成两条规则时压掉的，不是另一派算法",
            ),
            d("三才五行（按个位定）", Det, "1·2 木、3·4 火、5·6 土、7·8 金、9·0 水，多源一致"),
            d("81 数归一", Det, "mod-80 折回 1..=81，81 与 1 同位"),
            d("康熙笔画", Und, "🟡 数千汉字的繁体笔画属大查表，错一字毒整枝；本叶不内置，笔画由调用方提供并自负来源"),
            d("81 数吉凶判断", Und, "🟡 既是大查表又有流派分歧（熊崎本与各家改本出入不小）；本叶只给 81 数本身，不下吉凶断语"),
        ] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到字词契约上：元数据齐备，输入齐备时能取值。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = WugeEngine;
        assert!(!e.id().is_empty() && !e.name().is_empty());
        let q = WordQuery {
            text: Some("שלום".to_string()),
            surname: Some(vec![7]),
            given: Some(vec![16, 9]),
        };
        assert!(!e.profile().is_empty(), "每片叶都要显式声明确定性谱");
        let v = e.compute(&q).expect("输入齐备应能取值");
        assert_eq!(v["system"], e.id());
    }
}
