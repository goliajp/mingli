//! 大六壬的校验：古法工作例、全表课数对账、井栏等专课。

use super::plates::{heaven_plate, month_general_branch, plate_offset, STEM_LODGING};
use super::transmission::{branch_is_yang, derive_transmission, down_controls_up, shehai_depth, stem_is_yang, up_controls_down};
use super::*;

#[test]
fn worked_example_hai_general_zi_hour_jiazi_day() {
    // 古法工作例：亥将(11)子时(0)甲子日（干甲=0，支子=0）。
    // offset = (11-0) mod 12 = 11。四课上神应为 丑(1)/子(0)/亥(11)/戌(10)。
    let offset = plate_offset(11, 0);
    assert_eq!(offset, 11);
    let courses = four_courses(0, 0, offset);
    // 甲寄寅(2)，寅上见丑(1)→一课。
    assert_eq!(courses[0], Course { down: 2, up: 1 });
    // 丑(1)上见子(0)→二课。
    assert_eq!(courses[1], Course { down: 1, up: 0 });
    // 日支子(0)上见亥(11)→三课。
    assert_eq!(courses[2], Course { down: 0, up: 11 });
    // 亥(11)上见戌(10)→四课。
    assert_eq!(courses[3], Course { down: 11, up: 10 });
}

#[test]
fn month_general_from_sun_longitude() {
    // λ 刚过雨水(330)→亥(11)=登明；λ∈[0，30)→戌(10)；λ∈[300，330)→子(0)。
    assert_eq!(month_general_branch(331.0), 11);
    assert_eq!(MONTH_GENERAL_NAMES[month_general_branch(331.0) as usize], "登明");
    assert_eq!(month_general_branch(15.0), 10); // 河魁
    assert_eq!(month_general_branch(310.0), 0); // 神后
    // 全 360° 扫描：月将恒在 0..12。
    let mut x = 0.0;
    while x < 360.0 {
        assert!(month_general_branch(x) < 12);
        x += 5.0;
    }
}

#[test]
fn lodging_table_no_four_cardinals() {
    // 四正（子0午6卯3酉9）不作寄宫。
    for &b in &STEM_LODGING {
        assert!(![0u8, 3, 6, 9].contains(&b), "寄宫不应落四正： {b}");
    }
    // 丙戊同寄巳(5)、丁己同寄未(7)。
    assert_eq!(STEM_LODGING[2], STEM_LODGING[4]); // 丙=戊=巳
    assert_eq!(STEM_LODGING[3], STEM_LODGING[5]); // 丁=己=未
}

#[test]
fn heaven_plate_is_z12_rotation() {
    // 天盘是地盘的纯平移：12 宫各异、双射。
    for offset in 0..12u8 {
        let set: std::collections::HashSet<u8> = (0..12).map(|g| heaven_plate(g, offset)).collect();
        assert_eq!(set.len(), 12);
    }
}

#[test]
fn transmission_valid_when_present() {
    // 扫描多日多时辰：凡给出三传者，三传皆合法地支、且中末传由层层取上神自洽。
    for day in 1..=28u32 {
        for hour in [0u32, 6, 12, 18] {
            let c = compute(2024, 3, day, hour, 30, 8.0);
            // 课式总有；课式名稳定。
            let _ = c.pattern;
            if let Some(t) = c.transmission {
                assert!(t.iter().all(|&b| b < 12));
                // 层层取上神的那几门：中传 = 初传上神、末传 = 中传上神。
                // 昴星 / 别责 / 八专不走这条——它们的中末取干上神与支上神。
                if !matches!(c.pattern, Pattern::MaoXing | Pattern::BieZe | Pattern::BaZhuan) {
                    assert_eq!(t[1], heaven_plate(t[0], c.offset));
                    assert_eq!(t[2], heaven_plate(t[1], c.offset));
                }
            }
            // 天盘双射。
            let set: std::collections::HashSet<u8> = c.heaven.iter().copied().collect();
            assert_eq!(set.len(), 12);
        }
    }
}

#[test]
fn fuyin_when_general_equals_hour() {
    // 月将==时 → offset 0 → 伏吟，且给出三传。
    let mg = month_general_branch(331.0); // 11
    // 选 hour==mg 使 offset==0。
    let c = compute_via(0, 0, mg, mg);
    assert_eq!(c.offset, 0);
    assert_eq!(c.pattern, Pattern::FuYin);
    assert!(c.transmission.is_some());
}

#[test]
fn classification_covers_kede_and_yaoke() {
    // 构造覆盖：贼克类与遥克类都应在某些时辰出现。
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    for day in 1..=60u32 {
        for hour in 0..24u32 {
            let c = compute(2024, 1, 1 + (day % 28), hour, 0, 8.0);
            seen.insert(c.pattern);
        }
    }
    // 至少应见到重审/元首（贼克）与一种遥克或特殊式。
    assert!(seen.contains(&Pattern::ZhongShen) || seen.contains(&Pattern::YuanShou));
    assert!(seen.len() >= 2);
}

/// 测试辅助：直接给 （日干，日支，月将，时支） 起课。
pub(super) fn compute_via(stem: u8, branch: u8, mg: u8, hb: u8) -> Cast {
    compute_via_with(stem, branch, mg, hb, SheHaiSchool::Classical)
}

pub(super) fn compute_via_with(stem: u8, branch: u8, mg: u8, hb: u8, school: SheHaiSchool) -> Cast {
    let offset = plate_offset(mg, hb);
    let courses = four_courses(stem, branch, offset);
    let (pattern, transmission) = derive_transmission(&courses, stem, branch, offset, school);
    let mut heaven = [0u8; 12];
    for (g, h) in heaven.iter_mut().enumerate() {
        *h = heaven_plate(g as u8, offset);
    }
    Cast {
        day_stem: stem,
        day_branch: branch,
        hour_branch: hb,
        month_general: mg,
        month_general_name: MONTH_GENERAL_NAMES[mg as usize],
        offset,
        heaven,
        courses,
        pattern,
        pattern_label: pattern.label(),
        transmission,
    }
}

#[test]
fn fanyin_when_opposite() {
    // offset==6（天地相冲）→ 返吟。
    let c = compute_via(0, 0, 6, 0); // mg=6,hb=0 → offset 6
    assert_eq!(c.offset, 6);
    assert_eq!(c.pattern, Pattern::FanYin);
}

#[test]
fn special_patterns_have_none_transmission() {
    // 九宗门里只剩返吟无克一路（井栏射等）不强编三传，其余八门皆已取传。
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    for stem in 0..10u8 {
        for branch in 0..12u8 {
            if stem % 2 != branch % 2 {
                continue; // 干支阴阳须同，六十甲子
            }
            for mg in 0..12u8 {
                for hb in 0..12u8 {
                    let c = compute_via(stem, branch, mg, hb);
                    seen.insert(c.pattern);
                    assert!(c.transmission.is_some(), "九宗门取传已全覆盖，不该再有留空");
                }
            }
        }
    }
    for p in [Pattern::SheHai, Pattern::MaoXing, Pattern::BieZe, Pattern::BaZhuan] {
        assert!(seen.contains(&p), "{} 应在全枚举里出现", p.label());
    }
}

#[test]
fn full_scan_reaches_every_pattern_branch() {
    // 穷举 （日干×日支×月将×时支）=10×12×12×12=17280 组合，确保每条判定分支都被走到，
    // 并校验：十一种课式全部可达，且全部取传（九宗门已无留空）。
    use std::collections::HashSet;
    let mut patterns = HashSet::new();
    let mut fanyin_with_ke = false;
    let mut fanyin_no_ke = false;
    for stem in 0..10u8 {
        for branch in 0..12u8 {
            for mg in 0..12u8 {
                for hb in 0..12u8 {
                    let c = compute_via(stem, branch, mg, hb);
                    patterns.insert(c.pattern);
                    assert!(c.transmission.is_some(), "九宗门取传已全覆盖");
                    if c.pattern == Pattern::FanYin {
                        let courses = four_courses(stem, branch, plate_offset(mg, hb));
                        let has_ke = courses.iter().enumerate().any(|(i, cc)| {
                            down_controls_up(i, cc, stem) || up_controls_down(i, cc, stem)
                        });
                        if has_ke {
                            fanyin_with_ke = true;
                        } else {
                            fanyin_no_ke = true;
                        }
                    }
                }
            }
        }
    }
    // 全部 11 种课式都可达（判定树无死分支）。
    for p in [
        Pattern::ZhongShen,
        Pattern::YuanShou,
        Pattern::BiYong,
        Pattern::SheHai,
        Pattern::HaoShi,
        Pattern::TanShe,
        Pattern::MaoXing,
        Pattern::BieZe,
        Pattern::BaZhuan,
        Pattern::FuYin,
        Pattern::FanYin,
    ] {
        assert!(patterns.contains(&p), "课式 {p:?} 不可达");
    }
    assert!(fanyin_with_ke, "应存在有克返吟（走贼克类取传）");
    assert!(fanyin_no_ke, "应存在无克返吟（走井栏射）");
}

#[test]
fn deterministic() {
    let a = compute(2024, 6, 15, 14, 30, 8.0);
    let b = compute(2024, 6, 15, 14, 30, 8.0);
    assert_eq!(a.courses, b.courses);
    assert_eq!(a.pattern, b.pattern);
}

mod course_census {
    use super::*;

    /// 枚举六十甲子 × 十二局，收集某课式的全部命中。
    fn census(want: Pattern) -> Vec<(u8, u8, u8, Cast)> {
        let mut out = Vec::new();
        for stem in 0..10u8 {
            for branch in 0..12u8 {
                if stem % 2 != branch % 2 {
                    continue;
                }
                for offset in 0..12u8 {
                    // 月将 = 时支 + offset，这里直接以时支 0 遍历 offset
                    let c = super::tests::compute_via(stem, branch, offset, 0);
                    if c.pattern == want {
                        out.push((stem, branch, offset, c));
                    }
                }
            }
        }
        out
    }

    /// 昴星恰 16 课，刚 4 柔 12 —— 两部独立的书各自自报过这两个数。
    ///
    /// 《六壬大全》卷一末〈补论〉「凡昴星止十六课」；
    /// 《六壬粹言》卷二「昴星仰视格……计四课」「昴星俯视格……计一十二课」。
    /// 课数对上，说明取传规则与作者脑中的规则是同一个——比任何单条口诀都硬。
    #[test]
    fn the_mao_xing_census_matches_what_two_books_each_reported() {
        let all = census(Pattern::MaoXing);
        let gang = all.iter().filter(|(s, ..)| s % 2 == 0).count();
        let rou = all.len() - gang;
        assert_eq!((all.len(), gang, rou), (16, 4, 12), "昴星应 16 课（刚 4 柔 12）");
    }

    /// 别责恰 9 课，刚 3 柔 6，且日辰清单与《六壬大全》卷一小注逐条对上。
    ///
    /// 小注原文：「戊辰、戊午、丙辰三刚日各一课，辛未二课，辛丑二课，丁酉、辛酉各一课」。
    #[test]
    fn the_bie_ze_census_matches_the_nine_days_listed_in_the_gloss() {
        let all = census(Pattern::BieZe);
        let gang = all.iter().filter(|(s, ..)| s % 2 == 0).count();
        assert_eq!((all.len(), gang, all.len() - gang), (9, 3, 6), "别责应 9 课（刚 3 柔 6）");
        // 干支组合逐一对表：丙辰、戊辰、戊午各一，辛未、辛丑各二，丁酉、辛酉各一
        let mut tally: std::collections::BTreeMap<(u8, u8), usize> = std::collections::BTreeMap::new();
        for (s, b, ..) in &all {
            *tally.entry((*s, *b)).or_default() += 1;
        }
        // (干, 支, 课数)：丙2辰4 · 戊4辰4 · 戊4午6 · 辛7未7 · 辛7丑1 · 丁3酉9 · 辛7酉9
        let want: Vec<((u8, u8), usize)> =
            vec![((2, 4), 1), ((3, 9), 1), ((4, 4), 1), ((4, 6), 1), ((7, 1), 2), ((7, 7), 2), ((7, 9), 1)];
        assert_eq!(tally.into_iter().collect::<Vec<_>>(), want, "别责的九课日辰应与小注一致");
    }

    /// 八专恰 16 课，刚 6 柔 10；癸丑日一课不入（四课皆有克）；独足课有且仅有一课。
    ///
    /// 《六壬粹言》卷二「顺数三神格……计六课」「逆数三神格……计十课」；
    /// 《课经》「八专日有五，除癸丑日俱有克」；
    /// 《御定六壬直指》「独脚课兮止一名」——三传三字全同者唯一。
    #[test]
    fn the_ba_zhuan_census_and_the_single_footed_course() {
        let all = census(Pattern::BaZhuan);
        let gang = all.iter().filter(|(s, ..)| s % 2 == 0).count();
        assert_eq!((all.len(), gang, all.len() - gang), (16, 6, 10), "八专应 16 课（刚 6 柔 10）");
        // 癸(9)丑(1) 一课不入
        assert!(!all.iter().any(|(s, b, ..)| (*s, *b) == (9, 1)), "癸丑日四课皆有克，不入八专");
        // 五个八专日里只有四日出现
        let days: std::collections::BTreeSet<(u8, u8)> = all.iter().map(|(s, b, ..)| (*s, *b)).collect();
        assert_eq!(days.into_iter().collect::<Vec<_>>(), vec![(0, 2), (3, 7), (5, 7), (6, 8)]);
        // 独足：三传三字全同，唯一
        let single_footed: Vec<_> = all
            .iter()
            .filter(|(.., c)| {
                c.transmission.is_some_and(|t| t[0] == t[1] && t[1] == t[2])
            })
            .collect();
        assert_eq!(single_footed.len(), 1, "独足课止一名");
        let (s, b, _, c) = single_footed[0];
        assert_eq!((*s, *b), (5, 7), "独足课是己未日");
        assert_eq!(c.transmission, Some([9, 9, 9]), "三传酉酉酉");
    }

    /// 涉害的「受克深浅」数法：六个古籍算例逐条复算。
    ///
    /// 两处边界由算例定死，都不计：起点（天盘神所临的地盘位）与终点（该神的本家）。
    /// 《观月经》甲辰日「子加辰……巳上戊土、未土、未上己土、前又戌土，共四重」——
    /// 起点辰本身是土（克子水）却不在账上；「未土 ＋ 未上己土」分两重记，故寄干单独计。
    /// 《课经》甲午日「辰加寅，历卯木一重」——终点辰的寄干乙木若计就是两重，作一重故本家不计。
    #[test]
    fn the_depth_count_reproduces_six_worked_examples() {
        // (下神, 上神, 古籍所记重数)
        const ORACLE: [(u8, u8, u32); 6] = [
            (2, 10, 2),  // 《观月经》甲辰日：戌加寅，历卯木、乙木二重
            (4, 0, 4),   // 同上：子加辰，历巳戊土、未土、未己土、戌土四重
            (3, 1, 1),   // 《课经》丁卯日：丑加卯，只历辰中乙木一重
            (1, 11, 5),  // 同上：亥加丑，历辰戊未己戌土五重
            (2, 4, 1),   // 《课经》甲午日缀瑕例：辰加寅，历卯木一重
            (6, 8, 1),   // 同上：申加午，历丁火一重
        ];
        for (down, up, want) in ORACLE {
            let got = shehai_depth(&Course { down, up });
            assert_eq!(got, want, "{down} 上 {up} 的重数应为 {want}，实得 {got}");
        }
    }

    /// 两派涉害并非同一件事：至少存在一课，古法与近法给出不同的初传。
    #[test]
    fn the_two_shehai_schools_are_not_the_same_rule() {
        let mut differ = 0;
        for stem in 0..10u8 {
            for branch in 0..12u8 {
                if stem % 2 != branch % 2 {
                    continue;
                }
                for offset in 0..12u8 {
                    let a = super::tests::compute_via_with(stem, branch, offset, 0, SheHaiSchool::Classical);
                    let b = super::tests::compute_via_with(stem, branch, offset, 0, SheHaiSchool::ByPosition);
                    if a.pattern == Pattern::SheHai && a.transmission != b.transmission {
                        differ += 1;
                    }
                }
            }
        }
        assert!(differ > 0, "两派若处处同解，就不该建成两个流派");
    }

    /// 流派 id 往返。
    #[test]
    fn shehai_school_id_roundtrip() {
        for s in [SheHaiSchool::Classical, SheHaiSchool::ByPosition] {
            assert_eq!(SheHaiSchool::from_id(s.id()), Some(s));
        }
        assert_eq!(SheHaiSchool::from_id("unknown"), None);
        assert_eq!(SheHaiSchool::default(), SheHaiSchool::Classical);
    }
}

mod jing_lan {
    use super::*;

    /// 返吟无克（井栏射 / 无亲）恰六课，六组三传逐字对《六壬粹言》卷二所列。
    ///
    /// 原文：「无亲课。谓返吟无克，取支之驿马为用，中用支上神，末用干上神，曰无亲。**计六课**。
    /// 丁丑、己丑日三传亥未丑，辛丑日三传亥未辰，丁未己未日三传巳丑丑，辛未日三传巳丑辰。」
    ///
    /// 《六壬大全》卷一歌诀「若知六日该无克，丑未同干丁己辛」、卷七《订讹》
    /// 「盖无克者，惟丁未、己未、辛未、丁丑、己丑、辛丑六日」、
    /// 《注解大六壬指南》卷一同列六日，三处旁证。
    #[test]
    fn the_well_rail_course_has_exactly_six_days_with_the_transmissions_the_sources_list() {
        // (日干, 日支, 三传)
        const ORACLE: [(u8, u8, [u8; 3]); 6] = [
            (3, 1, [11, 7, 1]),  // 丁丑 → 亥未丑
            (5, 1, [11, 7, 1]),  // 己丑 → 亥未丑
            (7, 1, [11, 7, 4]),  // 辛丑 → 亥未辰
            (3, 7, [5, 1, 1]),   // 丁未 → 巳丑丑
            (5, 7, [5, 1, 1]),   // 己未 → 巳丑丑
            (7, 7, [5, 1, 4]),   // 辛未 → 巳丑辰
        ];
        // 六十甲子里返吟且无克的，恰是这六日
        let mut found = Vec::new();
        for stem in 0..10u8 {
            for branch in 0..12u8 {
                if stem % 2 != branch % 2 {
                    continue;
                }
                let c = super::tests::compute_via_with(stem, branch, 6, 0, SheHaiSchool::Classical);
                if c.pattern == Pattern::FanYin {
                    // 有克的返吟走贼克类取传，三传由层层取上神得出；无克的走井栏
                    let courses = four_courses(stem, branch, 6);
                    let has_ke = courses.iter().enumerate().any(|(i, cc)| {
                        down_controls_up(i, cc, stem) || up_controls_down(i, cc, stem)
                    });
                    if !has_ke {
                        found.push((stem, branch, c.transmission.expect("井栏射应取传")));
                    }
                }
            }
        }
        assert_eq!(found.len(), 6, "无克的返吟应恰六课，实得 {}", found.len());
        for (stem, branch, want) in ORACLE {
            let got = found
                .iter()
                .find(|(s, b, _)| (*s, *b) == (stem, branch))
                .unwrap_or_else(|| panic!("六日里应有 干{stem} 支{branch}"));
            assert_eq!(got.2, want, "干{stem} 支{branch} 的三传");
        }
        // 六日全是阴干配阴支——刚日返吟必有克，结构上不存在「刚日井栏射」
        for (stem, branch, _) in &found {
            assert!(!stem_is_yang(*stem) && !branch_is_yang(*branch), "井栏射只落六阴日");
        }
    }

    /// 《六壬大全》卷四引《黄帝初占》的实占算例：己丑岁，小吉（未）加丑，三传亥未丑。
    ///
    /// 原文：「用井栏射法。初传巳上登明为用，将得六合。中传丑上见小吉，将得天后。
    /// 末传未上见大吉，将得青龙。」登明＝亥、小吉＝未、大吉＝丑。
    #[test]
    fn the_worked_example_from_the_yellow_emperors_text() {
        let c = super::tests::compute_via_with(5, 1, 6, 0, SheHaiSchool::Classical); // 己丑
        assert_eq!(c.pattern, Pattern::FanYin);
        assert_eq!(c.transmission, Some([11, 7, 1]), "亥 → 未 → 丑");
    }

    /// 丁未 / 己未 同时满足「返吟无克」与「八专」，两门归类有争而**结果无争**。
    ///
    /// 《大全》卷七〈课经集〉与所引《观月经》把这两日划归八专（故说「无克惟四日」），
    /// 《订讹》、卷一歌诀、《粹言》、《指南》划归井栏（故说「六日」）。
    /// 按八专阴日法（支阴连本位逆数三神为初、中末俱取干上神）算丁未日返吟，
    /// 得初传未逆三 = 巳、中末皆丑，三传仍是巳丑丑，与井栏射一字不差。
    #[test]
    fn the_two_disputed_days_come_out_the_same_under_either_classification() {
        for stem in [3u8, 5] {
            let courses = four_courses(stem, 7, 6);
            // 井栏法
            let by_well = super::tests::compute_via_with(stem, 7, 6, 0, SheHaiSchool::Classical)
                .transmission
                .expect("应取传");
            // 八专阴日法：起点取四课上神，连本位逆数三位；中末皆干上神
            let by_bazhuan = [(courses[3].up + 10) % 12, courses[0].up, courses[0].up];
            assert_eq!(by_well, by_bazhuan, "干{stem} 未日：两门归类不同但三传应相同");
            assert_eq!(by_well, [5, 1, 1], "巳丑丑");
        }
    }
}
