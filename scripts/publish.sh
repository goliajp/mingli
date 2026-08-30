#!/usr/bin/env bash
# 按拓扑序把全部 crate 发到 crates.io。可中断、可续跑。
#
#   ./scripts/publish.sh --dry     # 只打印次序与每个的状态，不发
#   ./scripts/publish.sh --go      # 真发（不可撤回）
#
# **发布期间不许改源码树。** 这条不是建议：
#   - cargo publish 读工作树，树一脏就中止；
#   - 更隐蔽的是，已经发出去的那些版本是冻住的。给某个 crate 加一个 feature，
#     然后让还没发的下游去要那个 feature，下游就发不出去了——上游在索引上的
#     那一版没有它。本项目正是这么卡住的：`mingli-ephemeris 1.0.0` 发出去之后
#     才加了 `vsop87`，于是 `mingli-qizhengsiyu` 在解析阶段失败。
#     补救只有一条：整体升一个 patch 版本重发。
#
# 三件事必须在这里处理，否则一次跑不完：
#   1. crates.io 对**新** crate 限速（约 5 个突发之后每 10 分钟 1 个），所以遇 429 要等；
#   2. 下游要等上游在索引上可见才发得动，所以每发一个要轮询到它出现；
#   3. 已发过的要跳过，好让脚本能从中断处接着跑。
set -uo pipefail
cd "$(dirname "$0")/.."

VER=$(sed -n '/^\[workspace.package\]/,/^\[/p' Cargo.toml | sed -n 's/^version = "\(.*\)"/\1/p')
UA="mingli-release (takagi@golia.jp)"
mode=${1:---dry}

ORDER=$(./scripts/publish-rehearsal.sh 2>/dev/null | sed -n 's/^ *[0-9]\{1,2\}\. //p')
n=$(printf '%s\n' "$ORDER" | grep -c .)
[ "$n" -ge 35 ] || { echo "拓扑序只取到 $n 个，彩排是不是没过？" >&2; exit 1; }
echo "$n 个 crate，版本 $VER"

# 索引上有没有这个版本
on_index() {
  curl -sf -A "$UA" "https://crates.io/api/v1/crates/$1/$2" --max-time 20 >/dev/null 2>&1
}

wait_for_index() {
  local crate=$1 i=0
  while [ $i -lt 60 ]; do
    on_index "$crate" "$VER" && return 0
    sleep 10; i=$((i+1))
  done
  echo "  等了 10 分钟，$crate $VER 仍未出现在索引上" >&2; return 1
}

done_n=0; skip_n=0
for c in $ORDER; do
  if on_index "$c" "$VER"; then
    printf '  ⏭  %-22s 已在索引上\n' "$c"; skip_n=$((skip_n+1)); continue
  fi
  if [ "$mode" != "--go" ]; then
    printf '  ·  %-22s 待发\n' "$c"; continue
  fi
  # 限速时重试。每次等 11 分钟——比 10 分钟的窗口多一分钟，免得卡在边界上反复撞。
  attempt=0
  while :; do
    printf '  →  %-22s ' "$c"
    # 不加 --no-verify：按拓扑序发到这里时，它的依赖已经在索引上了，
    # 于是 cargo 会拿索引上的依赖真编一遍。彩排验不到的那一条
    # 「从 registry 装得起来」，正是在这里被验掉的。
    out=$(cargo publish -p "$c" 2>&1)
    rc=$?
    if [ $rc -eq 0 ]; then echo "✓"; break; fi
    if printf '%s' "$out" | grep -qi "already exists\|already uploaded"; then echo "已存在"; break; fi
    if printf '%s' "$out" | grep -qiE "429|too many requests|rate limit"; then
      attempt=$((attempt+1))
      if [ $attempt -gt 8 ]; then echo "✗ 限速重试八次仍不过"; exit 1; fi
      echo "限速，等 11 分钟（第 $attempt 次）"; sleep 660; continue
    fi
    echo "✗"; printf '%s\n' "$out" | tail -12; exit 1
  done
  wait_for_index "$c" || exit 1
  done_n=$((done_n+1))
done

echo
if [ "$mode" = "--go" ]; then
  echo "本次发出 $done_n 个，跳过 $skip_n 个已在索引上的"
else
  echo "以上是次序。要真发加 --go（不可撤回）"
fi
