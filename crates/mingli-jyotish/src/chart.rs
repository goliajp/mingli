//! 起盘：把岁差、九曜、分段与大运拼成一份完整命盘。

use super::*;

/// 一张 Jyotish（印度占星）排盘结果。
#[derive(Debug, Clone, Serialize)]
pub struct JyotishChart {
    /// Ayanamsa 流派 id。
    pub ayanamsa_id: &'static str,
    /// Ayanamsa 当时数值（度）。
    pub ayanamsa_deg: f64,
    /// 9 行星条目(navagraha)。
    pub grahas: Vec<GrahaPosition>,
    /// 月亮所在 nakshatra 的 Vimshottari mahadasha 主星(birth dasha)。
    pub birth_dasha_lord: &'static str,
    /// Vimshottari mahadasha 完整 timeline（9 段共 120 年，从 birth dasha 起）。
    pub mahadashas: Vec<Mahadasha>,
    /// Lagna（上升点）的恒星黄经（度，若 [`GeoLocation`] 给出）。
    pub lagna_lon: Option<f64>,
    /// Lagna 所在 rasi 索引（若计算）。
    pub lagna_rasi: Option<usize>,
    /// Lagna 所在 rasi 名（若计算）。
    pub lagna_rasi_name: Option<&'static str>,
    /// Lagna 所在 navamsa rasi 索引（若计算）。
    pub lagna_navamsa: Option<usize>,
    /// Lagna 所在 navamsa rasi 名（若计算）。
    pub lagna_navamsa_name: Option<&'static str>,
}

/// 在共享上下文 [`Moment`] 上排印度占星盘。`geo` 给定时算 Lagna（上升）。
#[must_use]
pub fn compute_at(m: &Moment, geo: Option<GeoLocation>, mode: Ayanamsa) -> JyotishChart {
    let jde = m.jde;
    let ay = ayanamsa(jde, mode);
    let grahas: Vec<GrahaPosition> = Graha::all().iter().map(|&g| graha_position(g, jde, ay)).collect();
    // 月亮 nakshatra 主星 = 命主 birth mahadasha 主星（Vimshottari 起算锚）。
    let moon = &grahas[1];
    let birth_dasha_lord = moon.nakshatra_lord;

    let (lagna_lon, lagna_rasi, lagna_rasi_name, lagna_navamsa, lagna_navamsa_name) = if let Some(g) = geo {
        // asc_mc 接 RAMC（本地恒星时）。本地 RAMC = GMST + 经度（东正）。
        let ramc = (m.sidereal_time + g.longitude).rem_euclid(360.0);
        let (asc_trop, _) = asc_mc(ramc, m.obliquity, g.latitude);
        let lagna = (asc_trop - ay).rem_euclid(360.0);
        let r = rasi_of(lagna);
        let nv = navamsa_of(lagna);
        (Some(lagna), Some(r), Some(RASI_NAMES[r]), Some(nv), Some(RASI_NAMES[nv]))
    } else {
        (None, None, None, None, None)
    };

    let mahadashas = vimshottari_timeline(moon.sidereal_lon, m.jd_ut);

    JyotishChart {
        ayanamsa_id: mode.id(),
        ayanamsa_deg: ay,
        grahas,
        birth_dasha_lord,
        mahadashas,
        lagna_lon,
        lagna_rasi,
        lagna_rasi_name,
        lagna_navamsa,
        lagna_navamsa_name,
    }
}

/// 由本地民用时刻起的入口参数集合。比平铺八个 `compute()` 形参更清晰，也避免 clippy 抱怨。
#[derive(Debug, Clone, Copy)]
pub struct BirthInput {
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
    /// 时区偏移小时（中国 +8、印度 +5.5）。
    pub tz: f64,
}

/// 由本地民用时刻排盘（独立入口，构造 [`Moment`]）。
#[must_use]
pub fn compute(b: BirthInput, geo: Option<GeoLocation>, mode: Ayanamsa) -> JyotishChart {
    compute_at(&Moment::new(b.year, b.month, b.day, b.hour, b.minute, b.tz), geo, mode)
}
