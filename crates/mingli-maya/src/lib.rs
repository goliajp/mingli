//! L3 叶（A 族 / 确定性）：玛雅历法的三套互锁计数。
//!
//! 玛雅把一个绝对日序（这里取民用儒略日 JDN）同时投影到三套循环上——这是「中国剩余定理」
//! 在异文化里的独立再现，也是 [`mingli_core::cyclic`] 的天然范例：
//!
//! - **Tzolkʼin**：`260 = 13 × 20`，由「13 数轮 × 20 日名轮」并进。`gcd(13,20)=1` ⇒ 组合周期 =
//!   `lcm(13,20)=260`，每 260 天 （数， 名） 不重复（[`tzolkin`]）。
//! - **Haab**：`365 = 18×20 + 5`（18 个 20 日月 + Wayeb 5 日），太阳年近似（[`haab`]）。
//! - **Long Count**：混合进制 `kin·winal·tun·katun·baktun`（20，18，20，20），线性日计数（[`long_count`]）。
//!
//! 历元用学界主流 **GMT correlation = 584283**：JDN 584283 = Long Count `0.0.0.0.0` =
//! Tzolkʼin `4 Ahau` = Haab `8 Cumku`（经放射性碳 + 天文 + 现存活历三证）。
//! 名表用 16 世纪 Yucatec（Landa）拼写。
//!
//! 语域注：本 crate 只做历日换算（确定性数学），不涉玛雅占卜释义。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "全部相位经 rem_euclid 落在 0..260 / 0..365 / 0..20 等小范围，窄化到 u8/usize 受控安全"
)]

mod engine;
pub use engine::MayaEngine;

use mingli_astro::Moment;
use serde::Serialize;

/// GMT correlation：JDN 584283 = Long Count `0.0.0.0.0`（学界主流）。
pub const GMT_CORRELATION: i64 = 584_283;
/// Thompson–Lounsbury 变体（落后 2 天），仅供参考，本 crate 默认用 [`GMT_CORRELATION`]。
pub const THOMPSON_LOUNSBURY: i64 = 584_285;

/// Tzolkʼin 20 日名（Yucatec / Landa 拼写），下标 0..20。历元 `0.0.0.0.0` 落 `Ahau`（下标 19）。
pub const TZOLKIN_DAYS: [&str; 20] = [
    "Imix", "Ik", "Akbal", "Kan", "Chicchan", "Cimi", "Manik", "Lamat", "Muluc", "Oc", "Chuen",
    "Eb", "Ben", "Ix", "Men", "Cib", "Caban", "Etznab", "Cauac", "Ahau",
];

/// Haab 19 个月名（18 个 20 日月 + Wayeb 5 日），下标 0..19。历元落 `Cumku`（下标 17）第 8 日。
pub const HAAB_MONTHS: [&str; 19] = [
    "Pop", "Wo", "Sip", "Sotz", "Sek", "Xul", "Yaxkin", "Mol", "Chen", "Yax", "Sak", "Keh", "Mak",
    "Kankin", "Muwan", "Pax", "Kayab", "Cumku", "Wayeb",
];

/// Long Count 各位的累积天数：kin， winal， tun， katun， baktun。
/// 注意混合进制：winal=20 kin，但 tun=18 winal。
pub const PLACE_DAYS: [i64; 5] = [1, 20, 360, 7_200, 144_000];

/// Tzolkʼin（13 数 × 20 名）。返回 （数 1..=13， 日名下标 0..20）。
///
/// 历元 JDN 584283 = `4 Ahau`：数 4、名下标 19。
#[must_use]
pub fn tzolkin(jdn: i64) -> (u8, usize) {
    let d = jdn - GMT_CORRELATION;
    let number = ((d + 3).rem_euclid(13) + 1) as u8; // d=0 → 4
    let name = (d + 19).rem_euclid(20) as usize; // d=0 → Ahau(19)
    (number, name)
}

/// Tzolkʼin 在 260 周期内的相位 `0..260`（用 [`mingli_core::cyclic::crt_combine`] 由两轮合成）。
///
/// 这正是「`Z₁₃ × Z₂₀ ≅ Z₂₆₀`（因 gcd=1）」的构造性体现。
#[must_use]
pub fn tzolkin_round(jdn: i64) -> i64 {
    let d = jdn - GMT_CORRELATION;
    let p13 = (d + 3).rem_euclid(13); // 数轮 0-based
    let p20 = (d + 19).rem_euclid(20); // 名轮 0-based
    mingli_core::cyclic::crt_combine(&[(p13, 13), (p20, 20)]).unwrap_or(0)
}

/// Haab（365）。返回 （月内日， 月名下标 0..19）。普通月日 0..=19，Wayeb 日 0..=4。
///
/// 历元 JDN 584283 = `8 Cumku`：月名下标 17、月内日 8（年内序 17×20+8=348）。
#[must_use]
pub fn haab(jdn: i64) -> (u8, usize) {
    let doy = haab_day_of_year(jdn);
    if doy < 360 {
        ((doy % 20) as u8, (doy / 20) as usize)
    } else {
        ((doy - 360) as u8, 18) // Wayeb
    }
}

/// Haab 年内序 `0..365`（0 = `0 Pop`）。
#[must_use]
pub fn haab_day_of_year(jdn: i64) -> i64 {
    let d = jdn - GMT_CORRELATION;
    (348 + d).rem_euclid(365) // 历元 8 Cumku = 348
}

/// Long Count：返回 `[baktun, katun, tun, winal, kin]`（高位在前）。
///
/// 历元 `0.0.0.0.0`；混合进制 kin∈0..20， winal∈0..18， tun∈0..20， katun∈0..20。
/// 历元之前（JDN < 584283）的日子返回负 baktun（线性外推，不截断）。
#[must_use]
pub fn long_count(jdn: i64) -> [i64; 5] {
    let mut d = jdn - GMT_CORRELATION;
    let baktun = d.div_euclid(144_000);
    d = d.rem_euclid(144_000);
    let katun = d / 7_200;
    d %= 7_200;
    let tun = d / 360;
    d %= 360;
    let winal = d / 20;
    let kin = d % 20;
    [baktun, katun, tun, winal, kin]
}

/// 一次玛雅历日换算的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Cast {
    /// 民用儒略日数。
    pub jdn: i64,
    /// Tzolkʼin 数 1..=13。
    pub tzolkin_number: u8,
    /// Tzolkʼin 日名。
    pub tzolkin_name: &'static str,
    /// Tzolkʼin 260 周期内相位 0..260。
    pub tzolkin_round: i64,
    /// Haab 月内日（普通月 0..20，Wayeb 0..5）。
    pub haab_day: u8,
    /// Haab 月名。
    pub haab_month: &'static str,
    /// Long Count `[baktun, katun, tun, winal, kin]`。
    pub long_count: [i64; 5],
}

/// 在共享上下文 [`Moment`] 上做玛雅历日换算（取其民用日序 JDN）。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    compute_from_jdn(m.civil_day)
}

/// 由民用儒略日数直接换算（核心入口）。
#[must_use]
pub fn compute_from_jdn(jdn: i64) -> Cast {
    let (tn, ti) = tzolkin(jdn);
    let (hd, hi) = haab(jdn);
    Cast {
        jdn,
        tzolkin_number: tn,
        tzolkin_name: TZOLKIN_DAYS[ti],
        tzolkin_round: tzolkin_round(jdn),
        haab_day: hd,
        haab_month: HAAB_MONTHS[hi],
        long_count: long_count(jdn),
    }
}

/// 由本地民用时刻换算（独立入口，构造 [`Moment`] 取其 JDN）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, 12, 0, tz))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_tables_well_formed() {
        assert_eq!(TZOLKIN_DAYS.len(), 20);
        assert_eq!(HAAB_MONTHS.len(), 19);
        assert_eq!(TZOLKIN_DAYS[19], "Ahau");
        assert_eq!(HAAB_MONTHS[17], "Cumku");
        assert_eq!(HAAB_MONTHS[18], "Wayeb");
    }

    #[test]
    fn era_oracle_0_0_0_0_0() {
        // JDN 584283 = 0.0.0.0.0 = 4 Ahau 8 Cumku（GMT correlation，多源 + 算术复核）。
        let c = compute_from_jdn(GMT_CORRELATION);
        assert_eq!(c.long_count, [0, 0, 0, 0, 0]);
        assert_eq!((c.tzolkin_number, c.tzolkin_name), (4, "Ahau"));
        assert_eq!((c.haab_day, c.haab_month), (8, "Cumku"));
    }

    #[test]
    fn end_of_13th_baktun_2012() {
        // 2012-12-21 = JDN 2456283 = 13.0.0.0.0 = 4 Ahau 3 Kankin。
        // 同时校验共享层民用日序与标准 JDN 一致。
        assert_eq!(mingli_astro::civil_day_number(2012, 12, 21), 2_456_283);
        let c = compute_from_jdn(2_456_283);
        assert_eq!(c.long_count, [13, 0, 0, 0, 0]);
        assert_eq!((c.tzolkin_number, c.tzolkin_name), (4, "Ahau"));
        assert_eq!((c.haab_day, c.haab_month), (3, "Kankin"));
        // 大循环算术自洽：584283 + 13×144000 = 2456283。
        assert_eq!(GMT_CORRELATION + 13 * PLACE_DAYS[4], 2_456_283);
    }

    #[test]
    fn compute_from_moment_matches_jdn() {
        let c = compute(2012, 12, 21, 0.0);
        assert_eq!(c.long_count, [13, 0, 0, 0, 0]);
        assert_eq!(c.tzolkin_name, "Ahau");
    }

    #[test]
    fn tzolkin_is_a_260_crt() {
        // （数，名） 在 260 天内两两不同；CRT 合成的环位逐日 +1 且在 0..260 内唯一覆盖。
        use std::collections::HashSet;
        let mut pairs = HashSet::new();
        let mut rounds = HashSet::new();
        for k in 0..260i64 {
            let jdn = GMT_CORRELATION + k;
            let (n, name) = tzolkin(jdn);
            assert!(pairs.insert((n, name)), "260 内 （数，名） 应唯一，k={k}");
            let r = tzolkin_round(jdn);
            assert!((0..260).contains(&r));
            assert!(rounds.insert(r), "260 内 CRT 环位应唯一，k={k}");
            // 逐日 +1（mod 260）：CRT 合成与两轮同步推进自洽。
            assert_eq!(tzolkin_round(jdn + 1), (r + 1).rem_euclid(260));
        }
        assert_eq!(rounds.len(), 260); // 满覆盖 0..260
        // 第 260 天回到起点（数，名 与 CRT 环位皆复位）。
        assert_eq!(tzolkin(GMT_CORRELATION + 260), tzolkin(GMT_CORRELATION));
        assert_eq!(tzolkin_round(GMT_CORRELATION + 260), tzolkin_round(GMT_CORRELATION));
        // 历元 4 Ahau 的 CRT 环位 = crt(3，19)。
        assert_eq!(
            tzolkin_round(GMT_CORRELATION),
            mingli_core::cyclic::crt_combine(&[(3, 13), (19, 20)]).unwrap()
        );
        // 组合周期 = lcm(13，20) = 260。
        assert_eq!(mingli_core::cyclic::cycle_period(&[13, 20]), 260);
    }

    #[test]
    fn haab_wraps_365_and_covers_wayeb() {
        // 年内序遍历 0..365：月名下标恒在 0..19，Wayeb 恰 5 天。
        let mut wayeb_days = 0;
        for k in 0..365i64 {
            let jdn = GMT_CORRELATION + k;
            let (day, mi) = haab(jdn);
            assert!(mi < 19);
            if mi == 18 {
                wayeb_days += 1;
                assert!(day < 5);
            } else {
                assert!(day < 20);
            }
        }
        assert_eq!(wayeb_days, 5);
        // 第 365 天 Haab 回到起点。
        assert_eq!(haab(GMT_CORRELATION + 365), haab(GMT_CORRELATION));
    }

    #[test]
    fn long_count_radix_roundtrip() {
        // 任取若干 JDN，Long Count 还原回总天数应等于 jdn - 历元。
        for &jdn in &[GMT_CORRELATION, 2_456_283, 2_460_311, GMT_CORRELATION + 123_456] {
            let lc = long_count(jdn);
            let total: i64 = lc.iter().zip(PLACE_DAYS.iter().rev()).map(|(&v, &p)| v * p).sum();
            assert_eq!(total, jdn - GMT_CORRELATION, "jdn={jdn}");
            // 各位进制范围。
            assert!((0..20).contains(&lc[1]) && (0..20).contains(&lc[2]));
            assert!((0..18).contains(&lc[3]) && (0..20).contains(&lc[4]));
        }
    }

    #[test]
    fn before_epoch_is_negative_baktun() {
        // 历元前一天：Long Count 退一个 kin（12.19.19.17.19 over a negative baktun frame）。
        let lc = long_count(GMT_CORRELATION - 1);
        let total: i64 = lc.iter().zip(PLACE_DAYS.iter().rev()).map(|(&v, &p)| v * p).sum();
        assert_eq!(total, -1);
    }
}
