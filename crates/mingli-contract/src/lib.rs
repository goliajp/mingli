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
///
/// 线上一律小写。这个枚举原本按 Rust 的拼法收发，于是直接吃 [`Query`] JSON 的两个入口
/// （`/api/route` 与 wasm 的 `parse_query`）只认 `"Male"`，而 HTTP 的 DTO 层、web 的
/// `types.ts`、各叶回声里的 `input.gender` 说的都是 `"male"`——同一个词在同一套 API 里
/// 有两种拼法，写不对的那一头会收到 422。`Male` / `Female` 仍以别名接受，旧调用不破。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    /// 男。
    #[serde(alias = "Male", alias = "男")]
    Male,
    /// 女。
    #[serde(alias = "Female", alias = "女")]
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
    /// 只带时刻的最小查询：性别 / 坐标 / 种子 / 姓名 / 流派全部缺省。
    ///
    /// 需要这些原子的叶自会在缺省下走它的降级路径（如八字不排大运、占星不出 Asc）。
    #[must_use]
    pub fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32, tz: f64) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            tz,
            gender: None,
            latitude: None,
            longitude: None,
            seed: None,
            name: None,
            schools: BTreeMap::new(),
        }
    }

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
/// 一个方位候选：盘上的某个要素落在哪一方。
///
/// 「寻方位」这个意图（[`QueryKind::Locative`]）要的是**结构**——哪个要素落在哪一宫、
/// 那一宫朝哪个方向——至于所寻之事该取哪一宫为用，各家不同，属判读，不在本层。
///
/// 这是端口层的词汇而不是某片叶的：奇门读值符值使与门奇、六壬读三传之支、小六壬读所落之宫，
/// 三者的**盘**毫无共同之处，但产出的**候选**是同一种东西。用例层只认这个形状，
/// 不必知道候选是从哪种盘上怎么读出来的。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Bearing {
    /// 来源叶的稳定 id。
    pub leaf: &'static str,
    /// 要素名（值符 / 值使 / 开门 / 乙奇 / 初传 …）。
    pub element: String,
    /// 落点的字面（奇门的「坎1」、六壬的「子」）。
    pub at: String,
    /// 方位。
    pub direction: &'static str,
    /// 附注：同宫的门 / 星 / 神 / 旺衰等结构事实，供判读。
    pub note: String,
}

/// 一片叶的**主判据**：这套系统据以起论的那个低基数分类量。
///
/// 四柱取日支（12 值）、紫微取命宫支（12）、西洋占星取太阳所在星座（12）、
/// 印度占星取月宿（27）、择日取建除（12）——每套系统都有这么一个「先看哪里」的量，
/// 它是该系统自己的领域概念，不是为了给谁做统计才有的。
///
/// 跨叶做信息论比较时正好用得上它（见 `mingli-analysis`），但那只是一个消费者：
/// 即使没有任何统计，「这套系统起论看哪一个量」仍然是这片叶该回答的问题。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Principal {
    /// 这个量叫什么（如「日支」「命宫支」「太阳星座」）。
    pub label: &'static str,
    /// 本次取值。取值域应当是低基数的有限集合。
    pub value: String,
}

/// 一片叶：在共享上下文上排盘并产出统一 JSON，并声明自己答什么、据什么起论。
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
    /// 从本叶的盘上读出方位候选；本叶与方位无关则返回空（默认）。
    ///
    /// 用例层的「寻方位」靠这个方法而不是去解析各叶的输出 JSON——叶改个字段名，
    /// 解析 JSON 的写法**不会编译报错**，只会静默少出候选；走这里则由类型系统盯着。
    fn bearings(&self, _m: &Moment, _q: &Query) -> Vec<Bearing> {
        Vec::new()
    }
    /// 本叶支持的流派集合（空=无流派分歧）；每叶应恰有一个 `default=true`。
    fn schools(&self) -> &'static [SchoolItem] {
        &[]
    }
    /// 本叶答哪几类问局。缺省只答[`Intent::Natal`]——每片时刻叶都能给出生切片。
    ///
    /// 编排层据此路由（见 `mingli_engine::route`）：加一片叶时，它答什么由它自己说，
    /// 不需要回头改端口层或编排层的任何清单。
    ///
    /// **判定标准是「当下算得出」，不是「传统上该答」。** 这条声明直接决定运行时路由，
    /// 声明了却产不出东西就是空跑，比少声明糟得多；而「算不算得出」看本叶的实现即可判定，
    /// 「传统上该不该答」则要考据，按本项目的规矩需要 ≥2 个独立来源。
    ///
    /// 某类问局这套系统传统上确实用得着、只是本叶还没实现，那是 [`profile`](CastingEngine::profile)
    /// 里一条 🟡 [`Determinism::Und`] 该说的话——并且要按规矩分清是「查过定不下」还是「还没查」。
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal]
    }
    /// 本叶盘面的读法：各字段是什么、传统上先看哪一处。没有则返回 `None`（默认）。
    ///
    /// 这是本叶的领域知识——「`strength.wuxing` 是五行力量分布、缺者宜补旺者宜泄」这种话，
    /// 只有写这片叶的人说得准。释义层把它原样交给后端，自己不攒也不改。
    fn reading_notes(&self) -> Option<&'static str> {
        None
    }
    /// 同一套计算换个主体读时的象义重映射（公司 / 物 / 事）。默认无——多数叶不含
    /// 宫位、十神、六亲这类随主体改变所指的概念，对它们 person 与其余主体等价。
    fn subject_notes(&self, _subject: Subject) -> Option<&'static str> {
        None
    }
    /// 本叶的[主判据][`Principal`]；本叶没有这样一个量则返回 `None`（默认）。
    ///
    /// 实现应当从自己的**强类型盘面**取，不要去解自己输出的那份 JSON——
    /// 改个字段名时，前者编译报错，后者只会静默失灵。
    fn principal(&self, _m: &Moment, _q: &Query) -> Option<Principal> {
        None
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
    /// 取本问局属于哪一类意图。
    #[must_use]
    pub fn intent(&self) -> Intent {
        match self {
            Self::Natal(_) => Intent::Natal,
            Self::Fortune { .. } => Intent::Fortune,
            Self::Event { .. } => Intent::Event,
            Self::Election { .. } => Intent::Election,
            Self::Synastry { .. } => Intent::Synastry,
            Self::Mundane { .. } => Intent::Mundane,
            Self::Locative { .. } => Intent::Locative,
            Self::Onomancy { .. } => Intent::Onomancy,
        }
    }

    /// 取意图稳定 id。
    #[must_use]
    pub fn id(&self) -> &'static str {
        self.intent().id()
    }
}

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

/// 八类问局。
///
/// 与 [`QueryKind`] 的关系：`QueryKind` 携带该问局**要哪些输入原子**，`Intent` 只是它的标签，
/// 用来回答「哪片叶答这一类」。做成枚举而不是字符串，是为了让「一片叶声明它答什么」这件事
/// 由类型系统盯着——写错一个字符串，那片叶会静默地什么都不答。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    /// 命：本命盘（出生切片）。
    Natal,
    /// 运：运势 / 流年 / 大运。
    Fortune,
    /// 事：占事。
    Event,
    /// 择：择吉。
    Election,
    /// 合：合盘。
    Synastry,
    /// 群/国：国运。
    Mundane,
    /// 寻：寻方位。
    Locative,
    /// 号：字 / 词（与时刻无关）。
    Onomancy,
}

impl Intent {
    /// 稳定 id（snake_case），与线上字面量一致。
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Natal => "natal",
            Self::Fortune => "fortune",
            Self::Event => "event",
            Self::Election => "election",
            Self::Synastry => "synastry",
            Self::Mundane => "mundane",
            Self::Locative => "locative",
            Self::Onomancy => "onomancy",
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
    /// 这一类问局。
    pub id: Intent,
    /// 中文显示名。
    pub name_zh: &'static str,
    /// 所需输入原子(instant/geo/sex/seed/text/category/window…)，用于 web 表单生成。
    pub atoms: &'static [&'static str],
    /// 输出形态（盘/势/断/期/序/配/位）。
    pub output_shape: &'static str,
    /// 实现状态。
    pub status: IntentStatus,
    /// 一句说明。
    pub note: &'static str,
}

/// 构造 [`IntentSpec`] 的简写（crate 私有）。
const fn i(
    id: Intent,
    name_zh: &'static str,
    atoms: &'static [&'static str],
    output_shape: &'static str,
    status: IntentStatus,
    note: &'static str,
) -> IntentSpec {
    IntentSpec { id, name_zh, atoms, output_shape, status, note }
}

/// 8 类问事意图的清单（声明式，与编排层的 `route` 同构对偶）。
///
/// 顺序：Natal / Fortune / Event / Election / Synastry / Mundane / Locative /
/// Onomancy（D 族字/词，与时刻无关）。
#[must_use]
pub fn intents() -> &'static [IntentSpec] {
    use IntentStatus::Live;
    const { &[
        i(
            Intent::Natal, "命（本命盘）",
            &["instant", "geo", "sex", "text(name)"],
            "盘（静态切片，全树并行 fan-out）", Live,
            "一个时刻的静态切片。全部时刻叶都答这一类，故它也是一片叶不作声明时的缺省",
        ),
        i(
            Intent::Fortune, "运（运势/流年/大运）",
            &["instant(birth)", "instant(target)", "sex"],
            "势（时间序列，playhead 切片）", Live,
            "本命固定、目标时刻在动：同一张底盘上取某一刻的切片，并沿时间轴铺成序列",
        ),
        i(
            Intent::Event, "事（占事）",
            &["instant(ask)", "seed(draw)", "text(question)"],
            "断（成败/吉凶/宜忌）", Live,
            "问事此刻加一次取机；取机的种子入盘，故同一次占问可复现",
        ),
        i(
            Intent::Election, "择（择吉）",
            &["window(start, end, grain)", "category（婚/葬/动土/行/开业…）"],
            "期/序（候选日按吉凶排名）", Live,
            "在一段时窗上逐日取要素并分档。事类宜忌各家出入大，不合成总分",
        ),
        i(
            Intent::Synastry, "合（合盘）",
            &["instant(a)", "instant(b)", "sex(a,b)"],
            "配（契合度/互补结构）", Live,
            "两张本命之间的互供关系，两个方向分别成立，不对称是常态",
        ),
        i(
            Intent::Mundane, "群/国（国运）",
            &["instant(polity)", "geo"],
            "势（国运势卜/年度盘）", Live,
            "以政体奠基时刻为起点的周期结构，沿年份展开；描述的是周期位置，不是对现实的断言",
        ),
        i(
            Intent::Locative, "寻（寻方位）",
            &["instant(ask)", "seed(draw)", "category（寻人/物/向）"],
            "位（方位/卦象）", Live,
            "于问事此刻起课，从盘上抽方位候选（奇门取值符 / 值使 / 三吉门 / 三奇落宫 → 后天八卦方位，六壬取三传或四课上神 → 十二支方位）；取用之法各家不同，不合成排名",
        ),
        i(
            Intent::Onomancy, "号（字/词）",
            &["text(name)", "strokes（姓笔画， 名笔画）"],
            "号（数字学生命灵数/姓名值/五格）", Live,
            "唯一不吃时刻的一类：入参是字与笔画，故走 WordEngine 而非 CastingEngine；\
             数字学同时吃出生日期，因此它两边都在",
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn every_label_is_populated() {
        for f in [Family::Cyclic, Family::Angular, Family::Sampling, Family::Hashing, Family::CrossCutting] {
            assert!(!f.label().is_empty());
        }
        for d in [Determinism::Det, Determinism::Sto, Determinism::Und] {
            assert!(!d.label().is_empty());
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
        // status 标签非空。
        assert!(!IntentStatus::Live.label().is_empty());
        assert!(!IntentStatus::Pending.label().is_empty());
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
}
