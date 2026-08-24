//! L3 叶（A 族 / 确定性）：藏历的**循环要素**（年级）。
//!
//! 本 crate 只算藏历里纯周期的年属性，不做闰月 / 缺日的完整推步（那需要 Phugpa/Tsurphu 传承的
//! 逐月天文计算，属另一层工程）：
//!
//! - **60 周期（rab byung）**：`5 元素 × 12 生肖`。性别不是自由轴——它**锁定在生肖上**（鼠虎龙马
//!   猴狗为阳，牛兔蛇羊鸡猪为阴），故组合数 = 5×12 = 60（而非 120）。每元素连跨两年（先阳后阴）。
//! - **年 mewa（sme ba，九宫）**：9 色循环，随年**逆行**（…4，3，2，1，9，8…）。其数字与汉地九星 /
//!   飞星年星完全一致（颜色叫法不同），可永久交叉校验。
//!
//! 历元锚（经多源 + 实算校验）：元素锚 1984 = 木鼠（真木鼠年，**非** 2020；2020 是铁鼠）；生肖锚
//! 2020 = 鼠；rab byung 60 周期历元 1027 = 阴火兔（时轮金刚翻译年）；mewa 由 2024=3（蓝）四重锚钉死。
//!
//! 诚实边界（🟡）：主流（非苯教）藏历**不给年份分配 parkha/卦**——parkha 只用于个人盘（需性别+年龄）。
//! 故本 crate **不输出年 parkha**，仅把 8 个 parkha 名与其后天八卦映射作为 [`PARKHA`] 参考常数暴露。
//! 元素用藏文本义 **Iron（铁）** 而非汉译 Metal。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    reason = "全部相位经 rem_euclid 落 0..60 / 0..9 等小范围，与 i64/usize 间换算受控安全"
)]

mod engine;
pub use engine::TibetanEngine;

use mingli_astro::Moment;
use serde::Serialize;

/// 12 生肖（序从鼠起，Wylie 转写英文名）。
pub const ANIMALS: [&str; 12] = [
    "Rat", "Ox", "Tiger", "Hare", "Dragon", "Snake", "Horse", "Sheep", "Monkey", "Bird", "Dog",
    "Pig",
];

/// 5 元素（顺序 Wood→Fire→Earth→Iron→Water；Iron 为藏文本义，非汉译 Metal）。
pub const ELEMENTS: [&str; 5] = ["Wood", "Fire", "Earth", "Iron", "Water"];

/// 9 个 mewa 颜色（按 mewa 数 1..9 索引，index 0 占位）。
pub const MEWA_COLORS: [&str; 10] = [
    "", "White", "Black", "Blue", "Green", "Yellow", "White", "Red", "White", "Maroon",
];

/// 8 个 parkha（藏文八卦）→ 对应汉卦（后天八卦排列）。年份不配卦，见本 crate 的诚实边界。
pub const PARKHA: [(&str, &str); 8] = [
    ("Kham", "坎"),
    ("Khon", "坤"),
    ("Zin", "震"),
    ("Zon", "巽"),
    ("Khen", "乾"),
    ("Da", "兑"),
    ("Gin", "艮"),
    ("Li", "离"),
];

#[inline]
fn mmod(a: i64, b: i64) -> i64 {
    a.rem_euclid(b)
}

/// 生肖下标 `0..12`（2020 = 鼠）。
#[must_use]
pub fn animal_index(year: i64) -> usize {
    mmod(year - 2020, 12) as usize
}

/// 元素下标 `0..5`（1984 = 木鼠；每元素连跨两年）。
#[must_use]
pub fn element_index(year: i64) -> usize {
    (mmod(year - 1984, 10) / 2) as usize
}

/// 是否阳年（公历偶数年为阳）。性别由此并锁定于生肖。
#[must_use]
pub fn is_male(year: i64) -> bool {
    year.rem_euclid(2) == 0
}

/// 六十周期中的位次 `1..=60`（1984 = 木阳鼠 = 第 1）。
#[must_use]
pub fn sexagenary_position(year: i64) -> i64 {
    mmod(year - 1984, 60) + 1
}

/// 第几个 rab byung（60 年周期，历元 1027）。
#[must_use]
pub fn rabjung_number(year: i64) -> i64 {
    (year - 1027).div_euclid(60) + 1
}

/// 本年在其 rab byung 内的序 `1..=60`（1027 = 第 1）。
#[must_use]
pub fn year_in_rabjung(year: i64) -> i64 {
    mmod(year - 1027, 60) + 1
}

/// Janson Table 15 的 parkha 次第：`1 li · 2 khon · 3 dwa · 4 khen · 5 kham · 6 gin · 7 zin · 8 zon`。
///
/// 第二源：tibastro 的 Parkha 条目把日 parkha 的次第写作
/// 「Li – Khön – Da – Khen – Kham – Kin – Tsin – Zön」，与本表逐名相同（Kin=Gin、Tsin=Zin）；
/// 它并给出月起卦「一/五/九月起 Kham、二/六/十月起 Da」，与 Janson E.10 的
/// 子辰申起 Kham、丑巳酉起 Da 相合。<https://www.tibastro.be/Parkha/ParkhaGeneral>
///
/// 这是**后天八卦的方位顺序**，与 [`PARKHA`] 那张按洛书数排的表不是同一个次序——
/// 历日与阴历日的两个公式都以本表编号为准，取名要经这里，不能拿编号直接索引 [`PARKHA`]。
pub const PARKHA_ORDER: [&str; 8] = ["Li", "Khon", "Da", "Khen", "Kham", "Gin", "Zin", "Zon"];

/// `amod`：值域 `1..=n` 的取模（Janson 全篇的约定）。
fn amod(x: i64, n: i64) -> i64 {
    (x - 1).rem_euclid(n) + 1
}

/// **历日 parkha**：由儒略日数直接得，与阴历推步无关。
///
/// Janson《Tibetan Calendar Mathematics》E.4：`trigram = (JD + 2) amod 8`（Table 15 编号）。
/// 历日的元素 / 性别 / 生肖 / 卦 / 数五者都是简单循环，周期各为 10 / 2 / 12 / 8 / 9。
#[must_use]
pub fn calendar_day_parkha(jdn: i64) -> i64 {
    amod(jdn + 2, 8)
}

/// **阴历日 parkha**：藏历某月第 `lunar_day` 日，该月生肖序为 `month_animal`（1=鼠…12=猪）。
///
/// Janson E.10：`(D + 30(A − 3)) amod 8 = (D + 6A + 6) amod 8`。
/// 锚在寅月初一为 Li——原文并给出全部十二月的起卦：寅午戌月起 Li、卯未亥月起 Zin、
/// 子辰申月起 Kham、丑巳酉月起 Da。
///
/// 之所以吃**生肖**而不是月号：月序到生肖的映射 Phugpa 与 Tsurphu 两派不同，
/// 以月号为参会把这层分歧悄悄带进公式。本 crate 不做藏历阴历推步（无闰月 / 缺日），
/// 故 `lunar_day` 与 `month_animal` 由调用方给出。
#[must_use]
pub fn lunar_day_parkha(lunar_day: i64, month_animal: i64) -> i64 {
    amod(lunar_day + 6 * month_animal + 6, 8)
}

/// 年 mewa `1..=9`（随年逆行）。等价于 Janson `amod(2−Y,9)`。
#[must_use]
pub fn mewa(year: i64) -> i64 {
    mmod(1 - year, 9) + 1
}

/// 一年藏历循环要素的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Cast {
    /// 公历年（≈ 藏历年；Losar 边界见 [`compute_at`] 注）。
    pub year: i64,
    /// 生肖名。
    pub animal: &'static str,
    /// 元素名（Wood/Fire/Earth/Iron/Water）。
    pub element: &'static str,
    /// 是否阳年。
    pub male: bool,
    /// 六十周期位次 `1..=60`。
    pub sexagenary: i64,
    /// 第几个 rab byung。
    pub rabjung: i64,
    /// 在 rab byung 内的序 `1..=60`。
    pub year_in_rabjung: i64,
    /// 年 mewa 数 `1..=9`。
    pub mewa: i64,
    /// 年 mewa 颜色。
    pub mewa_color: &'static str,
    /// 本历日的 parkha 编号 `1..=8`（Janson Table 15 次第）。
    pub day_parkha: i64,
    /// 本历日的 parkha 名。
    pub day_parkha_name: &'static str,
}

/// 算某公历年的藏历循环要素（核心入口，确定性、经多锚点校验）。
#[must_use]
pub fn compute_year(year: i64) -> Cast {
    compute_year_on(year, None)
}

/// 同 [`compute_year`]，另按给定儒略日数填历日 parkha。
fn compute_year_on(year: i64, jdn: Option<i64>) -> Cast {
    let m = mewa(year);
    let pk = jdn.map_or(1, calendar_day_parkha);
    Cast {
        year,
        day_parkha: pk,
        day_parkha_name: PARKHA_ORDER[(pk - 1) as usize],
        animal: ANIMALS[animal_index(year)],
        element: ELEMENTS[element_index(year)],
        male: is_male(year),
        sexagenary: sexagenary_position(year),
        rabjung: rabjung_number(year),
        year_in_rabjung: year_in_rabjung(year),
        mewa: m,
        mewa_color: MEWA_COLORS[m as usize],
    }
}

/// 在共享上下文 [`Moment`] 上算藏历年要素。
///
/// 注：藏历年以 Losar（约公历 2 月）换年；本 crate 不做 Losar 推步，直接取**公历年**为藏历年近似。
/// 故 1–2 月（Losar 前）的日期，其藏历年属性可能仍属前一年（🟡 边界）。年属性本身经锚点校验精确。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    compute_year_on(i64::from(m.year), Some(m.civil_day))
}

/// 由本地民用日期算（独立入口）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, 12, 0, tz))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_year(y: i64, el: &str, male: bool, animal: &str, sex: i64, mw: i64) {
        let c = compute_year(y);
        assert_eq!(c.element, el, "{y} element");
        assert_eq!(c.male, male, "{y} gender");
        assert_eq!(c.animal, animal, "{y} animal");
        assert_eq!(c.sexagenary, sex, "{y} sexagenary");
        assert_eq!(c.mewa, mw, "{y} mewa");
    }

    #[test]
    fn anchor_years_cross_verified() {
        // 全部来自研究的四重/多源锚点。
        assert_year(2024, "Wood", true, "Dragon", 41, 3);
        assert_year(2023, "Water", false, "Hare", 40, 4);
        assert_year(2025, "Wood", false, "Snake", 42, 2);
        assert_year(2026, "Fire", true, "Horse", 43, 1);
        assert_year(1027, "Fire", false, "Hare", 4, 1); // rab byung 历元
        assert_year(1984, "Wood", true, "Rat", 1, 7); // 真木鼠锚
        // 2020 是铁鼠（不是木鼠）——历元陷阱。mewa 逆行 2024=3 → 2020=7。
        assert_year(2020, "Iron", true, "Rat", 37, 7);
    }

    #[test]
    fn rabjung_anchors() {
        // 1027 = 第 1 个 rab byung 第 1 年；1987 = 第 17 个之始；2024 = 第 17 个第 38 年。
        assert_eq!(rabjung_number(1027), 1);
        assert_eq!(year_in_rabjung(1027), 1);
        assert_eq!(rabjung_number(1987), 17);
        assert_eq!(year_in_rabjung(1987), 1);
        let c = compute_year(2024);
        assert_eq!((c.rabjung, c.year_in_rabjung), (17, 38));
    }

    #[test]
    fn sixty_cycle_structure() {
        // 六十周期：位次 1..=60 在 60 年内恰好遍历一次；性别锁生肖（阳生肖恒阳年）。
        use std::collections::HashSet;
        let mut positions = HashSet::new();
        for y in 1984..1984 + 60 {
            let c = compute_year(y);
            assert!(positions.insert(c.sexagenary));
            assert!((1..=60).contains(&c.sexagenary));
            // 阳年 ↔ 阳生肖（鼠虎龙马猴狗，生肖下标为偶）。
            assert_eq!(c.male, animal_index(y).is_multiple_of(2));
        }
        assert_eq!(positions.len(), 60);
        // 元素每 2 年一换、每 10 年一轮。
        assert_eq!(compute_year(1984).element, compute_year(1985).element);
        assert_ne!(compute_year(1985).element, compute_year(1986).element);
        assert_eq!(compute_year(1984).element, compute_year(1994).element);
    }

    /// 阴历日 parkha 的公式，用原文自己给出的十二月起卦逐条对。
    ///
    /// 来源：Janson《Tibetan Calendar Mathematics》E.10——
    /// 「A Tiger month begins with trigram number 1, Li」并列出全部十二个月的起卦：
    /// 寅午戌起 Li、卯未亥起 Zin、子辰申起 Kham、丑巳酉起 Da。
    /// 公式与这份枚举是原文里两处彼此独立的陈述，互为校验。
    #[test]
    fn every_month_begins_with_the_trigram_the_source_names() {
        // (生肖序 1=鼠…12=猪, 该月初一的 parkha 名)
        let want = [
            (3, "Li"), (7, "Li"), (11, "Li"),
            (4, "Zin"), (8, "Zin"), (12, "Zin"),
            (1, "Kham"), (5, "Kham"), (9, "Kham"),
            (2, "Da"), (6, "Da"), (10, "Da"),
        ];
        for (animal, name) in want {
            let k = lunar_day_parkha(1, animal);
            assert_eq!(PARKHA_ORDER[(k - 1) as usize], name, "生肖序 {animal} 的月，初一应起 {name}");
        }
    }

    /// 三个代数等价的写法必须给同一个数——原文并列了它们，抄错一个就露馅。
    #[test]
    fn the_three_forms_of_the_formula_agree() {
        for animal in 1..=12_i64 {
            for day in 1..=30_i64 {
                let a = amod(day + 30 * (animal - 3), 8);
                let b = amod(day - 2 * animal - 2, 8);
                assert_eq!((a, b), (lunar_day_parkha(day, animal), lunar_day_parkha(day, animal)));
            }
        }
    }

    /// 历日 parkha 是 JD 的简单八循环（Janson E.4），且随年盘一起给出。
    #[test]
    fn the_calendar_day_trigram_cycles_with_the_julian_day() {
        // 原先三条断言，常量函数一条都不违反：值域由 `amod` 保证、「八日一轮」对常量
        // 天然成立、名字取自它自己报的编号。实测把整个函数换成 `1`，全量套件一条不红。
        // 改成钉住它真正该有的形状：任意连续八日恰好取遍 1..=8，且逐日进一、8 之后回 1。
        for start in [2_460_000_i64, 2_400_000, 2_500_000, 0] {
            let window: Vec<i64> = (0..8).map(|k| calendar_day_parkha(start + k)).collect();
            let mut sorted = window.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, (1..=8).collect::<Vec<_>>(), "自 {start} 起八日应取遍 1..=8");
            for pair in window.windows(2) {
                assert_eq!(pair[1], pair[0] % 8 + 1, "逐日进一");
            }
            assert_eq!(
                calendar_day_parkha(start),
                calendar_day_parkha(start + 8),
                "八日一轮"
            );
        }
        let c = compute(2024, 1, 1, 8.0);
        assert_eq!(c.day_parkha, calendar_day_parkha(mingli_astro::civil_day_number(2024, 1, 1)));
        assert_eq!(c.day_parkha_name, PARKHA_ORDER[(c.day_parkha - 1) as usize]);

        // 相位（哪一天是 Li）目前只有 Janson E.4 一源，找不到第二处可查的
        // 「某公历日 → 某 parkha」实据，故此处不作外部断言，只冻结现状防止无声漂移。
        // 若日后找到藏历历书的逐日 parkha 表，这两行应换成真正的 oracle。
        assert_eq!(calendar_day_parkha(2_460_000), 2, "冻结现状，非外部求证");
        assert_eq!(calendar_day_parkha(2_460_001), 3, "冻结现状，非外部求证");
    }

    /// 两张 parkha 表次序不同，不能拿编号互相索引——这条钉住那件事。
    #[test]
    fn the_two_trigram_tables_are_in_different_orders() {
        let luoshu: Vec<&str> = PARKHA.iter().map(|(n, _)| *n).collect();
        let order: Vec<&str> = PARKHA_ORDER.to_vec();
        assert_ne!(luoshu, order, "两表次序不同；公式给的编号只能经 PARKHA_ORDER 取名");
        let (mut a, mut b) = (luoshu, order);
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b, "两表收的是同八个卦，只是次序不同");
    }

    #[test]
    fn mewa_runs_backward_and_wraps() {
        // mewa 逐年逆行，每 9 年一轮，恒在 1..=9。
        for y in 1900..2100 {
            let m = mewa(y);
            assert!((1..=9).contains(&m));
            assert_eq!(mewa(y + 1), if m == 1 { 9 } else { m - 1 }); // 逆行
        }
        // 数字与汉地飞星年星一致：2024=3。
        assert_eq!(mewa(2024), 3);
        assert_eq!(MEWA_COLORS[mewa(2024) as usize], "Blue");
    }

    #[test]
    fn negative_years_safe() {
        // 历元前年份不 panic（mmod 防负），位次仍合法。
        let c = compute_year(-100);
        assert!((1..=60).contains(&c.sexagenary));
        assert!((1..=9).contains(&c.mewa));
        assert!((1..=60).contains(&c.year_in_rabjung));
    }

    #[test]
    fn reference_tables_well_formed() {
        assert_eq!(ANIMALS.len(), 12);
        assert_eq!(ELEMENTS.len(), 5);
        assert_eq!(PARKHA.len(), 8);
        assert_eq!(ELEMENTS[3], "Iron"); // 非 Metal
        // parkha 八卦映射含全部后天八卦。
        let trigrams: std::collections::HashSet<_> = PARKHA.iter().map(|&(_, t)| t).collect();
        assert_eq!(trigrams.len(), 8);
        assert!(trigrams.contains("坎") && trigrams.contains("离"));
    }

    #[test]
    fn compute_from_moment() {
        let c = compute(2024, 6, 15, 8.0);
        assert_eq!((c.element, c.animal, c.mewa), ("Wood", "Dragon", 3));
    }
}
