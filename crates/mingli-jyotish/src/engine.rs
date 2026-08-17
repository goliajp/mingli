//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, s, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query, SchoolItem};
use serde_json::Value;

/// 印度占星(Jyotish)叶（B 族）。仅 `jyotish` feature 开启时编译（依赖 astrology + ephemeris）。
/// 9 行星（含 Rahu/Ketu）+ 27 nakshatra + 12 rasi + Lagna，4 ayanamsa 流派。
#[derive(Debug, Default)]
pub struct JyotishEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(e: &JyotishEngine, m: &Moment, q: &Query) -> crate::JyotishChart {
    let geo = match (q.latitude, q.longitude) {
        (Some(latitude), Some(longitude)) => Some(mingli_ephemeris::GeoLocation { latitude, longitude }),
        _ => None,
    };
    let mode = crate::Ayanamsa::from_id(q.school_of(e.id(), "lahiri"))
        .unwrap_or_default();crate::compute_at(m, geo, mode)
}

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
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn answers(&self) -> &'static [Intent] {
        // 「运」答得起：本叶算 Vimshottari 大运时间线（dasha.rs 的 `vimshottari_timeline`），
        // 那正是「势」要的时间序列。「合」不答——kuta / porutham 一类的合婚计算本叶没有。
        &[Intent::Natal, Intent::Fortune]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 月亮所在宿——印度占星以月宿定本命。
        let c = chart(self, m, q);
        Some(Principal { label: "月宿(nakshatra)", value: c.grahas.get(1).map_or_else(String::new, |g| g.nakshatra_name.to_string()) })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d(
                "合婚（kuta / porutham，未实现）",
                Und,
                "🟡 这不是定不下，是还没做——南北两系判据不同（北印 Ashtakuta 八项、南印 Porutham 十项），\
                 各项取值表须逐项找 ≥2 源。本叶目前不出任何两盘比对，故不认领「合」这一类问局",
            ),
            d("9 行星 navagraha 恒星黄经", Det, "7 真行星走 VSOP87/ELP-2000/82（Lahiri J2000 容差<0.005°）；Rahu/Ketu 走 Meeus 22.4 月升交点公式"),
            d("27 nakshatra + Vimshottari 主星", Det, "Wikipedia/GrahaGuru/Vedicka 3 源完全一致"),
            d("12 rasi（白羊..双鱼）", Det, "与西洋 12 sign 一一对应，恒星黄道下"),
            d("Lagna（上升）", Det, "Asc(tropical) − ayanamsa；复用 astrology 平恒星时+平交角"),
            d("Ayanamsa（4 派）", Det, "Lahiri 用 SE 1956-01-01 anchor + 平岁差线性，容差 ±0.05° vs Swiss Ephemeris；KP/Raman/Fagan 用 J2000 静态偏移"),
            d("Vimshottari mahadasha timeline", Det, "9 主星固定年表（7/20/6/10/7/18/16/19/17 总 120），月亮 nakshatra 残余比例算 birth dasha 起止；9 段完整 timeline"),
            d("D-9 navamsa 分盘", Det, "公式 floor(lon×0.3)%12；校验三类(Movable/Fixed/Dual)起算 sign 与古典分类完全一致"),
            d("Antardasha（bhukti）子细分", Det, "时长 = 主星年数 × 子星年数 ÷ 120，首个子运即主星自己、其后依同一固定顺序循环（BPHS 51.1 与 51.2）；drik-panchanga、PyJHora、VedAstro 三个开源实现的源码常量逐条一致。九步之和恰铺满主运跨度"),
            d("Vimśottarī 一年折合多少天", Und, "🟡 原典只给年数比例、**不规定年长**；实查六个不同取值：儒略年 365.25(Wikipedia/Maitreya 默认)、savana 360(VedAstro 于 Raman ayanāṃśa)、回归年 365.24219、格里年 365.2425、365.2564(VedAstro 于 KP)、真恒星年 365.256364(PyJHora 默认)。drik-panchanga 源码自注「some say 360 days, others 365.25 or 365.2563 etc」，VedAstro 自注「vary as per the astrologer's preference」。本叶做成参数，默认儒略年，不写死"),
            d("Pratyantardaśā 及更深层", Und, "🟡 同一条比例规则再嵌一层（三实现皆如此），但层数与命名南北传统不一（北传 dasa-antardasa-pratyantardasa，南传 dasa-bhukti-antara-sukshma，各家给到 5/6/8 层不等）；本叶只出两层"),
            d("antardaśā 起始子星的变体", Und, "🟡 仅见 PyJHora 提供六档选项（主星/下一星/上一星 × 顺/逆），其自身只含混标注为「as calculated by various astrologers」，未取得任何文献出处；本叶只实现 BPHS 的「首个子运即主星」"),
            d(
                "十二个分盘（D-3/4/7/10/12/16/20/24/27/40/45/60）",
                Det,
                "Parasara 一系。两个彼此独立的开源实现逐条对照——kunjara/jyotish（PHP，每盘只实现此法）\
                 与 PyJHora（Python，每盘并列 3–6 法，取其 Parasara 默认）——在 12 盘 × 12 宫 × 300 点 \
                 共 43 200 个点上零分歧。D-10 另有原典 BPHS 6.13 直证",
            ),
            d(
                "分盘的其余诸法（Parivritti / Somanatha / Jaganatha 等）",
                Und,
                "🟡 每盘都不止一法：PyJHora 逐盘并列 3–6 种（Parivritti cyclic / even-reverse、\
                 Somanatha alternate、Jaganatha、以及若干 Parasara 变体如「偶宫逆数」）。\
                 本叶只出 Parasara 一系并在此声明其余，不静默选边。要收哪一法须各自找 ≥2 源",
            ),
            d("D-2 hora 落宫", Und, "🟡 BPHS 6.5-6 只说奇宫前半属日后半属月、偶宫相反，**没有指定落哪个宫**。日→Leo / 月→Cancer 是后世注家所补，另有 Raman、Kashinatha、Parivritti Dwaya、Somanatha 等至少五种活跃流派"),
            d("D-30 偶宫弧长", Und, "🟡 BPHS 6.27 作 vyatyayāt same（偶宫反转），反的是弧长还是只反主星，梵文两可；两个开源实现取另一读，实测分别在 6.7% 与 20% 的度数点上不合。另：份内度数原典未定义，各实现一律沿用等分 1°，与不等分宫位互相矛盾"),
            d("JHora 的 D-10 变体", Und, "🟡 Jagannatha Hora 对偶宫走「第 5 宫逆数 + 份内度数 30−x」（软件标 D-10 (5-8)），与 BPHS 6.13 冲突且无文本依据——node-jhora 的 varga audit 文档明记此事并把默认改回原典读法。本叶的 D-10 按原典"),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到统一契约上：元数据齐备、能出盘、确定性谱已声明、
    /// 有流派时恰有一个默认。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = JyotishEngine;
        assert!(!e.id().is_empty() && !e.name().is_empty());
        let m = Moment::new(1990, 6, 15, 14, 30, 8.0);
        let q = Query::at(1990, 6, 15, 14, 30, 8.0);
        assert!(!e.cast(&m, &q).is_null(), "每片叶都应产出非空盘面");
        assert!(!e.profile().is_empty(), "每片叶都要显式声明确定性谱");
        let defaults = e.schools().iter().filter(|s| s.default).count();
        assert!(e.schools().is_empty() || defaults == 1, "有流派的叶应恰有一个默认");
        assert!(!e.family().label().is_empty());
    }
}
