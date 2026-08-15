//! L3 叶（D 族 / 确定性）：西洋数字学（numerology）。
//!
//! 数字学是「符号 → 数值（查表 φ）→ 求和（幺半群同态）→ 数字根约化」的最纯范例，骨架全在
//! [`mingli_core::ringhash`]。本 crate 提供两套字母表与四个常见数：
//!
//! - **字母表**：Pythagorean（A=1…I=9 循环，复用 [`mingli_core::ringhash::pythagorean`]）与
//!   Chaldean（1..8，9 为神圣不配字母；按振动而非顺序，见 [`chaldean`]）。
//! - **生命灵数（Life Path）**：出生年月日各自约化后求和再约化（保留主数 11/22/33）。
//! - **生日数（Birthday）**：出生「日」约化。
//! - **表达数 / 灵魂数 / 人格数**：姓名全字母 / 元音 / 辅音之和约化。
//!
//! 约化用 [`mingli_core::ringhash::reduce_with_master`]（遇 11/22/33 停）。
//!
//! 语域注：数本身是确定计算；其「含义」属释义层，本 crate 不下断言。
//! 🟡 欠定项：生命灵数有「分量约化」与「全数字相加」两法（本 crate 用分量约化，多数教材主数法）；
//! 元音是否含 Y 随流派（本 crate 仅 AEIOU 为元音）。两者已文档化，不静默选边。

use mingli_astro::Moment;
use mingli_core::ringhash::{pythagorean, reduce_with_master, string_sum};
use serde::Serialize;

/// 字母表系统。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum System {
    /// Pythagorean：A=1…I=9 循环。
    Pythagorean,
    /// Chaldean：1..8（9 不配字母）。
    Chaldean,
}

/// Chaldean 字母值（A..Z，索引 0..26）。1：AIJQY 2：BKR 3：CGLS 4：DMT 5：EHNX 6：UVW 7：OZ 8：FP。
const CHALDEAN: [u64; 26] = [
    1, 2, 3, 4, 5, 8, 3, 5, 1, 1, 2, 3, 4, 5, 7, 8, 1, 2, 3, 4, 6, 6, 6, 5, 1, 7,
];

/// Chaldean 字母值；非 A-Z 返回 `None`。
#[must_use]
pub fn chaldean(c: char) -> Option<u64> {
    let u = c.to_ascii_uppercase();
    if u.is_ascii_uppercase() {
        Some(CHALDEAN[(u as usize) - ('A' as usize)])
    } else {
        None
    }
}

/// 某系统下的字母值。
#[must_use]
pub fn letter_value(c: char, system: System) -> Option<u64> {
    match system {
        System::Pythagorean => pythagorean(c),
        System::Chaldean => chaldean(c),
    }
}

/// 是否元音（仅 AEIOU；Y 不计，见 crate 文档 🟡）。
#[must_use]
pub fn is_vowel(c: char) -> bool {
    matches!(c.to_ascii_uppercase(), 'A' | 'E' | 'I' | 'O' | 'U')
}

/// 生命灵数的约化方法（流派分歧）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LifePathMethod {
    /// **Component（分量约化）**：y/m/d 各约化后求和，再约化（保留主数 11/22/33）。
    /// 当代 Pythagorean 学派多用此法，易识别中间主数。
    Component,
    /// **WholeSum（全数字直加）**：y/m/d 全部数字平铺相加，再约化（保留主数 11/22/33）。
    /// 古典 Chaldean / Kabbalistic 派常用，主数仅出现在最终一次约化。
    WholeSum,
}

/// 生命灵数（按指定流派算）。
#[must_use]
pub fn life_path_with(year: i64, month: u32, day: u32, method: LifePathMethod) -> u64 {
    #[allow(clippy::cast_sign_loss, reason = "出生年取正；下游仅数字根")]
    let yabs = year.unsigned_abs();
    match method {
        LifePathMethod::Component => {
            let y = reduce_with_master(digit_sum_u64(yabs));
            let m = reduce_with_master(u64::from(month));
            let d = reduce_with_master(u64::from(day));
            reduce_with_master(y + m + d)
        }
        LifePathMethod::WholeSum => {
            let s = digit_sum_u64(yabs) + digit_sum_u64(u64::from(month)) + digit_sum_u64(u64::from(day));
            reduce_with_master(s)
        }
    }
}

/// 生命灵数（默认 [`LifePathMethod::Component`]，分量约化法）。
#[must_use]
pub fn life_path(year: i64, month: u32, day: u32) -> u64 {
    life_path_with(year, month, day, LifePathMethod::Component)
}

/// 各位数字之和（多位年份用）。
fn digit_sum_u64(mut n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut s = 0;
    while n > 0 {
        s += n % 10;
        n /= 10;
    }
    s
}

/// 生日数：出生「日」约化（保留主数）。
#[must_use]
pub fn birthday_number(day: u32) -> u64 {
    reduce_with_master(u64::from(day))
}

/// 表达数（Expression / Destiny）：姓名全字母之和约化。
#[must_use]
pub fn expression(name: &str, system: System) -> u64 {
    reduce_with_master(string_sum(name, |c| letter_value(c, system)))
}

/// 灵魂数（Soul Urge）：姓名元音之和约化。
#[must_use]
pub fn soul_urge(name: &str, system: System) -> u64 {
    reduce_with_master(string_sum(name, |c| {
        if is_vowel(c) {
            letter_value(c, system)
        } else {
            None
        }
    }))
}

/// 人格数（Personality）：姓名辅音之和约化。
#[must_use]
pub fn personality(name: &str, system: System) -> u64 {
    reduce_with_master(string_sum(name, |c| {
        if c.is_ascii_alphabetic() && !is_vowel(c) {
            letter_value(c, system)
        } else {
            None
        }
    }))
}

/// 姓名数（某系统下的表达/灵魂/人格）。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct NameNumbers {
    /// 字母表系统。
    pub system: System,
    /// 表达数。
    pub expression: u64,
    /// 灵魂数。
    pub soul_urge: u64,
    /// 人格数。
    pub personality: u64,
}

/// 由姓名与系统算姓名数。
#[must_use]
pub fn name_numbers(name: &str, system: System) -> NameNumbers {
    NameNumbers {
        system,
        expression: expression(name, system),
        soul_urge: soul_urge(name, system),
        personality: personality(name, system),
    }
}

/// 一次数字学换算的结果。日期数恒有；姓名数在给出姓名时附上（两套系统）。
#[derive(Debug, Clone, Serialize)]
pub struct Cast {
    /// 生命灵数（按当前所选流派 `life_path_method`）。
    pub life_path: u64,
    /// 当前流派标识(`"component"` / `"whole_sum"`)。
    pub life_path_method: &'static str,
    /// 另一流派的生命灵数（对照）；Component 选中时此处 = WholeSum 值，反之亦然。
    pub life_path_alt: u64,
    /// 生日数。
    pub birthday: u64,
    /// 姓名数（Pythagorean），给出姓名时有。
    pub pythagorean: Option<NameNumbers>,
    /// 姓名数（Chaldean），给出姓名时有。
    pub chaldean: Option<NameNumbers>,
}

fn life_path_pair(year: i64, month: u32, day: u32, method: LifePathMethod) -> (u64, u64, &'static str) {
    let main = life_path_with(year, month, day, method);
    let (alt_method, id) = match method {
        LifePathMethod::Component => (LifePathMethod::WholeSum, "component"),
        LifePathMethod::WholeSum => (LifePathMethod::Component, "whole_sum"),
    };
    (main, life_path_with(year, month, day, alt_method), id)
}

/// 在共享上下文 [`Moment`] 上算日期数字学（不含姓名，默认 Component 流派）。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    compute_at_with(m, LifePathMethod::Component)
}

/// 在共享上下文 [`Moment`] 上算日期数字学（指定流派，不含姓名）。
#[must_use]
pub fn compute_at_with(m: &Moment, method: LifePathMethod) -> Cast {
    let (lp, alt, id) = life_path_pair(i64::from(m.year), m.month, m.day, method);
    Cast {
        life_path: lp,
        life_path_method: id,
        life_path_alt: alt,
        birthday: birthday_number(m.day),
        pythagorean: None,
        chaldean: None,
    }
}

/// 在共享上下文上算日期 + 姓名数字学（默认 Component；姓名两套字母表并出）。
#[must_use]
pub fn compute_named(m: &Moment, name: &str) -> Cast {
    compute_named_with(m, name, LifePathMethod::Component)
}

/// 在共享上下文上算日期 + 姓名数字学（指定生命灵数流派；姓名两套字母表并出）。
#[must_use]
pub fn compute_named_with(m: &Moment, name: &str, method: LifePathMethod) -> Cast {
    let (lp, alt, id) = life_path_pair(i64::from(m.year), m.month, m.day, method);
    Cast {
        life_path: lp,
        life_path_method: id,
        life_path_alt: alt,
        birthday: birthday_number(m.day),
        pythagorean: Some(name_numbers(name, System::Pythagorean)),
        chaldean: Some(name_numbers(name, System::Chaldean)),
    }
}

/// 由本地民用日期算（独立入口）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, 12, 0, tz))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaldean_table_groups() {
        // 1：AIJQY 2：BKR 3：CGLS 4：DMT 5：EHNX 6：UVW 7：OZ 8：FP；无 9。
        for c in ['A', 'I', 'J', 'Q', 'Y'] {
            assert_eq!(chaldean(c), Some(1));
        }
        assert_eq!(chaldean('B'), Some(2));
        assert_eq!(chaldean('F'), Some(8));
        assert_eq!(chaldean('O'), Some(7));
        assert_eq!(chaldean('Z'), Some(7));
        assert_eq!(chaldean('5'), None);
        assert_eq!(chaldean('a'), Some(1)); // 大小写一致
        // Chaldean 永不产出 9。
        assert!(('A'..='Z').all(|c| chaldean(c) != Some(9)));
    }

    #[test]
    fn pythagorean_via_ringhash() {
        assert_eq!(letter_value('A', System::Pythagorean), Some(1));
        assert_eq!(letter_value('I', System::Pythagorean), Some(9));
        assert_eq!(letter_value('J', System::Pythagorean), Some(1));
        assert_eq!(letter_value('Z', System::Pythagorean), Some(8));
    }

    #[test]
    fn life_path_worked_examples() {
        // 1990-06-15：年 1990→1+9+9+0=19→10→1；月 6；日 15→6；1+6+6=13→4。
        assert_eq!(life_path(1990, 6, 15), 4);
        // 流派对比：1990-06-15 → component 法 4，whole_sum 法 1+9+9+0+6+1+5=31→4（同值）。
        assert_eq!(life_path_with(1990, 6, 15, LifePathMethod::WholeSum), 4);
        // 1989-12-26 区分两派：
        //   Component: y=1+9+8+9=27→9, m=12→3, d=26→8 → 9+3+8=20→2
        //   WholeSum：  1+9+8+9+1+2+2+6=38→11（保留主数）
        assert_eq!(life_path_with(1989, 12, 26, LifePathMethod::Component), 2);
        assert_eq!(life_path_with(1989, 12, 26, LifePathMethod::WholeSum), 11);
        // 主数保留：某日期约化得 11 应停。2000-11-29：年2000→2，月11→11（停），日29→11（停）；2+11+11=24→6。
        assert_eq!(life_path(2000, 11, 29), 6);
        // 直接给出主数和示例：reduce_with_master 在求和处保留。
        // 1998-08-13：年1998→1+9+9+8=27→9；月8；日13→4；9+8+4=21→3。
        assert_eq!(life_path(1998, 8, 13), 3);
        // 边界：年 0（数字和=0）不 panic：0+1+1=2。
        assert_eq!(life_path(0, 1, 1), 2);
    }

    #[test]
    fn birthday_number_reduces() {
        assert_eq!(birthday_number(15), 6);
        assert_eq!(birthday_number(29), 11); // 主数停
        assert_eq!(birthday_number(4), 4);
    }

    #[test]
    fn name_numbers_pythagorean() {
        // "ABE" Pythagorean：A1 B2 E5 → 8（表达）。元音 A，E=1+5=6（灵魂）。辅音 B=2（人格）。
        let n = name_numbers("ABE", System::Pythagorean);
        assert_eq!(n.expression, 8);
        assert_eq!(n.soul_urge, 6);
        assert_eq!(n.personality, 2);
        // 非字母被跳过。
        assert_eq!(expression("A-B-E", System::Pythagorean), 8);
    }

    #[test]
    fn name_numbers_chaldean_differs() {
        // "FOX" Chaldean：F8 O7 X5 = 20 → 2。Pythagorean：F6 O6 X6=18→9。两系统不同。
        assert_eq!(expression("FOX", System::Chaldean), 2);
        assert_eq!(expression("FOX", System::Pythagorean), 9);
    }

    #[test]
    fn vowels_and_master_preserved() {
        assert!(is_vowel('a') && is_vowel('U'));
        assert!(!is_vowel('Y') && !is_vowel('B'));
        // 约化保留主数：构造和为 29 的名 → 表达数 11。
        // "K" =2(P).. 取一个和=29 的串：用 "INNN"？ I9 N5 N5 N5=24. 用 "RRR..." 略，直接验 reduce。
        assert_eq!(reduce_with_master(29), 11);
    }

    #[test]
    fn compute_paths() {
        let c = compute(1990, 6, 15, 8.0);
        assert_eq!(c.life_path, 4);
        assert!(c.pythagorean.is_none());
        let m = Moment::new(1990, 6, 15, 12, 0, 8.0);
        let cn = compute_named(&m, "Ada");
        assert_eq!(cn.life_path, 4);
        assert!(cn.pythagorean.is_some() && cn.chaldean.is_some());
        // 两系统对同名给不同表达数（除非碰巧相等）。
        let p = cn.pythagorean.unwrap();
        let ch = cn.chaldean.unwrap();
        assert_eq!(p.system, System::Pythagorean);
        assert_eq!(ch.system, System::Chaldean);
        // 确定性。
        assert_eq!(expression("Ada", System::Pythagorean), p.expression);
    }
}
