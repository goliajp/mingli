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

# 逐叶开关：二十四片各自单独装配一次。
#
# 这一段不是凑数——「只要其中一片」是这棵树对使用者的一个承诺，而承诺只有真的裁一次
# 才算数。装配根里每片一行 `#[cfg(feature = ...)]`，少写一行、或某片叶被别处无条件依赖，
# 单开它就会编不过（或悄悄把别的叶一并拉进来）。
for leaf in bazi ziwei astrology jyotish qizhengsiyu yijing geomancy sikidy ifa cartomancy \
            meihua xiaoliuren zeri maya pawukon mahabote liuren qimen taiyi tibetan \
            numerology gematria abjad wuge; do
  run "只装 $leaf" --no-default-features --features "$leaf" || fail=1
done

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
  wasm "wasm32 只装四柱一片"    --no-default-features --features bazi || fail=1
else
  printf '\n没装 wasm32-unknown-unknown，跳过 wasm 两条（rustup target add wasm32-unknown-unknown）\n'
fi

# 「关掉即裁掉」不能只验编译得过——编译永远过，星历照样被拉进来。
# 这一条直接查依赖图：轻量构建里 vsop87 必须不在。
printf '\n=== 轻量构建真的裁掉了星历\n'
if cargo tree -p mingli-wasm --no-default-features -e normal 2>/dev/null | grep -q vsop87; then
  printf '  ✗ 关掉 feature 后 vsop87 仍在依赖图里——「轻量构建」这句话不成立\n'
  printf '    多半是某个消费者按默认把星历叶全开了：继承来的依赖不许写 default-features=false，\n'
  printf '    要在根 manifest 把它设成 opt-in，再由各消费者显式声明要哪几片\n'
  fail=1
else
  printf '  ✓\n'
fi


# 每个 crate 各自单独跑一遍测试。
#
# `cargo test --workspace` 跑的是**合并后**的 feature 集：只要有一个成员开了某个 feature，
# 整条依赖图上的那个 crate 就带着它编。于是一个 crate 的 dev-dependency 少写了 feature，
# 整仓一起跑照样绿，单独跑才炸。`mingli-app` 就这么坏过一次——它的 dev-dependency
# 停在旧的三个叶名上，占卜那几片压根不在注册表里，八条测试全挂，而日常一次都没红过。
printf '\n=== 每个 crate 单独跑（feature 合并掩盖不了）\n'
members=$(cargo metadata --no-deps --format-version 1 \
  | tr ',' '\n' | grep -o '"name":"mingli-[a-z0-9-]*"' | cut -d'"' -f4 | sort -u)
n=0
for m in $members; do
  n=$((n+1))
  if cargo test -p "$m" 2>&1 | grep -qE '^error|FAILED'; then
    printf '  ✗ %s 单独跑挂了\n' "$m"
    fail=1
  fi
done
printf '  ✓ %d 个 crate 各自单独跑过\n' "$n"

if [ "$fail" -ne 0 ]; then
  printf '\nfeature 矩阵有组合未通过。\n'
  exit 1
fi
printf '\n全绿。\n'
