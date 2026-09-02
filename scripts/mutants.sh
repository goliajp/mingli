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
#
#    但整仓跑一遍要十几分钟，全程用它扫一个 crate 是几小时。所以分两趟：
#    第一趟只跑本包（几分钟）。这一趟若一条漏网都没有，结论已经比整仓那趟**更强**
#    ——「本包自己就拦住了全部」蕴含「整仓拦住了全部」，不必再跑。
#    只有出现漏网时才有第二趟：对出现漏网的那几个文件重扫一遍。
#
#    第二趟也不跑整仓。能拦住某个包里变异的测试，只可能在**它自己或依赖它的包**里
#    ——别的包根本编不到那段代码。`dependents.py` 算出这一组（astrology 是 8 个而非 39 个）。
#    这不是省事，是必要：第二趟的代价恰恰压在漏网身上（漏网 = 整组测试跑满才判定），
#    一次实测里 54 条漏网 × 990 秒的整仓套件要跑十五小时。
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
#    但先确认它是真的。cargo-mutants 的超时阈值由基线跑一遍推出来，而**基线只跑本包**
#    （实测 0.62 秒），加上 `--test-workspace` 之后每个变异跑的却是整仓套件（实测 349 秒）。
#    两个尺度差五百倍，于是凡是没被早早拦住的变异一律撞墙：一轮扫出十七个「超时」，
#    逐个复跑全是**被拦住的**，只是让它红的那个测试二进制排在后面。
#    所以下面先自己量一遍整仓套件，按实测的三倍给 `--timeout`。
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

# 先确认树是绿的——对着一棵红树扫，出来的名单没有意义。顺带量整仓套件要多久，
# 第二趟的超时阈值按它定（见坑二：cargo-mutants 自己推的那个阈值量错了尺度）。
# 第二趟要跑的那一组：本包，加上传递依赖它的包。别的包编不到被改的代码。
group=()
while IFS= read -r g; do [ -n "$g" ] && group+=("$g"); done < <(python3 scripts/dependents.py "$pkg")
[ "${#group[@]}" -ge 1 ] || { echo "算不出 $pkg 的依赖方" >&2; exit 1; }

pkg_args=()
for g in "${group[@]}"; do pkg_args+=(-p "$g"); done

echo "先量一遍这一组的测试（${#group[@]} 个包，顺带确认它们是绿的）……"
t0=$(date +%s)
cargo test "${pkg_args[@]}" >/dev/null 2>&1 || {
  echo "这一组测试没全绿，扫描出来的名单不作数" >&2; exit 1
}
grp_secs=$(( $(date +%s) - t0 ))
limit=$(( grp_secs * 3 ))
[ "$limit" -ge 60 ] || limit=60
echo "这一组 ${grp_secs}s；第二趟若要跑，超时阈值取 ${limit}s"

# 第一趟的阈值也要按实测给。
#
# cargo-mutants 自己推的那个阈值在机器忙的时候一次次把「慢」记成「挂死」：
# 实测过一轮报四个超时，逐个复跑全是被拦住的，只是花了 43–85 秒。
t0=$(date +%s)
cargo test -p "$pkg" >/dev/null 2>&1 || {
  echo "$pkg 自己的测试没全绿，扫描出来的名单不作数" >&2; exit 1
}
pkg_secs=$(( $(date +%s) - t0 ))
limit1=$(( pkg_secs * 3 ))
[ "$limit1" -ge 60 ] || limit1=60

out=$(mktemp -d)
args=(mutants -p "$pkg" --timeout "$limit1" --output "$out")
[ -z "$file" ] || args+=(-f "$file")
[ -z "$features" ] || args+=(--features "$features")

echo "第一趟：扫 $pkg${file:+ / $file}${features:+（档位 ${features}）}，只跑本包的测试（阈值 ${limit1}s）……"
cargo "${args[@]}" || true

# 拿已知清单对账，只留没见过的。
#
# 判断一条漏网是不是真缺口，功夫常常比扫一轮还大——证明 `+180` 与 `-180` 模 360 恒等、
# 确认某条分支经调用方走不到，都要单独推一遍。`mutants-known.txt` 把这些结论存下来。
# 它不隐藏什么：每轮照样全扫，清单只把已知的挪到一边；同时反查清单里已经不再出现的条目，
# 免得它变成一份越攒越旧的免罪符。
reconcile() {
  local pkg=$1 file=$2
  python3 - "$pkg" "$file" <<'PY'
import collections, pathlib, sys
pkg, missed_path = sys.argv[1], sys.argv[2]

def norm(line):
    # crates/x/src/y.rs:12:34: replace ... → ("y.rs", "replace ...")
    body = line.split("/src/", 1)[1] if "/src/" in line else line
    f, rest = body.split(":", 1)
    return f, rest.split(":", 2)[2].strip()

seen = collections.Counter()
for line in pathlib.Path(missed_path).read_text().splitlines():
    if line.strip():
        seen[norm(line)] += 1

known = collections.Counter()
reasons = {}
for line in pathlib.Path("scripts/mutants-known.txt").read_text().splitlines():
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    head, count, why = line.split("|", 2)
    kpkg, kfile, desc = head.split(" ", 2)
    if kpkg != pkg:
        continue
    known[(kfile, desc)] += int(count)
    reasons[(kfile, desc)] = why

new = {k: n - known.get(k, 0) for k, n in seen.items() if n > known.get(k, 0)}
gone = {k: known[k] - seen.get(k, 0) for k in known if known[k] > seen.get(k, 0)}

if new:
    print(f"
没见过的漏网 {sum(new.values())} 条：")
    for (f, d), n in sorted(new.items()):
        print(f"  {f}  {d}" + (f"  ×{n}" if n > 1 else ""))
if gone:
    print(f"
已知清单里这些条目这轮没再出现，去 scripts/mutants-known.txt 删掉：")
    for (f, d), n in sorted(gone.items()):
        print(f"  {f}  {d}" + (f"  ×{n}" if n > 1 else ""))
if not new and not gone:
    print("
漏网全部在已知清单里，且清单没有过期条目。")
sys.exit(1 if new or gone else 0)
PY
}

missed="$out/mutants.out/missed.txt"
timeout="$out/mutants.out/timeout.txt"
n_missed=$(wc -l < "$missed" 2>/dev/null | tr -d ' ' || echo 0)

# 第一趟有漏网，才需要第二趟：那些漏网可能正被别的包的测试拦着。
# 只重扫出现漏网的那几个文件，别的文件第一趟已经给出更强的结论。
if [ "$n_missed" -gt 0 ]; then
  files=$(sed 's/:.*//' "$missed" | sed 's|.*/src/|src/|' | sort -u)
  echo
  echo "第一趟漏网 ${n_missed} 条，落在这几个文件里；用这 ${#group[@]} 个包的测试复核："
  printf '%s\n' "$files" | sed 's/^/  /'
  out2=$(mktemp -d)
  args2=(mutants -p "$pkg" --timeout "$limit" --output "$out2")
  for g in "${group[@]}"; do args2+=(--test-package "$g"); done
  [ -z "$features" ] || args2+=(--features "$features")
  while IFS= read -r f; do [ -n "$f" ] && args2+=(-f "${f##*/}"); done <<< "$files"
  cargo "${args2[@]}" || true
  out=$out2
  missed="$out/mutants.out/missed.txt"
  timeout="$out/mutants.out/timeout.txt"
  n_missed=$(wc -l < "$missed" 2>/dev/null | tr -d ' ' || echo 0)
fi
n_timeout=$(wc -l < "$timeout" 2>/dev/null | tr -d ' ' || echo 0)

printf '\n漏网 %s 条，超时 %s 条。完整结果在 %s\n' "$n_missed" "$n_timeout" "$out/mutants.out"
[ "$n_missed" -eq 0 ] || { echo; echo '漏网：'; sed 's/^/  /' "$missed"; }
reconcile "$pkg" "$missed" || true
[ "$n_timeout" -eq 0 ] || { echo; echo '超时（多半是某条循环没有上限，见坑二）：'; sed 's/^/  /' "$timeout"; }

after=$(git status --porcelain | grep -v '^??' || true)
[ "$dirty" = "$after" ] || {
  printf '\n扫描结束后工作区变脏了——改坏的地方可能留在里面，先 git diff 看一眼。\n' >&2
  exit 1
}
