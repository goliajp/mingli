//! L3 叶（C 族）：地占 ʿilm al-raml（阿拉伯/欧洲传统的盾牌图）。
//!
//! 由可复现种子（[`mingli_core::sampler`]）随机起 **4 个母图**（每图 4 行、每行单/双点 = 1 位），
//! 经 [`mingli_core::gf2`] 的转置与 XOR 推出全部派生图：4 女图（母块转置）、4 侄图（成对 XOR）、
//! 2 见证、1 **法官**。法官 = 两见证 XOR，**恒为偶图**——这是 GF(2) 奇偶校验定理（每个母位
//! 经转置被计入偶数次而 mod-2 成对抵消），其穷举证明在 `mingli_core::gf2`。
//!
//! 语域注：本叶只做盾牌图的**GF(2) 结构**。16 个地占图的拉丁名/中文名（Via， Populus…）是一张
//! 需逐项核对权威文献的数据表（🟡），不在此凭记忆硬编；图以 4 位整数值（0..16）标识。

use mingli_core::gf2;
use mingli_core::sampler::SplitMix64;
use serde::Serialize;

/// 一张地占盘：4 母图推出的全部图，皆以 4 位整数值（0..16）表示，低位 = 第一行。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Reading {
    /// 4 母图（随机起）。
    pub mothers: [u8; 4],
    /// 4 女图 = 母块转置。
    pub daughters: [u8; 4],
    /// 4 侄图 = 母/女成对 XOR。
    pub nieces: [u8; 4],
    /// 2 见证 `[右, 左]` = 侄成对 XOR。
    pub witnesses: [u8; 2],
    /// 法官 = 两见证 XOR；**恒为偶图**。
    pub judge: u8,
    /// 法官点数奇偶（恒为 `true`，对每盘成立——结构上的纠错不变量）。
    pub judge_even: bool,
}

/// 由种子起 4 母图并推全盘（同种子 → 同盘，可复现）。
#[must_use]
pub fn cast(seed: u64) -> Reading {
    let mut rng = SplitMix64::new(seed);
    // 每母图 4 行，每行一位（单点=1/双点=0 的约定不影响代数）。
    let mothers: [gf2::Figure; 4] =
        std::array::from_fn(|_| u16::try_from(rng.below(16)).unwrap_or(0));
    from_mothers(mothers)
}

/// 由给定 4 母图推全盘（与随机起卦同一推导，便于校验/复算）。
#[must_use]
pub fn from_mothers(mothers: [gf2::Figure; 4]) -> Reading {
    let s = gf2::geomancy_shield(mothers);
    let m = |f: gf2::Figure| (f & 0xF) as u8;
    Reading {
        mothers: s.mothers.map(m),
        daughters: s.daughters.map(m),
        nieces: s.nieces.map(m),
        witnesses: s.witnesses.map(m),
        judge: m(s.judge),
        judge_even: gf2::is_even(s.judge),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_given_seed() {
        assert_eq!(cast(2024).judge, cast(2024).judge);
        assert_eq!(cast(2024).mothers, cast(2024).mothers);
        // 不同种子大概率给出不同母图。
        assert_ne!(cast(1).mothers, cast(2).mothers);
    }

    #[test]
    fn judge_always_even_over_seeds() {
        // GF(2) 奇偶校验定理在起卦路径上的体现（穷举证明在 core::gf2）。
        for seed in 0..500u64 {
            let r = cast(seed);
            assert!(r.judge_even, "seed {seed} 法官应为偶图");
            assert!(r.judge.count_ones().is_multiple_of(2));
        }
    }

    #[test]
    fn matches_core_shield_derivation() {
        // 叶的推导 = core::gf2 的盾牌推导（无额外语义偏移）。
        let mothers = [0b1011u16, 0b0110, 0b1110, 0b0001];
        let r = from_mothers(mothers);
        let s = gf2::geomancy_shield(mothers);
        assert_eq!(u16::from(r.daughters[0]), s.daughters[0]);
        assert_eq!(u16::from(r.witnesses[0]), s.witnesses[0]);
        assert_eq!(u16::from(r.judge), s.judge);
        // 见证→法官关系：judge = wr XOR wl。
        assert_eq!(r.judge, r.witnesses[0] ^ r.witnesses[1]);
    }

    #[test]
    fn all_figures_in_range() {
        let r = cast(7);
        for f in r
            .mothers
            .iter()
            .chain(&r.daughters)
            .chain(&r.nieces)
            .chain(&r.witnesses)
            .chain(std::iter::once(&r.judge))
        {
            assert!(*f < 16, "图值越界： {f}");
        }
    }
}
