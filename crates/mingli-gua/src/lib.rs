//! L2 主干：六十四卦格 (Z₂)⁶。
//!
//! 一爻为一个二进制位（阳=1，阴=0），自下而上：初爻=最低位（bit0），上爻=最高位（bit5）。
//! 八卦 = (Z₂)³（8 个），重卦 = 上卦×下卦 = (Z₂)⁶（64 个）。错/综/互/之卦皆为该位向量上的
//! 确定性变换，建于 [`mingli_core::gf2`]。
//!
//! **64 卦名 + 文王卦序**：由「上卦象+下卦象+卦名」传统全名(`HEXAGRAM_FULL_NAMES`)
//! 编码，经 ctext.org《序卦传》+ Wikipedia（中/英）+《周易正义》孔颖达疏 三源完全一致。
//! `HEXAGRAM_NAMES`（简称） / `HEXAGRAM_FULL_NAMES`（传统全名）按文王序 1..64 排列；`name_of(value)`
//! 由 binary value 反查简称，`wenwang_index_of(value)` 反查文王序。所有映射经穷举测试。

#![allow(
    clippy::cast_possible_truncation,
    reason = "重卦仅低 6 位有效，u16(gf2) → u8 窄化安全"
)]
#![allow(
    clippy::unreadable_literal,
    reason = "卦的二进制位型（如 0b010110）连写比加分隔符更直观对应六爻"
)]

use mingli_core::gf2;
use serde::Serialize;

/// 八卦名（按八卦值 0..7 索引；值 = 三爻二进制，初爻为最低位）。
pub const TRIGRAM_NAMES: [&str; 8] = ["坤", "震", "坎", "兑", "艮", "离", "巽", "乾"];
/// 八卦卦象符号（按值 0..7 索引）。
pub const TRIGRAM_SYMBOLS: [&str; 8] = ["☷", "☳", "☵", "☱", "☶", "☲", "☴", "☰"];
/// 先天八卦数（乾1兑2离3震4巽5坎6艮7坤8），按八卦值 0..7 索引。
pub const TRIGRAM_XIANTIAN: [u8; 8] = [8, 4, 6, 2, 7, 3, 5, 1];

/// 一个八卦（三爻），低 3 位有效（初爻=bit0）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Trigram(pub u8);

impl Trigram {
    /// 卦名。
    #[must_use]
    pub fn name(self) -> &'static str {
        TRIGRAM_NAMES[(self.0 & 0b111) as usize]
    }
    /// 卦象符号。
    #[must_use]
    pub fn symbol(self) -> &'static str {
        TRIGRAM_SYMBOLS[(self.0 & 0b111) as usize]
    }
    /// 先天八卦数（1..8）。
    #[must_use]
    pub fn xiantian(self) -> u8 {
        TRIGRAM_XIANTIAN[(self.0 & 0b111) as usize]
    }
}

/// 一个重卦（六爻），低 6 位有效（初爻=bit0，上爻=bit5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Hexagram(pub u8);

impl Hexagram {
    /// 由上卦、下卦组成重卦。
    #[must_use]
    pub fn from_trigrams(upper: Trigram, lower: Trigram) -> Self {
        Hexagram((lower.0 & 0b111) | ((upper.0 & 0b111) << 3))
    }
    /// 下卦（初/二/三爻）。
    #[must_use]
    pub fn lower(self) -> Trigram {
        Trigram(self.0 & 0b111)
    }
    /// 上卦（四/五/上爻）。
    #[must_use]
    pub fn upper(self) -> Trigram {
        Trigram((self.0 >> 3) & 0b111)
    }
    /// 六爻自下而上，`true`=阳。
    #[must_use]
    pub fn lines(self) -> [bool; 6] {
        let mut l = [false; 6];
        for (i, slot) in l.iter_mut().enumerate() {
            *slot = (self.0 >> i) & 1 == 1;
        }
        l
    }
    /// 错卦（旁通）：六爻全变 = 与 `0b111111` 异或。
    #[must_use]
    pub fn opposite(self) -> Self {
        Hexagram(gf2::xor(u16::from(self.0), 0b111111) as u8)
    }
    /// 综卦（反卦）：将卦上下颠倒读 = 六位逆序。
    #[must_use]
    pub fn reversed(self) -> Self {
        let mut r = 0u8;
        for i in 0..6 {
            if (self.0 >> i) & 1 == 1 {
                r |= 1 << (5 - i);
            }
        }
        Hexagram(r)
    }
    /// 互卦（中爻卦）：下卦取本卦 2、3、4 爻，上卦取本卦 3、4、5 爻。
    #[must_use]
    pub fn mutual(self) -> Self {
        let lower = (self.0 >> 1) & 0b111; // 2，3，4 爻
        let upper = (self.0 >> 2) & 0b111; // 3，4，5 爻
        Hexagram(lower | (upper << 3))
    }
    /// 之卦（变卦）：按变爻掩码 `mask`（bit_i=1 表第 i+1 爻为变爻）翻转得变卦。
    #[must_use]
    pub fn changed(self, mask: u8) -> Self {
        Hexagram(gf2::xor(u16::from(self.0), u16::from(mask & 0b111111)) as u8)
    }
    /// 卦的传统简称（乾/坤/屯/..）；见 [`HEXAGRAM_NAMES`]。
    #[must_use]
    pub fn name(self) -> &'static str {
        HEXAGRAM_NAMES[self.king_wen() as usize - 1]
    }
    /// 卦的传统全名（乾为天/坤为地/水雷屯/..）；见 [`HEXAGRAM_FULL_NAMES`]。
    #[must_use]
    pub fn full_name(self) -> &'static str {
        HEXAGRAM_FULL_NAMES[self.king_wen() as usize - 1]
    }
    /// 文王卦序 1..=64。
    #[must_use]
    pub fn king_wen(self) -> u8 {
        KING_WEN_OF_VALUE[(self.0 & 0b111111) as usize]
    }
}

/// 64 卦的传统简称，按**文王卦序** 1..=64 排列（下标 = `king_wen − 1`）。
///
/// 三源完全一致（ctext.org《序卦传》+ zh.wikipedia 六十四卦 + en.wikipedia King_Wen_sequence）。
/// 用「无」非「無」（易经传统固定写法，ctext《序卦传》第 25）；用「噬嗑」非「噬磕」（口字旁，通行本）。
pub const HEXAGRAM_NAMES: [&str; 64] = [
    "乾", "坤", "屯", "蒙", "需", "讼", "师", "比",
    "小畜", "履", "泰", "否", "同人", "大有", "谦", "豫",
    "随", "蛊", "临", "观", "噬嗑", "贲", "剥", "复",
    "无妄", "大畜", "颐", "大过", "坎", "离", "咸", "恒",
    "遁", "大壮", "晋", "明夷", "家人", "睽", "蹇", "解",
    "损", "益", "夬", "姤", "萃", "升", "困", "井",
    "革", "鼎", "震", "艮", "渐", "归妹", "丰", "旅",
    "巽", "兑", "涣", "节", "中孚", "小过", "既济", "未济",
];

/// 64 卦的传统全名（"上卦象+下卦象+卦名" 或纯卦 "X 为 Y"），按文王序排列。
///
/// 卦象→八卦映射：天=乾、地=坤、雷=震、风=巽、水=坎、火=离、山=艮、泽=兑。
/// `HEXAGRAM_FULL_NAMES[kw−1]` 由首字（上卦象）+ 第二字（下卦象）派生本卦 binary value
/// （见 [`KING_WEN_VALUES`]），允许多源校验。
pub const HEXAGRAM_FULL_NAMES: [&str; 64] = [
    "乾为天", "坤为地", "水雷屯", "山水蒙", "水天需", "天水讼", "地水师", "水地比",
    "风天小畜", "天泽履", "地天泰", "天地否", "天火同人", "火天大有", "地山谦", "雷地豫",
    "泽雷随", "山风蛊", "地泽临", "风地观", "火雷噬嗑", "山火贲", "山地剥", "地雷复",
    "天雷无妄", "山天大畜", "山雷颐", "泽风大过", "坎为水", "离为火", "泽山咸", "雷风恒",
    "天山遁", "雷天大壮", "火地晋", "地火明夷", "风火家人", "火泽睽", "水山蹇", "雷水解",
    "山泽损", "风雷益", "泽天夬", "天风姤", "泽地萃", "地风升", "泽水困", "水风井",
    "泽火革", "火风鼎", "震为雷", "艮为山", "风山渐", "雷泽归妹", "雷火丰", "火山旅",
    "巽为风", "兑为泽", "风水涣", "水泽节", "风泽中孚", "雷山小过", "水火既济", "火水未济",
];

/// 64 卦的 binary value（按文王序排列；`KING_WEN_VALUES[kw−1] = value`）。
///
/// 由 [`HEXAGRAM_FULL_NAMES`] 在 `const fn` 中机械派生(纯函数：上下卦象→Trigram value→重卦
/// value = `(upper << 3) | lower`)，不凭记忆硬编。
pub const KING_WEN_VALUES: [u8; 64] = {
    let mut v = [0u8; 64];
    let mut i = 0;
    while i < 64 {
        v[i] = value_from_full_name(HEXAGRAM_FULL_NAMES[i]);
        i += 1;
    }
    v
};

/// 反向映射：binary value 0..64 → 文王序 1..=64。由 [`KING_WEN_VALUES`] 反置。
pub const KING_WEN_OF_VALUE: [u8; 64] = {
    let mut r = [0u8; 64];
    let mut kw = 0;
    while kw < 64 {
        r[KING_WEN_VALUES[kw] as usize] = (kw + 1) as u8;
        kw += 1;
    }
    r
};

/// 由八卦象名（中文 UTF-8 单字）返回 Trigram value 0..8。
///
/// 字节映射（全部 3-byte UTF-8 字符）：
/// 八卦：乾=7、坤=0、震=1、巽=6、坎=2、离=5、艮=4、兑=3
/// 卦象别名：天=乾、地=坤、雷=震、风=巽、水=坎、火=离、山=艮、泽=兑。
const fn trigram_from_xiang(s: &[u8]) -> u8 {
    let (b0, b1, b2) = (s[0], s[1], s[2]);
    if b0 == 0xE5 && b1 == 0xA4 && b2 == 0xA9 { 7 } // 天=乾
    else if b0 == 0xE5 && b1 == 0x9C && b2 == 0xB0 { 0 } // 地=坤
    else if b0 == 0xE9 && b1 == 0x9B && b2 == 0xB7 { 1 } // 雷=震
    else if b0 == 0xE9 && b1 == 0xA3 && b2 == 0x8E { 6 } // 风=巽
    else if b0 == 0xE6 && b1 == 0xB0 && b2 == 0xB4 { 2 } // 水=坎
    else if b0 == 0xE7 && b1 == 0x81 && b2 == 0xAB { 5 } // 火=离
    else if b0 == 0xE5 && b1 == 0xB1 && b2 == 0xB1 { 4 } // 山=艮
    else if b0 == 0xE6 && b1 == 0xB3 && b2 == 0xBD { 3 } // 泽=兑
    else if b0 == 0xE4 && b1 == 0xB9 && b2 == 0xBE { 7 } // 乾（本字）
    else if b0 == 0xE5 && b1 == 0x9D && b2 == 0xA4 { 0 } // 坤（本字）
    else if b0 == 0xE9 && b1 == 0x9C && b2 == 0x87 { 1 } // 震（本字）
    else if b0 == 0xE5 && b1 == 0xB7 && b2 == 0xBD { 6 } // 巽（本字）
    else if b0 == 0xE5 && b1 == 0x9D && b2 == 0x8E { 2 } // 坎（本字）
    else if b0 == 0xE7 && b1 == 0xA6 && b2 == 0xBB { 5 } // 离（本字）
    else if b0 == 0xE8 && b1 == 0x89 && b2 == 0xAE { 4 } // 艮（本字）
    else if b0 == 0xE5 && b1 == 0x85 && b2 == 0x91 { 3 } // 兑（本字）
    // const 上下文求值：表里出现生字会直接编译失败，运行期到不了这里。
    else { panic!("trigram_from_xiang： 未识别的卦象字") }
}

/// 由传统全名派生 binary value（纯函数，const-eval 编译期）。
///
/// "X 为 Y"（纯卦）：取首字 X，作上下两卦（如「乾为天」=乾上乾下=63）。
/// 其它（"上卦象+下卦象+卦名"）：首字=上卦象，第二字=下卦象；value=(upper<<3)|lower。
const fn value_from_full_name(full: &str) -> u8 {
    let b = full.as_bytes();
    // 「为」 UTF-8 = E4 B8 BA；若第 4-6 字节（第二字）为「为」，即纯卦「X 为 Y」
    if b[3] == 0xE4 && b[4] == 0xB8 && b[5] == 0xBA {
        let t = trigram_from_xiang(b);
        (t << 3) | t
    } else {
        let upper = trigram_from_xiang(&[b[0], b[1], b[2]]);
        let lower = trigram_from_xiang(&[b[3], b[4], b[5]]);
        (upper << 3) | lower
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigram_table() {
        // 乾=值7（三阳）、坤=值0（三阴），先天数乾1坤8。
        assert_eq!(Trigram(7).name(), "乾");
        assert_eq!(Trigram(7).symbol(), "☰");
        assert_eq!(Trigram(7).xiantian(), 1);
        assert_eq!(Trigram(0).name(), "坤");
        assert_eq!(Trigram(0).xiantian(), 8);
        assert_eq!(Trigram(2).name(), "坎"); // 010
        assert_eq!(Trigram(5).name(), "离"); // 101
    }

    #[test]
    fn hexagram_split_roundtrip() {
        for v in 0..64u8 {
            let h = Hexagram(v);
            assert_eq!(Hexagram::from_trigrams(h.upper(), h.lower()), h);
        }
    }

    #[test]
    fn lines_bottom_up() {
        // 乾为天 = 六阳 = 0b111111
        assert_eq!(Hexagram(0b111111).lines(), [true; 6]);
        assert!(Hexagram(0b000001).lines()[0]); // 仅初爻为阳
    }

    #[test]
    fn opposite_is_full_flip() {
        assert_eq!(Hexagram(63).opposite(), Hexagram(0)); // 乾↔坤
        assert_eq!(Hexagram(0).opposite(), Hexagram(63));
        for v in 0..64u8 {
            assert_eq!(Hexagram(v).opposite().opposite(), Hexagram(v));
        }
    }

    #[test]
    fn reversed_is_bit_reversal() {
        assert_eq!(Hexagram(0b000001).reversed(), Hexagram(0b100000));
        assert_eq!(Hexagram(63).reversed(), Hexagram(63)); // 对称卦自反
        for v in 0..64u8 {
            assert_eq!(Hexagram(v).reversed().reversed(), Hexagram(v));
        }
    }

    #[test]
    fn mutual_nuclear() {
        assert_eq!(Hexagram(63).mutual(), Hexagram(63)); // 乾互乾
        assert_eq!(Hexagram(0).mutual(), Hexagram(0)); // 坤互坤
        // v=0b010001：下卦核=(v>>1)&7=0（坤），上卦核=(v>>2)&7=4（艮） → 0|4<<3=32
        assert_eq!(Hexagram(0b010001).mutual(), Hexagram(32));
    }

    #[test]
    fn changed_flips_marked_lines() {
        assert_eq!(Hexagram(0).changed(0b000001), Hexagram(1)); // 初爻变
        assert_eq!(Hexagram(0b111111).changed(0b111111), Hexagram(0)); // 六爻全变
        assert_eq!(Hexagram(0b101010).changed(0), Hexagram(0b101010)); // 无变爻
    }

    /// 64 卦名表非空 + 简称 ≤ 2 字符（实际 1-2 字）+ 全名 = 3 个 UTF-8 中文字符。
    #[test]
    fn name_tables_well_formed() {
        assert_eq!(HEXAGRAM_NAMES.len(), 64);
        assert_eq!(HEXAGRAM_FULL_NAMES.len(), 64);
        assert_eq!(KING_WEN_VALUES.len(), 64);
        assert_eq!(KING_WEN_OF_VALUE.len(), 64);
        for (i, name) in HEXAGRAM_NAMES.iter().enumerate() {
            assert!(!name.is_empty(), "kw {} 简称空", i + 1);
            assert!(name.chars().count() <= 2, "kw {} 简称过长： {}", i + 1, name);
            // 全名 3 字：「乾为天」型（1 字简称）或「水雷屯」型（1 字简称 = 末字）；
            // 4 字：「风天小畜」/「火雷噬嗑」型（2 字简称 = 末两字）。
            let full_len = HEXAGRAM_FULL_NAMES[i].chars().count();
            assert!(
                full_len == 3 || full_len == 4,
                "kw {} 全名 {} 长度 {} 应 ∈ {{3， 4}}",
                i + 1,
                HEXAGRAM_FULL_NAMES[i],
                full_len
            );
        }
    }

    /// 简称在全名中作为后缀出现（全名末尾 = 简称）。同时校验「噬嗑」「大畜」「大壮」等复合简称的全名末缀严格匹配。
    #[test]
    fn full_name_ends_with_short_name() {
        // 通用规则：简称 = 全名的最后 1-2 字。「乾」「坤」「坎」「离」「震」「艮」「巽」「兑」八纯卦因为
        // 用「X 为 Y」结构，简称即首字 X，故末字不是简称。其余 56 卦末缀严格 = 简称。
        let pure = ["乾", "坤", "坎", "离", "震", "艮", "巽", "兑"];
        for (i, name) in HEXAGRAM_NAMES.iter().enumerate() {
            let full = HEXAGRAM_FULL_NAMES[i];
            if pure.contains(name) {
                assert!(full.starts_with(name), "纯卦 {full} 应以 {name} 开头");
            } else {
                assert!(full.ends_with(name), "{full} 应以 {name} 结尾");
            }
        }
    }

    /// 64 卦的 binary value 双射：`KING_WEN_VALUES` 覆盖 0..64 各一次。
    #[test]
    fn king_wen_values_are_bijection() {
        let mut seen = [false; 64];
        for &v in &KING_WEN_VALUES {
            assert!(!seen[v as usize], "value {v} 在 KING_WEN_VALUES 中重复");
            seen[v as usize] = true;
        }
        assert!(seen.iter().all(|&x| x), "KING_WEN_VALUES 未覆盖全 64");
        // 反向同构
        for v in 0..64u8 {
            let kw = KING_WEN_OF_VALUE[v as usize];
            assert!((1..=64).contains(&kw), "value {v} 的文王序 {kw} 越界");
            assert_eq!(KING_WEN_VALUES[(kw - 1) as usize], v, "反向不一致 @ v={v}");
        }
    }

    /// 关键 oracle：核心几卦的 binary value 与文王序一致。
    #[test]
    fn hexagram_oracle_anchors() {
        // 乾 = 文王 1， value = 0b111111 = 63
        assert_eq!(KING_WEN_VALUES[0], 63);
        assert_eq!(Hexagram(63).name(), "乾");
        assert_eq!(Hexagram(63).full_name(), "乾为天");
        assert_eq!(Hexagram(63).king_wen(), 1);

        // 坤 = 文王 2， value = 0
        assert_eq!(KING_WEN_VALUES[1], 0);
        assert_eq!(Hexagram(0).name(), "坤");

        // 屯 = 文王 3 = 水雷屯 = 坎上震下 = (2<<3)|1 = 17
        assert_eq!(KING_WEN_VALUES[2], 17);
        assert_eq!(Hexagram(17).name(), "屯");
        assert_eq!(Hexagram(17).full_name(), "水雷屯");

        // 蒙 = 文王 4 = 山水蒙 = 艮上坎下 = (4<<3)|2 = 34
        assert_eq!(KING_WEN_VALUES[3], 34);

        // 坎（为水） = 文王 29， value = (2<<3)|2 = 18
        assert_eq!(KING_WEN_VALUES[28], 18);
        assert_eq!(Hexagram(18).name(), "坎");

        // 离（为火） = 文王 30， value = (5<<3)|5 = 45
        assert_eq!(KING_WEN_VALUES[29], 45);
        assert_eq!(Hexagram(45).name(), "离");

        // 既济 = 文王 63 = 水火既济 = 坎上离下 = (2<<3)|5 = 21
        assert_eq!(KING_WEN_VALUES[62], 21);
        assert_eq!(Hexagram(21).name(), "既济");

        // 未济 = 文王 64 = 火水未济 = 离上坎下 = (5<<3)|2 = 42
        assert_eq!(KING_WEN_VALUES[63], 42);
        assert_eq!(Hexagram(42).name(), "未济");
    }

    /// 「二二相耦，非覆即变」：32 对里恰 4 对纯错卦（乾坤/颐大过/坎离/中孚小过），
    /// 其余 28 对皆为综卦（覆，允许 4 对综错同形，如泰否/随蛊/渐归妹/既济未济）。
    /// 三源一致（en.wiki/zh.wiki/孔颖达《周易正义·序卦》）。
    #[test]
    fn king_wen_pair_property() {
        // 4 对纯错卦
        let pure_error_pairs: [(u8, u8); 4] = [(1, 2), (27, 28), (29, 30), (61, 62)];
        for (a, b) in pure_error_pairs {
            let va = KING_WEN_VALUES[(a - 1) as usize];
            let vb = KING_WEN_VALUES[(b - 1) as usize];
            assert_eq!(
                Hexagram(va).opposite(),
                Hexagram(vb),
                "{a}-{b} 应为纯错（全爻变）对"
            );
        }
        // 其余 28 对应为综卦(reversed)。若同时也是错卦（综错同形），仍满足综。
        for k in 1..=32u8 {
            let a = 2 * k - 1;
            let b = 2 * k;
            if pure_error_pairs.contains(&(a, b)) {
                continue;
            }
            let va = KING_WEN_VALUES[(a - 1) as usize];
            let vb = KING_WEN_VALUES[(b - 1) as usize];
            assert_eq!(
                Hexagram(va).reversed(),
                Hexagram(vb),
                "{a}-{b} 应为综卦（综覆）对；va={va} vb={vb}"
            );
        }
    }

    /// `value_from_full_name` 运行时穷举 64 行，确认与 const 派生的 `KING_WEN_VALUES` 一致。
    /// 同时 runtime 走「X 为 Y」与「[上][下][卦名]」两分支。
    #[test]
    fn value_from_full_name_runtime() {
        for i in 0..64 {
            assert_eq!(
                value_from_full_name(HEXAGRAM_FULL_NAMES[i]),
                KING_WEN_VALUES[i],
                "kw {} 全名 {} 派生值不一致",
                i + 1,
                HEXAGRAM_FULL_NAMES[i]
            );
        }
    }

    /// `trigram_from_xiang` 穷举两组 16 个字符，覆盖所有运行时分支（panic 分支不可达留）。
    #[test]
    fn trigram_from_xiang_covers_all_inputs() {
        // 卦象别名 8 个
        let aliases = [
            ("天", 7u8), ("地", 0), ("雷", 1), ("风", 6),
            ("水", 2), ("火", 5), ("山", 4), ("泽", 3),
        ];
        for (s, expected) in aliases {
            assert_eq!(trigram_from_xiang(s.as_bytes()), expected, "alias {s}");
        }
        // 八卦本字 8 个
        let proper = [
            ("乾", 7u8), ("坤", 0), ("震", 1), ("巽", 6),
            ("坎", 2), ("离", 5), ("艮", 4), ("兑", 3),
        ];
        for (s, expected) in proper {
            assert_eq!(trigram_from_xiang(s.as_bytes()), expected, "proper {s}");
        }
    }

    /// 简称无重复（64 卦名唯一）。
    #[test]
    fn names_are_unique() {
        let mut sorted: Vec<&str> = HEXAGRAM_NAMES.to_vec();
        sorted.sort_unstable();
        let n = sorted.len();
        sorted.dedup();
        assert_eq!(sorted.len(), n, "简称有重复");
    }
}
