#!/usr/bin/env bash
# 各发布档位的产物体积，与预算表逐档比对。
#
#   ./scripts/wasm-size.sh                    # 全部档位，超预算即退出码 1
#   ./scripts/wasm-size.sh chart-solo-yijing  # 只查这一档（探针与本地迭代用）
#   ./scripts/wasm-size.sh --record           # 把当前实测写回预算表（改动是有意的时候才用）
#
# 管线是写死的，四步一步不少：
#   cargo build --release --target wasm32-unknown-unknown
#     → wasm-bindgen --target web        （剥掉未导出的东西，加胶水）
#     → wasm-opt -Oz                     （体积优先的再优化）
#     → gzip -9                          （浏览器实际传的量）
# 少走一步数字就变——本项目曾经因为两次量法不同，把 1% 的管线差读成了 15 KB 的回归。
# 预算写死字节数而不是百分比：百分比随总量一起漂，退步看不出来。
#
# 两列不同待遇，因为它们的性质不同：
#   -Oz 那一列是产物本身，同样的输入给同样的字节，逐字节钉死；
#   gzip 那一列是传输量的代理，压缩率随字节排布浮动——实测一次 -Oz 不增反减
#   的改动让 gzip 涨了 63 字节（728,034 → 728,097，0.008%）。对它逐字节设闸，
#   闸就会被与体积无关的改动一直触发，然后人开始习惯性重录预算，闸就废了。
#   所以 gzip 给 0.5% 容差：真正的体积回归远大于此，噪声远小于此。
set -euo pipefail
cd "$(dirname "$0")/.."

BUDGET=scripts/wasm-budget.txt
LEAVES="bazi,ziwei,astrology,jyotish,qizhengsiyu,yijing,geomancy,sikidy,ifa,cartomancy,meihua,xiaoliuren,zeri,maya,pawukon,mahabote,liuren,qimen,taiyi,tibetan,numerology,gematria,abjad,wuge"

# 档位名|feature 串。排盘档只出 cast / cast_one，全档另有十四个跨叶用例出口。
PROFILES="
chart-solo-bazi|--no-default-features --features bazi
chart-solo-yijing|--no-default-features --features yijing
chart-chinese|--no-default-features --features bazi,ziwei,qimen,liuren,meihua,yijing,xiaoliuren,zeri,taiyi,tibetan
chart-all|--no-default-features --features $LEAVES
full|--no-default-features --features usecases,$LEAVES
"

for tool in wasm-bindgen wasm-opt; do
  command -v $tool >/dev/null || { echo "缺 $tool" >&2; exit 1; }
done

work=$(mktemp -d); trap 'rm -rf "$work"' EXIT
arg=${1:-}
record=""; only=""
case "$arg" in --record) record=--record;; "") ;; *) only=$arg;; esac
: > "$work/measured"

measure() {
  local name=$1 flags=$2
  # shellcheck disable=SC2086
  cargo build -q --release --target wasm32-unknown-unknown -p mingli-wasm $flags
  wasm-bindgen --target web --out-dir "$work/$name" \
    target/wasm32-unknown-unknown/release/mingli_wasm.wasm >/dev/null 2>&1
  wasm-opt -Oz -o "$work/$name/o.wasm" "$work/$name/mingli_wasm_bg.wasm"
  local raw gz
  raw=$(wc -c < "$work/$name/o.wasm" | tr -d ' ')
  gz=$(gzip -9 -c "$work/$name/o.wasm" | wc -c | tr -d ' ')
  printf '%s %s %s\n' "$name" "$raw" "$gz" >> "$work/measured"
}

echo "构建各档位（每档一次全量 release 构建，慢）"
while IFS='|' read -r name flags; do
  [ -n "$name" ] || continue
  [ -z "$only" ] || [ "$only" = "$name" ] || continue
  printf '  %-20s ' "$name"
  measure "$name" "$flags"
  tail -1 "$work/measured" | awk '{printf "%9s  gzip %8s\n", $2, $3}'
done <<< "$PROFILES"

if [ -n "$only" ] && [ "$record" = "--record" ]; then
  echo "--record 只能对全部档位用，否则表会被写残" >&2; exit 1
fi
if [ "$record" = "--record" ]; then
  {
    echo "# 各发布档位的体积预算（字节）。由 ./scripts/wasm-size.sh --record 写入。"
    echo "# 列：档位 -Oz后 gzip后。改这张表必须是有意的，且要写进 commit message。"
    cat "$work/measured"
  } > "$BUDGET"
  echo; echo "已写入 $BUDGET"; exit 0
fi

[ -f "$BUDGET" ] || { echo "没有预算表 $BUDGET——先跑一次 --record" >&2; exit 1; }

echo; over=0; seen=0
while read -r name raw gz; do
  want_raw=$(awk -v n="$name" '$1==n{print $2}' "$BUDGET")
  want_gz=$(awk -v n="$name" '$1==n{print $3}' "$BUDGET")
  if [ -z "$want_raw" ]; then
    printf '  ✗ %-20s 预算表里没有这一档\n' "$name"; over=$((over+1)); continue
  fi
  seen=$((seen+1))
  gz_ceil=$(( want_gz + want_gz / 200 ))   # +0.5%
  if [ "$raw" -gt "$want_raw" ]; then
    printf '  ✗ %-20s %9s / %8s  产物超预算 %s\n' "$name" "$raw" "$gz" "$want_raw"
    over=$((over+1))
  elif [ "$gz" -gt "$gz_ceil" ]; then
    printf '  ✗ %-20s %9s / %8s  gzip 超预算 %s（上限 %s）\n' "$name" "$raw" "$gz" "$want_gz" "$gz_ceil"
    over=$((over+1))
  else
    printf '  ✓ %-20s %9s / %8s  预算 %s / %s\n' "$name" "$raw" "$gz" "$want_raw" "$want_gz"
  fi
done < "$work/measured"

if [ -z "$only" ]; then
  n_budget=$(grep -cvE '^#|^$' "$BUDGET")
  [ "$seen" -eq "$n_budget" ] || { echo; echo "预算表有 $n_budget 档，只量到 $seen 档——两边对不上"; exit 1; }
fi
[ "$seen" -gt 0 ] || { echo "没量到任何一档（档位名写错了？）" >&2; exit 1; }
echo
[ "$over" -eq 0 ] || { echo "$over 档超预算"; exit 1; }
echo "$seen 档都在预算内"
