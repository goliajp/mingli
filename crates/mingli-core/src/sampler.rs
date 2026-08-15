//! S6 可审计随机种子 → 均匀抽样（家族 C 的随机源）。
//!
//! 占卜的「随机起卦」用**可复现的种子**驱动（反巴纳姆：种子留痕则同一占可复算）。
//! 提供 splitmix64 PRNG、独立二进制位（易经/地占起卦）、无放回洗牌（塔罗/卢恩抽牌）。

/// splitmix64：极简、确定性、可种子的 PRNG。
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// 以给定种子构造（同种子 → 同序列，可复现）。
    #[must_use]
    pub fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }
    /// 推进并返回下一个 64 位伪随机数。
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// `[0, n)` 上近似均匀整数。
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
    /// 一个均匀二进制位（家族 C：易经一爻、地占一行）。
    #[must_use]
    pub fn bit(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
}

/// 无放回均匀洗牌（Fisher-Yates）：返回 0..len 的一个置换，由种子决定、可复现。
/// 塔罗/卢恩"抽 k 张" = 取此置换前 k 个。
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    reason = "j ∈ [0，i] ≤ len ≤ usize::MAX，回写 usize 不会截断"
)]
pub fn shuffle(len: usize, seed: u64) -> Vec<usize> {
    let mut a: Vec<usize> = (0..len).collect();
    let mut rng = SplitMix64::new(seed);
    for i in (1..len).rev() {
        let j = rng.below(i as u64 + 1) as usize;
        a.swap(i, j);
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_given_seed() {
        assert_eq!(shuffle(78, 42), shuffle(78, 42)); // 同种子 → 同结果（可复现）
    }

    #[test]
    fn shuffle_is_permutation() {
        let mut s = shuffle(78, 12345); // 塔罗 78 张
        s.sort_unstable();
        assert_eq!(s, (0..78).collect::<Vec<_>>()); // 是 0..78 的置换（无放回）
    }

    #[test]
    fn different_seeds_differ() {
        assert_ne!(shuffle(78, 1), shuffle(78, 2));
    }

    #[test]
    fn bit_and_below_cover() {
        let mut rng = SplitMix64::new(7);
        // bit() 在足够多次内应同时产出 true 和 false。
        let mut seen_t = false;
        let mut seen_f = false;
        for _ in 0..64 {
            if rng.bit() {
                seen_t = true;
            } else {
                seen_f = true;
            }
        }
        assert!(seen_t && seen_f);
        // below(n) 恒在 [0，n)。
        let mut rng2 = SplitMix64::new(7);
        for _ in 0..100 {
            assert!(rng2.below(6) < 6);
        }
    }

    #[test]
    fn shuffle_len_one_and_zero() {
        assert_eq!(shuffle(1, 9), vec![0]);
        assert!(shuffle(0, 9).is_empty());
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn prop_splitmix_deterministic(seed in any::<u64>()) {
            let (mut a, mut b) = (SplitMix64::new(seed), SplitMix64::new(seed));
            for _ in 0..8 {
                prop_assert_eq!(a.next_u64(), b.next_u64());
            }
        }
        #[test]
        fn prop_below_in_range(seed in any::<u64>(), n in 1u64..1_000_000) {
            let mut r = SplitMix64::new(seed);
            for _ in 0..16 {
                prop_assert!(r.below(n) < n);
            }
        }
        #[test]
        fn prop_shuffle_is_permutation(len in 0usize..256, seed in any::<u64>()) {
            let mut p = shuffle(len, seed);
            prop_assert_eq!(p.len(), len);
            p.sort_unstable();
            prop_assert!(p.iter().copied().eq(0..len));
        }
    }
}
