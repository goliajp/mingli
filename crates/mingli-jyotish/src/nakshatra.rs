//! 二十七宿（nakshatra）· 十二宫（rāśi）· 九分盘（navāṃśa）：恒星黄经的分段。

/// 27 nakshatra 名（罗马转写，固定一套常用拼写）。
/// 起 Ashwini@恒星 0°（白羊 0°），每 13°20'。
pub const NAKSHATRA_NAMES: [&str; 27] = [
    "Ashwini", "Bharani", "Krittika", "Rohini", "Mrigashira", "Ardra",
    "Punarvasu", "Pushya", "Ashlesha", "Magha", "Purva Phalguni", "Uttara Phalguni",
    "Hasta", "Chitra", "Swati", "Vishakha", "Anuradha", "Jyeshtha",
    "Mula", "Purva Ashadha", "Uttara Ashadha", "Shravana", "Dhanishtha", "Shatabhisha",
    "Purva Bhadrapada", "Uttara Bhadrapada", "Revati",
];

/// 12 rasi（印度占星星座，与西洋占星 12 sign 一一对应）。索引 = 白羊 0..双鱼 11。
pub const RASI_NAMES: [&str; 12] = [
    "Mesha", "Vrishabha", "Mithuna", "Karka", "Simha", "Kanya",
    "Tula", "Vrishchika", "Dhanu", "Makara", "Kumbha", "Meena",
];

/// 从恒星黄经（度，`[0, 360)`）取所在 nakshatra 索引(0..27)。每个 nakshatra 占 360/27 度。
#[must_use]
pub fn nakshatra_of(sidereal_lon: f64) -> usize {
    let span = 360.0 / 27.0;
    (sidereal_lon.rem_euclid(360.0) / span).floor() as usize % 27
}

/// 从恒星黄经取所在 rasi 索引（0..12，白羊=0）。每 rasi 占 30°。
#[must_use]
pub fn rasi_of(sidereal_lon: f64) -> usize {
    (sidereal_lon.rem_euclid(360.0) / 30.0).floor() as usize % 12
}

/// D-9 navamsa 分盘：从恒星黄经取所在 navamsa rasi 索引(0..12)。
///
/// 公式：`navamsa = floor(lon × 12/30 × 3) mod 12 = floor(lon × 0.3) mod 12`。
/// 即每 navamsa 跨 360/108 = 10/3°（每 rasi 9 个 navamsa）。
///
/// 验证三类(Movable/Fixed/Dual)起算 sign（主流印度占星教材）：
/// - Movable rasi (Aries/Cancer/Libra/Capricorn， idx 0/3/6/9)：起本 sign。
///   如 Aries 0° → Aries(0)、Cancer 0° → Cancer(3)。
/// - Fixed rasi (Taurus/Leo/Scorpio/Aquarius， idx 1/4/7/10)：起本 sign + 8 mod 12。
///   如 Taurus 0° → Capricorn(9)、Leo 0° → Aries(0)、Scorpio 0° → Cancer(3)。
/// - Dual rasi (Gemini/Virgo/Sagittarius/Pisces， idx 2/5/8/11)：起本 sign + 4 mod 12。
///   如 Gemini 0° → Libra(6)、Virgo 0° → Capricorn(9)。
#[must_use]
pub fn navamsa_of(sidereal_lon: f64) -> usize {
    (sidereal_lon.rem_euclid(360.0) * 0.3).floor() as usize % 12
}
