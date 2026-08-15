//! S4 字符串 → 环求和 + 数字根（家族 D 的代数石）。
//!
//! 命理数字学/姓名学的统一骨架：符号（字母/汉字）→数值（查表 φ）→求和（幺半群同态 Σ*→ℤ）
//! →数字根（mod 9 变体） 或 mod N 约化。

/// 数字根：反复加各位直到一位（≈ `mod 9`，但 0→0、9→9）。
#[must_use]
pub fn digital_root(n: u64) -> u64 {
    if n == 0 {
        0
    } else {
        1 + (n - 1) % 9
    }
}

/// 各位数字之和。
#[must_use]
pub fn sum_digits(mut n: u64) -> u64 {
    let mut s = 0;
    while n > 0 {
        s += n % 10;
        n /= 10;
    }
    s
}

/// 带主数例外的约化（西洋数字学）：遇 11/22/33 停。
#[must_use]
pub fn reduce_with_master(n: u64) -> u64 {
    let mut x = n;
    loop {
        if matches!(x, 11 | 22 | 33) {
            return x;
        }
        if x < 10 {
            return x;
        }
        x = sum_digits(x);
    }
}

/// 幺半群同态：字符串经 `value` 映射求和。`value` 返回 `None` 的字符跳过。
#[must_use]
pub fn string_sum(s: &str, value: impl Fn(char) -> Option<u64>) -> u64 {
    s.chars().filter_map(value).sum()
}

/// Pythagorean 字母值：A=1..I=9， J=1..（仅 A-Z / a-z）。
#[must_use]
pub fn pythagorean(c: char) -> Option<u64> {
    let u = c.to_ascii_uppercase();
    if u.is_ascii_uppercase() {
        Some(((u as u64 - 'A' as u64) % 9) + 1)
    } else {
        None
    }
}

/// 五格 `mod-80` 归一到 `1..=81`（一种通行约定，属 🟡 欠定——版本有别，须配置）。
#[must_use]
pub fn fold_81(n: u64) -> u64 {
    if n == 0 {
        0
    } else {
        ((n - 1) % 80) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digital_roots() {
        assert_eq!(digital_root(0), 0);
        assert_eq!(digital_root(9), 9);
        assert_eq!(digital_root(18), 9);
        assert_eq!(digital_root(31), 4); // 1980-06-15 各位和=31→4
    }

    #[test]
    fn master_numbers_preserved() {
        assert_eq!(reduce_with_master(29), 11); // 2+9=11，停
        assert_eq!(reduce_with_master(38), 11); // 3+8=11
        assert_eq!(reduce_with_master(40), 4);
        assert_eq!(reduce_with_master(33), 33);
    }

    #[test]
    fn pythagorean_letters() {
        assert_eq!(pythagorean('A'), Some(1));
        assert_eq!(pythagorean('I'), Some(9));
        assert_eq!(pythagorean('J'), Some(1));
        assert_eq!(pythagorean('Z'), Some(8)); // (25%9)+1=8
        assert_eq!(pythagorean('5'), None);
        // "ABC" = 1+2+3
        assert_eq!(string_sum("ABC", pythagorean), 6);
    }

    #[test]
    fn five_grid_fold() {
        assert_eq!(fold_81(0), 0);
        assert_eq!(fold_81(1), 1);
        assert_eq!(fold_81(80), 80);
        assert_eq!(fold_81(81), 1);
        assert_eq!(fold_81(160), 80);
        assert_eq!(fold_81(161), 1);
    }

    #[test]
    fn sum_digits_and_zero() {
        assert_eq!(sum_digits(0), 0);
        assert_eq!(sum_digits(999), 27);
        assert_eq!(reduce_with_master(0), 0);
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn prop_digital_root_is_mod9_variant(n in any::<u64>()) {
            let dr = digital_root(n);
            if n == 0 {
                prop_assert_eq!(dr, 0);
            } else {
                prop_assert!((1..=9).contains(&dr));
                prop_assert_eq!(dr, 1 + (n - 1) % 9);
            }
        }
        #[test]
        fn prop_reduce_with_master_terminal(n in any::<u64>()) {
            // 结果要么是个位数，要么是主数 11/22/33。
            let r = reduce_with_master(n);
            prop_assert!(r < 10 || matches!(r, 11 | 22 | 33));
        }
        #[test]
        fn prop_fold81_in_range(n in any::<u64>()) {
            let f = fold_81(n);
            if n == 0 {
                prop_assert_eq!(f, 0);
            } else {
                prop_assert!((1..=81).contains(&f));
            }
        }
        #[test]
        fn prop_pythagorean_in_range(c in any::<char>()) {
            if let Some(v) = pythagorean(c) {
                prop_assert!((1..=9).contains(&v));
            }
        }
    }
}
