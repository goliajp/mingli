#!/usr/bin/env bash
# 同一份输入，debug 与 release 各排一遍全树，指纹必须一模一样。
#
# 为什么要有这条：整套测试只在 debug 下跑过，而 wasm 那扇门是 release 编的
# （`[profile.release]` 还带着 lto=thin、codegen-units=1）。优化档改变浮点收缩
# 或求值次序，节气时刻差一个 ulp 就可能把某一天推到边界另一侧——而两扇门一致性的
# 那套测试是 debug 下比的，看不见这件事。
#
# 判在这里，量在 `crates/mingli-registry/examples/tree_fingerprint.rs`。
set -euo pipefail
cd "$(dirname "$0")/.."

printf '同一棵树，两种优化档\n\n'

dbg=$(cargo run -q --example tree_fingerprint -p mingli-registry)
rel=$(cargo run -q --release --example tree_fingerprint -p mingli-registry)

printf '  debug    %s\n' "$dbg"
printf '  release  %s\n' "$rel"
printf '\n'

if [ "$dbg" = "$rel" ]; then
  printf '两档逐字节相同\n'
  exit 0
fi

printf '两档算出的盘不一样——优化改变了结果。\n'
printf 'wasm 是 release 编的，这意味着两扇门此刻给的是不同的盘。\n'
exit 1
