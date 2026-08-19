//! 认领一个问局，就得在答那个问局的用例里真的出现。
//!
//! `answers()` 是叶自己的声明，`route()` 照着它回答「谁来答这类问局」。可**真正答的是用例层**，
//! 而那两处之间没有任何东西对得上——一片叶可以认领「运」、被 `/api/route` 如实列出、
//! 而运势的输出里一个字都没有它。紫微就是这么过了很久：它的 `answers()` 一直写着
//! 「本叶算大限盘与流年入宫」，`major_limits` 在本命盘上，流年宫算完就丢，
//! 而 `/api/fortune` 里连大限都没有。没有任何测试会红——每一处单看都是对的。
//!
//! 这里把两侧摆在一起：认领「运」的每一片，都要在运势输出里有一处属于它的东西。
//! 「属于它的」由下面这张表点名，不靠猜——数个数会被加一减一蒙混，点名则要动就得改表，
//! 改的人得说清那一片的时间线去哪了。

use mingli_app::{bazi::fortune, Birth};
use mingli_contract::{AskTime, Gender, Intent};

/// 认领「运」的叶 → 它在运势输出里的落点，以及那是什么。
///
/// 加一片新的答「运」的叶时，这张表会红：那正是要问的问题——**它的时间线放在哪儿了**。
const FORTUNE_SLOTS: &[(&str, &str, &str)] = &[
    ("bazi", "timeline", "四柱：百年用神供给时序（`at` 是 t 时刻切片，同属这一片）"),
    ("jyotish", "dasha", "印度占星：Vimshottari 大运序列与当前段"),
    ("astrology", "progression", "西洋占星：二次推运（一日一年），五年一格"),
    ("ziwei", "ziwei", "紫微：所问之岁所在的大限步，与所问之年的流年宫"),
];

fn natal() -> Birth {
    Birth {
        year: 1987,
        month: 9,
        day: 17,
        hour: 15,
        minute: 0,
        tz: 8.0,
        gender: Some(Gender::Male),
        true_solar_time: false,
        longitude: None,
    }
}

#[test]
fn every_leaf_that_claims_the_fortune_intent_shows_up_in_the_fortune_answer() {
    let claimers: Vec<&str> = mingli_registry::registry()
        .iter()
        .filter(|e| e.answers().contains(&Intent::Fortune))
        .map(|e| e.id())
        .collect();
    assert!(!claimers.is_empty(), "没有叶认领「运」，取法怕是失效了");

    let t = AskTime { year: 2026, month: 8, day: 19, hour: 10, minute: 0, tz: 8.0 };
    let out = fortune(&natal(), &t, None).expect("样本给了性别，运势应算得出");

    for id in &claimers {
        let (_, slot, what) = FORTUNE_SLOTS
            .iter()
            .find(|(leaf, _, _)| leaf == id)
            .unwrap_or_else(|| {
                panic!(
                    "叶 `{id}` 认领了「运」，但本表没说它的时间线落在运势输出的哪一处。\n\
                     要么把它接进 `mingli_app::bazi::fortune`（并在这里点名），\n\
                     要么它本就不该认领「运」——`answers()` 是对外的承诺，不是备注。"
                )
            });
        let v = &out[*slot];
        assert!(
            !v.is_null(),
            "叶 `{id}` 认领了「运」，运势输出里 `{slot}` 却是空的（本该是：{what}）"
        );
    }

    // 反向：表里点了名的，都得真有叶在认领。否则这张表会慢慢变成一份过时的说明
    for (leaf, _, _) in FORTUNE_SLOTS {
        assert!(
            claimers.contains(leaf),
            "本表说 `{leaf}` 答「运」，但它的 `answers()` 里没有——表与现实要一致"
        );
    }
}

/// 认领「运」不能只是把本命盘原样再给一遍：**换一个所问之时，答案要跟着变**。
///
/// 这条防的是另一种空头支票——接是接进去了，但填的是与 t 无关的东西，
/// 于是「运」和「命」给出同一份内容，读的人无从知道自己看的是哪一个。
#[test]
fn the_fortune_answer_actually_moves_when_the_asked_time_moves() {
    let a = fortune(&natal(), &AskTime { year: 2026, month: 8, day: 19, hour: 10, minute: 0, tz: 8.0 }, None)
        .expect("应可算");
    let b = fortune(&natal(), &AskTime { year: 2044, month: 8, day: 19, hour: 10, minute: 0, tz: 8.0 }, None)
        .expect("应可算");

    for (leaf, slot, what) in FORTUNE_SLOTS {
        // 百年供给时序与推运整条是不随 t 变的（它们是一生的曲线，t 只决定读哪一段），
        // 故这两处比的是「t 时刻切片」而非整条
        let (x, y) = match *leaf {
            "bazi" => (&a["at"], &b["at"]),
            "jyotish" => (&a[*slot]["current"], &b[*slot]["current"]),
            _ => (&a[*slot], &b[*slot]),
        };
        if *leaf == "astrology" {
            continue; // 推运整条与 t 无关：它由本命定，界面按年龄高亮其中一格
        }
        assert_ne!(x, y, "叶 `{leaf}` 的「运」（{what}）在 2026 与 2044 给了同一份内容");
    }
}
