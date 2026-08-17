//! 数字学的校验：生命灵数、两套字母表的姓名数、主数不约化。

use super::*;

#[test]
fn chaldean_table_groups() {
    // 1：AIJQY 2：BKR 3：CGLS 4：DMT 5：EHNX 6：UVW 7：OZ 8：FP；无 9。
    for c in ['A', 'I', 'J', 'Q', 'Y'] {
        assert_eq!(chaldean(c), Some(1));
    }
    assert_eq!(chaldean('B'), Some(2));
    assert_eq!(chaldean('F'), Some(8));
    assert_eq!(chaldean('O'), Some(7));
    assert_eq!(chaldean('Z'), Some(7));
    assert_eq!(chaldean('5'), None);
    assert_eq!(chaldean('a'), Some(1)); // 大小写一致
    // Chaldean 永不产出 9。
    assert!(('A'..='Z').all(|c| chaldean(c) != Some(9)));
}

/// 两个流派互为对方的 alt：选谁，谁就是主值，另一个挂在 `life_path_alt` 上。
/// 这条同时钉住「本叶不替用户选边」——两说都在输出里。
#[test]
fn both_life_path_schools_are_reported_whichever_is_chosen() {
    let m = Moment::new(1980, 6, 15, 12, 0, 8.0);
    let comp = compute_at_with(&m, LifePathMethod::Component);
    let whole = compute_at_with(&m, LifePathMethod::WholeSum);
    assert_eq!(comp.life_path_method, "component");
    assert_eq!(whole.life_path_method, "whole_sum");
    // 互为主副：一边的主值就是另一边的备选
    assert_eq!(comp.life_path, whole.life_path_alt);
    assert_eq!(whole.life_path, comp.life_path_alt);
    // 缺省入口走 Component
    assert_eq!(compute_at(&m).life_path_method, comp.life_path_method);
}

/// Y 归属的三说：语境派（4 独立源）用 Decoz 的八条位置细则逐条对，
/// 「跟在元音后仍算元音」一支（2 独立源）用它自己举的例子对。
#[test]
fn the_three_y_conventions_each_match_the_examples_their_sources_give() {
    use YRule::{AfterVowel, Contextual, Never};
    let v = |name: &str, rule: YRule| -> Vec<bool> {
        let flags = vowel_flags(name, rule);
        name.chars().zip(flags).filter(|(c, _)| c.eq_ignore_ascii_case(&'y')).map(|(_, f)| f).collect()
    };

    // —— Decoz 八条（World Numerology），逐条对 ——
    // 1 首字母 + 后接辅音 → 元音
    for n in ["Yvonne", "Ylsa", "Yvette"] {
        assert_eq!(v(n, Contextual), vec![true], "{n}");
    }
    // 2 末字母 + 前为辅音 → 元音
    for n in ["Barry", "Tommy", "Jimmy"] {
        assert_eq!(v(n, Contextual), vec![true], "{n}");
    }
    // 3 首字母 + 后接元音 → 辅音
    for n in ["Yolanda", "Yammy"] {
        assert!(!v(n, Contextual)[0], "{n}");
    }
    // 4 末字母 + 前为元音 → 辅音
    for n in ["Mulrooney", "Mickey"] {
        assert_eq!(v(n, Contextual), vec![false], "{n}");
    }
    // 5 夹在两辅音之间 → 元音
    for n in ["Kyle", "Tyson"] {
        assert_eq!(v(n, Contextual), vec![true], "{n}");
    }
    // 6 夹在两元音之间 → 辅音
    assert_eq!(v("Eyarta", Contextual), vec![false]);
    // 7 / 8 一侧是元音 → 取辅音（Decoz 的 default，Token Rock「紧挨元音即辅音」同）
    assert_eq!(v("Maya", Contextual), vec![false]);
    assert_eq!(v("Troy", Contextual), vec![false]);
    assert_eq!(v("Wayne", Contextual), vec![false]);

    // —— AfterVowel 一支：跟在元音后面仍算元音（Lyn's / Astrala 举的例）——
    for n in ["Clayton", "Taylor", "May"] {
        assert_eq!(v(n, AfterVowel), vec![true], "{n}");
    }
    // 但后接元音时两说一致取辅音
    assert!(!v("Yolanda", AfterVowel)[0]);
    // 无元音相邻时两说也一致
    assert_eq!(v("Lynn", AfterVowel), v("Lynn", Contextual));

    // —— Never：一律辅音 ——
    for n in ["Yvonne", "Barry", "Kyle", "Clayton"] {
        assert!(v(n, Never).iter().all(|f| !f), "{n}");
    }
}

/// 词与词之间不相邻：空格断开后，Mary 的 Y 后面没有字母。
#[test]
fn adjacency_does_not_reach_across_a_space() {
    assert!(vowel_flags("Mary Ann", YRule::Contextual)[3], "Mary 的 Y 应算元音");
    // 若错误地跨词看邻居，后面是空格再后是 A，可能被误判
    assert!(vowel_flags("Mary anne", YRule::Contextual)[3]);
}

/// 三读并出：表达数不随 Y 归属变，灵魂 / 人格随之变，且主值 = 语境派。
#[test]
fn all_three_readings_are_reported_side_by_side() {
    let n = name_numbers("Barry", System::Pythagorean);
    assert_eq!(n.by_y_rule.len(), 3);
    assert_eq!(n.by_y_rule[0].y_rule, "contextual");
    assert_eq!((n.soul_urge, n.personality), (n.by_y_rule[0].soul_urge, n.by_y_rule[0].personality));
    // Barry 的 Y 在语境派算元音、在 Never 算辅音，两读必不同
    let never = n.by_y_rule.iter().find(|r| r.y_rule == "never").expect("三说齐全");
    assert_ne!(n.soul_urge, never.soul_urge, "Barry 含 Y，两说的灵魂数应不同");
    // 表达数与 Y 归属无关
    assert_eq!(n.expression, expression("Barry", System::Pythagorean));
    // 不含 Y 的名字三读必然相同
    let plain = name_numbers("Abel", System::Pythagorean);
    assert!(plain.by_y_rule.iter().all(|r| r.soul_urge == plain.soul_urge));
}

#[test]
fn master_numbers_stop_the_reduction() {
    assert_eq!(reduce_with_master(29), 11); // 2+9=11，停
    assert_eq!(reduce_with_master(38), 11); // 3+8=11
    assert_eq!(reduce_with_master(40), 4);
    assert_eq!(reduce_with_master(33), 33);
    assert_eq!(reduce_with_master(0), 0);
}

#[test]
fn pythagorean_letters() {
    assert_eq!(pythagorean('A'), Some(1));
    assert_eq!(pythagorean('I'), Some(9));
    assert_eq!(pythagorean('J'), Some(1));
    assert_eq!(pythagorean('Z'), Some(8)); // (25%9)+1=8
    assert_eq!(pythagorean('5'), None);
    assert_eq!(mingli_core::ringhash::string_sum("ABC", pythagorean), 6);
}

#[test]
fn pythagorean_via_ringhash() {
    assert_eq!(letter_value('A', System::Pythagorean), Some(1));
    assert_eq!(letter_value('I', System::Pythagorean), Some(9));
    assert_eq!(letter_value('J', System::Pythagorean), Some(1));
    assert_eq!(letter_value('Z', System::Pythagorean), Some(8));
}

#[test]
fn life_path_worked_examples() {
    // 1990-06-15：年 1990→1+9+9+0=19→10→1；月 6；日 15→6；1+6+6=13→4。
    assert_eq!(life_path(1990, 6, 15), 4);
    // 流派对比：1990-06-15 → component 法 4，whole_sum 法 1+9+9+0+6+1+5=31→4（同值）。
    assert_eq!(life_path_with(1990, 6, 15, LifePathMethod::WholeSum), 4);
    // 1989-12-26 区分两派：
    //   Component: y=1+9+8+9=27→9, m=12→3, d=26→8 → 9+3+8=20→2
    //   WholeSum：  1+9+8+9+1+2+2+6=38→11（保留主数）
    assert_eq!(life_path_with(1989, 12, 26, LifePathMethod::Component), 2);
    assert_eq!(life_path_with(1989, 12, 26, LifePathMethod::WholeSum), 11);
    // 主数保留：某日期约化得 11 应停。2000-11-29：年2000→2，月11→11（停），日29→11（停）；2+11+11=24→6。
    assert_eq!(life_path(2000, 11, 29), 6);
    // 直接给出主数和示例：reduce_with_master 在求和处保留。
    // 1998-08-13：年1998→1+9+9+8=27→9；月8；日13→4；9+8+4=21→3。
    assert_eq!(life_path(1998, 8, 13), 3);
    // 边界：年 0（数字和=0）不 panic：0+1+1=2。
    assert_eq!(life_path(0, 1, 1), 2);
}

#[test]
fn birthday_number_reduces() {
    assert_eq!(birthday_number(15), 6);
    assert_eq!(birthday_number(29), 11); // 主数停
    assert_eq!(birthday_number(4), 4);
}

#[test]
fn name_numbers_pythagorean() {
    // "ABE" Pythagorean：A1 B2 E5 → 8（表达）。元音 A，E=1+5=6（灵魂）。辅音 B=2（人格）。
    let n = name_numbers("ABE", System::Pythagorean);
    assert_eq!(n.expression, 8);
    assert_eq!(n.soul_urge, 6);
    assert_eq!(n.personality, 2);
    // 非字母被跳过。
    assert_eq!(expression("A-B-E", System::Pythagorean), 8);
}

#[test]
fn name_numbers_chaldean_differs() {
    // "FOX" Chaldean：F8 O7 X5 = 20 → 2。Pythagorean：F6 O6 X6=18→9。两系统不同。
    assert_eq!(expression("FOX", System::Chaldean), 2);
    assert_eq!(expression("FOX", System::Pythagorean), 9);
}

#[test]
fn vowels_and_master_preserved() {
    assert!(is_vowel('a') && is_vowel('U'));
    assert!(!is_vowel('Y') && !is_vowel('B'));
    // 约化保留主数：构造和为 29 的名 → 表达数 11。
    // "K" =2(P).. 取一个和=29 的串：用 "INNN"？ I9 N5 N5 N5=24. 用 "RRR..." 略，直接验 reduce。
    assert_eq!(reduce_with_master(29), 11);
}

#[test]
fn compute_paths() {
    let c = compute(1990, 6, 15, 8.0);
    assert_eq!(c.life_path, 4);
    assert!(c.pythagorean.is_none());
    let m = Moment::new(1990, 6, 15, 12, 0, 8.0);
    let cn = compute_named(&m, "Ada");
    assert_eq!(cn.life_path, 4);
    assert!(cn.pythagorean.is_some() && cn.chaldean.is_some());
    // 两系统对同名给不同表达数（除非碰巧相等）。
    let p = cn.pythagorean.unwrap();
    let ch = cn.chaldean.unwrap();
    assert_eq!(p.system, System::Pythagorean);
    assert_eq!(ch.system, System::Chaldean);
    // 确定性。
    assert_eq!(expression("Ada", System::Pythagorean), p.expression);
}
