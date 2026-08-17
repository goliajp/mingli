//! L3 叶（C 族）：地占 ʿilm al-raml（阿拉伯/欧洲传统的盾牌图）。
//!
//! 由可复现种子（[`mingli_core::sampler`]）随机起 **4 个母图**（每图 4 行、每行单/双点 = 1 位），
//! 经 [`mingli_core::gf2`] 的转置与 XOR 推出全部派生图：4 女图（母块转置）、4 侄图（成对 XOR）、
//! 2 见证、1 **法官**。法官 = 两见证 XOR，**恒为偶图**——这是 GF(2) 奇偶校验定理（每个母位
//! 经转置被计入偶数次而 mod-2 成对抵消），其穷举证明在 `mingli_core::gf2`。
//!
//! 图名见 [`FIGURE_NAMES`]，点阵三源一致。星占归属（行星 / 星座）与阿拉伯名另有分歧，不在此出。


mod engine;
pub use engine::GeomancyEngine;

use mingli_core::gf2;
use mingli_core::sampler::SplitMix64;
use serde::Serialize;

/// 16 个地占图的拉丁名，按本 crate 的 4 位整数值索引。
///
/// **位序**：bit0 = 第一行 = 火，bit3 = 第四行 = 土；**置位 = 单点**（奇 / active），清位 = 双点。
/// 这个位序不是随手定的——[`mingli_core::gf2::transpose4`] 让「女图 d 的第 i 位 = 母图 i 的第 d 位」，
/// 正是古法「女一由四母的火行依次组成」，所以第一行必须落在 bit0。
/// 位序搞反会让 Fortuna Major 与 Fortuna Minor 互换（两者互为上下翻转），故 [`tests`] 里逐图钉死。
///
/// 三源一致：Unicode 提案 L2/23-218（2023，已入 Unicode 17.0，给出编码规则原句
/// 「two dots is treated as 0 and one as 1, assuming the least-significant bit at the bottom」）·
/// Princeton「Medieval Geomancy」图版（Martin of Spain《De geomantia》英译）·
/// en.wikipedia《Geomantic figures》（其 impartial / partial / entering / exiting 四张分类表与本表全维度自洽）。
///
/// 🟡 不入码的部分：行星 / 星座归属两派冲突（Puer 与 Puella 的归属，Martin of Spain 一系与
/// Agrippa 一系相反，虽然两派的「名 ↔ 点阵」映射一致）；阿拉伯名 15/16 两源相符但同一图常有
/// 多个并行名（ʿuqla 亦作 thikāf 等），且 Puer 一图两源给出不同名（jawdala / faraḥ）。
pub const FIGURE_NAMES: [&str; 16] = [
    "Populus",        //  0 = 火双 气双 水双 土双（8 点）
    "Laetitia",       //  1 = 火单 气双 水双 土双（7 点）
    "Rubeus",         //  2 = 火双 气单 水双 土双（7 点）
    "Fortuna Minor",  //  3 = 火单 气单 水双 土双（6 点）
    "Albus",          //  4 = 火双 气双 水单 土双（7 点）
    "Amissio",        //  5 = 火单 气双 水单 土双（6 点）
    "Conjunctio",     //  6 = 火双 气单 水单 土双（6 点）
    "Cauda Draconis", //  7 = 火单 气单 水单 土双（5 点）
    "Tristitia",      //  8 = 火双 气双 水双 土单（7 点）
    "Carcer",         //  9 = 火单 气双 水双 土单（6 点）
    "Acquisitio",     // 10 = 火双 气单 水双 土单（6 点）
    "Puer",           // 11 = 火单 气单 水双 土单（5 点）
    "Fortuna Major",  // 12 = 火双 气双 水单 土单（6 点）
    "Puella",         // 13 = 火单 气双 水单 土单（5 点）
    "Caput Draconis", // 14 = 火双 气单 水单 土单（5 点）
    "Via",            // 15 = 火单 气单 水单 土单（4 点）
];

/// 某图的点阵：四行自上而下（火 · 气 · 水 · 土），每行 1（单点）或 2（双点）。
#[must_use]
pub const fn figure_points(value: u8) -> [u8; 4] {
    const fn row(value: u8, k: u8) -> u8 {
        if (value >> k) & 1 == 1 { 1 } else { 2 }
    }
    [row(value, 0), row(value, 1), row(value, 2), row(value, 3)]
}

/// 某图的总点数 4..=8。单点行记 1、双点行记 2，故 = 8 − 单点行数。
#[must_use]
#[allow(clippy::cast_possible_truncation, reason = "四位里的置位数最多 4，转 u8 不截断")]
pub const fn figure_dots(value: u8) -> u8 {
    8 - (value & 0xF).count_ones() as u8
}


/// 一盘里每个位置的图名（与 [`Reading`] 的各字段一一对应）。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ReadingNames {
    /// 4 母图名。
    pub mothers: [&'static str; 4],
    /// 4 女图名。
    pub daughters: [&'static str; 4],
    /// 4 侄图名。
    pub nieces: [&'static str; 4],
    /// 2 见证名 `[右, 左]`。
    pub witnesses: [&'static str; 2],
    /// 法官名。
    pub judge: &'static str,
}

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
    /// 各位置的图名（见 [`FIGURE_NAMES`]）。
    pub names: ReadingNames,
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
    let n = |f: gf2::Figure| FIGURE_NAMES[(f & 0xF) as usize];
    Reading {
        mothers: s.mothers.map(m),
        daughters: s.daughters.map(m),
        nieces: s.nieces.map(m),
        witnesses: s.witnesses.map(m),
        judge: m(s.judge),
        judge_even: gf2::is_even(s.judge),
        names: ReadingNames {
            mothers: s.mothers.map(n),
            daughters: s.daughters.map(n),
            nieces: s.nieces.map(n),
            witnesses: s.witnesses.map(n),
            judge: n(s.judge),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 16 图名与点阵的 oracle。点阵写成「火气水土」四行，1 = 单点、2 = 双点，
    /// 与 Unicode 提案 L2/23-218 的编码表、Princeton 图版、en.wikipedia 三处一致。
    ///
    /// **位序反了就会 Fortuna Major ↔ Fortuna Minor 互换**，所以这里逐图列全，不靠推。
    #[test]
    fn every_figure_name_matches_its_point_pattern() {
        // (值, 名, [火, 气, 水, 土], 总点数)
        const ORACLE: [(u8, &str, [u8; 4], u8); 16] = [
            (0, "Populus", [2, 2, 2, 2], 8),
            (1, "Laetitia", [1, 2, 2, 2], 7),
            (2, "Rubeus", [2, 1, 2, 2], 7),
            (3, "Fortuna Minor", [1, 1, 2, 2], 6),
            (4, "Albus", [2, 2, 1, 2], 7),
            (5, "Amissio", [1, 2, 1, 2], 6),
            (6, "Conjunctio", [2, 1, 1, 2], 6),
            (7, "Cauda Draconis", [1, 1, 1, 2], 5),
            (8, "Tristitia", [2, 2, 2, 1], 7),
            (9, "Carcer", [1, 2, 2, 1], 6),
            (10, "Acquisitio", [2, 1, 2, 1], 6),
            (11, "Puer", [1, 1, 2, 1], 5),
            (12, "Fortuna Major", [2, 2, 1, 1], 6),
            (13, "Puella", [1, 2, 1, 1], 5),
            (14, "Caput Draconis", [2, 1, 1, 1], 5),
            (15, "Via", [1, 1, 1, 1], 4),
        ];
        for (v, name, points, dots) in ORACLE {
            assert_eq!(FIGURE_NAMES[v as usize], name, "值 {v}");
            assert_eq!(figure_points(v), points, "{name} 的点阵");
            assert_eq!(figure_dots(v), dots, "{name} 的总点数");
            assert_eq!(points.iter().sum::<u8>(), dots, "{name} 点阵与总点数应对得上");
        }
        // 16 个名字互不重复
        let mut sorted = FIGURE_NAMES;
        sorted.sort_unstable();
        let mut deduped = sorted.to_vec();
        deduped.dedup();
        assert_eq!(deduped.len(), 16, "16 个名字应互不重复");
    }

    /// 维基那四张分类表反过来校验本表：偶 / 奇各 8、上下对称者恰 4、进 / 出各 6。
    ///
    /// 这几条都是 en.wikipedia《Geomantic figures》独立给出的分类，与点阵是两套说法；
    /// 它们全部对上，等于第三源的全维度交叉验证。
    #[test]
    fn the_classification_tables_agree_with_the_point_patterns() {
        let even: Vec<&str> = (0..16u8).filter(|v| figure_dots(*v).is_multiple_of(2)).map(|v| FIGURE_NAMES[v as usize]).collect();
        assert_eq!(even.len(), 8, "impartial（偶点数）应恰 8 个：{even:?}");
        assert_eq!(16 - even.len(), 8, "partial（奇点数）应恰 8 个");
        // 上下对称（回文点阵）者，维基记为「both」，恰 4 个
        let palindromic: Vec<&str> = (0..16u8)
            .filter(|v| {
                let p = figure_points(*v);
                p[0] == p[3] && p[1] == p[2]
            })
            .map(|v| FIGURE_NAMES[v as usize])
            .collect();
        assert_eq!(palindromic.len(), 4);
        for name in ["Populus", "Via", "Carcer", "Conjunctio"] {
            assert!(palindromic.contains(&name), "{name} 应在 both 组：{palindromic:?}");
        }
        // 其余 12 个两两互为上下翻转 → entering 6 / exiting 6
        let flip = |v: u8| (0..4u8).fold(0u8, |a, k| a | (((v >> k) & 1) << (3 - k)));
        let pairs = (0..16u8).filter(|v| flip(*v) != *v).count();
        assert_eq!(pairs, 12, "非对称者应恰 12 个，构成 6 对");
        for v in 0..16u8 {
            assert_eq!(flip(flip(v)), v, "翻转两次应回到自身");
        }
    }

    /// 法官恒为偶图这条 GF(2) 定理，用名字复述一遍：法官只可能是那 8 个偶图之一。
    #[test]
    fn the_judge_is_always_one_of_the_eight_even_figures() {
        for seed in 0..200u64 {
            let r = cast(seed);
            assert!(r.judge_even, "seed {seed}");
            assert!(figure_dots(r.judge).is_multiple_of(2), "seed {seed}：{} 应是偶图", r.names.judge);
            assert_eq!(r.names.judge, FIGURE_NAMES[r.judge as usize]);
        }
    }

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
