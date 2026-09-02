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

# npm 包名|体积档位名|feature 串|keywords（逗号分隔）|一句话说明|自检（node 表达式，须为真）
#
# 最后那一列不是装饰。`mingli-wasm-astrology-thin@1.1.0` 发出去时注册表是空的——
# 装配根登记那一行只认 `astrology` 不认 `astrology-thin`，于是 cast_one 返回 null、
# cast 返回 []。体积闸量到了字节、预算对得上、npm 照发不误，因为**没有一道闸问过
# 它算不算得出东西**。现在发之前先问。
PKGS="
mingli-wasm|full|usecases,$LEAVES|divination,astrology,bazi,yijing,wasm|Twenty-four divination algorithms in the browser: charts for all of them, plus the eight cross-leaf intents.|JSON.parse(M.cast(Q)).length >= 20
mingli-wasm-chart|chart-all|$LEAVES|divination,astrology,horoscope,almanac,wasm|Charts only, all twenty-four systems. Half the size of the full build; no cross-leaf use cases.|JSON.parse(M.cast(Q)).length >= 20
mingli-wasm-chinese|chart-chinese|bazi,ziwei,qimen,liuren,meihua,yijing,xiaoliuren,zeri,taiyi,tibetan|bazi,ziwei,qimen,yijing,wasm|The ten Chinese systems, charts only.|JSON.parse(M.cast(Q)).length >= 10
mingli-wasm-bazi|chart-solo-bazi|bazi|bazi,four-pillars,chinese,astrology,wasm|Four Pillars of Destiny alone.|JSON.parse(M.cast_one('bazi',Q)).chart.year.ganzhi.length > 0
mingli-wasm-yijing|chart-solo-yijing|yijing|yijing,iching,hexagram,divination,wasm|Yi Jing casting alone. The smallest build there is.|JSON.parse(M.cast_one('yijing',Q)).chart.primary_king_wen > 0
mingli-wasm-astrology-thin|astrology-thin|astrology-thin|astrology,horoscope,natal,ephemeris,wasm|Natal charts with a built-in ephemeris, truncated to a twentieth of the terms: 89% smaller than the full VSOP87D tables, at 4.3 arcseconds against published charts that resolve to the arcminute.|JSON.parse(M.cast_one('astrology',Q)).chart.planets.length >= 9
mingli-wasm-astrology-lite|astrology-lite|astrology-lite|astrology,horoscope,natal,ephemeris,wasm|Natal charts from planetary longitudes you supply: houses, cusps, aspects and signs, without the ephemeris. 90% smaller than carrying VSOP87D.|JSON.parse(M.astrology_with(Q, JSON.stringify([0,1,2,3,4,5,6,7,8]))).planets.length >= 9
"

for tool in wasm-bindgen wasm-opt npm; do
  command -v $tool >/dev/null || { echo "缺 $tool" >&2; exit 1; }
done

publish=${1:-}
out=dist/npm; rm -rf $out; mkdir -p $out

while IFS='|' read -r pkg profile feats kw blurb check; do
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

  # 发之前先问它算不算得出东西。
  cat > "$d/.selfcheck.mjs" <<CHECK
import { readFileSync } from 'node:fs';
import init, * as M from './mingli_wasm.js';
await init({ module_or_path: readFileSync(new URL('./mingli_wasm_bg.wasm', import.meta.url)) });
const Q = JSON.stringify({ year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
                           latitude: 31.23, longitude: 121.47, seed: 2024 });
if (!($check)) { console.error('FAILED'); process.exit(1); }
CHECK
  if ! (cd "$d" && node .selfcheck.mjs >/dev/null 2>&1); then
    printf '✗ %s 装配出来算不出东西——检查装配根有没有登记这一档\n' "$pkg"; exit 1
  fi
  rm -f "$d/.selfcheck.mjs"

  bytes=$(wc -c < "$d/mingli_wasm_bg.wasm" | tr -d ' ')
  # 发出去的字节要落在预算表那一行的余量内——差太多说明有一条管线走岔了。
  #
  # 从前这里要求**逐字节相等**。同机同工具链下那是对的，本项目也确实靠它抓到过
  # 「同一个包两条管线量出 1,512,240 与 1,528,046」。但这个脚本现在也在 CI 上跑，
  # 而同一份源码换台机器就差几百字节，相等便不再成立。留 1.5%（与 wasm-size.sh 同一条）：
  # 管线走岔是万级的差，机器差异是百级的，这条界分得开。
  want=$(awk -v n="$profile" '$1==n{print $2}' scripts/wasm-budget.txt)
  if [ -n "$want" ] && [ "$bytes" -gt "$(( want + want * 3 / 200 ))" ]; then
    printf '✗ %s 字节，预算表写的是 %s（上限 %s）——两条管线对不上\n' "$bytes" "$want" "$(( want + want * 3 / 200 ))"; exit 1
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
