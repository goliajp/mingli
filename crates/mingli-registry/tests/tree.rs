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
use std::path::{Path, PathBuf};

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
    // Fortune → 真有时间序列的叶。四片各有各的一路，且都由出生时刻单独可导出：
    // 四柱的大运/流年、印度占星的 Vimshottari、紫微的大限、西洋占星的二次推运。
    // 这四片曾只有前两片——后两片是把各自那条时间线做出来之后才回到这张名单上的。
    let r = route(&registry(), &QueryKind::Fortune { natal: sample(), t_target: ask_2026() });
    for id in ["bazi", "jyotish", "ziwei", "astrology"] {
        assert!(r.contains(&id), "叶 `{id}` 有自己的时间序列，应被路由到「运」");
    }
    // 反过来：没有时间序列的叶不该混进来
    assert!(!r.contains(&"maya"), "玛雅历只出某日的历法坐标，没有一生的时间线");
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
///
/// 第三列是这一条**当下属于哪一类**：`还没做` 指证据未查、只差施工；
/// `查过定不下` 指已查证而多源冲突，按铁律留白。一条从前者走到后者是**进展**，
/// 改这张表就是在记录那次进展——小六壬的方位已经这么走过一次（四神可定、小吉与空亡不可）。
/// 一条**做出来**了就从这张表上走掉——那时它不再是留白，而是该叶认领的能力，
/// 由「认领的意图必须路由得回来」那条守着。紫微的大限 / 流年与占星的推运已这么走掉。
/// 这张表**已经空了**——四条当初撤下来的能力各自有了归宿：
///
/// - 紫微「大限 / 流年」：做出来了（`mingli-ziwei` 的 `limit.rs`），现由该叶认领「运」
/// - 占星「行运」：做出来的是**二次推运**（`mingli-astrology` 的 `progression.rs`）——
///   行运本身落不到叶这一层（要两个时刻），那一条改写为该叶 profile 里的 🟡 并说明缘由
/// - 印度占星「合婚」：做出来了（`mingli-jyotish` 的 `kuta.rs`），逐项出区间
/// - 小六壬「六神配方位」：查证了，四定二不定，故仍不认领「寻」但理由已落在该叶 profile
///
/// 表空着不等于这条守卫没用了——**它是为下一次撤销准备的**。哪天某片叶因事实核对
/// 而撤下一项能力，那一项就该回到这里，并在它自己的 profile 里留下说明。
const REVOKED: [(&str, &str, &str); 0] = [];

#[test]
fn a_capability_that_was_taken_away_is_still_accounted_for() {
    let reg = registry();
    for (id, topic, kind) in REVOKED {
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
            it.note.contains(kind),
            "叶 `{id}` 的「{topic}」在本表里记作「{kind}」，note 里却看不出来——\n\
             「查过定不下」与「还没做」是两种留白，不能混；条目改了类别就要同时改这张表",
        );
    }
}


/// 仓库根：本文件在 `crates/mingli-registry/tests/`。
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("crates/").parent().expect("仓库根").to_path_buf()
}

/// `profile()` 里写「校验 X」的，X 必须在测试代码里真的出现。
///
/// 确定性谱是这棵树对外最硬的一句话：某一项标 Det 并注明「校验 甲子」「校验 Meeus 0.02」，
/// 读的人据此相信有一条测试在盯着那个值。可这两件事之间没有任何机械联系——
/// 改测试时删掉一条 oracle、或写声称时把值记错一位，`profile()` 照样这么印，
/// 而它是 `/api/cast` 与提示词都会带出去的东西。
///
/// 判据是「点名的字面量得在测试里找得到」：ISO 日期、干支、拉丁专名、带小数的容差。
/// 找的范围是**整个工作区的测试代码**而非该叶自己——七政四余的「校验 Meeus 0.02」
/// 落在 `mingli-ephemeris`（VSOP87 归它管），只搜本叶会误判成没有。
/// `engine.rs` 自身排除在外：声称不能拿自己作佐证。
#[test]
fn a_verification_claim_names_a_value_the_tests_actually_use() {
    let root = workspace_root();
    let mut corpus = String::new();
    let mut engines = Vec::new();
    for entry in std::fs::read_dir(root.join("crates")).expect("crates/ 应可读") {
        let dir = entry.expect("目录项应可读").path();
        collect_rs(&dir, &mut corpus, &mut engines);
    }
    collect_rs(&root.join("services"), &mut corpus, &mut engines);
    assert!(engines.len() > 15, "只找到 {} 个 engine.rs，扫描方式怕是失效了", engines.len());
    assert!(corpus.len() > 100_000, "测试语料只有 {} 字节，扫描方式怕是失效了", corpus.len());

    let mut bad = Vec::new();
    for (leaf, src) in &engines {
        // 干支只在本叶自己的测试里找：共享的 `mingli-ganzhi` 把六十甲子全枚举了一遍，
        // 拿它当佐证的话，任何一个干支都「找得到」，这一路就成了空判
        let own = own_tests(&root, leaf);
        for claim in src.split("校验").skip(1) {
            let claim: String = claim.chars().take_while(|c| *c != '"').take(60).collect();
            for tok in literals(&claim) {
                let is_ganzhi = tok.chars().count() == 2 && "甲乙丙丁戊己庚辛壬癸".contains(tok.chars().next().unwrap_or(' '));
                let found = if is_ganzhi { own.contains(&tok) } else { corpus.contains(&tok) };
                if !found {
                    let scope = if is_ganzhi { "本叶的测试" } else { "工作区的测试代码" };
                    bad.push(format!("{leaf} 声称「校验…{tok}…」，{scope}里找不到 `{tok}`"));
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "确定性谱里的校验声称没有对应的测试：\n  {}\n\
         要么补上那条 oracle，要么把声称改成它真正验过的东西",
        bad.join("\n  "),
    );
}

/// 某片叶自己的测试代码（不含 `engine.rs`）。
fn own_tests(root: &Path, leaf: &str) -> String {
    let mut s = String::new();
    let mut engines = Vec::new();
    collect_rs(&root.join("crates").join(leaf), &mut s, &mut engines);
    s
}

/// 递归收集 `.rs`：`engine.rs` 单独留作「声称」，其余并进「佐证」语料。
fn collect_rs(dir: &Path, corpus: &mut String, engines: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_rs(&p, corpus, engines);
        } else if p.extension().is_some_and(|e| e == "rs") {
            let Ok(text) = std::fs::read_to_string(&p) else { continue };
            if p.file_name().is_some_and(|n| n == "engine.rs") {
                let leaf = p.components().rev().nth(2).map_or_else(String::new, |c| c.as_os_str().to_string_lossy().into_owned());
                engines.push((leaf, text));
            } else {
                corpus.push_str(&text);
                corpus.push('\n');
            }
        }
    }
}

/// 一句声称里可核对的字面量：ISO 日期、干支、拉丁专名、带小数的数。
fn literals(claim: &str) -> Vec<String> {
    const GAN: &str = "甲乙丙丁戊己庚辛壬癸";
    const ZHI: &str = "子丑寅卯辰巳午未申酉戌亥";
    let ch: Vec<char> = claim.chars().collect();
    let mut out = Vec::new();
    for (i, c) in ch.iter().enumerate() {
        // 干支：一个天干紧跟一个地支
        if GAN.contains(*c) && ch.get(i + 1).is_some_and(|n| ZHI.contains(*n)) {
            out.push(format!("{c}{}", ch[i + 1]));
        }
    }
    // ISO 日期与带小数的数
    let bytes: Vec<char> = ch.clone();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == '-' || bytes[i] == '.') {
                i += 1;
            }
            let tok: String = bytes[start..i].iter().collect();
            let tok = tok.trim_end_matches(['-', '.']).to_string();
            if tok.len() == 10 && tok.matches('-').count() == 2 {
                out.push(tok); // ISO 日期
            } else if tok.contains('.') && tok.matches('.').count() == 1 && !tok.starts_with('0') {
                out.push(tok); // 容差之类；0.x 太常见，容易误判为「到处都有」
            }
        } else if bytes[i].is_ascii_uppercase() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            let tok: String = bytes[start..i].iter().collect();
            if tok.len() >= 4 {
                out.push(tok); // 拉丁专名
            }
        } else {
            i += 1;
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// 取机的种子：声明会动的必须真会动，声明不动的必须真不动，两次同种子必须一样。
///
/// 「取机的种子入盘，故同一次占问可复现」是问局清单对「事」这一类的承诺，
/// 而承诺落在两个方向上，此前只有易经一片被验过其中一个方向：
///
/// - **同种子两次必须一致**——不然「可复现」这句话不成立。每片叶都验，不限卜筮叶
/// - **声明 Sto 的必须真吃种子**——种子没接进去，每次占问出的是同一卦，取机就是假的。
///   按流派逐个验：梅花默认的时间起卦法本就不吃种子（它按时刻起），数字法才吃，
///   所以判据是「至少有一个流派会动」，不是「每个流派都动」
/// - **没声明 Sto 的必须不吃种子**——这一条是反向的，此前完全没人检查。
///   一片自称确定的叶若暗中受种子影响，它的 Det 声明就是假的，
///   而这种叶在盘面上看不出任何异样
#[test]
fn the_drawing_seed_reaches_exactly_the_leaves_that_declare_it() {
    let mk = |seed: Option<u64>| mingli_contract::Query {
        year: 2026, month: 8, day: 18, hour: 12, minute: 0, tz: 8.0,
        gender: Some(mingli_contract::Gender::Male),
        latitude: Some(31.23), longitude: Some(121.47),
        seed, name: Some("Ada".into()),
        schools: std::collections::BTreeMap::new(),
    };
    let m = mingli_astro::Moment::new(2026, 8, 18, 12, 0, 8.0);
    // 质数步长，避开与叶内取模的周期撞车
    let seeds: Vec<u64> = (1..=24).map(|i| i * 7919).collect();

    for e in mingli_registry::registry() {
        // 一、同种子两次一致
        assert_eq!(
            e.cast(&m, &mk(Some(42))),
            e.cast(&m, &mk(Some(42))),
            "叶 `{}` 同一时刻同一种子两次给出了不同的盘——「同一次占问可复现」不成立",
            e.id(),
        );

        // 二、看它到底吃不吃种子（把各流派都算上）
        let declared = e.profile().iter().any(|p| p.status == mingli_contract::Determinism::Sto);
        let base = e.cast(&m, &mk(Some(seeds[0])));
        let mut moves = seeds.iter().skip(1).any(|s| e.cast(&m, &mk(Some(*s))) != base);
        if !moves {
            for opt in e.schools().iter().filter(|x| !x.default) {
                let with = |s: u64| {
                    let mut q = mk(Some(s));
                    q.schools.insert(e.id().to_string(), opt.id.to_string());
                    e.cast(&m, &q)
                };
                let b = with(seeds[0]);
                if seeds.iter().skip(1).any(|s| with(*s) != b) {
                    moves = true;
                    break;
                }
            }
        }

        assert_eq!(
            moves, declared,
            "叶 `{}`：profile {} Sto，实测换种子{}——\n\
             声明 Sto 却不动 = 取机没接进去，每次占问出同一盘；\n\
             动了却没声明 Sto = 它其实不是确定性的，Det 那条是假的",
            e.id(),
            if declared { "声明了" } else { "没声明" },
            if moves { "盘面会变" } else { "盘面一动不动" },
        );
    }
}

/// 声称支持的年份区间，两端每片叶都要照样出得来。
///
/// README 写着「支持 1900–2100」，用例层的 `Birth::validate` 也照此收口。可「不被拒绝」
/// 与「算得对」是两件事：叶各自的锚点、查表、天文近似都有自己的适用范围，
/// 区间两端最容易碰到没人走过的分支——多数输入落在近几十年，端点从来没被跑过。
///
/// 判据取两条，都不涉及具体数值（那属各叶自己的 oracle）：不许 panic，
/// 且字段集要与区间中段的一致——少一个字段就是某条分支在端点上悄悄没走到。
/// 闰日一并扫，它是另一处只在特定日子才走到的分支。
#[test]
fn every_leaf_holds_up_at_both_ends_of_the_supported_range() {
    let edges = [
        (1900, 1, 1, 0, 0), (1900, 2, 4, 23, 59), (1900, 12, 31, 23, 59),
        (2100, 1, 1, 0, 0), (2100, 6, 15, 12, 0), (2100, 12, 31, 23, 59),
        (2000, 2, 29, 12, 0), (2024, 2, 29, 0, 0),
    ];
    let mk = |y, mo, d, h, mi| mingli_contract::Query {
        year: y, month: mo, day: d, hour: h, minute: mi, tz: 8.0,
        gender: Some(mingli_contract::Gender::Male),
        latitude: Some(31.23), longitude: Some(121.47),
        seed: Some(7), name: Some("Ada".into()),
        schools: std::collections::BTreeMap::new(),
    };
    let mid_q = mk(1990, 6, 15, 12, 0);
    let mid_m = mingli_astro::Moment::new(1990, 6, 15, 12, 0, 8.0);

    let mut trouble = Vec::new();
    for e in mingli_registry::registry() {
        let mid = e.cast(&mid_m, &mid_q);
        for (y, mo, d, h, mi) in edges {
            let q = mk(y, mo, d, h, mi);
            let m = mingli_astro::Moment::new(y, mo, d, h, mi, 8.0);
            let cast = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| e.cast(&m, &q)));
            let Ok(v) = cast else {
                trouble.push(format!("{} 在 {y}-{mo}-{d} 上 panic 了", e.id()));
                continue;
            };
            if let (Some(got), Some(want)) = (v.as_object(), mid.as_object()) {
                let miss: Vec<&str> = want.keys().filter(|k| !got.contains_key(*k)).map(String::as_str).collect();
                if !miss.is_empty() {
                    trouble.push(format!("{} 在 {y}-{mo}-{d} 上少了字段 {miss:?}", e.id()));
                }
            }
        }
    }
    assert!(
        trouble.is_empty(),
        "声称支持 1900–2100，但区间端点上出了问题：\n  {}",
        trouble.join("\n  "),
    );
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
