//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query};
use serde_json::Value;

/// 择日叶（A 族）。建除十二神 + 二十八宿值日 + 彭祖百忌 + 天乙贵人。
#[derive(Debug, Default)]
pub struct ZeriEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(_e: &ZeriEngine, m: &Moment, _q: &Query) -> crate::Cast {
crate::compute_at(m)
}

impl CastingEngine for ZeriEngine {
    fn id(&self) -> &'static str {
        "zeri"
    }
    fn name(&self) -> &'static str {
        "择日"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn reading_notes(&self) -> Option<&'static str> {
        Some("\n【字段语义提示（择日要素）】\n\
            - `jianchu` / `jianchu_pos`：建除十二神（建除满平定执破危成收开闭），由日支与月建相减得。\
              这是分档的依据：除危定执为黄道、成开可用、建满平收为黑道、破闭不可当。\n\
            - `grade` / `grade_label`：本日所属档次，即上面那条的结论。\n\
            - `mansion` / `mansion_index`：二十八宿值日，与建除是两套并行的分法。\n\
            - `tianyi_branches` / `tianyi_names`：天乙贵人所临之支（两支）。\n\
            - `pengzu_gan` / `pengzu_zhi`：彭祖百忌，按日干与日支各出一句忌事。\
              **这是逐条忌讳不是总评**，与建除分档互不统属。\n\
            - 🟡 事类宜忌各家出入大，本盘不合成总分；天乙贵人「庚辛逢虎马」一系单源未取，见确定性谱。\n\
            - **读法**：先说档次与建除，再挑百忌里与所问之事相干的一两条；不必把四套都铺开。")
    }
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal, Intent::Election]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 建除十二神——择日分档的依据。
        let c = chart(self, m, q);
        Some(Principal { label: "建除", value: c.jianchu.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("建除十二神", Det, "日支−月建支 on Z₁₂"),
            d("二十八宿值日", Det, "连续 Z₂₈，偏移 11 跨 341 年 5 锚校验"),
            d("彭祖百忌（干句+支句）", Det, "《钦定协纪辨方书》/通胜多源口诀，22 句固定查表"),
            d("天乙贵人（双地支）", Det, "通行版口诀 5 源一致：《三命通会》卷三、《五行精纪》卷十三/卷十四、《珞琭子三命消息赋注》(徐子平)、《渊海子平·论日贵》"),
            d("天乙贵人「庚辛逢虎马」一系", Und, "🟡 坊间归给《珞琭子赋》不成立——徐子平注与昙莹注全文皆无此诀，徐注用例反是通行版；《渊海子平》全文亦无。唯一原始出处为唐·李筌《太白阴经》卷十『庚辛之日旦理胜光暮理功曹』，属六壬旦暮贵人体系，单源；且该系把甲戊合并作旦丑暮未，不能只挪庚一格移植"),
            d("神煞的宜忌断语", Und, "🟡 本叶出的是**结构事实**：建除十二神、二十八宿值日、彭祖百忌干支句、天乙贵人双支。至于「某神煞当值则宜某事忌某事」，查下来连**同一个日名落在哪一天**都不是一套：建除按流传地区分秦除与楚除，中文维基《建除十二神》条记两者「神名皆为十二位，其名不尽相同」，且因两国历法不同、起算月相差二个月——同一天在两系下拿到的根本不是同一个神。传世的宜忌条文能取到的最早成套者是敦煌遗书《后唐同光二年历书》（「建日不开仓，除日不出财，满日不服药，平日不修沟，定日不做辞，执日不发病，破日不会客，危日不远行」），出土的睡虎地秦简《日书》只零散记「交日……以祭门行，行水，吉」，两者不成对照；通行的干支配位另有《淮南子·天文训》「寅为建，卯为除……丑为闭」一系。近世通书更是各承一脉——鳌头通书 / 象吉通书 / 永吉通书 / 清《协纪辨方书》并行，商用黄历自陈其版本差异正来自「参考书目不同」（天天黄历以《协纪辨方书》《玉匣记》《择吉纲要》为据）。事类划分本身也无统一分类。故只出结构、不下断语，判读交释义层"),
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
        let e = ZeriEngine;
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
