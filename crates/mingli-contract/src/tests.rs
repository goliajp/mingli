//! 端口层的契约测试与性质测试。

use super::*;
use serde_json::Value;

fn q() -> Query {
    Query::at(1990, 6, 15, 14, 30, 8.0)
}

fn t() -> AskTime {
    AskTime { year: 2026, month: 8, day: 16, hour: 12, minute: 0, tz: 8.0 }
}

/// 假叶：只实现必需项，用来验 trait 默认与 [`effective_school_id`] 的落默认行为。
#[derive(Debug, Default)]
struct Bare;
impl CastingEngine for Bare {
    fn id(&self) -> &'static str {
        "bare"
    }
    fn name(&self) -> &'static str {
        "裸叶"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, _m: &Moment, _q: &Query) -> Value {
        Value::Null
    }
}

/// 带流派的假叶。
#[derive(Debug, Default)]
struct Dressed;
impl CastingEngine for Dressed {
    fn id(&self) -> &'static str {
        "dressed"
    }
    fn name(&self) -> &'static str {
        "有流派的叶"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, _m: &Moment, _q: &Query) -> Value {
        Value::Null
    }
    fn profile(&self) -> &'static [DetItem] {
        const { &[d("某项", Determinism::Und, "流派分歧")] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[s("one", "甲", true, "默认"), s("two", "乙", false, "备选")] }
    }
}

#[test]
fn minimal_query_leaves_every_optional_atom_empty() {
    let q = q();
    assert_eq!((q.year, q.hour, q.tz), (1990, 14, 8.0));
    assert!(q.gender.is_none() && q.latitude.is_none() && q.seed.is_none() && q.name.is_none());
    assert!(q.schools.is_empty());
    // 未指定流派 → 落到调用方给的默认
    assert_eq!(q.school_of("bazi", "late_lichun"), "late_lichun");
}

#[test]
fn explicit_school_wins_over_the_default() {
    let mut q = q();
    q.schools.insert("bazi".to_string(), "early_sf".to_string());
    assert_eq!(q.school_of("bazi", "late_lichun"), "early_sf");
    assert_eq!(q.school_of("ziwei", "standard"), "standard", "只影响指定的那片叶");
}

#[test]
fn seed_is_explicit_or_derived_from_the_moment() {
    let m = Moment::new(1990, 6, 15, 14, 30, 8.0);
    let derived = effective_seed(&m, &q());
    assert_eq!(derived, m.jd_ut.to_bits(), "缺省种子由共享时刻派生 → 同一时刻可复现");
    let mut fixed = q();
    fixed.seed = Some(2024);
    assert_eq!(effective_seed(&m, &fixed), 2024);
}

#[test]
fn effective_school_falls_back_then_yields_to_explicit() {
    let bare = Bare;
    let dressed = Dressed;
    assert_eq!(effective_school_id(&bare, &q()), "", "无流派的叶给空串");
    assert_eq!(effective_school_id(&dressed, &q()), "one", "有流派的叶落到 default");
    let mut q = q();
    q.schools.insert("dressed".to_string(), "two".to_string());
    assert_eq!(effective_school_id(&dressed, &q), "two");
}

#[test]
fn trait_defaults_are_empty_declarations() {
    let bare = Bare;
    assert!(bare.profile().is_empty() && bare.schools().is_empty());
    assert!(!Dressed.profile().is_empty());
}

/// 端口层对「最小一片叶」的要求：四个必填项能经 trait object 拿到，
/// 两个默认项在不覆盖时给出空声明。这条同时钉住 `CastingEngine` 的对象安全。
#[test]
fn a_leaf_is_usable_through_the_trait_object() {
    let m = Moment::new(2000, 1, 1, 0, 0, 8.0);
    let q = q();
    for (e, want_id, want_family) in [
        (&Bare as &dyn CastingEngine, "bare", Family::Cyclic),
        (&Dressed, "dressed", Family::Sampling),
    ] {
        assert_eq!(e.id(), want_id);
        assert!(!e.name().is_empty(), "{want_id} 的显示名不许为空");
        assert_eq!(e.family(), want_family);
        assert_eq!(e.cast(&m, &q), Value::Null, "假叶排盘返回空值");
    }
    // 每片声明了流派的叶必须恰有一个 default。
    let defaults = Dressed.schools().iter().filter(|s| s.default).count();
    assert_eq!(defaults, 1, "有流派的叶应恰有一个 default");
}

#[test]
fn every_label_is_exactly_what_the_api_ships() {
    // 这条原先只断 `!label().is_empty()`。label() 每个分支返回的都是字面量，所以那个断言
    // 永远不可能红——把三个标签全换成 "xyzzy" 跑整个 workspace，只有 mingli-engine 里
    // 钉了 Cyclic 一个变体的那条红了，其余九个标签一路畅通。它们经 `cast_all_detailed`
    // 的 `family_label` 走到 API 与界面上，是对外契约，所以逐字钉住。
    //
    // 期望值写在穷尽 match 里：往枚举加变体而不来这里补一行，编译就过不去。
    fn family_label(f: Family) -> &'static str {
        match f {
            Family::Cyclic => "循环群/CRT",
            Family::Angular => "角度量化",
            Family::Sampling => "抽样/二进制",
            Family::Hashing => "哈希环",
            Family::CrossCutting => "飞布/横切",
        }
    }
    fn determinism_label(d: Determinism) -> &'static str {
        match d {
            Determinism::Det => "确定",
            Determinism::Sto => "随机·种子可复现",
            Determinism::Und => "欠定",
        }
    }
    fn status_label(s: IntentStatus) -> &'static str {
        match s {
            IntentStatus::Live => "已上线",
            IntentStatus::Pending => "待承接",
        }
    }

    let fams = [
        Family::Cyclic,
        Family::Angular,
        Family::Sampling,
        Family::Hashing,
        Family::CrossCutting,
    ];
    for f in fams {
        assert_eq!(f.label(), family_label(f), "{f:?} 的标签变了");
    }
    for d in [Determinism::Det, Determinism::Sto, Determinism::Und] {
        assert_eq!(d.label(), determinism_label(d), "{d:?} 的标签变了");
    }
    for s in [IntentStatus::Live, IntentStatus::Pending] {
        assert_eq!(s.label(), status_label(s), "{s:?} 的标签变了");
    }

    // 标签同时是界面上的分组名，重了两族就并成一堆，所以互不相同也要守。
    let mut seen = std::collections::HashSet::new();
    for f in fams {
        assert!(seen.insert(f.label()), "两个家族共用标签 {}", f.label());
    }
}

#[test]
fn querykind_survives_a_serde_round_trip() {
    let kind = QueryKind::Fortune { natal: q(), t_target: t() };
    let json = serde_json::to_string(&kind).expect("应可序列化");
    assert!(json.contains(r#""kind":"fortune""#), "内部标签用于 HTTP 契约");
    let back: QueryKind = serde_json::from_str(&json).expect("应可反序列化");
    assert_eq!(back.id(), "fortune");
}

#[test]
fn querykind_id_covers_all_variants() {
    // 8 个变体 id 全唯一，与 intents() 顺序对应。
    let kinds = [
        QueryKind::Natal(q()),
        QueryKind::Fortune { natal: q(), t_target: t() },
        QueryKind::Event { t_ask: t(), seed: 42, q_text: None },
        QueryKind::Election { window_start: t(), window_end: t(), category: "婚".into() },
        QueryKind::Synastry { a: q(), b: q() },
        QueryKind::Mundane { p_polity: q() },
        QueryKind::Locative { t_ask: t(), seed: 7, category: "寻物".into() },
        QueryKind::Onomancy { name: "李白".into(), surname_strokes: Some(7), given_strokes: Some(5) },
    ];
    let ids: Vec<&'static str> = kinds.iter().map(QueryKind::id).collect();
    assert_eq!(ids, vec!["natal", "fortune", "event", "election", "synastry", "mundane", "locative", "onomancy"]);
}

#[test]
fn intents_well_formed_and_aligned_with_querykind() {
    let specs = intents();
    assert_eq!(specs.len(), 8, "应有 8 类问事意图");
    // 每类恰出现一次 + 各字段非空。
    //
    // 「哪几片叶答这一类」不在这里查——那不是端口层知道的事，见 `CastingEngine::answers`。
    let mut seen = std::collections::BTreeSet::new();
    for s in specs {
        assert!(seen.insert(s.id), "意图应各出现一次，重了：{}", s.id.id());
        assert!(!s.name_zh.is_empty());
        assert!(!s.atoms.is_empty(), "{} atoms 应非空", s.id.id());
        assert!(!s.output_shape.is_empty());
        assert!(!s.note.is_empty());
    }
    // QueryKind 8 变体 id 与 intents 清单 id 一一对应。
    let kind_ids = [
        QueryKind::Natal(q()).id(),
        QueryKind::Fortune { natal: q(), t_target: t() }.id(),
        QueryKind::Event { t_ask: t(), seed: 0, q_text: None }.id(),
        QueryKind::Election { window_start: t(), window_end: t(), category: String::new() }.id(),
        QueryKind::Synastry { a: q(), b: q() }.id(),
        QueryKind::Mundane { p_polity: q() }.id(),
        QueryKind::Locative { t_ask: t(), seed: 0, category: String::new() }.id(),
        QueryKind::Onomancy { name: String::new(), surname_strokes: None, given_strokes: None }.id(),
    ];
    let spec_ids: Vec<&'static str> = specs.iter().map(|s| s.id.id()).collect();
    assert_eq!(kind_ids.to_vec(), spec_ids);
    // 8 意图全部 Live。
    let live_count = specs.iter().filter(|s| s.status == IntentStatus::Live).count();
    assert_eq!(live_count, 8, "8 意图全部 Live");
}

// ── 性质测试：端口层的契约要对**任意**载荷成立，不只对手写的那几个样本 ──
//
// 端口层是全树最内的公共形状，一处漂移会同时打到 24 片叶与全部承接层，
// 所以这里不满足于举例，直接对随机输入验性质。

use proptest::prelude::*;

/// 生成一个字段全随机（但数值有限）的 [`Query`]。
fn arb_query() -> impl Strategy<Value = Query> {
    (
        (-9999i32..9999, 1u32..13, 1u32..32, 0u32..24, 0u32..60, -12.0f64..14.0),
        (
            prop::option::of(prop_oneof![Just(Gender::Male), Just(Gender::Female)]),
            prop::option::of(-90.0f64..90.0),
            prop::option::of(-180.0f64..180.0),
            prop::option::of(any::<u64>()),
            prop::option::of("[a-zA-Z一-龥]{0,12}"),
            prop::collection::btree_map("[a-z]{1,8}", "[a-z]{1,8}", 0..4),
        ),
    )
        .prop_map(|((year, month, day, hour, minute, tz), (gender, latitude, longitude, seed, name, schools))| Query {
            year,
            month,
            day,
            hour,
            minute,
            tz,
            gender,
            latitude,
            longitude,
            seed,
            name,
            schools,
        })
}

fn arb_asktime() -> impl Strategy<Value = AskTime> {
    (-9999i32..9999, 1u32..13, 1u32..32, 0u32..24, 0u32..60, -12.0f64..14.0)
        .prop_map(|(year, month, day, hour, minute, tz)| AskTime { year, month, day, hour, minute, tz })
}

fn arb_kind() -> impl Strategy<Value = QueryKind> {
    prop_oneof![
        arb_query().prop_map(QueryKind::Natal),
        (arb_query(), arb_asktime()).prop_map(|(natal, t_target)| QueryKind::Fortune { natal, t_target }),
        (arb_asktime(), any::<u64>(), prop::option::of(".{0,20}"))
            .prop_map(|(t_ask, seed, q_text)| QueryKind::Event { t_ask, seed, q_text }),
        (arb_asktime(), arb_asktime(), "[a-z]{0,8}").prop_map(|(window_start, window_end, category)| {
            QueryKind::Election { window_start, window_end, category }
        }),
        (arb_query(), arb_query()).prop_map(|(a, b)| QueryKind::Synastry { a, b }),
        arb_query().prop_map(|p_polity| QueryKind::Mundane { p_polity }),
        (arb_asktime(), any::<u64>(), "[a-z]{0,8}")
            .prop_map(|(t_ask, seed, category)| QueryKind::Locative { t_ask, seed, category }),
        (".{0,16}", prop::option::of(1u32..40), prop::option::of(1u32..40))
            .prop_map(|(name, surname_strokes, given_strokes)| QueryKind::Onomancy {
                name,
                surname_strokes,
                given_strokes,
            }),
    ]
}

proptest! {
    /// `Query` 过一趟 JSON 必须原样回来——承接层与 wasm 两侧靠这条对齐。
    #[test]
    fn prop_query_survives_json(q in arb_query()) {
        let once = serde_json::to_value(&q).expect("Query 应可序列化");
        let back: Query = serde_json::from_value(once.clone()).expect("Query 应可反序列化");
        prop_assert_eq!(once, serde_json::to_value(&back).expect("再序列化应成功"));
    }

    /// 8 个变体都要能带着任意载荷过 JSON，且 `kind` tag 与 `id()` 始终一致。
    #[test]
    fn prop_querykind_survives_json_and_keeps_its_tag(k in arb_kind()) {
        let once = serde_json::to_value(&k).expect("QueryKind 应可序列化");
        prop_assert_eq!(once["kind"].as_str(), Some(k.id()), "tag 必须等于 id()");
        let back: QueryKind = serde_json::from_value(once.clone()).expect("QueryKind 应可反序列化");
        prop_assert_eq!(back.id(), k.id());
        prop_assert_eq!(once, serde_json::to_value(&back).expect("再序列化应成功"));
    }

    /// 流派选择只有两种结果：查询里点名的那个，或本叶自己的 default。
    #[test]
    fn prop_effective_school_is_pick_or_default(
        schools in prop::collection::btree_map("[a-z]{1,8}", "[a-z]{1,8}", 0..6),
    ) {
        let mut q = Query::at(2000, 1, 1, 0, 0, 0.0);
        q.schools = schools.clone();
        for e in [&Dressed as &dyn CastingEngine, &Bare] {
            let got = effective_school_id(e, &q);
            if let Some(pick) = schools.get(e.id()) {
                prop_assert_eq!(&got, pick, "点名了就该用点名的");
            } else {
                let default = e.schools().iter().find(|s| s.default).map_or("", |s| s.id);
                prop_assert_eq!(got, default, "没点名就该落 default（无流派则空串）");
            }
        }
    }

    /// 种子：给了用给的，没给则由时刻唯一决定（同一时刻两次调用必同值）。
    #[test]
    fn prop_seed_is_explicit_or_a_function_of_the_moment(
        seed in prop::option::of(any::<u64>()),
        (year, month, day) in (1900i32..2100, 1u32..13, 1u32..29),
    ) {
        let m = Moment::new(year, month, day, 12, 0, 8.0);
        let mut q = Query::at(2000, 1, 1, 0, 0, 8.0);
        q.seed = seed;
        let got = effective_seed(&m, &q);
        if let Some(s) = seed {
            prop_assert_eq!(got, s);
        } else {
            // 没给种子时由时刻唯一决定：同一时刻可复现，不同时刻不相撞
            prop_assert_eq!(got, effective_seed(&m, &q));
            let other = Moment::new(year, month, day, 13, 0, 8.0);
            prop_assert_ne!(got, effective_seed(&other, &q));
        }
    }
}


/// 端口的**默认实现**本身也要跑一遍。
///
/// 二十一片叶把 `answers` / `principal` / `reading_notes` / `subject_notes` / `bearings` /
/// `profile` / `schools` 全覆写了，于是默认分支一次也没执行过——覆盖率上看是 0，
/// 而它们恰恰是「加一片新叶时先得到什么」的定义。这里用一个只实现必填项的最小叶把它们逐条钉住。
mod defaults {
    use super::*;

    struct Minimal;

    impl CastingEngine for Minimal {
        fn id(&self) -> &'static str {
            "minimal"
        }
        fn name(&self) -> &'static str {
            "最小叶"
        }
        fn family(&self) -> Family {
            Family::Cyclic
        }
        fn cast(&self, _m: &Moment, _q: &Query) -> Value {
            Value::Null
        }
    }

    #[test]
    fn a_new_leaf_starts_out_answering_only_the_natal_intent() {
        let e = Minimal;
        assert_eq!(e.answers(), &[Intent::Natal], "缺省只答「命」");
        assert!(e.profile().is_empty(), "缺省无确定性谱——每片叶都得自己声明");
        assert!(e.schools().is_empty(), "缺省无流派");
        let (m, q) = (Moment::new(1990, 6, 15, 14, 30, 8.0), Query::at(1990, 6, 15, 14, 30, 8.0));
        assert!(e.bearings(&m, &q).is_empty(), "缺省不出方位候选");
        assert!(e.principal(&m, &q).is_none(), "缺省无主判据");
        assert!(e.reading_notes().is_none(), "缺省无读法");
        for s in [Subject::Person, Subject::Company, Subject::Product, Subject::Event] {
            assert!(e.subject_notes(s).is_none(), "缺省无主体重映射");
        }
    }

    struct MinimalWord;

    impl WordEngine for MinimalWord {
        fn id(&self) -> &'static str {
            "minimal-word"
        }
        fn name(&self) -> &'static str {
            "最小字词叶"
        }
        fn compute(&self, _q: &WordQuery) -> Result<Value, String> {
            Ok(Value::Null)
        }
    }

    #[test]
    fn the_word_port_has_the_same_empty_default() {
        assert!(MinimalWord.profile().is_empty());
    }
}


/// 枚举的每一条臂都要走到。
///
/// `Intent::id` / `Family::label` / `Determinism` / `IntentStatus::label` 这些 match，
/// 平时只有常用的几条被执行，其余臂一次没跑——覆盖率照出来是空的，
/// 而它们正是「这个枚举对外说什么」的全部内容。少一条就是少一个名字。
#[test]
fn every_arm_of_every_enum_says_something_distinct() {
    let intents = [
        Intent::Natal, Intent::Fortune, Intent::Event, Intent::Election,
        Intent::Synastry, Intent::Mundane, Intent::Locative, Intent::Onomancy,
    ];
    let ids: Vec<&str> = intents.iter().map(|i| i.id()).collect();
    assert_eq!(ids.len(), 8);
    for id in &ids {
        assert!(!id.is_empty() && id.chars().all(|c| c.is_ascii_lowercase()));
    }
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 8, "八个 id 不该重复");

    // `cn()` 是展示名（「公司/组织」），`from_str_opt` 收的是键（「公司」）——
    // 两者不是互逆的一对，故分开验，不假设往返。展示名经 mingli-interpret 的行文
    // 走到读者眼前，所以逐字钉住而不是只断非空；期望值写在穷尽 match 里。
    let want_cn = |s: Subject| match s {
        Subject::Person => "人",
        Subject::Company => "公司/组织",
        Subject::Product => "物/产品",
        Subject::Event => "事/事件",
    };
    for s in [Subject::Person, Subject::Company, Subject::Product, Subject::Event] {
        assert_eq!(s.cn(), want_cn(s), "{s:?} 的展示名变了");
    }
    for (key, want) in [
        ("person", Subject::Person), ("人", Subject::Person),
        ("company", Subject::Company), ("公司", Subject::Company),
        ("product", Subject::Product), ("object", Subject::Product),
        ("物", Subject::Product), ("产品", Subject::Product),
        ("event", Subject::Event), ("事", Subject::Event),
    ] {
        assert_eq!(Subject::from_str_opt(key), Some(want), "`{key}` 应解为 {want:?}");
    }
    assert_eq!(Subject::from_str_opt("没有这种主体"), None);
}
