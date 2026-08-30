//! L3 叶（D 族 / 确定性）：希伯来 gematria 多计法。
//!
//! gematria 是「字符串 → 整数」的同态映射；同一词在不同计法（Mispar）下得不同值，
//! 互参不互替。本 crate 实现七种主流计法，多源对照式并出：
//!
//! - **Mispar Hechrachi**（标准值，最通行）：`א=1…י=10, כ=20…צ=90, ק=100…ת=400`；
//!   五尾形（`ך ם ן ף ץ`）取本形值（20/40/50/80/90）。
//! - **Mispar Gadol**（大值）：仅五尾形取 `500/600/700/800/900`，本形同 Hechrachi。
//! - **Mispar Siduri**（序数）：字母按字母表序 `א=1…ת=22`；五尾形仍取本形序数。
//! - **Mispar Katan**（小值）：每字母 Hechrachi `mod 9`（`0 → 9`），逐字相加；
//!   **不再** mod 9，故整词值可 > 9。
//! - **Mispar Katan Mispari**（"数字根" / Integral Reduced Value）：对整词 Hechrachi
//!   反复求各位之和直至落入 `1..=9`，等价 `1 + (n − 1) mod 9`（`n = 0 → 0`）。
//! - **AtBash**：字母替换码 `i ↔ (23 − i)`，替换后按 Hechrachi 求和。耶利米 25：26
//!   著名加密：`בבל`（Babel， Hechrachi 34）⇌ `ששך`（Sheshach， atbash-Hechrachi 620）。
//! - **AlBam**：字母二半互换 `1..=11 ↔ 12..=22`（aleph–kaf ↔ lamed–tav），
//!   替换后按 Hechrachi 求和。
//!
//! 非希伯来字符跳过。字母值与映射皆定义性（多源一致，见 oracle 测试）。
//! 语域注：数值是确定计算，释义不在此 crate。

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "0..22 的字母索引转 u64 在所有目标平台都不截断"
)]

#[cfg(feature = "port")]
mod engine;
#[cfg(feature = "port")]
pub use engine::GematriaEngine;

#[cfg(feature = "serde")]
use serde::Serialize;

/// 希伯来 22 本形字母（字母表序）。
const LETTERS: [char; 22] = [
    'א', 'ב', 'ג', 'ד', 'ה', 'ו', 'ז', 'ח', 'ט', 'י', 'כ', 'ל', 'מ', 'נ', 'ס', 'ע', 'פ', 'צ',
    'ק', 'ר', 'ש', 'ת',
];

/// 22 本形的 Mispar Hechrachi 值（个位 / 十位 / 百位 三段）。
const VALUES: [u64; 22] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 200, 300, 400,
];

/// 计法（Mispar method）。
///
/// 多种计法在 Kabbalah 与希伯来语言学中都是定义性的——`compute` 一次性给出全部七法值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Method {
    /// Mispar Hechrachi（标准值，尾形 = 本形）。
    Hechrachi,
    /// Mispar Gadol（大值，尾形 500–900）。
    Gadol,
    /// Mispar Siduri（序数 1..=22，尾形 = 本形）。
    Siduri,
    /// Mispar Katan（小值：逐字母 Hechrachi mod 9， `0 → 9`，逐字相加；不再 mod 9）。
    Katan,
    /// Mispar Katan Mispari（数字根：整词 Hechrachi 反复求各位之和至 1..=9）。
    KatanMispari,
    /// AtBash 替换后取 Hechrachi（`i ↔ (23 − i)`）。
    AtBash,
    /// AlBam 替换后取 Hechrachi（前半 1..=11 ↔ 后半 12..=22）。
    AlBam,
}

/// 把字符归一到 22 本形的索引（`0..22`）；非希伯来字母返回 `None`。
///
/// 五个尾形（ך ם ן ף ץ）归一到其本形（כ מ נ פ צ）。
#[must_use]
fn base_index(c: char) -> Option<usize> {
    let normalized = match c {
        'ך' => 'כ',
        'ם' => 'מ',
        'ן' => 'נ',
        'ף' => 'פ',
        'ץ' => 'צ',
        _ => c,
    };
    LETTERS.iter().position(|&x| x == normalized)
}

/// AtBash 字母对换索引：`i → 21 − i`（0-based，对应 1-based 的 `i ↔ 23 − i`）。
#[must_use]
const fn atbash_index(i: usize) -> usize {
    21 - i
}

/// AlBam 字母对换索引：前半 0..=10 ↔ 后半 11..=21。
#[must_use]
const fn albam_index(i: usize) -> usize {
    if i < 11 {
        i + 11
    } else {
        i - 11
    }
}

/// 单字符的 gematria 值；非希伯来字母返回 `None`。
///
/// 注：[`Method::KatanMispari`] 是**整词**约化（不是逐字属性），
/// 对单字符调用此函数与 [`Method::Katan`] 等价。
#[must_use]
pub fn letter_value(c: char, method: Method) -> Option<u64> {
    match method {
        Method::Hechrachi => base_index(c).map(|i| VALUES[i]),
        Method::Gadol => match c {
            'ך' => Some(500),
            'ם' => Some(600),
            'ן' => Some(700),
            'ף' => Some(800),
            'ץ' => Some(900),
            _ => base_index(c).map(|i| VALUES[i]),
        },
        Method::Siduri => base_index(c).map(|i| i as u64 + 1),
        Method::Katan | Method::KatanMispari => base_index(c).map(|i| {
            let r = VALUES[i] % 9;
            if r == 0 {
                9
            } else {
                r
            }
        }),
        Method::AtBash => base_index(c).map(|i| VALUES[atbash_index(i)]),
        Method::AlBam => base_index(c).map(|i| VALUES[albam_index(i)]),
    }
}

/// 整词按指定计法求值（非希伯来字符跳过）。
#[must_use]
pub fn gematria(word: &str, method: Method) -> u64 {
    if matches!(method, Method::KatanMispari) {
        return digital_root_nine(gematria(word, Method::Hechrachi));
    }
    word.chars().filter_map(|c| letter_value(c, method)).sum()
}

/// 数字根（落入 `1..=9`，`0 → 0`）：`1 + (n − 1) mod 9`。
#[must_use]
const fn digital_root_nine(n: u64) -> u64 {
    if n == 0 {
        0
    } else {
        1 + (n - 1) % 9
    }
}

/// 一次 gematria 换算结果（七种计法并出，对照式输出）。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Cast {
    /// 标准值（Mispar Hechrachi）。
    pub hechrachi: u64,
    /// 大值（Mispar Gadol，尾形 500–900）。
    pub gadol: u64,
    /// 序数（Mispar Siduri，字母表序 1..=22）。
    pub siduri: u64,
    /// 小值（Mispar Katan，逐字 mod 9 后求和）。
    pub katan: u64,
    /// 数字根（Mispar Katan Mispari，整词 Hechrachi → 1..=9）。
    pub katan_mispari: u64,
    /// AtBash 替换后的 Hechrachi（`i ↔ 23 − i`）。
    pub atbash: u64,
    /// AlBam 替换后的 Hechrachi（前半 1..=11 ↔ 后半 12..=22）。
    pub albam: u64,
}

/// 同时计算所有七法（对照式输出）。
#[must_use]
pub fn compute(word: &str) -> Cast {
    Cast {
        hechrachi: gematria(word, Method::Hechrachi),
        gadol: gematria(word, Method::Gadol),
        siduri: gematria(word, Method::Siduri),
        katan: gematria(word, Method::Katan),
        katan_mispari: gematria(word, Method::KatanMispari),
        atbash: gematria(word, Method::AtBash),
        albam: gematria(word, Method::AlBam),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 七种计法的字母值，逐条对两个独立来源。
    ///
    /// 值表是这一片叶的全部——写错一个字母，所有含它的词都错，且错得不会有任何迹象。
    /// 两源逐条一致：
    ///
    /// - Jewish Virtual Library《Gematria》(jewishvirtuallibrary.org/gematria-2)：
    ///   「absolute or normative value … Alef equals 1, bet equals 2 … until yod, the tenth
    ///   letter, which equals 10. The next letter, kaf, equals 20」；尾形「500, 600, 700,
    ///   800, and 900, respectively」；「ordinal value, where each of the 22 letters … a number
    ///   between 1 and 22」；「reduced value … accomplished by removing the value of 10 or 100」。
    /// - TorahCalc《Explanations of Gematria Methods with Charts》(torahcalc.com/info/gematria)：
    ///   给出六种计法的完整对照表（Hechrachi / Gadol / Siduri / Katan / AtBash / AlBam），
    ///   AtBash「exchanges each letter's value for its opposite letter's value」，
    ///   AlBam「splits the alphabet in half and letters from the first half switch values with
    ///   letters from the second half」。
    ///
    /// 两源对**尾形在标准值下如何处理**说法一致（取本形值；取 500–900 的是 Gadol），
    /// 这一点是最容易两派分歧的地方，故单列 [`final_forms_normalize_in_hechrachi`] 钉住。
    #[test]
    fn hechrachi_classic_oracles() {
        assert_eq!(gematria("חי", Method::Hechrachi), 18); // 生命
        assert_eq!(gematria("שלום", Method::Hechrachi), 376); // 平安
        assert_eq!(gematria("אמת", Method::Hechrachi), 441); // 真理
    }

    #[test]
    fn final_forms_normalize_in_hechrachi() {
        for (sofit, base) in [('ך', 'כ'), ('ם', 'מ'), ('ן', 'נ'), ('ף', 'פ'), ('ץ', 'צ')] {
            assert_eq!(
                letter_value(sofit, Method::Hechrachi),
                letter_value(base, Method::Hechrachi)
            );
        }
    }

    #[test]
    fn gadol_final_forms_take_500_900() {
        for (sofit, expect) in [('ך', 500), ('ם', 600), ('ן', 700), ('ף', 800), ('ץ', 900)] {
            assert_eq!(letter_value(sofit, Method::Gadol), Some(expect));
        }
        for &c in &LETTERS {
            assert_eq!(letter_value(c, Method::Gadol), letter_value(c, Method::Hechrachi));
        }
        // שלום Gadol = 300 + 30 + 6 + 600(ם) = 936
        assert_eq!(gematria("שלום", Method::Gadol), 936);
    }

    #[test]
    fn siduri_letter_index() {
        for (i, &c) in LETTERS.iter().enumerate() {
            assert_eq!(letter_value(c, Method::Siduri), Some(i as u64 + 1));
        }
        // 尾形 = 本形
        assert_eq!(letter_value('ם', Method::Siduri), letter_value('מ', Method::Siduri));
        // חי = 8 + 10 = 18；שלום = 21 + 12 + 6 + 13 = 52；אמת = 1 + 13 + 22 = 36
        assert_eq!(gematria("חי", Method::Siduri), 18);
        assert_eq!(gematria("שלום", Method::Siduri), 52);
        assert_eq!(gematria("אמת", Method::Siduri), 36);
    }

    #[test]
    fn katan_per_letter_mod9() {
        // חי: 8 + (10%9=1) = 9; שלום: 3 + 3 + 6 + 4 = 16; אמת: 1 + 4 + 4 = 9
        assert_eq!(gematria("חי", Method::Katan), 9);
        assert_eq!(gematria("שלום", Method::Katan), 16);
        assert_eq!(gematria("אמת", Method::Katan), 9);
        // 每字母 Katan ∈ 1..=9
        for &c in &LETTERS {
            let v = letter_value(c, Method::Katan).unwrap();
            assert!((1..=9).contains(&v));
        }
    }

    #[test]
    fn katan_mispari_digital_root() {
        // 整词 Hechrachi 的数字根。
        assert_eq!(gematria("חי", Method::KatanMispari), 9); // 18 → 9
        assert_eq!(gematria("שלום", Method::KatanMispari), 7); // 376 → 16 → 7
        assert_eq!(gematria("אמת", Method::KatanMispari), 9); // 441 → 9
        assert_eq!(gematria("", Method::KatanMispari), 0); // 空词 → 0
        // 数字根 ≡ Hechrachi mod 9（非零时 0 → 9）
        for w in ["חי", "שלום", "אמת", "בראשית", "אלהים"] {
            let h = gematria(w, Method::Hechrachi);
            let k = gematria(w, Method::KatanMispari);
            assert_eq!(k, if h == 0 { 0 } else { 1 + (h - 1) % 9 });
        }
    }

    #[test]
    fn atbash_sheshach_oracle() {
        // 耶利米 25：26：בבל（Babylon， Hechrachi 34）的 atbash 加密为 ששך（Sheshach）；
        // sofit ך 在 Hechrachi 下 = 本形 כ，故 atbash 数值 = 300 + 300 + 20 = 620。
        assert_eq!(gematria("בבל", Method::Hechrachi), 34);
        assert_eq!(gematria("בבל", Method::AtBash), 620);
    }

    #[test]
    fn albam_pairing_oracle() {
        // אבג → לםנ（aleph→lamed, bet→mem, gimel→nun）；Hechrachi 6 vs AlBam 120
        assert_eq!(gematria("אבג", Method::Hechrachi), 6);
        assert_eq!(gematria("אבג", Method::AlBam), 120);
        // אמת AlBam = ל(30) + ב(2) + כ(20) = 52
        assert_eq!(gematria("אמת", Method::AlBam), 52);
    }

    #[test]
    fn substitutions_are_involutions() {
        // AtBash / AlBam 都是 22 字母上的对合：套两次回原字母。
        for (i, &c) in LETTERS.iter().enumerate() {
            assert_eq!(atbash_index(atbash_index(i)), i);
            assert_eq!(albam_index(albam_index(i)), i);
            // 字面验证：value(atbash²(c)) == value(c)
            let a1_val = letter_value(c, Method::AtBash).unwrap();
            let a1_char = LETTERS[VALUES.iter().position(|&v| v == a1_val).unwrap()];
            assert_eq!(letter_value(a1_char, Method::AtBash), letter_value(c, Method::Hechrachi));
            let l1_val = letter_value(c, Method::AlBam).unwrap();
            let l1_char = LETTERS[VALUES.iter().position(|&v| v == l1_val).unwrap()];
            assert_eq!(letter_value(l1_char, Method::AlBam), letter_value(c, Method::Hechrachi));
        }
    }

    #[test]
    fn non_hebrew_skipped_in_all_methods() {
        for m in [
            Method::Hechrachi,
            Method::Gadol,
            Method::Siduri,
            Method::Katan,
            Method::AtBash,
            Method::AlBam,
        ] {
            assert_eq!(letter_value('A', m), None);
            assert_eq!(letter_value('5', m), None);
            assert_eq!(gematria("", m), 0);
        }
        // 含空格 / 拉丁字符仅累加希伯来字母。
        assert_eq!(gematria("חי 18!", Method::Hechrachi), 18);
        assert_eq!(gematria("שלום world", Method::Siduri), 52);
    }

    #[test]
    fn compute_all_methods_oracle_shalom() {
        // שלום 的全 7 法值（推导式校验）：
        // - Hechrachi 300+30+6+40 = 376
        // - Gadol     300+30+6+600 = 936
        // - Siduri    21+12+6+13 = 52
        // - Katan     3+3+6+4 = 16
        // - KatanMispari digital_root(376) = 7
        // - AtBash: ש→ב(2), ל→כ(20), ו→פ(80), ם→מ→י(10)   sum=112
        // - AlBam : ש→י(10), ל→א(1), ו→פ(80), ם→מ→ב(2)    sum=93
        let c = compute("שלום");
        assert_eq!(c.hechrachi, 376);
        assert_eq!(c.gadol, 936);
        assert_eq!(c.siduri, 52);
        assert_eq!(c.katan, 16);
        assert_eq!(c.katan_mispari, 7);
        assert_eq!(c.atbash, 112);
        assert_eq!(c.albam, 93);
    }

    #[test]
    fn full_alphabet_hechrachi_values_match_table() {
        for (i, &c) in LETTERS.iter().enumerate() {
            assert_eq!(letter_value(c, Method::Hechrachi), Some(VALUES[i]));
        }
    }

    #[test]
    fn katan_mispari_is_single_digit() {
        // 凡整词 Hechrachi > 0 的，KatanMispari 总落 1..=9
        for w in ["א", "חי", "שלום", "אמת", "בראשית"] {
            let r = gematria(w, Method::KatanMispari);
            assert!((1..=9).contains(&r), "word={w} root={r}");
        }
    }
}
