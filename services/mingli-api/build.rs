use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn walk(path: &Path, files: &mut Vec<PathBuf>) {
    assert!(
        !fs::symlink_metadata(path).unwrap().file_type().is_symlink(),
        "build inputs must not be symlinks"
    );
    if path.is_dir() {
        println!("cargo:rerun-if-changed={}", path.display());
        for entry in fs::read_dir(path).unwrap() {
            walk(&entry.unwrap().path(), files);
        }
    } else {
        files.push(path.to_owned());
    }
}
fn part(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}
/// The repository this crate was checked out in, if it is being built there.
///
/// Built from a published package (`cargo publish` verification, `cargo install`,
/// a registry dependency) there is no repository around it: two levels up is
/// `target/` or the registry cache, and the old unconditional walk of `crates/`
/// panicked there. Only a directory whose `Cargo.toml` declares a workspace and
/// whose `services/mingli-api` is this manifest counts.
fn repository_root(manifest: &Path) -> Option<&Path> {
    let root = manifest.parent()?.parent()?;
    let is_workspace = fs::read_to_string(root.join("Cargo.toml"))
        .is_ok_and(|toml| toml.lines().any(|l| l.trim() == "[workspace]"));
    let is_this_member = root
        .join("services/mingli-api")
        .canonicalize()
        .is_ok_and(|p| manifest.canonicalize().is_ok_and(|m| m == p));
    (is_workspace && is_this_member).then_some(root)
}
fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // In the repository: every crate's source plus the workspace lockfile.
    // From a package: the package's own files, whose Cargo.lock pins every
    // dependency by version and checksum. A distinct domain tag keeps the two
    // kinds of digest from ever being compared as if they were the same thing.
    let (root, tag, mut files) = match repository_root(&manifest) {
        Some(root) => {
            let mut files = vec![root.join("Cargo.toml"), root.join("Cargo.lock")];
            walk(&root.join("crates"), &mut files);
            (root.to_owned(), b"mingli-source-v1".as_slice(), files)
        }
        None => (manifest.clone(), b"mingli-package-source-v1".as_slice(), Vec::new()),
    };
    walk(&manifest, &mut files);
    files.sort();
    files.dedup();
    let mut source = Sha256::new();
    part(&mut source, tag);
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        let name = path.strip_prefix(&root).unwrap().to_str().unwrap();
        part(&mut source, name.as_bytes());
        part(&mut source, &fs::read(path).unwrap());
    }
    let source = format!("{:x}", source.finalize());
    let rustc = Command::new(env::var("RUSTC").unwrap())
        .arg("-vV")
        .output()
        .unwrap();
    assert!(rustc.status.success());
    let mut build = Sha256::new();
    part(&mut build, b"mingli-build-v1");
    part(&mut build, source.as_bytes());
    part(&mut build, &rustc.stdout);
    for key in [
        "TARGET",
        "PROFILE",
        "OPT_LEVEL",
        "DEBUG",
        "CARGO_ENCODED_RUSTFLAGS",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
        part(&mut build, key.as_bytes());
        part(&mut build, env::var(key).unwrap_or_default().as_bytes());
    }
    let mut features: Vec<_> = env::vars()
        .filter(|(k, _)| k.starts_with("CARGO_FEATURE_"))
        .collect();
    features.sort();
    for (k, v) in features {
        part(&mut build, k.as_bytes());
        part(&mut build, v.as_bytes());
    }
    println!("cargo:rustc-env=MINGLI_SOURCE_SHA256={source}");
    println!(
        "cargo:rustc-env=MINGLI_BUILD_ID=mingli-v1-{:x}",
        build.finalize()
    );
}
