//! 择日要素的校验：建除十二神与 28 宿值日。

/// 分档忠实于口诀「建满平收黑，除危定执黄，成开皆可用，破闭不可当」。
#[test]
fn day_grades_follow_the_mnemonic() {
    let by_name: Vec<(&str, DayGrade)> =
        (0..12u8).map(|i| (JIANCHU[i as usize], day_grade(i))).collect();
    for (name, want) in [
        ("除", DayGrade::Huang), ("危", DayGrade::Huang), ("定", DayGrade::Huang), ("执", DayGrade::Huang),
        ("成", DayGrade::Usable), ("开", DayGrade::Usable),
        ("建", DayGrade::Hei), ("满", DayGrade::Hei), ("平", DayGrade::Hei), ("收", DayGrade::Hei),
        ("破", DayGrade::Avoid), ("闭", DayGrade::Avoid),
    ] {
        let got = by_name.iter().find(|(n, _)| *n == name).map(|(_, g)| *g);
        assert_eq!(got, Some(want), "{name} 的分档");
    }
    // 12 神恰好铺满四档，且排序权重单调
    assert_eq!((0..12u8).filter(|&i| day_grade(i) == DayGrade::Huang).count(), 4);
    assert_eq!((0..12u8).filter(|&i| day_grade(i) == DayGrade::Avoid).count(), 2);
    assert!(DayGrade::Huang.rank() < DayGrade::Usable.rank());
    assert!(DayGrade::Usable.rank() < DayGrade::Hei.rank());
    assert!(DayGrade::Hei.rank() < DayGrade::Avoid.rank());
    assert!((0..12u8).all(|i| !day_grade(i).label().is_empty()));
}

use super::*;

#[test]
fn jianchu_is_a_twelve_cycle() {
    let set: std::collections::HashSet<_> = JIANCHU.iter().collect();
    assert_eq!(set.len(), 12);
    assert_eq!(JIANCHU[0], "建");
    assert_eq!(JIANCHU[11], "闭");
}

#[test]
fn jian_falls_on_month_branch_day() {
    // 建日：日支 == 月建支 → 建除位 0 = 建。
    for mb in 0u8..12 {
        assert_eq!(jianchu::position(mb, mb), 0);
        // 逐日顺行：日支比月建支大 k → 第 k 神。
        for k in 0u8..12 {
            let db = (mb + k) % 12;
            assert_eq!(jianchu::position(mb, db), k);
        }
    }
}

#[test]
fn month_branch_from_solar_terms() {
    // λ=315°（立春）→寅(2)；λ=345°（惊蛰）→卯(3)；λ=0°（春分附近，已过惊蛰）→卯(3)；
    // λ=45°（立夏）→巳(5)；λ=285°（小寒）→丑(1)。
    assert_eq!(month_branch(315.0), 2); // 寅
    assert_eq!(month_branch(345.0), 3); // 卯
    assert_eq!(month_branch(0.0), 3); // 仍卯月（惊蛰至清明）
    assert_eq!(month_branch(45.0), 5); // 巳
    assert_eq!(month_branch(285.0), 1); // 丑
    // 全 360° 扫描：月支恒在 0..12。
    let mut i = 0.0;
    while i < 360.0 {
        assert!(month_branch(i) < 12);
        i += 7.5;
    }
}

#[test]
fn mansions_are_28_distinct() {
    let set: std::collections::HashSet<_> = mansion::MANSIONS.iter().collect();
    assert_eq!(set.len(), 28);
    assert_eq!(mansion::MANSIONS[0], "角");
    assert_eq!(mansion::MANSIONS[27], "轸");
}

#[test]
fn mansion_anchors_cross_verified() {
    // 多锚点（跨 341 年、独立来源）校验：index = (JDN+11) mod 28，角=0。
    // 公历日 → 民用日序 → 值日宿。
    let cases = [
        (2026, 6, 14, "昴"),  // 三源一致，最强锚（实时日历）
        (2026, 6, 1, "心"),   // 两源
        (2026, 1, 1, "井"),   // rekichu 月历
        (2024, 1, 5, "鬼"),   // hotdoglab 2024 鬼宿日列表
    ];
    for (y, mo, d, want) in cases {
        let jdn = mingli_astro::civil_day_number(y, mo, d);
        let idx = mansion::index_for_jdn(jdn);
        assert_eq!(mansion::MANSIONS[idx], want, "{y}-{mo}-{d} 值日宿");
    }
    // 1685-02-04 贞享改历历元：正月朔 = 星宿（JDN 2336529）。
    assert_eq!(mansion::MANSIONS[mansion::index_for_jdn(2_336_529)], "星");
    // 连续性：逐日 +1（mod 28）。
    let j = mingli_astro::civil_day_number(2024, 1, 5);
    for k in 0..56 {
        let i0 = mansion::index_for_jdn(j + k);
        let i1 = mansion::index_for_jdn(j + k + 1);
        assert_eq!(i1, (i0 + 1) % 28);
    }
}

#[test]
fn mansion_weekday_phase_lock() {
    // 28=4×7 → 值日宿与星期严格同相位：房/虚/星/昴 恒为星期日（JDN%7==... 一致）。
    // 取四个该日，验证它们的 JDN mod 7 全相同。
    let sundays = ["房", "虚", "星", "昴"];
    let mut weekday = None;
    for k in 0..(28 * 6) {
        let jdn = 2_460_311 + k;
        let name = mansion::MANSIONS[mansion::index_for_jdn(jdn)];
        if sundays.contains(&name) {
            let w = jdn.rem_euclid(7);
            assert_eq!(*weekday.get_or_insert(w), w, "宿 {name} 应恒同一星期");
        }
    }
    assert!(weekday.is_some());
}

#[test]
fn compute_is_deterministic_and_well_formed() {
    let a = compute(2024, 6, 15, 14, 30, 8.0);
    let b = compute(2024, 6, 15, 14, 30, 8.0);
    assert_eq!(a.jianchu, b.jianchu);
    assert!((a.jianchu_pos as usize) < 12);
    assert!(a.day_branch < 12 && a.month_branch < 12);
    // 名与位一致。
    assert_eq!(a.jianchu, JIANCHU[a.jianchu_pos as usize]);
    assert!((a.mansion_index as usize) < 28);
    assert_eq!(a.mansion, mansion::MANSIONS[a.mansion_index as usize]);
    // 彭祖/天乙字段一致性。
    assert_eq!(a.pengzu_gan, pengzu::GAN[a.day_stem as usize]);
    assert_eq!(a.pengzu_zhi, pengzu::ZHI[a.day_branch as usize]);
    assert_eq!(a.tianyi_branches, tianyi::TIANYI[a.day_stem as usize]);
    assert!(a.day_stem < 10);
}

#[test]
fn pengzu_tables_well_formed() {
    // 22 句皆非空、不重复、首字与干支顺序对应。
    assert_eq!(pengzu::GAN.len(), 10);
    assert_eq!(pengzu::ZHI.len(), 12);
    let stems = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
    let branches = [
        "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
    ];
    for (i, s) in stems.iter().enumerate() {
        assert!(
            pengzu::GAN[i].starts_with(s),
            "干句 {i} 应以 {s} 起，实为 {}",
            pengzu::GAN[i]
        );
        assert!(pengzu::GAN[i].contains("不"), "干句 {i} 缺『不』");
    }
    for (i, b) in branches.iter().enumerate() {
        assert!(
            pengzu::ZHI[i].starts_with(b),
            "支句 {i} 应以 {b} 起，实为 {}",
            pengzu::ZHI[i]
        );
        assert!(pengzu::ZHI[i].contains("不"), "支句 {i} 缺『不』");
    }
    // 22 句两两不重（无录入失误）。
    let all: std::collections::HashSet<_> =
        pengzu::GAN.iter().chain(pengzu::ZHI.iter()).collect();
    assert_eq!(all.len(), 22);
}

#[test]
fn pengzu_oracle_lines() {
    // 通胜/钦定协纪辨方书通行版逐句校验：抽 4 句 + 1 句关键（辛不合酱）避免录入错。
    assert_eq!(pengzu::gan(0), "甲不开仓 财物耗散");
    assert_eq!(pengzu::gan(7), "辛不合酱 主人不尝");
    assert_eq!(pengzu::gan(9), "癸不词讼 理弱敌强");
    assert_eq!(pengzu::zhi(0), "子不问卜 自惹祸殃");
    assert_eq!(pengzu::zhi(7), "未不服药 毒气入肠");
    assert_eq!(pengzu::zhi(11), "亥不嫁娶 不利新郎");
}

#[test]
fn tianyi_table_classical_couplet() {
    // 《三命通会》「甲戊庚牛羊，乙己鼠猴乡，丙丁猪鸡位，六辛逢虎马，壬癸兔蛇藏」
    // 双地支恒不等、与日干 mod 群结构一致（甲戊庚同组、乙己同组、丙丁同组、壬癸同组、辛独）。
    for stem in 0..10u8 {
        let [a, b] = tianyi::branches_for(stem);
        assert_ne!(a, b, "stem {stem} 双贵人应不同支");
        assert!(a < 12 && b < 12);
    }
    // 甲(0)/戊(4)/庚(6) → 牛(1)、未(7)
    assert_eq!(tianyi::branches_for(0), [1, 7]);
    assert_eq!(tianyi::branches_for(4), [1, 7]);
    assert_eq!(tianyi::branches_for(6), [1, 7]);
    // 乙(1)/己(5) → 子(0)、申(8)
    assert_eq!(tianyi::branches_for(1), [0, 8]);
    assert_eq!(tianyi::branches_for(5), [0, 8]);
    // 丙(2)/丁(3) → 亥(11)、酉(9)
    assert_eq!(tianyi::branches_for(2), [11, 9]);
    assert_eq!(tianyi::branches_for(3), [11, 9]);
    // 辛(7) 独 → 寅(2)、午(6)
    assert_eq!(tianyi::branches_for(7), [2, 6]);
    // 壬(8)/癸(9) → 卯(3)、巳(5)
    assert_eq!(tianyi::branches_for(8), [3, 5]);
    assert_eq!(tianyi::branches_for(9), [3, 5]);
}

#[test]
fn cast_couplet_for_1990_06_15() {
    // 1990-06-15 14：30 CST 八字日柱 = 辛亥（见 ziwei/bazi oracle）。
    // → 干句 = 辛不合酱 主人不尝；支句 = 亥不嫁娶 不利新郎；
    //   天乙贵人（辛） = 寅、午。日干支名 = 辛亥。
    let c = compute(1990, 6, 15, 14, 30, 8.0);
    assert_eq!(c.day_ganzhi_name, "辛亥");
    assert_eq!(c.pengzu_gan, "辛不合酱 主人不尝");
    assert_eq!(c.pengzu_zhi, "亥不嫁娶 不利新郎");
    assert_eq!(c.tianyi_names, ["寅", "午"]);
}
