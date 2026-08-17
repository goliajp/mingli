//! 六十四卦格的校验：八卦、上下卦与错综互之。

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
