//! 需求侧：有哪几类问局，各自要什么输入原子、出什么形态。
//!
//! 这里**不说谁来答**——那是各叶自己的声明（[`crate::CastingEngine::answers`]），
//! 由编排层在运行时合成。端口层若列出叶名，加一片叶就得回头改端口层，
//! 而漏改不报错：那片叶只是静默地不入任何路由。

use serde::Serialize;


/// 八类问局。
///
/// 与 [`crate::QueryKind`] 的关系：`QueryKind` 携带该问局**要哪些输入原子**，`Intent` 只是它的标签，
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
/// 与 [`crate::DetItem`]/[`crate::SchoolItem`] 同构对偶：profile/schools 声明「怎么算」（供给侧），
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
            "在一段时窗上逐日取要素并分档。事类宜忌各家出入大，不合成总分；\
             各家分档的粒度与判据也不同，故这一类目前由单叶作答，合成总排名等于替读者选边",
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
