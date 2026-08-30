#!/usr/bin/env bash
# README 自称的测试数，跟真跑一遍数出来的对不对得上。
#
# 别的门面数字（crate 数 / 叶数 / 问局数 / 端点表）由 `cargo test` 里的
# crates/mingli-registry/tests/readme.rs 守着——那些东西代码自己答得出来。
# 唯独「测试有多少条」只有把套件真跑一遍才知道，测试自己数不了自己，故单开一条。
#
#   ./scripts/test-count.sh            # 对一遍
#   ./scripts/test-count.sh --fix      # 数出来直接写回两份 README

set -euo pipefail

# BSD sed（macOS）要 `-i ''`，GNU sed（CI 上的 Linux）要 `-i` 且不能跟空串——
# 后者会把 '' 当成脚本、把真正的表达式当成文件名。这个脚本两边都要跑，故先认一次。
if sed --version >/dev/null 2>&1; then
  sedi() { sed -i "$@"; }
else
  sedi() { sed -i '' "$@"; }
fi

cd "$(dirname "$0")/.."

out=$(mktemp)
trap 'rm -f "$out"' EXIT

cargo test --workspace >"$out" 2>&1 || { cat "$out"; echo "套件没全绿，数出来的不作数" >&2; exit 1; }

# 数两遍，两种数法：一遍加各二进制的汇总行，一遍数逐条 `... ok`。
#
# 只有一种数法时，「非零但错」是拦不住的——下限只挡得住 0，而 `--fix` 会把错数
# 直接写回 README。两种算法同时错到同一个数上，比一种错难得多。
# 实测（2026-08-28）：两者皆 793，86 个汇总行，0 条 ignored。
actual=$(grep -E '^test result: ok\. [0-9]+ passed' "$out" | awk '{s+=$4} END {print s+0}')
lines=$(grep -cE '^test .* \.\.\. ok$' "$out" || true)
binaries=$(grep -cE '^test result: ok\.' "$out" || true)

[ "$actual" -gt 0 ] || { echo "一条测试都没数到——数法怕是失效了" >&2; exit 1; }
[ "$binaries" -gt 10 ] || {
  echo "只数到 ${binaries} 个测试二进制的汇总行——套件不会这么小，数法怕是失效了" >&2
  exit 1
}
if [ "$actual" != "$lines" ]; then
  # 变量名一律加花括号：`${actual}，` 这种写法会让 bash 把全角逗号的头一个字节
  # 并进变量名，于是 set -u 报 `actual?: unbound variable`——这条错误分支从前正是
  # 这么坏的：它真要开口的那一刻死在一句莫名其妙的 bash 报错上，而正常路径永远不经过它。
  echo "两种数法对不上：汇总行加总 ${actual}，逐条 ok 行 ${lines}" >&2
  echo "先弄清哪一种坏了——在这之前数出来的都不作数，更不该 --fix 写回 README" >&2
  exit 1
fi

fail=0
for f in README.md README.zh-CN.md; do
  claimed=$(grep -oE '[0-9]+ (tests green|个测试全绿)' "$f" | head -1 | grep -oE '^[0-9]+' || true)
  if [ -z "$claimed" ]; then
    echo "$f 里找不到测试数那一句——句式改过了，本脚本要跟着改" >&2
    fail=1
    continue
  fi
  if [ "$claimed" = "$actual" ]; then
    echo "✓ $f 自称 $claimed 个，实测 $actual 个（$binaries 个测试二进制，两种数法一致）"
  elif [ "${1:-}" = "--fix" ]; then
    sedi "s/${claimed} tests green/${actual} tests green/; s/${claimed} 个测试全绿/${actual} 个测试全绿/" "$f"
    sedi "s/# ${claimed} tests/# ${actual} tests/; s/# ${claimed} 个测试/# ${actual} 个测试/" "$f"
    echo "↻ $f $claimed → $actual"
  else
    echo "✗ $f 自称 $claimed 个，实测 $actual 个（--fix 可直接写回）" >&2
    fail=1
  fi
done
exit $fail
