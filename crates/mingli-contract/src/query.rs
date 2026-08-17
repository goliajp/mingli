//! 一次排盘的输入：时刻与它周围那些可缺省的原子。
//!
//! [`Query`] 是全叶共享的那一份；[`AskTime`] 是只带时间维的简化版；
//! [`QueryKind`] 把输入按问局归类，携带各类各自需要的原子。
//! 取机种子与主体类型也在这里——它们是输入，不是某一层的私产。

use crate::{Intent, Moment};
use serde::Serialize;
use std::collections::BTreeMap;

/// 性别（用于需要它的叶，如八字大运）。
///
/// 线上一律小写。这个枚举原本按 Rust 的拼法收发，于是凡是直接把 [`Query`] 从 JSON 解出来的
/// 地方只认 `"Male"`，而各叶盘里回声出去的 `input.gender` 写的是 `"male"`——同一个词在
/// 同一套契约里有两种拼法，写错的那一头会被拒。`Male` / `Female` 与 `男` / `女`
/// 都以别名接受，旧调用不破。
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

/// C 族叶的有效种子：显式 `q.seed` 优先，否则由共享时刻的儒略日比特派生（同一时刻可复现）。
#[must_use]
pub fn effective_seed(m: &Moment, q: &Query) -> u64 {
    q.seed.unwrap_or_else(|| m.jd_ut.to_bits())
}

// 以下是按问局归类的输入。与 `Query`（本命载荷）平行：`Query` 是「一个时刻加它周围的原子」，
// `QueryKind` 是「这次问的是哪一类，因而要哪些原子」。哪几片叶答某一类不在这里，见
// `crate::ports::CastingEngine::answers`。

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
/// `Natal` 直接复用 [`Query`] 作载荷——「一个时刻的切片」要的原子与共享输入恰好相同；
/// 其余变体各携该类问局的最小输入原子。
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QueryKind {
    /// **命**：本命盘——一个时刻的静态切片。
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
