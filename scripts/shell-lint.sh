#!/usr/bin/env bash
# 全角标点紧跟在 `$变量` 后面时，bash 会把标点的头一个字节吃进变量名。
#
#   echo "版本 $VER（按 …）"     →  ${VER（  →  unbound variable
#   echo "版本 ${VER}（按 …）"   →  对
#
# 这个坑本项目踩过三次，每次都只在**有话要说的那条路径**上炸：
# 平时那行不执行，`set -u` 也就不响，直到出错时脚本自己先死了。
# 所以立一条机械检查——它比记性可靠。
set -euo pipefail
cd "$(dirname "$0")/.."

# `$name` 后面紧跟全角标点；`${name}` 形式不算。
PAT='\$[A-Za-z_][A-Za-z0-9_]*(（|）|「|」|，|。|：|；|？|！|…|——)'

# 注释里出现不算——注释不执行，而本脚本自己的说明里就得举一个反例。
hits=$(grep -rnE "$PAT" scripts/*.sh web/e2e/*.mjs bench/browser/*.mjs 2>/dev/null |
       grep -vE ':[0-9]+:[[:space:]]*(#|//)' || true)
n=$(printf '%s\n' "$hits" | grep -c . || true)
files=$(ls scripts/*.sh 2>/dev/null | wc -l | tr -d ' ')
[ "$files" -ge 8 ] || { echo "只扫到 $files 个脚本，扫描面怕是失效了" >&2; exit 1; }

fail=0
if [ "$n" -gt 0 ]; then
  echo "$n 处变量名紧挨全角标点，会被吃掉一个字节——加花括号："
  printf '%s\n' "$hits" | sed 's/^/  /'
  fail=1
else
  echo "扫过 $files 个脚本，没有变量名挨着全角标点"
fi

# 写死 target/ 读产物。设了 CARGO_TARGET_DIR（或 .cargo/config 的 build.target-dir）时，
# 新产物写到别处，脚本读到的是 target/ 里留下的旧产物——体积闸量的就不是这次的构建。
# 实测过：干净配置下编出 160059 字节，脚本报的却是旧文件的 164031。取目录用 cargo metadata。
TPAT='(^|[^A-Za-z0-9_/$.{-])target/(debug|release|wasm32-|package)'
thits=$(grep -rnE "$TPAT" scripts/*.sh web/e2e/*.mjs bench/browser/*.mjs 2>/dev/null |
        grep -vE ':[0-9]+:[[:space:]]*(#|//)' || true)
tn=$(printf '%s\n' "$thits" | grep -c . || true)
if [ "$tn" -gt 0 ]; then
  echo "$tn 处写死了 target/ 下的产物路径——用 cargo metadata 的 target_directory："
  printf '%s\n' "$thits" | sed 's/^/  /'
  fail=1
else
  echo "没有写死 target/ 的产物路径"
fi
exit $fail
