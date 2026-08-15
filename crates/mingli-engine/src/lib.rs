//! L3.5 编排：把命理大树当作一张记忆化计算 DAG。
//!
//! - 共享层：一个输入 → 用 [`mingli_astro::Moment`] 把公共天文/历法子计算**算一次**。
//! - fan-out：注册表里每片叶（[`CastingEngine`]）在该共享上下文上排盘，**rayon 并行**。
//! - 统一输出：各叶输出 `serde_json::Value`，便于跨叶对齐比较（相关性分析）。
//!
//! 加新叶 = 实现 [`CastingEngine`] 并加入 [`registry`]，根与共享层不动。

use mingli_astro::Moment;
use rayon::prelude::*;
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
const fn d(aspect: &'static str, status: Determinism, note: &'static str) -> DetItem {
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

fn bazi_gender(g: Option<Gender>) -> Option<mingli_bazi::Gender> {
    g.map(|x| match x {
        Gender::Male => mingli_bazi::Gender::Male,
        Gender::Female => mingli_bazi::Gender::Female,
    })
}

fn ziwei_gender(g: Option<Gender>) -> Option<mingli_ziwei::Gender> {
    g.map(|x| match x {
        Gender::Male => mingli_ziwei::Gender::Male,
        Gender::Female => mingli_ziwei::Gender::Female,
    })
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

/// 四柱八字叶。
#[derive(Debug, Default)]
pub struct BaziEngine;

impl CastingEngine for BaziEngine {
    fn id(&self) -> &'static str {
        "bazi"
    }
    fn name(&self) -> &'static str {
        "四柱八字"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        use mingli_bazi::{BaziSchool, YearBreakMethod, ZiHourMethod};
        let school = match q.school_of(self.id(), "late_lichun") {
            "late_sf" => BaziSchool { zi_hour: ZiHourMethod::Late, year_break: YearBreakMethod::SpringFestival },
            "early_lichun" => BaziSchool { zi_hour: ZiHourMethod::Early, year_break: YearBreakMethod::LiChun },
            "early_sf" => BaziSchool { zi_hour: ZiHourMethod::Early, year_break: YearBreakMethod::SpringFestival },
            _ => BaziSchool::default(),
        };
        serde_json::to_value(mingli_bazi::compute_at_school(m, bazi_gender(q.gender), school))
            .unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("四柱·五行·十神", Det, "干支递推+节气界，校验 庚午/壬午/辛亥/乙未"),
            d("大运", Det, "节气距÷3 起运，阳男阴女顺、阴男阳女逆"),
            d("子时归属（可选）", Det, "晚子（主流）23-24→次日；早子（少数）23-24→当日"),
            d("年柱换岁（可选）", Det, "立春（主流）315°黄经；春节（民间）农历正月初一"),
            d("纳音·空亡", Det, "纳音=干支分组求和，空亡=日柱旬末两支(DET)"),
            d("地支藏干（人元）", Det, "通行「不分日固定表」，多源校验四源一致；🟡 巳/申异说取主流"),
            d("支藏十神", Det, "藏干对日主复用 ten_god；oracle 丁卯己酉己巳壬申 全验"),
            d("十二长生", Det, "传统派：阳干顺、阴干逆；校验各干临官=禄位；🟡 新派阴干顺行未开关"),
            d("旺衰量化（得令/得地/得势）", Und, "扶抑学派范式：三栏 0-30+ 综合 0-100；权重表无统一标准（各家月令权重 30%-60% 不一），本算法显式声明默认"),
            d("五行力量分布", Und, "天干 10/地支本 12/中 6/余 3/月支×1.5；权重同旺衰算法，流派分歧"),
            d("岁运叠加旺衰", Und, "本命基础上拼大运柱+流年柱，得令固定取本命月支；岁运折扣权重流派分歧"),
            d("格局（八正格/禄刃/暗格）", Det, "月令藏干透干：本→中→余，先透先定；月令本气=日主同五行 → 建禄/月刃；7 oracle 含 1987 暗食神/1990 暗七杀+5 构造透干分支"),
            d("从格/化格/专旺格/杂气", Und, "成立条件流派分歧大（身极弱+无救助、化神有无、月令分日用事），不机械化，留 INT 释义层"),
            d("用神/喜忌（扶抑+调候）", Und, "身强宜耗（官杀/财/食伤，取盘中最缺者）、身弱宜扶（印星优先）、中和走调候（寒月火/燥月水/春木金/秋金火/杂气日主同）；忌神=反方向。流派（扶抑/调候/通关/病药）无统一先后，本算法显式默认"),
            d("真太阳时校正（可选）", Det, "出生地经度差(±4 min/°)+ Spencer 均时差(±0.5 min) → 校正钟表时；跨时辰边界时时柱变。校验长沙 1987-09-17 15：00 钟表壬申 → 真太阳辛未"),
            d("三宫（命宫/身宫/胎元）", Det, "命宫=（月支−时支）mod12+五虎遁干；身宫=（月支+时支）mod12+五虎遁干；胎元=月柱干+1、支+3。校验 1987 → 癸丑/乙巳/庚子；🟡 命宫紫微版基于农历月与子平节气月支在节交日有差"),
            d("神煞（11 个常用）", Det, "日干锚 6 个（羊刃/禄/文昌/红艳/学堂/词馆，通行版固化） + 年支锚 4 个（桃花/驿马/华盖/将星，三合派生） + 日柱锚 1 个（魁罡严格 4 日）。多源校验（《三命通会》《渊海子平》+ 3 中文源）。校验 1987 → 年柱将星/月柱文昌+学堂/日柱驿马"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("late_lichun", "晚子·立春换年（主流）", true, "23-24 点归次日子时，立春为新年界。子平命理主流。"),
            s("late_sf", "晚子·春节换年", false, "夜子归次日，但年柱以农历正月初一为界。少数民间用法。"),
            s("early_lichun", "早子·立春换年", false, "夜子归当日（传统少数）；年柱仍立春。早期古法残留。"),
            s("early_sf", "早子·春节换年", false, "夜子归当日 + 春节换年。两少数派合用，极小众。"),
        ] }
    }
}

/// 紫微斗数叶。
#[derive(Debug, Default)]
pub struct ZiweiEngine;

impl CastingEngine for ZiweiEngine {
    fn id(&self) -> &'static str {
        "ziwei"
    }
    fn name(&self) -> &'static str {
        "紫微斗数"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        let school = mingli_ziwei::SihuaSchool::from_id(q.school_of(self.id(), "standard"))
            .unwrap_or_default();
        serde_json::to_value(mingli_ziwei::compute_at_with(m, ziwei_gender(q.gender), school))
            .unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("十二宫·五行局·主星", Det, "Z₁₂ 群作用+五行局，校验 命宫亥·土五局·紫微申"),
            d("4 辅星（昌曲辅弼）", Det, "古典通行口诀（《紫微斗数·安文昌文曲星诀》+ 维基/iztro 实现双证），1990 庚午校验"),
            d("四化（禄/权/科/忌）", Det, "通行版 5 源完全一致；全书本（王亭之）庚/壬科星分歧 — 王亭之亲文+《全书》古本双证"),
            d("四化派（戊/癸）", Und, "戊/癸的派别分歧本次研究未获多源证据，两派统一取通行表；待权威钦天派文献再补"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("standard", "通行版（中州/三合派）", true, "5 独立源完全一致(cnblogs/51xingli×2/vocus/wikipedia)；庚=太阴化科，壬=左辅化科"),
            s("quanshu", "全书本（王亭之版）", false, "庚=天府化科（王亭之亲文 haozh.com）；壬=天府化科（《全书》古本）；其余 8 干同通行版"),
        ] }
    }
}

/// 西洋占星本命盘叶（B 族）。仅 `astrology` feature 开启时编译（连带 VSOP87 星历）。
#[cfg(feature = "astrology")]
#[derive(Debug, Default)]
pub struct AstrologyEngine;

#[cfg(feature = "astrology")]
impl CastingEngine for AstrologyEngine {
    fn id(&self) -> &'static str {
        "astrology"
    }
    fn name(&self) -> &'static str {
        "西洋占星"
    }
    fn family(&self) -> Family {
        Family::Angular
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        let geo = match (q.latitude, q.longitude) {
            (Some(latitude), Some(longitude)) => Some(mingli_astrology::GeoLocation {
                latitude,
                longitude,
            }),
            _ => None,
        };
        let house_system = mingli_astrology::HouseSystem::from_id(q.school_of(self.id(), "placidus"));
        serde_json::to_value(mingli_astrology::compute_at(m, geo, house_system))
            .unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::Det;
        const { &[
            d("行星落座·相位", Det, "VSOP87 视黄经，太阳校验 Meeus 0.02°"),
            d("月亮落座", Det, "ELP-2000/82 (astro crate)，校验 Meeus 47.a < 5″ 与 Diana(AA) < 0.2°"),
            d("Asc/MC", Det, "平恒星时+平交角，校验 Diana(AA) < 0.5°"),
            d("分宫制(Placidus/Koch/WholeSign/Equal/Porphyry)", Det, "Placidus/Koch 移植 swehouse.c+Diana 12 cusp<0.05°(pyswisseph oracle)；极区回落 Porphyry"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("placidus", "Placidus 半弧三分", true, "占星圈业界默认；移植 Swiss swehouse.c；极区(|φ|≥66.5°)回落 Porphyry"),
            s("koch", "Koch 等赤经四分", false, "Walter Koch 1962；移植 Swiss swehouse.c 'K' case；极区(|φ|≥66.5°)回落 Porphyry"),
            s("whole_sign", "整宫制 Whole Sign", false, "古典占星与希腊占星派常用；一宫=一星座；极区可用"),
            s("equal", "Equal 等宫", false, "从上升起每 30° 一宫；MC 不作 10 宫尖；极区可用"),
            s("porphyry", "Porphyry 黄道三分", false, "1/10/4/7=Asc/MC/IC/DC；中间宫尖在黄道弧上三分；极区可用"),
        ] }
    }
}

/// 印度占星(Jyotish)叶（B 族）。仅 `jyotish` feature 开启时编译（依赖 astrology + ephemeris）。
/// 9 行星（含 Rahu/Ketu）+ 27 nakshatra + 12 rasi + Lagna，4 ayanamsa 流派。
#[cfg(feature = "jyotish")]
#[derive(Debug, Default)]
pub struct JyotishEngine;

#[cfg(feature = "jyotish")]
impl CastingEngine for JyotishEngine {
    fn id(&self) -> &'static str {
        "jyotish"
    }
    fn name(&self) -> &'static str {
        "印度占星"
    }
    fn family(&self) -> Family {
        Family::Angular
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        let geo = match (q.latitude, q.longitude) {
            (Some(latitude), Some(longitude)) => Some(mingli_astrology::GeoLocation { latitude, longitude }),
            _ => None,
        };
        let mode = mingli_jyotish::Ayanamsa::from_id(q.school_of(self.id(), "lahiri"))
            .unwrap_or_default();
        serde_json::to_value(mingli_jyotish::compute_at(m, geo, mode)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("9 行星 navagraha 恒星黄经", Det, "7 真行星走 VSOP87/ELP-2000/82（Lahiri J2000 容差<0.005°）；Rahu/Ketu 走 Meeus 22.4 月升交点公式"),
            d("27 nakshatra + Vimshottari 主星", Det, "Wikipedia/GrahaGuru/Vedicka 3 源完全一致"),
            d("12 rasi（白羊..双鱼）", Det, "与西洋 12 sign 一一对应，恒星黄道下"),
            d("Lagna（上升）", Det, "Asc(tropical) − ayanamsa；复用 astrology 平恒星时+平交角"),
            d("Ayanamsa（4 派）", Det, "Lahiri 用 SE 1956-01-01 anchor + 平岁差线性，容差 ±0.05° vs Swiss Ephemeris；KP/Raman/Fagan 用 J2000 静态偏移"),
            d("Vimshottari mahadasha timeline", Det, "9 主星固定年表（7/20/6/10/7/18/16/19/17 总 120），月亮 nakshatra 残余比例算 birth dasha 起止；9 段完整 timeline"),
            d("D-9 navamsa 分盘", Det, "公式 floor(lon×0.3)%12；校验三类(Movable/Fixed/Dual)起算 sign 与古典分类完全一致"),
            d("Antardasha/Pratyantar 子细分", Und, "本叶给完整 mahadasha 9 段；antardasha （mahadasha 内 9 步子细分） 留后续"),
            d("其它分盘(D-10/D-12/...)", Und, "本叶给 D-1(rasi) + D-9(navamsa) 两核心；其余 vargas 留后续"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("lahiri", "Lahiri （印度官方）", true, "Indian Astronomical Ephemeris 1955；SE_SIDM=1；1956-01-01 anchor 强权威"),
            s("krishnamurti", "Krishnamurti (KP)", false, "K. S. Krishnamurti；与 Lahiri 差 ~−6′"),
            s("raman", "Raman", false, "B. V. Raman；与 Lahiri 差 ~−1°26′46″"),
            s("fagan_bradley", "Fagan-Bradley", false, "西方 sidereal 学派；与 Lahiri 差 ~+0°53′01″"),
        ] }
    }
}

/// 七政四余（中国本土星占）叶（B 族）。仅 `qizhengsiyu` feature 开启时编译。
/// 10 体黄经（七政 + 罗㬋/计都/月孛三余；紫炁 🟡 不入） + 28 宿值日 + 12 sign 归宫。
#[cfg(feature = "qizhengsiyu")]
#[derive(Debug, Default)]
pub struct QizhengsiyuEngine;

#[cfg(feature = "qizhengsiyu")]
impl CastingEngine for QizhengsiyuEngine {
    fn id(&self) -> &'static str {
        "qizhengsiyu"
    }
    fn name(&self) -> &'static str {
        "七政四余"
    }
    fn family(&self) -> Family {
        Family::Angular
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(mingli_qizhengsiyu::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("七政地心黄经（日月水金火木土）", Det, "VSOP87 行星 + ELP-2000/82 月亮(apparent)，太阳校验 Meeus 0.02°"),
            d("罗㬋（月平升交点）", Det, "Meeus AA 第 47 章 eq 47.7 精确五项 + SOFA J2000 校验 0.14″；通行近代取升交点"),
            d("计都 = 罗㬋 + 180°", Det, "汤若望《时宪历》后通行近代/印度对位"),
            d("月孛（月平远地点）", Det, "Meeus AA p.343 月平近地点 + 180°，PyMeeus/NASA GSFC/soniakeys 三源系数字符级一致"),
            d("沈括古版四余流派", Und, "🟡 沈括《梦溪笔谈》古法定义在二手源中相互矛盾：某源称罗=升交点/计=月远地点；另源称罗=降交点。原典考证未稳，本叶不提供沈括版流派，只走通行近代版"),
            d("28 宿值日", Det, "(JDN+11) mod 28，跨 5 锚点 341 年交叉验证（沿 zeri::mansion）"),
            d("12 sign 归宫（30° 等分）", Det, "回归黄道整宫归宫，天文公认无歧义"),
            d("紫炁", Und, "🟡 无天文实体；中文维基明文「找不著对应的天文现象」；五种互不兼容定义（虚星/月近地点/月轨中点/木余气/天狼星）无可代入时间公式，swisseph 等主流库均不提供"),
            d("12 次落宫（星纪/玄枵/...）", Und, "🟡 古籍三源分歧实质：《尔雅》给标志宿、《汉书·律历志》按度数（多宿跨次）、通行表整宿归一次；不强编"),
            d("28 宿分黄道（古制不等长）", Und, "🟡 每宿距度由观测得 + 岁差校正涉大查表；本叶只做值日 （JDN 周期）"),
        ] }
    }
}

/// 易经起卦叶（C 族）。三钱法、种子可复现。
#[derive(Debug, Default)]
pub struct YijingEngine;

impl CastingEngine for YijingEngine {
    fn id(&self) -> &'static str {
        "yijing"
    }
    fn name(&self) -> &'static str {
        "易经起卦"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        let method = match q.school_of(self.id(), "three_coins") {
            "yarrow_stalks" => mingli_yijing::Method::YarrowStalks,
            _ => mingli_yijing::Method::ThreeCoins,
        };
        let cst = mingli_yijing::cast(method, effective_seed(m, q));
        serde_json::to_value(cst).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Sto};
        const { &[
            d("六爻·本卦/之卦", Sto, "种子可复现；两流派概率分布 1/8·3/8·3/8·1/8（三钱）、1/16·5/16·7/16·3/16（蓍草） 均校验"),
            d("六十四卦名 + 文王序", Det, "三源校验（ctext《序卦传》/zh.wiki/en.wiki），「二二相耦」定理（4 纯错对 + 28 综对）穷举证明"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("three_coins", "三钱法", true, "三枚铜钱六掷；概率 6：7：8：9 = 1：3：3：1（老阴/少阳/少阴/老阳）"),
            s("yarrow_stalks", "蓍草法", false, "五十蓍策十八变（模拟分布）；概率 6：7：8：9 = 1：5：7：3"),
        ] }
    }
}

/// 地占叶（C 族）。4 母图→盾牌图，种子可复现，法官恒为偶。
#[derive(Debug, Default)]
pub struct GeomancyEngine;

impl CastingEngine for GeomancyEngine {
    fn id(&self) -> &'static str {
        "geomancy"
    }
    fn name(&self) -> &'static str {
        "地占"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(mingli_geomancy::cast(effective_seed(m, q))).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Sto, Und};
        const { &[
            d("四母→盾牌图", Sto, "种子可复现"),
            d("法官恒为偶", Det, "GF(2) 线性，穷举证于 core::gf2"),
            d("16 figure 名", Und, "查表待补，只显 GF(2) 四点结构"),
        ] }
    }
}

/// Sikidy 叶（C 族）。4 母列→16 列，种子可复现，C15 恒为偶。
#[derive(Debug, Default)]
pub struct SikidyEngine;

impl CastingEngine for SikidyEngine {
    fn id(&self) -> &'static str {
        "sikidy"
    }
    fn name(&self) -> &'static str {
        "Sikidy"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(mingli_sikidy::cast(effective_seed(m, q))).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Sto, Und};
        const { &[
            d("四母→16 列", Sto, "种子可复现，同地占 GF(2) 代数"),
            d("创世者 C15 恒偶", Det, "与地占法官同一定理"),
            d("16 列名表", Und, "查表待补"),
        ] }
    }
}

/// Ifá 叶（C 族）。双 figure→256 odu，种子可复现。
#[derive(Debug, Default)]
pub struct IfaEngine;

impl CastingEngine for IfaEngine {
    fn id(&self) -> &'static str {
        "ifa"
    }
    fn name(&self) -> &'static str {
        "Ifá"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(mingli_ifa::cast(effective_seed(m, q))).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Sto, Und};
        const { &[
            d("双 figure→256 odu", Sto, "种子可复现；16×16 = (Z₂)⁴ 组合"),
            d("256 odu 名", Und, "查表（错一个毒整枝）待补，结构已建"),
        ] }
    }
}

/// 抽牌叶（C 族）。schools 暴露五种 deck：塔罗 78 / 大阿卡纳 22 / Lenormand 36 /
/// Elder Futhark 24 / Younger Futhark 16。统一三张牌阵，种子可复现。
#[derive(Debug, Default)]
pub struct TarotEngine;

impl CastingEngine for TarotEngine {
    fn id(&self) -> &'static str {
        "tarot"
    }
    fn name(&self) -> &'static str {
        "塔罗"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        // `tarot_full_marseilles` / `tarot_major_marseilles` 复合 id 同时编码 deck + Tarot 流派。
        let school = q.school_of(self.id(), "tarot_full");
        let (deck_id, order) = match school {
            "tarot_full_marseilles" => ("tarot_full", mingli_cartomancy::TarotOrder::Marseilles),
            "tarot_major_marseilles" => ("tarot_major", mingli_cartomancy::TarotOrder::Marseilles),
            id => (id, mingli_cartomancy::TarotOrder::RiderWaite),
        };
        let deck = mingli_cartomancy::Deck::from_id(deck_id)
            .unwrap_or(mingli_cartomancy::Deck::TarotFull);
        serde_json::to_value(mingli_cartomancy::draw_deck_with_order(deck, order, 3, effective_seed(m, q)))
            .unwrap_or(Value::Null)
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const {
            &[
                s("tarot_full", "塔罗 78 (RWS)", true, "Rider-Waite-Smith 1909：8=Strength/11=Justice；Major 22 + Minor 56，允许逆位"),
                s("tarot_full_marseilles", "塔罗 78 (Marseilles)", false, "Tarot de Marseille 传统：8=Justice/11=Strength；牌副同 RWS"),
                s("tarot_major", "塔罗大阿卡纳 22 (RWS)", false, "仅 Major Arcana，RWS 顺序"),
                s("tarot_major_marseilles", "塔罗大阿卡纳 22 (Marseilles)", false, "仅 Major，Marseilles 顺序（8/11 互换）"),
                s("lenormand", "Petit Lenormand 36", false, "传统不用逆位；Hechtel 1799 The Game of Hope 标准"),
                s("elder_futhark", "Elder Futhark 卢恩 24", false, "古日耳曼/古英，允许逆位；BabelStone Runic block U+16A0-U+16FF"),
                s("younger_futhark", "Younger Futhark 卢恩 16", false, "维京时期 Long-branch 简化卢恩，允许逆位"),
            ]
        }
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::Sto;
        const { &[
            d("抽牌·正逆位", Sto, "无放回 Fisher–Yates 置换 + 逆位 bit（Lenormand 除外），种子可复现"),
            d("牌副大小", Sto, "由 schools 选择：78/22/36/24/16 五种 deck × Tarot RWS/Marseilles 两派 = 7 组合"),
            d("牌名·中文译名·Unicode 字符", Sto, "多源校验入码：Tarot Major（en.wiki+zh.wiki+Biddy 3源）/Minor（花色×等级生成）/Lenormand（4源）/Futhark（BabelStone+Runic block 2源）"),
        ] }
    }
}

/// 梅花易数叶（时间起卦·确定性）。年支/月/日/时辰 mod8/mod6 → 卦，不用种子。
#[derive(Debug, Default)]
pub struct MeihuaEngine;

impl CastingEngine for MeihuaEngine {
    fn id(&self) -> &'static str {
        "meihua"
    }
    fn name(&self) -> &'static str {
        "梅花易数"
    }
    fn family(&self) -> Family {
        // 时间→模运算→卦，属 A 族（确定性循环），非随机抽样。
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        let method = mingli_meihua::Method::from_id(q.school_of(self.id(), "time"))
            .unwrap_or_default();
        let cst = mingli_meihua::compute_at_with(m, method, effective_seed(m, q));
        serde_json::to_value(cst).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Sto};
        const { &[
            d("本/互/之卦·体用五行（时间法）", Det, "农历量 mod8/mod6，确定；同时刻同卦"),
            d("本/互/之卦（数字法）", Sto, "两数由种子高低 32 位派生，同种子可复现"),
            d("六十四卦名 + 文王序", Det, "三源校验，「二二相耦」定理穷举证明"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("time", "时间起卦法", true, "邵雍古法：年支/月/日/时辰 mod8/mod6；确定性，同时刻同卦"),
            s("numbers", "数字（报数）法", false, "首数为上卦，次数为下卦，（首+次+时辰） mod6 为动爻；两数由种子拆解派生（C 族风格）"),
        ] }
    }
}

/// 小六壬叶（A 族·时间起课，确定性）。月→日→时辰在 Z₆ 上掐指。
#[derive(Debug, Default)]
pub struct XiaoliurenEngine;

impl CastingEngine for XiaoliurenEngine {
    fn id(&self) -> &'static str {
        "xiaoliuren"
    }
    fn name(&self) -> &'static str {
        "小六壬"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(mingli_xiaoliuren::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::Det;
        const { &[d("六神掐指（月→日→时）", Det, "Z₆ 连续位移，六神为定义性有序环")] }
    }
}

/// 择日叶（A 族）。建除十二神 + 二十八宿值日 + 彭祖百忌 + 天乙贵人。
#[derive(Debug, Default)]
pub struct ZeriEngine;

impl CastingEngine for ZeriEngine {
    fn id(&self) -> &'static str {
        "zeri"
    }
    fn name(&self) -> &'static str {
        "择日"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(mingli_zeri::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("建除十二神", Det, "日支−月建支 on Z₁₂"),
            d("二十八宿值日", Det, "连续 Z₂₈，偏移 11 跨 341 年 5 锚校验"),
            d("彭祖百忌（干句+支句）", Det, "《钦定协纪辨方书》/通胜多源口诀，22 句固定查表"),
            d("天乙贵人（双地支）", Det, "《三命通会》通行版『甲戊庚牛羊』口诀"),
            d("天乙贵人（《珞琭子赋》变体）", Und, "庚归虎马，无多源校验源，不入码"),
            d("其余神煞宜忌", Und, "随流派分歧，不下断言"),
        ] }
    }
}

/// 玛雅历叶（A 族·CRT）。Tzolkʼin 260 + Haab 365 + Long Count。
#[derive(Debug, Default)]
pub struct MayaEngine;

impl CastingEngine for MayaEngine {
    fn id(&self) -> &'static str {
        "maya"
    }
    fn name(&self) -> &'static str {
        "玛雅历"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(mingli_maya::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::Det;
        const { &[d("Tzolkʼin·Haab·Long Count", Det, "GMT 历元 584283，校验 0.0.0.0.0 与 2012-12-21 双锚")] }
    }
}

/// 巴厘 Pawukon 叶（A 族·多并行週）。210 上的十个 wewaran。
#[derive(Debug, Default)]
pub struct PawukonEngine;

impl CastingEngine for PawukonEngine {
    fn id(&self) -> &'static str {
        "pawukon"
    }
    fn name(&self) -> &'static str {
        "巴厘Pawukon"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(mingli_pawukon::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("十週（简单/派生/卡日）", Det, "210=2·3·5·7，锚 day0=2020-07-05 校验 Galungan"),
            d("Ekawara/Dwiwara 奇偶向", Und, "源间一处冲突，采信两个独立实现"),
        ] }
    }
}

/// 缅甸 Mahabote 叶（A 族）。本命核心数 = （缅历年 − 星期） mod 7。
#[derive(Debug, Default)]
pub struct MahaboteEngine;

impl CastingEngine for MahaboteEngine {
    fn id(&self) -> &'static str {
        "mahabote"
    }
    fn name(&self) -> &'static str {
        "缅甸Mahabote"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(mingli_mahabote::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("核心数·七宫·八天週行星", Det, "（缅历年−星期） mod 7，校验 2000-01-01=Adipati"),
            d("宫义·宫间关系", Und, "无自洽单源，不下断言"),
        ] }
    }
}

/// 大六壬叶（⟂ 横切）。天地盘 + 四课 + 三传课式。
#[derive(Debug, Default)]
pub struct LiurenEngine;

impl CastingEngine for LiurenEngine {
    fn id(&self) -> &'static str {
        "liuren"
    }
    fn name(&self) -> &'static str {
        "大六壬"
    }
    fn family(&self) -> Family {
        Family::CrossCutting
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(mingli_liuren::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("天地盘·寄宫四课", Det, "月将加时 Z₁₂ 旋转，校验 亥将子时甲子日"),
            d("三传·贼克/比用/遥克/伏返", Det, "取传规则明确"),
            d("三传·涉害/昴星/别责/八专", Und, "取传流派分歧，诚实返 None 不强编"),
        ] }
    }
}

/// 奇门遁甲叶（⟂ 横切）。定局（阴阳遁+三元）+ 地盘三奇六仪。
#[derive(Debug, Default)]
pub struct QimenEngine;

impl CastingEngine for QimenEngine {
    fn id(&self) -> &'static str {
        "qimen"
    }
    fn name(&self) -> &'static str {
        "奇门遁甲"
    }
    fn family(&self) -> Family {
        Family::CrossCutting
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(mingli_qimen::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("阴阳遁·三元局·地盘三奇六仪", Det, "72 局表（±3 不变量自检），校验阳遁一局"),
            d("时柱·旬首六仪·旬空·值符宫之根", Det, "60 甲子 6 旬穷举校验，1987-09-17 15：00 时柱壬申/甲子旬遁戊/旬空戌亥 oracle"),
            d("值符干·值符宫·值符星·九星原配", Det, "时干甲遁旬首六仪；值符宫 = 实际值符干在地盘的位置；值符星 = 旬首所在宫原配九星（蓬芮冲辅禽心柱任英）。1987 oracle 时干壬→艮8宫，值符星天冲"),
            d("天盘九星旋转·八门·八神", Und, "中宫寄宫法/八门数法/八神序 3 处流派开关，无权威排盘软件 oracle，暂缺"),
        ] }
    }
}

/// 藏历循环叶（A 族）。60 周期（元素×生肖）+ 年 mewa。
#[derive(Debug, Default)]
pub struct TibetanEngine;

impl CastingEngine for TibetanEngine {
    fn id(&self) -> &'static str {
        "tibetan"
    }
    fn name(&self) -> &'static str {
        "藏历循环"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(mingli_tibetan::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("60 周期·年 mewa", Det, "5 元素×12 生肖；mewa 逆行，校验 2024=木阳龙·mewa3"),
            d("年 parkha", Und, "主流藏历无年卦（仅个人盘），故不输出"),
        ] }
    }
}

/// 太乙神数叶（⟂ 横切）。太乙积年 → 太乙行八宫（三年一宫·阳顺阴逆）+ 三才。
#[derive(Debug, Default)]
pub struct TaiyiEngine;

impl CastingEngine for TaiyiEngine {
    fn id(&self) -> &'static str {
        "taiyi"
    }
    fn name(&self) -> &'static str {
        "太乙神数"
    }
    fn family(&self) -> Family {
        Family::CrossCutting
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(mingli_taiyi::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("太乙行八宫·三才·积年", Det, "三年一宫·廿四年一周，积年锚《金镜式经》724=1937281"),
            d("文昌·始击·主客算·诸将", Und, "源间分歧，暂缺"),
            d("落宫绝对相位", Und, "遵引文规则，精校待权威排盘软件"),
        ] }
    }
}

/// 数字学叶（D 族·哈希环）。日期生命灵数 + 生日数；给出姓名时附表达/灵魂/人格数（两套字母表）。
#[derive(Debug, Default)]
pub struct NumerologyEngine;

impl CastingEngine for NumerologyEngine {
    fn id(&self) -> &'static str {
        "numerology"
    }
    fn name(&self) -> &'static str {
        "数字学"
    }
    fn family(&self) -> Family {
        Family::Hashing
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        let method = match q.school_of(self.id(), "component") {
            "whole_sum" => mingli_numerology::LifePathMethod::WholeSum,
            _ => mingli_numerology::LifePathMethod::Component,
        };
        let cst = match &q.name {
            Some(name) => mingli_numerology::compute_named_with(m, name, method),
            None => mingli_numerology::compute_at_with(m, method),
        };
        serde_json::to_value(cst).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("姓名数（双字母表并出）", Det, "Pythagorean/Chaldean 同时输出，无需选择"),
            d("生命灵数（可选 Component/WholeSum）", Det, "两派算法已实现并交叉校验；每次同时给出主+alt"),
            d("Y 元音归属", Und, "Y 是否计入元音/辅音随细分流派，本叶按辅音处理"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("component", "分量约化（Pythagorean 学派）", true, "y/m/d 各约化后求和再约化；现代数字学常用"),
            s("whole_sum", "全数字直加（Chaldean 学派）", false, "ymd 全数字平铺相加再约化；古典 Chaldean/Kabbalistic 派常用"),
        ] }
    }
}

/// 已注册的全部叶。加新叶在此登记即并入并行 fan-out。
#[must_use]
pub fn registry() -> Vec<Box<dyn CastingEngine>> {
    vec![
        Box::new(BaziEngine),
        Box::new(ZiweiEngine),
        #[cfg(feature = "astrology")]
        Box::new(AstrologyEngine),
        #[cfg(feature = "jyotish")]
        Box::new(JyotishEngine),
        #[cfg(feature = "qizhengsiyu")]
        Box::new(QizhengsiyuEngine),
        Box::new(YijingEngine),
        Box::new(GeomancyEngine),
        Box::new(SikidyEngine),
        Box::new(IfaEngine),
        Box::new(TarotEngine),
        Box::new(MeihuaEngine),
        Box::new(XiaoliurenEngine),
        Box::new(ZeriEngine),
        Box::new(MayaEngine),
        Box::new(PawukonEngine),
        Box::new(MahaboteEngine),
        Box::new(LiurenEngine),
        Box::new(QimenEngine),
        Box::new(TaiyiEngine),
        Box::new(TibetanEngine),
        Box::new(NumerologyEngine),
    ]
}

/// 一个输入 → 共享层算一次 → **并行**排所有叶 → `id → 盘(JSON)`。
#[must_use]
pub fn cast_all(q: &Query) -> BTreeMap<String, Value> {
    let m = Moment::new(q.year, q.month, q.day, q.hour, q.minute, q.tz);
    registry()
        .into_par_iter()
        .map(|e| (e.id().to_string(), e.cast(&m, q)))
        .collect()
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

fn effective_school_id(e: &dyn CastingEngine, q: &Query) -> String {
    if let Some(sel) = q.schools.get(e.id()) {
        return sel.clone();
    }
    e.schools()
        .iter()
        .find(|s| s.default)
        .map_or_else(String::new, |s| s.id.to_string())
}

/// 只算**单片**叶（按 id）——共享层仍只算一次，但仅排该叶（释义/单叶请求用，省去其余 18 叶）。
/// 未知 id 返回 `None`。
#[must_use]
pub fn cast_one(id: &str, q: &Query) -> Option<LeafOutput> {
    let e = registry().into_iter().find(|e| e.id() == id)?;
    let m = Moment::new(q.year, q.month, q.day, q.hour, q.minute, q.tz);
    let effective_school = effective_school_id(e.as_ref(), q);
    Some(LeafOutput {
        id: e.id(),
        name: e.name(),
        family: e.family(),
        family_label: e.family().label(),
        profile: e.profile(),
        schools: e.schools(),
        effective_school,
        chart: e.cast(&m, q),
    })
}

/// 同 [`cast_all`]，但保留注册表**顺序**并附带每叶元数据（id/name/family）。
#[must_use]
pub fn cast_all_detailed(q: &Query) -> Vec<LeafOutput> {
    let m = Moment::new(q.year, q.month, q.day, q.hour, q.minute, q.tz);
    registry()
        .into_par_iter()
        .map(|e| {
            let effective_school = effective_school_id(e.as_ref(), q);
            LeafOutput {
                id: e.id(),
                name: e.name(),
                family: e.family(),
                family_label: e.family().label(),
                profile: e.profile(),
                schools: e.schools(),
                effective_school,
                chart: e.cast(&m, q),
            }
        })
        .collect()
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
/// 每变体携带其所需的**输入原子**；一切意图最终都映射到一组叶（见 [`route`]）。
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
    /// 默认路由叶（声明式；运行时实际可用以 [`route`] 输出 ∩ 当前 registry 为准）。
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

/// 8 类问事意图的清单（声明式，与 [`route`] 同构对偶）。
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

/// 把一个问局意图路由到具体的叶 id 列表（运行时，过滤当前 registry 实际启用）。
///
/// `Natal` 走全 registry（顺序与 [`registry`] 一致）；其余意图按 [`intents`] 的
/// `default_leaves` 并与当前 registry 取交集（feature flag 关掉的叶自动剔除）。
///
/// # Panics
///
/// 不会发生：[`QueryKind::id`] 的 8 个返回值与 [`intents`] 清单 8 项 id 一一对应，
/// 测试 `intents_well_formed_and_aligned_with_querykind` 守卫此不变量。
#[must_use]
pub fn route(kind: &QueryKind) -> Vec<&'static str> {
    let reg = registry();
    let available: std::collections::HashSet<&'static str> =
        reg.iter().map(|e| e.id()).collect();
    if matches!(kind, QueryKind::Natal(_)) {
        return reg.iter().map(|e| e.id()).collect();
    }
    let spec = intents()
        .iter()
        .find(|s| s.id == kind.id())
        .expect("QueryKind::id 必须在 intents() 清单内");
    spec.default_leaves
        .iter()
        .copied()
        .filter(|id| available.contains(id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let out = cast_all(&sample());
        assert_eq!(out.len(), registry().len());
        assert!(out.contains_key("bazi"));
        assert!(out.contains_key("ziwei"));
        assert!(out.contains_key("astrology"));
        // 跨叶一瞥：同一输入下，八字双子月与西洋太阳双子座并存可对齐比较
        assert_eq!(out["astrology"]["planets"][0]["sign"], "双子"); // 1990-06-15 太阳
    }

    #[test]
    fn all_registered_leaves_well_formed() {
        // 遍历注册表，逐叶检查 id/name/family 元数据齐备、cast 产出非空。
        let expected = [
            ("bazi", "四柱八字", Family::Cyclic),
            ("ziwei", "紫微斗数", Family::Cyclic),
            ("astrology", "西洋占星", Family::Angular),
            ("jyotish", "印度占星", Family::Angular),
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
        let out = cast_all(&sample());
        for id in ["yijing", "geomancy", "sikidy", "ifa", "tarot", "meihua"] {
            assert!(out.contains_key(id), "缺少 C 族叶 {id}");
        }
    }

    #[test]
    fn shared_layer_matches_standalone() {
        // 共享上下文复用结果 ≡ 各叶独立排盘（记忆化不改变结果）。
        let q = sample();
        let m = Moment::new(q.year, q.month, q.day, q.hour, q.minute, q.tz);
        let bazi_shared = mingli_bazi::compute_at(&m, bazi_gender(q.gender));
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
        let out = cast_all(&sample());
        assert!(out["yijing"]["primary_upper"].is_string());
        let again = cast_all(&sample());
        assert_eq!(out["yijing"]["primary"], again["yijing"]["primary"]);
        let mut q = sample();
        q.seed = Some(123);
        let a = cast_all(&q);
        let b = cast_all(&q);
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
        assert_eq!(r[2].id(), "astrology");
        assert_eq!(r[2].name(), "西洋占星");
        assert_eq!(r[2].family(), Family::Angular);
    }

    #[test]
    fn female_and_no_gender() {
        let mut q = sample();
        q.gender = Some(Gender::Female); // 庚午阳年女 → 大运逆行
        let out = cast_all(&q);
        assert_eq!(out["bazi"]["dayun"]["forward"], false);
        assert_eq!(out["ziwei"]["ming_branch"], "亥"); // 紫微不依赖性别
        q.gender = None;
        let out2 = cast_all(&q);
        assert!(out2["bazi"]["dayun"].is_null()); // 无性别不排大运
    }

    #[test]
    fn astrology_angles_when_geo_given() {
        // 无坐标 → 占星只出落座，无 Asc/MC。
        let out = cast_all(&sample());
        assert!(out["astrology"]["angles"].is_null());
        assert!(out["astrology"]["houses"].is_null());
        // 给出坐标（上海）→ 占星出 Asc/MC + 整宫制十二宫。
        let mut q = sample();
        q.latitude = Some(31.23);
        q.longitude = Some(121.47);
        let out2 = cast_all(&q);
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
        let out = cast_all_detailed(&sample());
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
        let full = cast_all_detailed(&q);
        // cast_one 与 cast_all_detailed 的对应叶逐项一致（只是少算其余叶）。
        for id in ["bazi", "astrology", "liuren", "numerology"] {
            let one = cast_one(id, &q).unwrap();
            let from_full = full.iter().find(|l| l.id == id).unwrap();
            assert_eq!(one.id, from_full.id);
            assert_eq!(one.chart, from_full.chart);
            assert_eq!(one.profile.len(), from_full.profile.len());
        }
        assert!(cast_one("nope", &q).is_none());
    }

    #[test]
    fn cast_all_detailed_preserves_order_and_meta() {
        let out = cast_all_detailed(&sample());
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
        // 五家族都出现。
        let fams: std::collections::HashSet<Family> = out.iter().map(|l| l.family).collect();
        for f in [Family::Cyclic, Family::Angular, Family::Sampling, Family::Hashing, Family::CrossCutting] {
            assert!(fams.contains(&f), "缺家族 {f:?}");
        }
    }

    #[test]
    fn numerology_leaf_date_and_name() {
        // 无姓名：数字学只出日期数（生命灵数/生日数）。
        let out = cast_all(&sample());
        assert!(out["numerology"]["life_path"].is_number());
        assert!(out["numerology"]["pythagorean"].is_null()); // 无姓名
        // 1990-06-15 生命灵数 = 4。
        assert_eq!(out["numerology"]["life_path"], 4);
        // 给姓名：附表达/灵魂/人格数（两套字母表）。
        let mut q = sample();
        q.name = Some("Ada Lovelace".to_string());
        let out2 = cast_all(&q);
        assert!(out2["numerology"]["pythagorean"]["expression"].is_number());
        assert!(out2["numerology"]["chaldean"]["expression"].is_number());
        assert_eq!(r_family("numerology"), Family::Hashing);
    }

    #[test]
    fn outputs_are_correct() {
        let out = cast_all(&sample());
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
    fn querykind_id_covers_all_variants() {
        // 8 个变体 id 全唯一，与 intents() 顺序对应。
        let kinds = [
            QueryKind::Natal(sample()),
            QueryKind::Fortune { natal: sample(), t_target: ask_2026() },
            QueryKind::Event { t_ask: ask_2026(), seed: 42, q_text: None },
            QueryKind::Election { window_start: ask_2026(), window_end: ask_2026(), category: "婚".into() },
            QueryKind::Synastry { a: sample(), b: sample() },
            QueryKind::Mundane { p_polity: sample() },
            QueryKind::Locative { t_ask: ask_2026(), seed: 7, category: "寻物".into() },
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
        // id 唯一 + 非空字段 + atoms/leaves 非空。
        let mut seen = std::collections::HashSet::new();
        for s in specs {
            assert!(seen.insert(s.id), "意图 id 应唯一： {}", s.id);
            assert!(!s.name_zh.is_empty());
            assert!(!s.atoms.is_empty(), "{} atoms 应非空", s.id);
            assert!(!s.default_leaves.is_empty(), "{} default_leaves 应非空", s.id);
            assert!(!s.output_shape.is_empty());
            assert!(!s.note.is_empty());
        }
        // QueryKind 8 变体 id 与 intents 清单 id 一一对应。
        let kind_ids = [
            QueryKind::Natal(sample()).id(),
            QueryKind::Fortune { natal: sample(), t_target: ask_2026() }.id(),
            QueryKind::Event { t_ask: ask_2026(), seed: 0, q_text: None }.id(),
            QueryKind::Election { window_start: ask_2026(), window_end: ask_2026(), category: String::new() }.id(),
            QueryKind::Synastry { a: sample(), b: sample() }.id(),
            QueryKind::Mundane { p_polity: sample() }.id(),
            QueryKind::Locative { t_ask: ask_2026(), seed: 0, category: String::new() }.id(),
            QueryKind::Onomancy { name: String::new(), surname_strokes: None, given_strokes: None }.id(),
        ];
        let spec_ids: Vec<&'static str> = specs.iter().map(|s| s.id).collect();
        assert_eq!(kind_ids.to_vec(), spec_ids);
        // natal + onomancy + fortune 为 Live，其余 5 个 Pending。
        let live_count = specs.iter().filter(|s| s.status == IntentStatus::Live).count();
        assert_eq!(live_count, 3, "natal + onomancy + fortune 为 Live");
    }

    #[test]
    fn intents_natal_covers_registry() {
        // natal 意图的 default_leaves 应恰为 registry 全集（声明式守卫：加新叶必须同步）。
        let natal = intents().iter().find(|s| s.id == "natal").unwrap();
        let reg_ids: std::collections::HashSet<&'static str> =
            registry().iter().map(|e| e.id()).collect();
        let intent_ids: std::collections::HashSet<&'static str> =
            natal.default_leaves.iter().copied().collect();
        assert_eq!(intent_ids, reg_ids, "natal.default_leaves 应与 registry 一致");
    }

    #[test]
    fn intents_non_natal_leaves_subset_of_registry() {
        // 非 Natal 意图的 default_leaves 全部在 registry 内（否则 route 会过滤掉）。
        let reg_ids: std::collections::HashSet<&'static str> =
            registry().iter().map(|e| e.id()).collect();
        for s in intents().iter().filter(|s| s.id != "natal") {
            for leaf in s.default_leaves {
                assert!(reg_ids.contains(leaf), "{} 意图引用未注册叶 {}", s.id, leaf);
            }
        }
    }

    #[test]
    fn route_natal_returns_full_registry_in_order() {
        let r = route(&QueryKind::Natal(sample()));
        let reg_order: Vec<&'static str> = registry().iter().map(|e| e.id()).collect();
        assert_eq!(r, reg_order, "Natal 路由应等于 registry 顺序");
    }

    #[test]
    fn route_non_natal_dispatches_to_declared_leaves() {
        // Fortune → 时间序列叶 （bazi/ziwei/jyotish/astrology 等）。
        let r = route(&QueryKind::Fortune { natal: sample(), t_target: ask_2026() });
        assert!(r.contains(&"bazi"));
        assert!(r.contains(&"ziwei"));
        // Event → 卜筮叶。
        let r = route(&QueryKind::Event { t_ask: ask_2026(), seed: 42, q_text: None });
        assert!(r.contains(&"yijing"));
        assert!(r.contains(&"tarot"));
        assert!(!r.contains(&"bazi"), "Event 不路由本命型叶");
        // Election → zeri 等。
        let r = route(&QueryKind::Election { window_start: ask_2026(), window_end: ask_2026(), category: "婚".into() });
        assert!(r.contains(&"zeri"));
        // Mundane → 太乙等。
        let r = route(&QueryKind::Mundane { p_polity: sample() });
        assert!(r.contains(&"taiyi"));
        // Locative → 六壬等。
        let r = route(&QueryKind::Locative { t_ask: ask_2026(), seed: 7, category: "寻物".into() });
        assert!(r.contains(&"liuren"));
        // Onomancy → numerology（在 registry）；gematria/abjad/wuge 是 /api/word 字词库不在 cast registry。
        let r = route(&QueryKind::Onomancy { name: "Ada".into(), surname_strokes: None, given_strokes: None });
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
        let out = cast_all(&q);
        assert_eq!(out["bazi"]["year"]["ganzhi"], "庚午");
        assert_eq!(out["bazi"]["day"]["ganzhi"], "辛亥");
        assert_eq!(out["ziwei"]["ming_branch"], "亥");
        assert_eq!(route(&QueryKind::Natal(q)).len(), registry().len());
    }
}
