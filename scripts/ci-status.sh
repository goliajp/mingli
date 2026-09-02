#!/usr/bin/env bash
# 上一次 push 之后 CI 是绿是红。
#
# 本地全绿不等于 CI 全绿：CI 上还有本地不跑的那几样——wasm 工具链、浏览器截图、
# 发版彩排、体积闸、契约快照，以及**共享机器上的计时**。
#
# 实测代价：2026-08-29 到 09-03 之间 CI 一直红，而我每次只看本地就 push 了二十多个提交。
# 三个原因各不相同，没有一个是本地看得见的：
#   1. 前端那个 job 要跑 `npm-pack.sh`，可它没装 wasm-bindgen（步骤放错了 job）
#   2. 「一片普通叶不许比中位数贵 20 倍」——本地 ziwei 是 10.2 倍，CI 上抖到正好 20.0
#   3. 我把 `(days / 3.0).max(0.0)` 改成 `days / 3.0`，打断了一条探测的 sed 表达式
#
#   ./scripts/ci-status.sh          # 看最近一次
#   ./scripts/ci-status.sh 5        # 看最近五次
#
# push 完跑一次它，比等下一个人发现便宜得多。

set -euo pipefail
cd "$(dirname "$0")/.."

command -v gh >/dev/null || { echo "缺 gh（GitHub CLI）" >&2; exit 1; }

n=${1:-1}
rows=$(gh run list --limit "$n" --json conclusion,status,displayTitle,databaseId 2>/dev/null) || {
  echo "取不到 CI 状态——gh 没登录，或这个仓库没有 Actions" >&2; exit 1
}

python3 - "$rows" <<'PY'
import json, sys

rows = json.loads(sys.argv[1])
if not rows:
    print("没有运行记录")
    raise SystemExit(0)

bad = 0
for r in rows:
    if r["status"] != "completed":
        mark, note = "…", f"还在跑（{r['status']}）"
    elif r["conclusion"] == "success":
        mark, note = "✓", "绿"
    else:
        mark, note = "✗", r["conclusion"]
        bad += 1
    print(f"  {mark} {note:<10} {r['displayTitle'][:64]}")
    if mark == "✗":
        print(f"      gh run view {r['databaseId']} --log-failed")

raise SystemExit(1 if bad else 0)
PY
