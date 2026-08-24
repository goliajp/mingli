//! L3 叶（A 族 / 确定性）：巴厘岛 Pawukon 历。
//!
//! Pawukon 是一个 `210 = 2·3·5·7` 天的大循环，上面**同时并行**跑十个长度 1..10 的週（wewaran）。
//! 这是 [`mingli_core::cyclic`]「多并行轮」最丰富的范例，但比纯 CRT 多了两类不规则结构：
//!
//! - **简单 mod 週**：Triwara(3)/Pancawara(5)/Sadwara(6)/Saptawara(7) 直接 `day mod n`
//!   （由 [`mingli_core::cyclic::parallel_phases`] 一次取齐）。
//! - **派生週**：Dasawara(10)/Dwiwara(2)/Ekawara(1) 由「Pancawara urip + Saptawara urip」之和定
//!   （`urip` 是各日的数值权重）。
//! - **卡日週**：Caturwara(4)/Astawara(8)/Sangawara(9) 因 `n ∤ 210`，靠固定断点重复某日对齐。
//!
//! 日序对齐：`day = (JDN − 146) mod 210`，`day 0 = 公历 2020-07-05 = Redite·Paing·Wuku Sinta`
//! （锚点 146 = Dershowitz & Reingold baliEpoch，经多源 + 实算校验）。
//!
//! 语域注：本 crate 只做历日週序换算（确定性），不涉巴厘占卜释义。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    reason = "全部相位经 rem_euclid 落在 0..210 等小范围，与 i64/usize 间换算受控安全"
)]

mod engine;
pub use engine::PawukonEngine;

use mingli_astro::Moment;
use serde::Serialize;

/// baliEpoch：`day = (JDN − BALI_EPOCH) mod 210`，day 0 = 2020-07-05。
pub const BALI_EPOCH: i64 = 146;

/// Triwara(3)。`day mod 3`。
pub const TRIWARA: [&str; 3] = ["Pasah", "Beteng", "Kajeng"];
/// Pancawara(5)。`day mod 5`，index 0 = Paing（计算起点，非民俗书写起点 Umanis）。
pub const PANCAWARA: [&str; 5] = ["Paing", "Pon", "Wage", "Kliwon", "Umanis"];
/// Sadwara(6)。`day mod 6`。
pub const SADWARA: [&str; 6] = ["Tungleh", "Aryang", "Urukung", "Paniron", "Was", "Maulu"];
/// Saptawara(7)。`day mod 7`，index 0 = Redite。
pub const SAPTAWARA: [&str; 7] = [
    "Redite", "Soma", "Anggara", "Buda", "Wraspati", "Sukra", "Saniscara",
];
/// Dasawara(10)。由 urip 之和 `mod 10` 取。
pub const DASAWARA: [&str; 10] = [
    "Pandita", "Pati", "Suka", "Duka", "Sri", "Manuh", "Manusa", "Raja", "Dewa", "Raksasa",
];
/// Caturwara(4) 成员（卡日週）。
pub const CATURWARA: [&str; 4] = ["Sri", "Laba", "Jaya", "Menala"];
/// Astawara(8) 成员（卡日週）。
pub const ASTAWARA: [&str; 8] = [
    "Sri", "Indra", "Guru", "Yama", "Ludra", "Brahma", "Kala", "Uma",
];
/// Sangawara(9) 成员（卡日週）。
pub const SANGAWARA: [&str; 9] = [
    "Dangu", "Jangur", "Gigis", "Nohan", "Ogan", "Erangan", "Urungan", "Tulus", "Dadi",
];
/// 30 个 Wuku（每 7 天一个），`day / 7`。
pub const WUKU: [&str; 30] = [
    "Sinta", "Landep", "Ukir", "Kulantir", "Tolu", "Gumbreg", "Wariga", "Warigadean",
    "Julungwangi", "Sungsang", "Dungulan", "Kuningan", "Langkir", "Medangsia", "Pujut", "Pahang",
    "Krulut", "Merakih", "Tambir", "Medangkungan", "Matal", "Uye", "Menail", "Prangbakat", "Bala",
    "Ugu", "Wayang", "Klawu", "Dukut", "Watugunung",
];

/// Pancawara 各日 urip（与 [`PANCAWARA`] 同序）：Paing=9，Pon=7，Wage=4，Kliwon=8，Umanis=5。
const PANCAWARA_URIP: [u32; 5] = [9, 7, 4, 8, 5];
/// Saptawara 各日 urip（与 [`SAPTAWARA`] 同序）：Redite=5..Saniscara=9。
const SAPTAWARA_URIP: [u32; 7] = [5, 4, 3, 7, 8, 6, 9];

/// 由民用日序（JDN）得 Pawukon 日序 `0..210`。
#[must_use]
pub fn pawukon_day(jdn: i64) -> usize {
    (jdn - BALI_EPOCH).rem_euclid(210) as usize
}

/// Caturwara(4) 下标：day<71 用 `day%4`；day∈{71，72} 卡 Jaya(2)；之后用 `(day−2)%4`。
#[must_use]
pub fn caturwara_index(day: usize) -> usize {
    if day < 71 {
        day % 4
    } else if day <= 72 {
        2 // Jaya
    } else {
        (day - 2) % 4
    }
}

/// Astawara(8) 下标：day<71 用 `day%8`；day∈{71，72} 卡 Kala(6)；之后用 `(day−2)%8`。
#[must_use]
pub fn astawara_index(day: usize) -> usize {
    if day < 71 {
        day % 8
    } else if day <= 72 {
        6 // Kala
    } else {
        (day - 2) % 8
    }
}

/// Sangawara(9) 下标：day≤3 卡 Dangu(0)；之后用 `(day−3)%9`。
#[must_use]
pub fn sangawara_index(day: usize) -> usize {
    if day <= 3 {
        0 // Dangu
    } else {
        (day - 3) % 9
    }
}

/// 一日 Pawukon 全週的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Cast {
    /// Pawukon 日序 `0..210`。
    pub day: usize,
    /// 该日 urip = Pancawara urip + Saptawara urip（派生週的依据）。
    pub urip: u32,
    /// Wuku 名（`day / 7`）。
    pub wuku: &'static str,
    /// Triwara。
    pub triwara: &'static str,
    /// Pancawara。
    pub pancawara: &'static str,
    /// Sadwara。
    pub sadwara: &'static str,
    /// Saptawara。
    pub saptawara: &'static str,
    /// Dasawara（urip 之和 mod 10）。
    pub dasawara: &'static str,
    /// Dwiwara：urip 之和为偶 = Menga（开），为奇 = Pepet（闭）。
    pub dwiwara: &'static str,
    /// Ekawara：urip 之和为奇的日子是 Luang，为偶的日子**没有** Ekawara（`None`）。
    ///
    /// 一日週并非每天都有——这一点四源明言（en.wikipedia「is not a day of the one-day week」、
    /// Hindu Alukta「Ekawaranya tidak ada」、balinese-date-js-lib 的 `EkaWara.VOID`、
    /// sakacalendar 的 `bukan luang`），故用 `Option` 而不是必填枚举。
    pub ekawara: Option<&'static str>,
    /// Caturwara（卡日週）。
    pub caturwara: &'static str,
    /// Astawara（卡日週）。
    pub astawara: &'static str,
    /// Sangawara（卡日週）。
    pub sangawara: &'static str,
}

/// 由 Pawukon 日序 `0..210` 算全週（核心入口）。
#[must_use]
pub fn compute_from_day(day: usize) -> Cast {
    // 简单 mod 週一次取齐（Triwara/Pancawara/Sadwara/Saptawara）。
    let phases = mingli_core::cyclic::parallel_phases(day as i64, &[3, 5, 6, 7]);
    let (tri, pan, sad, sap) = (
        phases[0] as usize,
        phases[1] as usize,
        phases[2] as usize,
        phases[3] as usize,
    );
    let urip = PANCAWARA_URIP[pan] + SAPTAWARA_URIP[sap];
    Cast {
        day,
        urip,
        wuku: WUKU[day / 7],
        triwara: TRIWARA[tri],
        pancawara: PANCAWARA[pan],
        sadwara: SADWARA[sad],
        saptawara: SAPTAWARA[sap],
        dasawara: DASAWARA[(urip % 10) as usize],
        dwiwara: if urip.is_multiple_of(2) { "Menga" } else { "Pepet" },
        ekawara: if urip.is_multiple_of(2) { None } else { Some("Luang") },
        caturwara: CATURWARA[caturwara_index(day)],
        astawara: ASTAWARA[astawara_index(day)],
        sangawara: SANGAWARA[sangawara_index(day)],
    }
}

/// 在共享上下文 [`Moment`] 上算 Pawukon 全週（取其民用日序）。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    compute_from_day(pawukon_day(m.civil_day))
}

/// 由本地民用日期算 Pawukon 全週（独立入口）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, 12, 0, tz))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day0_is_2020_07_05() {
        // 锚点：2020-07-05 = day 0 = Redite·Paing·Sinta。
        assert_eq!(mingli_astro::civil_day_number(2020, 7, 5), 2_459_036);
        assert_eq!(pawukon_day(2_459_036), 0);
        let c = compute_from_day(0);
        assert_eq!((c.saptawara, c.pancawara, c.wuku), ("Redite", "Paing", "Sinta"));
    }

    #[test]
    fn oracle_galungan_2025_04_23() {
        // 节日定义锚：Galungan 必为 Buda·Kliwon·Dungulan（day 73）。
        let jdn = mingli_astro::civil_day_number(2025, 4, 23);
        assert_eq!(jdn, 2_460_789);
        let c = compute_from_day(pawukon_day(jdn));
        assert_eq!(c.day, 73);
        assert_eq!((c.saptawara, c.pancawara, c.wuku), ("Buda", "Kliwon", "Dungulan"));
        // urip = 7(Buda) + 8(Kliwon) = 15 → Dasawara Manuh、Ekawara Luang、Dwiwara Pepet。
        assert_eq!(c.urip, 15);
        assert_eq!(c.dasawara, "Manuh");
        assert_eq!(c.ekawara, Some("Luang"));
        assert_eq!(c.dwiwara, "Pepet");
        // 卡日週（day73>72）：Caturwara Menala、Astawara Uma、Sangawara Tulus。
        assert_eq!((c.caturwara, c.astawara, c.sangawara), ("Menala", "Uma", "Tulus"));
    }

    #[test]
    fn oracle_wikipedia_2021_01_05() {
        // Wikipedia 工作示例：day 184 = Anggara·Umanis·Wuku Wayang·Sadwara Was。
        let jdn = mingli_astro::civil_day_number(2021, 1, 5);
        assert_eq!(jdn, 2_459_220);
        let c = compute_from_day(pawukon_day(jdn));
        assert_eq!(c.day, 184);
        assert_eq!(c.saptawara, "Anggara");
        assert_eq!(c.pancawara, "Umanis");
        assert_eq!(c.wuku, "Wayang");
        assert_eq!(c.sadwara, "Was");
    }

    #[test]
    fn name_tables_well_formed() {
        assert_eq!(WUKU.len(), 30);
        assert_eq!(TRIWARA.len(), 3);
        assert_eq!(DASAWARA.len(), 10);
        assert_eq!(SANGAWARA.len(), 9);
        assert_eq!(PANCAWARA_URIP.len(), 5);
        assert_eq!(SAPTAWARA_URIP.len(), 7);
    }

    #[test]
    fn stuck_day_weeks_cover_all_members_over_210() {
        // 卡日週遍历整个 210 周期：所有下标合法，且各成员都被覆盖到。
        use std::collections::HashSet;
        let (mut c4, mut c8, mut c9) = (HashSet::new(), HashSet::new(), HashSet::new());
        for day in 0..210 {
            let i4 = caturwara_index(day);
            let i8 = astawara_index(day);
            let i9 = sangawara_index(day);
            assert!(i4 < 4 && i8 < 8 && i9 < 9);
            c4.insert(i4);
            c8.insert(i8);
            c9.insert(i9);
        }
        assert_eq!(c4.len(), 4);
        assert_eq!(c8.len(), 8);
        assert_eq!(c9.len(), 9);
        // 卡日点：day 71、72 分别落 Jaya(2)/Kala(6)；day 0..=3 全 Dangu(0)。
        assert_eq!(caturwara_index(71), 2);
        assert_eq!(astawara_index(72), 6);
        for d in 0..=3 {
            assert_eq!(sangawara_index(d), 0);
        }
    }

    /// 卡日週的两条独立对照。
    ///
    /// 原有的那条只验「八个成员都出现过」与「下标 < 8」——后者由 `% 8` 保证，前者
    /// 很多种错法都满足。实测把 `astawara_index` 正常段整段改成常量 Kala，
    /// 全量套件一条不红，等于八曜的正常段从没被看过。
    ///
    /// 这里换成与 Dershowitz & Reingold《Calendrical Calculations》的闭式逐日对照。
    /// 那个式子的形状与本仓的三分支写法完全不同——它用一个 `max` 把卡日点吸收掉，
    /// 没有分支——所以两边同时错成一样的可能性很小。
    ///
    /// 名序与卡日规则两源相合：
    /// 1. <https://en.wikipedia.org/wiki/Pawukon_calendar>
    /// 2. <https://factsanddetails.com/indonesia/Minorities_and_Regions/sub6_3h/entry-9758.html>
    ///
    /// 两处都作 Astawara = Sri·Indra·Guru·Yama·Ludra·Brahma·Kala·Uma、
    /// Caturwara = Sri·Laba·Jaya·Menala，且「本该在第 72 日结束的那一周里，
    /// 倒数第二日重复」——倒数第二在八曜正是 Kala、在四曜正是 Jaya。
    ///
    /// 附一句留给下一个读到这里的人：按上面那句散文推，Kala 该连出现两天，而实算是
    /// 0 基第 70、71、72 连出现三天。查过 D&R 的闭式，它算出的也是三天，两边逐日相同。
    /// 散文那句是粗略说法，代码没有偏一天。
    #[test]
    fn the_stuck_day_weeks_agree_with_the_reference_closed_form() {
        // D&R：asatawara = max(6, 4 + (day − 70) mod 210) mod 8，caturwara = asatawara mod 4。
        let reference_astawara = |day: usize| -> usize {
            let shifted = (day + 210 - 70) % 210;
            core::cmp::max(6, 4 + shifted) % 8
        };
        for day in 0..210 {
            assert_eq!(
                astawara_index(day),
                reference_astawara(day),
                "第 {day} 日的八曜与 D&R 闭式不合"
            );
            assert_eq!(
                caturwara_index(day),
                reference_astawara(day) % 4,
                "第 {day} 日的四曜应为八曜下标模 4"
            );
        }

        // 卡日点前后的实际名序，逐日写死。
        let names: Vec<&str> = (66..78).map(|d| ASTAWARA[astawara_index(d)]).collect();
        assert_eq!(
            names,
            [
                "Guru", "Yama", "Ludra", "Brahma", "Kala", "Kala", "Kala", "Uma", "Sri", "Indra",
                "Guru", "Yama"
            ]
        );

        // 210 不被 8 整除，靠重复把 Kala 多占两日补齐：26×8 = 208，Kala 得 28。
        // 四曜同理：52×4 = 208，Jaya 得 54。
        let mut eight = [0u32; 8];
        let mut four = [0u32; 4];
        for day in 0..210 {
            eight[astawara_index(day)] += 1;
            four[caturwara_index(day)] += 1;
        }
        assert_eq!(eight, [26, 26, 26, 26, 26, 26, 28, 26], "八曜各名在 210 日内的次数");
        assert_eq!(four, [52, 52, 54, 52], "四曜各名在 210 日内的次数");
        assert_eq!(eight.iter().sum::<u32>(), 210);
        assert_eq!(four.iter().sum::<u32>(), 210);
        assert_eq!(ASTAWARA[6], "Kala");
        assert_eq!(CATURWARA[2], "Jaya");
    }

    #[test]
    fn cycle_closes_at_210() {
        // 整个盘每 210 天复位。
        let a = compute_from_day(0);
        let jdn0 = BALI_EPOCH;
        let c_wrap = compute_from_day(pawukon_day(jdn0 + 210));
        assert_eq!(c_wrap.day, a.day);
        assert_eq!(c_wrap.saptawara, a.saptawara);
        assert_eq!(c_wrap.caturwara, a.caturwara);
        // 组合简单週周期 = lcm(3，5，6，7) = 210。
        assert_eq!(mingli_core::cyclic::cycle_period(&[3, 5, 6, 7]), 210);
    }

    /// urip 权重表的 oracle：5 个独立源逐值一致。
    ///
    /// Babad Bali（巴厘本地 wewaran 大表）· en.wikipedia「Pawukon calendar」·
    /// Reingold–Dershowitz《Calendrical Calculations》参考实现的 `i_values`/`j_values` ·
    /// sakacalendar（Java）· balinese-date-js-lib（TypeScript）。
    #[test]
    fn urip_weights_match_five_independent_sources() {
        // Pancawara 按本 crate 的排序（Paing 起）：Paing 9、Pon 7、Wage 4、Kliwon 8、Umanis 5
        assert_eq!(PANCAWARA_URIP, [9, 7, 4, 8, 5]);
        // 换成 Umanis 起的通行列法应得 5, 9, 7, 4, 8（R&D 参考实现的 i_values）
        let umanis_first: Vec<u32> = (0..5).map(|k| PANCAWARA_URIP[(k + 4) % 5]).collect();
        assert_eq!(umanis_first, vec![5, 9, 7, 4, 8]);
        // Saptawara Redite 起：5, 4, 3, 7, 8, 6, 9（R&D 的 j_values，与 Babad Bali 同）
        assert_eq!(SAPTAWARA_URIP, [5, 4, 3, 7, 8, 6, 9]);
    }

    /// 派生週的奇偶向：urip 之和为奇 → Luang + Pepet，为偶 → 无 Ekawara + Menga。
    ///
    /// 六个独立源同向：en.wikipedia（以 `urip+1` 写，等价）· Reingold–Dershowitz 参考实现
    /// （`luang ⇔ (1+S) mod 10 为偶 ⇔ S 为奇`）· sakacalendar · balinese-date-js-lib ·
    /// Sastra Bali · Hindu Alukta。仅见一处相反记载，与 Sastra Bali 同文转载且自身
    /// Eka / Dwi 两栏互相矛盾，判为转载讹误，不建流派。
    #[test]
    fn ekawara_dwiwara_parity_matches_six_independent_sources() {
        // Buda(urip 7) + Kliwon(urip 8) = 15，奇 → Luang / Pepet
        let odd = compute_from_day((0..210).find(|&d| compute_from_day(d).urip == 15).expect("应有 urip=15 的日"));
        assert_eq!((odd.ekawara, odd.dwiwara), (Some("Luang"), "Pepet"));
        // Soma(4) + Wage(4) = 8，偶 → 无 Ekawara / Menga
        let even = compute_from_day((0..210).find(|&d| compute_from_day(d).urip == 8).expect("应有 urip=8 的日"));
        assert_eq!((even.ekawara, even.dwiwara), (None, "Menga"));
        // 210 天里两种都得出现，否则上面两条是空转
        let luang = (0..210).filter(|&d| compute_from_day(d).ekawara.is_some()).count();
        assert!(luang > 0 && luang < 210, "Luang 日应既非零也非全部，实得 {luang}/210");
    }

    #[test]
    fn ekawara_dwiwara_parity_consistent() {
        // Ekawara 仅在 urip 为奇时出现；Dwiwara 随同奇偶。遍历 210 天自洽。
        for day in 0..210 {
            let c = compute_from_day(day);
            if c.urip.is_multiple_of(2) {
                assert_eq!(c.ekawara, None);
                assert_eq!(c.dwiwara, "Menga");
            } else {
                assert_eq!(c.ekawara, Some("Luang"));
                assert_eq!(c.dwiwara, "Pepet");
            }
        }
    }

    #[test]
    fn compute_from_moment_matches() {
        let c = compute(2025, 4, 23, 0.0);
        assert_eq!(c.wuku, "Dungulan");
        assert_eq!(c.saptawara, "Buda");
    }
}
