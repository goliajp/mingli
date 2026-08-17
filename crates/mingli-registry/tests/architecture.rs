//! 架构守卫：依赖方向由测试来守，而不是靠自觉。
//!
//! 读取工作区里每个 crate 的 `[dependencies]` 段，断言三条规则：
//!
//! 1. **只向内**：每个 crate 只能依赖层级严格更低的 crate；
//! 2. **编排层不认识叶**：`mingli-engine` 只许依赖端口层；
//! 3. **端口层最干净**：`mingli-contract` 只许依赖共享时刻所在的 L1。
//!
//! 叶与叶之间不得互相依赖，这条由规则 1 蕴含（同层不满足「严格更低」）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 层级序号。数字只表示相对上下，不表示距离。
fn layers() -> BTreeMap<&'static str, u8> {
    let mut m = BTreeMap::new();
    // L0 数学根
    m.insert("mingli-core", 0);
    // L1 物理石
    for c in ["mingli-astro", "mingli-ephemeris"] {
        m.insert(c, 1);
    }
    // L2 端口 + 符号主干
    for c in ["mingli-contract", "mingli-ganzhi", "mingli-gua", "mingli-luoshu"] {
        m.insert(c, 2);
    }
    // L3 叶
    for c in [
        "mingli-bazi", "mingli-ziwei", "mingli-astrology", "mingli-jyotish", "mingli-qizhengsiyu",
        "mingli-yijing", "mingli-geomancy", "mingli-sikidy", "mingli-ifa", "mingli-cartomancy",
        "mingli-meihua", "mingli-xiaoliuren", "mingli-zeri", "mingli-maya", "mingli-pawukon",
        "mingli-mahabote", "mingli-liuren", "mingli-qimen", "mingli-taiyi", "mingli-tibetan",
        "mingli-numerology", "mingli-gematria", "mingli-abjad", "mingli-wuge",
    ] {
        m.insert(c, 3);
    }
    // L4 编排机制与释义
    for c in ["mingli-engine", "mingli-interpret"] {
        m.insert(c, 4);
    }
    // L5 跨叶分析（消费编排结果）
    m.insert("mingli-analysis", 5);
    // L6 用例
    m.insert("mingli-app", 6);
    // L7 装配根
    m.insert("mingli-registry", 7);
    // L8 承接层
    m.insert("mingli-api", 8);
    m.insert("mingli-wasm", 8);
    m
}

fn workspace_root() -> PathBuf {
    // …/crates/mingli-registry → 仓库根
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).unwrap().to_path_buf()
}

/// 取一个 crate 的 `[dependencies]` 段里的工作区内部依赖（dev-dependencies 不计——
/// 测试与基准可以自由装配全树，那不构成生产依赖）。
fn internal_deps(manifest: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(manifest).expect("Cargo.toml 应可读");
    let mut in_deps = false;
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = line == "[dependencies]";
            continue;
        }
        if in_deps
            && line.starts_with("mingli-")
            && let Some(name) = line.split_whitespace().next()
        {
            out.push(name.to_string());
        }
    }
    out
}

fn manifests() -> Vec<(String, PathBuf)> {
    let root = workspace_root();
    let mut out = Vec::new();
    for dir in ["crates", "services"] {
        for entry in std::fs::read_dir(root.join(dir)).expect("目录应存在").flatten() {
            let manifest = entry.path().join("Cargo.toml");
            if manifest.is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                out.push((name, manifest));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn every_dependency_points_strictly_inward() {
    let layers = layers();
    let mut violations = Vec::new();
    for (name, manifest) in manifests() {
        let Some(&own) = layers.get(name.as_str()) else {
            panic!("crate `{name}` 未登记层级——新增 crate 时请在本测试的 layers() 里定位它");
        };
        for dep in internal_deps(&manifest) {
            let Some(&other) = layers.get(dep.as_str()) else {
                panic!("依赖 `{dep}` 未登记层级");
            };
            if other >= own {
                violations.push(format!("{name}(L{own}) → {dep}(L{other})"));
            }
        }
    }
    assert!(violations.is_empty(), "依赖必须只向内，以下越界：\n  {}", violations.join("\n  "));
}

#[test]
fn orchestration_does_not_know_any_leaf() {
    let deps = internal_deps(&workspace_root().join("crates/mingli-engine/Cargo.toml"));
    assert_eq!(
        deps,
        ["mingli-contract"],
        "编排层只许依赖端口层——一旦它认识某片叶，「长新叶不动根」就破了"
    );
}

#[test]
fn the_port_layer_stays_thin() {
    let deps = internal_deps(&workspace_root().join("crates/mingli-contract/Cargo.toml"));
    assert_eq!(deps, ["mingli-astro"], "端口层只许依赖共享时刻所在的 L1");
}

#[test]
fn the_composition_root_is_the_only_place_that_lists_leaves() {
    let layers = layers();
    let leaf_count = |manifest: &Path| {
        internal_deps(manifest)
            .iter()
            .filter(|d| layers.get(d.as_str()) == Some(&3))
            .count()
    };
    let root = workspace_root();
    assert!(leaf_count(&root.join("crates/mingli-registry/Cargo.toml")) >= 24, "装配根应列出全部叶");
    // 用例层允许认识少数具体叶（四柱/紫微是它的领域），但不该变成第二个装配根。
    assert!(leaf_count(&root.join("crates/mingli-app/Cargo.toml")) <= 3, "用例层不该退化成装配根");
    for outer in ["services/mingli-api", "crates/mingli-wasm"] {
        assert_eq!(leaf_count(&root.join(outer).join("Cargo.toml")), 0, "{outer} 应经装配根取叶");
    }
}

#[test]
fn the_roots_never_reach_up_into_a_leaf() {
    // 规则 1 已蕴含这一条，但根被叶污染是最贵的一种回退（放大半径 = 全树），
    // 值得一条点名到底的断言：说清是哪个根、伸手够了哪片叶。
    let layers = layers();
    let root = workspace_root();
    let mut violations = Vec::new();
    for (name, level) in [
        ("mingli-core", 0u8),
        ("mingli-astro", 1),
        ("mingli-ephemeris", 1),
        ("mingli-contract", 2),
        ("mingli-ganzhi", 2),
        ("mingli-gua", 2),
        ("mingli-luoshu", 2),
    ] {
        let manifest = root.join("crates").join(name).join("Cargo.toml");
        assert!(manifest.is_file(), "根 crate `{name}` 不见了——是改名还是被删了？");
        assert_eq!(layers.get(name), Some(&level), "根 crate `{name}` 的层级登记与本测试不符");
        for dep in internal_deps(&manifest) {
            if layers.get(dep.as_str()).is_some_and(|&l| l >= 3) {
                violations.push(format!("根 {name}(L{level}) 伸手够到了 {dep}"));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "L0–L2 是全树的地基，只许被依赖、不许依赖上层：\n  {}\n\
         若某个根真的需要叶的知识，那说明这块知识本来就属于叶——把它下沉，别把根拉高。",
        violations.join("\n  ")
    );
}

#[test]
fn the_interpretation_layer_only_knows_the_ports() {
    // 释义层不该认识任何叶、也不该认识编排实现——它吃的是 `LeafOutput` 与 JSON 串，
    // 两者都由端口层定义。一旦它伸手去拿某片叶的具体类型，护栏就不再是「对任意叶都成立」的了。
    let deps = internal_deps(&workspace_root().join("crates/mingli-interpret/Cargo.toml"));
    assert_eq!(
        deps,
        ["mingli-contract"],
        "释义层只许依赖端口层：它组装的是提示词，不是盘——盘由谁算、算的是什么，它不必知道"
    );
}
