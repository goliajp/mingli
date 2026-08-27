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


# README 那张 wasm 体积表，逐格重量一遍。
#
# 「关掉即裁掉」上面已经验过（依赖图里没有 vsop87），但**裁掉多少**是 README 对外给的数字，
# 而那种数字没人验就会慢慢失真：一片叶悄悄拖进星历，体积翻倍，表里还写着 0.57。
#
# 读数只认 **cargo 自己汇报的产物路径**（`--message-format=json`）。别用「删掉 .wasm 再 build」
# 那一招：cargo 的指纹不看产物在不在，删了照样判定 up-to-date，于是一个也不重新链接——
# 写这段时先踩了这个坑，五种配置量出三种一模一样的数，看上去像「feature 没起作用」。
printf '\n=== README 的 wasm 体积表\n'
if rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown; then
  wasm_size() {
    cargo build -p mingli-wasm --release --target wasm32-unknown-unknown "$@" --message-format=json 2>/dev/null \
      | python3 -c "
import sys, json, os
p = None
for line in sys.stdin:
    try: m = json.loads(line)
    except Exception: continue
    if m.get('reason') == 'compiler-artifact' and m.get('target', {}).get('name') == 'mingli_wasm':
        for f in m.get('filenames', []):
            if f.endswith('.wasm'):
                p = f
print(os.path.getsize(p) if p and os.path.exists(p) else 0)"
  }
  # 行名（README 两份都认这一列）· 期望 MB · feature 组合
  while IFS='|' read -r label want flags; do
    [ -n "$label" ] || continue
    # shellcheck disable=SC2086
    got=$(wasm_size $flags)
    if [ "$got" -eq 0 ]; then
      printf '  ✗ %s：cargo 没报告 .wasm 产物\n' "$label"; fail=1; continue
    fi
    if ! python3 - "$label" "$want" "$got" <<'PYEOF'
import sys
label, want, got = sys.argv[1], float(sys.argv[2]), int(sys.argv[3])
mb = got / 1048576
if abs(mb - want) > 0.03:
    print(f"  ✗ {label}：README 写 {want:.2f} MB，实测 {mb:.2f} MB（{got} 字节）")
    sys.exit(1)
print(f"  ✓ {label} {mb:.2f} MB")
PYEOF
    then fail=1; fi
  done <<'ROWS'
只骨架|0.53|--no-default-features
只四柱|0.57|--no-default-features --features bazi
四柱+紫微|0.60|--no-default-features --features bazi,ziwei
只西洋占星|1.32|--no-default-features --features astrology
全部二十四片|1.83|
ROWS
else
  printf '  没装 wasm32-unknown-unknown，跳过\n'
fi

if [ "$fail" -ne 0 ]; then
  printf '\nfeature 矩阵有组合未通过。\n'
  exit 1
fi
printf '\n全绿。\n'
