//! S4 字符串 → 环求和 + 数字根（家族 D 的代数石）。
//!
//! 命理数字学/姓名学的统一骨架：符号（字母/汉字）→数值（查表 φ）→求和（幺半群同态 Σ*→ℤ）
//! →数字根（mod 9 变体） 或 mod N 约化。
//!
//! 骨架在此、**表在叶**：字母值表、主数例外、mod-80 归一这些随流派而变的约定，
//! 各自住在用到它的叶里（`mingli-numerology` / `mingli-wuge`），本模块只留纯代数。

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
    // 至多二十位：u64 的十进制位数上限。从前是 `while n > 0` 靠 `n /= 10` 收敛，
    // 把它改成 `%=` 或把比较取反，循环就不动了（变异扫描三个超时出自这里）。
    for _ in 0..20 {
        if n == 0 {
            break;
        }
        s += n % 10;
        n /= 10;
    }
    s
}

/// 幺半群同态：字符串经 `value` 映射求和。`value` 返回 `None` 的字符跳过。
#[must_use]
pub fn string_sum(s: &str, value: impl Fn(char) -> Option<u64>) -> u64 {
    s.chars().filter_map(value).sum()
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
    fn string_sum_skips_unmapped_and_is_additive() {
        // value 返回 None 的字符跳过；同态：Σ(a·b) = Σa + Σb
        let v = |c: char| c.to_digit(10).map(u64::from);
        assert_eq!(string_sum("a1b2c3", v), 6);
        assert_eq!(string_sum("", v), 0);
        assert_eq!(string_sum("12", v) + string_sum("34", v), string_sum("1234", v));
    }

    #[test]
    fn sum_digits_and_zero() {
        assert_eq!(sum_digits(0), 0);
        assert_eq!(sum_digits(999), 27);
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
        fn prop_sum_digits_never_exceeds_input(n in 1..u64::MAX) {
            // 各位和 ≤ 原数（十进制展开的基本性质），且同余 mod 9
            prop_assert!(sum_digits(n) <= n);
            prop_assert_eq!(sum_digits(n) % 9, n % 9);
        }
    }
}
