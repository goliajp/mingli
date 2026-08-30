#!/usr/bin/env bash
# 把各具名档位打成 npm 包。
#
#   ./scripts/npm-pack.sh              # 只打包，产物在 dist/npm/
#   ./scripts/npm-pack.sh --publish    # 打包并 npm publish（不可撤回）
#
# 走的是与 scripts/wasm-size.sh **同一条**管线：cargo build → wasm-bindgen → wasm-opt -Oz。
# 两处必须同一条，否则预算表里的字节数与真正发出去的字节数是两回事，
# 而那正是本项目已经踩过一次的坑（同一个包，两条管线量出 1,512,240 与 1,528,046）。
set -euo pipefail
cd "$(dirname "$0")/.."

VER=$(sed -n '/^\[workspace.package\]/,/^\[/p' Cargo.toml | sed -n 's/^version = "\(.*\)"/\1/p')
REPO=$(sed -n '/^\[workspace.package\]/,/^\[/p' Cargo.toml | sed -n 's/^repository = "\(.*\)"/\1/p')
LIC=$(sed -n '/^\[workspace.package\]/,/^\[/p' Cargo.toml | sed -n 's/^license = "\(.*\)"/\1/p')
[ -n "$VER" ] && [ -n "$REPO" ] && [ -n "$LIC" ] || { echo "读不出 workspace 的版本 / 仓库 / 许可" >&2; exit 1; }

LEAVES="bazi,ziwei,astrology,jyotish,qizhengsiyu,yijing,geomancy,sikidy,ifa,cartomancy,meihua,xiaoliuren,zeri,maya,pawukon,mahabote,liuren,qimen,taiyi,tibetan,numerology,gematria,abjad,wuge"

# npm 包名|体积档位名|feature 串|keywords（逗号分隔）|一句话说明
PKGS="
mingli-wasm|full|usecases,$LEAVES|divination,astrology,bazi,yijing,wasm|Twenty-four divination algorithms in the browser: charts for all of them, plus the eight cross-leaf intents.
mingli-wasm-chart|chart-all|$LEAVES|divination,astrology,horoscope,almanac,wasm|Charts only, all twenty-four systems. Half the size of the full build; no cross-leaf use cases.
mingli-wasm-chinese|chart-chinese|bazi,ziwei,qimen,liuren,meihua,yijing,xiaoliuren,zeri,taiyi,tibetan|bazi,ziwei,qimen,yijing,wasm|The ten Chinese systems, charts only.
mingli-wasm-bazi|chart-solo-bazi|bazi|bazi,four-pillars,chinese,astrology,wasm|Four Pillars of Destiny alone.
mingli-wasm-yijing|chart-solo-yijing|yijing|yijing,iching,hexagram,divination,wasm|Yi Jing casting alone. The smallest build there is.
"

for tool in wasm-bindgen wasm-opt npm; do
  command -v $tool >/dev/null || { echo "缺 $tool" >&2; exit 1; }
done

publish=${1:-}
out=dist/npm; rm -rf $out; mkdir -p $out

while IFS='|' read -r pkg profile feats kw blurb; do
  [ -n "$pkg" ] || continue
  d="$out/$pkg"; mkdir -p "$d"
  printf '  %-22s ' "$pkg"
  cargo build -q --release --target wasm32-unknown-unknown -p mingli-wasm \
    --no-default-features --features "$feats"
  wasm-bindgen --target web --out-dir "$d" \
    target/wasm32-unknown-unknown/release/mingli_wasm.wasm >/dev/null 2>&1
  wasm-opt -Oz -o "$d/mingli_wasm_bg.wasm.opt" "$d/mingli_wasm_bg.wasm"
  mv "$d/mingli_wasm_bg.wasm.opt" "$d/mingli_wasm_bg.wasm"
  rm -f "$d/.gitignore" "$d/package.json"

  bytes=$(wc -c < "$d/mingli_wasm_bg.wasm" | tr -d ' ')
  # 发出去的字节必须就是预算表里那一行——两处不一致说明有一条管线走岔了。
  want=$(awk -v n="$profile" '$1==n{print $2}' scripts/wasm-budget.txt)
  if [ -n "$want" ] && [ "$bytes" != "$want" ]; then
    printf '✗ %s 字节，预算表写的是 %s——两条管线对不上\n' "$bytes" "$want"; exit 1
  fi

  KEYWORDS=$(printf '%s' "$kw" | awk -F, '{for(i=1;i<=NF;i++) printf "%s\"%s\"", (i>1?", ":""), $i}')
  cat > "$d/package.json" <<JSON
{
  "name": "$pkg",
  "version": "$VER",
  "description": "$blurb",
  "license": "$LIC",
  "repository": { "type": "git", "url": "git+$REPO.git" },
  "keywords": [$KEYWORDS],
  "type": "module",
  "main": "mingli_wasm.js",
  "module": "mingli_wasm.js",
  "types": "mingli_wasm.d.ts",
  "sideEffects": false,
  "files": ["mingli_wasm.js", "mingli_wasm_bg.wasm", "mingli_wasm.d.ts", "mingli_wasm_bg.wasm.d.ts", "README.md"]
}
JSON
  cat > "$d/README.md" <<MD
# $pkg

$blurb

\`\`\`js
import init, { cast_one } from '$pkg';
await init();
const chart = JSON.parse(cast_one('bazi', JSON.stringify({
  year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0
})));
\`\`\`

Module size: $bytes bytes before compression. Every published profile has a byte
budget checked in at \`scripts/wasm-budget.txt\`; CI fails on a regression.

Same algorithms as the \`mingli\` crates on crates.io — one source, two shapes.
See $REPO.
MD
  printf '%9s 字节\n' "$bytes"
done <<< "$PKGS"

echo
if [ "$publish" = "--publish" ]; then
  npm whoami >/dev/null || { echo "没登录 npm" >&2; exit 1; }
  for d in $out/*/; do
    printf '  发布 %s ... ' "$(basename "$d")"
    (cd "$d" && npm publish --access public >/dev/null) && echo "✓" || { echo "✗"; exit 1; }
  done
  echo "已发布"
else
  echo "产物在 $out/。要真发出去加 --publish（不可撤回）"
fi
