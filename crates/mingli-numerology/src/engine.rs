//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{
    d, s, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query, SchoolItem, WordEngine,
    WordQuery,
};
use serde_json::Value;

/// 数字学叶（D 族·哈希环）。日期生命灵数 + 生日数；给出姓名时附表达/灵魂/人格数（两套字母表）。
#[derive(Debug, Default)]
pub struct NumerologyEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(e: &NumerologyEngine, m: &Moment, q: &Query) -> crate::Cast {
    // 本叶同时长在两条端口上（时刻 + 字词），两者都有 `id`，故点明取哪一条
    let method = match q.school_of(<NumerologyEngine as CastingEngine>::id(e), "component") {
        "whole_sum" => crate::LifePathMethod::WholeSum,
        _ => crate::LifePathMethod::Component,
    };
    match &q.name {
        Some(name) => crate::compute_named_with(m, name, method),
        None => crate::compute_at_with(m, method),
    }
}

impl CastingEngine for NumerologyEngine {
    fn id(&self) -> &'static str {
        "numerology"
    }
    fn name(&self) -> &'static str {
        "数字学"
    }
    fn family(&self) -> Family {
        Family::Hashing
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn reading_notes(&self) -> Option<&'static str> {
        Some("\n【字段语义提示（西洋数字学）】\n\
            - `life_path` / `life_path_method` / `life_path_alt`：生命灵数，由出生日期数字根得。\
              **两种算法**：component 逐段约化后再加、whole_sum 全部数字一次加总，\
              少数日期两法结果不同，故两个都给。\n\
            - `birthday`：生日数（出生日当天的数字）。\n\
            - `pythagorean` / `chaldean`：两套字母表各出一组姓名数——\
              `expression` 表达数（全名）、`soul_urge` 灵魂数（元音）、`personality` 人格数（辅音）。\
              两表的字母取值不同（Pythagorean A=1..I=9 循环；Chaldean 1..8，9 留空），故结果不同是正常的。\n\
            - `by_y_rule`：Y 作元音还是辅音，各家不一，故按不同处理并出多组。\n\
            - 主数 11 / 22 / 33 不再约化。\n\
            - 🟡 W 是否可作元音证据不足，本叶一律作辅音，见确定性谱。\n\
            - **读法**：先说生命灵数，再挑一套字母表说姓名三数；两套并说容易乱，选一套并注明。")
    }
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal, Intent::Onomancy]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 生命灵数——由出生日期数字根得。
        let c = chart(self, m, q);
        Some(Principal { label: "生命灵数", value: c.life_path.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("姓名数（双字母表并出）", Det, "Pythagorean/Chaldean 同时输出，无需选择"),
            d("Chaldean 字母表本身各家不同", Und, "🟡 三份公布的对照表在四个字母上互不相同：\
                本叶取的一支作 1:AIJQY 2:BKR 3:CGLS 4:DMT 5:EHNX 6:UVW 7:OZ 8:FP\
                （namevibrations、thelawofattraction 等多处同述，即 Cheiro 一系「不给 9」的古典表）；\
                professionalnumerology.com 的表把 S 作 2、X 作 4、Q 作 8、Y 作 6；\
                astronumero.org 另出一版自称 improved，把 S 作 6、X 作 5、H 作 8，\
                并把 9 分给 E 与 T——而「9 不分给字母」正是古典 Chaldean 的定义性特征。\
                本叶只走古典一支并在此声明其余，不静默选边；要收哪一支须各自找 ≥2 源"),
            d("生命灵数（可选 Component/WholeSum）", Det, "两派算法已实现并交叉校验；每次同时给出主+alt"),
            d("Y 元音归属（三说并出）", Det, "语境派『Y 紧邻元音则作辅音』4 独立源(Decoz/World Numerology、Token Rock、Felicia Bender、Crystal Logic)，可复现 Decoz 全部八条位置细则；『跟在元音后仍算元音』一支 2 独立源(Lyn's、Astrala)；『一律辅音』1 二手源(Bender 转述 Juno Jordan)。三读同时输出，不替调用方选边"),
            d("Y 归属的按音节条款", Und, "🟡 语境两派都还带一条『该音节里没有别的元音时 Y 算元音』(如 Bryan)，须分音节才能判；本叶无音节切分器，不实现也不假装实现"),
            d("W 是否可作元音", Und, "🟡 Matthew/Drew/Owen 一类里 W 算元音的说法仅 2 源(其一只有立场无规则)，且 Decoz 明确反对；强度不足，本叶一律作辅音"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("component", "分量约化（Pythagorean 学派）", true, "y/m/d 各约化后求和再约化；现代数字学常用"),
            s("whole_sum", "全数字直加（Chaldean 学派）", false, "ymd 全数字平铺相加再约化；古典 Chaldean/Kabbalistic 派常用"),
        ] }
    }
}

/// 字词端口：数字学的**姓名**那一半。
///
/// 这一片同时长在两条契约上，因为它本来就吃两种输入：生命灵数由出生日期得（时刻叶那一半），
/// 姓名三数只吃字（字词叶这一半）。`answers()` 里的 `Intent::Onomancy` 一直写着本叶答「字」，
/// 而在此之前本叶只实现了时刻那一条端口——认领了一类问局，却没有能答它的入口。
///
/// 两套字母表（Pythagorean / Chaldean）都出，不替调用方选边——这与本叶的确定性谱一致：
/// 两套各有传承且给出不同的数，静默选一套等于替读者拍板。
impl WordEngine for NumerologyEngine {
    fn id(&self) -> &'static str {
        "numerology"
    }
    fn name(&self) -> &'static str {
        // 与时刻端口同名：这是**同一套术数**，模态由端点决定，不是两样东西
        "数字学"
    }
    fn compute(&self, q: &WordQuery) -> Result<Value, String> {
        let name = q.text.clone().unwrap_or_default();
        if name.trim().is_empty() {
            return Err("姓名数需要 text（拉丁字母姓名）".into());
        }
        // 形状与其余字词叶一致：`system` / `input` / `result`——三片都这样，
        // 承接层与前端才不必为第四片另开一条分支
        Ok(serde_json::json!({
            "system": "numerology",
            "input": name,
            "result": {
                "pythagorean": crate::name_numbers(&name, crate::System::Pythagorean),
                "chaldean": crate::name_numbers(&name, crate::System::Chaldean),
            },
        }))
    }
    fn profile(&self) -> &'static [DetItem] {
        <Self as CastingEngine>::profile(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到统一契约上：元数据齐备、能出盘、确定性谱已声明、
    /// 有流派时恰有一个默认。
    ///
    /// 本叶长在**两条**端口上，故方法要点名走哪一条——两边都有 `id` / `name` / `profile`。
    /// 这不是消歧义的权宜：两条端口答的是两类问局（「命」吃出生日期，「字」吃姓名），
    /// 各自的元数据本来就该各自成立。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = NumerologyEngine;
        assert!(!CastingEngine::id(&e).is_empty() && !CastingEngine::name(&e).is_empty());
        assert!(!WordEngine::id(&e).is_empty() && !WordEngine::name(&e).is_empty());
        let m = Moment::new(1990, 6, 15, 14, 30, 8.0);
        let q = Query::at(1990, 6, 15, 14, 30, 8.0);
        assert!(!e.cast(&m, &q).is_null(), "每片叶都应产出非空盘面");
        assert!(!CastingEngine::profile(&e).is_empty(), "每片叶都要显式声明确定性谱");
        let defaults = e.schools().iter().filter(|s| s.default).count();
        assert!(e.schools().is_empty() || defaults == 1, "有流派的叶应恰有一个默认");
        assert!(!e.family().label().is_empty());
    }

    /// 字词端口：姓名三数出得来，两套字母表都给，缺 text 时明确报错而不是给个空壳。
    #[test]
    fn the_word_port_gives_both_alphabets_and_refuses_an_empty_name() {
        let e = NumerologyEngine;
        let out = WordEngine::compute(&e, &WordQuery { text: Some("John Smith".into()), ..WordQuery::default() })
            .expect("给了姓名就该算得出");
        for k in ["pythagorean", "chaldean"] {
            assert!(!out["result"][k].is_null(), "两套字母表都要给，缺了 `{k}`");
            assert!(
                out["result"][k]["expression"].as_u64().is_some_and(|n| n > 0),
                "`{k}` 的表达数应是正数"
            );
        }
        // 两套是两套，不是一套的转发。单个姓名上撞出同一个数是常事——表达数要归约到 1..9，
        // 九分之一的概率而已（"John Smith" 两套都得 8，第一版就栽在这个巧合上）。
        // 真正的性质是「存在分歧」：若一套只是转发另一套，它们会**处处**相同。
        let names = ["John Smith", "Ada Lovelace", "Rachel Carson", "Kurt Godel", "Emmy Noether"];
        let differs = names.iter().any(|n| {
            let v = WordEngine::compute(&e, &WordQuery { text: Some((*n).to_string()), ..WordQuery::default() })
                .expect("给了姓名就该算得出");
            v["result"]["pythagorean"] != v["result"]["chaldean"]
        });
        assert!(differs, "五个姓名上两套字母表给出完全相同的读数，其中一套多半只是转发");
        assert!(
            WordEngine::compute(&e, &WordQuery::default()).is_err(),
            "缺 text 应明确报错——给个空壳等于让调用方以为算过了"
        );
    }
}
