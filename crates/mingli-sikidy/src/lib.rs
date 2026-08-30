//! L3 叶（C 族）：马达加斯加 Sikidy。
//!
//! 由可复现种子（[`mingli_core::sampler`]）随机起 **4 个母列**（各 4 位），经 [`mingli_core::gf2`]
//! 的转置与逐列 XOR 树生成 **16 列**。第 15 列 C15（创世者，0-based idx 14）= GF(2) 线性组合，
//! **恒为偶**——与地占「法官恒偶」是同一条 GF(2) 奇偶校验定理（Ascher 联系 Hamming 1948 纠错码），
//! 穷举证明在 `mingli_core::gf2`。
//!
//! **列编号用 Ascher 的生成序**，见 [`COLUMN_ROLES`]——这一点必须先说清楚，因为学界流通着两套
//! 互不兼容的编号，连英文维基的条目内部都混用了两套（算法段用生成序、名表用空间序）。
//! 各列的角色见 [`COLUMN_ROLES`]，16 个图（四行点阵）的马达加斯加名分歧太大，不在此出。


#[cfg(feature = "port")]
mod engine;
#[cfg(feature = "port")]
pub use engine::SikidyEngine;

use mingli_core::gf2;
use mingli_core::sampler::SplitMix64;
#[cfg(feature = "serde")]
use serde::Serialize;

/// 16 列各自的角色（中文意译），按**生成序**编号，`COLUMN_ROLES[k]` = 第 `k+1` 列。
///
/// ## 两套编号，别搞混
///
/// 同一个结构在文献里有两种编号方式，差别只在八个派生列怎么排号：
///
/// - **生成序**（Ascher 1997 / Gomez 2015，**本 crate 用这套**）：
///   C9=C7⊕C8、C10=C5⊕C6、C11=C3⊕C4、C12=C1⊕C2、C13=C9⊕C10、C14=C11⊕C12、C15=C13⊕C14、C16=C15⊕C1
/// - **空间序**（Chemillier 2007 / Jaovelo-Dzao，按盘面自左至右）：
///   9=7+8、11=5+6、13=3+4、15=1+2、10=9+11、14=13+15、12=10+14、16=12+1
///
/// 两者描述同一个结构，换算见 [`SPATIAL_POSITION`]。**创世者列在生成序里是第 15、在空间序里是第 12**，
/// 英文维基的算法段与名表分属两套，照抄会踩雷。
///
/// ## 来源
///
/// 三个互相独立的来源在角色语义上交叉印证（马语原词的字面义与英译逐条对得上）：
/// Marcia Ascher《Malagasy Sikidy: A Case in Ethnomathematics》(Historia Mathematica 24, 1997, Fig. 3) ·
/// Chemillier 等《Aspects mathématiques et cognitifs de la divination sikidy à Madagascar》(L'Homme 181, 2007) ·
/// Dahle 研究经 Sibree 转述《Divination among the Malagasy》(Folk-Lore 3, 1892)。
///
/// 🟡 第 6 与第 14 列**三源三说**（第 6：the bad intentions / abily「奴隶」/ Marìna；
/// 第 14：the people / saily / Mpànontàny「发问者」），故留 `None`，不硬选一说。
pub const COLUMN_ROLES: [Option<&str>; 16] = [
    Some("问卜者"),   // C1  tale / Talé
    Some("财物"),     // C2  maly / Harèna
    Some("第三"),     // C3  fahatelo / Fàhatèlo
    Some("土地"),     // C4  bilady / Vòhitra
    Some("子"),       // C5  fianahana / Zatòvo（马语「青年」）
    None,             // C6  三源三说
    Some("女"),       // C7  alisay / Vèhivàvy（马语「女人」）
    Some("敌"),       // C8  fahavalo / Fàhavàlo（马语「敌人」）
    Some("第九·灵"),  // C9  fahasivy / Fàhasìvy
    Some("食"),       // C10 haja / Nìa（马语「食物」）
    Some("先祖"),     // C11 asorita / Asòrotàny
    Some("路"),       // C12 safary / Làlana（马语「路」）
    Some("占者"),     // C13 ombiasy / Màsina（占者 / 圣者）
    None,             // C14 三源三说
    Some("创世者"),   // C15 haky / Andrìamànitra（神）
    Some("屋"),       // C16 kiba / Tràno（马语「房子」）
];

/// 生成序第 `k+1` 列在**空间序**里的位号（1-based）。用来跟 Chemillier 系文献对表。
pub const SPATIAL_POSITION: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 13, 15, 10, 14, 12, 16];

/// 一盘 Sikidy：16 列（各 4 位，0..16），C15 为创世者列。
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Reading {
    /// 4 母列（随机起）。
    pub mothers: [u8; 4],
    /// 全 16 列 C1..C16（顺序 0-based）。
    pub columns: [u8; 16],
    /// 创世者列 C15（生成序第 15，空间序第 12；idx 14）；**恒为偶**。
    ///
    /// 三源同指此列：Ascher 称 "the creator"、Dahle–Sibree 作 Andrìamànitra（神）、
    /// Chemillier 作 haky。三源也都记同一条结构性质——此列必为偶（Dahle：落「奴」图则整盘作废重起）。
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

    /// Dahle 1892 用**名字**给出的八条生成规则，逐条对上本 crate 的公式。
    ///
    /// 这是一个格外硬的 oracle：19 世纪的民族志按马语列名写规则，与 Ascher 1997 的代数写法
    /// 出自完全不同的传统与年代，八条一条不差。原文（Sibree 转述，Folk-Lore 3, 1892）：
    ///
    /// ```text
    /// (a) Talé + Harèna → Làlana          (e) Vèhivàvy + Fàhavàlo → Fàhasìvy
    /// (b) Fàhatèlo + Vòhitra → Asòrotàny  (f) Nìa + Fàhasìvy → Màsina
    /// (c) Làlana + Asòrotàny → Mpànontàny (g) Màsina + Mpànontàny → Andrìamànitra
    /// (d) Zatòvo + Marìna → Nìa           (h) Andrìamànitra + Talé → Tràno
    /// ```
    #[test]
    fn the_eight_named_rules_from_1892_reproduce_the_formulas() {
        // 名 → 生成序列号（1-based），取自 COLUMN_ROLES 的对照
        const TALE: usize = 1;
        const HARENA: usize = 2;
        const FAHATELO: usize = 3;
        const VOHITRA: usize = 4;
        const ZATOVO: usize = 5;
        const MARINA: usize = 6;
        const VEHIVAVY: usize = 7;
        const FAHAVALO: usize = 8;
        const FAHASIVY: usize = 9;
        const NIA: usize = 10;
        const ASOROTANY: usize = 11;
        const LALANA: usize = 12;
        const MASINA: usize = 13;
        const MPANONTANY: usize = 14;
        const ANDRIAMANITRA: usize = 15;
        const TRANO: usize = 16;
        const RULES: [(usize, usize, usize); 8] = [
            (TALE, HARENA, LALANA),                  // a
            (FAHATELO, VOHITRA, ASOROTANY),          // b
            (LALANA, ASOROTANY, MPANONTANY),         // c
            (ZATOVO, MARINA, NIA),                   // d
            (VEHIVAVY, FAHAVALO, FAHASIVY),          // e
            (NIA, FAHASIVY, MASINA),                 // f
            (MASINA, MPANONTANY, ANDRIAMANITRA),     // g
            (ANDRIAMANITRA, TALE, TRANO),            // h
        ];
        for mothers in [[0b1011u16, 0b0110, 0b1110, 0b0001], [0, 15, 5, 10], [7, 7, 7, 7]] {
            let c = from_mothers(mothers).columns;
            for (a, b, out) in RULES {
                assert_eq!(c[out - 1], c[a - 1] ^ c[b - 1], "规则 {a}+{b}→{out} 不成立");
            }
        }
        // 创世者列就是规则 (g) 的产物
        assert_eq!(ANDRIAMANITRA - 1, 14, "创世者列在生成序里是第 15（0-based 14）");
    }

    /// 生成序 ↔ 空间序的换算：两套编号描述同一结构，创世者在两套里分别是第 15 与第 12。
    #[test]
    fn the_two_numbering_conventions_map_onto_each_other() {
        // 前 8 列两套同号
        for (k, &spatial) in SPATIAL_POSITION.iter().take(8).enumerate() {
            assert_eq!(spatial, u8::try_from(k + 1).expect("列号在 u8 内"));
        }
        // 派生 8 列是一个双射
        let mut seen = SPATIAL_POSITION;
        seen.sort_unstable();
        assert_eq!(seen.to_vec(), (1..=16u8).collect::<Vec<_>>(), "应是 1..=16 的置换");
        assert_eq!(SPATIAL_POSITION[14], 12, "创世者：生成序第 15 = 空间序第 12");
        // 空间序的生成式在换算后应与本 crate 的公式一致：空间 10 = 空间 9 ⊕ 空间 11
        let g = |spatial: u8| SPATIAL_POSITION.iter().position(|&p| p == spatial).expect("位号存在");
        let c = from_mothers([0b1011, 0b0110, 0b1110, 0b0001]).columns;
        assert_eq!(c[g(10)], c[g(9)] ^ c[g(11)], "空间序 10 = 9 + 11");
        assert_eq!(c[g(12)], c[g(10)] ^ c[g(14)], "空间序 12 = 10 + 14");
    }

    /// 角色表：14 个位置三源语义一致，第 6 与第 14 三源三说，留空。
    #[test]
    fn the_two_contested_columns_are_left_empty() {
        assert_eq!(COLUMN_ROLES.iter().filter(|r| r.is_none()).count(), 2);
        assert!(COLUMN_ROLES[5].is_none(), "第 6 列三源三说");
        assert!(COLUMN_ROLES[13].is_none(), "第 14 列三源三说");
        assert_eq!(COLUMN_ROLES[14], Some("创世者"));
        for (k, role) in COLUMN_ROLES.iter().enumerate() {
            if let Some(r) = role {
                assert!(!r.is_empty(), "第 {} 列的角色不该是空串", k + 1);
            }
        }
    }

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
