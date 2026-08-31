#!/usr/bin/env bash
# 变异扫描：把源码改坏一处，看测试红不红。红了叫「拦住」，绿了叫「漏网」。
#
# 跟 `guard-probe.sh` 是两件事。那支脚本种的是**挑好的**错，问的是「这条守卫真在守吗」；
# 这支是穷举：把一个 crate 里每个能改的地方都改一遍，问「有没有哪处改了没人管」。
# 前者查已知的守卫，后者找没人守的地方。
#
#   ./scripts/mutants.sh mingli-astro          # 扫整个 crate
#   ./scripts/mutants.sh mingli-astro lib.rs   # 只扫一个文件
#
# 一轮以小时计，所以它不进 CI，是手上工具。
#
# ---- 三个踩过的坑，写在这里省得下次再踩 ----
#
# 一、`-p` 只跑那个包自己的测试。
#    漏网名单是「本包测试没拦住」，不是「整个仓库没拦住」。工作区里别处的测试
#    可能正拦着它。所以名单要按 `cargo test --workspace` 复核一遍再动手。
#
# 二、超时不是拦住。
#    改坏之后测试挂死，扫描记的是 timeout，跟「拦住」分开列。它既没被拦也不算漏网，
#    实际是最糟的一种：挂死不给任何可读的东西。见到 timeout 先去找那条没有上限的循环。
#
# 三、机器忙的时候，超时可能是假的。
#    cargo-mutants 的超时阈值由基线跑一遍推出来。别的活把机器压住时，一个其实
#    「当场就被拦住」的变异也会因为编译被拖慢而记成超时。实测过一次：某个
#    `julian_day` 的变异记成 148 秒超时，单独复跑同一个变异，9 条测试在 0.03 秒里全红。
#    所以见到超时先单独复跑那一个，别急着去找不存在的死循环。
#
# 四、扫描期间别改源码。
#    cargo-mutants 自己拷一份树去改，但共用 target/。同时编译会互相拖到看不出快慢，
#    更要紧的是：中途被强杀时，改坏的那一处可能留在工作区里。留下的错会被下一轮
#    当成原始内容——本仓库真发生过，一个 `* 365.25` 就这样活过四轮，每轮都汇报「已还原」。
#    收尾时这里会验一次工作区是否干净，就是为这件事。

set -euo pipefail
cd "$(dirname "$0")/.."

pkg=${1:-}
[ -n "$pkg" ] || { echo "用法：./scripts/mutants.sh <crate> [文件]" >&2; exit 1; }
file=${2:-}

command -v cargo-mutants >/dev/null 2>&1 || {
  echo "没装 cargo-mutants：cargo install cargo-mutants" >&2; exit 1
}

dirty=$(git status --porcelain | grep -v '^??' || true)
[ -z "$dirty" ] || {
  printf '工作区对 HEAD 不干净，先处理：\n%s\n' "$dirty" >&2
  printf '扫描要拿当前源码当基准，带着未提交的改动扫，出来的名单说不清是谁的。\n' >&2
  exit 1
}

out=$(mktemp -d)
args=(mutants -p "$pkg" --output "$out")
[ -z "$file" ] || args+=(-f "$file")

echo "扫 $pkg${file:+ / $file}，一轮以小时计……"
cargo "${args[@]}" || true

missed="$out/mutants.out/missed.txt"
timeout="$out/mutants.out/timeout.txt"
n_missed=$(wc -l < "$missed" 2>/dev/null | tr -d ' ' || echo 0)
n_timeout=$(wc -l < "$timeout" 2>/dev/null | tr -d ' ' || echo 0)

printf '\n漏网 %s 条，超时 %s 条。完整结果在 %s\n' "$n_missed" "$n_timeout" "$out/mutants.out"
[ "$n_missed" -eq 0 ] || { echo; echo '漏网（记得按 workspace 复核，见坑一）：'; sed 's/^/  /' "$missed"; }
[ "$n_timeout" -eq 0 ] || { echo; echo '超时（多半是某条循环没有上限，见坑二）：'; sed 's/^/  /' "$timeout"; }

after=$(git status --porcelain | grep -v '^??' || true)
[ "$dirty" = "$after" ] || {
  printf '\n扫描结束后工作区变脏了——改坏的地方可能留在里面，先 git diff 看一眼。\n' >&2
  exit 1
}
