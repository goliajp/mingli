//! L3 叶（C 族）：西非约鲁巴 Ifá。
//!
//! 一次占卜得一个 **odu**，由左右两个 figure 组成，每个 figure 是 4 个二进制标记（单/双）。
//! 故全集 = 16 × 16 = **256 odu**。本叶用可复现种子（[`mingli_core::sampler`]）抽 8 个二进制位，
//! 经 [`mingli_core::gf2`] 打包成两个 figure，定出 odu 序号（`right·16 + left`，0..256）。
//!
//! 单 figure（4 位）与地占 / Sikidy 的图同构（同一 (Z₂)⁴）；Ifá 的特点是**有序成对**得 2⁸ 空间。
//!
//! # 两条容易弄反、弄反就看不出来的约定
//!
//! Bascom《Ifa Divination》(1969) p. 41 特意警告：左右弄反就是**另一个 odù**，有另一套断辞与献祭。
//! 弄反之后名字仍然合法，所以必须钉死：
//!
//! - **行序**：第一次投出的标记画在**顶行**，依次往下（Bascom Fig. 2 面板 A 直接把八次投掷的编号
//!   画在了八个位置上：顶行右→顶行左→次行右→次行左→…）。
//! - **左右**：**右列先出、为长**（ọ̀tún，男），左列后出（òsì，女）；复合名**右名在前**。
//!
//! 16 主 odù 的名与点阵见 [`PRINCIPAL_NAMES`]。🟡 十六者的**排序**无定本，见 [`PRINCIPAL_NAMES`] 说明；
//! 256 个复合 odù 的名与经文属需逐项核对的庞大数据表，尼日利亚 / 古巴 / 贝宁三系拼写系统性不同，
//! **绝不在此凭记忆硬编**——本叶只按右名 + 左名拼出复合名。


mod engine;
pub use engine::IfaEngine;

use mingli_core::gf2;
use mingli_core::sampler::SplitMix64;
use serde::Serialize;

/// odu 总数 = 16 × 16。
pub const ODU_COUNT: u16 = 256;

/// 16 主 odù（Ojú Odù）的名，按本 crate 的 4 位整数值索引。
///
/// **编码**：bit `i` = 第 `i` 行（bit0 = 顶行），置位 = **单画**、清位 = 双画。
/// 这个 0/1 赋值是本 crate 的内部表示，不是传统约定——传统只写单画 / 双画本身。
/// 可引用的原始写法是 Bascom Table 3 的四位 `1/2` 串（顶行在最左，1 = 单、2 = 双），
/// 例如 `Ọkanran = 2221`；本表与该串的换算见测试。
///
/// **来源**：Bascom《Ifa Divination: Communication Between Gods and Men》(1969)
/// Table 1「The Sixteen Basic Figures of Ifa」(p. 4) 与 Table 3「The Order of the Basic Ifa Figures」
/// (p. 48) 为一手；en.wikipedia《Ifá》两张表（Yoruba 与 Fon 两传统各写一遍）、
/// Wikimedia Commons《The Meji Odus》图、以及经地占图样反推的跨系统对照表，四处逐条相符。
///
/// 🟡 **排序无定本，故本表按数值索引而不按名次排**。Bascom Table 3 自己就并列了两套
/// （A. Ifẹ̀ 与 B. Southwestern Yoruba，差异在第 5–8 与 11–14 位），并在 p. 47 记明
/// 「另有二十一套排序被记录在案」。任何「第 N 号 odù」的说法都必须先说是哪一套。
///
/// 🟡 拼写有并行写法：Òdí / Èdí、Ọ̀sá / Ọ̀ṣá、Òtúrúpọ̀n / Òtúúrúpọ̀n、Òfún / Ọ̀ràngún。
pub const PRINCIPAL_NAMES: [&str; 16] = [
    "Ọ̀yẹ̀kú",         //  0 = 双双双双（Bascom 2222）
    "Ọ̀bàrà",          //  1 = 单双双双（1222）
    "Ìká",             //  2 = 双单双双（2122）
    "Ìrosùn",          //  3 = 单单双双（1122）
    "Òtúrúpọ̀n",       //  4 = 双双单双（2212）
    "Ọ̀ṣẹ́",           //  5 = 单双单双（1212）
    "Ìwòrì",           //  6 = 双单单双（2112）
    "Ògúndá",          //  7 = 单单单双（1112）
    "Ọ̀kànràn",        //  8 = 双双双单（2221）
    "Òdí",             //  9 = 单双双单（1221）
    "Òfún",            // 10 = 双单双单（2121）
    "Ìrẹtẹ̀",          // 11 = 单单双单（1121）
    "Ọ̀wọ́nrín",       // 12 = 双双单单（2211）
    "Òtúrá",           // 13 = 单双单单（1211）
    "Ọ̀sá",            // 14 = 双单单单（2111）
    "Ogbè",            // 15 = 单单单单（1111）
];

/// 某 figure 的 Bascom 式四位串：顶行在最左，`1` = 单画、`2` = 双画。
#[must_use]
pub fn bascom_notation(figure: u8) -> String {
    (0..4).map(|i| if (figure >> i) & 1 == 1 { '1' } else { '2' }).collect()
}

/// 一个 odu：右左两 figure（各 0..16）及合成序号（0..256）。
#[derive(Debug, Clone, Serialize)]
pub struct Odu {
    /// 右 figure（**先得，为长**；ọ̀tún，男），0..16。
    pub right: u8,
    /// 左 figure（后得；òsì，女），0..16。
    pub left: u8,
    /// odu 序号 = `right·16 + left`，0..256。右在高位，与「右名在前」一致。
    pub index: u16,
    /// 右 figure 的 4 个标记（`marks[0]` = **顶行**，`true` = 单画）。
    pub right_marks: [bool; 4],
    /// 左 figure 的 4 个标记（同上）。
    pub left_marks: [bool; 4],
    /// 右 figure 的主 odù 名。
    pub right_name: &'static str,
    /// 左 figure 的主 odù 名。
    pub left_name: &'static str,
    /// 复合名：**右名在前**（Bascom p. 41：反过来就是另一个 odù）。左右相同时为「X Méjì」。
    pub name: String,
    /// 左右两半相同（Méjì，即 16 主 odù 之一）。
    pub meji: bool,
}

/// 抽一个 figure：4 个独立二进制标记打包成 4 位值。
fn draw_figure(rng: &mut SplitMix64) -> (u8, [bool; 4]) {
    let marks: [bool; 4] = std::array::from_fn(|_| rng.bit());
    ((gf2::pack(&marks) & 0xF) as u8, marks)
}

/// 由种子占一个 odu（同种子 → 同 odu，可复现）。**右先于左**（右为长）。
#[must_use]
pub fn cast(seed: u64) -> Odu {
    let mut rng = SplitMix64::new(seed);
    let (right, right_marks) = draw_figure(&mut rng);
    let (left, left_marks) = draw_figure(&mut rng);
    from_halves(right, right_marks, left, left_marks)
}

/// 由给定的右 / 左两半组装（便于按已知盘面复算与校验）。
#[must_use]
pub fn from_halves(right: u8, right_marks: [bool; 4], left: u8, left_marks: [bool; 4]) -> Odu {
    let right_name = PRINCIPAL_NAMES[(right & 0xF) as usize];
    let left_name = PRINCIPAL_NAMES[(left & 0xF) as usize];
    let meji = right == left;
    Odu {
        right,
        left,
        index: u16::from(right) * 16 + u16::from(left),
        right_marks,
        left_marks,
        right_name,
        left_name,
        name: if meji { format!("{right_name} Méjì") } else { format!("{right_name} {left_name}") },
        meji,
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
            assert_eq!(o.index, u16::from(o.right) * 16 + u16::from(o.left), "右在高位");
            assert!(o.left < 16 && o.right < 16);
            // 标记打包自洽。
            assert_eq!(u16::from(o.left), gf2::pack(&o.left_marks));
            assert_eq!(u16::from(o.right), gf2::pack(&o.right_marks));
            // 名与 figure 值对得上，复合名右先左后
            assert_eq!(o.right_name, PRINCIPAL_NAMES[o.right as usize]);
            assert_eq!(o.left_name, PRINCIPAL_NAMES[o.left as usize]);
            assert_eq!(o.meji, o.right == o.left);
            if o.meji {
                assert_eq!(o.name, format!("{} Méjì", o.right_name));
            } else {
                assert!(o.name.starts_with(o.right_name), "复合名应以右名起：{}", o.name);
                assert!(o.name.ends_with(o.left_name), "复合名应以左名收：{}", o.name);
            }
        }
    }

    /// Bascom 1969 Figure 2（p. 41）画出的那一盘，同时锁住行序、列序、命名序三件事。
    ///
    /// 图里右列自上而下是 `II II II I`（双双双单 = Bascom 记法 2221 = Ọkanran），
    /// 左列是 `I I II I`（单单双单 = 1121 = Irẹtẹ）。Bascom 在正文里点名：
    /// 「the figure is Ọkanran Irẹtẹ, and not Irẹtẹ Ọkanran. Because the latter is
    /// another figure, with a different set of predictions and sacrifices」。
    ///
    /// 行序、左右、命名任何一处弄反，这条都会红——而且弄反之后名字**仍然是合法 odù 名**，
    /// 正是需要 oracle 而不是靠肉眼的那种失效。
    #[test]
    fn bascom_figure_two_pins_row_order_column_order_and_naming() {
        // marks[0] = 顶行，true = 单画
        let right_marks = [false, false, false, true]; // 2 2 2 1
        let left_marks = [true, true, false, true]; //    1 1 2 1
        let right = (gf2::pack(&right_marks) & 0xF) as u8;
        let left = (gf2::pack(&left_marks) & 0xF) as u8;
        assert_eq!(bascom_notation(right), "2221", "右半的 Bascom 记法");
        assert_eq!(bascom_notation(left), "1121", "左半的 Bascom 记法");

        let o = from_halves(right, right_marks, left, left_marks);
        assert_eq!(o.right_name, "Ọ̀kànràn");
        assert_eq!(o.left_name, "Ìrẹtẹ̀");
        assert_eq!(o.name, "Ọ̀kànràn Ìrẹtẹ̀", "右名在前——反过来是另一个 odù");
        assert!(!o.meji);

        // 反过来确实是另一个 odù，序号也不同
        let swapped = from_halves(left, left_marks, right, right_marks);
        assert_eq!(swapped.name, "Ìrẹtẹ̀ Ọ̀kànràn");
        assert_ne!(swapped.index, o.index);
    }

    /// 16 主 odù 的名 ↔ 点阵，逐条对 Bascom Table 3 的四位串。
    #[test]
    fn the_sixteen_principal_names_match_bascoms_notation() {
        // (Bascom 四位串（顶行在最左，1=单 2=双）, 名)
        const ORACLE: [(&str, &str); 16] = [
            ("1111", "Ogbè"),
            ("2222", "Ọ̀yẹ̀kú"),
            ("2112", "Ìwòrì"),
            ("1221", "Òdí"),
            ("1122", "Ìrosùn"),
            ("2211", "Ọ̀wọ́nrín"),
            ("1222", "Ọ̀bàrà"),
            ("2221", "Ọ̀kànràn"),
            ("1112", "Ògúndá"),
            ("2111", "Ọ̀sá"),
            ("2122", "Ìká"),
            ("2212", "Òtúrúpọ̀n"),
            ("1211", "Òtúrá"),
            ("1121", "Ìrẹtẹ̀"),
            ("1212", "Ọ̀ṣẹ́"),
            ("2121", "Òfún"),
        ];
        for (notation, name) in ORACLE {
            let value = notation
                .chars()
                .enumerate()
                .fold(0u8, |acc, (i, c)| if c == '1' { acc | (1 << i) } else { acc });
            assert_eq!(PRINCIPAL_NAMES[value as usize], name, "{notation} 应是 {name}");
            assert_eq!(bascom_notation(value), notation, "{name} 的记法应回到 {notation}");
        }
        // 16 个名互不重复，且 16 个值全部用上
        let mut sorted = PRINCIPAL_NAMES;
        sorted.sort_unstable();
        let mut uniq = sorted.to_vec();
        uniq.dedup();
        assert_eq!(uniq.len(), 16);
    }

    #[test]
    fn covers_full_odu_space() {
        // 不同种子应铺满 256 odu 的相当一部分（抽样覆盖性）。
        let seen: std::collections::HashSet<u16> = (0..3000u64).map(|s| cast(s).index).collect();
        assert!(seen.len() > 200, "仅覆盖 {} 个 odu，期望铺满 256 之多数", seen.len());
    }
}
