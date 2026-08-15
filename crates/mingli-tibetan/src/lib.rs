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

/// 8 个 parkha（藏文八卦）→ 对应汉卦（后天八卦排列）。仅作参考常数；主流藏历无年 parkha。
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
}

/// 算某公历年的藏历循环要素（核心入口，确定性、经多锚点校验）。
#[must_use]
pub fn compute_year(year: i64) -> Cast {
    let m = mewa(year);
    Cast {
        year,
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
    compute_year(i64::from(m.year))
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
