//! 发三传：先判课式，再依该课式取传。
//!
//! 九宗门的次第是有序的——贼克在前，逐门下推，第一个成立的即是本课的课式。
//! 涉害一门的取用法两派各行其是（数不数受克深浅），故它是本叶唯一带流派开关的地方。

use crate::plates::{heaven_plate, STEM_LODGING};
use crate::{Course, Pattern, SheHaiSchool};
use mingli_ganzhi::branch_element;

/// 地盘位的孟仲季档次：孟（寅申巳亥）0 ＞ 仲（子午卯酉）1 ＞ 季（辰戌丑未）2。
///
/// **看的是天盘神所临的地盘位，不是天盘神自己**——《六壬粹言》的复等例把这一点钉死了：
/// 戊辰日一课子加巳、四课午加亥，子午本是仲，书却说「俱在孟位上」，因为巳、亥是孟。
/// 定义域是 `0..12`。本函数**不做**模十二归位——越界的输入落到通配臂被当作「季」。
/// 调用方传的是地盘位，恒在范围内；写明是因为通配臂看不出这件事。
pub(crate) fn meng_zhong_ji(ground: u8) -> u8 {
    match ground {
        2 | 8 | 5 | 11 => 0,
        0 | 6 | 3 | 9 => 1,
        _ => 2,
    }
}

/// 涉害「受克深浅」：天盘神 `up` 临地盘 `down`，自 `down` 的下一格顺行到 `up` 本家的前一格，
/// 沿途每遇一个克 `up` 的住户记一重。住户 = 该地支本身 ＋ 寄于该宫的天干。
///
/// 两处边界由古籍算例定死，都**不计**：起点（`down` 本身）与终点（`up` 的本家）。
/// 《观月经》甲辰日「子加辰……巳上戊土、未土、未上己土、前又戌土，共四重」——
/// 起点辰本身是土（克子水）却不在账上；「未土 ＋ 未上己土」分开记两重，故寄干单独计。
/// 《课经》甲午日「辰加寅，历卯木一重」——终点辰的寄干乙木若计就是两重，作一重故本家不计。
pub(crate) fn shehai_depth(course: &Course) -> u32 {
    let target = branch_element(course.up);
    let mut depth = 0;
    let mut g = (course.down + 1) % 12;
    while g != course.up {
        if branch_element(g).controls() == target {
            depth += 1;
        }
        for stem in 0..10u8 {
            if STEM_LODGING[stem as usize] == g
                && mingli_ganzhi::stem_element(stem).controls() == target
            {
                depth += 1;
            }
        }
        g = (g + 1) % 12;
    }
    depth
}

/// 四课「下位」的五行。
///
/// 一课写作「干上神／**日干**」——下位是日干本身，不是它的寄宫。这一条要紧：
/// 乙丁戊辛癸五干的寄宫地支五行与干不同（乙木寄辰土、丁火寄未土、戊土寄巳火、
/// 辛金寄戌土、癸水寄丑土），拿寄宫五行去判贼克，这五干的一课会判错，
/// 连带把课式判到别的门里去。二三四课的下位都是地支，照常取支五行。
pub(crate) fn down_element(idx: usize, c: &Course, day_stem: u8) -> mingli_ganzhi::Element {
    if idx == 0 {
        mingli_ganzhi::stem_element(day_stem)
    } else {
        branch_element(c.down)
    }
}

/// 下贼上：下位五行克上神五行。
pub(crate) fn down_controls_up(idx: usize, c: &Course, day_stem: u8) -> bool {
    down_element(idx, c, day_stem).controls() == branch_element(c.up)
}
/// 上克下：上神五行克下位五行。
pub(crate) fn up_controls_down(idx: usize, c: &Course, day_stem: u8) -> bool {
    branch_element(c.up).controls() == down_element(idx, c, day_stem)
}

/// 由初传地支「层层取天盘上神」得三传。
pub(crate) fn transmit_from(first: u8, offset: u8) -> [u8; 3] {
    let mid = heaven_plate(first, offset);
    let last = heaven_plate(mid, offset);
    [first, mid, last]
}

/// 天干阴阳：甲丙戊庚壬（偶）为阳。地支阴阳：子寅辰午申戌（偶）为阳。
pub(crate) fn stem_is_yang(stem: u8) -> bool {
    stem.is_multiple_of(2)
}
pub(crate) fn branch_is_yang(branch: u8) -> bool {
    branch.is_multiple_of(2)
}

/// 判课式并取传。
pub(crate) fn derive_transmission(
    courses: &[Course; 4],
    day_stem: u8,
    day_branch: u8,
    offset: u8,
    school: SheHaiSchool,
) -> (Pattern, Option<[u8; 3]>) {
    // 伏吟 / 返吟：先判天地盘几何。
    if offset == 0 {
        // 伏吟：阳日（自任）初传=干上神，阴日（自信）初传=支上神。中末层层取上神。
        let first = if stem_is_yang(day_stem) {
            courses[0].up
        } else {
            courses[2].up
        };
        return (Pattern::FuYin, Some(transmit_from(first, offset)));
    }
    if offset == 6 {
        // 返吟：有克走贼克类；中末传层层取上神（offset=6 即取冲，天然由 heaven_plate 实现）。
        if let Some(first) =
            single_kede(courses, true, day_stem).or_else(|| single_kede(courses, false, day_stem))
        {
            return (Pattern::FanYin, Some(transmit_from(first, offset)));
        }
        // 返吟无克 —— 井栏射 / 无亲。初传取**日支驿马**，中传支上神，末传干上神。
        //
        // 《六壬粹言》卷二写得最直白：「无亲课。谓返吟无克，取支之驿马为用，中用支上神，
        // 末用干上神，曰无亲。计六课。」并列全了六组三传。《六壬大全》卷七〈课经集一〉
        // 「井栏格」条、卷四引《黄帝初占》的算例、《注解大六壬指南》卷一，取法逐字相合。
        //
        // 古籍原本的说法是「以支辰傍射敌上神为用……如傍井倚栏，斜冲射之」——取日支斜冲之宫
        // 上所乘的天盘神。返吟盘下巳上必是亥、亥上必是巳，与驿马取值恒同，两种写法在本门内无差。
        //
        // 不分刚柔：无克的六日全是阴干配阴支（丁己辛 × 丑未），刚日返吟必有克，
        // 结构上不存在「刚日井栏射」。《大全》卷七径称之为「六阴日课」。
        // （《大全》卷一有一句夹注「阳日用辰，阴日用日」与自身下半句及其余四源全部冲突，
        //   措辞恰是昴星法的原话，疑为窜入的旁注，不采信。）
        let group = mingli_ganzhi::sanhe_group_index(day_branch) as usize;
        let first = mingli_ganzhi::YIMA[group];
        return (Pattern::FanYin, Some([first, courses[2].up, courses[0].up]));
    }

    // 贼克法：下贼上优先于上克下。
    let zei = courses.iter().enumerate().any(|(i, c)| down_controls_up(i, c, day_stem));
    let ke = courses.iter().enumerate().any(|(i, c)| up_controls_down(i, c, day_stem));

    if zei {
        return resolve_kede(courses, true, Pattern::ZhongShen, day_stem, offset, school);
    }
    if ke {
        return resolve_kede(courses, false, Pattern::YuanShou, day_stem, offset, school);
    }

    let day_lodging = STEM_LODGING[day_stem as usize];
    // 八专先于遥克判：《六壬粹言》卷一驳《订讹》——「伏吟课既无遥克之例，
    // 而八专何独有取于遥克耶？且既取遥克，则古来当不设独足一课矣」。
    // 独足课（己未日酉加未，三传酉酉酉）在 720 课里有且仅有一课，正是不取遥克才存在。
    if day_lodging == day_branch {
        // 阳日自干上神连本位顺数三位，阴日自四课上神连本位逆数三位；中末皆取干上神。
        // 「连本位」四源明证：卷一夹注「连本位数」、《课经》算例「干上阳神亥，顺数至丑」、
        // 《指南》今注「以丑为一，顺数至三」、《粹言》「连根顺数三神」。
        let first = if stem_is_yang(day_stem) {
            (courses[0].up + 2) % 12
        } else {
            (courses[3].up + 10) % 12
        };
        return (Pattern::BaZhuan, Some([first, courses[0].up, courses[0].up]));
    }

    // 无上下克：遥克。
    let day_elem = mingli_ganzhi::stem_element(day_stem);
    // 蒿矢：天盘神克日干（寄宫）。
    if let Some(first) = courses
        .iter()
        .map(|c| c.up)
        .find(|&u| branch_element(u).controls() == day_elem)
    {
        return (Pattern::HaoShi, Some(transmit_from(first, offset)));
    }
    // 弹射：日干克天盘神。
    if let Some(first) = courses
        .iter()
        .map(|c| c.up)
        .find(|&u| day_elem.controls() == branch_element(u))
    {
        return (Pattern::TanShe, Some(transmit_from(first, offset)));
    }

    // 别责：四课不全（去重后 < 4 课）。
    let distinct: std::collections::HashSet<(u8, u8)> =
        courses.iter().map(|c| (c.down, c.up)).collect();
    if distinct.len() < 4 {
        // 阳日「干合上头神」：取日干之合干的寄宫，用该宫的上神；
        // 阴日「支前三合」：取日支三合局前一位（支 + 4）**本身**发用，不再取其上神。
        // 中末皆取干上神。柔日取本身还是取其上神，《订讹》曾存疑，
        // 《六壬粹言》卷一裁定「仍以古法为是」（取本身），《课经》《指南》算例亦然。
        let first = if stem_is_yang(day_stem) {
            let partner_lodging = STEM_LODGING[((day_stem + 5) % 10) as usize];
            heaven_plate(partner_lodging, offset)
        } else {
            (day_branch + 4) % 12
        };
        return (Pattern::BieZe, Some([first, courses[0].up, courses[0].up]));
    }

    // 昴星：余下。阳日仰视地盘酉位之上神，阴日俯视天盘酉所临之地盘支。
    // 中末：**阳日先支后干、阴日先干后支**——《课经》给了理据「刚日本乎天者亲上，末传归干；
    // 柔日本乎地者亲下，末传归辰」。五源无异议。
    let (first, mid, last) = if stem_is_yang(day_stem) {
        (heaven_plate(9, offset), courses[2].up, courses[0].up)
    } else {
        ((9 + 12 - offset) % 12, courses[0].up, courses[2].up)
    };
    (Pattern::MaoXing, Some([first, mid, last]))
}

/// 单一克对：返回唯一受克之上神；多于一则 None（交给比用/涉害）。
pub(crate) fn single_kede(courses: &[Course; 4], zei: bool, day_stem: u8) -> Option<u8> {
    let ups: Vec<u8> = courses
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            if zei {
                down_controls_up(*i, c, day_stem)
            } else {
                up_controls_down(*i, c, day_stem)
            }
        })
        .map(|(_, c)| c.up)
        .collect();
    if ups.len() == 1 {
        Some(ups[0])
    } else {
        None
    }
}

/// 贼克 / 比用 / 涉害的取传。`zei` 为真时取下贼上诸课，否则取上克下诸课。
pub(crate) fn resolve_kede(
    courses: &[Course; 4],
    zei: bool,
    base: Pattern,
    day_stem: u8,
    offset: u8,
    school: SheHaiSchool,
) -> (Pattern, Option<[u8; 3]>) {
    let hits: Vec<Course> = courses
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            if zei {
                down_controls_up(*i, c, day_stem)
            } else {
                up_controls_down(*i, c, day_stem)
            }
        })
        .map(|(_, c)| *c)
        .collect();
    if hits.len() == 1 {
        return (base, Some(transmit_from(hits[0].up, offset)));
    }
    // 比用：取与日干同阴阳之上神。
    let yang = stem_is_yang(day_stem);
    let bi: Vec<Course> = hits.iter().copied().filter(|c| branch_is_yang(c.up) == yang).collect();
    if bi.len() == 1 {
        return (Pattern::BiYong, Some(transmit_from(bi[0].up, offset)));
    }
    // 俱比取比者，俱不比取全部克课 —— 这一组进涉害。
    let mut pool = if bi.is_empty() { hits } else { bi };

    if school == SheHaiSchool::Classical {
        // 第一层：受克深者为用。
        let deepest = pool.iter().map(shehai_depth).max().unwrap_or(0);
        pool.retain(|c| shehai_depth(c) == deepest);
    }
    // 第二层：孟 ＞ 仲 ＞ 季，按天盘神**所临的地盘位**判。
    let best = pool.iter().map(|c| meng_zhong_ji(c.down)).min().unwrap_or(2);
    pool.retain(|c| meng_zhong_ji(c.down) == best);
    // 第三层（复等 / 缀瑕）：阳日取干上神，阴日取支上神。
    let first = if pool.len() == 1 {
        pool[0].up
    } else {
        let prefer = if yang { courses[0].up } else { courses[2].up };
        pool.iter().map(|c| c.up).find(|&u| u == prefer).unwrap_or(pool[0].up)
    };
    (Pattern::SheHai, Some(transmit_from(first, offset)))
}
