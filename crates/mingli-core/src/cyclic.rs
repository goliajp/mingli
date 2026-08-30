//! S1 循环群 + 中国剩余定理（家族 A 的代数石）。
//!
//! 核心事实：`Z_m × Z_n ≅ Z_mn` 当且仅当 `gcd(m,n)=1`。否则同时取模的像是阶为
//! `lcm(m,n)` 的**对角子群**，可达组合数 = `lcm`（< `m·n`）。
//! 这是干支(10×12→60≠120)、玛雅 Tzolkʼin(13×20→260)、藏历(5×12→60) 的统一解释。

/// 最大公约数（欧几里得），取绝对值。
#[must_use]
pub fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// 最小公倍数；任一为 0 则返回 0。
///
/// 契约：调用方保证 `lcm(a,b)` 不溢出 `i64`（命理域模数均很小，如 10/12/13/20）。
/// 对极大互质模数会溢出——这是显式的域约束，非静默兜底。[`cycle_period`] 同此约束。
#[must_use]
pub fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        0
    } else {
        (a / gcd(a, b)) * b
    }
}

/// 扩展欧几里得：返回 (g， x， y) 使 a·x + b·y = g = gcd(a，b)。
fn ext_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (g, x, y) = ext_gcd(b, a % b);
        (g, y, x - (a / b) * y)
    }
}

/// 合并两个同余式 x≡r1(mod m1)， x≡r2(mod m2)。
/// 相容则返回 (r， lcm)；不相容（对角子群外）返回 None。
fn crt_pair(r1: i64, m1: i64, r2: i64, m2: i64) -> Option<(i64, i64)> {
    let (g, p, _) = ext_gcd(m1, m2);
    if (r2 - r1).rem_euclid(g) != 0 {
        return None; // 不在对角子群上（如干支里异阴阳的干支组合）
    }
    let l = m1 / g * m2;
    let mul = (r2 - r1) / g;
    // `step` 取 `m2 / g` 的任何倍数都算出同一个 x：下一行的 x 是 `r1 + m1 * k`，
    // 而它最终只按 `m1 * m2 / g` 取模，所以只有 `k mod (m2 / g)` 进得了结果。
    // 同理 `l` 放大成任何倍数也无所谓——x 构造出来就已经小于 `m1 * m2 / g`。
    // 变异测试会把这两处报成「没被拦住」，那是等价变异，不是缺口。
    let step = m2 / g;
    let x = r1 + m1 * (p.rem_euclid(step) * mul.rem_euclid(step)).rem_euclid(step);
    Some((x.rem_euclid(l), l))
}

/// 合并一组同余式 `(residue, modulus)`。相容返回 `Some(x mod lcm)`，否则 `None`。
/// 例：干支 `[(stem,10),(branch,12)]`；玛雅 `[(num,13),(name,20)]`。
#[must_use]
pub fn crt_combine(congruences: &[(i64, i64)]) -> Option<i64> {
    let mut acc = (0i64, 1i64); // x≡0 (mod 1)
    for &(r, m) in congruences {
        acc = crt_pair(acc.0, acc.1, r.rem_euclid(m), m)?;
    }
    Some(acc.0)
}

/// 多个轮联合循环的周期 = 各模数的 lcm。
#[must_use]
pub fn cycle_period(moduli: &[i64]) -> i64 {
    moduli.iter().fold(1, |a, &m| lcm(a, m))
}

/// 由 `n` 个并行轮（各长 `moduli[i]`），给定起点天数 `day`，返回各轮的相位（0-based）。
/// 这是「巴厘 Pawukon 多并行週」「干支日」的统一形态。
#[must_use]
pub fn parallel_phases(day: i64, moduli: &[i64]) -> Vec<i64> {
    moduli.iter().map(|&m| day.rem_euclid(m)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn reachable(m: i64, n: i64) -> usize {
        let mut set = HashSet::new();
        for a in 0..m {
            for b in 0..n {
                if let Some(x) = crt_combine(&[(a, m), (b, n)]) {
                    set.insert(x);
                }
            }
        }
        set.len()
    }

    #[test]
    fn ganzhi_is_diagonal_subgroup_not_product() {
        // 干支：10×12，gcd=2 → 可达 60 个组合（同阴阳），不是 120。
        assert_eq!(gcd(10, 12), 2);
        assert_eq!(cycle_period(&[10, 12]), 60);
        assert_eq!(reachable(10, 12), 60, "干支应为 60 个对角子群元素，非 120");
        // 甲子=0
        assert_eq!(crt_combine(&[(0, 10), (0, 12)]), Some(0));
        // 甲（0，阳）+丑（1，阴）：异阴阳 → 不可达
        assert!(crt_combine(&[(0, 10), (1, 12)]).is_none());
    }

    #[test]
    fn maya_tzolkin_is_clean_crt() {
        // 玛雅 13×20 互质 → 完整乘积，260 个组合全可达。
        assert_eq!(gcd(13, 20), 1);
        assert_eq!(cycle_period(&[13, 20]), 260);
        assert_eq!(reachable(13, 20), 260);
    }

    #[test]
    fn tibetan_5x12_cleaner_than_ganzhi() {
        // 藏历 5 元素 × 12 生肖，互质 → 完整 Z₅×Z₁₂≅Z₆₀（比干支干净）。
        assert_eq!(gcd(5, 12), 1);
        assert_eq!(reachable(5, 12), 60, "藏历 5×12 应 60 个全可达");
    }

    #[test]
    fn crt_recovers_residues() {
        // 任意可达组合：CRT 出的 x 应还原各分量余数。
        let x = crt_combine(&[(3, 13), (7, 20)]).unwrap();
        assert_eq!(x % 13, 3);
        assert_eq!(x % 20, 7);
    }

    /// 还原校验只走过互质的 13×20；而这个仓库真正天天用的是 gcd=2 的 10×12。
    /// 合并式在 `g > 1` 时要靠 `m2 / g` 定步长，互质时 `g = 1` 让这一步看不出对错。
    /// 这里把六十甲子每一个可达组合都还原一遍。
    #[test]
    fn crt_recovers_residues_on_the_non_coprime_pair_too() {
        let mut reached = 0;
        for stem in 0..10i64 {
            for branch in 0..12i64 {
                let Some(x) = crt_combine(&[(stem, 10), (branch, 12)]) else {
                    // 异阴阳的组合本就不在对角子群上。
                    assert_ne!(stem % 2, branch % 2, "干{stem} 支{branch} 同阴阳却不可达");
                    continue;
                };
                assert_eq!(stem % 2, branch % 2, "干{stem} 支{branch} 异阴阳却可达");
                assert!((0..60).contains(&x), "干{stem} 支{branch} 落在 0..60 之外：{x}");
                assert_eq!(x.rem_euclid(10), stem, "干{stem} 支{branch} 还原不出干");
                assert_eq!(x.rem_euclid(12), branch, "干{stem} 支{branch} 还原不出支");
                reached += 1;
            }
        }
        assert_eq!(reached, 60);
    }

    #[test]
    fn gcd_lcm_edges() {
        assert_eq!(gcd(-12, 8), 4); // 取绝对值
        assert_eq!(lcm(0, 5), 0); // 任一为 0
        assert_eq!(lcm(4, 6), 12);
    }

    #[test]
    fn crt_inconsistent_returns_none() {
        // x≡0(mod4)， x≡1(mod6)：gcd(4，6)=2 不整除 (1-0) → 无解
        assert!(crt_combine(&[(0, 4), (1, 6)]).is_none());
    }

    #[test]
    fn pawukon_parallel_phases() {
        // Pawukon 核心互质週 2·3·5·7，联合周期 210。
        assert_eq!(cycle_period(&[2, 3, 5, 7]), 210);
        let p = parallel_phases(211, &[2, 3, 5, 7]);
        assert_eq!(p, vec![1, 1, 1, 1]); // 第 211 天 = 第 1 天的相位（210 循环）
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn prop_gcd_divides_both(a in 1i64..1_000_000, b in 1i64..1_000_000) {
            let g = gcd(a, b);
            prop_assert!(g >= 1);
            prop_assert_eq!(a % g, 0);
            prop_assert_eq!(b % g, 0);
        }
        #[test]
        fn prop_gcd_lcm_product(a in 1i64..100_000, b in 1i64..100_000) {
            // gcd·lcm = a·b（输入有界，乘积不溢出 i64）。
            prop_assert_eq!(gcd(a, b) * lcm(a, b), a * b);
        }
        #[test]
        fn prop_crt_satisfies_all_congruences(
            v in 0i64..1_000_000,
            moduli in prop::collection::vec(2i64..50, 1..4),
        ) {
            // 由公共值 v 造一致同余组，合并解须满足每条同余。
            let cong: Vec<(i64, i64)> = moduli.iter().map(|&m| (v % m, m)).collect();
            let r = crt_combine(&cong).expect("一致同余必有解");
            for &(_, m) in &cong {
                prop_assert_eq!(r.rem_euclid(m), v.rem_euclid(m));
            }
        }
        #[test]
        fn prop_parallel_phases_in_range(
            day in any::<i64>(),
            moduli in prop::collection::vec(1i64..100, 1..6),
        ) {
            for (p, &m) in parallel_phases(day, &moduli).iter().zip(&moduli) {
                prop_assert!(*p >= 0 && *p < m);
            }
        }
        #[test]
        fn prop_cycle_period_pair_is_lcm(a in 1i64..10_000, b in 1i64..10_000) {
            prop_assert_eq!(cycle_period(&[a, b]), lcm(a, b));
        }
    }
}
