//! 确定性谱的质量守卫。
//!
//! 每片叶都要在 `profile()` 里声明自己算得准的到哪、从哪开始不准。这份声明是本项目
//! 对外最要紧的诚实凭据，所以它自己也要有守卫——否则一句「流派分歧」四个字就能糊过去，
//! 读者既不知道查过谁，也不知道分歧卡在哪一步。
//!
//! 这里守两件事：**每条 Und 都带 🟡 标记**（便于全仓一眼扫出欠定面），
//! **note 长度够写下实质内容**。长度不是为了凑字数——是为了让「查过哪些源、
//! 各家怎么说、缺口在哪」这三样至少写得下一样。

use mingli_contract::{DetItem, Determinism};
use mingli_registry::{registry, word_registry};

/// Und 项的 note 至少要这么长（按**字符**计，中文一字算一个）。
const MIN_UND_NOTE_CHARS: usize = 20;
/// Det / Sto 项的 note 也不许空着——它得说清凭什么算得准。
const MIN_NOTE_CHARS: usize = 6;

/// 收齐全部叶的 profile：`(叶 id, 项名, 档次, note)`。
fn all_items() -> Vec<(&'static str, &'static DetItem)> {
    let mut out: Vec<(&'static str, &'static DetItem)> = Vec::new();
    for e in registry() {
        let id = e.id();
        for item in e.profile() {
            out.push((id, item));
        }
    }
    for e in word_registry() {
        let id = e.id();
        for item in e.profile() {
            out.push((id, item));
        }
    }
    out
}

#[test]
fn every_leaf_declares_a_determinism_profile() {
    let mut empty = Vec::new();
    for e in registry() {
        if e.profile().is_empty() {
            empty.push(e.id());
        }
    }
    for e in word_registry() {
        if e.profile().is_empty() {
            empty.push(e.id());
        }
    }
    assert!(
        empty.is_empty(),
        "这几片叶没有声明确定性谱：{empty:?}\n\
         每片叶都要说清自己算得准的到哪、从哪开始不准——两种契约（CastingEngine 与 WordEngine）一视同仁。"
    );
}

#[test]
fn every_undetermined_item_carries_the_marker_and_says_something() {
    let mut bad = Vec::new();
    for (leaf, item) in all_items() {
        if item.status != Determinism::Und {
            continue;
        }
        let chars = item.note.chars().count();
        if !item.note.contains('🟡') {
            bad.push(format!("[{leaf}] {} —— 缺 🟡 标记", item.aspect));
        } else if chars < MIN_UND_NOTE_CHARS {
            bad.push(format!("[{leaf}] {} —— note 只有 {chars} 字：「{}」", item.aspect, item.note));
        }
    }
    assert!(
        bad.is_empty(),
        "欠定项的 note 不合格：\n  {}\n\n\
         每条 Und 都要能追溯：**查过哪些源、各家怎么说、缺口卡在哪一步**，至少写得下一样。\n\
         「随流派分歧」「未查」这类四五个字的说法，读者既不知道你查没查、也不知道该从哪接手。\n\
         注意「查过了定不下」与「根本没查」是两种不同的状态，note 里要分清。",
        bad.join("\n  ")
    );
}

#[test]
fn determinate_items_say_what_they_rest_on() {
    let mut bad = Vec::new();
    for (leaf, item) in all_items() {
        if item.status == Determinism::Und {
            continue;
        }
        let chars = item.note.chars().count();
        if chars < MIN_NOTE_CHARS {
            bad.push(format!("[{leaf}] {} ({:?}) —— note 只有 {chars} 字", item.aspect, item.status));
        }
    }
    assert!(
        bad.is_empty(),
        "确定项也要说清凭什么算得准（校验源 / oracle / 结构不变量）：\n  {}",
        bad.join("\n  ")
    );
}

/// 项名不许重复——同一片叶里两条同名声明，读者无从分辨说的是哪一处。
#[test]
fn no_leaf_declares_the_same_item_twice() {
    let mut dup = Vec::new();
    for e in registry() {
        let mut names: Vec<&str> = e.profile().iter().map(|i| i.aspect).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        if names.len() != before {
            dup.push(e.id());
        }
    }
    assert!(dup.is_empty(), "这几片叶的 profile 有重名项：{dup:?}");
}

/// 全仓的确定性分布：这个数字会随施工变化，测试只做**下界**约束，
/// 防止有人把「算不准」的东西悄悄改标成算得准来让别的检查过关。
#[test]
fn the_undetermined_surface_stays_visible() {
    let items = all_items();
    let und = items.iter().filter(|(_, i)| i.status == Determinism::Und).count();
    assert!(items.len() > 80, "全仓声明项总数不该缩水，实得 {}", items.len());
    assert!(
        und > 0,
        "一条欠定都没有反而可疑——这套体系里真流派分歧客观存在，全标成确定说明有人在粉饰"
    );
}
