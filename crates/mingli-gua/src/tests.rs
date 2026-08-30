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

/// 八卦三张表逐格钉住，外加一条来自 Unicode 的独立对照。
///
/// 上面那条只问了 `Trigram(7).symbol()`，而取符号时的 `& 0b111` 若写成 `| 0b111`，
/// 下标恒为 7、返回的永远是 ☰——恰好是被问的那一个。八格都问才看得出来。
#[test]
fn every_trigram_has_its_own_name_symbol_and_number() {
    // 值即三爻二进制，bit0 = 初爻，1 = 阳。先天数取自宋人「乾一兑二离三震四，
    // 巽五坎六艮七坤八」。
    const ROWS: [(u8, &str, &str, u8); 8] = [
        (0, "坤", "☷", 8),
        (1, "震", "☳", 4),
        (2, "坎", "☵", 6),
        (3, "兑", "☱", 2),
        (4, "艮", "☶", 7),
        (5, "离", "☲", 3),
        (6, "巽", "☴", 5),
        (7, "乾", "☰", 1),
    ];
    for (v, name, sym, xt) in ROWS {
        assert_eq!(Trigram(v).name(), name, "值 {v} 的卦名");
        assert_eq!(Trigram(v).symbol(), sym, "值 {v} 的卦象符号");
        assert_eq!(Trigram(v).xiantian(), xt, "值 {v} 的先天数");
    }
    // 三张表各自不重样——重了就有两卦共用一个格子。
    let (mut names, mut syms, mut nums) = (
        std::collections::HashSet::new(),
        std::collections::HashSet::new(),
        std::collections::HashSet::new(),
    );
    for (_, name, sym, xt) in ROWS {
        assert!(names.insert(name), "卦名 {name} 出现两次");
        assert!(syms.insert(sym), "符号 {sym} 出现两次");
        assert!(nums.insert(xt), "先天数 {xt} 出现两次");
    }
    // 独立对照：Unicode 的 Yijing Trigram Symbols 区段 U+2630..=U+2637 按
    // 乾兑离震巽坎艮坤 排列，也就是把 7−v 的三位二进制倒过来读。符号表若与它对不上，
    // 要么我们的表错了，要么符号根本不是那个字。
    for (v, _, sym, _) in ROWS {
        let rev = ((7 - v) & 1) << 2 | ((7 - v) & 2) | ((7 - v) >> 2);
        let cp = char::from_u32(0x2630 + u32::from(rev)).unwrap();
        assert_eq!(sym, cp.to_string(), "值 {v} 的符号与 Unicode 排序对不上");
    }
}

/// 卦象字 → 卦值：十六个字逐个问。
///
/// 这张表只在编译期派生 64 卦全名时走过，而那 64 个名字用不到它的多数分支组合，
/// 于是「哪个字得哪个值」从来没有被单独问过一次。这条问的是认得的字答得对不对；
/// 「除了这些字一个都不认」是下面那条的事，两条各管一半。
#[test]
fn every_xiang_character_maps_to_its_trigram() {
    for (s, want) in [
        // 八个卦象别名
        ("天", 7u8), ("地", 0), ("雷", 1), ("风", 6),
        ("水", 2), ("火", 5), ("山", 4), ("泽", 3),
        // 八卦本字
        ("乾", 7), ("坤", 0), ("震", 1), ("巽", 6),
        ("坎", 2), ("离", 5), ("艮", 4), ("兑", 3),
    ] {
        assert_eq!(trigram_from_xiang(s.as_bytes()), want, "「{s}」应得 {want}");
    }
}

/// 这十六个字之外，一个都不许认。
///
/// 上面那条只问它「认得的字答得对吗」，而它的注释还说了另一半：表里出现生字要当场
/// 炸掉。二十个把某条 `&&` 放松成 `||` 的变异体全活在这一半上——放松后多认的字
/// 恰好都不在那十六个里，逐字问永远问不到。这里把整个 CJK 基本区扫一遍，
/// 要求被接受的**正好**是那十六个。
#[test]
fn nothing_but_those_sixteen_characters_is_accepted() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {})); // 两万次拒绝不必刷屏
    let mut accepted: Vec<char> = Vec::new();
    for cp in 0x4E00..=0x9FFFu32 {
        let Some(ch) = char::from_u32(cp) else { continue };
        let mut buf = [0u8; 4];
        let bytes = ch.encode_utf8(&mut buf).as_bytes();
        if bytes.len() != 3 {
            continue;
        }
        let owned = [bytes[0], bytes[1], bytes[2]];
        if std::panic::catch_unwind(|| trigram_from_xiang(&owned)).is_ok() {
            accepted.push(ch);
        }
    }
    std::panic::set_hook(prev);
    let mut want: Vec<char> = "天地雷风水火山泽乾坤震巽坎离艮兑".chars().collect();
    want.sort_unstable();
    accepted.sort_unstable();
    assert_eq!(
        accepted.iter().collect::<String>(),
        want.iter().collect::<String>(),
        "被接受的字集变了"
    );
}

/// 「X 为 Y」的判定要认准「为」这个字，不是它字节里的某一段。
///
/// 纯卦与非纯卦走两条完全不同的路：前者取首字重叠成上下卦，后者取头两字分别作上下卦。
/// 判定写成三个字节的连比，把其中任何一个 `&&` 放松成 `||` 都不会改动那 64 个真名字的
/// 归类，于是双射、锚点、配对一概看不出来。要一个第二字并非「为」、字节却与它有交集的
/// 名字才问得出来——「中」是 E4 B8 AD，与「为」E4 B8 BA 只差末字节。
#[test]
fn the_pure_hexagram_test_keys_on_the_character_not_a_byte() {
    // 真名字先各走一遍：纯卦上下同卦，非纯卦上下不同。
    assert_eq!(value_from_full_name("乾为天"), 0b111_111);
    assert_eq!(value_from_full_name("坤为地"), 0b000_000);
    assert_eq!(value_from_full_name("坎为水"), 0b010_010);
    assert_eq!(value_from_full_name("天风姤"), (7 << 3) | 6);
    assert_eq!(value_from_full_name("风天小畜"), (6 << 3) | 7);
    // 第二字不是「为」，就不该走纯卦那条路；此处「中」不是卦象字，应当炸。
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let got = std::panic::catch_unwind(|| value_from_full_name("乾中天"));
    std::panic::set_hook(prev);
    assert!(got.is_err(), "「乾中天」被当成了纯卦，得 {got:?}——「为」的判定放宽了");
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
    // 上面两条都被「恒返六阳」满足——挑的正是坏实现也答得对的两个点。爻自下而上数，
    // 位 i 即第 i+1 爻，要混合卦才问得出这回事。
    assert_eq!(Hexagram(0b000000).lines(), [false; 6]); // 坤为地
    assert_eq!(Hexagram(0b000001).lines(), [true, false, false, false, false, false]);
    assert_eq!(Hexagram(0b100000).lines(), [false, false, false, false, false, true]);
    assert_eq!(Hexagram(0b010101).lines(), [true, false, true, false, true, false]);
    assert_eq!(Hexagram(0b101010).lines(), [false, true, false, true, false, true]);
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
