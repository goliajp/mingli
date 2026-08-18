//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, s, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query, SchoolItem};
use serde_json::Value;

/// 契约层性别 → 本叶性别。
fn leaf_gender(g: Option<mingli_contract::Gender>) -> Option<crate::Gender> {
    g.map(|x| match x {
        mingli_contract::Gender::Male => crate::Gender::Male,
        mingli_contract::Gender::Female => crate::Gender::Female,
    })
}

/// 紫微斗数叶。
#[derive(Debug, Default)]
pub struct ZiweiEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(e: &ZiweiEngine, m: &Moment, q: &Query) -> crate::ZiweiChart {
    let school = crate::SihuaSchool::from_id(q.school_of(e.id(), "standard"))
        .unwrap_or_default();crate::compute_at_with(m, leaf_gender(q.gender), school)
}

impl CastingEngine for ZiweiEngine {
    fn id(&self) -> &'static str {
        "ziwei"
    }
    fn name(&self) -> &'static str {
        "紫微斗数"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn answers(&self) -> &'static [Intent] {
        // 「运」答得起：本叶算大限盘（`limit.rs`，十年一宫）与流年入宫，那正是「势」要的时间序列，
        // 且由出生时刻单独可导出。**但性别缺省时大限出不来**——顺逆由「年干阴阳 + 性别」定，
        // 缺一不可，那种情形下盘上的 `major_limits` 为空，与四柱大运的处置一致。
        &[Intent::Natal, Intent::Fortune]
    }
    fn reading_notes(&self) -> Option<&'static str> {
        Some("\n【字段语义提示】`palaces[12]` 十二宫（命/兄弟/夫妻/子女/财帛/疾厄/迁移/交友/官禄/田宅/福德/父母）；\
            每宫地支干支+主星；`is_ming/is_shen` 标命/身宫；`wuxing_ju` / `ju_number` 五行局（由命宫纳音得）；\
            `aux 4`（昌曲辅弼）+ `sihua`（四化：禄权科忌）；`major_limits` **大限盘**——十年一宫，`start_age` 即五行局数（起运岁）、`forward` 顺逆、`steps[]` 十二步各带起讫岁与所值宫；**性别缺省时为空**，因为顺逆由「年干阴阳 + 性别」定，缺一不可。看某岁的运，就取该岁落在哪一步、再看那一宫的星曜；`ming_branch` / `ming_ganzhi` 命宫所落之支与其干支——**起盘的第一处落点，五行局由它的纳音得**；`shen_branch` 身宫支（后天表现）；`ziwei_branch` / `tianfu_branch` 紫微与天府所落之支——两星分主南北斗，其余诸星依它们排布，故这两处定了整盘的骨架。**结合星曜组合的传统吉凶含义给出整体格局评估**（如紫微在命=尊贵但需辅、贪狼=多才善变需修身、太阴在命=情感细腻喜独处、化禄主进财、化忌主阻滞等），并对各宫给出有利/不利倾向。")
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 命宫所落之支——紫微起盘的第一处落点。
        let c = chart(self, m, q);
        Some(Principal { label: "命宫支", value: c.ming_branch })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d(
                "大限 / 流年",
                Det,
                "两源同述三条规矩（iztro《紫微斗数基础》与知乎《命局、大限、小限、流年讲解》）：\
                 起运岁 = 五行局数（水二 2 岁起、木三 3、金四 4、土五 5、火六 6）；\
                 第一大限固定落命宫，此后十年一宫；顺逆由「年干阴阳 + 性别」定——阳男阴女顺、阴男阳女逆。\
                 校验取两源都逐宫列出的宫名串：顺行作 命→父母→福德→田宅→官禄→交友，\
                 逆行作 命→兄弟→夫妻→子女→财帛→疾厄——把顺逆或宫名排布方向弄反，起运岁仍对而这两串立破。\
                 流年按太岁支入宫，不涉顺逆也不涉性别。\
                 🟡 小限与斗君未实现；性别缺省时不出大限（顺逆定不下就不给）",
            ),
            d("十二宫·五行局·主星", Det, "Z₁₂ 群作用+五行局，校验 命宫亥·土五局·紫微申"),
            d("4 辅星（昌曲辅弼）", Det, "古典通行口诀（《紫微斗数·安文昌文曲星诀》+ 维基/iztro 实现双证），1990 庚午校验"),
            d("四化（禄/权/科/忌）", Det, "通行版 5 源完全一致；中州派(王亭之)在戊/庚/壬三干的化科上分歧，源自「辅弼不入四化」一条学理，三干同开"),
            d("戊干化科", Det, "通行作右弼 ≥7 独立源(《全书》原诀、维基全集全书两栏、梁若瑜飞星派、钦天门、星林学苑、紫微台、紫微杨)；中州派作太阳 3~4 独立源(王亭之两处亲文、九千飞星版本对照表、蓝天空)。两派并存，均已实现"),
            d("癸干化科", Det, "非分歧项——查过《全书》原诀「癸破巨阴贪狼停」及全集/闽派/北派河洛/占验门/钦天门/梁派飞星/中州派陆斌兆/中州派王亭之十家版本表，癸行逐字全同；多篇对照文亦明写「争议只在戊庚壬三干」"),
            d("庚干化科的底本出处", Und, "🟡 《紫微斗数全书》传本自身有异文：一转录本原诀作「庚日武府同」(天府科/天同忌)，另一版本对照表把「全书·闽派」栏记作「阳武同阴」(天同科/太阴忌)、而把天府科归中州派。本叶按中州派学理归入 quanshu 表，但《全书》底本究竟作何未定"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("standard", "通行版（中州/三合派）", true, "5 独立源完全一致(cnblogs/51xingli×2/vocus/wikipedia)；庚=太阴化科，壬=左辅化科"),
            s("quanshu", "中州派（王亭之版）", false, "主张辅弼不入四化：戊=太阳化科、庚=天府化科、壬=天府化科；其余 7 干同通行版。三干同源于一条学理，不可只开其一。id 沿用历史命名"),
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
        let e = ZiweiEngine;
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
