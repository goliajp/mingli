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

#[cfg(feature = "port")]
mod engine;
#[cfg(feature = "port")]
pub use engine::YijingEngine;

use mingli_core::sampler::SplitMix64;
use mingli_gua::{Hexagram, Trigram};
#[cfg(feature = "serde")]
use serde::Serialize;

/// 起卦法（仅影响爻值的概率分布）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Method {
    /// 三钱法：6/7/8/9 ~ 1/8， 3/8， 3/8， 1/8。
    ThreeCoins,
    /// 蓍草法：6/7/8/9 ~ 1/16， 5/16， 7/16， 3/16。
    YarrowStalks,
}

/// 一爻：营数值（6/7/8/9）及由它定出的阴阳与是否变爻。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
        // 三钱法的爻值分布是 B(3, ½) 的直接推论：三枚各正反等概，背面记 3、正面记 2，
        // 和为 6/7/8/9，概率 1/8、3/8、3/8、1/8。**这是可自行推导的，不依赖任何一家的说法**——
        // 也正因如此，它与蓍草法的不同（下一条）才是体系差异而非误差。
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
        // 蓍草法的分布 1/16、5/16、7/16、3/16 由「四营十八变」的分策规则决定，
        // 与三钱法明显不同：老阳 3/16 远高于三钱的 1/8，老阴 1/16 远低于 1/8，
        // 故蓍草法的变爻更偏向老阳。这一组数是历代注家反复引用的定值，
        // 也可由分策过程自行复算（每变去一、四揲、归奇），两条路殊途同归。
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

    /// 按传统约定把六爻装成卦：初爻在下，下卦 = 初二三爻，上卦 = 四五上爻。
    ///
    /// 刻意与 `cast` 里那段位运算分开写：`cast` 只有一个 `prim |= 1 << i` 的循环，
    /// 从不显式区分上下卦，也从不说哪一爻在下。这里把约定摊开写，才有东西可对。
    fn assemble_bottom_up(lines: &[Line; 6]) -> (Hexagram, Trigram, Trigram) {
        let bit = |i: usize| u8::from(lines[i].yang);
        let lower = bit(0) | (bit(1) << 1) | (bit(2) << 2); // 初、二、三
        let upper = bit(3) | (bit(4) << 1) | (bit(5) << 2); // 四、五、上
        (Hexagram(lower | (upper << 3)), Trigram(lower), Trigram(upper))
    }

    /// 先把上面那个装配函数钉在通行本卦序上——否则它可能跟 `cast` 错得一模一样，
    /// 两边一起颠倒还是对得上，等于什么也没验。
    ///
    /// 「自下而上、最下一爻称初」见朱熹《周易本义·筮仪》所述揲蓍成爻之序，
    /// 以及《系辞上》大衍之数章；两处所指相同。卦序取通行本，与
    /// `mingli_gua` 已三源校验的 `HEXAGRAM_NAMES` 一致。
    #[test]
    fn the_assembly_convention_lands_on_the_received_hexagram_order() {
        let ln = Line::from_value;
        // 初二三阳、四五上阴 → 下乾上坤 → 地天泰（卦名自上而下读，与爻序自下而上不同）
        let (h, lower, upper) = assemble_bottom_up(&[ln(7), ln(7), ln(7), ln(8), ln(8), ln(8)]);
        assert_eq!((lower.name(), upper.name()), ("乾", "坤"));
        assert_eq!(h.full_name(), "地天泰");
        assert_eq!(h.king_wen(), 11);

        // 反过来：下坤上乾 → 天地否
        let (h, lower, upper) = assemble_bottom_up(&[ln(8), ln(8), ln(8), ln(7), ln(7), ln(7)]);
        assert_eq!((lower.name(), upper.name()), ("坤", "乾"));
        assert_eq!(h.full_name(), "天地否");
        assert_eq!(h.king_wen(), 12);

        // 下震上坎 → 水雷屯：震为阳阴阴（自下而上）、坎为阴阳阴
        let (h, lower, upper) = assemble_bottom_up(&[ln(7), ln(8), ln(8), ln(8), ln(7), ln(8)]);
        assert_eq!((lower.name(), upper.name()), ("震", "坎"));
        assert_eq!(h.full_name(), "水雷屯");
        assert_eq!(h.king_wen(), 3);

        // 六阳 / 六阴两端
        let (h, ..) = assemble_bottom_up(&[ln(9); 6]);
        assert_eq!((h.full_name(), h.king_wen()), ("乾为天", 1));
        let (h, ..) = assemble_bottom_up(&[ln(6); 6]);
        assert_eq!((h.full_name(), h.king_wen()), ("坤为地", 2));
    }

    /// 摇出来的六爻与对外报出的那一卦，是不是同一件事。
    ///
    /// 此前一处也没验过。原有的测试全都与爻序无关：两种起卦法的分布不看爻位，
    /// `resulting_flips_only_changing_lines` 比的是 `primary` 与 `changing_mask`
    /// 自己之间的关系，而 `resulting = primary.changed(mask)` 本来就保证这一点，
    /// 它从没把 `mask` 跟 `lines[i].changing` 对上；卦名只验了「在八名之列」。
    ///
    /// 实测（2026-08-23）：把六爻上下颠倒、把上卦名接成下卦、把变爻掩码单独颠倒，
    /// 三种改法各自跑全量套件，一条守卫都不红——每一卦都会变成另一卦，卦名、文王序、
    /// 所指的爻辞全不同，而没有任何地方察觉。
    #[test]
    fn what_is_reported_is_the_hexagram_the_lines_actually_make() {
        for method in [Method::ThreeCoins, Method::YarrowStalks] {
            for seed in 0..300u64 {
                let c = cast(method, seed);
                let (primary, lower, upper) = assemble_bottom_up(&c.lines);
                assert_eq!(c.primary, primary, "{method:?} 种子 {seed}：本卦与六爻不符");
                assert_eq!(c.primary_lower, lower.name(), "{method:?} 种子 {seed}：本卦下卦");
                assert_eq!(c.primary_upper, upper.name(), "{method:?} 种子 {seed}：本卦上卦");
                assert_eq!(c.primary_name, primary.name());
                assert_eq!(c.primary_full_name, primary.full_name());
                assert_eq!(c.primary_king_wen, primary.king_wen());

                // 变爻掩码必须逐位对上每一爻自己的 changing，而不只是与之卦自洽。
                for (i, line) in c.lines.iter().enumerate() {
                    assert_eq!(
                        (c.changing_mask >> i) & 1 == 1,
                        line.changing,
                        "{method:?} 种子 {seed} 第 {} 爻：掩码与营数 {} 不符",
                        i + 1,
                        line.value
                    );
                }

                // 之卦同样要从「翻过变爻之后的六爻」重新装一遍，而不是信 changed()。
                let flipped: [Line; 6] = std::array::from_fn(|i| {
                    let l = c.lines[i];
                    Line { yang: l.yang != l.changing, ..l }
                });
                let (resulting, rlower, rupper) = assemble_bottom_up(&flipped);
                assert_eq!(c.resulting, resulting, "{method:?} 种子 {seed}：之卦与六爻不符");
                assert_eq!(c.resulting_lower, rlower.name());
                assert_eq!(c.resulting_upper, rupper.name());
                assert_eq!(c.resulting_king_wen, resulting.king_wen());
            }
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
