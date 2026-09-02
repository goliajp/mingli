//! 紫微斗数的校验：五行局、起紫微、十四主星与四化。

use super::*;

/// 紫微星落宫，五个局数 × 三十天逐格钉住。
///
/// 这是**基准**：下面要改 [`ziwei_branch`] 的写法（那里有一个没有上限的循环，
/// 变异扫描在里面留了三个超时），改完这张表必须逐格不变。
///
/// 表里的数由改动前的实现算出。它答的是「改写有没有改变行为」，
/// 不是「紫微落宫对不对」——后者由下面几条对公开排盘的比对守着。
#[test]
fn the_ziwei_palace_table_is_unchanged() {
    const EXPECTED: [[u8; 30]; 5] = [
        [1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 0, 0, 1, 1, 2, 2, 3, 3, 4],
        [4, 1, 2, 5, 2, 3, 6, 3, 4, 7, 4, 5, 8, 5, 6, 9, 6, 7, 10, 7, 8, 11, 8, 9, 0, 9, 10, 1, 10, 11],
        [11, 4, 1, 2, 0, 5, 2, 3, 1, 6, 3, 4, 2, 7, 4, 5, 3, 8, 5, 6, 4, 9, 6, 7, 5, 10, 7, 8, 6, 11],
        [6, 11, 4, 1, 2, 7, 0, 5, 2, 3, 8, 1, 6, 3, 4, 9, 2, 7, 4, 5, 10, 3, 8, 5, 6, 11, 4, 9, 6, 7],
        [9, 6, 11, 4, 1, 2, 10, 7, 0, 5, 2, 3, 11, 8, 1, 6, 3, 4, 0, 9, 2, 7, 4, 5, 1, 10, 3, 8, 5, 6],
    ];
    for (row, ju) in (2_u32..=6).enumerate() {
        for (col, day) in (1_u32..=30).enumerate() {
            assert_eq!(
                ziwei_branch(day, ju),
                EXPECTED[row][col],
                "{ju} 局第 {day} 日的紫微落宫变了"
            );
        }
    }
}

// 起紫微多点验证（用 iztro + 掌中诀核对，子=0 编号）
#[test]
fn ziwei_position_points() {
    assert_eq!(ziwei_branch(23, 5), 8); // 申
    assert_eq!(ziwei_branch(23, 4), 6); // 午
    assert_eq!(ziwei_branch(28, 2), 3); // 卯
    assert_eq!(ziwei_branch(1, 2), 1); // 丑
    assert_eq!(ziwei_branch(1, 3), 4); // 辰
    assert_eq!(ziwei_branch(1, 4), 11); // 亥
    assert_eq!(ziwei_branch(1, 5), 6); // 午
    assert_eq!(ziwei_branch(2, 5), 11); // 亥
    assert_eq!(ziwei_branch(2, 6), 6); // 午
}

#[test]
fn sample_1990_06_15() {
    const MAJOR: [&str; 14] = [
        "紫微","天机","太阳","武曲","天同","廉贞",
        "天府","太阴","贪狼","巨门","天相","天梁","七杀","破军",
    ];
    let chart = compute(BirthInput {
        year: 1990,
        month: 6,
        day: 15,
        hour: 14,
        minute: 30,
        tz: 8.0,
        gender: Some(Gender::Male),
    });
    assert_eq!(chart.ming_branch, "亥");
    assert_eq!(chart.shen_branch, "丑"); // 身宫在福德（丑）
    assert_eq!(chart.ming_ganzhi, "丁亥");
    assert_eq!(chart.wuxing_ju, "土五局");
    assert_eq!(chart.ju_number, 5);
    assert_eq!(chart.ziwei_branch, "申");
    assert_eq!(chart.tianfu_branch, "申");
    // 命宫（亥）主星应含巨门
    let ming = chart.palaces.iter().find(|p| p.is_ming).unwrap();
    assert!(
        ming.stars.iter().any(|s| s == "巨门"),
        "命宫主星应含巨门，实得 {:?}",
        ming.stars
    );
    // 十四主星 + 4 辅星（文昌/文曲/左辅/右弼） = 18 颗，无遗漏。
    let total: usize = chart.palaces.iter().map(|p| p.stars.len()).sum();
    assert_eq!(total, 18);
    // 单独校验 14 主星仍齐（过滤 4 辅星）。
    let major_count: usize = chart.palaces.iter()
        .flat_map(|p| p.stars.iter())
        .filter(|s| MAJOR.contains(&s.as_str())).count();
    assert_eq!(major_count, 14);
    // 十二宫名俱全
    assert!(chart.palaces.iter().any(|p| p.name == "福德"));
}

#[test]
fn ju_mapping_all() {
    for (e, n, nm) in [
        (Element::Water, 2, "水二局"),
        (Element::Wood, 3, "木三局"),
        (Element::Metal, 4, "金四局"),
        (Element::Earth, 5, "土五局"),
        (Element::Fire, 6, "火六局"),
    ] {
        assert_eq!(ju_from_element(e), n);
        assert_eq!(ju_name(n), nm);
    }
}

#[test]
fn aux_stars_1990_06_15_oracle() {
    // 1990-06-15 14：30 CST： 农历五月廿三 未时（时支=7）
    // 公式校验：文昌=(10-7)%12=3=卯、文曲=(4+7)%12=11=亥、左辅=(4+5-1)%12=8=申、右弼=(10-(5-1))%12=6=午。
    let chart = compute(BirthInput {
        year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
        gender: Some(Gender::Male),
    });
    let where_star = |s: &str| {
        chart.palaces.iter().find(|p| p.stars.iter().any(|x| x == s)).map(|p| p.branch.clone())
    };
    assert_eq!(where_star("文昌").as_deref(), Some("卯"));
    assert_eq!(where_star("文曲").as_deref(), Some("亥"));
    assert_eq!(where_star("左辅").as_deref(), Some("申"));
    assert_eq!(where_star("右弼").as_deref(), Some("午"));
    // 18 颗（十四主星 + 4 辅星）无遗漏。
    let total: usize = chart.palaces.iter().map(|p| p.stars.len()).sum();
    assert_eq!(total, 18);

    // 十四主星逐颗落宫。
    //
    // 上面只钉了四颗辅星与一个总数——十四主星各在哪一宫从来没问过。
    // 紫微星系那六颗的偏移是负的（天机 −1、太阳 −3、武曲 −4、天同 −5、廉贞 −8），
    // 把其中一个负号去掉，星就换个宫落，而总数仍是 18、辅星也不动，于是没人红。
    //
    // 这张表钉的是转写：值由当前实现在这张盘上算出，答的是「偏移有没有被改过」。
    // 「紫微起得对不对」由 `ziwei_position_points` 与 `sample_1990_06_15` 对 iztro 守着。
    for (star, want) in [
        // 紫微星系：以紫微为基准逆行 0/−1/−3/−4/−5/−8
        ("紫微", "申"), ("天机", "未"), ("太阳", "巳"),
        ("武曲", "辰"), ("天同", "卯"), ("廉贞", "子"),
        // 天府星系：以天府为基准顺行 0/+1/+2/+3/+4/+5/+6/+10
        ("天府", "申"), ("太阴", "酉"), ("贪狼", "戌"), ("巨门", "亥"),
        ("天相", "子"), ("天梁", "丑"), ("七杀", "寅"), ("破军", "午"),
    ] {
        assert_eq!(where_star(star).as_deref(), Some(want), "{star} 落宫");
    }

    // 命宫与身宫的标记：恰好各一个，且落在算出来的那一宫上。
    //
    // `is_ming` / `is_shen` 是由 `b == ming` / `b == shen` 得来的。把 `==` 写成 `!=`，
    // 十二宫里会有十一个被标成命宫——而没有一条测试看过这两个字段。
    let ming: Vec<&str> = chart.palaces.iter().filter(|p| p.is_ming).map(|p| p.branch.as_str()).collect();
    let shen: Vec<&str> = chart.palaces.iter().filter(|p| p.is_shen).map(|p| p.branch.as_str()).collect();
    assert_eq!(ming.len(), 1, "命宫应恰好一个，实得 {ming:?}");
    assert_eq!(shen.len(), 1, "身宫应恰好一个，实得 {shen:?}");
}

#[test]
fn sihua_1990_geng_standard_school_oracle() {
    // 1990 庚午年（年干=庚， stem_id=6）。通行版四化：太阳禄/武曲权/太阴科/天同忌。
    // 落宫（已知盘）：太阳=巳、武曲=辰、太阴=酉、天同=卯。
    let chart = compute(BirthInput {
        year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
        gender: Some(Gender::Male),
    });
    assert_eq!(chart.sihua.school_id, "standard");
    assert_eq!(chart.sihua.lu_star, "太阳");
    assert_eq!(chart.sihua.lu_branch.as_deref(), Some("巳"));
    assert_eq!(chart.sihua.quan_star, "武曲");
    assert_eq!(chart.sihua.quan_branch.as_deref(), Some("辰"));
    assert_eq!(chart.sihua.ke_star, "太阴");
    assert_eq!(chart.sihua.ke_branch.as_deref(), Some("酉"));
    assert_eq!(chart.sihua.ji_star, "天同");
    assert_eq!(chart.sihua.ji_branch.as_deref(), Some("卯"));
}

#[test]
fn sihua_1990_geng_quanshu_school_oracle() {
    // 同 1990 庚午，中州派（王亭之版）：太阳禄 / 武曲权 / 天府科 / 天同忌。
    // 天府=申 → 化科分歧。其余三化同通行。
    let chart = compute_with(
        BirthInput { year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0, gender: Some(Gender::Male) },
        SihuaSchool::Quanshu,
    );
    assert_eq!(chart.sihua.school_id, "quanshu");
    assert_eq!(chart.sihua.lu_star, "太阳"); // 与通行同
    assert_eq!(chart.sihua.lu_branch.as_deref(), Some("巳"));
    assert_eq!(chart.sihua.quan_star, "武曲");
    assert_eq!(chart.sihua.quan_branch.as_deref(), Some("辰"));
    assert_eq!(chart.sihua.ke_star, "天府"); // 分歧：通行=太阴、全书=天府
    assert_eq!(chart.sihua.ke_branch.as_deref(), Some("申"));
    assert_eq!(chart.sihua.ji_star, "天同");
    assert_eq!(chart.sihua.ji_branch.as_deref(), Some("卯"));
}

/// 两派只在化科上分岔，且恰好分在受「辅弼不入四化」影响的三干：戊(4)、庚(6)、壬(8)。
///
/// 戊与壬在通行表里正是由右弼、左辅化科的两干；庚随该派学理一并调整。禄 / 权 / 忌 十干全等。
#[test]
fn the_two_schools_diverge_exactly_where_the_helper_stars_would_have_transformed() {
    const DIVERGENT: [u8; 3] = [4, 6, 8];
    for stem_id in 0..10u8 {
        let s = sihua_for(stem_id, SihuaSchool::Standard);
        let q = sihua_for(stem_id, SihuaSchool::Quanshu);
        assert_eq!((s.lu, s.quan, s.ji), (q.lu, q.quan, q.ji), "stem {stem_id} 只该在化科上分歧");
        if DIVERGENT.contains(&stem_id) {
            assert_ne!(s.ke, q.ke, "stem {stem_id} 科应分歧");
        } else {
            assert_eq!(s.ke, q.ke, "stem {stem_id} 科应一致");
        }
    }
    // 该派的立论是辅弼不化科——通行表里凡由左辅 / 右弼化科的干，此派必换掉
    for stem_id in 0..10u8 {
        let s = sihua_for(stem_id, SihuaSchool::Standard);
        if matches!(s.ke, "左辅" | "右弼") {
            let q = sihua_for(stem_id, SihuaSchool::Quanshu);
            assert!(!matches!(q.ke, "左辅" | "右弼"), "stem {stem_id}：辅弼不该出现在此派的化科位");
        }
    }
}

/// 癸干十家一致：查过《紫微斗数全书》原诀「癸破巨阴贪狼停」、全集栏、闽派、北派 / 河洛、
/// 占验门、钦天门、梁若瑜飞星派、中州派陆斌兆、中州派王亭之，癸行逐字相同。
#[test]
fn the_gui_stem_is_the_same_in_every_school() {
    for school in [SihuaSchool::Standard, SihuaSchool::Quanshu] {
        let g = sihua_for(9, school);
        assert_eq!((g.lu, g.quan, g.ke, g.ji), ("破军", "巨门", "太阴", "贪狼"));
    }
}

#[test]
fn compute_at_default_school_equals_standard() {
    let mo = Moment::new(1990, 6, 15, 14, 30, 8.0);
    let a = compute_at(&mo, Some(Gender::Male));
    let b = compute_at_with(&mo, Some(Gender::Male), SihuaSchool::Standard);
    assert_eq!(a.sihua.school_id, b.sihua.school_id);
    assert_eq!(a.sihua.ke_star, b.sihua.ke_star);
    assert_eq!(a.ming_branch, b.ming_branch);
}

#[test]
fn sihua_school_id_roundtrip() {
    for s in [SihuaSchool::Standard, SihuaSchool::Quanshu] {
        assert_eq!(SihuaSchool::from_id(s.id()), Some(s));
    }
    assert_eq!(SihuaSchool::from_id("unknown"), None);
    assert_eq!(SihuaSchool::default(), SihuaSchool::Standard);
}

#[test]
fn no_gender_ok() {
    let chart = compute(BirthInput {
        year: 2000,
        month: 1,
        day: 1,
        hour: 0,
        minute: 0,
        tz: 8.0,
        gender: None,
    });
    assert_eq!(chart.palaces.len(), 12);
}

/// 五行局：端到端接回已验的六十甲子纳音表，而不是只验一张盘的结论。
///
/// 局由**命宫干支**查纳音定，水二 / 木三 / 金四 / 土五 / 火六
/// （<https://www.ziwei.my/zi-wei-dou-shu-portfolio/wu-xing-ju-note-1/> 与
/// <https://zhuanlan.zhihu.com/p/1893764310146200691> 两处同述；后者并记局数即大限起运岁，
/// 本叶不出大限，故只用前一半）。
///
/// 原先只断言了一张 1990 年盘得「土五局」。那验的是一个结论，验不出映射本身——
/// 把纳音的五行换个次序接到局数上，那张盘仍可能碰巧对。这里改为：
/// 扫一批生日，逐盘拿它自己的 `ming_ganzhi` 去查纳音，再核局数与局名对不对得上。
/// 纳音那张表已在 `mingli-ganzhi` 里逐条对过全六十条，于是这条链两端都有据。
#[test]
fn the_bureau_follows_the_nayin_of_the_life_palace() {
    use mingli_ganzhi::{nayin_element, parse_ganzhi, Element};

    let want_ju = |e: Element| match e {
        Element::Water => (2, "水二局"),
        Element::Wood => (3, "木三局"),
        Element::Metal => (4, "金四局"),
        Element::Earth => (5, "土五局"),
        Element::Fire => (6, "火六局"),
    };

    let mut seen = std::collections::BTreeSet::new();
    for (y, mo, d) in [
        (1990, 6, 15), (1987, 9, 17), (2024, 1, 1), (2000, 2, 29), (1961, 7, 1),
        (1955, 3, 8), (1972, 11, 30), (2011, 5, 20), (1938, 8, 4), (2043, 12, 25),
    ] {
        for h in [0u32, 5, 11, 17, 23] {
            let chart = compute(BirthInput {
                year: y, month: mo, day: d, hour: h, minute: 0, tz: 8.0, gender: Some(Gender::Male),
            });
            let gz = parse_ganzhi(&chart.ming_ganzhi)
                .unwrap_or_else(|| panic!("命宫干支「{}」解析不了", chart.ming_ganzhi));
            let (num, name) = want_ju(nayin_element(gz));
            assert_eq!(
                chart.ju_number, num,
                "{y}-{mo}-{d} {h}时：命宫 {} 纳音属{}，应 {name}，实得「{}」",
                chart.ming_ganzhi,
                nayin_element(gz).name(),
                chart.wuxing_ju,
            );
            assert_eq!(chart.wuxing_ju, name, "局数与局名对不上");
            seen.insert(num);
        }
    }
    assert_eq!(
        seen,
        [2, 3, 4, 5, 6].into_iter().collect::<std::collections::BTreeSet<_>>(),
        "五个局应当都出现过，实际只见到 {seen:?}——有局取不到说明推导链上某处收窄了",
    );
}

/// 四化：**十干全覆盖**，并与传统口诀交叉。
///
/// 原先只验了庚一干（1990 年那张盘），另外九干四十条里的三十六条没人对过。
/// 四化是这套系统最常用的部件之一，错一干就是十分之一的盘全错。
///
/// 通行版参照：<https://ccziwei.com/ziwei-doushu/articles/ziwei-doushu-tianggan-sihua-biao>
/// 与 <https://vocus.cc/article/6646c651fd89780001ef63be> 等处同表
/// （`SihuaSchool::Standard` 的文档另记 5 独立源）。
///
/// 第二条判据是**口诀**「甲廉破武阳、乙机梁紫阴、丙同机昌廉、丁阴同机巨、戊贪阴右机、
/// 己武贪梁曲、庚阳武阴同、辛巨阳曲昌、壬梁紫左武、癸破巨阴贪」——它是同一张表的
/// 另一种编码（practitioner 背的就是这个），逐字取首。表若在某一格抄错，两条判据会一起红。
#[test]
fn the_four_transformations_cover_all_ten_stems() {
    // (禄, 权, 科, 忌)，甲→癸
    const STANDARD: [(&str, &str, &str, &str); 10] = [
        ("廉贞", "破军", "武曲", "太阳"), // 甲
        ("天机", "天梁", "紫微", "太阴"), // 乙
        ("天同", "天机", "文昌", "廉贞"), // 丙
        ("太阴", "天同", "天机", "巨门"), // 丁
        ("贪狼", "太阴", "右弼", "天机"), // 戊
        ("武曲", "贪狼", "天梁", "文曲"), // 己
        ("太阳", "武曲", "太阴", "天同"), // 庚
        ("巨门", "太阳", "文曲", "文昌"), // 辛
        ("天梁", "紫微", "左辅", "武曲"), // 壬
        ("破军", "巨门", "太阴", "贪狼"), // 癸
    ];
    // 口诀逐字：每干四字，依次取禄/权/科/忌之星名的辨识字
    const MNEMONIC: [&str; 10] = [
        "廉破武阳", "机梁紫阴", "同机昌廉", "阴同机巨", "贪阴右机",
        "武贪梁曲", "阳武阴同", "巨阳曲昌", "梁紫左武", "破巨阴贪",
    ];
    const GAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];

    for (k, (lu, quan, ke, ji)) in STANDARD.iter().enumerate() {
        let got = sihua_for(u8::try_from(k).expect("0..10"), SihuaSchool::Standard);
        assert_eq!(
            (got.lu, got.quan, got.ke, got.ji),
            (*lu, *quan, *ke, *ji),
            "{} 干的四化对不上",
            GAN[k],
        );
        // 口诀交叉：第 j 字应是第 j 化之星的辨识字
        let want: Vec<char> = MNEMONIC[k].chars().collect();
        for (j, star) in [got.lu, got.quan, got.ke, got.ji].into_iter().enumerate() {
            assert!(
                star.contains(want[j]),
                "{} 干第 {} 化：口诀作「{}」，表作「{star}」",
                GAN[k],
                j + 1,
                want[j],
            );
        }
    }
}

/// 中州派与通行版恰好在三干上分岔，且那三处是同一条学理的三个后果。
///
/// 王亭之一系主张左辅右弼属辅曜、不入四化，于是通行表里由右弼化科的戊、
/// 由左辅化科的壬都要换星，庚随之调整。**三干必须一起变**——只改其一两处，
/// 这个流派自身就不自洽了，而盘面上看不出任何异样。原先只验了庚。
#[test]
fn the_zhongzhou_school_diverges_on_exactly_three_stems() {
    let mut diverged = Vec::new();
    for k in 0..10u8 {
        let a = sihua_for(k, SihuaSchool::Standard);
        let b = sihua_for(k, SihuaSchool::Quanshu);
        if (a.lu, a.quan, a.ke, a.ji) != (b.lu, b.quan, b.ke, b.ji) {
            diverged.push((k, b));
        }
    }
    let ids: Vec<u8> = diverged.iter().map(|(k, _)| *k).collect();
    assert_eq!(ids, vec![4, 6, 8], "应恰在戊(4)/庚(6)/壬(8) 三干分岔，实为 {ids:?}");
    for (k, b) in &diverged {
        assert_eq!(b.ke, if *k == 4 { "太阳" } else { "天府" }, "第 {k} 干中州派的化科");
    }
    // 该派的立论：左辅右弼一概不入四化——十干扫一遍，两星都不该出现
    for k in 0..10u8 {
        let s = sihua_for(k, SihuaSchool::Quanshu);
        for star in [s.lu, s.quan, s.ke, s.ji] {
            assert!(
                star != "左辅" && star != "右弼",
                "中州派主张左辅右弼不入四化，第 {k} 干却出了「{star}」",
            );
        }
    }
}

/// 大限：起运岁、顺逆、宫名串三条都对源，而不是只验一张盘。
///
/// 两源同述（<https://iztro.com/learn/basis>、<https://zhuanlan.zhihu.com/p/718987833>）：
/// 起运岁 = 五行局数；第一大限固定在命宫；顺逆由「年干阴阳 + 性别」定，阳男阴女顺、阴男阳女逆。
///
/// 判据取**两源都逐宫列出的那两串宫名**，比只验起岁强得多：
/// 顺行作 命→父母→福德→田宅→官禄→交友，逆行作 命→兄弟→夫妻→子女→财帛→疾厄。
/// 把顺逆判反、或把宫名的排布方向弄反，这两串立刻不成立，而起运岁仍会是对的。
#[test]
fn the_major_limits_walk_the_palaces_both_sources_list() {
    use crate::limit::major_limits;

    // 阳年（甲=0）+ 男 → 顺行；命宫取子（0）便于逐宫核对
    let f = major_limits(0, 3, 0, true);
    assert!(f.forward, "阳男应顺行");
    assert_eq!(f.start_age, 3, "木三局 3 岁起运");
    assert_eq!(f.steps[0].start_age, 3);
    assert_eq!(f.steps[0].end_age, 12, "十年一限，含两端");
    assert_eq!(f.steps[1].start_age, 13);
    let names: Vec<&str> = f.steps.iter().take(6).map(|s| s.palace).collect();
    assert_eq!(
        names,
        ["命宫", "父母", "福德", "田宅", "官禄", "交友"],
        "顺行的宫名串（两源同列）",
    );

    // 阴年（乙=1）+ 男 → 逆行
    let b = major_limits(0, 2, 1, true);
    assert!(!b.forward, "阴男应逆行");
    assert_eq!(b.start_age, 2, "水二局 2 岁起运");
    let names: Vec<&str> = b.steps.iter().take(6).map(|s| s.palace).collect();
    assert_eq!(
        names,
        ["命宫", "兄弟", "夫妻", "子女", "财帛", "疾厄"],
        "逆行的宫名串（两源同列）",
    );

    // 四种组合的顺逆：阳男顺、阴女顺、阴男逆、阳女逆
    for (stem, male, want) in [(0, true, true), (1, false, true), (1, true, false), (0, false, false)] {
        let g = major_limits(0, 5, stem, male);
        assert_eq!(g.forward, want, "年干{stem} 性别男={male} 的顺逆");
    }

    // 十二步走满一轮：地支不重不漏
    let seen: std::collections::BTreeSet<u8> = f.steps.iter().map(|s| s.branch_index).collect();
    assert_eq!(seen.len(), 12, "十二步应走遍十二宫");

    // 步序与每一步的起讫岁数，十二步逐个对。
    //
    // 上面只问了第 0、1 步的 start_age 与第 0 步的 end_age，`step` 字段一次也没问过。
    // 于是把 `step: i + 1` 写成 `i * 1`（步序从 0 起）、把 `end_age` 里的 `i * 10`
    // 写成 `i / 10`（第 0 步照样是 ju+9，之后全错）都没有测试红。
    for (i, st) in f.steps.iter().enumerate() {
        let i = u32::try_from(i).expect("十二步");
        assert_eq!(st.step, i + 1, "步序从 1 起，逐步加一");
        assert_eq!(st.start_age, 3 + i * 10, "第 {} 步的起岁", i + 1);
        assert_eq!(st.end_age, st.start_age + 9, "一限十年，含两端");
    }
}

/// 流年：太岁支入宫，不涉顺逆也不涉性别。
#[test]
fn the_annual_palace_follows_the_year_branch() {
    use crate::limit::annual_palace;
    // 2024 甲辰年 → 年支辰（4）
    assert_eq!(annual_palace(0, 2024).0, 4, "2024 为辰年");
    assert_eq!(annual_palace(0, 2020).0, 0, "2020 为子年");
    assert_eq!(annual_palace(0, 1990).0, 6, "1990 为午年");
    // 同一年、不同命宫 → 支序不变，宫名随命宫移
    assert_eq!(annual_palace(0, 2024).0, annual_palace(5, 2024).0);
    assert_ne!(annual_palace(0, 2024).1, annual_palace(5, 2024).1);
    // 十二年走遍十二宫
    let seen: std::collections::BTreeSet<u8> = (2020..2032).map(|y| annual_palace(0, y).0).collect();
    assert_eq!(seen.len(), 12);

    // 宫名本身也要钉住，不能只说「两个命宫下不一样」。
    //
    // 宫名序 = (命宫支 − 太岁支) mod 12。把那个减号写成加号，两个命宫下的名字**仍然**
    // 各不相同，上面那条不等式照样成立——所以它拦不住。这里逐个报出该是哪一宫。
    for (ming, year, want) in [
        (0_u8, 2024_i32, "官禄"), // 辰(4) 入命宫子(0) 起的盘：(0−4) mod 12 = 8
        (0, 2020, "命宫"),        // 子年子命 → 太岁入命
        (0, 1990, "迁移"),        // 午(6) → (0−6) mod 12 = 6
        (5, 2024, "兄弟"),        // (5−4) mod 12 = 1 → 兄弟
    ] {
        let (_, name) = annual_palace(ming, year);
        assert_eq!(name, want, "命宫支 {ming}、{year} 年的流年宫");
    }
}
