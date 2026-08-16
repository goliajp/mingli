//! 端口层：叶与编排之间的契约。
//!
//! 这里只有**抽象**——一片叶要长成什么样（[`CastingEngine`]）、一次排盘的输入是什么
//! （[`Query`]）、一片叶如何声明自己的确定性边界与流派（[`DetItem`] / [`SchoolItem`]）、
//! 需求侧有哪几类问局（[`IntentSpec`]）。
//!
//! 依赖方向：叶实现这里的 trait，编排层消费这里的 trait，**双方都不认识对方**。
//! 本 crate 除共享时刻 [`mingli_astro::Moment`] 外不依赖任何领域实现。

pub use mingli_astro::Moment;

use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// 计算家族。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Family {
    /// A 循环群 / CRT（时间→模运算）。
    Cyclic,
    /// B 角度量化（星历→黄经→分段）。
    Angular,
    /// C 抽样 / 二进制（熵→有限格）。
    Sampling,
    /// D 哈希环（字符串→数→约化）。
    Hashing,
    /// ⟂ 飞布 / 横切（群作用）。
    CrossCutting,
}

impl Family {
    /// 家族中文标签（承接层展示用）。
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Family::Cyclic => "循环群/CRT",
            Family::Angular => "角度量化",
            Family::Sampling => "抽样/二进制",
            Family::Hashing => "哈希环",
            Family::CrossCutting => "飞布/横切",
        }
    }
}

/// 确定性谱：标注一项计算是确定算的、随机可复现的、还是流派欠定。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Determinism {
    /// 🟢 确定：由 Rust 确定性算出（多校验权威值/已知量）。
    Det,
    /// 🎲 随机·种子可复现：抽样/起卦，给定种子可复现。
    Sto,
    /// 🟡 欠定：流派分歧 / 待权威校验 / 大查表未取证——引擎诚实留空而非臆造。
    Und,
}

impl Determinism {
    /// 中文标签。
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Determinism::Det => "确定",
            Determinism::Sto => "随机·种子可复现",
            Determinism::Und => "欠定",
        }
    }
}

/// 确定性谱的一项：某计算方面的确定性等级与说明。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct DetItem {
    /// 计算方面（如「四柱」「Asc/MC」「三传」）。
    pub aspect: &'static str,
    /// 确定性等级。
    pub status: Determinism,
    /// 一句说明（校验依据 / 为何欠定）。
    pub note: &'static str,
}

/// 构造 [`DetItem`] 的简写。
#[must_use]
pub const fn d(aspect: &'static str, status: Determinism, note: &'static str) -> DetItem {
    DetItem { aspect, status, note }
}

/// 性别（用于需要它的叶，如八字大运）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
pub enum Gender {
    /// 男。
    Male,
    /// 女。
    Female,
}

/// 排盘查询（共享输入）。
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct Query {
    /// 公历年。
    pub year: i32,
    /// 公历月 1..12。
    pub month: u32,
    /// 公历日 1..31。
    pub day: u32,
    /// 时 0..23。
    pub hour: u32,
    /// 分 0..59。
    pub minute: u32,
    /// 时区偏移小时（中国 +8，日本 +9）。
    pub tz: f64,
    /// 性别（可选）。
    pub gender: Option<Gender>,
    /// 出生地纬度（度，北纬为正；占星 Asc/MC 需要，可选）。
    pub latitude: Option<f64>,
    /// 出生地经度（度，东经为正；占星 Asc/MC 需要，可选）。
    pub longitude: Option<f64>,
    /// C 族（抽样/起卦）的可复现种子；`None` 时由共享时刻派生（见 [`effective_seed`]）。
    pub seed: Option<u64>,
    /// 姓名（D 族数字学用，可选）。拉丁字母按 Pythagorean/Chaldean 取值。
    pub name: Option<String>,
    /// **流派选择**：key=叶 `id`，value=该叶选定的流派 id（见各叶 `schools()`）。
    /// 缺省即按各叶 `default = true` 的流派算。同一次 `cast_all` 可为不同叶分别指定。
    #[serde(default)]
    pub schools: BTreeMap<String, String>,
}

impl Query {
    /// 取叶 `engine_id` 的流派 id。未指定时返回 `default_id`。
    #[must_use]
    pub fn school_of<'a>(&'a self, engine_id: &str, default_id: &'a str) -> &'a str {
        self.schools.get(engine_id).map_or(default_id, |s| s.as_str())
    }
}

/// 叶的一个流派。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SchoolItem {
    /// 流派稳定 id（代码内使用，小写英数）。
    pub id: &'static str,
    /// 显示名（承接层展示）。
    pub name: &'static str,
    /// 是否默认流派（每叶应恰有一个默认）。
    pub default: bool,
    /// 一句说明：差异点 / 校验依据 / 流派归属。
    pub note: &'static str,
}

/// 构造 [`SchoolItem`] 的简写。
#[must_use]
pub const fn s(id: &'static str, name: &'static str, default: bool, note: &'static str) -> SchoolItem {
    SchoolItem { id, name, default, note }
}

/// C 族叶的有效种子：显式 `q.seed` 优先，否则由共享时刻的儒略日比特派生（同一时刻可复现）。
#[must_use]
pub fn effective_seed(m: &Moment, q: &Query) -> u64 {
    q.seed.unwrap_or_else(|| m.jd_ut.to_bits())
}
/// 一片叶：在共享上下文上排盘并产出统一 JSON。
pub trait CastingEngine: Send + Sync {
    /// 稳定标识（作为输出 map 的 key）。
    fn id(&self) -> &'static str;
    /// 显示名。
    fn name(&self) -> &'static str;
    /// 计算家族。
    fn family(&self) -> Family;
    /// 在共享上下文 `m` 上排盘，输出统一 JSON。
    fn cast(&self, m: &Moment, q: &Query) -> Value;
    /// 确定性谱：本叶各方面是确定/随机/欠定。默认空，每叶覆盖以显式声明 DET/STO/UND 边界。
    fn profile(&self) -> &'static [DetItem] {
        &[]
    }
    /// 本叶支持的流派集合（空=无流派分歧）；每叶应恰有一个 `default=true`。
    fn schools(&self) -> &'static [SchoolItem] {
        &[]
    }
}
/// 一片叶的带元数据输出（承接层展示用：id / 显示名 / 家族 / 盘）。
#[derive(Debug, Clone, Serialize)]
pub struct LeafOutput {
    /// 稳定标识。
    pub id: &'static str,
    /// 显示名。
    pub name: &'static str,
    /// 计算家族。
    pub family: Family,
    /// 家族中文标签。
    pub family_label: &'static str,
    /// 确定性谱（DET/STO/UND 边界）。
    pub profile: &'static [DetItem],
    /// 本叶支持的流派（空 = 无流派分歧）。
    pub schools: &'static [SchoolItem],
    /// 当前实际生效的流派 id（从 `q.schools` 取；若未指定，落到 default；无流派则空串）。
    pub effective_school: String,
    /// 排盘结果（统一 JSON）。
    pub chart: Value,
}

/// 取某叶在本次查询下实际生效的流派 id（未指定则落到该叶的 default，无流派则空串）。
#[must_use]
pub fn effective_school_id(e: &dyn CastingEngine, q: &Query) -> String {
    if let Some(sel) = q.schools.get(e.id()) {
        return sel.clone();
    }
    e.schools()
        .iter()
        .find(|s| s.default)
        .map_or_else(String::new, |s| s.id.to_string())
}
// ============================================================================
// 问局：QueryKind enum + 路由层 + 意图清单
// 与 `Query`（本命载荷）平行，声明 8 类问事意图(natal/fortune/event/election/
// synastry/mundane/locative/onomancy)。
// 算法树(profile/schools) 与 问局模型(intents/route) 同构对偶：前者声明「怎么算」，
// 后者声明「被谁调用」。同一批叶两种读法。
// ============================================================================

/// 占测时刻（用于 Event/Election/Locative/Fortune 等问局的「问的此刻」或时窗端点）。
///
/// 比 [`Query`] 简化：只携时间维原子，不带性别/坐标/姓名/种子；后者由各意图按需另带。
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct AskTime {
    /// 公历年。
    pub year: i32,
    /// 公历月 1..12。
    pub month: u32,
    /// 公历日 1..31。
    pub day: u32,
    /// 时 0..23。
    pub hour: u32,
    /// 分 0..59。
    pub minute: u32,
    /// 时区偏移小时。
    pub tz: f64,
}

/// 问局（需求侧）分类，按「时间轴与切面」模型组织。
///
/// 每变体携带其所需的**输入原子**；一切意图最终都映射到一组叶（由编排层的 `route` 在运行时定夺）。
/// `Natal` 是当前 webapp 唯一已实现形态，载荷复用现 [`Query`] 结构（向后兼容，API 不破坏）。
/// 其余变体携最小输入原子。
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QueryKind {
    /// **命**：本命盘（出生切片）。当前 webapp 唯一已实现形态。
    Natal(Query),
    /// **运**：本命 + 目标时刻 → 运势/流年/dasha 定位。
    Fortune {
        /// 出生切片。
        natal: Query,
        /// 目标时刻（流年/大运扫描端点）。
        t_target: AskTime,
    },
    /// **事**：占事（问事此刻 + 取机动作）。
    Event {
        /// 问事此刻。
        t_ask: AskTime,
        /// 取机种子（摇钱/抽牌/数蓍/random 派生）。
        seed: u64,
        /// 问句（只入释义不入算）。
        q_text: Option<String>,
    },
    /// **择**：时窗扫描 + 排序（择吉）。
    Election {
        /// 时窗起。
        window_start: AskTime,
        /// 时窗止。
        window_end: AskTime,
        /// 事类（婚/葬/动土/行/开业…）。
        category: String,
    },
    /// **合**：合盘（N=2 起，合婚/合伙）。
    Synastry {
        /// 甲方本命。
        a: Query,
        /// 乙方本命。
        b: Query,
    },
    /// **群/国**：政体奠基时刻 → 国运盘。
    Mundane {
        /// 政体奠基时刻（立国/开国大典/政权更替）。
        p_polity: Query,
    },
    /// **寻**：取方位（占课为主）。
    Locative {
        /// 问事此刻。
        t_ask: AskTime,
        /// 取机种子。
        seed: u64,
        /// 事类（寻人/寻物/寻方向）。
        category: String,
    },
    /// **号**：字/词模态（姓名笔画/字母值，与时刻无关）。
    Onomancy {
        /// 姓名（数字学/gematria/abjad 字母值）。
        name: String,
        /// 姓笔画（五格用，可选）。
        surname_strokes: Option<u32>,
        /// 名笔画（五格用，可选）。
        given_strokes: Option<u32>,
    },
}

impl QueryKind {
    /// 取意图稳定 id。
    #[must_use]
    pub fn id(&self) -> &'static str {
        match self {
            Self::Natal(_) => "natal",
            Self::Fortune { .. } => "fortune",
            Self::Event { .. } => "event",
            Self::Election { .. } => "election",
            Self::Synastry { .. } => "synastry",
            Self::Mundane { .. } => "mundane",
            Self::Locative { .. } => "locative",
            Self::Onomancy { .. } => "onomancy",
        }
    }
}

/// 意图的实现状态：Live（已上线）/Pending（结构已声明、算力已在叶里、尚无承接端点）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum IntentStatus {
    /// 🟢 已上线：Natal 在所有现有端点工作。
    Live,
    /// 🟡 待承接：算力已在叶里，尚无对应端点形态。
    Pending,
}

impl IntentStatus {
    /// 中文标签。
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Live => "已上线",
            Self::Pending => "待承接",
        }
    }
}

/// 问局意图规格：每意图所需输入原子 + 默认路由叶 + 输出形态 + 实现状态。
///
/// 与 [`DetItem`]/[`SchoolItem`] 同构对偶：profile/schools 声明「怎么算」（供给侧），
/// intents 声明「被谁调用」（需求侧）。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct IntentSpec {
    /// 稳定 id(snake_case)。
    pub id: &'static str,
    /// 中文显示名。
    pub name_zh: &'static str,
    /// 所需输入原子(instant/geo/sex/seed/text/category/window…)，用于 web 表单生成。
    pub atoms: &'static [&'static str],
    /// 默认路由叶（声明式；运行时实际可用以编排层 `route` 的输出 ∩ 当前注册表为准）。
    pub default_leaves: &'static [&'static str],
    /// 输出形态（盘/势/断/期/序/配/位）。
    pub output_shape: &'static str,
    /// 实现状态。
    pub status: IntentStatus,
    /// 一句说明（算力在哪、还差什么）。
    pub note: &'static str,
}

/// 构造 [`IntentSpec`] 的简写（crate 私有）。
const fn i(
    id: &'static str,
    name_zh: &'static str,
    atoms: &'static [&'static str],
    default_leaves: &'static [&'static str],
    output_shape: &'static str,
    status: IntentStatus,
    note: &'static str,
) -> IntentSpec {
    IntentSpec { id, name_zh, atoms, default_leaves, output_shape, status, note }
}

/// 8 类问事意图的清单（声明式，与编排层的 `route` 同构对偶）。
///
/// 顺序：Natal / Fortune / Event / Election / Synastry / Mundane / Locative /
/// Onomancy（D 族字/词，与时刻无关）。
#[must_use]
pub fn intents() -> &'static [IntentSpec] {
    use IntentStatus::{Live, Pending};
    const { &[
        i(
            "natal", "命（本命盘）",
            &["instant", "geo", "sex", "text(name)"],
            // 全 21 叶。守卫见 tests::intents_natal_covers_registry。
            &[
                "bazi", "ziwei", "astrology", "jyotish", "qizhengsiyu",
                "yijing", "geomancy", "sikidy", "ifa", "tarot", "meihua",
                "xiaoliuren", "zeri", "maya", "pawukon", "mahabote",
                "liuren", "qimen", "taiyi", "tibetan", "numerology",
            ],
            "盘（静态切片，全树并行 fan-out）", Live,
            "出生切片；webapp 唯一已实现意图，/api/cast 全 21 叶",
        ),
        i(
            "fortune", "运（运势/流年/大运）",
            &["instant(birth)", "instant(target)", "sex"],
            &["bazi", "ziwei", "jyotish", "astrology"],
            "势（时间序列，playhead 切片）", Live,
            "/api/fortune 给本命+t切片+大运段定位+运层旺衰+用神供给度+100年时序；算力底层走 bazi.fortune_at + fortune_supply_timeline",
        ),
        i(
            "event", "事（占事）",
            &["instant(ask)", "seed(draw)", "text(question)"],
            &["yijing", "meihua", "liuren", "qimen", "tarot", "geomancy", "ifa", "sikidy"],
            "断（成败/吉凶/宜忌）", Pending,
            "卜筮叶 effective_seed 能力已在，差「问事此刻 + 取机动作」UI 与释义「断」模板；判词 🟡 交 LLM",
        ),
        i(
            "election", "择（择吉）",
            &["window(start, end, grain)", "category（婚/葬/动土/行/开业…）"],
            &["zeri", "xiaoliuren"],
            "期/序（候选日按吉凶排名）", Pending,
            "zeri 单点正算已绿，差时窗扫描 + 排序；事类宜忌口诀 🟡",
        ),
        i(
            "synastry", "合（合盘）",
            &["instant(a)", "instant(b)", "sex(a,b)"],
            &["bazi", "astrology", "jyotish"],
            "配（契合度/互补结构）", Pending,
            "bazi 已有 /api/team N×N 互补矩阵雏形；astrology 合盘几何相位待加；契合度权重 🟡",
        ),
        i(
            "mundane", "群/国（国运）",
            &["instant(polity)", "geo"],
            &["taiyi", "qimen", "astrology"],
            "势（国运势卜/年度盘）", Pending,
            "太乙是国运首选术，从被当本命喂纠正为国运盘；qimen 国家奇门 + astrology mundane",
        ),
        i(
            "locative", "寻（寻方位）",
            &["instant(ask)", "seed(draw)", "category（寻人/物/向）"],
            &["liuren", "qimen", "xiaoliuren"],
            "位（方位/卦象）", Pending,
            "六壬/奇门起课算力在，差「方位」输出形态；取传/方位判读 🟡",
        ),
        i(
            "onomancy", "号（字/词）",
            &["text(name)", "strokes（姓笔画， 名笔画）"],
            // numerology 已在 registry；gematria/abjad/wuge 是 D 族字词库 crate，
            // /api/word 端点不在 cast registry 内。
            &["numerology"],
            "号（数字学生命灵数/姓名值/五格）", Live,
            "/api/word 端点已实现 gematria/abjad/wuge，numerology 在 cast registry。本意图已部分上线",
        ),
    ] }
}


/// 字/词模态的一次查询（D 族：与出生时刻无关，吃文字或笔画）。
#[derive(Debug, Clone, Default, Serialize, serde::Deserialize)]
pub struct WordQuery {
    /// 待取值的词（希伯来 gematria / 阿拉伯 abjad）。
    pub text: Option<String>,
    /// 姓各字笔画（五格用）。
    pub surname: Option<Vec<u32>>,
    /// 名各字笔画（五格用）。
    pub given: Option<Vec<u32>>,
}

/// 一片**字词叶**：不吃共享时刻，只吃文字/笔画。
///
/// 与 [`CastingEngine`] 平行的第二个端口——D 族里 gematria / abjad / wuge 这三片叶
/// 与时间无关，无法进入 moment fan-out，于是单列一条契约。
pub trait WordEngine: Send + Sync {
    /// 稳定标识（HTTP `system` 字段与输出 key）。
    fn id(&self) -> &'static str;
    /// 显示名。
    fn name(&self) -> &'static str;
    /// 取值。输入不足时返回 `Err` 并给出面向调用方的中文说明。
    ///
    /// # Errors
    ///
    /// 当查询缺少该叶必需的输入原子时返回错误说明（如五格缺姓或名的笔画）。
    fn compute(&self, q: &WordQuery) -> Result<Value, String>;
    /// 确定性谱。默认空，每叶覆盖。
    fn profile(&self) -> &'static [DetItem] {
        &[]
    }
}

