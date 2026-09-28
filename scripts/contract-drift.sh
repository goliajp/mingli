#!/usr/bin/env bash
# 当前工作树的 HTTP 契约，与上一次提交的，逐字节比。
#
# `api-snapshot.sh` 自己只会「抓一份」与「跟给定的一份比」。CI 里那一步是同一个构建
# save 完立刻 check，证的是跨进程确定性（Rust 的 HashMap 每进程换种子，`/api/analysis`
# 当初正是这么坏的），不是「改动前后一致」——那一半脚本头部写着「留给本地」，
# 于是铁律五「对外契约不悄悄变」一直没有执行面。
#
# 这支把上一版建在 git worktree 里，用它抓基准，再用当前构建比。
# 差异不必然是错——但必须是有意的，并写进 commit message。
set -euo pipefail
cd "$(dirname "$0")/.."

BASE_REF=${1:-HEAD}
WORK=$(mktemp -d)
SNAP=$(mktemp)
cleanup() { git worktree remove --force "$WORK" >/dev/null 2>&1 || true; rm -rf "$WORK" "$SNAP"; }
trap cleanup EXIT

printf '\n把 %s 的契约抓下来，跟当前工作树比\n\n' "$BASE_REF"

git worktree add -q --detach "$WORK" "$BASE_REF"
# 基准版建在固定的独立目录里，不与当前版共用：两棵树的工作区 crate 产物名相同，
# 共用一个 target 会互相覆盖，基准可能根本不是上一版（实测种下的错因此漏过）。
# 目录不随每次运行新建，第三方依赖第二次起就是热的；先前用 worktree 自己的空 target，
# 每次都把全部依赖冷编一遍。
TARGET=${CARGO_TARGET_DIR:-$PWD/target}
BASE_TARGET=$TARGET/contract-base
( cd "$WORK" && CARGO_TARGET_DIR="$BASE_TARGET" cargo build -q -p mingli-api )
cp "$BASE_TARGET/debug/mingli-api" "$WORK/mingli-api-base"
MINGLI_SNAPSHOT_BIN="$WORK/mingli-api-base" ./scripts/api-snapshot.sh save "$SNAP" | sed 's/^/  基准 /'

cargo build -q -p mingli-api
MINGLI_SNAPSHOT_BIN="$TARGET/debug/mingli-api" ./scripts/api-snapshot.sh check "$SNAP"
