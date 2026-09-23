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

# 末尾的 `|| true` 不是兜底，是让下面那条告警有机会开口：`set -euo pipefail` 下
# grep 无匹配即非零，整条赋值会当场终止脚本——退出码是 1，却一个字也不打，
# 而「解析坏了」与「叶少了」正是要靠那句话分开。
LEAVES=$(sed -n '/^\[features\]/,$p' crates/mingli-registry/Cargo.toml \
  | grep -oE '^[a-z][a-z0-9_-]* = \[' | sed 's/ = \[//' | grep -vxE 'default|full' | sort || true)
LEAF_COUNT=$(printf '%s\n' "$LEAVES" | grep -c . || true)

# 解析失效长得跟「本来就没有叶」一模一样：那时循环跑零次，脚本照样绿，
# 而它其实什么都没验。所以先把推导本身钉住——少于二十片就是解析坏了，不是叶少了。
if [ "$LEAF_COUNT" -lt 20 ]; then
  printf '\n✗ 从 manifest 只推出 %s 个叶 feature——多半是 [features] 段的写法变了，\n' "$LEAF_COUNT"
  printf '   这一段于是什么都没验。先修推导，别让它静默跑零次。\n'
  exit 1
fi

fail=0
run "全开（默认）"                                                     || fail=1
run "全关（无星历，轻量构建）"  --no-default-features                    || fail=1
run "只开 astrology"           --no-default-features --features astrology || fail=1
run "只开 jyotish"             --no-default-features --features jyotish   || fail=1
run "只开 qizhengsiyu"         --no-default-features --features qizhengsiyu || fail=1

# 逐叶开关：每片各自单独装配一次。
#
# 这一段不是凑数——「只要其中一片」是这棵树对使用者的一个承诺，而承诺只有真的裁一次
# 才算数。装配根里每片一行 `#[cfg(feature = ...)]`，少写一行、或某片叶被别处无条件依赖，
# 单开它就会编不过（或悄悄把别的叶一并拉进来）。
#
# 叶名从装配根的 manifest 推出来，不写死：写死的名单在加新叶时会静默漏掉那一片，
# 而「可裁」对它就此失效，没有任何东西会说一声。
printf '\n=== 逐叶单装（%s 片，名单由 manifest 推出）\n' "$LEAF_COUNT"
for leaf in $LEAVES; do
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
# 拿一个**真的装着叶**的轻量档位来验，不是空壳。
#
# 从前这里查的是 `--no-default-features`：一片叶都不装，当然没有 vsop87——
# 那句「轻量构建裁掉了星历」对一个什么也算不出的空壳成立，等于没验。
# 同一形状的洞让 `mingli-wasm-astrology-thin@1.1.0` 带着空注册表发了出去。
printf '\n=== 轻量构建真的裁掉了星历（且叶还在）\n'
LIGHT="bazi,ziwei,yijing,meihua,qimen"
light_tree=$(cargo tree -p mingli-wasm --no-default-features --features "$LIGHT" -e normal --prefix none 2>/dev/null | awk '{print $1}' | sort -u)
for want in mingli-bazi mingli-ziwei mingli-yijing mingli-meihua mingli-qimen; do
  grep -qx "$want" <<<"$light_tree" || {
    printf '  ✗ 轻量档位里没有 %s——这一档装到手什么也算不出，而下面那句「没有星历」是白拿的\n' "$want"
    fail=1
  }
done
if grep -qx vsop87 <<<"$light_tree"; then
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
# 门面 crate 叫 `mingli`，没有连字符——从前这里写死了 `mingli-` 前缀，
# 于是它一个人被漏在这条检查之外，而输出照样是「都跑过」。
members=$(cargo metadata --no-deps --format-version 1 \
  | tr ',' '\n' | grep -oE '"name":"mingli(-[a-z0-9-]+)?"' | cut -d'"' -f4 | sort -u)
n=0
for m in $members; do
  n=$((n+1))
  if cargo test -p "$m" 2>&1 | grep -qE '^error|FAILED'; then
    printf '  ✗ %s 单独跑挂了\n' "$m"
    fail=1
  fi
done
# 下限：少数了一个不会有任何报错，只会让那个 crate 从此不被单独跑。
if [ "$n" -lt 35 ]; then
  printf '  ✗ 只跑到 %d 个 crate，枚举方式怕是失效了\n' "$n"; fail=1
else
  printf '  ✓ %d 个 crate 各自单独跑过\n' "$n"
fi


# README 那张 wasm 体积表，核对它与预算表逐格一致。
#
# 「关掉即裁掉」上面已经验过（依赖图里没有 vsop87），但**裁掉多少**是 README 对外给的数字，
# 而那种数字没人验就会慢慢失真：一片叶悄悄拖进星历，体积翻倍，表里还写着旧值。
#
printf '\n=== README 的 wasm 体积表与预算表一致\n'
# 这里**不再自己量一遍**。曾经量过：用 cargo 直出的裸 .wasm，既没过 wasm-bindgen
# 也没过 wasm-opt，于是同一个包在这里是 1.86 MB、在体积闸里是 1.48 MB，
# 两个数都对、说的却不是同一件事，而 README 只能照着其中一个写。
# 现在只有 scripts/wasm-budget.txt 一个数源：体积闸对着它比，npm-pack 发包前对着它核，
# 这里核 README 是否照它写。三处同一个数。
if [ ! -f scripts/wasm-budget.txt ]; then
  printf '  ✗ 没有 scripts/wasm-budget.txt——先跑 ./scripts/wasm-size.sh --record\n'; fail=1
else
  n_row=0
  while read -r name raw gz; do
    case "$name" in ''|'#'*) continue;; esac
    n_row=$((n_row+1))
    # 与 crates/mingli-registry/tests/readme.rs 用同一条换算：整数的四舍五入，
    # `(b + 512) / 1024`。从前这里用的是 Python 的 `:.0f`，它在 .5 上取偶——
    # 166400 字节正好是 162.5 KB，一边得 162、一边得 163，两个守卫对着同一张预算表吵架。
    kb_raw="$(( (raw + 512) / 1024 )) KB"
    kb_gz="$(( (gz + 512) / 1024 )) KB"
    for md in README.md README.zh-CN.md; do
      if ! grep -q "| $kb_raw | $kb_gz |" "$md"; then
        printf '  ✗ %s 少了 %s 那一行（应是 %s / %s）\n' "$md" "$name" "$kb_raw" "$kb_gz"; fail=1
      fi
    done
  done < scripts/wasm-budget.txt
  [ "$n_row" -ge 5 ] || { printf '  ✗ 预算表只有 %s 行，读法怕是失效了\n' "$n_row"; fail=1; }
  [ "$fail" -ne 0 ] || printf '  ✓ %s 档都与预算表对上\n' "$n_row"
fi

if [ "$fail" -ne 0 ]; then
  printf '\nfeature 矩阵有组合未通过。\n'
  exit 1
fi
printf '\n全绿。\n'
