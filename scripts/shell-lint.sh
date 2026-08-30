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

if [ "$n" -gt 0 ]; then
  echo "$n 处变量名紧挨全角标点，会被吃掉一个字节——加花括号："
  printf '%s\n' "$hits" | sed 's/^/  /'
  exit 1
fi
echo "扫过 $files 个脚本，没有变量名挨着全角标点"
