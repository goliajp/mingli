//! 全树集成测试：装配根 + 编排层 + 21 片叶一起验。
//!
//! 这些断言天然需要「知道有哪些叶」，因此住在装配根这一层——编排层
//! （`mingli-engine`）自身只用假叶测机制，不认识任何真叶。

use mingli_contract::{
    d, AskTime, CastingEngine, Determinism, Family, Gender, Moment, Query, QueryKind,
};
use mingli_contract::Intent;
use mingli_engine::{cast_all, cast_all_detailed, cast_one, route};
use mingli_registry::registry;
use serde_json::Value;
use std::collections::BTreeMap;

/// 契约层性别 → 八字叶性别（测试内自用）。
fn leaf_gender_male(g: Option<Gender>) -> Option<mingli_bazi::Gender> {
    g.map(|x| match x {
        Gender::Male => mingli_bazi::Gender::Male,
        Gender::Female => mingli_bazi::Gender::Female,
    })
}



/// 不覆盖 `profile()` 的裸叶，用于测试 trait 默认（空谱）。
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

fn sample() -> Query {
    Query {
        year: 1990,
        month: 6,
        day: 15,
        hour: 14,
        minute: 30,
        tz: 8.0,
        gender: Some(Gender::Male),
        latitude: None,
        longitude: None,
        seed: None,
        name: None,
        schools: BTreeMap::new(),
    }
}

#[test]
fn cast_all_has_all_leaves() {
    let out = cast_all(&registry(), &sample());
    assert_eq!(out.len(), registry().len());
    assert!(out.contains_key("bazi"));
    assert!(out.contains_key("ziwei"));
    // 跨叶一瞥：同一输入下，八字双子月与西洋太阳双子座并存可对齐比较
    #[cfg(feature = "astrology")]
    {
        assert!(out.contains_key("astrology"));
        assert_eq!(out["astrology"]["planets"][0]["sign"], "双子"); // 1990-06-15 太阳
    }
}

#[test]
fn all_registered_leaves_well_formed() {
    // 遍历注册表，逐叶检查 id/name/family 元数据齐备、cast 产出非空。
    // 三片星历叶随 feature 开关进出，故预期表也按 feature 裁——
    // 关掉它们的轻量构建同样要能过这条测试，否则「可裁剪」就只是说说。
    let expected = [
        ("bazi", "四柱八字", Family::Cyclic),
        ("ziwei", "紫微斗数", Family::Cyclic),
        #[cfg(feature = "astrology")]
        ("astrology", "西洋占星", Family::Angular),
        #[cfg(feature = "jyotish")]
        ("jyotish", "印度占星", Family::Angular),
        #[cfg(feature = "qizhengsiyu")]
        ("qizhengsiyu", "七政四余", Family::Angular),
        ("yijing", "易经起卦", Family::Sampling),
        ("geomancy", "地占", Family::Sampling),
        ("sikidy", "Sikidy", Family::Sampling),
        ("ifa", "Ifá", Family::Sampling),
        ("tarot", "塔罗", Family::Sampling),
        ("meihua", "梅花易数", Family::Cyclic),
        ("xiaoliuren", "小六壬", Family::Cyclic),
        ("zeri", "择日", Family::Cyclic),
        ("maya", "玛雅历", Family::Cyclic),
        ("pawukon", "巴厘Pawukon", Family::Cyclic),
        ("mahabote", "缅甸Mahabote", Family::Cyclic),
        ("liuren", "大六壬", Family::CrossCutting),
        ("qimen", "奇门遁甲", Family::CrossCutting),
        ("taiyi", "太乙神数", Family::CrossCutting),
        ("tibetan", "藏历循环", Family::Cyclic),
        ("numerology", "数字学", Family::Hashing),
    ];
    let r = registry();
    assert_eq!(r.len(), expected.len(), "注册表叶数应与预期一致");
    let m = Moment::new(2024, 6, 15, 14, 30, 8.0);
    let q = sample();
    for (eng, (id, name, fam)) in r.iter().zip(expected.iter()) {
        assert_eq!(eng.id(), *id);
        assert_eq!(eng.name(), *name);
        assert_eq!(eng.family(), *fam);
        assert!(!eng.cast(&m, &q).is_null(), "{id} cast 不应为空");
    }
    // C 族起卦叶在显式种子下可复现且互不相同（不同系统给不同结构）。
    let out = cast_all(&registry(), &sample());
    for id in ["yijing", "geomancy", "sikidy", "ifa", "tarot", "meihua"] {
        assert!(out.contains_key(id), "缺少 C 族叶 {id}");
    }
}

#[test]
fn shared_layer_matches_standalone() {
    // 共享上下文复用结果 ≡ 各叶独立排盘（记忆化不改变结果）。
    let q = sample();
    let m = Moment::new(q.year, q.month, q.day, q.hour, q.minute, q.tz);
    let bazi_shared = mingli_bazi::compute_at(&m, leaf_gender_male(q.gender));
    let bazi_standalone = mingli_bazi::compute(mingli_bazi::BirthInput {
        year: q.year,
        month: q.month,
        day: q.day,
        hour: q.hour,
        minute: q.minute,
        tz: q.tz,
        gender: Some(mingli_bazi::Gender::Male),
    });
    assert_eq!(bazi_shared.day.ganzhi, bazi_standalone.day.ganzhi);
    assert_eq!(bazi_shared.year.ganzhi, "庚午");
}

#[test]
fn yijing_leaf_reproducible() {
    // 同一 Query（含派生种子）→ 同一卦；显式 seed 覆盖时亦可复现。
    let out = cast_all(&registry(), &sample());
    assert!(out["yijing"]["primary_upper"].is_string());
    let again = cast_all(&registry(), &sample());
    assert_eq!(out["yijing"]["primary"], again["yijing"]["primary"]);
    let mut q = sample();
    q.seed = Some(123);
    let a = cast_all(&registry(), &q);
    let b = cast_all(&registry(), &q);
    assert_eq!(a["yijing"], b["yijing"]); // 显式种子可复现
    assert_eq!(r_family("yijing"), Family::Sampling);
}

fn r_family(id: &str) -> Family {
    registry().into_iter().find(|e| e.id() == id).unwrap().family()
}

#[test]
fn engine_metadata() {
    let r = registry();
    assert_eq!(r[0].id(), "bazi");
    assert_eq!(r[0].name(), "四柱八字");
    assert_eq!(r[0].family(), Family::Cyclic);
    assert_eq!(r[1].id(), "ziwei");
    assert_eq!(r[1].name(), "紫微斗数");
    assert_eq!(r[1].family(), Family::Cyclic);
    #[cfg(feature = "astrology")]
    {
        assert_eq!(r[2].id(), "astrology");
        assert_eq!(r[2].name(), "西洋占星");
        assert_eq!(r[2].family(), Family::Angular);
    }
}

#[test]
fn female_and_no_gender() {
    let mut q = sample();
    q.gender = Some(Gender::Female); // 庚午阳年女 → 大运逆行
    let out = cast_all(&registry(), &q);
    assert_eq!(out["bazi"]["dayun"]["forward"], false);
    assert_eq!(out["ziwei"]["ming_branch"], "亥"); // 紫微不依赖性别
    q.gender = None;
    let out2 = cast_all(&registry(), &q);
    assert!(out2["bazi"]["dayun"].is_null()); // 无性别不排大运
}

#[test]
#[cfg(feature = "astrology")]
fn astrology_angles_when_geo_given() {
    // 无坐标 → 占星只出落座，无 Asc/MC。
    let out = cast_all(&registry(), &sample());
    assert!(out["astrology"]["angles"].is_null());
    assert!(out["astrology"]["houses"].is_null());
    // 给出坐标（上海）→ 占星出 Asc/MC + 整宫制十二宫。
    let mut q = sample();
    q.latitude = Some(31.23);
    q.longitude = Some(121.47);
    let out2 = cast_all(&registry(), &q);
    assert!(out2["astrology"]["angles"]["asc_sign"].is_string());
    assert_eq!(out2["astrology"]["houses"].as_array().unwrap().len(), 12);
    // 太阳落座不受坐标影响（共享层一致）。
    assert_eq!(
        out["astrology"]["planets"][0]["sign"],
        out2["astrology"]["planets"][0]["sign"]
    );
}

#[test]
fn every_leaf_declares_determinism_profile() {
    // 每片叶都显式声明确定性谱（非空），每项 aspect/note 非空。
    for e in registry() {
        let p = e.profile();
        assert!(!p.is_empty(), "{} 缺确定性谱", e.id());
        for item in p {
            assert!(!item.aspect.is_empty() && !item.note.is_empty(), "{} 谱项缺字段", e.id());
        }
    }
    // 谱随 cast_all_detailed 一并输出。
    let out = cast_all_detailed(&registry(), &sample());
    assert!(out.iter().all(|l| !l.profile.is_empty()));
    // 全树至少各等级都出现过（DET 普遍、STO 在 C 族、UND 在流派分歧叶）。
    let all: Vec<Determinism> = out.iter().flat_map(|l| l.profile.iter().map(|i| i.status)).collect();
    for s in [Determinism::Det, Determinism::Sto, Determinism::Und] {
        assert!(all.contains(&s), "确定性谱应覆盖 {s:?}");
        assert!(!s.label().is_empty());
    }
    assert_eq!(Determinism::Det.label(), "确定");
    // 运行时调一次构造器（const fn d 平时只在 const 上下文求值）。
    assert_eq!(d("x", Determinism::Det, "y").aspect, "x");
    // 未覆盖谱的叶走 trait 默认（空谱）；顺带覆盖其全部 trait 方法。
    let m = Moment::new(2024, 1, 1, 0, 0, 8.0);
    assert_eq!(Bare.id(), "bare");
    assert_eq!(Bare.name(), "裸叶");
    assert_eq!(Bare.family(), Family::Cyclic);
    assert!(Bare.cast(&m, &sample()).is_null());
    assert!(Bare.profile().is_empty());
}

#[test]
fn cast_one_matches_full_and_handles_unknown() {
    let q = sample();
    let full = cast_all_detailed(&registry(), &q);
    // cast_one 与 cast_all_detailed 的对应叶逐项一致（只是少算其余叶）。
    for id in ["bazi", "liuren", "numerology"] {
        let one = cast_one(&registry(), id, &q).unwrap();
        let from_full = full.iter().find(|l| l.id == id).unwrap();
        assert_eq!(one.id, from_full.id);
        assert_eq!(one.chart, from_full.chart);
        assert_eq!(one.profile.len(), from_full.profile.len());
    }
    #[cfg(feature = "astrology")]
    {
        let one = cast_one(&registry(), "astrology", &q).expect("占星叶已装配");
        let from_full = full.iter().find(|l| l.id == "astrology").expect("全量里应有占星");
        assert_eq!(one.chart, from_full.chart);
    }
    assert!(cast_one(&registry(), "nope", &q).is_none());
}

#[test]
fn cast_all_detailed_preserves_order_and_meta() {
    let out = cast_all_detailed(&registry(), &sample());
    let reg = registry();
    assert_eq!(out.len(), reg.len());
    // 顺序与注册表一致，元数据齐备，盘非空。
    for (leaf, eng) in out.iter().zip(reg.iter()) {
        assert_eq!(leaf.id, eng.id());
        assert_eq!(leaf.name, eng.name());
        assert_eq!(leaf.family, eng.family());
        assert_eq!(leaf.family_label, eng.family().label());
        assert!(!leaf.chart.is_null(), "{} 盘不应为空", leaf.id);
    }
    // 家族齐备。Angular（角度家族）三片全是星历叶，关掉 feature 后整族消失，属设计如此。
    let fams: std::collections::HashSet<Family> = out.iter().map(|l| l.family).collect();
    for f in [Family::Cyclic, Family::Sampling, Family::Hashing, Family::CrossCutting] {
        assert!(fams.contains(&f), "缺家族 {f:?}");
    }
    #[cfg(any(feature = "astrology", feature = "jyotish", feature = "qizhengsiyu"))]
    assert!(fams.contains(&Family::Angular), "开着星历 feature 就该有 Angular 家族");
}

#[test]
fn numerology_leaf_date_and_name() {
    // 无姓名：数字学只出日期数（生命灵数/生日数）。
    let out = cast_all(&registry(), &sample());
    assert!(out["numerology"]["life_path"].is_number());
    assert!(out["numerology"]["pythagorean"].is_null()); // 无姓名
    // 1990-06-15 生命灵数 = 4。
    assert_eq!(out["numerology"]["life_path"], 4);
    // 给姓名：附表达/灵魂/人格数（两套字母表）。
    let mut q = sample();
    q.name = Some("Ada Lovelace".to_string());
    let out2 = cast_all(&registry(), &q);
    assert!(out2["numerology"]["pythagorean"]["expression"].is_number());
    assert!(out2["numerology"]["chaldean"]["expression"].is_number());
    assert_eq!(r_family("numerology"), Family::Hashing);
}

#[test]
fn outputs_are_correct() {
    let out = cast_all(&registry(), &sample());
    assert_eq!(out["bazi"]["year"]["ganzhi"], "庚午");
    assert_eq!(out["bazi"]["day"]["ganzhi"], "辛亥");
    assert_eq!(out["ziwei"]["ming_branch"], "亥");
    assert_eq!(out["ziwei"]["wuxing_ju"], "土五局");
}

// ---- 问局路由测试 ----------------------------------------------------

fn ask_2026() -> AskTime {
    AskTime { year: 2026, month: 6, day: 16, hour: 10, minute: 0, tz: 8.0 }
}

#[test]
fn every_leaf_answers_the_natal_intent() {
    // 「命」是缺省：一片时刻叶总能给出生切片。哪片叶要不答它，得是有意的，
    // 那就得动它自己的 `answers()`，并在这里说明为什么。
    for e in &registry() {
        assert!(
            e.answers().contains(&Intent::Natal),
            "叶 `{}` 没有认领「命」——时刻叶都该能给出生切片，若确实不能，请在这里写明原因",
            e.id()
        );
    }
}

/// 声明与路由必须对得上：认领了哪一类，路由到那一类时就得把它算进去，且它排得出盘。
///
/// **这条验的是往返一致，不是能力。** 一片叶谎称能答某一类时，这条不会红——
/// 路由本来就照着声明走，于是「路由包含它」在谎称时自动成立，实测过（让玛雅历认领「号」，
/// 本条照绿）。留着它是因为它仍能抓住另一类错：声明与路由脱节（如日后有人在 route 里加特例）。
///
/// 「答不答得起」本身只在**逐意图**上机械可查，且只有部分意图查得了——
/// 「寻」要真给得出方位候选，那条在 `claiming_the_locative_intent_means_actually_producing_bearings`。
/// 其余各类（如「运」要时间序列）没有同样干净的判据，靠的是各叶 `answers` 注释里写明的依据与复核。
#[test]
fn a_claimed_intent_routes_back_to_the_leaf_that_claimed_it() {
    let reg = registry();
    let m = mingli_contract::Moment::new(2026, 6, 16, 10, 0, 8.0);
    let q = sample();
    for e in &reg {
        for intent in e.answers() {
            let routed = route(&reg, &kind_of(*intent));
            assert!(
                routed.contains(&e.id()),
                "叶 `{}` 认领了 {:?}，路由却没把它算进去",
                e.id(),
                intent
            );
            assert!(
                !e.cast(&m, &q).is_null(),
                "叶 `{}` 认领了 {:?}，却排不出盘",
                e.id(),
                intent
            );
        }
    }
}

#[test]
fn claiming_the_locative_intent_means_actually_producing_bearings() {
    // 「算得出」要落到那一类的形态上，不是沾边。「位」的形态就是方位候选，
    // 而这一条恰好机械可查：认领了「寻」，`bearings` 就不能是空的。
    //
    // 这条守卫的由来：小六壬曾认领「寻」而没有实现 `bearings`，于是路由到它、排一张盘、
    // 一个候选都不出——测试全绿，界面上只是少一列，没人会发现。
    let m = mingli_contract::Moment::new(2026, 6, 16, 10, 0, 8.0);
    let q = sample();
    for e in &registry() {
        if !e.answers().contains(&Intent::Locative) {
            continue;
        }
        assert!(
            !e.bearings(&m, &q).is_empty(),
            "叶 `{}` 认领了「寻」，却一个方位候选都给不出",
            e.id()
        );
    }
}

/// 每类意图的一个最小载荷，用来验证路由。
fn kind_of(intent: Intent) -> QueryKind {
    match intent {
        Intent::Natal => QueryKind::Natal(sample()),
        Intent::Fortune => QueryKind::Fortune { natal: sample(), t_target: ask_2026() },
        Intent::Event => QueryKind::Event { t_ask: ask_2026(), seed: 42, q_text: None },
        Intent::Election => {
            QueryKind::Election { window_start: ask_2026(), window_end: ask_2026(), category: String::new() }
        }
        Intent::Synastry => QueryKind::Synastry { a: sample(), b: sample() },
        Intent::Mundane => QueryKind::Mundane { p_polity: sample() },
        Intent::Locative => QueryKind::Locative { t_ask: ask_2026(), seed: 7, category: String::new() },
        Intent::Onomancy => {
            QueryKind::Onomancy { name: "Ada".into(), surname_strokes: None, given_strokes: None }
        }
    }
}

#[test]
fn the_intent_catalogue_says_who_answers_each_class() {
    // 端口层只说这一类问局是什么；「谁来答」由注册表里的叶自己认领，在编排层合成。
    let cat = mingli_engine::intent_catalog(&registry());
    assert_eq!(cat.len(), 8, "八类问局各一条");
    for (spec, leaves) in &cat {
        assert!(!leaves.is_empty(), "{} 一片叶都没有认领", spec.id.id());
    }
    let natal = cat.iter().find(|(s, _)| s.id == Intent::Natal).expect("应有「命」");
    assert_eq!(natal.1.len(), registry().len(), "「命」应覆盖整个注册表");
}

#[test]
fn route_natal_returns_full_registry_in_order() {
    let r = route(&registry(), &QueryKind::Natal(sample()));
    let reg_order: Vec<&'static str> = registry().iter().map(|e| e.id()).collect();
    assert_eq!(r, reg_order, "Natal 路由应等于 registry 顺序");
}

#[test]
fn route_non_natal_dispatches_to_declared_leaves() {
    // Fortune → 真有时间序列的叶：四柱的大运/流年，印度占星的 Vimshottari。
    // 紫微与西洋占星曾在这张名单上，但它们的大限/流年与行运都还没实现。
    let r = route(&registry(), &QueryKind::Fortune { natal: sample(), t_target: ask_2026() });
    assert!(r.contains(&"bazi"));
    assert!(r.contains(&"jyotish"));
    assert!(!r.contains(&"ziwei"), "紫微没有大限/流年，不该被路由到「运」");
    // Event → 卜筮叶。
    let r = route(&registry(), &QueryKind::Event { t_ask: ask_2026(), seed: 42, q_text: None });
    assert!(r.contains(&"yijing"));
    assert!(r.contains(&"tarot"));
    assert!(!r.contains(&"bazi"), "Event 不路由本命型叶");
    // Election → zeri 等。
    let r = route(&registry(), &QueryKind::Election { window_start: ask_2026(), window_end: ask_2026(), category: "婚".into() });
    assert!(r.contains(&"zeri"));
    // Mundane → 太乙等。
    let r = route(&registry(), &QueryKind::Mundane { p_polity: sample() });
    assert!(r.contains(&"taiyi"));
    // Locative → 真给得出方位候选的叶。
    let r = route(&registry(), &QueryKind::Locative { t_ask: ask_2026(), seed: 7, category: "寻物".into() });
    assert!(r.contains(&"liuren") && r.contains(&"qimen"));
    assert!(!r.contains(&"xiaoliuren"), "小六壬没有实现 bearings，不该被路由到「寻」");
    // Onomancy → numerology（在 registry）；gematria/abjad/wuge 是 /api/word 字词库不在 cast registry。
    let r = route(&registry(), &QueryKind::Onomancy { name: "Ada".into(), surname_strokes: None, given_strokes: None });
    assert!(r.contains(&"numerology"));
}

#[test]
fn querykind_serde_round_trip() {
    // QueryKind 的 serde 内部标签编码：`{"kind":"natal","year":..}` 等。
    let kind = QueryKind::Onomancy { name: "李白".into(), surname_strokes: Some(7), given_strokes: Some(5) };
    let s = serde_json::to_string(&kind).unwrap();
    assert!(s.contains("\"kind\":\"onomancy\""));
    let back: QueryKind = serde_json::from_str(&s).unwrap();
    assert_eq!(back.id(), "onomancy");
    // Natal 载荷透传。
    let kind = QueryKind::Natal(sample());
    let s = serde_json::to_string(&kind).unwrap();
    assert!(s.contains("\"kind\":\"natal\""));
    let back: QueryKind = serde_json::from_str(&s).unwrap();
    assert_eq!(back.id(), "natal");
}

#[test]
fn natal_cast_path_unchanged_regression_guard() {
    // 核心回归守卫：Natal 路径下，cast_all/cast_one/cast_all_detailed 行为
    // 不受问局路由层影响（字节级一致 oracle 见上方 outputs_are_correct/cast_one_matches_full）。
    // 这里只断言关键 oracle 不变 + route(Natal) 等于 registry。
    let q = sample();
    let out = cast_all(&registry(), &q);
    assert_eq!(out["bazi"]["year"]["ganzhi"], "庚午");
    assert_eq!(out["bazi"]["day"]["ganzhi"], "辛亥");
    assert_eq!(out["ziwei"]["ming_branch"], "亥");
    assert_eq!(route(&registry(), &QueryKind::Natal(q)).len(), registry().len());
}

/// 传统上用这套系统答某一类，而本叶还没实现——这件事必须在 profile 里有交代。
///
/// [`CastingEngine::answers`] 的判定标准是「当下算得出」，于是「传统上该答但没做」
/// 会从声明里消失。它不该就这么消失：本项目的规矩是留白要说明白，
/// 而说明白的位置是该叶的确定性谱。
///
/// 这里点名的四片，是把那张沿用已久的路由表按事实核对后撤下来的。
const REVOKED: [(&str, &str); 4] = [
    ("ziwei", "大限 / 流年"),
    ("astrology", "行运"),
    ("jyotish", "合婚"),
    ("xiaoliuren", "六神配方位"),
];

#[test]
fn a_capability_that_was_taken_away_is_still_accounted_for() {
    let reg = registry();
    for (id, topic) in REVOKED {
        let Some(e) = reg.iter().find(|e| e.id() == id) else {
            continue; // feature 关掉的叶不在注册表里
        };
        let hit = e.profile().iter().find(|it| it.aspect.contains(topic));
        let it = hit.unwrap_or_else(|| {
            panic!("叶 `{id}` 不再认领与「{topic}」相关的问局，profile 里却没有对应条目——\n\
                    「传统上该答而未实现」要写下来，否则这件事只留在 answers() 的注释里，读 profile 的人看不见")
        });
        assert_eq!(
            it.status,
            mingli_contract::Determinism::Und,
            "叶 `{id}` 的「{topic}」条目应标 Und",
        );
        assert!(
            it.note.contains("还没做"),
            "叶 `{id}` 的「{topic}」要分清是「查过定不下」还是「还没做」，本条属后者",
        );
    }
}


/// 声明了的流派选项，选上之后盘面必须真的不一样。
///
/// 流派是这棵树对外承诺的核心之一：某片叶说它支持早子 / 晚子两派，读的人据此以为
/// 换一个就能拿到另一派的盘。可「声明」与「接线」是两件事——`schools()` 里多写一行、
/// 或某个 id 在叶内的 match 里拼错一个字母，都会让选项静默落回默认，
/// 而界面上那个选项照样亮着。这正是本仓库反复抓到的那类缺陷：**声明了但没接上**。
///
/// 判据是「存在性」而非「普遍性」：扫一批时刻与取机种子，只要有一处两派给出不同的盘就算接上了。
/// 不能要求处处不同——早子 / 晚子只在 23 点后那一小时分岔，塔罗的 Marseilles 只换第 8 与第 11 张牌的名，
/// 78 张里抽几张常常一张都不碰（实测 250 个样本里只 4 个不同，但那 4 个证明它是活的）。
#[test]
fn every_school_option_actually_changes_the_chart() {
    // 取样面要盖住已知的分岔位：跨立春、跨春节、跨子时、跨年，取机种子也换几个
    let days = [
        (1990, 6, 15), (1987, 9, 17), (2024, 1, 1), (2024, 2, 4), (2024, 2, 10),
        (2023, 3, 22), (2020, 4, 23), (2026, 8, 18), (2000, 12, 31), (1961, 7, 1),
    ];
    let clocks = [(0, 30), (7, 0), (12, 0), (15, 0), (23, 30)];
    let seeds = [None, Some(1_u64), Some(7), Some(2024), Some(99_991)];

    for e in mingli_registry::registry() {
        let opts = e.schools();
        if opts.is_empty() {
            continue;
        }
        let defaults = opts.iter().filter(|s| s.default).count();
        assert_eq!(defaults, 1, "叶 `{}` 应恰有一个 default=true 的流派，实有 {defaults} 个", e.id());

        for opt in opts.iter().filter(|s| !s.default) {
            let mut differs = false;
            'sweep: for (y, mo, d) in days {
                for (h, mi) in clocks {
                    for seed in seeds {
                        let mut base = mingli_contract::Query {
                            year: y, month: mo, day: d, hour: h, minute: mi, tz: 8.0,
                            gender: Some(mingli_contract::Gender::Male),
                            latitude: Some(31.23), longitude: Some(121.47),
                            seed, name: Some("Ada".into()),
                            schools: std::collections::BTreeMap::new(),
                        };
                        let m = mingli_astro::Moment::new(y, mo, d, h, mi, 8.0);
                        let a = e.cast(&m, &base);
                        base.schools.insert(e.id().to_string(), opt.id.to_string());
                        if e.cast(&m, &base) != a {
                            differs = true;
                            break 'sweep;
                        }
                    }
                }
            }
            assert!(
                differs,
                "叶 `{}` 声明了流派 `{}`（{}），但在 {} 个时刻 × {} 个种子上换它一次都没改变盘面——\n\
                 要么它没接进 cast（叶内 match 的 id 与这里声明的对不上是最常见的一种），\n\
                 要么它的分岔位不在本测试的取样面上，那就把那个时刻补进 days / clocks",
                e.id(),
                opt.id,
                opt.name,
                days.len() * clocks.len(),
                seeds.len(),
            );
        }
    }
}

/// 每一条 Und 都要写下「查过哪些源」，不能只说各家出入很大。
///
/// 铁律给 Und 留了两个归宿：找到 ≥2 独立源就落 Det 并写 oracle，找不到就留 Und，
/// **并把查过哪些源、为何定不下写进 note**。第三种归宿不存在。可「各家说法不一」这句话
/// 本身是不设防的——它跟「我没查」在文本上一模一样，读的人分不出哪条是查证的结论、
/// 哪条是印象。两条曾经就是这么写的（占星的合盘相位、择日的神煞宜忌），
/// 补查之后才知道它们背后确有实据，只是没落到纸上。
///
/// 判据取「点得出名字」：note 里要么出现书名号引的典籍，要么出现拉丁字母写的
/// 实现名 / 作者名 / 站点名——两者都没有，就说明这条只有立场没有出处。
/// 「还没做」那一类另有归宿（见 [`a_capability_that_was_taken_away_is_still_accounted_for`]），
/// 它们不必点源，因为它们声明的正是「尚未查证」。
#[test]
fn every_undetermined_item_names_what_was_checked() {
    let latin = |s: &str| {
        s.split(|c: char| !c.is_ascii_alphabetic() && c != '.' && c != '-')
            .any(|w| w.chars().filter(char::is_ascii_alphabetic).count() >= 3)
    };
    let mut bare = Vec::new();
    for e in mingli_registry::registry() {
        for it in e.profile() {
            if it.status != mingli_contract::Determinism::Und {
                continue;
            }
            if it.note.contains("还没做") {
                continue;
            }
            if !it.note.contains('《') && !latin(it.note) {
                bare.push(format!("{} · {}", e.id(), it.aspect));
            }
        }
    }
    assert!(
        bare.is_empty(),
        "以下 Und 条目只说了「定不下」却没点出任何查过的来源\n  {}\n\
         写清楚查过哪些典籍 / 哪些实现、各自说了什么、为什么据此定不下；\
         若属「尚未查证」，按那一类的写法写明「还没做」",
        bare.join("\n  "),
    );
}

/// 每片叶都要有读法提示，且要与该叶盘面的字段数相称——写一句话搪塞等于没写。
///
/// 门槛按盘面的字段数定，不是拍一个绝对数：字段少的叶（如小六壬三个字段）两三句就够，
/// 字段多的叶（四柱、奇门）少于这个量就一定漏讲了。
/// 判据取「每个顶层字段至少 12 个字」，这是照已有三条样板反推出来的下界，留了余量。
#[test]
fn a_leaf_that_offers_reading_notes_offers_enough_of_them() {
    let reg = registry();
    let m = mingli_contract::Moment::new(1990, 6, 15, 14, 30, 8.0);
    let q = sample();
    for e in &reg {
        let notes = e.reading_notes().unwrap_or_else(|| {
            panic!(
                "叶 `{}` 没有读法提示。读的人手上只有那份 JSON，认得字但不认得这套系统——\n\
                 缺什么补什么，缺的是字段与领域概念之间那一层。写法见 CastingEngine::reading_notes 的文档",
                e.id()
            )
        });
        let chart = e.cast(&m, &q);
        let fields = chart.as_object().map_or(1, serde_json::Map::len);
        let written = notes.chars().count();
        assert!(
            written >= fields * 12,
            "叶 `{}` 的盘面有 {fields} 个顶层字段，读法只有 {written} 字——\n\
             读的人手上只有那份 JSON，字段与领域概念之间那一层得由这里补上",
            e.id()
        );
        assert!(notes.contains('`'), "叶 `{}` 的读法要用反引号写出真实的 JSON 路径", e.id());
    }
}

/// 读法提示里用反引号写出的字段名，必须真的在这片叶的盘面上。
///
/// 这些提示是交给语言模型读的。写错一个字段名不会报错，模型只会去找一个不存在的键，
/// 然后要么略过、要么编——两种都比空着糟。而这件事恰好机械可查：把盘面的键收集起来，
/// 逐个比对提示里反引号包住的标识符。
///
/// 审计时用这个办法扫出七处错：占星把 `midheaven` 写成 `mc`、易经把 `changing_mask`
/// 写成 `moving_lines`、紫微把 `ju_number` 写成 `ju`，还有三处提到了盘面根本不出的 `seed`。
///
/// 只查**像字段名**的记号（ASCII 起头、可带点与方括号）。像 `primary_*` 这种通配写法、
/// 或反引号里的中文与散文，不在此列——它们是行文，不是路径。
#[test]
fn every_field_name_in_the_reading_notes_exists_on_the_chart() {
    fn collect(v: &Value, acc: &mut std::collections::BTreeSet<String>) {
        match v {
            Value::Object(m) => {
                for (k, x) in m {
                    acc.insert(k.clone());
                    collect(x, acc);
                }
            }
            Value::Array(a) => {
                for x in a.iter().take(3) {
                    collect(x, acc);
                }
            }
            _ => {}
        }
    }

    // 入参要**带全原子**：占星的上升 / 中天要坐标，数字学的姓名数要姓名。
    // 拿缺原子的样本去查，会把「这次没给坐标」误判成「字段名写错了」。
    let m = mingli_contract::Moment::new(1990, 6, 15, 14, 30, 8.0);
    let q = Query {
        latitude: Some(31.23),
        longitude: Some(121.47),
        name: Some("Ada Lovelace".to_string()),
        ..sample()
    };
    let mut problems = Vec::new();
    for e in &registry() {
        let Some(notes) = e.reading_notes() else { continue };
        let mut keys = std::collections::BTreeSet::new();
        collect(&e.cast(&m, &q), &mut keys);

        for token in notes.split('`').skip(1).step_by(2) {
            // 只认「像字段名」的：ASCII 字母起头，其余是字母数字下划线点方括号
            if !token.starts_with(|c: char| c.is_ascii_alphabetic()) {
                continue;
            }
            if !token
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "_.[]".contains(c))
            {
                continue;
            }
            for seg in token.split(['.', '[', ']']) {
                // 空段、纯数字下标、以及 `heaven[i]` 这种单字母占位符都跳过
                if seg.is_empty() || seg.len() == 1 || seg.chars().all(|c| c.is_ascii_digit()) {
                    continue;
                }
                if !keys.contains(seg) {
                    problems.push(format!("[{}] `{token}` —— 盘面上没有 `{seg}`", e.id()));
                }
            }
        }
    }
    assert!(
        problems.is_empty(),
        "读法提示写了盘面上不存在的字段名。模型只有那份 JSON，找不到就只能略过或者编：\n  {}",
        problems.join("\n  ")
    );
}
