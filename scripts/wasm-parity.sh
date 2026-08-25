#!/usr/bin/env bash
# native 与 wasm 排出来的是不是同一棵树。
#
# 为什么要单独有这条：`profile-parity.sh` 比的是 native 的两个优化档，同一个目标三元组。
# wasm32-unknown-unknown 是另一个目标——它没有系统 libm，三角函数由 compiler-builtins
# 那套编进模块（实测其导入表只有两项 wasm-bindgen 胶水，无任何宿主数学函数），
# 而 native 链的是系统 libm。两套实现在最后一两位上不同，于是浏览器里算出的盘
# 与服务器上算出的**不逐字节相同**。这条把「差多少、差在哪」钉住，不让它长大。
#
# 判在 scripts/wasm-parity.mjs，量在 examples/tree_fingerprint.rs 与 scripts/wasm-cast.mjs。
set -euo pipefail
cd "$(dirname "$0")/.."

for tool in node wasm-bindgen; do
  command -v "$tool" >/dev/null 2>&1 || { printf '缺 %s，这条跑不了\n' "$tool"; exit 127; }
done

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

printf '\n两扇门，同一批时刻\n\n'

cargo run -q --example tree_fingerprint -p mingli-registry -- "$work/native.json" \
  | sed 's/^/  native  /'

cargo build -q -p mingli-wasm --release --target wasm32-unknown-unknown
wasm-bindgen --target nodejs --out-dir "$work/wasm" \
  target/wasm32-unknown-unknown/release/mingli_wasm.wasm
node scripts/wasm-cast.mjs "$work/wasm/mingli_wasm.js" "$work/wasm.json"

printf '\n'
node scripts/wasm-parity.mjs "$work/native.json" "$work/wasm.json"
