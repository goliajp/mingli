#!/usr/bin/env bash
# 反过来验守卫：往源码里种一个已知的错，看该守卫是不是真的红。
#
# 一条永远绿的守卫和一条真守着东西的守卫，在日常 `cargo test` 里长得一模一样。
# 分辨它们只有一个办法——**把它该拦的东西种回去，看它拦不拦**。
#
# 这件事本来是靠人现场做一次、然后写进 commit message 的。可那样的结论留在历史里，
# 留不进仓库：接手的人打开测试文件，看不出哪条被验过、哪条只是看着像在守。
# 这个脚本把那一次手工探测变成可以重跑的东西。
#
#   ./scripts/guard-probe.sh              # 全跑
#   ./scripts/guard-probe.sh 架构          # 只跑名字含「架构」的组
#
# 每条探测由四样东西定义：组名、被探的测试名、要改的文件、改法（sed 表达式）。
# 脚本会先确认那个测试名**真的存在**——测试改了名而探测没跟着改，是这类脚本最容易
# 出的那种坏法：它照常绿，因为它跑的是零个测试。
#
# 本脚本会改工作区文件再改回来。中途被打断也会还原（trap）。
#
# 它自己的三条报错路径都不是纸上的分支——写第一版时全都真出现过：
# 「配错对」（把释义层的违规配给了装配根那条守卫，那条根本不看释义层）、
# 「种不下去」（sed 表达式没匹配上，于是它跑的是一份没被改过的源码，照常绿）、
# 「一个也没拦」（尚未出现，是这个脚本存在的理由）。

set -euo pipefail
cd "$(dirname "$0")/.."

filter=${1:-}
pass=0; fail=0; skipped=0; mismatched=0
declare -a BACKUPS=()

restore() {
  for b in "${BACKUPS[@]:-}"; do
    [ -n "$b" ] || continue
    orig=${b%.guardprobe.bak}
    [ -f "$b" ] && mv "$b" "$orig"
  done
  BACKUPS=()
}
trap 'restore' EXIT INT TERM

# probe <组名> <crate> <测试名> <文件> <sed 表达式>
probe() {
  local group=$1 pkg=$2 test=$3 file=$4 expr=$5
  if [ -n "$filter" ] && [[ "$group" != *"$filter"* ]]; then return 0; fi

  printf '  %-46s ' "$group"

  if ! grep -q "fn $test\b" "$(dirname "$file")"/../tests/*.rs 2>/dev/null \
     && ! grep -rq "fn $test\b" crates/*/tests services/*/tests 2>/dev/null; then
    printf '⊘ 测试 %s 不存在（改名了？）\n' "$test"
    skipped=$((skipped+1)); return 0
  fi

  cp "$file" "$file.guardprobe.bak"; BACKUPS+=("$file.guardprobe.bak")

  if ! sed -i '' "$expr" "$file" 2>/dev/null; then
    printf '⊘ sed 没改动任何东西\n'; restore; skipped=$((skipped+1)); return 0
  fi
  if cmp -s "$file" "$file.guardprobe.bak"; then
    printf '⊘ 种下去的错没落地（表达式没匹配上）\n'; restore; skipped=$((skipped+1)); return 0
  fi

  # 编译失败也算红——有些错是编译期拦下的，机制不同但同样拦住了
  local red='test result: FAILED|^error(\[|:)|could not compile'
  local out
  out=$(cargo test -p "$pkg" "$test" 2>&1 || true)

  if grep -qE "$red" <<<"$out"; then
    restore; printf '✓ 红了\n'; pass=$((pass+1)); return 0
  fi

  # 点名的那条没红。再问一句：**别人拦住了吗**——这两件事完全不同。
  # 「配错对」是探测表写错了，改表即可；「一个也没拦」才是真的有洞。
  local whole
  whole=$(cargo test -p "$pkg" 2>&1 || true)
  restore
  if grep -qE "$red" <<<"$whole"; then
    printf '⚠ %s 没红，但同包别的守卫拦住了（配错对）\n' "$test"
    mismatched=$((mismatched+1))
  else
    printf '✗ 种了错，整包没有一条守卫红\n'
    fail=$((fail+1))
  fi
}

printf '\n往源码里种错，看守卫红不红\n\n'

# ── 架构 ──────────────────────────────────────────────────────────
probe "架构：编排层偷看了一片叶" mingli-registry orchestration_does_not_know_any_leaf \
  crates/mingli-engine/Cargo.toml \
  's|^\[dependencies\]|[dependencies]\nmingli-bazi = { workspace = true }|'

probe "架构：释义层直连了一片叶" mingli-registry the_interpretation_layer_only_knows_the_ports \
  crates/mingli-interpret/Cargo.toml \
  's|^\[dependencies\]|[dependencies]\nmingli-bazi = { workspace = true }|'

probe "架构：叶横向依赖另一片叶" mingli-registry every_dependency_points_strictly_inward \
  crates/mingli-yijing/Cargo.toml \
  's|^\[dependencies\]|[dependencies]\nmingli-ziwei = { workspace = true }|'

probe "架构：交付层抄叶名而不写 full" mingli-registry the_delivery_layer_says_which_leaves_it_ships \
  services/mingli-api/Cargo.toml \
  's|features = \["full"\]|features = ["bazi", "ziwei"]|'

probe "架构：承接层绕过装配根直连叶" mingli-registry the_composition_root_is_the_only_place_that_lists_leaves \
  services/mingli-api/Cargo.toml \
  's|^\[dependencies\]|[dependencies]\nmingli-bazi = { workspace = true }|'

# ── 契约 ──────────────────────────────────────────────────────────
probe "契约：一片叶悄悄从注册表消失" mingli-registry cast_all_has_all_leaves \
  crates/mingli-registry/src/lib.rs \
  's|Box::new(mingli_ziwei::ZiweiEngine),||'

probe "契约：占卜类的种子不再抵达叶" mingli-registry the_drawing_seed_reaches_exactly_the_leaves_that_declare_it \
  crates/mingli-yijing/src/engine.rs \
  's|effective_seed(|0u64.wrapping_add(0 * effective_seed(|'

# ── 数值 ──────────────────────────────────────────────────────────
probe "数值：日柱错一位" mingli-registry natal_cast_path_unchanged_regression_guard \
  crates/mingli-ganzhi/src/cycle.rs \
  's|^pub const DAY_ANCHOR_JDN: i64 = 2_460_311;|pub const DAY_ANCHOR_JDN: i64 = 2_460_312;|'


# ── 契约面：改了不报错、只是答得不一样的那类 ────────────────────
probe "契约：认不出的 subject 又被当成人盘" mingli-api an_unrecognised_subject_is_refused_rather_than_read_as_a_person \
  services/mingli-api/src/routes/natal.rs \
  's|None => return bad_request(format!("subject 认不出|None => mingli_interpret::Subject::Person, #[allow(unreachable_code)] _ => return bad_request(format!("subject 认不出|'

probe "契约：路由加了一条 README 没写的" mingli-registry the_endpoint_table_lists_every_route_and_no_others \
  services/mingli-api/src/lib.rs \
  's|"/api/mundane"|"/api/mundanex"|'

probe "自陈：Und 条目丢了 🟡 标记" mingli-registry every_undetermined_item_carries_the_marker_and_says_something \
  crates/mingli-abjad/src/engine.rs \
  's|Und, "🟡 多数计法归|Und, "多数计法归|'

probe "自陈：读法提到盘上没有的字段" mingli-registry every_field_name_in_the_reading_notes_exists_on_the_chart \
  crates/mingli-bazi/src/engine.rs \
  's|- `xunkong`：|- `xunkongz`：|'

probe "自陈：README 说的探测条数与实际不符" mingli-registry the_number_of_planted_faults_is_what_the_script_plants \
  README.md \
  's|plants 13 known faults|plants 12 known faults|'

printf '\n%d 条红了，%d 条配错对，%d 条一个也没拦，%d 条跳过\n' \
  "$pass" "$mismatched" "$fail" "$skipped"
[ "$fail" -eq 0 ] && [ "$skipped" -eq 0 ] && [ "$mismatched" -eq 0 ]
