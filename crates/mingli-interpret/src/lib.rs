//! L3.5 释义层（INT）：把**已算好的**盘面翻成人话，与「算」严格分离。
//!
//! 铁律：释义只读引擎输出（[`mingli_engine::LeafOutput`]），**绝不修改/重算**任何数字或名称。
//! - DET 项忠实转述其含义；🟡UND 项须明说「流派分歧/引擎诚实留空」，不替它编。
//! - 用语中性克制，不下绝对断言、不作预言；结尾标「仅供研究与娱乐」。
//! - 释义结果一律标 **🔮INT（LLM 生成，非计算）**，与确定盘面区分。
//!
//! 本 crate 是纯确定部分：[`build_prompt`]（组装带护栏的提示词）+ [`Interpreter`] 抽象 +
//! 离线确定性后端 [`Template`]（无 LLM 时的忠实转述）。真正的 LLM 后端（如 claude CLI）是外部
//! 非确定 I/O，放在承接层（services）实现 [`Interpreter`]，可随时替换。

use mingli_engine::{Determinism, LeafOutput};

/// 主体类型：同一套四柱计算给不同主体读出不同象义。
///
/// **计算层完全 DET 同源**（干支/五行/十神/旺衰对任何主体一致）；
/// **只解读层换映射**。person 是默认；company/product/event 适配「物有时刻 → 八字」（择日的逆运算）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject {
    /// 人（默认）：传统人盘。年=祖根、月=父母青年、日=自身/配偶、时=子女晚年。
    Person,
    /// 公司/组织：年=创立根基/行业属性、月=成长环境/团队、日=主体/核心、时=前景/产出。
    Company,
    /// 物（有时刻发布的产品/建筑/开张）：同公司盘（择日的镜像）。
    Product,
    /// 事（已发生事件）：用于复盘事的性质与走向。
    Event,
}

impl Subject {
    /// 从字符串解析(`"person"/"company"/"product"/"event"`)。
    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "person" | "人" => Some(Self::Person),
            "company" | "公司" => Some(Self::Company),
            "product" | "object" | "物" | "产品" => Some(Self::Product),
            "event" | "事" => Some(Self::Event),
            _ => None,
        }
    }
    /// 中文展示名。
    #[must_use]
    pub fn cn(self) -> &'static str {
        match self {
            Self::Person => "人",
            Self::Company => "公司/组织",
            Self::Product => "物/产品",
            Self::Event => "事/事件",
        }
    }
}

/// 护栏系统指令（所有释义共享）。
pub const GUARDRAIL: &str = "你是术数释义助手。下面是【已由确定性引擎算好】的一片盘面。规则：\
1) 只解释，绝不修改、重算或新增任何数字与名称；\
2) 标 DET 的部分忠实转述其含义；标 UND（欠定） 的部分须明说『此处流派分歧、引擎诚实留空』，不要替它杜撰；\
3) **可以给出吉凶 / 喜忌 / 适合 / 不适合 / 有利 / 不利 等评估**，结合结构事实（强/弱、用神/忌神、格局、神煞、星曜组合）给出基于传统命理推断的建议；评估须有依据（简述基于哪几项结构），避免空泛断言；\
4) 250 字以内，简体中文，结尾标注『仅供研究与娱乐』。";

/// 各叶 chart JSON 字段的语义提示——让 LLM 知道字段不是算命断语，只是结构事实。
/// 没有提示的叶返回 None。
#[must_use]
pub fn semantic_hints(leaf_id: &str) -> Option<&'static str> {
    match leaf_id {
        "bazi" => Some("\n【字段语义提示（读懂 JSON 字段后给出有据的吉凶/喜忌评估）】\n\
            - `year/month/day/hour`：本命四柱（天干+地支+五行+纳音+空亡+藏干+支藏十神+十二长生）。\n\
            - `strength.score(0-100) / .level（强/偏强/中和/偏弱/弱）`：日主能量量级。强者宜抑（取食伤/财/官杀为用），弱者宜扶（取印/比劫为用），中和者走调候。\n\
            - `strength.wuxing`：五行力量分布(%)，求和约 100，标识命局五行结构（缺/旺）。**缺者宜补，旺者宜泄**，可据此给出五行喜忌建议。\n\
            - `pattern.name`：命格分类。正官格主稳正/事业；七杀格主开拓/有锐气需驾驭；正财格主稳定财源；偏财格主活财/商机；正印格主学问/受护；偏印格主特殊才/独行；食神格主创作/丰乐；伤官格主才华/锋芒；建禄格主自立；月刃格主刚强/需修养。给出基于格局的事业、情感、性格倾向评估。\n\
            - `yongshen.primary_wuxing / .secondary_wuxing / .avoid_wuxing`：命局**所喜 / 所忌的五行** — 用神 = 吉、忌神 = 凶 的命理底色。建议用户在用神所属方位/季节/颜色/职业类型上行动有利，避忌神所属。\n\
            - `three_houses.ming_gong/.shen_gong/.tai_yuan`：命宫（结构主轴）、身宫（后天表现）、胎元（承气宫） — 传统附加柱。\n\
            - `year/month/day/hour.shensha[]`：该柱命中的神煞名。可给出传统吉凶倾向：吉神 — 文昌/学堂（利文教）、天乙贵人（受助）、将星（掌权）、禄（食禄）、红艳（异性缘）； 凶神/双刃 — 桃花（感性多缘）、华盖（孤独有灵性）、驿马（流动）、羊刃（刚强带刑）、魁罡（刚毅性烈）。释义时挑 1-3 个最显著的展开命理传统吉凶含义。\n\
            - `dayun.pillars[10]`：十步大运，每步约 10 年。\n\
            - 释义时**挑最值得一说的 1-3 处**结合用神，直接给出该命的总体吉凶倾向与具体行运建议。"),
        "ziwei" => Some("\n【字段语义提示】`palaces[12]` 十二宫（命/兄弟/夫妻/子女/财帛/疾厄/迁移/交友/官禄/田宅/福德/父母）；\
            每宫地支干支+主星；`is_ming/is_shen` 标命/身宫；`ju` 五行局（由命宫纳音得）；\
            `aux 4`（昌曲辅弼）+ `sihua`（四化：禄权科忌）。**结合星曜组合的传统吉凶含义给出整体格局评估**（如紫微在命=尊贵但需辅、贪狼=多才善变需修身、太阴在命=情感细腻喜独处、化禄主进财、化忌主阻滞等），并对各宫给出有利/不利倾向。"),
        _ => None,
    }
}

/// 确定性等级的中文标记。
fn det_mark(s: Determinism) -> &'static str {
    match s {
        Determinism::Det => "DET 确定",
        Determinism::Sto => "STO 随机·可复现",
        Determinism::Und => "UND 欠定🟡",
    }
}

/// 主体语义重映射：告诉 LLM 这个盘要按哪种主体读。
/// 仅 bazi 这种含「宫位/十神/六亲」概念的叶受影响；其它叶 person 与 company 等价。
#[must_use]
pub fn subject_hints(subject: Subject, leaf_id: &str) -> Option<&'static str> {
    if subject == Subject::Person {
        return None;
    }
    match (leaf_id, subject) {
        ("bazi", Subject::Company) => Some("\n【主体重映射：公司/组织盘】\n\
            - **宫位象义**：年柱=创立根基/行业属性、月柱=成长环境/团队/管理层、日柱=主体/核心业务、时柱=前景/产出/未来方向。\n\
            - **十神商业义**：正官=制度合规、七杀=竞争压力、正财=稳定营收、偏财=机会财/投机、\
              食神=产品创新/口碑、伤官=创意/突破、正印=资质背书/品牌、偏印=独特资源/Know-how、\
              比肩=合伙人/同业、劫财=竞品/分摊成本。\n\
            - **用神/旺衰**：依然是命局结构事实（该补/该忌的五行），只是「补什么」可对应公司资源配置方向。\n\
            - **诚实**：宫位重映射「有传统依据但无统一标准」；只论结构倾向，不做经营断言。"),
        ("bazi", Subject::Product) => Some("\n【主体重映射：物/产品盘（择日的逆运算）】\n\
            - **宫位象义**：年柱=出厂/上市背景、月柱=产品定位/品类、日柱=产品本体/核心特性、时柱=用户反馈/生命周期。\n\
            - **十神物象义**：正财=稳定使用价值、偏财=突发场景红利、食神/伤官=表达力/差异化、印=技术背书、\
              官杀=合规/外力压力（平台规则等）、比劫=同类竞品。\n\
            - 物盘=择日盘的镜像：为物**选**好时辰即给它一个好「八字」，反过来读则是看它继承了什么时辰之气。"),
        ("bazi", Subject::Event) => Some("\n【主体重映射：事/事件盘】\n\
            - **宫位象义**：年柱=事的背景/大环境、月柱=诱发因素/参与方、日柱=事的核心走向、时柱=结果显化/后续。\n\
            - **十神事象义**：正官=既定规则、七杀=突发冲击、正财=资源所得、偏财=意外变数、\
              食神=和缓发展、伤官=变数/反转、印=支持/缓冲、比劫=同盟/对峙。\n\
            - 事盘多用于已发生事件的复盘，不作未来预言。"),
        _ => None,
    }
}

/// 由一片叶组装带护栏的释义提示词（确定性，可校验）。默认主体 = Person。
#[must_use]
pub fn build_prompt(leaf: &LeafOutput) -> String {
    build_prompt_with_subject(leaf, Subject::Person)
}

/// 同 [`build_prompt`]，但显式指定主体类型；非 Person 时附加主体重映射段。
#[must_use]
pub fn build_prompt_with_subject(leaf: &LeafOutput, subject: Subject) -> String {
    let mut s = String::with_capacity(512);
    s.push_str(GUARDRAIL);
    if subject != Subject::Person {
        s.push_str("\n【主体类型】本盘的主体不是「人」而是「");
        s.push_str(subject.cn());
        s.push_str("」 — 请按下方主体重映射读宫位/十神；计算层（干支/五行/旺衰/用神）不变，只换象义。");
    }
    s.push_str("\n\n【系统】");
    s.push_str(leaf.name);
    s.push('（');
    s.push_str(leaf.family_label);
    s.push_str(" 族）\n【确定性谱】\n");
    for it in leaf.profile {
        s.push_str("- [");
        s.push_str(det_mark(it.status));
        s.push_str("] ");
        s.push_str(it.aspect);
        s.push('：');
        s.push_str(it.note);
        s.push('\n');
    }
    s.push_str("【盘面 JSON】\n");
    s.push_str(&serde_json::to_string(&leaf.chart).unwrap_or_default());
    if let Some(hint) = semantic_hints(leaf.id) {
        s.push_str(hint);
    }
    if let Some(hint) = subject_hints(subject, leaf.id) {
        s.push_str(hint);
    }
    s
}

/// 团队合盘释义的护栏。
pub const TEAM_GUARDRAIL: &str = "你是术数释义助手。下面是已由确定性引擎算好的【团队合盘】结果。规则：\
1) 只解释，绝不修改或新增任何数字与名称；\
2) **可以给出合婚 / 合伙 / 团队配置的契合度评估** — 结合每人用神在彼此盘中的供给度、团队五行结构 / 强弱互补来给出有利 / 不利的命理倾向；\
3) 评估须有依据（简述基于哪几项数据），并指出可参考的方向（如「五行互补」「主用神高供给」等）；\
4) 250 字以内，简体中文，结尾标注『仅供研究与娱乐』。";

/// 由团队合盘结果（JSON 形式）组装团队释义提示词。
///
/// 输入是 `/api/team` 端点返回的完整 JSON（包含 members/team_wuxing/team_weakest/team_strongest/complement_matrix）。
/// 由调用方序列化好直接传入，本函数不解析也不假设结构，避免 `serde_json` 依赖耦合。
#[must_use]
pub fn build_team_prompt(team_json: &str) -> String {
    let mut s = String::with_capacity(team_json.len() + TEAM_GUARDRAIL.len() + 512);
    s.push_str(TEAM_GUARDRAIL);
    s.push_str("\n\n【团队合盘 JSON】\n");
    s.push_str(team_json);
    s.push_str("\n\n【字段语义提示】\n\
        - `members[i].strength.level` 是该成员日主能量量级；`.yongshen.primary_wuxing` 是该成员**所喜的五行**（补之则吉）。\n\
        - `team_wuxing` 是全队五行分布平均；`team_weakest/strongest` 标出最缺/最旺项。\n\
        - `complement_matrix[i][j]` = 成员 j 的盘中、成员 i 主用神五行的占比(%) — j 越能补 i = 数越大，可作为「j 对 i 有利」的命理依据。\n\
        - 释义时：① 点出团队五行结构（强项 / 短板，以及对应的职能 / 资源利弊）；② 挑 1-2 对**互补显著**的关系给出「谁补谁多 → 利谁」的合伙建议；\
          ③ 若团队五行某项严重缺失(<10%)，指出该项对应职能为团队明显短板，可考虑外援补强；给出有利方向 + 需注意的潜在不利点。");
    s
}

/// 释义后端抽象（可替换：离线 [`Template`] / claude CLI / 其它 LLM）。
pub trait Interpreter {
    /// 由提示词产出释义文本。
    ///
    /// # Errors
    /// 后端不可用 / 调用失败时返回错误。
    fn interpret(&self, prompt: &str) -> std::io::Result<String>;
    /// 后端标识（展示用，如 `"claude-cli"` / `"template"`）。
    fn backend(&self) -> &'static str;
}

/// 离线确定性后端：不调 LLM，忠实把确定性谱转成一段话（无 LLM 时的诚实兜底，且可校验）。
#[derive(Debug, Default)]
pub struct Template;

impl Interpreter for Template {
    fn interpret(&self, prompt: &str) -> std::io::Result<String> {
        // 仅从提示词里回显护栏意图 + 标记这是模板而非 LLM。确定性。
        let has_und = prompt.contains("UND 欠定");
        let mut out = String::from("（模板转述·非 LLM）此盘面各项已由确定性引擎算出；");
        out.push_str("DET 项为确定结果、忠实呈现");
        if has_und {
            out.push_str("，🟡UND 项流派分歧或引擎诚实留空、未予杜撰");
        }
        out.push_str("。仅供研究与娱乐。");
        Ok(out)
    }
    fn backend(&self) -> &'static str {
        "template"
    }
}

/// 一次释义的结果（标 🔮INT，非计算）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct Interpretation {
    /// 叶 id。
    pub leaf: String,
    /// 释义文本。
    pub text: String,
    /// 后端标识。
    pub backend: &'static str,
    /// 始终为 INT（提醒前端：这是 LLM/模板生成，非确定计算）。
    pub kind: &'static str,
}

/// 用给定后端释义一片叶。
///
/// # Errors
/// 后端 `interpret` 失败时透传错误。
pub fn interpret_leaf(it: &dyn Interpreter, leaf: &LeafOutput) -> std::io::Result<Interpretation> {
    interpret_leaf_with_subject(it, leaf, Subject::Person)
}

/// 释义团队合盘：接受 `/api/team` 的 JSON 输出，返回 Interpretation(leaf="team")。
///
/// # Errors
/// 后端 `interpret` 失败时透传错误。
pub fn interpret_team(it: &dyn Interpreter, team_json: &str) -> std::io::Result<Interpretation> {
    let text = it.interpret(&build_team_prompt(team_json))?;
    Ok(Interpretation {
        leaf: "team".to_string(),
        text,
        backend: it.backend(),
        kind: "INT",
    })
}

/// 释义一片叶，显式指定主体类型。
///
/// # Errors
/// 后端 `interpret` 失败时透传错误。
pub fn interpret_leaf_with_subject(
    it: &dyn Interpreter,
    leaf: &LeafOutput,
    subject: Subject,
) -> std::io::Result<Interpretation> {
    let text = it.interpret(&build_prompt_with_subject(leaf, subject))?;
    Ok(Interpretation {
        leaf: leaf.id.to_string(),
        text,
        backend: it.backend(),
        kind: "INT",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mingli_engine::{cast_all_detailed, Gender, Query};

    fn sample_leaf(id: &str) -> LeafOutput {
        let q = Query {
            year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
            gender: Some(Gender::Male), latitude: Some(31.23), longitude: Some(121.47),
            seed: None, name: Some("Ada".to_string()),
            schools: std::collections::BTreeMap::new(),
        };
        cast_all_detailed(&q).into_iter().find(|l| l.id == id).unwrap()
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
}
