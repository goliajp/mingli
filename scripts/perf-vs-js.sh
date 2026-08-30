#!/usr/bin/env bash
# 浏览器形态的性能闸：跟当地最强的 JS 实现比，同机同输入。
#
#   ./scripts/perf-vs-js.sh              # 对拍 + 计时，四柱慢于对手即红
#   ./scripts/perf-vs-js.sh --record     # 重录星历漂移表
#
# 只在浏览器这个主场比。"我们比 Python 快 20 倍" 不能替代
# "我们比浏览器里最快的 JS 实现快多少"——每个发布形态有它自己的记分牌。
set -euo pipefail
cd "$(dirname "$0")/.."
[ -d dist/npm/mingli-wasm-bazi ] || { echo "先跑 ./scripts/npm-pack.sh 打出档位" >&2; exit 1; }
command -v node >/dev/null || { echo "缺 node" >&2; exit 1; }
[ -d bench/browser/node_modules ] || (cd bench/browser && npm i --silent)
exec node bench/browser/compare.mjs "${2:-2000}" ${1:-}
