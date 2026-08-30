//! 门面与装配根必须列出同一批叶。
//!
//! 门面的清单里没有任何一片叶——它的每个 feature 都转发给装配根。但「转发哪些」
//! 仍是一张手写的表，漏掉一片不会有任何报错：`cargo build` 照过，只是那片叶
//! 在 `mingli` 这个名字下不存在，而在 `mingli-registry` 下存在。这条测试读两份
//! 清单，逼它们对上。

use std::{fs, path::PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

/// 取一份 manifest 的 `[features]` 段里，形如 `名 = [...]` 的 feature 名。
fn features(manifest: &str, keep: impl Fn(&str) -> bool) -> Vec<String> {
    let body = manifest.split("[features]").nth(1).expect("清单里应有 [features] 段");
    body.lines()
        .take_while(|l| !l.starts_with('['))
        .filter_map(|l| l.split_once(" = ").map(|(n, rest)| (n.trim().to_string(), rest.to_string())))
        .filter(|(n, rest)| !n.starts_with('#') && keep(rest))
        .map(|(n, _)| n)
        .filter(|n| n != "default" && n != "full")
        .collect()
}

#[test]
fn the_facade_forwards_exactly_the_leaves_the_composition_root_registers() {
    let root = workspace_root();
    let reg = fs::read_to_string(root.join("crates/mingli-registry/Cargo.toml")).unwrap();
    let facade = fs::read_to_string(root.join("crates/mingli/Cargo.toml")).unwrap();

    // 叶名取自装配根的 `full`——它就是「我全都要」的定义。不要去认
    // `= ["dep:mingli-` 那种形状：一片叶的 feature 值多写一项（`astrology`
    // 为了区分带不带本地星历就多写了一项），那种认法立刻少认一片。
    let mut want: Vec<String> = reg
        .split("full = [")
        .nth(1)
        .expect("装配根应有 full")
        .split(']')
        .next()
        .expect("full 应是一个数组")
        .split(',')
        .filter_map(|s| {
            let s = s.trim().trim_matches('"');
            (!s.is_empty()).then(|| s.to_string())
        })
        .collect();
    let mut got = features(&facade, |rest| rest.contains("mingli-registry/"));
    want.sort();
    got.sort();

    assert!(want.len() >= 20, "只从装配根读出 {} 片叶，读法怕是失效了", want.len());

    // 两条，方向相反，缺一不可：
    // 少转发一片，那片叶在 `mingli` 这个名字下就不存在；
    // 转发一个装配根没有的名字，`cargo build` 会炸在使用者那边而不是这里。
    for leaf in &want {
        assert!(got.contains(leaf), "门面没有转发 `{leaf}`——它在装配根的 full 里");
    }
    for name in &got {
        assert!(
            reg.contains(&format!("\n{name} = [")),
            "门面转发 `{name}`，而装配根没有这个 feature"
        );
    }

    // `full` 也要是全的——它是「我全都要」那个名字，少一片同样没有报错。
    let full = facade.split("full = [").nth(1).expect("门面应有 full").split(']').next().unwrap();
    for leaf in &want {
        assert!(full.contains(&format!("\"{leaf}\"")), "门面的 full 里少了 `{leaf}`");
    }
}
