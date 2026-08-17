//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query};
use serde_json::Value;

/// 太乙神数叶（⟂ 横切）。太乙积年 → 太乙行八宫（三年一宫·阳顺阴逆）+ 三才。
#[derive(Debug, Default)]
pub struct TaiyiEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(_e: &TaiyiEngine, m: &Moment, _q: &Query) -> crate::Cast {
crate::compute_at(m)
}

impl CastingEngine for TaiyiEngine {
    fn id(&self) -> &'static str {
        "taiyi"
    }
    fn name(&self) -> &'static str {
        "太乙神数"
    }
    fn family(&self) -> Family {
        Family::CrossCutting
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn reading_notes(&self) -> Option<&'static str> {
        Some("\n【字段语义提示（太乙神数，读懂后给出有据的周期位置判读）】\n\
            - `jinian`：太乙积年，自历元累计的年数，一切起算之本。`ju` 是本年入第几局。\n\
            - `yang_dun`：阳遁主升发外向、阴遁主收敛内守，是全盘基调。\n\
            - `taiyi`：太乙本身。`palace` 落宫（1..9，**不入中五**）、`gua` 该宫之卦、\
              `sancai` 三才（理天 / 理地 / 理人）、`step` 与 `year_in_palace` 是行宫进度——\
              太乙三年一宫、廿四年一周，看它走到哪一段，就知道所处周期位置。\n\
            - `wenchang` / `shiji`：二目。文昌属主、始击属客，「因主而生客」。\
              各带 `position`（目位）、`name`（所临神名）、`direction`（方位）、\
              `da_jiang`（大将宫）、`can_jiang`（参将宫）。主客两造的强弱由此看。\n\
            - `jishen`：计神，与太乙同为起算之枢。\n\
            - 🟡 本盘只出二目一系；君臣民基 / 大游小游 / 四神 / 十精等其余诸神未收（见确定性谱），\
              留白不是算不出而是未查，别当漏算。\n\
            - **读法**：先看太乙落宫与三才定基调，再看二目主客强弱，最后以行宫进度收束——\
              说的是周期结构与所处位置，不是对现实的预言。")
    }
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal, Intent::Mundane]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 太乙所落之宫。
        let c = chart(self, m, q);
        Some(Principal { label: "太乙宫", value: c.taiyi.palace.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("太乙行八宫·三才·积年", Det, "三年一宫·廿四年一周，积年锚《金镜式经》724=1937281"),
            d("文昌（主目）", Det, "《太乙金镜式经》卷一「推天目所在法」：积年以十八去之，命起武德顺行十六神，遇阴德、大武重留一算；阴遁命起吕申、重留和德大炅（《太乙统宗宝鉴》卷一）。两双计位各占两算故周法十八"),
            d("计神", Det, "阳遁起寅、阴遁起申，逆行十二辰，不入四维。两部原典均载"),
            d("始击（客目）", Det, "《金镜式经》卷二「以計神加和徳宫，求文昌所臨宫」；《统宗》卷二「計神既加和徳之宫，却視天上文昌所臨之下，而為始擊神也」。即 始击 =（文昌 + 和德 − 计神）mod 16"),
            d("主算 / 客算 / 大将 / 参将", Det, "自目位顺行累加沿途正宫宫数至太乙宫止：起点正宫计其宫数、间神计 1，间神不累加，终点太乙宫不计入（《金镜式经》卷二第八条 +《统宗》卷二第七条）；「去十用零」整十者以九去之，参将 = 大将三因后再去十（《统宗》卷二第八条）。诸将可落中五，太乙自身不游中五"),
            d("太乙九宫配法", Det, "乾 1 · 离 2 · 艮 3 · 震 4 · 中 5 · 兑 6 · 坤 7 · 坎 8 · 巽 9，**与洛书不同**（洛书一宫是坎）。三源一致，且由算例反证：局 11 客算得 4 只在此配法下成立"),
            d("校验", Det, "两则纪年实例全字段吻合且出自两部不同的书：唐天復二年（局 11，太乙 4 宫，文昌高丛，始击阳德，客算 4）与秦二世二年（局 55，太乙 3 宫，文昌武德，始击和德）；三个纪年锚点（开元十二年局 49 / 天復二年局 11 / 秦二世二年局 55）在同一 72 局环上自洽"),
            d("144 局立成表全表校验", Und, "🟡 《太乙金镜式经》卷三载《陽局立成》《隂局立成》各 72 局逐局给全字段，确是原典自带的黄金校验集。\
                 取过一次未成：ctext.org 的四库本（wiki chapter=456008）有 OCR 文本，但**多列表格被压成了连排文字**——\
                 表头与格子混在同一行、相邻数局首尾相接，列边界已不可复原；另一版是影印图片，非机读。\
                 照那份重建等于猜列对齐，故不录。要落这张全表须取到保留了列结构的本子（点校本或逐页图像 + 人工校读）。\
                 目前仍以两则纪年实例 + 结构不变量把关"),
            d("定计目（及定算 / 定大将 / 定参将）", Und, "🟡 只见《太乙统宗宝鉴》卷二第九条；《太乙金镜式经》自称「運式之儀有八」，无此条。两书的真实差异就在这一项，单源不实现"),
            d("君臣民基 / 大游小游 / 四神 / 十精等其余诸神", Und, "🟡 现只做了二目一系（文昌 / 始击 → 主客算 → 大将 / 参将），这一系两部原典明载且有纪年实例可校。其余诸神的起法未查，既未确认多源也未确认分歧——是「没查」而不是「查了定不下」，两者性质不同，此处如实记前者"),
            d("落宫绝对相位", Det, "由三个纪年锚点交叉锁定，见上"),
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
        let e = TaiyiEngine;
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
