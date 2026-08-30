#!/usr/bin/env bash
# 从公开 registry 装一遍，跑一遍。
#
#   ./scripts/installed-smoke.sh
#
# 这是发版彩排明说自己验不到的那一条。`cargo publish --dry-run` 会拿内部依赖去
# crates.io 上找，首次发版前一个都找不到，所以「从 registry 装得起来」只能发完再验。
# 本脚本在仓库之外建两个临时工程——一个 cargo、一个 npm——只按版本号依赖，
# 不走任何 path，然后跑起来对答案。
#
# 顺带核一件事：npm 包里那份 wasm 的字节数必须等于 scripts/wasm-budget.txt 里的数。
# 发出去的东西与预算表说的不是一回事，比预算表本身错了更难发现。
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT=$PWD

VER=$(sed -n '/^\[workspace.package\]/,/^\[/p' Cargo.toml | sed -n 's/^version = "\(.*\)"/\1/p')
MAJOR=${VER%%.*}
work=$(mktemp -d); trap 'rm -rf "$work"' EXIT
fail=0

echo "从 registry 装 ${VER}（按 \"^$MAJOR\" 解析）"
echo
echo "一 · Rust：mingli-bazi，关掉缺省 feature"
mkdir -p "$work/rs/src"
cat > "$work/rs/Cargo.toml" <<EOF
[package]
name = "smoke"
version = "0.0.0"
edition = "2021"
[dependencies]
mingli-bazi = { version = "$MAJOR", default-features = false }
EOF
cat > "$work/rs/src/main.rs" <<'EOF'
fn main() {
    let c = mingli_bazi::compute(mingli_bazi::BirthInput {
        year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
        gender: Some(mingli_bazi::Gender::Male),
    });
    println!("{} {} {} {}", c.year.ganzhi, c.month.ganzhi, c.day.ganzhi, c.hour.ganzhi);
}
EOF
if got=$(cd "$work/rs" && cargo run -q 2>/dev/null); then
  # 这四柱与 lunar-javascript 对同一时刻给出的完全一致（见 bench/browser/compare.mjs）。
  if [ "$got" = "庚午 壬午 辛亥 乙未" ]; then
    printf '  ✓ 装得起来，且四柱对：%s\n' "$got"
  else
    printf '  ✗ 装起来了但答案不对：%s\n' "$got"; fail=1
  fi
  n=$(cd "$work/rs" && cargo tree -e normal --prefix none 2>/dev/null | awk '{print $1}' | sort -u | grep -c .)
  dirty=$(cd "$work/rs" && cargo tree -e normal --prefix none 2>/dev/null | awk '{print $1}' | grep -cE '^(serde|serde_json|syn|quote)$' || true)
  if [ "$dirty" -ne 0 ]; then printf '  ✗ 关掉缺省之后仍拖进 %s 棵 serde 一族\n' "$dirty"; fail=1
  else printf '  ✓ 依赖 %s 棵，无 serde\n' "$n"; fi
else
  printf '  ✗ 装不起来（这一档还没发？）\n'; fail=1
fi

echo
echo "二 · npm：mingli-wasm-yijing"
mkdir -p "$work/js"
(cd "$work/js" && npm init -y >/dev/null 2>&1 && npm i --silent "mingli-wasm-yijing@$VER" >/dev/null 2>&1) || {
  printf '  ✗ 装不起来（这一档还没发？）\n'; fail=1; }
if [ -f "$work/js/node_modules/mingli-wasm-yijing/mingli_wasm_bg.wasm" ]; then
  cat > "$work/js/run.mjs" <<'EOF'
import { readFileSync } from 'node:fs';
import init, { cast_one } from 'mingli-wasm-yijing';
await init({ module_or_path: readFileSync(new URL('./node_modules/mingli-wasm-yijing/mingli_wasm_bg.wasm', import.meta.url)) });
const out = JSON.parse(cast_one('yijing', JSON.stringify({
  year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0, seed: 2024 })));
if (out.id !== 'yijing' || !out.chart || out.chart.primary_king_wen == null) {
  console.log('BAD'); process.exit(1);
}
console.log('OK', out.chart.primary_king_wen);
EOF
  if r=$(cd "$work/js" && node run.mjs 2>/dev/null) && [ "${r%% *}" = "OK" ]; then
    printf '  ✓ 装得起来，起卦得文王序第 %s 卦\n' "${r#OK }"
  else
    printf '  ✗ 装起来了但跑不出卦\n'; fail=1
  fi
  got_bytes=$(wc -c < "$work/js/node_modules/mingli-wasm-yijing/mingli_wasm_bg.wasm" | tr -d ' ')
  want_bytes=$(awk '$1=="chart-solo-yijing"{print $2}' "$ROOT/scripts/wasm-budget.txt")
  if [ -n "$want_bytes" ] && [ "$got_bytes" != "$want_bytes" ]; then
    printf '  ✗ 发出去的是 %s 字节，预算表写的是 %s\n' "$got_bytes" "$want_bytes"; fail=1
  else
    printf '  ✓ %s 字节，与预算表一致\n' "$got_bytes"
  fi
fi

echo
[ "$fail" -eq 0 ] || { echo "两种形态里有装不起来或对不上的"; exit 1; }
echo "两种形态都能从公开 registry 装起来并跑出正确答案"
