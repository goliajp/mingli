#!/usr/bin/env bash
# 只开一片叶时，产物里不许出现别的叶。
#
#   ./scripts/leaf-isolation.sh          # 全部二十四片各查一遍
#   ./scripts/leaf-isolation.sh bazi     # 只查这一片
#
# 查的是 wasm32 目标下的依赖图而不是符号表：release + wasm-opt 之后名字段已被剥掉，
# 从产物里认叶只能靠体积推断，而依赖图是精确的，且不必构建。
#
# 一片叶被拉进来只有两条路——注册表按 feature 接上，或者别的 crate 无条件依赖它。
# 后者不会有任何报错，只会让产物默默变大，这条闸就是为它立的。
set -euo pipefail
cd "$(dirname "$0")/.."

# 叶名取自注册表的 [features]，而不是另抄一份：抄漏一片就少查一片，且不会有人发现。
# 叶名取自装配根 `full` 那张表——它就是「我全都要」的定义，且已被
# crates/mingli/tests/facade.rs 守着。不要去认 `= ["dep:mingli-` 那种形状：
# 一片叶的 feature 值一旦多写一项（`astrology` 就是），那种认法立刻少认一片。
LEAVES=$(sed -n '/^full = \[/,/^\]/p' crates/mingli-registry/Cargo.toml |
         grep -oE '"[a-z][a-z0-9_]*"' | tr -d '"' | sort -u)
n_leaves=$(printf '%s\n' "$LEAVES" | grep -c .)
if [ "$n_leaves" -lt 20 ]; then
  echo "只解析出 $n_leaves 片叶，解析方式怕是失效了" >&2; exit 1
fi

# feature 名 → crate 名。绝大多数同名，塔罗那片的 id 与 crate 名不一致。
crate_of() { case "$1" in cartomancy) echo mingli-cartomancy;; *) echo "mingli-$1";; esac; }

# 这些不是叶：共享层与装配层，出现在任何构建里都正常。
INFRA=" mingli-core mingli-astro mingli-contract mingli-engine mingli-registry mingli-wasm
        mingli-ganzhi mingli-gua mingli-luoshu mingli-ephemeris mingli-analysis
        mingli-interpret mingli-app "

want="${1:-}"
bad=0; checked=0
for leaf in $LEAVES; do
  [ -z "$want" ] || [ "$want" = "$leaf" ] || continue
  keep=$(crate_of "$leaf")
  tree=$(cargo tree -p mingli-wasm --target wasm32-unknown-unknown -e normal --prefix none \
           --no-default-features --features "$leaf" 2>/dev/null | awk '{print $1}' | sort -u)
  [ -n "$tree" ] || { echo "  ✗ ${leaf}：cargo tree 无输出" >&2; bad=$((bad+1)); continue; }
  intruders=""
  for other in $LEAVES; do
    oc=$(crate_of "$other")
    [ "$oc" = "$keep" ] && continue
    case "$INFRA" in *" $oc "*) continue;; esac
    grep -qx "$oc" <<<"$tree" && intruders="$intruders $oc"
  done
  checked=$((checked+1))
  if [ -n "$intruders" ]; then
    printf '  ✗ %-14s 混进了：%s\n' "$leaf" "${intruders# }"
    bad=$((bad+1))
  else
    printf '  ✓ %-14s 干净\n' "$leaf"
  fi
done

[ "$checked" -gt 0 ] || { echo "一片也没查到（叶名写错了？）" >&2; exit 1; }
echo
if [ "$bad" -gt 0 ]; then
  echo "$checked 片里 $bad 片带进了别的叶——只要其中一片的使用者，正在为另外几片付体积"
  exit 1
fi
echo "$checked 片各自干净"
