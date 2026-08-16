//! L3 叶（C 族）：马达加斯加 Sikidy。
//!
//! 由可复现种子（[`mingli_core::sampler`]）随机起 **4 个母列**（各 4 位），经 [`mingli_core::gf2`]
//! 的转置与逐列 XOR 树生成 **16 列**。第 15 列 C15（创世者，0-based idx 14）= GF(2) 线性组合，
//! **恒为偶**——与地占「法官恒偶」是同一条 GF(2) 奇偶校验定理（Ascher 联系 Hamming 1948 纠错码），
//! 穷举证明在 `mingli_core::gf2`。
//!
//! 语域注：本叶只做 16 列的**GF(2) 矩阵结构**。各列/图的马达加斯加名与吉凶象征属需逐项核对文献的
//! 数据表（🟡），不在此凭记忆硬编；列以 4 位整数值（0..16）标识。


mod engine;
pub use engine::SikidyEngine;

use mingli_core::gf2;
use mingli_core::sampler::SplitMix64;
use serde::Serialize;

/// 一盘 Sikidy：16 列（各 4 位，0..16），C15 为创世者列。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Reading {
    /// 4 母列（随机起）。
    pub mothers: [u8; 4],
    /// 全 16 列 C1..C16（顺序 0-based）。
    pub columns: [u8; 16],
    /// 创世者列 C15（idx 14）；**恒为偶**。
    pub seer: u8,
    /// C15 点数奇偶（恒为 `true`）。
    pub seer_even: bool,
}

/// 由种子起 4 母列并生成 16 列（同种子 → 同盘，可复现）。
#[must_use]
pub fn cast(seed: u64) -> Reading {
    let mut rng = SplitMix64::new(seed);
    let mothers: [gf2::Figure; 4] =
        std::array::from_fn(|_| u16::try_from(rng.below(16)).unwrap_or(0));
    from_mothers(mothers)
}

/// 由给定 4 母列生成 16 列（与随机起卦同一推导，便于校验/复算）。
#[must_use]
pub fn from_mothers(mothers: [gf2::Figure; 4]) -> Reading {
    let c = gf2::sikidy_columns(mothers);
    let cols = c.map(|f| (f & 0xF) as u8);
    Reading {
        mothers: cols[0..4].try_into().unwrap_or([0; 4]),
        columns: cols,
        seer: cols[14],
        seer_even: gf2::is_even(c[14]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_given_seed() {
        assert_eq!(cast(2024).columns, cast(2024).columns);
        assert_ne!(cast(1).mothers, cast(2).mothers);
    }

    #[test]
    fn seer_c15_always_even_over_seeds() {
        for seed in 0..500u64 {
            let r = cast(seed);
            assert!(r.seer_even, "seed {seed} C15 应为偶");
            assert!(r.seer.count_ones().is_multiple_of(2));
        }
    }

    #[test]
    fn matches_core_columns() {
        let mothers = [0b1011u16, 0b0110, 0b1110, 0b0001];
        let r = from_mothers(mothers);
        let c = gf2::sikidy_columns(mothers);
        assert_eq!(u16::from(r.seer), c[14]);
        // 头 4 列 = 母列。
        for (got, &want) in r.mothers.iter().zip(mothers.iter()) {
            assert_eq!(u16::from(*got), want);
        }
        // C15 = C13 XOR C14（生成树最后一步）。
        assert_eq!(r.columns[14], r.columns[12] ^ r.columns[13]);
    }

    #[test]
    fn all_columns_in_range() {
        let r = cast(9);
        assert!(r.columns.iter().all(|&f| f < 16));
    }
}
