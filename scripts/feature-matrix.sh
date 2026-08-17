#!/usr/bin/env bash
# 三种 feature 组合下 registry 都要绿，外加 wasm 目标真能编。
#
# 三片星历叶（astrology / jyotish / qizhengsiyu）是可裁的——关掉它们能得到不含 VSOP87
# 的轻量构建。可「能裁」这件事只在真的裁一次跑一遍时才算数：默认组合永远开着全部 feature，
# 于是任何写死「共 21 片」「r[2] 是占星」的断言都不会在日常测试里露馅。
#
# wasm 那两条同理：日常 `cargo test` 跑的是宿主目标，wasm32 编不过要等到发版才发现。
#
#   ./scripts/feature-matrix.sh
#
# 需要 `rustup target add wasm32-unknown-unknown`；没装就跳过并说一声。

set -euo pipefail
cd "$(dirname "$0")/.."

run() {
  printf '\n=== %s\n' "$1"
  shift
  if cargo test -p mingli-registry "$@" 2>&1 | tail -25 | grep -qE '^error|FAILED'; then
    printf '  ✗ 挂了\n'
    return 1
  fi
  printf '  ✓\n'
}

fail=0
run "全开（默认）"                                                     || fail=1
run "全关（无星历，轻量构建）"  --no-default-features                    || fail=1
run "只开 astrology"           --no-default-features --features astrology || fail=1
run "只开 jyotish"             --no-default-features --features jyotish   || fail=1
run "只开 qizhengsiyu"         --no-default-features --features qizhengsiyu || fail=1

wasm() {
  printf '\n=== %s\n' "$1"
  shift
  if cargo check -p mingli-wasm --target wasm32-unknown-unknown "$@" 2>&1 | tail -25 | grep -qE '^error'; then
    printf '  ✗ 挂了\n'
    return 1
  fi
  printf '  ✓\n'
}

if rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown; then
  wasm "wasm32 全开"                                   || fail=1
  wasm "wasm32 轻量（无星历）"  --no-default-features    || fail=1
else
  printf '\n没装 wasm32-unknown-unknown，跳过 wasm 两条（rustup target add wasm32-unknown-unknown）\n'
fi

if [ "$fail" -ne 0 ]; then
  printf '\nfeature 矩阵有组合未通过。\n'
  exit 1
fi
printf '\n全绿。\n'
