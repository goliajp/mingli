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

    /// SplitMix64 的输出向量——钉住具体的数，不只是「同种子同结果」。
    ///
    /// 本文件其余几条测试验的都是**自洽**：同种子给同结果、是个置换、不同种子不同、
    /// bit() 两种值都出现。这些性质有一个共同的盲区——**换一套算术它们照样全绿**。
    /// 而这个发生器是全仓「同一次取机可复核」那句承诺的根：易经的爻、地占的四母、
    /// 塔罗的洗牌、Ifá、Sikidy 全从它来。它的输出一旦改变，历史上每一张带取机的盘
    /// 都会静默变成另一张，且没有任何东西会红——契约快照也不行，
    /// 那是同一个二进制跑两遍自比，两边一起变。
    ///
    /// 两个独立来源给出同一组数：
    ///
    /// - Vigna 的参考实现 `prng.di.unimi.it/splitmix64.c`（Java 8 SplittableRandom
    ///   的定增量版本，doi:10.1145/2714064.2660195）。取回原文编译实跑，
    ///   非仅阅读——下面 seed 0 / 1 / 0xDEADBEEF 三组各前五个数与本实现逐位相同。
    /// - Rosetta Code《Pseudo-random numbers/Splitmix64》公布的期望输出：
    ///   seed 1234567 前五个数为 6457827717110365317 / 3203168211198807973 /
    ///   9817491932198370423 / 4593380528125082431 / 16408922859458223821。
    ///
    /// 两源均与本实现一致。改动这个函数就是改动一份对外承诺，不是重构。
    #[test]
    fn splitmix64_matches_the_published_reference_vectors() {
        let take5 = |seed: u64| -> [u64; 5] {
            let mut r = SplitMix64::new(seed);
            [r.next_u64(), r.next_u64(), r.next_u64(), r.next_u64(), r.next_u64()]
        };
        assert_eq!(
            take5(0),
            [
                0xE220_A839_7B1D_CDAF,
                0x6E78_9E6A_A1B9_65F4,
                0x06C4_5D18_8009_454F,
                0xF88B_B8A8_724C_81EC,
                0x1B39_896A_51A8_749B,
            ],
            "seed 0 的前五个输出与 Vigna 参考实现不符"
        );
        assert_eq!(
            take5(1),
            [
                0x910A_2DEC_8902_5CC1,
                0xBEEB_8DA1_658E_EC67,
                0xF893_A2EE_FB32_555E,
                0x71C1_8690_EE42_C90B,
                0x71BB_54D8_D101_B5B9,
            ],
            "seed 1 的前五个输出与 Vigna 参考实现不符"
        );
        assert_eq!(
            take5(0xDEAD_BEEF),
            [
                0x4ADF_B90F_68C9_EB9B,
                0xDE58_6A31_41A1_0922,
                0x021F_BC2F_8E1C_FC1D,
                0x7466_CE73_7BE1_6790,
                0x3BFA_8764_F685_BD1C,
            ],
            "seed 0xDEADBEEF 的前五个输出与 Vigna 参考实现不符"
        );
        // 第二个来源给的是十进制，照抄原样，不换算成十六进制再比——
        // 换算这一步本身就是可能出错的地方
        assert_eq!(
            take5(1_234_567),
            [
                6_457_827_717_110_365_317,
                3_203_168_211_198_807_973,
                9_817_491_932_198_370_423,
                4_593_380_528_125_082_431,
                16_408_922_859_458_223_821,
            ],
            "seed 1234567 的前五个输出与 Rosetta Code 公布的期望值不符"
        );
    }

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
