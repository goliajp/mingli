//! 格局：伏吟 / 反吟、三奇合吉门、以及天地盘干相加构成的诸格。
//!
//! 只出**结构性**判定（能由盘面直接读出的）。各格的吉凶归类照录古籍自身的分卷
//! （吉格 / 凶格），判读仍属释义层。

use super::*;

/// 三吉门：开 · 休 · 生。
pub const JI_MEN: [&str; 3] = ["开门", "休门", "生门"];

/// 三奇：乙（日奇）· 丙（月奇）· 丁（星奇）。
pub const SAN_QI: [&str; 3] = ["乙", "丙", "丁"];

/// 一处「三奇合吉门」：某宫的天盘三奇与三吉门同宫。
///
/// ⚠ 这**不是**三奇得使。《烟波钓叟歌》把二者分作两句（「吉门偶尔合三奇」与
/// 「三奇得使诚堪使」），《奇门遁甲秘笈大全》卷十五也把「三奇上吉门格」与
/// 「三奇得使格」分列为两条。得使是天盘奇加地盘旬首之仪，见 [`QiDeShi`]。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct QiGate {
    /// 宫号 1..=9。
    pub palace: u8,
    /// 天盘三奇之一。
    pub qi: &'static str,
    /// 同宫的吉门。
    pub gate: &'static str,
}

/// 盘面结构格局。
///
/// 这里只出**结构事实**（哪几处成立）与古籍自身的吉凶归类，不出断语——判读属释义层。
/// 只收多源无争议的几类：伏吟 / 反吟（由旋转格数直接判定）、三奇合吉门、
/// 天地盘干相加的八格（见 [`STEM_PATTERNS`]）与三奇得使（见 [`QI_DE_SHI_PAIRS`]）。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[allow(
    clippy::struct_excessive_bools,
    reason = "星/门 × 伏吟/反吟 是四条彼此独立、可同时成立的判定，摊平比塞进枚举更贴合盘面"
)]
pub struct Patterns {
    /// 星伏吟：天盘九星各归原宫（旋转 0 格）。
    pub star_fu_yin: bool,
    /// 星反吟：天盘九星各落原宫的对冲宫（旋转 4 格）。
    pub star_fan_yin: bool,
    /// 门伏吟：八门各归本位（旋转 0 格）。
    pub gate_fu_yin: bool,
    /// 门反吟：八门各落本位的对冲宫（旋转 4 格）。
    pub gate_fan_yin: bool,
    /// 干伏吟的宫号：该宫天盘干与地盘干相同。
    pub stem_fu_yin_palaces: Vec<u8>,
    /// 全盘伏吟：星 · 门 · 干三者俱伏。
    pub full_fu_yin: bool,
    /// 三奇合吉门的各处（不是得使，见 [`QiGate`]）。
    pub qi_gates: Vec<QiGate>,
    /// 三奇得使（严格版）的各处。
    pub qi_de_shi: Vec<QiDeShi>,
    /// 天地盘干相加构成的诸格。
    pub stem_patterns: Vec<StemPattern>,
}

/// 一处三奇得使（严格版）：天盘三奇落在**地盘对应旬首之仪**上。
///
/// 六组配对四源一致（《奇门遁甲统宗》卷一奇门四十格 ·《遁甲演义》卷二 ·
/// 《奇门法窍》卷六吉格注释 ·《奇门遁甲秘笈大全》卷十五）：
/// 乙配甲戌己、甲午辛；丙配甲子戊、甲申庚；丁配甲辰壬、甲寅癸。
/// 口诀自证：「乙逢犬马丙鼠猴，六丁玉女骑龙虎」——犬戌马午归乙、鼠子猴申归丙、龙辰虎寅归丁。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct QiDeShi {
    /// 宫号 1..=9。
    pub palace: u8,
    /// 三奇（乙 / 丙 / 丁）。
    pub qi: &'static str,
    /// 地盘之仪。
    pub yi: &'static str,
    /// 该仪所对应的旬首。
    pub xun_head: &'static str,
    /// 同一判据下**同时**构成的凶格。
    ///
    /// 六组里有三组与凶格判据完全相同——不是可能共现，是同一个盘面：
    /// 地盘辛恒为甲午辛，所以「乙加甲午」与「乙加辛（青龙逃走）」是一回事。
    /// 《遁甲演义》卷二判这三组「尚有微疵不吉……如遇本旬直符同临其上，方可用之而吉」。
    pub conflicting: Option<&'static str>,
}

/// 一处天地盘干相加之格：天盘某干落在地盘某干之上。
///
/// 收录的七格（含互为反向的三对）在四层独立编纂里条件与方向全部一致：
/// 《奇门遁甲统宗》卷一「奇门四十格」·《遁甲演义》卷二逐格详解（引赤松子 / 王璋）·
/// 《奇门法窍》卷六吉凶格注释 ·《奇门遁甲秘笈大全》卷十五。
/// 《遁甲演义》所引王璋的表述最直白，两处都写作「**天上**六 X 加**地下**六 Y」，方向无歧义。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StemPattern {
    /// 宫号 1..=9。
    pub palace: u8,
    /// 格名（正名；异写见 [`STEM_PATTERNS`] 的注）。
    pub name: &'static str,
    /// 天盘干。
    pub sky: &'static str,
    /// 地盘干。
    pub earth: &'static str,
    /// 古籍**自身**的分卷归类：「吉」或「凶」。照录，不是本层的判断。
    pub classical_class: &'static str,
}

/// 圆周上相隔 4 格即对冲宫（坎 1 ↔ 离 9 · 坤 2 ↔ 艮 8 · 震 3 ↔ 兑 7 · 巽 4 ↔ 乾 6）。
const FAN_YIN_SHIFT: u8 = 4;

/// 三奇得使的六组配对：`(三奇, 地盘之仪, 旬首, 同判据的凶格)`。
pub const QI_DE_SHI_PAIRS: [(&str, &str, &str, Option<&str>); 6] = [
    ("乙", "己", "甲戌", None),
    ("乙", "辛", "甲午", Some("青龙逃走")),
    ("丙", "戊", "甲子", None), // 同时是飞鸟跌穴，同为吉格，不算冲突
    ("丙", "庚", "甲申", Some("荧入太白")),
    ("丁", "壬", "甲辰", None),
    ("丁", "癸", "甲寅", Some("朱雀投江")),
];

/// 天地盘干相加之格：`(天盘干, 地盘干, 正名, 古籍归类)`。
///
/// 异写：螣蛇夭矫在《遁甲演义》系作「腾蛇跃跷」，《奇门法窍》与《秘笈大全》作「妖蹻 / 妖矫」，
/// 《统宗》兼用「妖矫 / 夭矫」——同一格。青龙返首亦作「青龙回首」。
///
/// 三对反向格的吉凶不对称：返首 / 跌穴俱吉，而猖狂 / 逃走俱凶、夭矫 / 投江俱凶——
/// 方向弄反不会吉凶颠倒，但会张冠李戴（《秘笈大全》卷十一：「乙加辛宜防败北，辛加乙宜勿图谋」）。
pub const STEM_PATTERNS: [(&str, &str, &str, &str); 8] = [
    ("戊", "丙", "青龙返首", "吉"),
    ("丙", "戊", "飞鸟跌穴", "吉"),
    ("辛", "乙", "白虎猖狂", "凶"),
    ("乙", "辛", "青龙逃走", "凶"),
    ("癸", "丁", "螣蛇夭矫", "凶"),
    ("丁", "癸", "朱雀投江", "凶"),
    ("丙", "庚", "荧入太白", "凶"),
    ("庚", "丙", "太白入荧", "凶"),
];

/// 判盘面格局（只出结构，不下断语）。
#[must_use]
pub fn patterns(earth: &[&'static str; 9], sky: &SkyPlate, gates: &GatePlate) -> Patterns {
    let stem_fu_yin_palaces: Vec<u8> = (1..=9u8)
        .filter(|&p| {
            let k = p as usize - 1;
            !sky.stems[k].is_empty() && sky.stems[k] == earth[k]
        })
        .collect();
    let star_fu_yin = sky.shift == 0;
    let gate_fu_yin = gates.shift == 0;
    let qi_gates = (1..=9u8)
        .filter_map(|p| {
            let k = p as usize - 1;
            let qi = SAN_QI.iter().find(|&&q| q == sky.stems[k])?;
            let gate = JI_MEN.iter().find(|&&g| g == gates.gates[k])?;
            Some(QiGate { palace: p, qi, gate })
        })
        .collect();
    // 中五宫无天盘干（寄坤二），扫描时天盘干为空串，自然不匹配任何格。
    let qi_de_shi = (1..=9u8)
        .filter_map(|p| {
            let k = p as usize - 1;
            let (qi, yi, xun_head, conflicting) = QI_DE_SHI_PAIRS
                .iter()
                .find(|(q, y, _, _)| *q == sky.stems[k] && *y == earth[k])?;
            Some(QiDeShi { palace: p, qi, yi, xun_head, conflicting: *conflicting })
        })
        .collect();
    let stem_patterns = (1..=9u8)
        .filter_map(|p| {
            let k = p as usize - 1;
            let (sky_stem, earth_stem, name, class) = STEM_PATTERNS
                .iter()
                .find(|(s, e, _, _)| *s == sky.stems[k] && *e == earth[k])?;
            Some(StemPattern {
                palace: p,
                name,
                sky: sky_stem,
                earth: earth_stem,
                classical_class: class,
            })
        })
        .collect();
    Patterns {
        star_fu_yin,
        star_fan_yin: sky.shift == FAN_YIN_SHIFT,
        gate_fu_yin,
        gate_fan_yin: gates.shift == FAN_YIN_SHIFT,
        full_fu_yin: star_fu_yin && gate_fu_yin && !stem_fu_yin_palaces.is_empty(),
        stem_fu_yin_palaces,
        qi_gates,
        qi_de_shi,
        stem_patterns,
    }
}
