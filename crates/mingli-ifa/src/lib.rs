//! L3 叶（C 族）：西非约鲁巴 Ifá。
//!
//! 一次占卜得一个 **odu**，由左右两个 figure 组成，每个 figure 是 4 个二进制标记（单/双）。
//! 故全集 = 16 × 16 = **256 odu**。本叶用可复现种子（[`mingli_core::sampler`]）抽 8 个二进制位，
//! 经 [`mingli_core::gf2`] 打包成左右两 figure，定出 odu 序号（`left·16 + right`，0..256）。
//!
//! 单 figure（4 位）与地占/Sikidy 的图同构（同一 (Z₂)⁴）；Ifá 的特点是**有序成对**得 2⁸ 空间。
//!
//! 语域注：本叶只做 256 odu 的**组合结构**。256 个 odu 名（Ogbe-Ogbe， Oyeku…）及其经文属需逐项
//! 核对文献的庞大数据表（🟡），错一个即毒化整枝，**绝不在此凭记忆硬编**；odu 以序号标识。

use mingli_core::gf2;
use mingli_core::sampler::SplitMix64;
use serde::Serialize;

/// odu 总数 = 16 × 16。
pub const ODU_COUNT: u16 = 256;

/// 一个 odu：左右两 figure（各 0..16）及合成序号（0..256）。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Odu {
    /// 左 figure（先得，高位侧），0..16。
    pub left: u8,
    /// 右 figure，0..16。
    pub right: u8,
    /// odu 序号 = `left·16 + right`，0..256。
    pub index: u16,
    /// 左 figure 的 4 个标记（低位在前，`true`=单标记/阳）。
    pub left_marks: [bool; 4],
    /// 右 figure 的 4 个标记。
    pub right_marks: [bool; 4],
}

/// 抽一个 figure：4 个独立二进制标记打包成 4 位值。
fn draw_figure(rng: &mut SplitMix64) -> (u8, [bool; 4]) {
    let marks: [bool; 4] = std::array::from_fn(|_| rng.bit());
    ((gf2::pack(&marks) & 0xF) as u8, marks)
}

/// 由种子占一个 odu（同种子 → 同 odu，可复现）。左先于右。
#[must_use]
pub fn cast(seed: u64) -> Odu {
    let mut rng = SplitMix64::new(seed);
    let (left, left_marks) = draw_figure(&mut rng);
    let (right, right_marks) = draw_figure(&mut rng);
    Odu {
        left,
        right,
        index: u16::from(left) * 16 + u16::from(right),
        left_marks,
        right_marks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_given_seed() {
        assert_eq!(cast(2024).index, cast(2024).index);
        assert_ne!(cast(1).index, cast(2).index);
    }

    #[test]
    fn index_composition_and_range() {
        for seed in 0..400u64 {
            let o = cast(seed);
            assert!(o.index < ODU_COUNT, "odu 序越界 {}", o.index);
            assert_eq!(o.index, u16::from(o.left) * 16 + u16::from(o.right));
            assert!(o.left < 16 && o.right < 16);
            // 标记打包自洽。
            assert_eq!(u16::from(o.left), gf2::pack(&o.left_marks));
            assert_eq!(u16::from(o.right), gf2::pack(&o.right_marks));
        }
    }

    #[test]
    fn covers_full_odu_space() {
        // 不同种子应铺满 256 odu 的相当一部分（抽样覆盖性）。
        let seen: std::collections::HashSet<u16> = (0..3000u64).map(|s| cast(s).index).collect();
        assert!(seen.len() > 200, "仅覆盖 {} 个 odu，期望铺满 256 之多数", seen.len());
    }
}
