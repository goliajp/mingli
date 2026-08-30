//! 公开出去的东西得有人用——只有自己的测试够得着的公开函数，是算了却没接出去的活。
//!
//! 这条守的是一类真出过两次的缺陷，两次都长得一模一样：
//!
//! - `mingli_ziwei::limit::annual_palace` 算流年宫。该叶的 `answers()` 一直写着它算这个，
//!   而输出里从来没有过——算完就丢，只有它自己的测试在调。
//! - `mingli_astrology::progression::decades` 是给盘面用的十年刻度。盘面后来一格推运都不带了，
//!   它却留着，文档还写着「盘面所出的那一份」，描述的是两次提交前就不存在的状态。
//!
//! 两次都不会报错，也不会让任何测试变红：函数是对的、测试是绿的，只是**没人问它**。
//! 靠读代码发现不了，因为每一处单看都合理。
//!
//! ## 判「够不够得着」的办法
//!
//! 在**去掉 `#[cfg(test)]` 块之后的**源码里找这个名字：
//!
//! - 别的文件里出现 `名字(` → 有人调
//! - 自己文件里出现不止一次（定义算一次）→ 内部自用
//! - 出现 `"名字"` → serde 的 `default = "..."` 之类按字符串引用的
//!
//! 一个都没有 = 只有测试够得着。那时只有两条路：接出去，或者删掉。
//! 第三条路是把它列进 [`ALLOWED`] 并写明理由——那张表要能读懂，不是豁免清单。

use std::path::{Path, PathBuf};

/// 允许「只有测试够得着」的公开函数，以及为什么。
///
/// 每一条都要说清它面向的是谁：不是「暂时没人用」，而是**它的调用方不在这份源码里**。
const ALLOWED: &[(&str, &str)] = &[
    ("router", "承接层给测试用的进程内组装口，doc 里写明了缘由；生产走 `router_with` 注入后端"),
    ("analysis", "wasm 导出，调用方是浏览器里的 JS，不在这份源码里"),
    ("longitudes", "wasm 导出，调用方是浏览器里的 JS。它存在是因为排一张盘里九成七的时间花在这九个数上（实测整盘 286.7 µs、只算位置 278.1 µs），只要位置的人不该被迫排一张盘"),
    ("astrology_with", "wasm 导出，调用方是浏览器里的 JS。它存在是因为位置本地算要背 VSOP87D 那份表——实测同一段排盘代码，本地算 857,633 字节、位置由调用方给 79,863 字节，差 90.7%"),
    ("cast_all", "编排层的简单形态（`id → 盘`）。仓内两条交付路都要 meta，走的是 `cast_all_detailed`；这一个是给只要盘的下游用的"),
    ("trigram_name", "薄包装，doc 明写「便于无 gua 依赖的调用方读名」——面向的就是仓外"),
    ("compute_year", "藏历的年入口（只给年、不给时刻）。叶自己走 `compute_at`，这一个面向只关心年度循环要素的调用方"),
];

/// 不问「有没有仓内调用方」的层：L0 石头与 L1/L2 主干。
///
/// 按本仓自己的分类，这几层的用途就是**换个项目还能用**——它们的调用方本来就该在仓外。
/// 拿「仓内没人调」去判它们，等于把设计目标当成缺陷。
/// 叶（L3）与其上则相反：那里的公开函数没人问，就是算了却没接出去。
const REUSABLE_LAYERS: &[&str] = &[
    "mingli-core",
    "mingli-astro",
    "mingli-ephemeris",
    "mingli-contract",
    "mingli-ganzhi",
    "mingli-gua",
    "mingli-luoshu",
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).expect("仓库根").to_path_buf()
}

/// 递归收集 `crates/*/src` 与 `services/*/src` 下的 `.rs`。
fn sources() -> Vec<(String, String)> {
    fn walk(dir: &Path, out: &mut Vec<(String, String)>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs")
                && let Ok(t) = std::fs::read_to_string(&p)
            {
                out.push((p.display().to_string(), t));
            }
        }
    }
    let root = workspace_root();
    let mut out = Vec::new();
    for top in ["crates", "services"] {
        let Ok(entries) = std::fs::read_dir(root.join(top)) else { continue };
        for e in entries.flatten() {
            walk(&e.path().join("src"), &mut out);
        }
    }
    out
}

/// 去掉 `#[cfg(test)]` 之后的那一块（内联 `mod tests` 与单个测试函数都算）。
///
/// 不做这一步，「只有测试在调」就跟「有人在调」分不开——而那正是要分的那件事。
fn without_tests(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(at) = rest.find("#[cfg(test)]") {
        out.push_str(&rest[..at]);
        let after = &rest[at..];
        // 跳到该项的第一个 `{`，再括号配平地跳过整块
        let Some(open) = after.find('{') else { break };
        let bytes = after.as_bytes();
        let mut depth = 0usize;
        let mut end = open;
        for (i, b) in bytes.iter().enumerate().skip(open) {
            match b {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// 名字作为「整个标识符」在**代码行**里出现了几次。
///
/// 只认 `名字(` 是不够的：`jdn.map_or(1, calendar_day_parkha)` 把函数当值传，
/// 后面没有括号——第一版就这么把一个真有人用的函数报成了孤儿。
/// 反过来注释行不算：doc 里提一句不等于有人调它，那正是要分辨的事。
fn mentions(hay: &str, name: &str) -> usize {
    let mut n = 0;
    for line in hay.lines() {
        let t = line.trim_start();
        if t.starts_with("//") {
            continue;
        }
        let mut from = 0;
        while let Some(at) = line[from..].find(name) {
            let abs = from + at;
            let before_ok = abs == 0 || {
                let c = line[..abs].chars().next_back().unwrap_or(' ');
                !c.is_alphanumeric() && c != '_'
            };
            let after = line[abs + name.len()..].chars().next().unwrap_or(' ');
            if before_ok && !after.is_alphanumeric() && after != '_' {
                n += 1;
            }
            from = abs + name.len();
        }
    }
    n
}

#[test]
fn every_public_function_is_reachable_from_something_that_is_not_a_test() {
    let files = sources();
    let stripped: Vec<(String, String)> = files.iter().map(|(p, t)| (p.clone(), without_tests(t))).collect();

    let mut total = 0usize;
    let mut orphans = Vec::new();
    for (path, body) in &stripped {
        for line in body.lines() {
            let line = line.trim_start();
            let Some(rest) = line.strip_prefix("pub fn ") else { continue };
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if name.is_empty() || name.starts_with('_') {
                continue;
            }
            total += 1;
            if ALLOWED.iter().any(|(n, _)| *n == name) {
                continue;
            }
            if REUSABLE_LAYERS.iter().any(|c| path.contains(&format!("/{c}/src/"))) {
                continue;
            }
            let elsewhere: usize =
                stripped.iter().filter(|(p, _)| p != path).map(|(_, t)| mentions(t, &name)).sum();
            // 定义那一行自己就长成 `pub fn 名字(`，`mentions` 会数上它，所以自用的门槛是 >1。
            // 第一版写成 `own == 0`，于是这条永远不成立、整条守卫空转——
            // 种一个真的没人问的函数进去，它照样绿。反例探测台就是这么抓到的。
            let own = mentions(body, &name);
            if elsewhere == 0 && own <= 1 {
                orphans.push(format!("{path} :: {name}"));
            }
        }
    }

    assert!(total > 300, "只扫出 {total} 个公开函数，扫法怕是失效了");
    assert!(
        orphans.is_empty(),
        "这些公开函数只有测试够得着——算了却没接出去：\n  {}\n\n\
         两条路：接到用例/交付层去，或者删掉。若它的调用方本就不在这份源码里\n\
         （wasm 导出、serde 按字符串引用、给下游 crate 用的口），列进 ALLOWED 并写明面向谁。",
        orphans.join("\n  ")
    );
}
