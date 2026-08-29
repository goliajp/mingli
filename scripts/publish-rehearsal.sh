#!/usr/bin/env bash
# 发版彩排：把能在发版**之前**查的都查掉，并给出拓扑发布次序。
#
#   ./scripts/publish-rehearsal.sh
#
# 查得到的：清单元数据齐备、keywords / categories 合规、内部依赖写了版本且与
# workspace 版本一致、打包文件表里没有不该出的东西、拓扑序无环。
#
# 查不到的（且必须说清楚，不许假装查了）：**从 registry 装得起来**。
# `cargo publish --dry-run` 会拿内部依赖去 crates.io 上找，首次发版前一个都找不到，
# 连 `--no-verify` 也过不去——它在解析阶段就停了。这一条只能靠自底向上真发一遍来验，
# 所以本脚本的绿不等于「发得出去」，只等于「该准备的都准备好了」。
set -euo pipefail
cd "$(dirname "$0")/.."

WS_VER=$(sed -n '/^\[workspace.package\]/,/^\[/p' Cargo.toml | sed -n 's/^version = "\(.*\)"/\1/p')
[ -n "$WS_VER" ] || { echo "读不出 workspace 版本" >&2; exit 1; }

bad=0
say_bad() { printf '  ✗ %s\n' "$1"; bad=$((bad+1)); }

echo "workspace 版本 $WS_VER"
echo
echo "一 · 内部依赖的版本与 workspace 一致"
while read -r line; do
  name=${line%% *}
  ver=$(sed -n "s/^$name = { version = \"\([^\"]*\)\".*/\1/p" Cargo.toml)
  if [ -z "$ver" ]; then
    say_bad "$name 在 [workspace.dependencies] 里没写 version——发版时 crates.io 会拒"
  elif [ "$ver" != "$WS_VER" ]; then
    say_bad "$name 写的是 $ver，workspace 是 $WS_VER——两处必须一致"
  fi
done < <(grep -E '^mingli-[a-z0-9]+ = \{' Cargo.toml)
[ "$bad" -eq 0 ] && echo "  ✓ 都对上了"

echo
echo "二 · 每个 crate 的清单元数据"
CRATES=$(ls -d crates/*/ services/*/ 2>/dev/null | sed 's|/$||')
n_pub=0
for d in $CRATES; do
  m="$d/Cargo.toml"
  grep -q '^publish = false' "$m" && continue
  n_pub=$((n_pub+1))
  name=$(sed -n 's/^name = "\(.*\)"/\1/p' "$m" | head -1)
  for field in description keywords categories; do
    grep -q "^$field" "$m" || say_bad "$name 缺 $field"
  done
  # crates.io：keywords 最多 5 个，每个不超过 20 字符
  kw=$(sed -n 's/^keywords = \[\(.*\)\]/\1/p' "$m")
  if [ -n "$kw" ]; then
    n_kw=$(printf '%s' "$kw" | tr ',' '\n' | grep -c .)
    [ "$n_kw" -le 5 ] || say_bad "$name 有 $n_kw 个 keyword，crates.io 上限是 5"
    printf '%s' "$kw" | tr ',' '\n' | tr -d ' "' | while read -r k; do
      [ ${#k} -le 20 ] || echo "  ✗ $name 的 keyword \`$k\` 超过 20 字符"
    done
  fi
done
echo "  待发布 $n_pub 个 crate"

echo
echo "三 · 打包文件表里不许有不该出的东西"
for d in $CRATES; do
  m="$d/Cargo.toml"
  grep -q '^publish = false' "$m" && continue
  name=$(sed -n 's/^name = "\(.*\)"/\1/p' "$m" | head -1)
  files=$(cargo package --list --allow-dirty -p "$name" 2>/dev/null || true)
  if [ -z "$files" ]; then say_bad "$name 打不出包（cargo package --list 无输出）"; continue; fi
  stray=$(printf '%s\n' "$files" | grep -E '^(docs/|\.claude/|\.dev/)' | tr '\n' ' ' || true)
  [ -z "$stray" ] || say_bad "$name 的包里混进了：$stray"
done
[ "$bad" -eq 0 ] && echo "  ✓ 都干净"

echo
echo "四 · 拓扑发布次序"
order=$(python3 - <<'PY'
import pathlib, re, sys
crates = {}
for m in list(pathlib.Path('crates').glob('*/Cargo.toml')) + list(pathlib.Path('services').glob('*/Cargo.toml')):
    t = m.read_text()
    if re.search(r'^publish = false', t, re.M): continue
    name = re.search(r'^name = "(.*)"', t, re.M).group(1)
    deps = set(re.findall(r'^(mingli-[a-z0-9]+) = ', t, re.M))
    crates[name] = deps
out, seen, mark = [], set(), {}
def visit(n, stack):
    if n in seen: return
    if mark.get(n) == 1:
        print("CYCLE " + " → ".join(stack + [n])); sys.exit(1)
    mark[n] = 1
    for d in sorted(crates.get(n, ())):
        if d in crates: visit(d, stack + [n])
    mark[n] = 2; seen.add(n); out.append(n)
for n in sorted(crates): visit(n, [])
print(" ".join(out))
PY
)
case "$order" in
  CYCLE*) say_bad "依赖有环：${order#CYCLE }";;
  *) i=0; for c in $order; do i=$((i+1)); printf '  %2d. %s\n' "$i" "$c"; done;;
esac

echo
if [ "$bad" -gt 0 ]; then echo "$bad 处要先修，才谈得上发版"; exit 1; fi
echo "该准备的都准备好了。注意：这不等于「从 registry 装得起来」——那一条要自底向上真发一遍才验得到。"
