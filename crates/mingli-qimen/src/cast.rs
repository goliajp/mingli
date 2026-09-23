//! 起局：把节气、干支、四盘拼成一次完整排盘。

use super::*;

/// 时柱旬：奇门时家盘以**时柱**为核心，旬首决定值符所遁之仪、旬空标失时之支。
///
/// **算法**（沿用 [`mingli_ganzhi::xun_head_branch`] / [`mingli_ganzhi::xun_yi`] / [`mingli_ganzhi::xunkong`]）：
/// - 旬首支 = `(time_branch − time_stem + 12) mod 12`，定时柱所在 6 旬之一。
/// - 旬首六仪 = 旬首甲所遁的仪干：甲子→戊 / 甲戌→己 / 甲申→庚 / 甲午→辛 / 甲辰→壬 / 甲寅→癸。
/// - 旬空 2 支 = 旬首支 +10 / +11 mod 12（本旬 10 干配不上的两位地支）。
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
pub(crate) fn time_branch(hour: u32, minute: u32) -> u8 {
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
pub(crate) fn effective_zhi_fu_stem(time_stem: u8, head_yi_stem: u8) -> u8 {
    if time_stem == 0 { head_yi_stem } else { time_stem }
}

/// 某天干在地盘九宫所在的宫(1..=9)。
///
/// 地盘恰好摆着三奇六仪九个干（甲遁不上盘），而值符干经 [`effective_zhi_fu_stem`]
/// 之后必是这九个之一，故必然找得到。找不到就是地盘坏了——与其把 0 当宫号传下去
/// 污染天盘旋转与八神布局，不如就地炸掉。
fn earth_position_of_stem(earth: &[&str; 9], stem_name: &str) -> u8 {
    for (k, &s) in earth.iter().enumerate() {
        if s == stem_name {
            return (k + 1) as u8;
        }
    }
    unreachable!("值符干 {stem_name} 应在地盘九宫之一")
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
