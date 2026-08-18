#!/usr/bin/env bash
# 把整套 HTTP 响应抓成一份文本，用来证明「改完之后线上还是原来那个形状」。
#
# 端点测试验的是「这个端点该有的性质还在」，验不了「它一个字节都没变」。重构承接层时
# 真正要防的是后者：handler 里悄悄多补一个字段、少序列化一段、把某个错误从 400 挪成
# 500，任何测试都不会红，前端却会漂。
#
#   ./scripts/api-snapshot.sh save snap-before.txt     # 抓一份存下来
#   ./scripts/api-snapshot.sh check snap-before.txt    # 再抓一份，跟它比
#
# 要抓别的构建（比如改动前的二进制，用 git worktree 建一个），传 MINGLI_SNAPSHOT_BIN：
#
#   git worktree add /tmp/base HEAD~1
#   (cd /tmp/base && cargo build -p mingli-api)
#   MINGLI_SNAPSHOT_BIN=/tmp/base/target/debug/mingli-api ./scripts/api-snapshot.sh save base.txt
#
# 释义端点走**离线模板**后端（MINGLI_INTERPRET_BACKEND=template）：模板的输出是确定的，
# 于是它们的 200 路径也能逐字节比。走 LLM 则每次不同，那只能比 400 路径——
# 而「算不出来的时候怎么答」同样是确定的，两条都收。

set -euo pipefail
cd "$(dirname "$0")/.."

mode=${1:-}
file=${2:-}
if [ "$mode" != save ] && [ "$mode" != check ]; then
  echo "用法：$0 {save|check} <文件>" >&2
  exit 2
fi
if [ -z "$file" ]; then
  echo "用法：$0 $mode <文件>" >&2
  exit 2
fi

BIN=${MINGLI_SNAPSHOT_BIN:-target/debug/mingli-api}
PORT=${MINGLI_SNAPSHOT_PORT:-6099}
B="http://127.0.0.1:$PORT"

if [ ! -x "$BIN" ]; then
  echo "先构建：cargo build -p mingli-api（或用 MINGLI_SNAPSHOT_BIN 指别的二进制）" >&2
  exit 1
fi

out=$(mktemp)
body=$(mktemp)
trap 'rm -f "$out" "$body"; [ -n "${pid:-}" ] && kill "$pid" 2>/dev/null' EXIT

MINGLI_API_BIND="127.0.0.1:$PORT" MINGLI_INTERPRET_BACKEND=template "$BIN" >/dev/null 2>&1 &
pid=$!
# 服务刚起时端口还没开，连不上要重试——不重试的话抓回来的会是一整版 000，
# 而一整版 000 长得跟「所有端点都变了」一模一样。
curl -s --retry-connrefused --retry 30 --retry-delay 1 -m 60 -o /dev/null "$B/api/health"

N='{"year":1990,"month":6,"day":15,"hour":14,"minute":30,"tz":8,"gender":"male","latitude":31.23,"longitude":121.47,"seed":2024,"name":"Ada Lovelace"}'
T='{"year":2026,"month":8,"day":16,"hour":10,"minute":0,"tz":8}'
T2='{"year":2026,"month":8,"day":20,"hour":10,"minute":0,"tz":8}'

g() { printf '### GET %s\n' "$1" >>"$out"; curl -s -o "$body" -w '%{http_code}\n' "$B$1" >>"$out"; cat "$body" >>"$out"; printf '\n' >>"$out"; }
p() { printf '### POST %s\n' "$1" >>"$out"; curl -s -o "$body" -w '%{http_code}\n' -XPOST "$B$1" -H 'content-type: application/json' -d "$2" >>"$out"; cat "$body" >>"$out"; printf '\n' >>"$out"; }

g /api/health
g /api/intents
g /api/analysis
p /api/route "$(printf '%s' "$N" | sed 's/^{/{"kind":"natal",/')"
p /api/bazi "$N"
p /api/bazi '{"year":1990}'
# 三类「看着像合法、其实不存在」的输入。不收的话，历法换算会把 2 月 31 日悄悄挪成 3 月 3 日
p /api/bazi '{"year":1990,"month":2,"day":31,"hour":14,"tz":8}'
p /api/bazi '{"year":1990,"month":6,"day":15,"hour":14,"tz":99}'
p /api/bazi '{"year":1990,"month":6,"day":15,"hour":14,"tz":8,"latitude":91}'
p /api/ziwei "$N"
p /api/ziwei '{"year":1800,"month":1,"day":1,"hour":0,"tz":8}'
p /api/cast "$N"
p /api/bazi/overlay-strength "$(printf '%s' "$N" | sed 's/}$/,"extras":["丙午","庚申"]}/')"
p /api/bazi/overlay-strength "$(printf '%s' "$N" | sed 's/}$/,"extras":["不是干支"]}/')"
p /api/fortune "{\"natal\":$N,\"t_target\":$T}"
p /api/interpret "$(printf '%s' "$N" | sed 's/}$/,"leaf":"没有这片叶"}/')"
# 认不出的主体要被拒，不能静默落回人盘（拼错 company 的人会以为自己看的是公司盘）
p /api/interpret "$(printf '%s' "$N" | sed 's/}$/,"leaf":"bazi","subject":"compnay"}/')"
p /api/team "{\"members\":[{\"year\":1990,\"month\":6,\"day\":15,\"hour\":14,\"tz\":8,\"gender\":\"male\",\"name\":\"A\"},{\"year\":1987,\"month\":3,\"day\":2,\"hour\":9,\"tz\":8,\"gender\":\"female\",\"name\":\"B\"}]}"
p /api/team '{"members":[]}'
p /api/team/interpret '{"members":[]}'
p /api/word '{"system":"gematria","text":"chai"}'
p /api/word '{"system":"没有这个系统","text":"x"}'
p /api/event "{\"t_ask\":$T,\"seed\":7,\"question\":\"能成吗\"}"
p /api/election "{\"window_start\":$T,\"window_end\":$T2,\"category\":\"婚\"}"
p /api/election "{\"window_start\":$T2,\"window_end\":$T}"
p /api/election/interpret "{\"window_start\":$T2,\"window_end\":$T}"

# 释义的 200 路径：离线模板输出确定，故可逐字节比
p /api/event/interpret "{\"t_ask\":$T,\"seed\":7}"
p /api/election/interpret "{\"window_start\":$T,\"window_end\":$T2,\"category\":\"婚\"}"
p /api/locative/interpret "{\"t_ask\":$T,\"seed\":7,\"category\":\"财\"}"
p /api/mundane/interpret '{"founded_at":{"year":1949,"month":10,"day":1,"hour":15,"minute":0,"tz":8},"latitude":39.9,"longitude":116.4,"target_year":2026,"span":3}'
p /api/synastry/interpret '{"a":{"year":1990,"month":6,"day":15,"hour":14,"tz":8,"gender":"male","name":"A"},"b":{"year":1987,"month":3,"day":2,"hour":9,"tz":8,"gender":"female","name":"B"}}'
p /api/team/interpret '{"members":[{"year":1990,"month":6,"day":15,"hour":14,"tz":8,"gender":"male","name":"A"},{"year":1987,"month":3,"day":2,"hour":9,"tz":8,"gender":"female","name":"B"}]}'
p /api/interpret "$(printf '%s' "$N" | sed 's/}$/,"leaf":"bazi"}/')"
p /api/locative "{\"t_ask\":$T,\"seed\":7,\"category\":\"财\"}"
# 这一刻奇门的值符落中五宫。中宫不在圆周上、星门神俱不入，若不按「中 5 寄坤 2」归并，
# 出来的候选会是「方位：中」加一条四段全空的附注
p /api/locative '{"t_ask":{"year":2026,"month":8,"day":1,"hour":9,"minute":0,"tz":8},"seed":7,"category":"财"}'
p /api/synastry "{\"a\":{\"year\":1990,\"month\":6,\"day\":15,\"hour\":14,\"tz\":8,\"gender\":\"male\",\"name\":\"A\"},\"b\":{\"year\":1987,\"month\":3,\"day\":2,\"hour\":9,\"tz\":8,\"gender\":\"female\",\"name\":\"B\"}}"
p /api/mundane '{"founded_at":{"year":1949,"month":10,"day":1,"hour":15,"minute":0,"tz":8},"latitude":39.9,"longitude":116.4,"target_year":2026,"span":3}'

# 旁证：量具坏掉的样子跟数据长得一样，所以抓完先看抓没抓着。
if grep -qE '^000$' "$out"; then
  echo "✗ 有请求没连上（服务没起来？）" >&2
  exit 1
fi
n=$(grep -c '^### ' "$out")
if [ "$n" != 37 ]; then
  echo "✗ 只抓到 $n 个请求，应是 37" >&2
  exit 1
fi

if [ "$mode" = save ]; then
  cp "$out" "$file"
  echo "✓ 抓下 $n 个请求 → $file"
else
  if [ ! -f "$file" ]; then
    # ${} 不能省：紧跟中文标点时，bash 会把那几个字节读进变量名，
    # 于是这条提示自己先崩成 unbound variable，读的人看到的是 bash 报错而不是提示
    echo "✗ 没有 ${file}，先跑一次 save" >&2
    exit 1
  fi
  if diff -u "$file" "$out"; then
    echo "✓ $n 个请求逐字节相同"
  else
    echo "✗ 与 $file 有出入（上面是 diff：< 是存档，> 是这次）" >&2
    exit 1
  fi
fi
