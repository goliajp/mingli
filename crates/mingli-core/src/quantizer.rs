//! S2 圆 S¹ 角度 → Z_n 分段（家族 B 的代数石）。
//!
//! 把一个连续角度（来自 L1 星历的黄经，或任何 [0，360) 量）量化进 n 个桶。
//! 西洋占星/Jyotish/七政四余共用：唯一差别在喂进来的角度（回归 vs 恒星黄经）。

/// 角度归一到 `[0,360)`。
#[inline]
#[must_use]
pub fn norm360(deg: f64) -> f64 {
    deg.rem_euclid(360.0)
}

/// 角度差归一到 `(-180, 180]`（用于求两角的有向最短差，如黄经收敛迭代）。
#[inline]
#[must_use]
pub fn norm180(deg: f64) -> f64 {
    let a = deg.rem_euclid(360.0);
    if a > 180.0 {
        a - 360.0
    } else {
        a
    }
}

/// 等分量化：360° 分 `n` 桶，返回桶号 `0..n`。
/// 例：`sector(λ,12)` → 星座；`sector(λ,24)` → 节气见下。
#[inline]
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "floor 后的值恒在 [0，n) 内，截断与符号丢失不会发生"
)]
pub fn sector(deg: f64, n: u32) -> u32 {
    let w = 360.0 / f64::from(n);
    (norm360(deg) / w).floor() as u32 % n
}

/// 桶内剩余角度（如「星座内度数」）。
#[inline]
#[must_use]
pub fn within(deg: f64, n: u32) -> f64 {
    let w = 360.0 / f64::from(n);
    norm360(deg).rem_euclid(w)
}

/// 二十四节气序号：太阳黄经每 15° 一个，0°=春分（序 0）。返回 `0..24`。
#[inline]
#[must_use]
pub fn solar_term_index(sun_longitude: f64) -> u32 {
    sector(sun_longitude, 24)
}

/// 不等分量化：给定升序边界 `bounds`（如占星 terms 每宫 5 段），返回落入的段号。
/// `bounds` 为各段**起点**（首段起点应为 0），长度 = 段数。
#[must_use]
pub fn unequal_sector(x: f64, span: f64, bounds: &[f64]) -> usize {
    let v = x.rem_euclid(span);
    let mut idx = 0;
    for (i, &b) in bounds.iter().enumerate() {
        if v >= b {
            idx = i;
        } else {
            break;
        }
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_ranges() {
        assert!((norm360(370.0) - 10.0).abs() < 1e-9);
        assert!((norm360(-10.0) - 350.0).abs() < 1e-9);
        assert!((norm180(190.0) + 170.0).abs() < 1e-9);
        assert!((norm180(180.0) - 180.0).abs() < 1e-9);
        assert!((norm180(-10.0) + 10.0).abs() < 1e-9);
    }

    #[test]
    fn zodiac_sectors() {
        assert_eq!(sector(0.0, 12), 0); // 0°白羊
        assert_eq!(sector(35.0, 12), 1); // 金牛
        assert_eq!(sector(359.9, 12), 11); // 双鱼
        assert!((within(35.0, 12) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn solar_terms() {
        assert_eq!(solar_term_index(0.0), 0); // 春分
        assert_eq!(solar_term_index(90.0), 6); // 夏至（第6个15°）
        assert_eq!(solar_term_index(315.0), 21); // 立春(315/15=21)
        assert_eq!(solar_term_index(360.0), 0); // wrap
    }

    #[test]
    fn egyptian_terms_of_aries() {
        // 白羊界：木0–6 金6–12 水12–20 火20–25 土25–30（起点）。
        let bounds = [0.0, 6.0, 12.0, 20.0, 25.0];
        assert_eq!(unequal_sector(3.0, 30.0, &bounds), 0); // 木
        assert_eq!(unequal_sector(15.0, 30.0, &bounds), 2); // 水
        assert_eq!(unequal_sector(28.0, 30.0, &bounds), 4); // 土
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn prop_norm360_in_range(d in -1e7f64..1e7) {
            prop_assert!((0.0..360.0).contains(&norm360(d)));
        }
        #[test]
        fn prop_norm180_in_range(d in -1e7f64..1e7) {
            let n = norm180(d); // 值域 (-180， 180]
            prop_assert!(n > -180.0 && n <= 180.0);
        }
        #[test]
        fn prop_sector_in_range(d in -1e7f64..1e7, n in 1u32..360) {
            prop_assert!(sector(d, n) < n);
        }
        #[test]
        fn prop_within_below_sector_width(d in -1e7f64..1e7, n in 1u32..360) {
            prop_assert!(within(d, n) < 360.0 / f64::from(n) + 1e-9);
        }
    }
}
