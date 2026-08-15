//! S3 二进制格 (Z₂)^k + GF(2) 线性代数（家族 C 的代数石）。
//!
//! 覆盖易经卦(2⁶)、地占/Ifá 单图(2⁴)、Sikidy（GF(2) 矩阵）。核心运算：位↔索引、
//! XOR（GF(2) 加法）、转置、奇偶校验。本模块用穷举测试**证明**地占「法官恒偶」与
//! Sikidy「C15 恒偶」是同一条 GF(2) 奇偶校验定理。

/// 一个 figure / 卦象 = 低 k 位的位向量（用 u16 承载，k≤16）。
pub type Figure = u16;

/// GF(2) 加法（逐位 XOR）。
#[inline]
#[must_use]
pub fn xor(a: Figure, b: Figure) -> Figure {
    a ^ b
}

/// 总点数奇偶：true = 偶（even / 双点多），即置位数为偶。
#[inline]
#[must_use]
pub fn is_even(f: Figure) -> bool {
    f.count_ones().is_multiple_of(2)
}

/// 把 `k` 个二进制位（低位在前）打包成 [`Figure`]。
#[must_use]
pub fn pack(bits: &[bool]) -> Figure {
    bits.iter()
        .enumerate()
        .fold(0u16, |acc, (i, &b)| if b { acc | (1 << i) } else { acc })
}

/// 把 Figure 的低 `k` 位拆成位向量（低位在前）。
#[must_use]
pub fn unpack(f: Figure, k: usize) -> Vec<bool> {
    (0..k).map(|i| (f >> i) & 1 == 1).collect()
}

/// 把 4 个「母图」（各 4 位）按 4×4 矩阵转置成 4 个「女图」。
/// 女图 `d` 的第 `i` 位 = 母图 `i` 的第 `d` 位。地占 / Sikidy 共用此操作。
#[must_use]
pub fn transpose4(mothers: [Figure; 4]) -> [Figure; 4] {
    let mut out = [0u16; 4];
    for (d, slot) in out.iter_mut().enumerate() {
        for (i, &m) in mothers.iter().enumerate() {
            if (m >> d) & 1 == 1 {
                *slot |= 1 << i;
            }
        }
    }
    out
}

/// 地占 shield chart：由 4 母图推出全部派生图。
#[derive(Debug, Clone, Copy)]
pub struct Shield {
    /// 4 母图（随机起，每图 4 位）。
    pub mothers: [Figure; 4],
    /// 4 女图 = 母块转置。
    pub daughters: [Figure; 4],
    /// 4 侄图 = 母/女成对 XOR。
    pub nieces: [Figure; 4],
    /// 2 见证 `[右, 左]` = 侄成对 XOR。
    pub witnesses: [Figure; 2],
    /// 法官 = 两见证 XOR；**恒为偶图**（GF(2) 奇偶校验定理）。
    pub judge: Figure,
}

/// 由 4 母图推出完整 shield chart（女/侄/见证/法官）。
#[must_use]
pub fn geomancy_shield(mothers: [Figure; 4]) -> Shield {
    let d = transpose4(mothers);
    let n = [
        xor(mothers[0], mothers[1]),
        xor(mothers[2], mothers[3]),
        xor(d[0], d[1]),
        xor(d[2], d[3]),
    ];
    let wr = xor(n[0], n[1]);
    let wl = xor(n[2], n[3]);
    Shield {
        mothers,
        daughters: d,
        nieces: n,
        witnesses: [wr, wl],
        judge: xor(wr, wl),
    }
}

/// Sikidy：由 4 随机母列（各 4 位）生成 16 列，返回创世者列 C15(0-based idx 14)。
/// C5–C8 = 母块转置；C9–C16 = 逐列 XOR 树。
#[must_use]
pub fn sikidy_columns(mothers: [Figure; 4]) -> [Figure; 16] {
    let mut c = [0u16; 16];
    c[0..4].copy_from_slice(&mothers);
    let t = transpose4(mothers);
    c[4..8].copy_from_slice(&t);
    c[8] = xor(c[6], c[7]); // C9 = C7⊕C8
    c[9] = xor(c[4], c[5]); // C10 = C5⊕C6
    c[10] = xor(c[2], c[3]); // C11 = C3⊕C4
    c[11] = xor(c[0], c[1]); // C12 = C1⊕C2
    c[12] = xor(c[8], c[9]); // C13 = C9⊕C10
    c[13] = xor(c[10], c[11]); // C14 = C11⊕C12
    c[14] = xor(c[12], c[13]); // C15 = C13⊕C14（创世者）
    c[15] = xor(c[14], c[0]); // C16 = C15⊕C1
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn even_figures_count_is_eight() {
        // 16 个 4 位 figure 中，偶图恰 8 个（popcount 0/2/4）。
        let n = (0u16..16).filter(|&f| is_even(f)).count();
        assert_eq!(n, 8);
    }

    #[test]
    fn pack_unpack_roundtrip() {
        for f in 0u16..64 {
            assert_eq!(pack(&unpack(f, 6)), f);
        }
    }

    #[test]
    fn geomancy_judge_always_even() {
        // 穷举全部 16⁴ = 65536 种母图组合，法官恒为偶图（GF(2) 奇偶校验定理）。
        for combo in 0u32..(16 * 16 * 16 * 16) {
            let m = [
                (combo & 0xF) as u16,
                ((combo >> 4) & 0xF) as u16,
                ((combo >> 8) & 0xF) as u16,
                ((combo >> 12) & 0xF) as u16,
            ];
            let s = geomancy_shield(m);
            assert!(is_even(s.judge), "法官应恒为偶图， mothers={m:?}");
        }
    }

    #[test]
    fn sikidy_c15_always_even() {
        // 同一条定理在 Sikidy 上的体现：创世者列 C15 恒为偶。
        for combo in 0u32..(16 * 16 * 16 * 16) {
            let m = [
                (combo & 0xF) as u16,
                ((combo >> 4) & 0xF) as u16,
                ((combo >> 8) & 0xF) as u16,
                ((combo >> 12) & 0xF) as u16,
            ];
            let c = sikidy_columns(m);
            assert!(is_even(c[14]), "Sikidy C15 应恒为偶， mothers={m:?}");
        }
    }

    #[test]
    fn transpose_is_involution() {
        // 转置两次还原。
        for combo in [0x0123u32, 0xFEDC, 0xABCD] {
            let m = [
                (combo & 0xF) as u16,
                ((combo >> 4) & 0xF) as u16,
                ((combo >> 8) & 0xF) as u16,
                ((combo >> 12) & 0xF) as u16,
            ];
            assert_eq!(transpose4(transpose4(m)), m);
        }
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn prop_pack_unpack_roundtrip(bits in prop::collection::vec(any::<bool>(), 0..16)) {
            prop_assert_eq!(unpack(pack(&bits), bits.len()), bits);
        }
        #[test]
        fn prop_is_even_matches_popcount_parity(f in any::<u16>()) {
            prop_assert_eq!(is_even(f), f.count_ones() % 2 == 0);
        }
        #[test]
        fn prop_transpose4_is_involution(combo in any::<u16>()) {
            let m = [combo & 0xF, (combo >> 4) & 0xF, (combo >> 8) & 0xF, (combo >> 12) & 0xF];
            prop_assert_eq!(transpose4(transpose4(m)), m);
        }
    }
}
