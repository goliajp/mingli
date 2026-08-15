//! L3 叶（C 族）：易经起卦。
//!
//! 由可复现的种子（[`mingli_core::sampler`]）驱动随机起卦，得六爻的「老阴6/少阳7/少阴8/老阳9」
//! 值，组成**本卦**与（按变爻）**之卦**，六爻的错综互之变化全部复用主干 [`mingli_gua`] 的位向量代数。
//!
//! 两种起卦法的差别**只在四个爻值的概率分布**（同一套抽样接口）：
//! - 三钱法 `ThreeCoins`：三枚硬币，阳面计数 → 6/7/8/9 的概率 = 1/8， 3/8， 3/8， 1/8（二项 B(3，½)）。
//! - 蓍草法 `YarrowStalks`：传统揲蓍过程的等价分布 → 1/16， 5/16， 7/16， 3/16。
//!
//! 语域注：本叶只做起卦的**抽样 + 卦的结构**。**六十四卦的文王序号与卦名**属需逐项核对权威文献的
//! 数据表（🟡），与 [`mingli_gua`] 一致地不在此凭记忆硬编；卦以「上卦/下卦」复合标识。八卦名仅 8 项、已核实。

#![allow(
    clippy::unreadable_literal,
    reason = "卦的二进制位型（如 0b101010）连写比加分隔符更直观对应六爻，沿用 mingli-gua 约定"
)]

use mingli_core::sampler::SplitMix64;
use mingli_gua::{Hexagram, Trigram};
use serde::Serialize;

/// 起卦法（仅影响爻值的概率分布）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Method {
    /// 三钱法：6/7/8/9 ~ 1/8， 3/8， 3/8， 1/8。
    ThreeCoins,
    /// 蓍草法：6/7/8/9 ~ 1/16， 5/16， 7/16， 3/16。
    YarrowStalks,
}

/// 一爻：营数值（6/7/8/9）及由它定出的阴阳与是否变爻。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Line {
    /// 营数：6 老阴、7 少阳、8 少阴、9 老阳。
    pub value: u8,
    /// 阴阳：`true`=阳（7 或 9）。
    pub yang: bool,
    /// 是否为变爻（老阴 6 或老阳 9）。
    pub changing: bool,
}

impl Line {
    /// 由营数值（6/7/8/9）构造。阳 = 奇数（7/9）；变爻 = 老阴/老阳（6/9）。
    #[must_use]
    pub fn from_value(value: u8) -> Self {
        Line {
            value,
            yang: value % 2 == 1,
            changing: value == 6 || value == 9,
        }
    }
}

/// 一次完整起卦的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Cast {
    /// 起卦法。
    pub method: Method,
    /// 六爻，自下而上（索引 0 = 初爻）。
    pub lines: [Line; 6],
    /// 本卦（六爻阴阳，复用 [`mingli_gua`]）。
    pub primary: Hexagram,
    /// 本卦上卦名 / 下卦名（八卦名已核实）。
    pub primary_upper: &'static str,
    /// 见 [`Cast::primary_upper`]。
    pub primary_lower: &'static str,
    /// 变爻掩码：bit_i=1 表第 i+1 爻为变爻。
    pub changing_mask: u8,
    /// 之卦（按变爻翻转本卦；无变爻时等于本卦）。
    pub resulting: Hexagram,
    /// 之卦上卦名 / 下卦名。
    pub resulting_upper: &'static str,
    /// 见 [`Cast::resulting_upper`]。
    pub resulting_lower: &'static str,
    /// 本卦简称（乾/坤/屯/.. — 三源校验，见 `mingli_gua::HEXAGRAM_NAMES`）。
    pub primary_name: &'static str,
    /// 本卦传统全名（乾为天/水雷屯/.. — 三源校验，见 `mingli_gua::HEXAGRAM_FULL_NAMES`）。
    pub primary_full_name: &'static str,
    /// 本卦的文王卦序 1..=64。
    pub primary_king_wen: u8,
    /// 之卦简称。
    pub resulting_name: &'static str,
    /// 之卦传统全名。
    pub resulting_full_name: &'static str,
    /// 之卦的文王卦序 1..=64。
    pub resulting_king_wen: u8,
}

/// 抽一个营数值（6/7/8/9）。三钱用三枚硬币的阳面计数；蓍草用一个 `[0,16)` 的等概抽样按区间映射。
fn draw_value(method: Method, rng: &mut SplitMix64) -> u8 {
    match method {
        // 三阳面计数 h∈0..=3，值=6+h：B(3，½) → 6：1/8，7：3/8，8：3/8，9：1/8。
        Method::ThreeCoins => {
            let h = u8::from(rng.bit()) + u8::from(rng.bit()) + u8::from(rng.bit());
            6 + h
        }
        // 1/16，5/16，7/16，3/16：区间 {0}→6， {1..=5}→7， {6..=12}→8， {13..=15}→9。
        Method::YarrowStalks => match rng.below(16) {
            0 => 6,
            1..=5 => 7,
            6..=12 => 8,
            _ => 9,
        },
    }
}

/// 以给定起卦法与种子起一卦（同法同种子 → 同卦，可复现）。
#[must_use]
pub fn cast(method: Method, seed: u64) -> Cast {
    let mut rng = SplitMix64::new(seed);
    let lines: [Line; 6] = std::array::from_fn(|_| Line::from_value(draw_value(method, &mut rng)));

    let mut prim = 0u8;
    let mut mask = 0u8;
    for (i, ln) in lines.iter().enumerate() {
        if ln.yang {
            prim |= 1 << i;
        }
        if ln.changing {
            mask |= 1 << i;
        }
    }
    let primary = Hexagram(prim);
    let resulting = primary.changed(mask);
    Cast {
        method,
        lines,
        primary,
        primary_upper: primary.upper().name(),
        primary_lower: primary.lower().name(),
        changing_mask: mask,
        resulting,
        resulting_upper: resulting.upper().name(),
        resulting_lower: resulting.lower().name(),
        primary_name: primary.name(),
        primary_full_name: primary.full_name(),
        primary_king_wen: primary.king_wen(),
        resulting_name: resulting.name(),
        resulting_full_name: resulting.full_name(),
        resulting_king_wen: resulting.king_wen(),
    }
}

/// 八卦值 → 卦名（薄包装，便于无 gua 依赖的调用方读名）。
#[must_use]
pub fn trigram_name(value: u8) -> &'static str {
    Trigram(value).name()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_classification_table() {
        // 6 老阴：阴、变；7 少阳：阳、不变；8 少阴：阴、不变；9 老阳：阳、变。
        let cases = [
            (6u8, false, true),
            (7, true, false),
            (8, false, false),
            (9, true, true),
        ];
        for (v, yang, changing) in cases {
            let l = Line::from_value(v);
            assert_eq!((l.yang, l.changing), (yang, changing), "营数 {v}");
        }
    }

    #[test]
    fn deterministic_given_seed() {
        let a = cast(Method::ThreeCoins, 2024);
        let b = cast(Method::ThreeCoins, 2024);
        assert_eq!(a.primary, b.primary);
        assert_eq!(a.changing_mask, b.changing_mask);
        assert_eq!(a.resulting, b.resulting);
        // 不同种子应（大概率）不同——至少本卦或变爻其一不同。
        let c = cast(Method::ThreeCoins, 99);
        assert!(a.primary != c.primary || a.changing_mask != c.changing_mask);
    }

    #[test]
    fn resulting_flips_only_changing_lines() {
        // 之卦 = 本卦在变爻位翻转；非变爻位不动。
        for seed in 0..200u64 {
            let cst = cast(Method::YarrowStalks, seed);
            for i in 0..6 {
                let p = (cst.primary.0 >> i) & 1;
                let r = (cst.resulting.0 >> i) & 1;
                let changed = (cst.changing_mask >> i) & 1 == 1;
                assert_eq!(p != r, changed, "seed {seed} 爻 {i}");
            }
        }
    }

    #[test]
    fn three_coins_distribution() {
        // B(3，½)：6，7，8，9 ≈ 1/8，3/8，3/8，1/8。大样本统计落在容差内。
        let mut cnt = [0u32; 4]; // 6,7,8,9
        let n = 60_000u32;
        let mut rng = SplitMix64::new(7);
        for _ in 0..n {
            cnt[(draw_value(Method::ThreeCoins, &mut rng) - 6) as usize] += 1;
        }
        let want = [0.125, 0.375, 0.375, 0.125];
        for (i, w) in want.iter().enumerate() {
            let p = f64::from(cnt[i]) / f64::from(n);
            assert!((p - w).abs() < 0.01, "三钱 {}={p:.3} 应≈{w}", i + 6);
        }
    }

    #[test]
    fn yarrow_distribution() {
        // 1/16,5/16,7/16,3/16。
        let mut cnt = [0u32; 4];
        let n = 80_000u32;
        let mut rng = SplitMix64::new(13);
        for _ in 0..n {
            cnt[(draw_value(Method::YarrowStalks, &mut rng) - 6) as usize] += 1;
        }
        let want = [1.0 / 16.0, 5.0 / 16.0, 7.0 / 16.0, 3.0 / 16.0];
        for (i, w) in want.iter().enumerate() {
            let p = f64::from(cnt[i]) / f64::from(n);
            assert!((p - w).abs() < 0.01, "蓍草 {}={p:.3} 应≈{w:.3}", i + 6);
        }
    }

    #[test]
    fn trigram_names_resolve() {
        let cst = cast(Method::ThreeCoins, 42);
        // 上/下卦名取自 gua 已核实的 8 名。
        assert!(mingli_gua::TRIGRAM_NAMES.contains(&cst.primary_upper));
        assert!(mingli_gua::TRIGRAM_NAMES.contains(&cst.resulting_lower));
        assert_eq!(trigram_name(7), "乾");
    }

    #[test]
    fn no_changing_lines_means_resulting_equals_primary() {
        // 构造一个全少阳/少阴的人造卦：直接验代数关系（mask=0 → 之卦=本卦）。
        let p = Hexagram(0b101010);
        assert_eq!(p.changed(0), p);
    }
}
