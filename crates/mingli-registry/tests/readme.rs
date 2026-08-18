//! README 里对外声称的数字与清单，必须与仓库现状一致。
//!
//! 两份 README 是这个项目的门面：crate 数、叶数、问局数、端点表、分层树。它们都是**可测的**，
//! 却全靠人记得改——加一片叶、加一个端点、拆一个 crate，README 不会有任何反应，
//! 于是门面慢慢变成一份过时的说明书，而读的人无从分辨哪一行还作数。
//!
//! 这里把每一项都对回真实来源：workspace 成员表、注册表、[`mingli_contract::Intent`]、
//! api 的路由表。中英两份一起对，顺带保证它们彼此没有说岔。
//!
//! 截图屏数不在这里对：那个数只有截图工具自己算得准（它的屏表由叶表切片拼出来），
//! 在这边重算一遍手法等于把同一处逻辑写两份，见 `web/e2e/shoot.mjs` 收尾处。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("crates/").parent().expect("仓库根").to_path_buf()
}

fn read(rel: &str) -> String {
    let p = workspace_root().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} 应可读：{e}", p.display()))
}

/// 两份 README（英文原本 + 中文），逐份检查。
fn readmes() -> [(&'static str, String); 2] {
    [("README.md", read("README.md")), ("README.zh-CN.md", read("README.zh-CN.md"))]
}

/// workspace 成员数（`[workspace] members` 里的路径条数）。
fn member_count() -> usize {
    let root = read("Cargo.toml");
    let body = root.split("members").nth(1).expect("根清单应有 members");
    let body = &body[..body.find(']').expect("members 表应闭合")];
    body.matches("crates/").count() + body.matches("services/").count()
}

/// 在文中找 `<数字> + 某个说法`，返回那个数字。找不到该说法返回 `None`。
///
/// 刻意不写成「文中出现过 N 吗」：README 里到处是数字，随便撞上一个就绿，
/// 守卫等于没有（本文件的截图屏数那条就这么假绿过一次，后来搬去了截图工具里）。
fn number_before(text: &str, phrase: &str) -> Option<usize> {
    let at = text.find(phrase)?;
    let head = text[..at].trim_end();
    let digits: String = head.chars().rev().take_while(char::is_ascii_digit).collect();
    digits.chars().rev().collect::<String>().parse().ok()
}

/// 开头那行摘要里的一项：(项目名, 数字后面紧挨着的说法, 应为多少)。
type Claim = (&'static str, &'static str, usize);

#[test]
fn the_headline_counts_match_the_workspace() {
    let crates = member_count();
    let leaves = mingli_registry::registry().len();
    let words = mingli_registry::word_registry().len();
    let intents = mingli_contract::intents().len();

    // 每份 README 的开头一行，各自的说法
    let claims: [(&str, [Claim; 5]); 2] = [
        (
            "README.md",
            [
                ("crate 数", " crates ", crates),
                ("叶总数", " leaves (", leaves + words),
                ("时刻叶数", " time-driven", leaves),
                ("字词叶数", " word-driven", words),
                ("问局数", " intents,", intents),
            ],
        ),
        (
            "README.zh-CN.md",
            [
                ("crate 数", " 个 crate ", crates),
                ("叶总数", " 片叶（", leaves + words),
                ("时刻叶数", " 片时刻叶", leaves),
                ("字词叶数", " 片字词叶", words),
                ("问局数", " 类问局", intents),
            ],
        ),
    ];

    for (name, text) in readmes() {
        let (_, checks) = claims.iter().find(|(f, _)| *f == name).expect("两份都该有说法表");
        let head = text.lines().find(|l| l.starts_with("> ")).expect("开头该有一行摘要");
        for (label, phrase, want) in checks {
            let got = number_before(head, phrase)
                .unwrap_or_else(|| panic!("{name} 的开头一行里找不到「…{phrase}」——句式改过了，本测试要跟着改\n  {head}"));
            assert_eq!(got, *want, "{name} 的 {label} 写的是 {got}，实为 {want}\n  {head}");
        }
    }
}

#[test]
fn every_workspace_member_appears_in_the_layer_tree() {
    let members: BTreeSet<String> = {
        let root = read("Cargo.toml");
        let body = root.split("members").nth(1).expect("根清单应有 members");
        let body = &body[..body.find(']').expect("members 表应闭合")];
        body.split('"')
            .filter(|s| s.starts_with("crates/") || s.starts_with("services/"))
            .filter_map(|s| s.rsplit('/').next())
            .map(str::to_string)
            .collect()
    };
    for (name, text) in readmes() {
        let missing: Vec<_> = members.iter().filter(|m| !text.contains(m.as_str())).collect();
        assert!(missing.is_empty(), "{name} 的分层树漏了这些 crate：{missing:?}");
    }
}

#[test]
fn the_endpoint_table_lists_every_route_and_no_others() {
    let router = read("services/mingli-api/src/lib.rs");
    let listed: BTreeSet<String> = router
        .split(".route(\"")
        .skip(1)
        .filter_map(|s| s.split('"').next())
        .map(str::to_string)
        .collect();
    assert!(listed.len() > 10, "路由表只解析出 {} 条，解析方式怕是失效了", listed.len());

    for (name, text) in readmes() {
        for r in &listed {
            assert!(text.contains(r.as_str()), "{name} 的端点表漏了 `{r}`");
        }
        // 反向：表里写的每条也得真在路由表上
        for line in text.lines().filter(|l| l.starts_with("| `")) {
            for tok in line.split('`').flat_map(|s| s.split_whitespace()) {
                if tok.starts_with("/api/") {
                    assert!(listed.contains(tok), "{name} 的端点表写了 `{tok}`，但 router 上没有这条");
                }
            }
        }
    }
}
