#!/usr/bin/env bash
# 只想直接调一片叶的类型化 API 时，依赖链上不许出现 serde。
#
#   ./scripts/leaf-deps.sh          # 全部叶
#   ./scripts/leaf-deps.sh bazi     # 只查这一片
#
# 每片叶都有 `compute(...) -> Chart` 这样的类型化出口，JSON 只是它接进注册表时
# 才需要的那一层。把 `CastingEngine` 实现放在 `port` feature 之后，
# `--no-default-features` 的使用者就不必为一层他不经过的边界付编译时间——
# 实测四柱那一片的依赖棵数因此从 21 降到 4，去掉的里面有 serde_derive 与 syn 两个 proc-macro。
#
# 查的是依赖图而不是产物：native 链接期会把没调到的 serde 代码丢掉，
# 于是「产物里没有」看起来永远成立，而编译时间照付。
set -euo pipefail
cd "$(dirname "$0")/.."

# 叶名取自装配根的 [features]，不另抄一份。
LEAVES=$(sed -n '/^\[features\]/,$p' crates/mingli-registry/Cargo.toml |
         grep -E '^[a-z][a-z0-9_]* = \["dep:mingli-' | sed -E 's/.*\["dep:(mingli-[a-z0-9]+)".*/\1/' | sort -u)
n_leaves=$(printf '%s\n' "$LEAVES" | grep -c .)
[ "$n_leaves" -ge 20 ] || { echo "只解析出 $n_leaves 片叶，解析方式怕是失效了" >&2; exit 1; }

want="${1:-}"; bad=0; checked=0
for crate in $LEAVES; do
  [ -z "$want" ] || [ "mingli-$want" = "$crate" ] || continue
  # `|| true`：cargo tree 失败时不许把整个脚本静默带走——那样「一片都没查」
  # 长得跟「全都干净」一模一样。这条注释是因为真踩过。
  tree=$(cargo tree -p "$crate" --no-default-features -e normal --prefix none 2>/dev/null |
         awk '{print $1}' | sort -u || true)
  [ -n "$tree" ] || { printf '  ✗ %-20s cargo tree 无输出（这一档编不过？）\n' "$crate"; bad=$((bad+1)); continue; }
  # 同上：grep 没匹配到就返回 1，而「没匹配到」正是本脚本要的那个结果。
  dirty=$(printf '%s\n' "$tree" | grep -E '^(serde|serde_core|serde_json|serde_derive|syn|quote|proc-macro2)$' | tr '\n' ' ' || true)
  # 依赖图干净还不够：cargo tree 只解析不编译，一片「解析得出、编不过」的叶
  # 在只看依赖图的闸下与真正干净的叶长得一模一样。
  if ! cargo build -q -p "$crate" --no-default-features 2>/dev/null; then
    printf '  ✗ %-20s 依赖图干净，但这一档编不过\n' "$crate"; bad=$((bad+1)); continue
  fi
  checked=$((checked+1))
  n=$(printf '%s\n' "$tree" | grep -c .)
  if [ -n "$dirty" ]; then
    printf '  ✗ %-20s %2s 棵，含：%s\n' "$crate" "$n" "$dirty"; bad=$((bad+1))
  else
    printf '  ✓ %-20s %2s 棵，无 serde\n' "$crate" "$n"
  fi
done

[ "$checked" -gt 0 ] || { echo "一片也没查到（叶名写错了？）" >&2; exit 1; }
echo
[ "$bad" -eq 0 ] || { echo "$checked 片里 $bad 片仍拖着 serde"; exit 1; }
echo "$checked 片的类型化出口都不带 serde"
