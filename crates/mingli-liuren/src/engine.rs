//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, s, Bearing, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query, SchoolItem};
use serde_json::Value;

/// 大六壬叶（⟂ 横切）。天地盘 + 四课 + 三传课式。
#[derive(Debug, Default)]
pub struct LiurenEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(e: &LiurenEngine, m: &Moment, q: &Query) -> crate::Cast {
    let school = crate::SheHaiSchool::from_id(q.school_of(e.id(), "classical"))
        .unwrap_or_default();crate::compute_at_with(m, school)
}

impl CastingEngine for LiurenEngine {
    fn id(&self) -> &'static str {
        "liuren"
    }
    fn name(&self) -> &'static str {
        "大六壬"
    }
    fn family(&self) -> Family {
        Family::CrossCutting
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn bearings(&self, m: &Moment, q: &Query) -> Vec<Bearing> {
        let school = crate::SheHaiSchool::from_id(q.school_of(self.id(), "classical")).unwrap_or_default();
        crate::bearings_of(&crate::compute_at_with(m, school))
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("classical", "古法（先数涉害深浅）", true, "《六壬大全》卷一歌诀「涉害行来本家止，路逢多克为用取」；卷七《课经》《袖中金》《观月经》皆同；《六壬粹言》卷一称「此古法也」。本叶六个古籍算例复算全中"),
            s("by_position", "近法（只按孟仲季）", false, "陈公献《六壬指南》系「涉害取法，只以孟仲季为准，不以涉害深浅为义，此《指南》所用之法，切记」；《六壬粹言》卷一记「近来诸家，均未用之者」"),
        ] }
    }

    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal, Intent::Event, Intent::Locative]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 日支——六壬布四课自日辰起。
        let c = chart(self, m, q);
        Some(Principal { label: "日支", value: c.day_branch.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("天地盘·寄宫四课", Det, "月将加时 Z₁₂ 旋转，校验 亥将子时甲子日"),
            d("一课下位取日干五行", Det, "一课写作「干上神／日干」，下位是干不是寄宫；乙丁戊辛癸五干的寄宫地支五行与干不同（乙木寄辰土等），取寄宫会判错贼克并连带错判课式。由三门的全表课数对账确认"),
            d("三传·贼克/比用/遥克/伏返", Det, "取传规则明确"),
            d("三传·昴星", Det, "阳日仰视地盘酉上神、阴日俯视天盘酉下之支；**阳日中取支上末取干上，阴日中取干上末取支上**（《课经》理据：刚日本乎天者亲上末传归干，柔日本乎地者亲下末传归辰）。5 源无异议；恰 16 课（刚 4 柔 12），与《六壬大全》卷一「凡昴星止十六课」及《六壬粹言》「计四课／计一十二课」对账"),
            d("三传·别责", Det, "阳日取日干合干之寄宫上神，阴日取日支三合前一位（支+4）**本身**发用；中末皆取干上神。柔日取本身还是取其上神，《订讹》存疑而《六壬粹言》卷一裁「仍以古法为是」；恰 9 课（刚 3 柔 6），日辰清单与《六壬大全》卷一小注逐条对上"),
            d("三传·八专", Det, "阳日自干上神**连本位**顺数三位，阴日自四课上神连本位逆数三位，中末皆取干上神。八专**不取遥克**（《六壬粹言》卷一驳《订讹》：伏吟无遥克之例，八专何独取之；且取遥克则独足课不存）；恰 16 课（刚 6 柔 10），癸丑日因四课皆有克不入，独足课（己未日三传酉酉酉）有且仅有一课"),
            d("三传·涉害·数法", Det, "自天盘神所临地盘位的**下一格**顺行至其**本家的前一格**，沿途每遇一个克它的住户记一重；住户 = 地支本身 + 寄于该宫的天干。起点与终点皆不计——两处边界由《观月经》《课经》的算例定死，六个古籍算例复算全中"),
            d("三传·涉害·取用两派", Und, "🟡 古法先数受克深浅、深者为用（《六壬大全》卷一歌诀、卷七《课经》《袖中金》《观月经》、《御定六壬直指》、《六壬粹言》卷一「此古法也」）；近法不数深浅、直接孟＞仲＞季（陈公献《六壬指南》明言「不以涉害深浅为义，此《指南》所用之法，切记」，《粹言》记「近来诸家均未用之者」）。两派各有多源、各自点名对方，做成 `schools` 开关，默认古法"),
            d("三传·涉害·孟仲季按地盘判", Det, "看天盘神**所临的地盘位**，不是天盘神自己——《六壬粹言》复等例：戊辰日一课子加巳、四课午加亥，子午本是仲却称「俱在孟位上」，因巳亥是孟。3 源一致。异说见《观月经》注与尉山人「上克下取天盘孟仲」，《大全》编者只批「存之」不敢裁定"),
            d("三传·返吟无克（井栏射 / 无亲）", Det, "初取日支驿马、中取支上神、末取干上神。《六壬粹言》卷二「取支之驿马为用，中用支上神，末用干上神，曰无亲。计六课」并列全六组三传；《六壬大全》卷七〈课经集一〉井栏格条、卷四引《黄帝初占》算例、《注解大六壬指南》卷一，取法逐字相合。不分刚柔——无克六日全是阴干配阴支（丁己辛 × 丑未），刚日返吟必有克"),
            d("井栏射的格名与归类", Und, "🟡 格名三说：「井栏射」「无亲」正指此路，「无依」在《订讹》里却指返吟全门，指称冲突。归类亦有争：丁未 / 己未 同时满足返吟无克与八专，《课经集》与《观月经》划归八专（故说「无克惟四日」），《订讹》《粹言》《指南》划归井栏（故说「六日」）——**两法算出的三传完全相同（巳丑丑）**，故只影响标注不影响输出。另《大全》卷一夹注「阳日用辰，阴日用日」与其自身下半句及其余四源全部冲突、措辞恰是昴星法原话，疑为窜入旁注，未采信；无别本可校"),
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
        let e = LiurenEngine;
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
