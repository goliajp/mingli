use super::guardrails::natal::{det_mark, semantic_hints};
use super::*;
use mingli_contract::{Determinism, LeafOutput};

use mingli_contract::{Gender, Query};
use mingli_engine::cast_all_detailed;
use mingli_registry::registry;

fn sample_leaf(id: &str) -> LeafOutput {
let q = Query {
    year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
    gender: Some(Gender::Male), latitude: Some(31.23), longitude: Some(121.47),
    seed: None, name: Some("Ada".to_string()),
    schools: std::collections::BTreeMap::new(),
};
cast_all_detailed(&registry(), &q).into_iter().find(|l| l.id == id).unwrap()
}

#[test]
fn prompt_has_guardrails_and_chart() {
let leaf = sample_leaf("liuren"); // 含 DET + UND
let p = build_prompt(&leaf);
// 护栏关键句在内。
assert!(p.contains("绝不修改"));
assert!(p.contains("仅供研究与娱乐"));
assert!(p.contains("引擎诚实留空"));
// 系统名 + 家族 + 确定性谱 + 盘面 JSON 都在。
assert!(p.contains("大六壬"));
assert!(p.contains("UND 欠定🟡")); // liuren 有欠定项
assert!(p.contains("DET 确定"));
assert!(p.contains("\"pattern\"")); // 盘面 JSON 字段
}

#[test]
fn template_backend_is_deterministic_and_honest() {
let leaf = sample_leaf("liuren");
let r1 = interpret_leaf(&Template, &leaf).unwrap();
let r2 = interpret_leaf(&Template, &leaf).unwrap();
assert_eq!(r1.text, r2.text); // 确定
assert_eq!(r1.backend, "template");
assert_eq!(r1.kind, "INT");
assert_eq!(r1.leaf, "liuren");
assert!(r1.text.contains("仅供研究与娱乐"));
assert!(r1.text.contains("🟡UND")); // liuren 有欠定 → 提示
// 纯 DET 叶（maya 无 UND 项）→ 不提 UND。
let maya = interpret_leaf(&Template, &sample_leaf("maya")).unwrap();
assert!(!maya.text.contains("🟡UND"));
}

#[test]
fn bazi_prompt_has_semantic_hints_and_jixiong_allowed() {
let leaf = sample_leaf("bazi");
let p = build_prompt(&leaf);
// 新版护栏：允许给吉凶/喜忌评估，但要有依据。
assert!(p.contains("吉凶"));
assert!(p.contains("评估"));
// bazi 字段语义提示：strength/pattern/yongshen/three_houses 都在 hint 里。
assert!(p.contains("strength.score"));
assert!(p.contains("pattern.name"));
assert!(p.contains("yongshen.primary_wuxing"));
assert!(p.contains("three_houses"));
// shensha 字段也有语义提示（包含吉凶倾向引导）
assert!(p.contains("shensha"));
assert!(p.contains("文昌"));
assert!(p.contains("仅供研究与娱乐"));
// 盘面 JSON 包含新字段（serialize 自动）
assert!(p.contains("\"strength\""));
assert!(p.contains("\"yongshen\""));
assert!(p.contains("\"pattern\""));
assert!(p.contains("\"three_houses\""));
}

#[test]
fn qimen_prompt_explains_all_four_plates() {
let leaf = sample_leaf("qimen");
let p = build_prompt(&leaf);
// 四盘的字段都要有语义提示，缺一层 LLM 就读不懂盘
for field in ["earth[9]", "sky.stems[9]", "sky.stars[9]", "star_vigor[9]", "gates.gates[9]", "spirits.spirits[9]", "patterns"] {
    assert!(p.contains(field), "缺字段提示：{field}");
}
// 传统判读语在场：三奇、三吉门、九星吉凶、伏吟利主不利客、旬空
assert!(p.contains("三奇") && p.contains("六仪"));
assert!(p.contains("开/休/生") && p.contains("死/惊/伤"));
assert!(p.contains("天蓬") && p.contains("旺相"));
assert!(p.contains("利主不利客"));
assert!(p.contains("旬空"));
// 值符值使是读盘入口，必须点明
assert!(p.contains("值符宫") && p.contains("值使"));
// 护栏仍在
assert!(p.contains("仅供研究与娱乐"));
}

#[test]
fn only_the_semantically_rich_leaves_carry_hints() {
// 结构复杂、需要读法引导的叶才给提示；纯循环叶不给，免得徒增噪音。
for id in ["bazi", "ziwei", "qimen"] {
    assert!(semantic_hints(id).is_some(), "{id} 应有语义提示");
}
for id in ["maya", "pawukon", "nope"] {
    assert!(semantic_hints(id).is_none());
}
}

#[test]
fn event_prompt_asks_for_a_verdict_not_a_life_reading() {
let json = r#"{"asked_at":{"year":2026},"seed":2024,"question":"此事成否","leaves":[{"id":"yijing"}]}"#;
let p = build_event_prompt(json);
// 占事要落到「断」，并要求处理多叶指向不一致
assert!(p.contains("占事") && p.contains("断"));
assert!(p.contains("成与不成") && p.contains("宜忌"));
assert!(p.contains("相左"));
// 无问句时不得虚构所问
assert!(p.contains("不要虚构"));
// 取机可复现这一点要点明
assert!(p.contains("复核"));
// 读法提示覆盖到各族卜筮叶
for k in ["易经", "六壬", "奇门", "塔罗", "地占"] {
    assert!(p.contains(k), "读法缺 {k}");
}
assert!(p.contains("仅供研究与娱乐"));
assert!(p.contains(json), "盘面 JSON 应原样注入");
}

#[test]
fn event_interpretation_round_trips_through_the_offline_backend() {
let r = interpret_event(&Template, r#"{"seed":1,"leaves":[]}"#).expect("模板后端应可用");
assert_eq!((r.leaf.as_str(), r.kind), ("event", "INT"));
assert_eq!(r.backend, "template");
}

#[test]
fn election_prompt_asks_to_pick_days_and_flags_the_uncombined_score() {
let json = r#"{"category":"婚","scanned_days":21,"candidates":[{"day_ganzhi":"己卯","jianchu":"危","grade":"Huang"}]}"#;
let p = build_election_prompt(json);
assert!(p.contains("择吉") && p.contains("挑出"));
// 分档是粗筛、引擎没合成总分——这一点必须告诉 LLM
assert!(p.contains("粗筛") && p.contains("没有合成总分"));
// 百忌与事类的直接冲突要点出；破闭不推荐；无事类不虚构
assert!(p.contains("亥不嫁娶") && p.contains("破 / 闭日") && p.contains("不要虚构"));
// 字段与建除通行义都在
for k in ["candidates[]", "grade", "pengzu_gan", "tianyi", "定宜定盟"] {
    assert!(p.contains(k), "缺 {k}");
}
assert!(p.contains("仅供研究与娱乐"));
assert!(p.contains(json));
}

#[test]
fn election_interpretation_round_trips_through_the_offline_backend() {
let r = interpret_election(&Template, r#"{"candidates":[]}"#).expect("模板后端应可用");
assert_eq!((r.leaf.as_str(), r.kind, r.backend), ("election", "INT", "template"));
}

#[test]
fn locative_prompt_asks_for_a_bearing_and_names_its_school() {
let json = r#"{"category":"寻物","bearings":[{"leaf":"qimen","element":"值符","at":"艮8","direction":"东北"}]}"#;
let p = build_locative_prompt(json);
assert!(p.contains("寻方位") && p.contains("一到两个方位"));
// 取用之法各家不同——必须说明依的是哪一路，且不写成唯一正解
assert!(p.contains("各家不同") && p.contains("哪一路") && p.contains("唯一正解"));
// 六壬三传欠定要照实说；无所寻不虚构
assert!(p.contains("流派分歧") && p.contains("四课上神") && p.contains("不要虚构"));
for k in ["bearings[]", "direction", "坎北", "子北", "戌亥西北"] {
    assert!(p.contains(k), "缺 {k}");
}
assert!(p.contains("仅供研究与娱乐") && p.contains(json));
}

#[test]
fn locative_interpretation_round_trips_through_the_offline_backend() {
let r = interpret_locative(&Template, r#"{"bearings":[]}"#).expect("模板后端应可用");
assert_eq!((r.leaf.as_str(), r.kind, r.backend), ("locative", "INT", "template"));
}

#[test]
fn synastry_prompt_reads_supply_in_both_directions() {
let json = r#"{"a_name":"甲","b_name":"乙","a_supplies_b":23,"b_supplies_a":18,"detail":{}}"#;
let p = build_synastry_prompt(json);
assert!(p.contains("合盘") && p.contains("契合度评估"));
// 两个方向都要说，且要识别不对称
assert!(p.contains("a_supplies_b") && p.contains("b_supplies_a") && p.contains("不对称"));
// 不下关系指令
assert!(p.contains("该不该在一起"));
assert!(p.contains("仅供研究与娱乐") && p.contains(json));
}

#[test]
fn synastry_interpretation_round_trips_through_the_offline_backend() {
let r = interpret_synastry(&Template, r#"{"a_supplies_b":1}"#).expect("模板后端应可用");
assert_eq!((r.leaf.as_str(), r.kind, r.backend), ("synastry", "INT", "template"));
}

#[test]
fn mundane_prompt_describes_cycles_and_refuses_political_forecasting() {
let json = r#"{"founded_at":{"year":1949},"target_year":2026,"annual":{"palace":3,"year_in_palace":2},"timeline":[],"span":24}"#;
let p = build_mundane_prompt(json);
assert!(p.contains("周期结构") && p.contains("不是对现实政体"));
assert!(p.contains("不点评现实政治") && p.contains("不指名任何现任人物"));
// 三年一宫 / 廿四年一周 / 三期 与「哪一宫的第几年」都要在
assert!(p.contains("三年居一宫") && p.contains("廿四年一周") && p.contains("哪一宫的第几年"));
// 未收的诸神不许补
assert!(p.contains("文昌") && p.contains("不要替它补出"));
for k in ["timeline[]", "year_in_palace", "enters_palace", "annual"] {
    assert!(p.contains(k), "缺 {k}");
}
assert!(p.contains("仅供研究与娱乐") && p.contains(json));
}

#[test]
fn mundane_interpretation_round_trips_through_the_offline_backend() {
let r = interpret_mundane(&Template, r#"{"timeline":[]}"#).expect("模板后端应可用");
assert_eq!((r.leaf.as_str(), r.kind, r.backend), ("mundane", "INT", "template"));
}

#[test]
fn team_prompt_allows_compatibility_assessment() {
let team_json = r#"{"members":[{"name":"A","strength":{"level":"偏强"}}],"team_wuxing":{"wood":11,"fire":20,"earth":23,"metal":32,"water":14},"team_weakest":{"wuxing":"木","pct":11},"team_strongest":{"wuxing":"金","pct":32},"complement_matrix":[[11]]}"#;
let p = build_team_prompt(team_json);
// 新版：允许给契合度评估
assert!(p.contains("契合度评估"));
assert!(p.contains("有利"));
assert!(p.contains("依据"));
// 字段语义提示
assert!(p.contains("team_weakest"));
assert!(p.contains("complement_matrix[i][j]"));
// JSON 内容已注入
assert!(p.contains(team_json));
assert!(p.contains("仅供研究与娱乐"));
}

#[test]
fn team_interpret_round_trip_with_template() {
let team_json = r#"{"members":[],"team_wuxing":{},"complement_matrix":[]}"#;
let r = interpret_team(&Template, team_json).unwrap();
assert_eq!(r.leaf, "team");
assert_eq!(r.kind, "INT");
assert_eq!(r.backend, "template");
assert!(r.text.contains("仅供研究与娱乐"));
}

#[test]
fn subject_parse_round_trip() {
for (s, ex) in [
    ("person", Subject::Person), ("人", Subject::Person),
    ("company", Subject::Company), ("公司", Subject::Company),
    ("product", Subject::Product), ("object", Subject::Product), ("物", Subject::Product),
    ("event", Subject::Event), ("事", Subject::Event),
] {
    assert_eq!(Subject::from_str_opt(s), Some(ex));
}
assert!(Subject::from_str_opt("xxx").is_none());
}

#[test]
fn subject_remap_only_for_bazi_and_non_person() {
let bazi = sample_leaf("bazi");
// Person 主体：无主体段
let p = build_prompt_with_subject(&bazi, Subject::Person);
assert!(!p.contains("主体类型"));
assert!(!p.contains("主体重映射"));
// Company：出现重映射段 + 商业义
let c = build_prompt_with_subject(&bazi, Subject::Company);
assert!(c.contains("公司/组织"));
assert!(c.contains("正财=稳定营收"));
assert!(c.contains("不做经营断言"));
// Product：产品象义
let pr = build_prompt_with_subject(&bazi, Subject::Product);
assert!(pr.contains("产品定位"));
assert!(pr.contains("择日盘的镜像"));
// Event：复盘语义，不预言
let ev = build_prompt_with_subject(&bazi, Subject::Event);
assert!(ev.contains("不作未来预言"));
// 其它叶(maya) + Company：无 bazi 特定重映射（返回 None）
let maya = sample_leaf("maya");
let mc = build_prompt_with_subject(&maya, Subject::Company);
assert!(!mc.contains("正财=稳定营收"));
}

#[test]
fn semantic_hints_only_for_known_leaves() {
assert!(semantic_hints("bazi").is_some());
assert!(semantic_hints("ziwei").is_some());
assert!(semantic_hints("maya").is_none());
assert!(semantic_hints("nonexistent").is_none());
}

#[test]
fn det_marks_cover_all_levels() {
assert_eq!(det_mark(Determinism::Det), "DET 确定");
assert_eq!(det_mark(Determinism::Sto), "STO 随机·可复现");
assert_eq!(det_mark(Determinism::Und), "UND 欠定🟡");
}

#[test]
fn custom_backend_via_trait() {
// 模拟可替换后端：自定义 Interpreter。
struct Echo;
impl Interpreter for Echo {
    fn interpret(&self, prompt: &str) -> std::io::Result<String> {
        Ok(format!("len={}", prompt.len()))
    }
    fn backend(&self) -> &'static str {
        "echo"
    }
}
let r = interpret_leaf(&Echo, &sample_leaf("bazi")).unwrap();
assert_eq!(r.backend, "echo");
assert!(r.text.starts_with("len="));
}
