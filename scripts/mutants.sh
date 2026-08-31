#!/usr/bin/env bash
# 变异扫描：把源码改坏一处，看测试红不红。红了叫「拦住」，绿了叫「漏网」。
#
# 跟 `guard-probe.sh` 是两件事。那支脚本种的是**挑好的**错，问的是「这条守卫真在守吗」；
# 这支是穷举：把一个 crate 里每个能改的地方都改一遍，问「有没有哪处改了没人管」。
# 前者查已知的守卫，后者找没人守的地方。
#
#   ./scripts/mutants.sh mingli-astro                       # 扫整个 crate
#   ./scripts/mutants.sh mingli-astro lib.rs                # 只扫一个文件
#   ./scripts/mutants.sh mingli-ephemeris '' eph-lite       # 带上非默认档位
#
# 一轮以小时计，所以它不进 CI，是手上工具。
#
# ---- 三个踩过的坑，写在这里省得下次再踩 ----
#
# 一、只跑本包的测试会虚报漏网。
#    `-p` 挑的是「在哪个包里生成变异」，默认也只跑那个包自己的测试。于是别处测试
#    拦着的东西会被报成漏网——实测：`geocentric_ecliptic_longitudes` 整个函数体被
#    替换成空都「没人拦」，而拦它的测试在 `mingli-astrology` 里。
#    所以这里固定加 `--test-workspace true`：慢，但名单是真的。
#
#    同一件事的另一面：**被 cfg 掉的代码白送漏网**。扫描按当前档位编译，而变异是
#    照着源文件生成的——`#[cfg(feature = "x")]` 里的代码在 x 没开时压根没编进去，
#    改它当然没人红。实测：`mingli-ephemeris` 按默认档扫出 44 条漏网，其中 22 条
#    整块落在 `eph-lite` 里，而那正是 wasm 真正发出去的那条路。第三个参数就是为这个。
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
[ -n "$pkg" ] || { echo "用法：./scripts/mutants.sh <crate> [文件] [档位]" >&2; exit 1; }
file=${2:-}
features=${3:-}

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
args=(mutants -p "$pkg" --test-workspace true --output "$out")
[ -z "$file" ] || args+=(-f "$file")
[ -z "$features" ] || args+=(--features "$features")

echo "扫 $pkg${file:+ / $file}${features:+（档位 ${features}）}，一轮以小时计……"
cargo "${args[@]}" || true

missed="$out/mutants.out/missed.txt"
timeout="$out/mutants.out/timeout.txt"
n_missed=$(wc -l < "$missed" 2>/dev/null | tr -d ' ' || echo 0)
n_timeout=$(wc -l < "$timeout" 2>/dev/null | tr -d ' ' || echo 0)

printf '\n漏网 %s 条，超时 %s 条。完整结果在 %s\n' "$n_missed" "$n_timeout" "$out/mutants.out"
[ "$n_missed" -eq 0 ] || { echo; echo '漏网：'; sed 's/^/  /' "$missed"; }
[ "$n_timeout" -eq 0 ] || { echo; echo '超时（多半是某条循环没有上限，见坑二）：'; sed 's/^/  /' "$timeout"; }

after=$(git status --porcelain | grep -v '^??' || true)
[ "$dirty" = "$after" ] || {
  printf '\n扫描结束后工作区变脏了——改坏的地方可能留在里面，先 git diff 看一眼。\n' >&2
  exit 1
}
