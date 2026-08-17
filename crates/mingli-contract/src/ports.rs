//! 两个端口本身。
//!
//! [`CastingEngine`] 是吃时刻的那一条，[`WordEngine`] 是吃字与笔画的那一条；
//! 一片叶实现其一，编排层只认这两个 trait，双方都不认识对方。

use crate::{DetItem, Family, Intent, Moment, Query, SchoolItem, Subject};
use serde::Serialize;
use serde_json::Value;


///
/// 「寻方位」这个意图（[`crate::QueryKind::Locative`]）要的是**结构**——哪个要素落在哪一宫、
/// 那一宫朝哪个方向——至于所寻之事该取哪一宫为用，各家不同，属判读，不在本层。
///
/// 这是端口层的词汇而不是某片叶的：奇门读值符值使与门奇、六壬读三传之支、小六壬读所落之宫，
/// 三者的**盘**毫无共同之处，但产出的**候选**是同一种东西。用例层只认这个形状，
/// 不必知道候选是从哪种盘上怎么读出来的。
/// 一个方位候选：盘上的某个要素落在哪一方。
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
    /// **判定标准是「当下算得出这一类的 [`output_shape`](crate::IntentSpec::output_shape)」，
    /// 不是「传统上该答」。** 这条声明直接决定运行时路由，声明了却产不出东西就是空跑，
    /// 比少声明糟得多；而「算不算得出」看本叶的实现即可判定，「传统上该不该答」则要考据，
    /// 按本项目的规矩需要 ≥2 个独立来源。
    ///
    /// 「算得出」要落到那个形态上，不是「沾边」：一片叶能给某个时刻的值，不等于它答得起
    /// 「势（时间序列）」；能给某个神煞落宫，不等于它答得起「位（方位）」——后者要真的
    /// 产出方位候选（[`bearings`](CastingEngine::bearings)）。
    ///
    /// 某类问局这套系统传统上确实用得着、只是本叶还没实现，那是 [`profile`](CastingEngine::profile)
    /// 里一条 🟡 [`crate::Determinism::Und`] 该说的话——并且要按规矩分清是「查过定不下」还是「还没查」。
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
