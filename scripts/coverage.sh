#!/usr/bin/env bash
# 覆盖率基线，并把「低于门槛的文件」逐条列出来。
#
# 覆盖率本身不是目标——把它当 KPI 会催生只跑代码不断言的测试。这里只当**探照灯**用：
# 照出哪几段从没被执行过，然后逐条判「该补测试」还是「测不到且有理由」。
# 后者要在下面的 EXPLAINED 里点名，改的人得说清为什么。
#
#   ./scripts/coverage.sh          # 跑一遍，列出低于门槛的文件
#   ./scripts/coverage.sh 95       # 自定门槛
#
# 需要 cargo-llvm-cov：cargo install cargo-llvm-cov --locked

set -euo pipefail
cd "$(dirname "$0")/.."

THRESHOLD=${1:-90}

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "没装 cargo-llvm-cov：cargo install cargo-llvm-cov --locked" >&2
  exit 1
fi

out=$(mktemp)
trap 'rm -f "$out"' EXIT

# 必须先 clean：陈留的 profraw 会让覆盖率虚低，而虚低的数字长得跟「真没覆盖」一模一样
cargo llvm-cov clean --workspace
cargo llvm-cov --workspace --json --output-path "$out" >/dev/null

python3 - "$out" "$THRESHOLD" <<'PY'
import json, sys

data = json.load(open(sys.argv[1]))
threshold = float(sys.argv[2])

# 低于门槛且**已经判过**的文件：每条写明为什么测不到。
# 名单要动就得改这里，改的人得说清理由——与 architecture.rs 的两张名单同一个用法。
EXPLAINED = {
    "services/mingli-api/src/main.rs":
        "服务入口：绑端口并永久 serve，测试里跑它就不会返回",
    "services/mingli-api/src/backend.rs":
        "claude CLI 那条是外部进程。测试一律走离线模板（Interpret::Offline），"
        "正是为了不依赖机器上装没装、也不让一次测试跑一分钟",
    "crates/mingli-wasm/src/lib.rs":
        "错误路径返回 JsValue，宿主目标上构造不出来；ok-path 已逐个覆盖",
    "crates/mingli-app/src/interpret.rs":
        "未覆盖的是「主后端失败 → 回退离线模板」那一支。制造失败要么起外部进程"
        "（这台机器上恰好装着，一次六十秒且不确定），要么造一个必然失败的后端注进去；"
        "后者已由 api 层的 error 形状测试覆盖，此处不重复",
    "services/mingli-api/src/routes/natal.rs":
        "同上：释义 handler 的 500 分支。形状由 endpoints.rs 的错误响应测试直接钉住",
    "services/mingli-api/src/routes/event.rs": "同 natal.rs",
    "services/mingli-api/src/routes/locative.rs": "同 natal.rs",
    "services/mingli-api/src/routes/mundane.rs": "同 natal.rs",
    "services/mingli-api/src/routes/synastry.rs": "同 natal.rs",
    "crates/mingli-liuren/src/bearings.rs":
        "未覆盖的是「三传不出时改列四课上神」那一支。扫 1980–1989 共 2880 课，"
        "无三传的一课都没有——取传九门总能给出三传，这一支实际到不了。"
        "该 crate 有一条测试把这个事实钉住（the_nine_gates_always_yield_a_transmission），"
        "哪天真出现无三传的课它会红，届时这条判词也要改",
    "crates/mingli-contract/src/intent.rs":
        "未覆盖的是 `const fn i(...)` 构造器——它只在 `const {}` 块里求值，编译期跑完，"
        "运行时永不执行。同理 declare.rs 的 `const fn s`。这类不是没测，是构造使然不可达",
    "crates/mingli-contract/src/declare.rs": "同 intent.rs：`const fn s` 只在编译期求值",
    "crates/mingli-app/src/lib.rs":
        "Birth::validate 的几条越界分支只在 api 层被触发，用例层自己的测试不构造非法输入",
}

files = data["data"][0]["files"]
total = data["data"][0]["totals"]["regions"]
print(f"总覆盖 {total['percent']:.2f}%（{total['count'] - total['covered']} / {total['count']} 未覆盖）\n")

unexplained = []
for f in sorted(files, key=lambda x: x["summary"]["regions"]["percent"]):
    pct = f["summary"]["regions"]["percent"]
    if pct >= threshold:
        continue
    path = f["filename"].split("mingli/")[-1]
    why = EXPLAINED.get(path)
    mark = "已判" if why else "待判"
    print(f"  [{mark}] {pct:6.2f}%  {path}")
    if why:
        print(f"          {why}")
    else:
        unexplained.append(path)

if unexplained:
    print(f"\n有 {len(unexplained)} 个文件低于 {threshold}% 且未判过。")
    print("逐条判「补测试」还是「测不到且有理由」——后者写进本脚本的 EXPLAINED。")
    sys.exit(1)
print(f"\n低于 {threshold}% 的文件都判过了。")
PY
