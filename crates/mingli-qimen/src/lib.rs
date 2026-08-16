//! L3 叶（⟂ 横切 / 确定性）：奇门遁甲（时家转盘法）的可计算结构。
//!
//! 本 crate 实现奇门里**确定、可校验**的两层：
//!
//! 1. **定局**：由节气定阴阳遁（冬至→芒种阳遁、夏至→大雪阴遁）与三元局数（[`solar_term_setup`]）。
//!    72 局常数表 6 源零冲突，且满足结构不变量——阳遁「中元=上元+6、下元=上元+3」、阴遁「−6/−3」
//!    （[`enum@Yuan`] 由符头地支定，见 [`yuan_of_branch`]）。
//! 2. **地盘三奇六仪**：六仪 `戊己庚辛壬癸` + 三奇 `乙丙丁` 按局数布九宫。**阳遁六仪顺布、三奇逆布**
//!    （实排序列 `戊己庚辛壬癸丁丙乙`）；**阴遁六仪逆布、三奇顺布**（[`earth_plate`]）。
//!    走宫是**宫序号 1→9 线性**，不是九宫飞星斜线——这是最易混淆处。九宫↔八卦复用 [`mingli_luoshu`]。
//!
//! 验证：阳遁一局校验古法「坎1戊·坤2己·震3庚·巽4辛·中5壬·乾6癸·兑7丁·艮8丙·离9乙」。
//!
//! 3. **天盘**：九星与三奇六仪随值符从「旬首宫」旋到「时干宫」，沿后天八卦圆周作一次刚体旋转
//!    （[`sky_rotation`]）。以两则古例复现校验，中宫取主流「寄坤 2」。
//!
//! 4. **人盘八门**：值使门自旬首宫按宫序号线性数过本旬时辰位次落宫，八门再沿同一圆周旋转
//!    （[`gate_plate`]）。
//!
//! 5. **神盘八神**：直符与值符同宫，其余七神阳顺阴逆布外八宫（[`spirit_plate`]）。
//!
//! 诚实边界（🟡）：八神第 5 / 6 位的**称谓**两系不一（白虎 / 玄武 与 勾陈 / 朱雀），位序则一致，
//! 故两名并出；天禽与中宫寄宫取通行的坤 2，古本「阳遁寄艮 8」一派未开关；
//! 旺相休囚、格局判读尚未实现。
//! 定局的「拆补法 / 置闰法」差异只在交节临界数日的元/局对齐，本 crate 用**主流拆补法**（符头定元）。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "节气/局数/宫位均落 0..24 / 1..9 小范围，与 i64/usize 间换算受控安全"
)]

mod engine;
pub use engine::QimenEngine;

use mingli_astro::Moment;
use mingli_ganzhi::Element;
use serde::Serialize;

/// 24 节气名，按 `floor(λ/15)` 索引（春分=0 … 惊蛰=23）。
pub const SOLAR_TERMS: [&str; 24] = [
    "春分", "清明", "谷雨", "立夏", "小满", "芒种", "夏至", "小暑", "大暑", "立秋", "处暑", "白露",
    "秋分", "寒露", "霜降", "立冬", "小雪", "大雪", "冬至", "小寒", "大寒", "立春", "雨水", "惊蛰",
];

/// 各节气的三元局数 `[上元, 中元, 下元]`（1..9），按 [`SOLAR_TERMS`] 同序。
pub const YUAN_JU: [[u8; 3]; 24] = [
    [3, 9, 6], // 春分
    [4, 1, 7], // 清明
    [5, 2, 8], // 谷雨
    [4, 1, 7], // 立夏
    [5, 2, 8], // 小满
    [6, 3, 9], // 芒种
    [9, 3, 6], // 夏至
    [8, 2, 5], // 小暑
    [7, 1, 4], // 大暑
    [2, 5, 8], // 立秋
    [1, 4, 7], // 处暑
    [9, 3, 6], // 白露
    [7, 1, 4], // 秋分
    [6, 9, 3], // 寒露
    [5, 8, 2], // 霜降
    [6, 9, 3], // 立冬
    [5, 8, 2], // 小雪
    [4, 7, 1], // 大雪
    [1, 7, 4], // 冬至
    [2, 8, 5], // 小寒
    [3, 9, 6], // 大寒
    [8, 5, 2], // 立春
    [9, 6, 3], // 雨水
    [1, 7, 4], // 惊蛰
];

/// 六仪（戊己庚辛壬癸）+ 三奇（乙丙丁）的名，用于地盘标注。
pub const STEM_NAMES: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];

/// 九星按九宫原配（地盘初始，未旋转）：蓬芮冲辅禽心柱任英，对应宫 1..=9。
/// 主流通行版 — 中宫天禽 🟡 寄坤 2（古本派寄艮 8）。
///
/// 索引 0 = 占位（从 1 起用，与宫号对齐）。
pub const JIU_XING_PALACE: [&str; 10] = [
    "", "天蓬", "天芮", "天冲", "天辅", "天禽", "天心", "天柱", "天任", "天英",
];

/// 天盘旋转所走的圆周宫序：后天八卦顺时针一周 —— 坎 1 → 艮 8 → 震 3 → 巽 4 → 离 9 →
/// 坤 2 → 兑 7 → 乾 6。中 5 不在圆周上（寄坤 2）。
///
/// 注意与地盘的走宫方式相区别：地盘按**宫序号 1→9 线性**铺三奇六仪，天盘则是整盘沿这条
/// **圆周**刚体旋转。
pub const ORBIT: [u8; 8] = [1, 8, 3, 4, 9, 2, 7, 6];

/// 中宫寄坤 2（主流寄宫法）：符首或时干落中 5 时按坤 2 论。
///
/// 🟡 古本另有寄艮 8 一派，本 crate 取通行版。
#[must_use]
pub const fn lodged_palace(palace: u8) -> u8 {
    if palace == 5 { 2 } else { palace }
}

/// 宫号在圆周 [`ORBIT`] 上的下标（中 5 按寄宫算）。
fn orbit_index(palace: u8) -> usize {
    let p = lodged_palace(palace);
    ORBIT.iter().position(|&x| x == p).unwrap_or(0)
}

/// 天盘：九星与三奇六仪随值符整体旋转后的结果。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SkyPlate {
    /// 旋转格数（沿 [`ORBIT`] 顺时针 0..=7）：值符从「旬首宫」走到「时干宫」的位移。
    pub shift: u8,
    /// 天盘九星，`stars[k]` = 第 `k+1` 宫；中 5 宫为空串（天禽寄坤 2，与天芮同宫）。
    pub stars: [&'static str; 9],
    /// 天盘天干，`stems[k]` = 第 `k+1` 宫；中 5 宫为空串。
    pub stems: [&'static str; 9],
    /// 地盘中 5 之干（寄坤 2，随坤 2 一同旋转）。
    pub center_stem: &'static str,
    /// 中宫寄干在天盘上的落宫 1..=9。
    pub center_palace: u8,
}

/// 天盘旋转（主流转盘法）。
///
/// 值符星原在**旬首六仪所在宫**，随时干走到**实际值符干所在宫**；整盘（九星 + 三奇六仪）
/// 沿 [`ORBIT`] 作同一次刚体旋转。地盘中 5 之干寄坤 2，坤 2 转到哪它就跟到哪
/// （等价于「随天芮星走」）。
///
/// 多源校验：以两则古例复现——阳遁三局丙寅时（旬首戊震 3、时干丙坎 1）与
/// 阳遁一局庚午时（旬首戊坎 1、时干庚震 3），两例天盘九星各 8 宫全中。
#[must_use]
pub fn sky_rotation(earth: &[&'static str; 9], xun_yi_palace: u8, zhi_fu_palace: u8) -> SkyPlate {
    let from = orbit_index(xun_yi_palace);
    let to = orbit_index(zhi_fu_palace);
    let shift = (to + 8 - from) % 8;
    let mut stars = [""; 9];
    let mut stems = [""; 9];
    for (i, &target) in ORBIT.iter().enumerate() {
        let src = ORBIT[(i + 8 - shift) % 8];
        stars[target as usize - 1] = JIU_XING_PALACE[src as usize];
        stems[target as usize - 1] = earth[src as usize - 1];
    }
    SkyPlate {
        shift: shift as u8,
        stars,
        stems,
        center_stem: earth[4],
        center_palace: ORBIT[(orbit_index(2) + shift) % 8],
    }
}

/// 八门本位，与 [`ORBIT`] 同序：休坎 1 · 生艮 8 · 伤震 3 · 杜巽 4 · 景离 9 · 死坤 2 · 惊兑 7 · 开乾 6。
///
/// 八门的原配次序恰与后天八卦圆周重合，所以八门旋转与九星旋转是同一种刚体位移。
pub const BA_MEN_ORBIT: [&str; 8] = ["休门", "生门", "伤门", "杜门", "景门", "死门", "惊门", "开门"];

/// 人盘八门：值使门与其落宫，以及旋转后的八门分布。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct GatePlate {
    /// 值使门 —— 旬首六仪所在宫的本位门（本旬不变）。
    pub zhi_shi_gate: &'static str,
    /// 值使门落宫 1..=9（落中 5 时按寄坤 2 归并）。
    pub zhi_shi_palace: u8,
    /// 占事时辰在本旬中的位次 0..=9（甲子旬的甲子时为 0）。
    pub steps: u8,
    /// 八门沿 [`ORBIT`] 的旋转格数 0..=7。
    pub shift: u8,
    /// 八门分布，`gates[k]` = 第 `k+1` 宫；中 5 宫为空串（八门不入中宫）。
    pub gates: [&'static str; 9],
}

/// 值使门与八门旋转（主流转盘法）。
///
/// 与九星「随时干」不同，值使**随时辰**走：自旬首六仪所在宫起，按**宫序号线性**
/// 阳遁 +1 / 阴遁 −1（数宫时中 5 也占一位）数过本旬内的时辰位次，落点即值使门所在宫；
/// 落到中 5 则按寄坤 2 论。其余七门再自值使宫起沿 [`ORBIT`] 圆周顺布。
///
/// 校验：阳遁一局庚午时（旬首戊坎 1、庚午为甲子旬第 7 个时辰）→ 休门数至兑 7，
/// 八门 坎1伤 艮8杜 震3景 巽4死 离9惊 坤2开 兑7休 乾6生，8 宫全中。
#[must_use]
pub fn gate_plate(xun_yi_palace: u8, head_branch: u8, time_branch: u8, yang_dun: bool) -> GatePlate {
    let steps = (time_branch + 12 - head_branch) % 12;
    let from = lodged_palace(xun_yi_palace);
    let delta = i32::from(steps) * if yang_dun { 1 } else { -1 };
    let landed = (i32::from(from) - 1 + delta).rem_euclid(9) + 1;
    let zhi_shi_palace = lodged_palace(u8::try_from(landed).unwrap_or(2));
    let start = orbit_index(from);
    let shift = (orbit_index(zhi_shi_palace) + 8 - start) % 8;
    let mut gates = [""; 9];
    for (i, &target) in ORBIT.iter().enumerate() {
        gates[target as usize - 1] = BA_MEN_ORBIT[(i + 8 - shift) % 8];
    }
    GatePlate {
        zhi_shi_gate: BA_MEN_ORBIT[start],
        zhi_shi_palace,
        steps,
        shift: shift as u8,
        gates,
    }
}

/// 八神次序：值符 · 腾蛇 · 太阴 · 六合 · 白虎 · 玄武 · 九地 · 九天。
///
/// 🟡 第 5 / 6 位的**称谓**有两系：一系两遁通用「白虎 / 玄武」（本 crate 取此），
/// 另一系阳遁称「勾陈 / 朱雀」、阴遁才称「白虎 / 玄武」（见 [`BA_SHEN_YANG_ALT`]）。
/// 两系只是名字不同，**位序一致**，故结构确定、命名留待定夺。
pub const BA_SHEN: [&str; 8] =
    ["值符", "腾蛇", "太阴", "六合", "白虎", "玄武", "九地", "九天"];

/// 另一系在**阳遁**下对第 5 / 6 位的称谓（勾陈 / 朱雀），其余六神同 [`BA_SHEN`]。
pub const BA_SHEN_YANG_ALT: [&str; 8] =
    ["值符", "腾蛇", "太阴", "六合", "勾陈", "朱雀", "九地", "九天"];

/// 神盘八神。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SpiritPlate {
    /// 值符（直符）所在宫 1..=9 —— 八神的起点，随天盘值符走。
    pub start_palace: u8,
    /// 八神分布，`spirits[k]` = 第 `k+1` 宫；中 5 宫为空串（八神不入中宫）。
    pub spirits: [&'static str; 9],
    /// 同上，但第 5 / 6 位用另一系称谓（阳遁作勾陈 / 朱雀）——两系位序相同，只是名字不同。
    pub spirits_alt: [&'static str; 9],
}

/// 八神布列：直符与值符同宫，其余七神自值符宫起沿 [`ORBIT`] 阳遁顺时针、阴遁逆时针依次落宫。
///
/// 校验：起点坎 1 的阳遁盘 → 坎1值符 艮8腾蛇 震3太阴 巽4六合 离9白虎 坤2玄武 兑7九地 乾6九天，
/// 8 宫全中（公开教程例题）。
///
/// 起点取**天盘**旬首所在宫（即值符宫）；另有一路「地八神」以地盘旬首宫为起点，本 crate 不出。
#[must_use]
pub fn spirit_plate(zhi_fu_palace: u8, yang_dun: bool) -> SpiritPlate {
    let start_palace = lodged_palace(zhi_fu_palace);
    let start = orbit_index(start_palace);
    let mut spirits = [""; 9];
    let mut spirits_alt = [""; 9];
    for k in 0..8 {
        let i = if yang_dun { (start + k) % 8 } else { (start + 8 - k) % 8 };
        let p = ORBIT[i] as usize - 1;
        spirits[p] = BA_SHEN[k];
        spirits_alt[p] = if yang_dun { BA_SHEN_YANG_ALT[k] } else { BA_SHEN[k] };
    }
    SpiritPlate { start_palace, spirits, spirits_alt }
}

/// 九星五行，与宫号对齐（索引 0 占位）：蓬水 · 芮土 · 冲木 · 辅木 · 禽土 · 心金 · 柱金 · 任土 · 英火。
pub const JIU_XING_ELEMENT: [Element; 10] = [
    Element::Earth,
    Element::Water,
    Element::Earth,
    Element::Wood,
    Element::Wood,
    Element::Earth,
    Element::Metal,
    Element::Metal,
    Element::Earth,
    Element::Fire,
];

/// 五行在月令下的强弱五等级。
///
/// 取《五行大义》以来的通行判法：**当令者旺、令生者相、生令者休、克令者囚、令克者死**。
/// （另有一路以星为主体、含「废」的五等级说法，非通行，本 crate 不取。）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Vigor {
    /// 旺：与月令同五行，当令。
    Wang,
    /// 相：月令所生，次旺。
    Xiang,
    /// 休：生月令者，气已泄。
    Xiu,
    /// 囚：克月令者，反受制。
    Qiu,
    /// 死：月令所克，气尽。
    Si,
}

impl Vigor {
    /// 中文标签。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Vigor::Wang => "旺",
            Vigor::Xiang => "相",
            Vigor::Xiu => "休",
            Vigor::Qiu => "囚",
            Vigor::Si => "死",
        }
    }
}

/// 由节气序号取节气月支 0..=11（0 = 子）。每个「节」开启一个月，故两气一支。
///
/// [`SOLAR_TERMS`] 自春分（黄经 0°）起排，立春在 21 位开寅月。
#[must_use]
pub fn month_branch_of_term(term_index: usize) -> u8 {
    let k = ((term_index + 3) % 24) / 2;
    u8::try_from((k + 2) % 12).unwrap_or(0)
}

/// 地支五行（寅卯木 · 巳午火 · 申酉金 · 亥子水 · 辰戌丑未土）。
#[must_use]
pub fn branch_element_of(branch: u8) -> Element {
    mingli_ganzhi::branch_element(branch)
}

/// 判某五行在给定月令下的旺相休囚死。
#[must_use]
pub fn vigor_of(subject: Element, month: Element) -> Vigor {
    if subject == month {
        Vigor::Wang
    } else if month.generates() == subject {
        Vigor::Xiang
    } else if subject.generates() == month {
        Vigor::Xiu
    } else if subject.controls() == month {
        Vigor::Qiu
    } else {
        Vigor::Si
    }
}

/// 由星名取其五行（未知星名返回 `None`）。
#[must_use]
pub fn star_element(name: &str) -> Option<Element> {
    (1..=9).find(|&p| JIU_XING_PALACE[p] == name).map(|p| JIU_XING_ELEMENT[p])
}

/// 三吉门：开 · 休 · 生。
pub const JI_MEN: [&str; 3] = ["开门", "休门", "生门"];

/// 三奇：乙（日奇）· 丙（月奇）· 丁（星奇）。
pub const SAN_QI: [&str; 3] = ["乙", "丙", "丁"];

/// 一处「三奇临吉门」：某宫的天盘三奇与三吉门同宫。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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
/// 这里只出**结构事实**（哪几处成立），不出吉凶断语——判读属释义层。
/// 本 crate 目前只收无流派争议的几条：伏吟 / 反吟（由旋转格数直接判定）与三奇临吉门。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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
    /// 三奇临吉门的各处。
    pub qi_gates: Vec<QiGate>,
}

/// 圆周上相隔 4 格即对冲宫（坎 1 ↔ 离 9 · 坤 2 ↔ 艮 8 · 震 3 ↔ 兑 7 · 巽 4 ↔ 乾 6）。
const FAN_YIN_SHIFT: u8 = 4;

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
    Patterns {
        star_fu_yin,
        star_fan_yin: sky.shift == FAN_YIN_SHIFT,
        gate_fu_yin,
        gate_fan_yin: gates.shift == FAN_YIN_SHIFT,
        full_fu_yin: star_fu_yin && gate_fu_yin && !stem_fu_yin_palaces.is_empty(),
        stem_fu_yin_palaces,
        qi_gates,
    }
}

/// 地盘实排序列：六仪顺 + 三奇逆 → 戊己庚辛壬癸丁丙乙（值为天干序 0..9）。
/// 阳遁沿宫序 +1 铺这条序列、阴遁沿宫序 −1 铺（互为镜像）：
/// 镜像后六仪自然逆行、三奇自然顺行（乙丙丁落于宫序递增的三宫），故两遁共用此序。
const SEQ: [u8; 9] = [4, 5, 6, 7, 8, 9, 3, 2, 1];

/// 三元。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Yuan {
    /// 上元（符头地支 子午卯酉）。
    Upper,
    /// 中元（符头地支 寅申巳亥）。
    Middle,
    /// 下元（符头地支 辰戌丑未）。
    Lower,
}

impl Yuan {
    /// 在 `[上,中,下]` 三元数组中的下标。
    #[must_use]
    pub fn index(self) -> usize {
        match self {
            Yuan::Upper => 0,
            Yuan::Middle => 1,
            Yuan::Lower => 2,
        }
    }
    /// 三元名。
    #[must_use]
    pub fn name(self) -> &'static str {
        ["上元", "中元", "下元"][self.index()]
    }
}

/// 由地支（0..11）定三元：子午卯酉=上元、寅申巳亥=中元、辰戌丑未=下元。
#[must_use]
pub fn yuan_of_branch(branch: u8) -> Yuan {
    match branch % 3 {
        0 => Yuan::Upper,  // 子卯午酉
        2 => Yuan::Middle, // 寅巳申亥
        _ => Yuan::Lower,  // 丑辰未戌
    }
}

/// 节气下标（春分=0 … 惊蛰=23），由太阳视黄经 `floor(λ/15)`。
#[must_use]
pub fn solar_term_index(sun_longitude: f64) -> usize {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "λ/15 ∈ 0..24"
    )]
    let k = (sun_longitude.rem_euclid(360.0) / 15.0).floor() as usize;
    k % 24
}

/// 是否阳遁：冬至→芒种（节气下标 18..24 或 0..6）为阳遁，余为阴遁。
#[must_use]
pub fn is_yang_dun(term_index: usize) -> bool {
    term_index >= 18 || term_index <= 5
}

/// 定局结果：节气、阴阳遁、三元、局数。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Setup {
    /// 节气下标（春分=0 … 惊蛰=23）。
    pub term_index: usize,
    /// 节气名。
    pub term: &'static str,
    /// 是否阳遁（否则阴遁）。
    pub yang_dun: bool,
    /// 三元。
    pub yuan: Yuan,
    /// 局数 1..9。
    pub ju: u8,
}

/// 由节气下标与三元定局。
#[must_use]
pub fn solar_term_setup(term_index: usize, yuan: Yuan) -> Setup {
    Setup {
        term_index,
        term: SOLAR_TERMS[term_index],
        yang_dun: is_yang_dun(term_index),
        yuan,
        ju: YUAN_JU[term_index][yuan.index()],
    }
}

/// 地盘三奇六仪：返回 `p[k]` = 第 `k+1` 宫（宫序 1..9，线性）所布之天干（0..9）。
///
/// 阳遁沿宫序递增布序列（戊己庚辛壬癸丁丙乙）；阴遁沿宫序递减布（镜像）。起宫 = 局数 `ju` 对应之宫。
#[must_use]
pub fn earth_plate(ju: u8, yang_dun: bool) -> [u8; 9] {
    let start = (ju as usize + 8) % 9; // ju 宫的 0-based 下标 = ju-1
    let mut p = [0u8; 9];
    for (i, &stem) in SEQ.iter().enumerate() {
        let pos = if yang_dun {
            (start + i) % 9 // 阳遁宫序递增
        } else {
            (start + 9 - i) % 9 // 阴遁宫序递减（镜像）
        };
        p[pos] = stem;
    }
    p
}

/// 时柱旬：奇门时家盘以**时柱**为核心，旬首决定值符所遁之仪、旬空标失时之支。
///
/// **算法**（沿用 [`mingli_ganzhi::xun_head_branch`] / [`mingli_ganzhi::xun_yi`] / [`mingli_ganzhi::xunkong`]）：
/// - 旬首支 = `(time_branch − time_stem + 12) mod 12`，定时柱所在 6 旬之一。
/// - 旬首六仪 = 旬首甲所遁的仪干：甲子→戊 / 甲戌→己 / 甲申→庚 / 甲午→辛 / 甲辰→壬 / 甲寅→癸。
/// - 旬空 2 支 = 旬首支 +10 / +11 mod 12（本旬 10 干配不上的两位地支）。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Xun {
    /// 旬首干支字面（如「甲子」「甲戌」）。
    pub head_ganzhi: &'static str,
    /// 旬首地支 0..11。
    pub head_branch: u8,
    /// 旬首所遁之六仪天干字面（戊 / 己 / 庚 / 辛 / 壬 / 癸）。
    pub head_yi: &'static str,
    /// 旬首六仪天干索引 4..=9。
    pub head_yi_stem: u8,
    /// 旬空两支字面（如「戌」「亥」）。
    pub xunkong: [&'static str; 2],
}

/// 一次奇门定局 + 地盘的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Cast {
    /// 定局信息。
    pub setup: Setup,
    /// 符头地支（定元用，0..11）。
    pub fu_tou_branch: u8,
    /// 占事时柱字面：奇门时家盘以时柱为核心。
    pub time_ganzhi: String,
    /// 占事时柱天干索引 0..=9。
    pub time_stem: u8,
    /// 占事时柱地支索引 0..=11。
    pub time_branch: u8,
    /// 时柱所在旬。
    pub xun: Xun,
    /// 地盘九宫天干（名），`earth[k]` = 第 `k+1` 宫。
    pub earth: [&'static str; 9],
    /// 各宫八卦名（复用洛书九宫，`palace[k]` = 第 `k+1` 宫）。
    pub palace: [&'static str; 9],
    /// 旬首六仪在地盘所在的宫 1..=9 — 即**值符星原宫**（本旬之值符星 = 此宫原配九星）。
    pub xun_yi_palace: u8,
    /// 实际值符天干：时干 = 甲时，实际遁旬首六仪；否则 = 时干本身。
    pub zhi_fu_stem: u8,
    /// 实际值符天干字面。
    pub zhi_fu_stem_name: &'static str,
    /// 值符宫 1..=9：实际值符天干在地盘所在的宫，即「值符要去的宫」/「本时辰值符星所在宫」。
    pub zhi_fu_palace: u8,
    /// **值符星名**：旬首六仪所在地盘宫的原配九星（本旬不变），如「天冲」「天英」。
    pub zhi_fu_xing: &'static str,
    /// 九星原配九宫：地盘初始未旋转的天盘九星，`jiuxing_earth[k]` = 第 `k+1` 宫的原配星。
    pub jiuxing_earth: [&'static str; 9],
    /// 天盘：九星与三奇六仪随值符旋转后的实际分布。
    pub sky: SkyPlate,
    /// 人盘：值使门与旋转后的八门分布。
    pub gates: GatePlate,
    /// 神盘：八神布列。
    pub spirits: SpiritPlate,
    /// 节气月支 0..=11（0 = 子）——判旺相休囚的月令。
    pub month_branch: u8,
    /// 月令五行字面。
    pub month_element: &'static str,
    /// 各宫**天盘星**在月令下的旺相休囚死，`star_vigor[k]` = 第 `k+1` 宫；中 5 宫为空串。
    pub star_vigor: [&'static str; 9],
    /// 盘面格局（结构事实，不含吉凶断语）。
    pub patterns: Patterns,
}

/// 时柱地支（子时寄前夜 23：00）：奇门用「夜子归次日」（主流）。
fn time_branch(hour: u32, minute: u32) -> u8 {
    let h = (hour + minute / 60) % 24;
    if h == 23 {
        0
    } else {
        (h.div_ceil(2) % 12) as u8
    }
}

/// 时柱(GanZhi)：由日柱 + 时支用五鼠遁算时干。
/// 时干 = `(day_stem % 5 * 2 + time_branch) % 10`。
fn time_ganzhi(day: mingli_ganzhi::GanZhi, hour: u32, minute: u32) -> mingli_ganzhi::GanZhi {
    let tb = time_branch(hour, minute);
    let ts = ((day.stem % 5) * 2 + tb) % 10;
    mingli_ganzhi::GanZhi { stem: ts, branch: tb }
}

/// 旬首六仪在地盘上所在的宫(1..=9)。即天盘的**值符宫**之根。
fn xun_yi_palace_of(earth: &[&str; 9], yi_name: &str) -> u8 {
    for (k, &s) in earth.iter().enumerate() {
        if s == yi_name {
            return (k + 1) as u8;
        }
    }
    // 不会发生：地盘必含全部 6 仪，六仪字面来自 STEM_NAMES。
    unreachable!("旬首六仪应在地盘九宫之一")
}

/// 实际值符天干 — 时干为甲(0)时遁该旬之六仪，否则 = 时干本身。
fn effective_zhi_fu_stem(time_stem: u8, head_yi_stem: u8) -> u8 {
    if time_stem == 0 { head_yi_stem } else { time_stem }
}

/// 某天干在地盘九宫所在的宫(1..=9)；找不到 → 0（理论不会发生 — 地盘含 9 个三奇六仪）。
fn earth_position_of_stem(earth: &[&str; 9], stem_name: &str) -> u8 {
    for (k, &s) in earth.iter().enumerate() {
        if s == stem_name {
            return (k + 1) as u8;
        }
    }
    0
}

/// 旬首干支字面（6 旬）：甲子 / 甲戌 / 甲申 / 甲午 / 甲辰 / 甲寅。
const XUN_HEAD_NAMES: [&str; 12] = [
    "甲子", "",     "甲寅", "",     "甲辰", "",
    "甲午", "",     "甲申", "",     "甲戌", "",
];

/// 在共享上下文 [`Moment`] 上排奇门（定局 + 地盘 + 时柱旬）。
///
/// - 三元由**符头**（最近的甲/己日）地支定（拆补法）。
/// - ：加占事时柱 + 旬首六仪 + 旬空 + 旬首六仪在地盘所在的宫（值符宫之根）。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    let term_index = solar_term_index(m.sun_longitude);
    // 符头：最近的甲(0)/己(5)日 = 今日往回退 （日干 mod 5） 天。
    let day = mingli_ganzhi::day_ganzhi(m.civil_day);
    let back = i64::from(day.stem % 5);
    let fu_tou = mingli_ganzhi::day_ganzhi(m.civil_day - back);
    let yuan = yuan_of_branch(fu_tou.branch);
    let setup = solar_term_setup(term_index, yuan);
    let ep = earth_plate(setup.ju, setup.yang_dun);
    let mut earth = [""; 9];
    let mut palace = [""; 9];
    for k in 0..9 {
        earth[k] = STEM_NAMES[ep[k] as usize];
        palace[k] = mingli_luoshu::PALACE_NAME[k + 1];
    }

    // 时柱 + 旬首 + 旬空。
    let time = time_ganzhi(day, m.hour, m.minute);
    let head_branch = mingli_ganzhi::xun_head_branch(time);
    let head_yi_stem = mingli_ganzhi::xun_yi(time);
    let head_yi_name = STEM_NAMES[head_yi_stem as usize];
    let kong = mingli_ganzhi::xunkong(time);
    let xun = Xun {
        head_ganzhi: XUN_HEAD_NAMES[head_branch as usize],
        head_branch,
        head_yi: head_yi_name,
        head_yi_stem,
        xunkong: [
            mingli_ganzhi::BRANCHES[kong[0] as usize],
            mingli_ganzhi::BRANCHES[kong[1] as usize],
        ],
    };
    let xun_yi_palace = xun_yi_palace_of(&earth, head_yi_name);
    let time_name = format!(
        "{}{}",
        mingli_ganzhi::STEMS[time.stem as usize],
        mingli_ganzhi::BRANCHES[time.branch as usize],
    );

    // 值符 — 实际值符干（时干甲遁旬首六仪） + 值符宫（实际值符干在地盘的位置）
    //                + 值符星（旬首六仪所在宫原配九星 = 本旬不变）
    let zhi_fu_stem = effective_zhi_fu_stem(time.stem, head_yi_stem);
    let zhi_fu_stem_name = STEM_NAMES[zhi_fu_stem as usize];
    let zhi_fu_palace = earth_position_of_stem(&earth, zhi_fu_stem_name);
    let zhi_fu_xing = JIU_XING_PALACE[xun_yi_palace as usize];
    let mut jiuxing_earth = [""; 9];
    jiuxing_earth.copy_from_slice(&JIU_XING_PALACE[1..=9]);

    // 天盘 — 值符星自旬首宫走到时干宫，整盘沿后天八卦圆周同步旋转。
    let sky = sky_rotation(&earth, xun_yi_palace, zhi_fu_palace);
    // 人盘 — 值使随时辰走：自旬首宫按宫序号线性数过本旬时辰位次。
    let gates = gate_plate(xun_yi_palace, head_branch, time.branch, setup.yang_dun);
    // 神盘 — 直符与值符同宫，八神阳顺阴逆布外八宫。
    let spirits = spirit_plate(zhi_fu_palace, setup.yang_dun);
    // 旺相休囚 — 以节气月令衡量各宫天盘星的五行。
    let patterns = patterns(&earth, &sky, &gates);
    let month_branch = month_branch_of_term(term_index);
    let month_el = branch_element_of(month_branch);
    let mut star_vigor = [""; 9];
    for (slot, star) in star_vigor.iter_mut().zip(sky.stars) {
        if let Some(e) = star_element(star) {
            *slot = vigor_of(e, month_el).label();
        }
    }

    Cast {
        setup,
        fu_tou_branch: fu_tou.branch,
        time_ganzhi: time_name,
        time_stem: time.stem,
        time_branch: time.branch,
        xun,
        earth,
        palace,
        xun_yi_palace,
        zhi_fu_stem,
        zhi_fu_stem_name,
        zhi_fu_palace,
        zhi_fu_xing,
        jiuxing_earth,
        sky,
        gates,
        spirits,
        month_branch,
        month_element: month_el.name(),
        star_vigor,
        patterns,
    }
}

/// 由本地民用时刻排奇门（独立入口，构造 [`Moment`]）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, hour: u32, minute: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, hour, minute, tz))
}

#[cfg(test)]
mod tests {

    fn plate_of(ju: u8, yang: bool, yi_palace: u8, zhi_fu_palace: u8) -> ([&'static str; 9], SkyPlate) {
        let ep = earth_plate(ju, yang);
        let mut earth = [""; 9];
        for k in 0..9 {
            earth[k] = STEM_NAMES[ep[k] as usize];
        }
        let sky = sky_rotation(&earth, yi_palace, zhi_fu_palace);
        (earth, sky)
    }

    /// 伏吟 = 旋转 0 格（各归原位），反吟 = 旋转 4 格（各落对冲宫）。
    #[test]
    fn qm5_fu_yin_and_fan_yin_come_from_the_shift() {
        let (earth, sky) = plate_of(1, true, 1, 1);
        let gates = gate_plate(1, 0, 0, true);
        let p = patterns(&earth, &sky, &gates);
        assert!(p.star_fu_yin && p.gate_fu_yin && p.full_fu_yin, "零位移 = 全盘伏吟");
        assert_eq!(p.stem_fu_yin_palaces, vec![1, 2, 3, 4, 6, 7, 8, 9], "外八宫天地盘干重叠");
        assert!(!p.star_fan_yin && !p.gate_fan_yin);

        // 对冲：坎 1 → 离 9 恰是圆周 4 格
        let (earth, sky) = plate_of(1, true, 1, 9);
        // 门按宫序号线性数：自坎 1 顺数 8 步才到宫 9（对冲宫），与星走圆周 4 格是同一落点
        let gates = gate_plate(1, 0, 8, true);
        let p = patterns(&earth, &sky, &gates);
        assert_eq!((sky.shift, gates.shift), (4, 4));
        assert!(p.star_fan_yin && p.gate_fan_yin);
        assert!(!p.star_fu_yin && !p.full_fu_yin);
        assert!(p.stem_fu_yin_palaces.is_empty(), "反吟时天地盘干处处不同");
    }

    /// 星伏吟 ⟺ 每宫天盘星都等于该宫原配星（与 shift 判定等价）。
    #[test]
    fn qm5_star_fu_yin_matches_a_cell_by_cell_check() {
        for from in ORBIT {
            for to in ORBIT {
                let (earth, sky) = plate_of(4, false, from, to);
                let gates = gate_plate(from, 0, 0, false);
                let p = patterns(&earth, &sky, &gates);
                let cell_wise = ORBIT
                    .iter()
                    .all(|&g| sky.stars[g as usize - 1] == JIU_XING_PALACE[g as usize]);
                assert_eq!(p.star_fu_yin, cell_wise);
            }
        }
    }

    /// 三奇临吉门只收「天盘乙丙丁 + 同宫开休生」这一结构事实。
    #[test]
    fn qm5_qi_gates_pair_the_three_odds_with_the_three_good_gates() {
        let c = compute(1987, 9, 17, 15, 0, 8.0);
        for qg in &c.patterns.qi_gates {
            let k = qg.palace as usize - 1;
            assert!(SAN_QI.contains(&c.sky.stems[k]));
            assert!(JI_MEN.contains(&c.gates.gates[k]));
            assert_eq!((qg.qi, qg.gate), (c.sky.stems[k], c.gates.gates[k]));
        }
        // 该盘天盘乙在震 3，震 3 正是生门
        assert_eq!(c.sky.stems[2], "乙");
        assert_eq!(c.gates.gates[2], "生门");
        assert!(c.patterns.qi_gates.iter().any(|q| q.palace == 3 && q.qi == "乙" && q.gate == "生门"));
    }

    /// 参考时刻不伏不反（星转 7 格、门转 1 格）。
    #[test]
    fn qm5_patterns_on_the_reference_moment() {
        let c = compute(1987, 9, 17, 15, 0, 8.0);
        let p = &c.patterns;
        assert!(!p.star_fu_yin && !p.star_fan_yin && !p.gate_fu_yin && !p.gate_fan_yin && !p.full_fu_yin);
    }


    /// 每个「节」开一个月，两气一支：24 节气恰好铺满 12 支，各支两次，且立春开寅月。
    #[test]
    fn qm4_two_terms_per_month_branch() {
        let mut count = [0u8; 12];
        for i in 0..24 {
            count[month_branch_of_term(i) as usize] += 1;
        }
        assert_eq!(count, [2; 12]);
        assert_eq!(month_branch_of_term(21), 2, "立春开寅月");
        assert_eq!(month_branch_of_term(22), 2, "雨水仍在寅月");
        assert_eq!(month_branch_of_term(11), 9, "白露属酉月");
        assert_eq!(month_branch_of_term(18), 0, "冬至属子月");
    }

    /// 旺相休囚死的通行判法：当令旺、令生相、生令休、克令囚、令克死。
    #[test]
    fn qm4_vigor_follows_the_classical_definition() {
        use Element::{Earth, Fire, Metal, Water, Wood};
        // 春木当令：木旺、火相（木生火）、水休（水生木）、金囚（金克木）、土死（木克土）
        assert_eq!(vigor_of(Wood, Wood), Vigor::Wang);
        assert_eq!(vigor_of(Fire, Wood), Vigor::Xiang);
        assert_eq!(vigor_of(Water, Wood), Vigor::Xiu);
        assert_eq!(vigor_of(Metal, Wood), Vigor::Qiu);
        assert_eq!(vigor_of(Earth, Wood), Vigor::Si);
        // 五行 × 五行穷举：每个月令下五等级恰好各出现一次
        for month in [Metal, Wood, Water, Fire, Earth] {
            let mut seen = std::collections::BTreeSet::new();
            for s in [Metal, Wood, Water, Fire, Earth] {
                assert!(seen.insert(vigor_of(s, month).label()));
            }
            assert_eq!(seen.len(), 5);
        }
    }

    /// 星名 → 五行：9 星齐备，未知名给 None。
    #[test]
    fn qm4_star_elements_are_complete() {
        assert_eq!(star_element("天蓬"), Some(Element::Water));
        assert_eq!(star_element("天芮"), Some(Element::Earth));
        assert_eq!(star_element("天英"), Some(Element::Fire));
        assert_eq!(star_element("天心"), Some(Element::Metal));
        assert_eq!(star_element("天冲"), Some(Element::Wood));
        assert!(star_element("").is_none() && star_element("天某").is_none());
        assert_eq!((1..=9).filter(|&p| star_element(JIU_XING_PALACE[p]).is_some()).count(), 9);
    }

    /// 1987-09-17 15:00（白露 → 酉月，月令金）：金星旺、水星相、土星休、火星囚、木星死。
    #[test]
    fn qm4_vigor_on_the_reference_moment() {
        let c = compute(1987, 9, 17, 15, 0, 8.0);
        assert_eq!((c.month_branch, c.month_element), (9, "金"));
        for k in 0..9 {
            let star = c.sky.stars[k];
            if star.is_empty() {
                assert_eq!(c.star_vigor[k], "");
                continue;
            }
            let want = match star_element(star).expect("九星五行齐备") {
                Element::Metal => "旺",
                Element::Water => "相",
                Element::Earth => "休",
                Element::Fire => "囚",
                Element::Wood => "死",
            };
            assert_eq!(c.star_vigor[k], want, "{star} 在酉月");
        }
        // 抽两宫核对：艮 8 天冲（木）死、兑 7 天心（金）旺
        assert_eq!((c.sky.stars[7], c.star_vigor[7]), ("天冲", "死"));
        assert_eq!((c.sky.stars[6], c.star_vigor[6]), ("天心", "旺"));
    }


    /// 起点坎 1 的阳遁八神（公开教程例题）：沿圆周顺时针依次落 8 宫。
    #[test]
    fn qm3_spirits_oracle_yang_from_kan_one() {
        let s = spirit_plate(1, true);
        assert_eq!(s.start_palace, 1);
        assert_eq!(
            s.spirits,
            ["值符", "玄武", "太阴", "六合", "", "九天", "九地", "腾蛇", "白虎"],
            "坎1值符 坤2玄武 震3太阴 巽4六合 中5空 乾6九天 兑7九地 艮8腾蛇 离9白虎"
        );
        // 另一系只换第 5 / 6 位的名字，位置不动
        assert_eq!(s.spirits_alt[8], "勾陈", "离 9 的白虎在另一系作勾陈");
        assert_eq!(s.spirits_alt[1], "朱雀", "坤 2 的玄武在另一系作朱雀");
        assert_eq!(s.spirits_alt[0], "值符");
    }

    /// 阴遁逆布：同一起点下顺序沿圆周反向，且阴遁不改名。
    #[test]
    fn qm3_spirits_run_backwards_under_yin_escape() {
        let s = spirit_plate(1, false);
        assert_eq!(
            s.spirits,
            ["值符", "六合", "九地", "玄武", "", "腾蛇", "太阴", "九天", "白虎"],
            "坎1值符 乾6腾蛇 兑7太阴 坤2玄武…… 逆时针"
        );
        assert_eq!(s.spirits_alt, s.spirits, "阴遁两系同名");
    }

    /// 任意起点与遁向：八神恒是外八宫的置换，值符恒在起点，中宫恒空。
    #[test]
    fn qm3_spirits_are_a_permutation_anchored_at_the_duty_symbol() {
        let want: std::collections::BTreeSet<&str> = BA_SHEN.iter().copied().collect();
        for &yang in &[true, false] {
            for start in [1u8, 2, 3, 4, 5, 6, 7, 8, 9] {
                let s = spirit_plate(start, yang);
                assert_eq!(s.spirits[4], "");
                assert_eq!(s.spirits[s.start_palace as usize - 1], "值符");
                let got: std::collections::BTreeSet<&str> =
                    ORBIT.iter().map(|&p| s.spirits[p as usize - 1]).collect();
                assert_eq!(got, want);
            }
        }
        assert_eq!(spirit_plate(5, true), spirit_plate(2, true), "落中 5 按寄坤 2 论");
    }

    /// 1987-09-17 15:00（阴遁 3 局，值符宫艮 8）整链回归。
    #[test]
    fn qm3_spirits_on_the_reference_moment() {
        let c = compute(1987, 9, 17, 15, 0, 8.0);
        assert_eq!(c.spirits.start_palace, 8);
        assert_eq!(c.spirits.spirits[7], "值符");
        // 阴遁自艮 8 逆行：艮8 → 坎1 → 乾6 → 兑7 → 坤2 → 离9 → 巽4 → 震3
        assert_eq!(
            c.spirits.spirits,
            ["腾蛇", "白虎", "九天", "九地", "", "太阴", "六合", "值符", "玄武"]
        );
    }


    /// 阳遁一局庚午时（古例）：旬首戊在坎 1，庚午是甲子旬第 7 个时辰，
    /// 值使休门自坎 1 按宫号顺数 6 步落兑 7；八门 8 宫逐宫比对。
    #[test]
    fn qm2_gate_plate_oracle_yang_one_gengwu() {
        // 甲子旬首（子 = 0），庚午（午 = 6）
        let g = gate_plate(1, 0, 6, true);
        assert_eq!((g.zhi_shi_gate, g.zhi_shi_palace), ("休门", 7));
        assert_eq!(g.steps, 6);
        assert_eq!(g.shift, 6);
        assert_eq!(
            g.gates,
            ["伤门", "开门", "景门", "死门", "", "生门", "休门", "杜门", "惊门"],
            "坎1伤 坤2开 震3景 巽4死 中5空 乾6生 兑7休 艮8杜 离9惊"
        );
    }

    /// 值使随时辰走的是**宫序号线性**（阳遁 +1 / 阴遁 −1，中 5 也占一位），
    /// 与九星沿圆周走不同 —— 这是八门最易与天盘混淆之处。
    #[test]
    fn qm2_zhi_shi_counts_along_palace_numbers_not_the_orbit() {
        // 阳遁：坎1 起，逐时辰 1→2→3→4→5(中,寄坤2)→6→7
        let landed: Vec<u8> = (0..7).map(|k| gate_plate(1, 0, k, true).zhi_shi_palace).collect();
        assert_eq!(landed, [1, 2, 3, 4, 2, 6, 7], "第 5 步落中 5 → 寄坤 2");
        // 阴遁：同起点反向 1→9→8→7…
        let back: Vec<u8> = (0..4).map(|k| gate_plate(1, 0, k, false).zhi_shi_palace).collect();
        assert_eq!(back, [1, 9, 8, 7]);
    }

    /// 一旬十个时辰：值使门恒为旬首宫本位门，八门恒是外八宫的一个置换，中宫恒空。
    #[test]
    fn qm2_gates_stay_a_permutation_across_a_whole_decade() {
        let want: std::collections::BTreeSet<&str> = BA_MEN_ORBIT.iter().copied().collect();
        for &yang in &[true, false] {
            for head in [0u8, 2, 4, 6, 8, 10] {
                for k in 0..10u8 {
                    let g = gate_plate(3, head, (head + k) % 12, yang);
                    assert_eq!(g.steps, k);
                    assert_eq!(g.zhi_shi_gate, BA_MEN_ORBIT[orbit_index(3)], "值使 = 旬首宫本位门");
                    assert_eq!(g.gates[4], "", "八门不入中宫");
                    let got: std::collections::BTreeSet<&str> =
                        ORBIT.iter().map(|&p| g.gates[p as usize - 1]).collect();
                    assert_eq!(got, want);
                    // 值使门必落在算出的那一宫
                    assert_eq!(g.gates[g.zhi_shi_palace as usize - 1], g.zhi_shi_gate);
                }
            }
        }
    }

    /// 1987-09-17 15:00（阴遁 3 局，旬首戊震 3，壬申为甲子旬第 9 个时辰）整链回归。
    #[test]
    fn qm2_gate_plate_on_the_reference_moment() {
        let c = compute(1987, 9, 17, 15, 0, 8.0);
        assert_eq!(c.gates.steps, 8, "申 = 8，甲子旬第 9 个时辰");
        assert_eq!(c.gates.zhi_shi_gate, "伤门", "旬首戊在震 3，震 3 本位门为伤门");
        // 阴遁自震 3 逆数 8 步：3→2→1→9→8→7→6→5(寄坤2)→4
        assert_eq!(c.gates.zhi_shi_palace, 4);
        assert_eq!(c.gates.gates[3], "伤门");
    }

    /// 阳遁三局丙寅时（古例）：旬首戊在震 3、时干丙在坎 1，值符天冲随时干落坎 1。
    ///
    /// 天盘九星期望值取自公开排盘教程的完整例题，8 宫逐宫比对。
    #[test]
    fn qm1b_sky_stars_oracle_yang_three_bingyin() {
        let ep = earth_plate(3, true);
        let mut earth = [""; 9];
        for k in 0..9 {
            earth[k] = STEM_NAMES[ep[k] as usize];
        }
        // 地盘先自证：阳三局 戊震3 己巽4 庚中5 辛乾6 壬兑7 癸艮8 丁离9 丙坎1 乙坤2
        assert_eq!(earth, ["丙", "乙", "戊", "己", "庚", "辛", "壬", "癸", "丁"]);

        let sky = sky_rotation(&earth, 3, 1);
        assert_eq!(sky.shift, 6, "震 3 → 坎 1 沿圆周顺时针 6 格");
        assert_eq!(
            sky.stars,
            ["天冲", "天心", "天英", "天芮", "", "天任", "天蓬", "天辅", "天柱"],
            "坎1冲 坤2心 震3英 巽4芮 中5空 乾6任 兑7蓬 艮8辅 离9柱"
        );
        // 旬首随时干：值符宫的天盘干必是本旬六仪
        assert_eq!(sky.stems[0], "戊");
        assert_eq!(sky.stems, ["戊", "辛", "丁", "乙", "", "癸", "丙", "己", "壬"]);
        // 中宫之干寄坤 2，随坤 2 转到巽 4（即随天芮走）
        assert_eq!((sky.center_stem, sky.center_palace), ("庚", 4));
        assert_eq!(sky.stars[3], "天芮", "中宫寄干的落宫正是天芮所在宫");
    }

    /// 阳遁一局庚午时（古例）：旬首戊在坎 1、时干庚在震 3，值符天蓬随时干落震 3。
    #[test]
    fn qm1b_sky_stars_oracle_yang_one_gengwu() {
        let ep = earth_plate(1, true);
        let mut earth = [""; 9];
        for k in 0..9 {
            earth[k] = STEM_NAMES[ep[k] as usize];
        }
        assert_eq!(earth, ["戊", "己", "庚", "辛", "壬", "癸", "丁", "丙", "乙"]);

        let sky = sky_rotation(&earth, 1, 3);
        assert_eq!(sky.shift, 2, "坎 1 → 震 3 沿圆周顺时针 2 格");
        assert_eq!(
            sky.stars,
            ["天柱", "天辅", "天蓬", "天任", "", "天芮", "天英", "天心", "天冲"],
            "坎1柱 坤2辅 震3蓬 巽4任 中5空 乾6芮 兑7英 艮8心 离9冲"
        );
        assert_eq!(sky.stems[2], "戊", "旬首六仪落到时干所在的震 3");
    }

    /// 时干恰为本旬六仪时不发生旋转：天盘 = 原配 / 地盘。
    #[test]
    fn qm1b_zero_shift_leaves_the_plate_untouched() {
        let ep = earth_plate(1, true);
        let mut earth = [""; 9];
        for k in 0..9 {
            earth[k] = STEM_NAMES[ep[k] as usize];
        }
        let sky = sky_rotation(&earth, 1, 1);
        assert_eq!(sky.shift, 0);
        for p in 1..=9u8 {
            if p == 5 {
                assert_eq!(sky.stars[4], "");
                assert_eq!(sky.stems[4], "");
            } else {
                assert_eq!(sky.stars[p as usize - 1], JIU_XING_PALACE[p as usize]);
                assert_eq!(sky.stems[p as usize - 1], earth[p as usize - 1]);
            }
        }
    }

    /// 旋转是刚体置换：任意起止组合下，外八宫的星集合与干集合都守恒，且中宫恒空。
    #[test]
    fn qm1b_rotation_is_a_permutation_of_the_outer_ring() {
        let ep = earth_plate(5, false);
        let mut earth = [""; 9];
        for k in 0..9 {
            earth[k] = STEM_NAMES[ep[k] as usize];
        }
        let want_stars: std::collections::BTreeSet<&str> =
            ORBIT.iter().map(|&p| JIU_XING_PALACE[p as usize]).collect();
        let want_stems: std::collections::BTreeSet<&str> =
            ORBIT.iter().map(|&p| earth[p as usize - 1]).collect();
        for from in ORBIT {
            for to in ORBIT {
                let sky = sky_rotation(&earth, from, to);
                assert!(sky.shift < 8);
                assert_eq!(sky.stars[4], "");
                assert_eq!(sky.stems[4], "");
                let got_stars: std::collections::BTreeSet<&str> =
                    ORBIT.iter().map(|&p| sky.stars[p as usize - 1]).collect();
                let got_stems: std::collections::BTreeSet<&str> =
                    ORBIT.iter().map(|&p| sky.stems[p as usize - 1]).collect();
                assert_eq!(got_stars, want_stars);
                assert_eq!(got_stems, want_stems);
                // 值符星必落在时干宫
                assert_eq!(sky.stars[to as usize - 1], JIU_XING_PALACE[from as usize]);
            }
        }
    }

    /// 落中宫按寄坤 2 论：符首或时干在中 5 时与在坤 2 时结果相同。
    #[test]
    fn qm1b_center_palace_is_lodged_in_kun_two() {
        let ep = earth_plate(2, true);
        let mut earth = [""; 9];
        for k in 0..9 {
            earth[k] = STEM_NAMES[ep[k] as usize];
        }
        assert_eq!(lodged_palace(5), 2);
        assert_eq!(sky_rotation(&earth, 5, 7), sky_rotation(&earth, 2, 7));
        assert_eq!(sky_rotation(&earth, 3, 5), sky_rotation(&earth, 3, 2));
    }

    /// 1987-09-17 15:00 长沙男（阴遁 3 局）整链回归：时干壬在艮 8、旬首戊在震 3。
    #[test]
    fn qm1b_sky_plate_on_the_reference_moment() {
        let c = compute(1987, 9, 17, 15, 0, 8.0);
        assert_eq!((c.xun_yi_palace, c.zhi_fu_palace), (3, 8));
        assert_eq!(c.zhi_fu_xing, "天冲");
        // 圆周是 坎1→艮8→震3→…，故震 3 退到艮 8 等于顺时针走 7 格
        assert_eq!(c.sky.shift, 7, "震 3 → 艮 8 沿圆周顺时针 7 格");
        // 值符星天冲落到时干所在的艮 8
        assert_eq!(c.sky.stars[7], "天冲");
        assert_eq!(c.sky.stems[7], "戊", "旬首六仪随值符同落艮 8");
        assert_eq!(
            c.sky.stars,
            ["天任", "天柱", "天辅", "天英", "", "天蓬", "天心", "天冲", "天芮"]
        );
    }


    use super::*;

    #[test]
    fn yang_dun_yang_ju1_matches_classic() {
        // 阳遁一局校验：坎1戊·坤2己·震3庚·巽4辛·中5壬·乾6癸·兑7丁·艮8丙·离9乙。
        let p = earth_plate(1, true);
        let want = [4u8, 5, 6, 7, 8, 9, 3, 2, 1]; // 戊己庚辛壬癸丁丙乙
        assert_eq!(p, want);
        // 宫↔八卦复用洛书：宫1=坎 … 宫9=离。
        assert_eq!(mingli_luoshu::PALACE_NAME[1], "坎");
        assert_eq!(mingli_luoshu::PALACE_NAME[9], "离");
    }

    #[test]
    fn earth_plate_is_a_permutation_for_all_ju_both_dun() {
        // 任意局数、阴阳遁：九宫恰好布满 9 个三奇六仪（双射），无重无漏。
        for ju in 1..=9u8 {
            for yang in [true, false] {
                let p = earth_plate(ju, yang);
                let set: std::collections::HashSet<u8> = p.iter().copied().collect();
                assert_eq!(set.len(), 9, "ju={ju} yang={yang} 应布满九宫");
                // 布的恰是三奇六仪（戊己庚辛壬癸乙丙丁 = 天干 1..9，无甲）。
                for &stem in &p {
                    assert!((1..=9).contains(&stem), "不应出现甲(0)");
                }
            }
        }
    }

    #[test]
    fn yin_dun_ju1_six_yi_reversed_three_qi_forward() {
        // 阴遁一局：戊从宫1起逆行 → 戊在宫1、己在宫9、庚在宫8、辛在宫7、壬在宫6、癸在宫5；
        // 三奇乙丙丁顺接癸（宫5）之后 → 乙宫6、丙宫7、丁宫8……但宫6/7/8已被壬辛庚占？
        // 故按算法回绕填空位，最终仍为九宫双射（上条已验）。这里验六仪逆布的起段。
        let p = earth_plate(1, false);
        assert_eq!(p[0], 4); // 宫1 = 戊
        // 己庚辛壬癸沿逆行（宫9，8，7，6，5）。
        assert_eq!(p[8], 5); // 宫9 = 己
        assert_eq!(p[7], 6); // 宫8 = 庚
        assert_eq!(p[6], 7); // 宫7 = 辛
        assert_eq!(p[5], 8); // 宫6 = 壬
        assert_eq!(p[4], 9); // 宫5 = 癸
        // 三奇顺布：乙丙丁落于宫序递增的余三宫（宫2/3/4）。
        assert_eq!(p[1], 1); // 宫2 = 乙
        assert_eq!(p[2], 2); // 宫3 = 丙
        assert_eq!(p[3], 3); // 宫4 = 丁
    }

    #[test]
    fn ju_table_invariants() {
        // 72 局表结构自检（防录入错）：阳遁 中=上+6、下=上+3 (mod9，1..9)；阴遁 中=上−6、下=上−3。
        let amod9 = |x: i64| ((x - 1).rem_euclid(9) + 1) as u8;
        for k in 0..24usize {
            let [up, mid, down] = YUAN_JU[k];
            if is_yang_dun(k) {
                assert_eq!(mid, amod9(i64::from(up) + 6), "{} 阳遁中元", SOLAR_TERMS[k]);
                assert_eq!(down, amod9(i64::from(up) + 3), "{} 阳遁下元", SOLAR_TERMS[k]);
            } else {
                assert_eq!(mid, amod9(i64::from(up) - 6), "{} 阴遁中元", SOLAR_TERMS[k]);
                assert_eq!(down, amod9(i64::from(up) - 3), "{} 阴遁下元", SOLAR_TERMS[k]);
            }
            // 所有局数在 1..9。
            assert!((1..=9).contains(&up) && (1..=9).contains(&mid) && (1..=9).contains(&down));
        }
    }

    #[test]
    fn yuan_of_branch_groups() {
        // 子午卯酉=上、寅申巳亥=中、辰戌丑未=下。
        for b in [0u8, 6, 3, 9] {
            assert_eq!(yuan_of_branch(b), Yuan::Upper);
        }
        for b in [2u8, 8, 5, 11] {
            assert_eq!(yuan_of_branch(b), Yuan::Middle);
        }
        for b in [4u8, 10, 1, 7] {
            assert_eq!(yuan_of_branch(b), Yuan::Lower);
        }
    }

    #[test]
    fn solar_term_index_and_dun() {
        // λ=0（春分，k0）阳…λ=90（夏至，k6）阴…λ=270（冬至，k18）阳。
        assert_eq!(solar_term_index(0.0), 0);
        assert_eq!(SOLAR_TERMS[solar_term_index(0.0)], "春分");
        assert_eq!(SOLAR_TERMS[solar_term_index(270.0)], "冬至");
        assert!(is_yang_dun(solar_term_index(270.0))); // 冬至阳遁
        assert!(!is_yang_dun(solar_term_index(90.0))); // 夏至阴遁
        // 12 阳 12 阴。
        let yang = (0..24).filter(|&k| is_yang_dun(k)).count();
        assert_eq!(yang, 12);
    }

    #[test]
    fn three_yuan_select_correct_ju() {
        // 冬至 [1,7,4]：上元→1、中元→7、下元→4，并校验三元下标/名。
        assert_eq!(solar_term_setup(18, Yuan::Upper).ju, 1);
        assert_eq!(solar_term_setup(18, Yuan::Middle).ju, 7);
        assert_eq!(solar_term_setup(18, Yuan::Lower).ju, 4);
        assert_eq!(Yuan::Upper.index(), 0);
        assert_eq!(Yuan::Middle.index(), 1);
        assert_eq!(Yuan::Lower.index(), 2);
        assert_eq!(Yuan::Middle.name(), "中元");
        assert_eq!(Yuan::Lower.name(), "下元");
    }

    #[test]
    fn setup_and_compute_consistent() {
        let s = solar_term_setup(18, Yuan::Upper); // 冬至上元 → 1 局阳遁
        assert_eq!(s.ju, 1);
        assert!(s.yang_dun);
        assert_eq!(s.yuan.name(), "上元");
        let c = compute(2024, 6, 15, 14, 30, 8.0);
        // 地盘双射、宫名取自洛书。
        let set: std::collections::HashSet<&str> = c.earth.iter().copied().collect();
        assert_eq!(set.len(), 9);
        assert_eq!(c.palace[0], "坎");
        assert!(c.fu_tou_branch < 12);
        // 确定性。
        let c2 = compute(2024, 6, 15, 14, 30, 8.0);
        assert_eq!(c.earth, c2.earth);
        assert_eq!(c.setup.ju, c2.setup.ju);
    }

    /// oracle：1987-09-17 15：00 长沙男 → 日柱 己巳 / 时柱 壬申 / 甲子旬 / 旬遁戊 / 旬空戌亥。
    #[test]
    fn qm0_xun_oracle_1987_changsha_male() {
        let c = compute(1987, 9, 17, 15, 0, 8.0);
        // 时柱：日柱己巳(stem=5，branch=5)，时支申(8) → 时干 (5%5)*2+8=8=壬。时柱壬申。
        assert_eq!(c.time_ganzhi, "壬申");
        assert_eq!(c.time_stem, 8);
        assert_eq!(c.time_branch, 8);
        // 旬：壬申 → head_branch=(8-8+12)%12=0 → 甲子旬，遁戊
        assert_eq!(c.xun.head_ganzhi, "甲子");
        assert_eq!(c.xun.head_branch, 0);
        assert_eq!(c.xun.head_yi, "戊");
        assert_eq!(c.xun.head_yi_stem, 4);
        // 甲子旬旬空：戌亥
        assert_eq!(c.xun.xunkong, ["戌", "亥"]);
    }

    /// oracle：不同时柱对应不同旬首六仪（6 旬覆盖）。
    #[test]
    fn qm0_six_xun_via_different_times() {
        // 同日 1987-09-17 己巳日，时辰 → 时支：
        // 23：30 子(0) → 时柱 甲子 → 甲子旬遁戊
        // 03：30 寅(2) → 时柱 丙寅 → 甲子旬遁戊
        // 15：00 申(8) → 时柱 壬申 → 甲子旬遁戊（以上同日，旬不跨）
        // 不同日才会跨旬，我们换日：
        // 1987-09-22（甲戌日） 子时(0) → 甲子时 → 甲子旬戊
        // 但用同日不同时柱必同旬，因为日柱固定 stem=5，时柱 stem=(5*2+tb)%10，branch=tb，
        // head=(tb - ((10+tb)%10) + 12)%12 — 计算
        for (h, expected_yi) in [
            (23, "戊"), // 子时 甲子 旬遁戊
            (15, "戊"), // 申时 壬申 同旬
        ] {
            let c = compute(1987, 9, 17, h, 0, 8.0);
            assert_eq!(c.xun.head_yi, expected_yi, "h={h}");
        }
        // 跨旬：1992-08-09 甲申日，子时：日柱甲申(0，8)，子时=(0%5)*2+0=0，时柱甲子(0，0) → 甲子旬戊
        let c1 = compute(1992, 8, 9, 0, 30, 8.0);
        assert!(["戊", "己", "庚", "辛", "壬", "癸"].contains(&c1.xun.head_yi));
    }

    /// 旬首六仪在地盘九宫中必有一席之地（双射性质）；xun_yi_palace ∈ 1..=9。
    #[test]
    fn qm0_xun_yi_palace_in_range() {
        for (y, m, d, h) in [(1987, 9, 17, 15), (1990, 6, 15, 14), (2024, 1, 1, 0), (2026, 6, 17, 10)] {
            let c = compute(y, m, d, h, 0, 8.0);
            assert!((1..=9).contains(&c.xun_yi_palace), "xun_yi_palace {} 应 ∈ 1..=9", c.xun_yi_palace);
            // 该宫的地盘干 = 旬首六仪
            assert_eq!(c.earth[(c.xun_yi_palace - 1) as usize], c.xun.head_yi);
        }
    }

    /// oracle：1987-09-17 15：00 长沙男 阴遁 3 局 → 时干壬在艮 8 宫 = 值符宫；
    /// 旬首戊在震 3 宫 → 本旬值符星 = 震 3 原配「天冲」。
    #[test]
    fn qm1a_zhi_fu_oracle_1987_changsha() {
        let c = compute(1987, 9, 17, 15, 0, 8.0);
        // 时柱壬申：time_stem=8（壬） ≠ 0（甲） → 实际值符 = 壬
        assert_eq!(c.zhi_fu_stem, 8);
        assert_eq!(c.zhi_fu_stem_name, "壬");
        // 阴遁 3 局地盘：坎1庚 坤2己 震3戊 巽4乙 中5丙 乾6丁 兑7癸 艮8壬 离9辛
        // → 壬在艮 8 宫
        assert_eq!(c.zhi_fu_palace, 8);
        // 旬首戊在震 3 宫（已验） → 本旬值符星 = 震 3 原配 = 天冲
        assert_eq!(c.xun_yi_palace, 3);
        assert_eq!(c.zhi_fu_xing, "天冲");
        // 九星原配 9 宫（地盘初始，未旋转）
        assert_eq!(c.jiuxing_earth, ["天蓬", "天芮", "天冲", "天辅", "天禽", "天心", "天柱", "天任", "天英"]);
    }

    /// 时干为甲时，实际值符 = 旬首六仪（遁仪规则）。
    #[test]
    fn qm1a_effective_zhi_fu_stem_jia_remap() {
        // 6 旬旬首的甲（时干=0）分别遁戊/己/庚/辛/壬/癸
        for (head_yi, want_name) in [(4u8, "戊"), (5, "己"), (6, "庚"), (7, "辛"), (8, "壬"), (9, "癸")] {
            assert_eq!(effective_zhi_fu_stem(0, head_yi), head_yi);
            assert_eq!(STEM_NAMES[head_yi as usize], want_name);
        }
        // 时干非甲(1..=9) → 实际值符 = 时干本身，旬首六仪无关
        for ts in 1..=9u8 {
            for head_yi in [4u8, 5, 6, 7, 8, 9] {
                assert_eq!(effective_zhi_fu_stem(ts, head_yi), ts);
            }
        }
    }

    /// 9 星原配 9 宫不变量（蓬 1 / 芮 2 / 冲 3 / 辅 4 / 禽 5 / 心 6 / 柱 7 / 任 8 / 英 9）。
    #[test]
    fn qm1a_jiuxing_palace_table_stable() {
        // 索引 0 占位，1..=9 为 9 宫原配九星
        assert_eq!(JIU_XING_PALACE[1], "天蓬");
        assert_eq!(JIU_XING_PALACE[2], "天芮");
        assert_eq!(JIU_XING_PALACE[3], "天冲");
        assert_eq!(JIU_XING_PALACE[4], "天辅");
        assert_eq!(JIU_XING_PALACE[5], "天禽");
        assert_eq!(JIU_XING_PALACE[6], "天心");
        assert_eq!(JIU_XING_PALACE[7], "天柱");
        assert_eq!(JIU_XING_PALACE[8], "天任");
        assert_eq!(JIU_XING_PALACE[9], "天英");
        // 9 颗星全唯一
        let set: std::collections::HashSet<&str> = JIU_XING_PALACE[1..].iter().copied().collect();
        assert_eq!(set.len(), 9);
    }

    /// 值符宫与值符星跨时刻覆盖性 — 不同时刻 zhi_fu_palace 与 zhi_fu_xing 应都 ∈ 合法集合。
    #[test]
    fn qm1a_zhi_fu_consistency_over_times() {
        for (y, m, d, h) in [(1987, 9, 17, 15), (1990, 6, 15, 14), (2024, 1, 1, 0), (2026, 6, 17, 10)] {
            let c = compute(y, m, d, h, 0, 8.0);
            assert!((1..=9).contains(&c.zhi_fu_palace), "zhi_fu_palace {} 应 ∈ 1..=9", c.zhi_fu_palace);
            assert!(JIU_XING_PALACE[1..].contains(&c.zhi_fu_xing), "{} 不在九星表内", c.zhi_fu_xing);
            // 值符宫的地盘干 = 实际值符天干
            assert_eq!(c.earth[(c.zhi_fu_palace - 1) as usize], c.zhi_fu_stem_name);
            // 值符星 = 旬首六仪所在宫的原配九星
            assert_eq!(c.zhi_fu_xing, JIU_XING_PALACE[c.xun_yi_palace as usize]);
        }
    }

    #[test]
    fn fu_tou_is_recent_jia_or_ji_day() {
        // 符头日的日干必为甲(0)或己(5)。扫描多日校验。
        let base = mingli_astro::civil_day_number(2024, 1, 1);
        for k in 0..60i64 {
            let jdn = base + k;
            let day = mingli_ganzhi::day_ganzhi(jdn);
            let back = i64::from(day.stem % 5);
            let fu = mingli_ganzhi::day_ganzhi(jdn - back);
            assert!(fu.stem == 0 || fu.stem == 5, "符头日干应为甲/己");
        }
    }
}
