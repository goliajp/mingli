//! 群 / 国运用例：政体奠基时刻 → 立国盘 + 太乙行宫时间线 + 目标年的年度盘。
//!
//! 国运不是一张静态盘。太乙三年居一宫、廿四年转一周，本身就是**年分辨率的势**：
//! 以立国时刻起盘得「体」，再沿年份把太乙所居之宫铺开得「势」，目标年那一格就是「年度盘」。
//!
//! 只出结构：宫 / 卦 / 三才 / 入宫年数 / 阴阳遁。宫的吉凶与「对该国意味着什么」交释义层，
//! 且释义层被要求克制——这是周期结构的描述，不是对现实政体的预言。

use mingli_contract::{AskTime, CastingEngine, LeafOutput, Query, QueryKind};
use serde::Serialize;
use std::collections::BTreeMap;

/// 时间线上的一年。
#[derive(Debug, Clone, Serialize)]
pub struct YearStep {
    /// 公历年。
    pub year: i32,
    /// 立国后第几年（立国年为 0）。
    pub age: i32,
    /// 太乙所居洛书宫数（1..9，不入 5）。
    pub palace: u8,
    /// 该宫八卦名。
    pub gua: &'static str,
    /// 入宫年数 1..=3。
    pub year_in_palace: u8,
    /// 三才（理天 / 理地 / 理人）。
    pub sancai: &'static str,
    /// 阳遁 / 阴遁。
    pub yang_dun: bool,
    /// 是否是换宫之年（入宫第一年）。
    pub enters_palace: bool,
}

/// 一次国运推演的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Mundane {
    /// 奠基时刻。
    pub founded_at: AskTime,
    /// 目标年（年度盘所在年）。
    pub target_year: i32,
    /// 立国盘：按意图路由的各叶在奠基时刻的盘。
    pub founding: Vec<LeafOutput>,
    /// 目标年的年度太乙盘（同为奠基之日的年周年时刻）。
    pub annual: Option<YearStep>,
    /// 太乙行宫时间线：自立国年起共 `span` 年。
    pub timeline: Vec<YearStep>,
    /// 时间线覆盖的年数。
    pub span: u32,
}

/// 时间线默认长度：太乙廿四年转一周。
const DEFAULT_SPAN: u32 = 24;
/// 时间线上限：三期七十二年。
const MAX_SPAN: u32 = 72;

/// 某一年的太乙一步。
///
/// 直连太乙叶取**强类型**的盘，不再从它的输出 JSON 里按字符串键抠字段。
/// 从前这里读回 `gua` / `sancai` 的字符串后，要在一张硬编码表里按名找回 `'static` 引用，
/// 查不到就静默返回空串——那张表抄的还是洛书的卦名，而太乙用的是自家的九宫配法。
/// 用例层依赖具体领域实体是允许方向（同 bazi / ziwei / zeri），比抄一份叶的词汇表干净。
fn step_for(year: i32, founded: &AskTime) -> YearStep {
    let m = mingli_astro::Moment::new(
        year, founded.month, founded.day, founded.hour, founded.minute, founded.tz,
    );
    let c = mingli_taiyi::compute_at(&m);
    YearStep {
        year,
        age: year - founded.year,
        palace: c.taiyi.palace,
        gua: c.taiyi.gua,
        year_in_palace: c.taiyi.year_in_palace,
        sancai: c.taiyi.sancai,
        yang_dun: c.yang_dun,
        enters_palace: c.taiyi.year_in_palace == 1,
    }
}

/// 国运：奠基时刻起立国盘，沿年份铺太乙行宫，给出目标年的年度盘。
///
/// `span` 缺省 24（一周）、上限 72（三期）。时间线**始终包含目标年**：若目标年落在自立国起的
/// 头一段之外，窗口整段后移到目标年所在的那一周（保持廿四年对齐），这样年度盘那一格总在图上。
///
/// # Errors
///
/// 注册表内没有可路由的叶，或 `span` 为 0 时返回说明。
pub fn cast(
    reg: &[Box<dyn CastingEngine>],
    founded: &AskTime,
    latitude: Option<f64>,
    longitude: Option<f64>,
    target_year: Option<i32>,
    span: Option<u32>,
) -> Result<Mundane, String> {
    let span = span.unwrap_or(DEFAULT_SPAN).min(MAX_SPAN);
    if span == 0 {
        return Err("时间线至少 1 年".into());
    }
    let polity = Query {
        year: founded.year,
        month: founded.month,
        day: founded.day,
        hour: founded.hour,
        minute: founded.minute,
        tz: founded.tz,
        gender: None,
        latitude,
        longitude,
        seed: None,
        name: None,
        schools: BTreeMap::new(),
    };
    let ids = mingli_engine::route(reg, &QueryKind::Mundane { p_polity: polity.clone() });
    if ids.is_empty() {
        return Err("当前注册表内没有可用于国运的叶".into());
    }
    let founding = ids.iter().filter_map(|id| mingli_engine::cast_one(reg, id, &polity)).collect();
    let target_year = target_year.unwrap_or(founded.year);
    // 窗口起点：立国年，或按廿四年对齐后移到能盖住目标年的那一周
    let span_i = i32::try_from(span).unwrap_or(i32::MAX);
    let start = if target_year >= founded.year + span_i {
        founded.year + ((target_year - founded.year) / 24) * 24
    } else {
        founded.year
    };
    let timeline: Vec<YearStep> = (0..span_i).map(|k| step_for(start + k, founded)).collect();
    debug_assert!(!timeline.is_empty(), "span 已经过下界校验，时间线不该为空");
    let annual = (target_year >= founded.year).then(|| step_for(target_year, founded));
    Ok(Mundane { founded_at: founded.clone(), target_year, founding, annual, timeline, span })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mingli_registry::registry;

    /// 1949-10-01 15:00 北京：一个众所周知的奠基时刻，只用作结构测试的锚。
    fn prc() -> AskTime {
        AskTime { year: 1949, month: 10, day: 1, hour: 15, minute: 0, tz: 8.0 }
    }

    #[test]
    fn routes_to_taiyi_qimen_astrology() {
        let m = cast(&registry(), &prc(), Some(39.9), Some(116.4), None, None).expect("应可推");
        let ids: Vec<&str> = m.founding.iter().map(|l| l.id).collect();
        assert_eq!(ids, ["taiyi", "qimen", "astrology"]);
    }

    #[test]
    fn timeline_walks_the_palaces_three_years_at_a_time() {
        let m = cast(&registry(), &prc(), None, None, None, None).expect("应可推");
        assert_eq!((m.span, m.timeline.len()), (24, 24));
        assert_eq!(m.timeline[0].age, 0);
        // 三年一宫：入宫年数沿 1,2,3,1,2,3… 循环，换宫之年恰是 year_in_palace == 1
        for w in m.timeline.windows(2) {
            let (a, b) = (&w[0], &w[1]);
            assert_eq!(b.age, a.age + 1);
            if a.year_in_palace < 3 {
                assert_eq!((b.palace, b.year_in_palace), (a.palace, a.year_in_palace + 1));
                assert!(!b.enters_palace);
            } else {
                assert_ne!(b.palace, a.palace, "住满三年必换宫");
                assert_eq!(b.year_in_palace, 1);
                assert!(b.enters_palace);
            }
            assert_ne!(b.palace, 5, "太乙不入中宫");
        }
        // 廿四年恰好八宫各居三年
        let mut seen: Vec<u8> = m.timeline.iter().map(|s| s.palace).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 8);
    }

    #[test]
    fn annual_chart_is_the_target_years_step() {
        let m = cast(&registry(), &prc(), None, None, Some(2026), None).expect("应可推");
        let a = m.annual.expect("目标年在立国后应有年度盘");
        assert_eq!((a.year, a.age), (2026, 77));
        // 与时间线同一算法：把 span 拉到 78 年，第 77 格应与之一致
        let long = cast(&registry(), &prc(), None, None, Some(2026), Some(78)).expect("应可推");
        assert_eq!(long.span, 72, "上限三期七十二年");
        assert!(matches!(cast(&registry(), &prc(), None, None, Some(1900), None), Ok(Mundane { annual: None, .. })), "目标年早于立国无年度盘");
    }

    #[test]
    fn timeline_always_contains_the_target_year() {
        let m = cast(&registry(), &prc(), None, None, Some(2026), None).expect("应可推");
        assert_eq!(m.timeline.len(), 24);
        assert!(m.timeline.iter().any(|s| s.year == 2026), "目标年应在时间线内");
        // 窗口按廿四年对齐：1949 + 72 = 2021 起
        assert_eq!(m.timeline[0].year, 2021);
        assert_eq!(m.timeline[0].age % 24, 0);
        // 目标年在头一段内时窗口不动
        let early = cast(&registry(), &prc(), None, None, Some(1960), None).expect("应可推");
        assert_eq!(early.timeline[0].year, 1949);
    }

    #[test]
    fn span_bounds() {
        assert!(cast(&registry(), &prc(), None, None, None, Some(0)).is_err());
        let one = cast(&registry(), &prc(), None, None, None, Some(1)).expect("一年应可");
        assert_eq!(one.timeline.len(), 1);
    }
}
