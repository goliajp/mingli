//! S5 有限集上的群作用 / 置换（横切 ⟂ 的代数石）。
//!
//! 统一三种"旋转/布置"机制：紫白飞星（Z₉ 的 ±k 作用）、紫微安星/月将加时（Z₁₂ 位移）、
//! 以及抽样族的无放回置换（Fisher-Yates，见 [`crate::sampler`]）。

/// 循环群 `Z_n` 上的位移：`(start + k) mod n`（forward）或 `(start − k) mod n`（backward）。
#[inline]
#[must_use]
pub fn shift(start: i64, k: i64, n: i64, forward: bool) -> i64 {
    if forward {
        (start + k).rem_euclid(n)
    } else {
        (start - k).rem_euclid(n)
    }
}

/// 紫白飞星：洛书数 1..9 的阳顺阴逆飞布。给入中数 `center` 与步数 `k`，返回该步落数 1..9。
/// 顺飞 = 数字递增、逆飞 = 递减（同一洛书路径）。
#[inline]
#[must_use]
pub fn flying_star(center: i64, k: i64, forward: bool) -> i64 {
    shift(center - 1, k, 9, forward) + 1
}

/// 把 1-based 的「从某宫起、数到第 m 步」化为地支宫位（0-based）。
/// 用于紫微「寅起正月顺数生月、再逆数生时」等掐指类。
#[inline]
#[must_use]
pub fn count_to(start: i64, steps: i64, n: i64, forward: bool) -> i64 {
    shift(start, steps, n, forward)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flying_star_wraps() {
        // 八运 8 入中顺飞：8→9→1→2…
        assert_eq!(flying_star(8, 0, true), 8);
        assert_eq!(flying_star(8, 1, true), 9);
        assert_eq!(flying_star(8, 2, true), 1);
        assert_eq!(flying_star(8, 3, true), 2);
        // 逆飞：8→7→6…
        assert_eq!(flying_star(8, 1, false), 7);
    }

    #[test]
    fn ziwei_ming_palace_shift() {
        // 紫微命宫：寅(2)起正月顺数至 m 月、再逆数生时 hb（子=0 约定）。
        // m=5（五月）， hb=未(7) → 命宫应在亥(11)。
        let after_month = count_to(2, 5 - 1, 12, true); // 顺数生月
        let ming = count_to(after_month, 7, 12, false); // 逆数生时
        assert_eq!(ming, 11); // 亥
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn prop_shift_forward_then_back_is_identity(
            start in 0i64..1000, k in 0i64..1000, n in 1i64..100,
        ) {
            let f = shift(start, k, n, true);
            prop_assert!(f >= 0 && f < n);
            prop_assert_eq!(shift(f, k, n, false), start.rem_euclid(n));
        }
        #[test]
        fn prop_flying_star_in_1_9(center in 1i64..10, k in 0i64..1000, fwd in any::<bool>()) {
            prop_assert!((1..=9).contains(&flying_star(center, k, fwd)));
        }
        #[test]
        fn prop_count_to_in_range(
            start in -100i64..100, steps in -100i64..100, n in 1i64..50, fwd in any::<bool>(),
        ) {
            let c = count_to(start, steps, n, fwd);
            prop_assert!(c >= 0 && c < n);
        }
    }
}
