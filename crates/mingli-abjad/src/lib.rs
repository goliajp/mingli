//! L3 叶（D 族 / 确定性）：阿拉伯 abjad 数字（ḥisāb al-jummal）多序变体。
//!
//! abjad 把阿拉伯 28 字母按古闪族字序赋值 `1..1000` 并求和（同态 Σ*→ℤ）。本 crate
//! 实现两大主流字母序，结果对照式并出：
//!
//! - **Mashriqī（东方序）**：阿拉伯东方/中东通行，
//!   `ا=1 ب=2 ج=3 د=4 ه=5 و=6 ز=7 ح=8 ط=9 ي=10 ك=20 ل=30 م=40 ن=50
//!    س=60 ع=70 ف=80 ص=90 ق=100 ر=200 ش=300 ت=400 ث=500 خ=600 ذ=700
//!    ض=800 ظ=900 غ=1000`。
//! - **Maghribī（西方/北非序）**：安达卢西亚‐马格里布传统。22 个字母与 Mashriqī 同，
//!   六个差异——`س=300 ش=1000 ص=60 ض=90 ظ=800 غ=900`（其余位置完全相同）。
//!
//! **校验**：
//! - `الله`（Allah）= 1+30+30+5 = 66 — 两序皆 66（字母不在差异区）。
//! - `بسم`（"In the name of"）= 102 (Mashriqī) / 342 (Maghribī)（`س`：60↔300）。
//! - `محمد`（Muhammad）= 92 — 两序皆 92。
//! - `شمس`（sun）= 400 (Mashriqī) / 1340 (Maghribī)（差异区 ش + س）。
//!
//! 另对常见书写变体做**标准归一**（多数 abjad 计算器一致）：hamza 各形（أ إ آ ء ئ ؤ）与
//! taa marbuta（ة）、alef maqsura（ى）归到其本字母。其余字符（空格/标点/拉丁/数字）跳过。
//!
//! 诚实边界（🟡）：taa marbuta 归到 ه(5) 是常见约定，少数计法按发音作 ت(400)——本 crate
//! 取 5 并在此标注。

#[cfg(feature = "port")]
mod engine;
#[cfg(feature = "port")]
pub use engine::AbjadEngine;

#[cfg(feature = "serde")]
use serde::Serialize;

/// 阿拉伯 abjad 字母序变体。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Order {
    /// Mashriqī（东方序，通行）：`س=60 ش=300 ص=90 ض=800 ظ=900 غ=1000`。
    Mashriqi,
    /// Maghribī（西方/北非序）：`س=300 ش=1000 ص=60 ض=90 ظ=800 غ=900`。
    Maghribi,
}

/// 单字符 abjad 值（指定序）；非阿拉伯字母（含已归一变体外）返回 `None`。
#[must_use]
pub fn letter_value_in(c: char, order: Order) -> Option<u64> {
    // 先按 Mashriqī 表识别字符（含书写变体归一），再按 order remap 差异字母。
    let mashriqi = match c {
        'ا' | 'أ' | 'إ' | 'آ' | 'ء' => 1,
        'ب' => 2,
        'ج' => 3,
        'د' => 4,
        'ه' | 'ة' => 5,
        'و' | 'ؤ' => 6,
        'ز' => 7,
        'ح' => 8,
        'ط' => 9,
        'ي' | 'ئ' | 'ى' => 10,
        'ك' => 20,
        'ل' => 30,
        'م' => 40,
        'ن' => 50,
        'س' => 60,
        'ع' => 70,
        'ف' => 80,
        'ص' => 90,
        'ق' => 100,
        'ر' => 200,
        'ش' => 300,
        'ت' => 400,
        'ث' => 500,
        'خ' => 600,
        'ذ' => 700,
        'ض' => 800,
        'ظ' => 900,
        'غ' => 1000,
        _ => return None,
    };
    Some(match order {
        Order::Mashriqi => mashriqi,
        Order::Maghribi => maghribi_remap(mashriqi),
    })
}

/// 把 Mashriqī 值映射到 Maghribī 值。
///
/// 仅六个字母在两序间换值，其余 22 个字母值相同：
/// `س 60↔300, ش 300↔1000, ص 90↔60, ض 800↔90, ظ 900↔800, غ 1000↔900`。
#[must_use]
const fn maghribi_remap(mashriqi_value: u64) -> u64 {
    match mashriqi_value {
        60 => 300,
        90 => 60,
        300 => 1000,
        800 => 90,
        900 => 800,
        1000 => 900,
        v => v,
    }
}

/// 单字符 abjad 值（默认 Mashriqī 序）。
#[must_use]
pub fn letter_value(c: char) -> Option<u64> {
    letter_value_in(c, Order::Mashriqi)
}

/// 整词 abjad（指定序）。
#[must_use]
pub fn abjad_in(word: &str, order: Order) -> u64 {
    word.chars().filter_map(|c| letter_value_in(c, order)).sum()
}

/// 整词 abjad（默认 Mashriqī 序）。
#[must_use]
pub fn abjad(word: &str) -> u64 {
    abjad_in(word, Order::Mashriqi)
}

/// 一次 abjad 换算结果（两序对照式并出）。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Cast {
    /// Mashriqī 序总值（东方/通行序）。
    pub mashriqi: u64,
    /// Maghribī 序总值（西方/北非序）。
    pub maghribi: u64,
}

/// 同时计算两种字母序的 abjad 值。
#[must_use]
pub fn compute(word: &str) -> Cast {
    Cast {
        mashriqi: abjad_in(word, Order::Mashriqi),
        maghribi: abjad_in(word, Order::Maghribi),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_mashriqi_oracles() {
        assert_eq!(abjad("الله"), 66);
        assert_eq!(abjad("بسم"), 102);
    }

    #[test]
    fn maghribi_oracles_at_differing_letters() {
        // 字母都不在差异区 → 两序同。
        assert_eq!(abjad_in("الله", Order::Maghribi), 66);
        assert_eq!(abjad_in("محمد", Order::Maghribi), 92);
        // 差异区：س 60→300
        assert_eq!(abjad_in("بسم", Order::Maghribi), 2 + 300 + 40);
        // 差异区：ش+م+س = 1000+40+300 vs 300+40+60
        assert_eq!(abjad_in("شمس", Order::Mashriqi), 400);
        assert_eq!(abjad_in("شمس", Order::Maghribi), 1340);
        // 差异区：ض 800→90 （+ و=6 + ء→ا=1， 不变区）
        assert_eq!(abjad_in("ضوء", Order::Mashriqi), 807);
        assert_eq!(abjad_in("ضوء", Order::Maghribi), 97);
        // 差异区：ظ 900→800 (+ ف=80 + ر=200)
        assert_eq!(abjad_in("ظفر", Order::Mashriqi), 1180);
        assert_eq!(abjad_in("ظفر", Order::Maghribi), 1080);
        // 差异区：غ 1000→900 (+ س 60→300 + ل=30)
        assert_eq!(abjad_in("غسل", Order::Mashriqi), 1090);
        assert_eq!(abjad_in("غسل", Order::Maghribi), 1230);
    }

    #[test]
    fn maghribi_remap_six_letters_only() {
        // 仅 6 字母两序换值；其余 22 字母两序相同。
        let all = "ابجدهوزحطيكلمنسعفصقرشتثخذضظغ";
        let mut diff_count = 0;
        for c in all.chars() {
            let m = letter_value_in(c, Order::Mashriqi).unwrap();
            let g = letter_value_in(c, Order::Maghribi).unwrap();
            if m != g {
                diff_count += 1;
                // 双向映射：Maghribi(Mashriqi(c)) 必属差异表
                assert!(
                    matches!((m, g), (60, 300) | (90, 60) | (300, 1000) | (800, 90) | (900, 800) | (1000, 900)),
                    "unexpected remap {c}: {m}→{g}",
                );
            }
        }
        assert_eq!(diff_count, 6);
    }

    #[test]
    fn mashriqi_block_structure_still_correct() {
        // 个位 1-9。
        let units = "ابجدهوزحط";
        let uv: Vec<u64> = units.chars().map(|c| letter_value(c).unwrap()).collect();
        assert_eq!(uv, (1..=9).collect::<Vec<_>>());
        // 十位 10-90。
        let tens = "يكلمنسعفص";
        let tv: Vec<u64> = tens.chars().map(|c| letter_value(c).unwrap()).collect();
        assert_eq!(tv, (1..=9).map(|n| n * 10).collect::<Vec<_>>());
        // 百位 100-900。
        let hundreds = "قرشتثخذضظ";
        let hv: Vec<u64> = hundreds.chars().map(|c| letter_value(c).unwrap()).collect();
        assert_eq!(hv, (1..=9).map(|n| n * 100).collect::<Vec<_>>());
        // 千：غ=1000（Mashriqī）。
        assert_eq!(letter_value('غ'), Some(1000));
        // Mashriqī 全 28 字母值唯一覆盖 {1..9, 10..90, 100..900, 1000}。
        let all = "ابجدهوزحطيكلمنسعفصقرشتثخذضظغ";
        let set: std::collections::HashSet<u64> =
            all.chars().map(|c| letter_value(c).unwrap()).collect();
        assert_eq!(set.len(), 28);
    }

    #[test]
    fn maghribi_block_structure_same_codomain() {
        // Maghribī 序值域与 Mashriqī 完全相同（仅是字母→值的双射重排）。
        let all = "ابجدهوزحطيكلمنسعفصقرشتثخذضظغ";
        let m_set: std::collections::HashSet<u64> = all
            .chars()
            .map(|c| letter_value_in(c, Order::Mashriqi).unwrap())
            .collect();
        let g_set: std::collections::HashSet<u64> = all
            .chars()
            .map(|c| letter_value_in(c, Order::Maghribi).unwrap())
            .collect();
        assert_eq!(m_set, g_set);
        assert_eq!(g_set.len(), 28);
    }

    #[test]
    fn variant_normalization_works_for_both_orders() {
        // 书写变体归一在两序下都成立。
        for order in [Order::Mashriqi, Order::Maghribi] {
            for c in ['أ', 'إ', 'آ', 'ء'] {
                assert_eq!(letter_value_in(c, order), Some(1));
            }
            assert_eq!(letter_value_in('ؤ', order), Some(6));
            assert_eq!(letter_value_in('ئ', order), Some(10));
            assert_eq!(letter_value_in('ى', order), Some(10));
            assert_eq!(letter_value_in('ة', order), Some(5));
        }
    }

    #[test]
    fn non_arabic_skipped() {
        for order in [Order::Mashriqi, Order::Maghribi] {
            assert_eq!(letter_value_in('A', order), None);
            assert_eq!(letter_value_in('7', order), None);
            assert_eq!(abjad_in("", order), 0);
        }
        assert_eq!(abjad("الله!  (66)"), 66);
    }

    #[test]
    fn compute_outputs_both_orders() {
        let c = compute("بسم");
        assert_eq!(c.mashriqi, 102);
        assert_eq!(c.maghribi, 342);
        // 全不变区字符串：两序值相等
        let c = compute("الله");
        assert_eq!(c.mashriqi, c.maghribi);
        assert_eq!(c.mashriqi, 66);
    }
}
