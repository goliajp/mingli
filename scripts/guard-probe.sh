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
#   ./scripts/guard-probe.sh -前端         # 跑除「前端」以外的组
#
# 前端那一族要 :6026 与 :6027 都在应答，所以 CI 里它挂在已经起了服务的那个 job 上，
# 别处用 `-前端` 排除。分开跑而不是「服务不在就当过」——后者正是这脚本要治的病。
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


# BSD sed（macOS）要 `-i ''`，GNU sed（CI 上的 Linux）要 `-i` 且不能跟空串——
# 后者会把 '' 当成脚本、把真正的表达式当成文件名。这个脚本两边都要跑，故先认一次。
if sed --version >/dev/null 2>&1; then
  sedi() { sed -i "$@"; }
else
  sedi() { sed -i '' "$@"; }
fi

filter=${1:-}
exclude=""
case "$filter" in -?*) exclude=${filter#-}; filter="";; esac

# 这一组要不要跑
wanted() {
  [ -z "$filter" ] || [[ "$1" == *"$filter"* ]] || return 1
  [ -z "$exclude" ] || [[ "$1" != *"$exclude"* ]] || return 1
  return 0
}
pass=0; fail=0; skipped=0; mismatched=0
declare -a BACKUPS=()

restore() {
  for b in "${BACKUPS[@]:-}"; do
    [ -n "$b" ] || continue
    orig=${b%.guardprobe.bak}
    if [ -d "$b" ]; then
      rm -rf "$orig"; mv "$b" "$orig"
    elif [ -f "$b" ]; then
      mv "$b" "$orig"
    fi
  done
  BACKUPS=()
}

# 把种错落到一个文件或一整棵目录上。目录是给前端用的：一个字段名散在
# types.ts、视图、样式里，只改其中一个盖不住——`wired.mjs` 是全 src 搜的。
plant() {
  local target=$1 expr=$2
  if [ -d "$target" ]; then
    cp -R "$target" "$target.guardprobe.bak"; BACKUPS+=("$target.guardprobe.bak")
    while IFS= read -r f; do sedi "$expr" "$f"; done < <(
      find "$target" -type f \( -name '*.ts' -o -name '*.tsx' -o -name '*.css' \)
    )
    ! diff -rq "$target" "$target.guardprobe.bak" >/dev/null 2>&1
  else
    cp "$target" "$target.guardprobe.bak"; BACKUPS+=("$target.guardprobe.bak")
    sedi "$expr" "$target" 2>/dev/null || true
    ! cmp -s "$target" "$target.guardprobe.bak"
  fi
}
trap 'restore' EXIT INT TERM

# probe <组名> <crate> <测试名> <文件> <sed 表达式>
probe() {
  local group=$1 pkg=$2 test=$3 file=$4 expr=$5
  wanted "$group" || return 0

  printf '  %-46s ' "$group"

  if ! grep -q "fn $test\b" "$(dirname "$file")"/../tests/*.rs 2>/dev/null \
     && ! grep -rq "fn $test\b" crates/*/tests services/*/tests 2>/dev/null; then
    printf '⊘ 测试 %s 不存在（改名了？）\n' "$test"
    skipped=$((skipped+1)); return 0
  fi

  cp "$file" "$file.guardprobe.bak"; BACKUPS+=("$file.guardprobe.bak")

  if ! sedi "$expr" "$file" 2>/dev/null; then
    printf '⊘ sed 没改动任何东西\n'; restore; skipped=$((skipped+1)); return 0
  fi
  if cmp -s "$file" "$file.guardprobe.bak"; then
    printf '⊘ 种下去的错没落地（表达式没匹配上）\n'; restore; skipped=$((skipped+1)); return 0
  fi

  # 清单类的种错（Cargo.toml）本来就可能编译期就被拦下——那是正当的拦法，算红。
  # 源码类的不行：`.rs` 改完编不过，说明**表达式写坏了**，那一跑测的不是守卫是语法。
  # 第一版把两者混为一谈，于是「handler 多加一个字段」那条靠一个没用到的 import 假红了一次。
  local red='test result: FAILED|^error(\[|:)|could not compile'
  case "$file" in
    *.toml) ;;
    *)
      if cargo build -p "$pkg" --tests 2>&1 | grep -qE '^error(\[|:)|could not compile'; then
        restore
        printf '⊘ 种下去的错编译不过（表达式写坏了，这一跑证明不了守卫）\n'
        skipped=$((skipped+1)); return 0
      fi
      ;;
  esac
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


# probe_cmd <组名> <要跑的命令> <文件> <sed 表达式> [确认命令]
#
# 前端那几条守卫不是 cargo 测试，是 node 脚本，所以另走一条：不认测试名，认命令的退出码。
# 代价是少了「配错对」那一栏——命令要么红要么不红，无从分辨是不是别人替它红的。
# 换来的是这一族能被探到，而在此之前它们一条也没有。
#
# 需要 :6026 与 :6027 都在应答；不在就跳过并说一声（当成绿是这类脚本最坏的坏法）。
#
# 「确认命令」是给经由 dev server 生效的种错用的：改的是磁盘上的文件，跑的却是浏览器里
# 那份模块，中间隔着 Vite 的文件监视。监视漏掉一次，页面拿到的还是旧代码，于是断言照常
# 绿——而那个绿说的是「没测到」，不是「守卫失效」。CI 上就这么报过一次假的失守。
# 给了确认命令就先轮询它，确认不上则报跳过，不下结论。
probe_cmd() {
  local group=$1 cmd=$2 file=$3 expr=$4 confirm=${5:-}
  wanted "$group" || return 0

  printf '  %-46s ' "$group"
  for port in 6026 6027; do
    if ! curl -sf -m3 "http://127.0.0.1:$port/" >/dev/null 2>&1 \
       && ! curl -sf -m3 "http://127.0.0.1:$port/api/health" >/dev/null 2>&1; then
      printf '⊘ :%s 没应答，前端这一族跑不了\n' "$port"
      skipped=$((skipped+1)); return 0
    fi
  done

  if ! plant "$file" "$expr"; then
    printf '⊘ 种下去的错没落地（表达式没匹配上）\n'; restore; skipped=$((skipped+1)); return 0
  fi

  if [ -n "$confirm" ]; then
    local ok=0 i=0
    while [ "$i" -lt 30 ]; do
      if ( cd web && eval "$confirm" ) >/dev/null 2>&1; then ok=1; break; fi
      i=$((i+1)); sleep 1
    done
    if [ "$ok" -ne 1 ]; then
      restore
      printf '⊘ 服务没在供应种下的错（dev server 的文件监视没跟上），不下结论\n'
      skipped=$((skipped+1)); return 0
    fi
  fi

  local rc=0
  ( cd web && eval "$cmd" ) >/dev/null 2>&1 || rc=$?
  restore

  if [ "$rc" -ne 0 ]; then
    printf '✓ 红了\n'; pass=$((pass+1))
  else
    printf '✗ 种了错它还是绿的\n'; fail=$((fail+1))
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

probe "架构：测试用的注册表少装了几片" mingli-registry the_delivery_layer_says_which_leaves_it_ships \
  crates/mingli-app/Cargo.toml \
  's|mingli-registry = { workspace = true, features = \["full"\] }|mingli-registry = { workspace = true, features = ["astrology", "jyotish"] }|'

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
  's|plants 20 known faults|plants 19 known faults|'

# ── 两道门 ────────────────────────────────────────────────────────
# 这一族是真出过的那种坏法：HTTP 那边补上校验，wasm 那边忘了，两扇门收的东西不一样。
probe "两道门：HTTP 少做一项 wasm 做了的校验" mingli-api the_two_doors_refuse_the_same_things \
  services/mingli-api/src/dto.rs \
  's|    mingli_app::validate_coords(req.latitude, req.longitude)|    let _ = \&req.latitude; Ok(())|'

probe "承接层：handler 往结果里加了一个字段" mingli-api natal_endpoints_pass_the_use_case_through_untouched \
  services/mingli-api/src/routes/natal.rs \
  's|Json(mingli_app::bazi::natal(&birth(&req))).into_response()|{ let mut v = serde_json::to_value(mingli_app::bazi::natal(\&birth(\&req))).unwrap_or_default(); v["probe_extra"] = serde_json::Value::Bool(true); Json(v).into_response() }|'

# ── 成本 ──────────────────────────────────────────────────────────
probe "成本：一片叶重新驮上百年推运" mingli-registry no_single_leaf_dominates_the_cost_of_casting_the_whole_tree \
  crates/mingli-astrology/src/engine.rs \
  's|        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)|        let mut v = serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null);\n        let c = chart(self, m, q);\n        v["progression"] = serde_json::to_value(crate::progression(m.jde, \&c.planets, 100, 1)).unwrap_or(Value::Null);\n        v|'

# ── 流派 ──────────────────────────────────────────────────────────
probe "流派：选项收下了却不改盘" mingli-registry every_school_option_actually_changes_the_chart \
  crates/mingli-bazi/src/engine.rs \
  's|"early_sf" => BaziSchool { zi_hour: ZiHourMethod::Early, year_break: YearBreakMethod::SpringFestival },|"early_sf" => BaziSchool::default(),|'

# ── 前端 ──────────────────────────────────────────────────────────
probe_cmd "前端：新字段没有任何一处显示" \
  'node e2e/wired.mjs' \
  web/src \
  's|aspects|aspects_probe_renamed|g'

probe_cmd "前端：渲染里又读起了时钟" \
  'node e2e/shoot.mjs 30-运势' \
  web/src/hooks/useTimeline.ts \
  's|  const nowAge = Math.max(0, Math.min(MAX_AGE, (nowMs - birthMs) / MS_PER_YEAR))|  const nowAge = Math.max(0, Math.min(MAX_AGE, (Date.now() - birthMs) / MS_PER_YEAR))|' \
  'curl -s http://127.0.0.1:6026/src/hooks/useTimeline.ts | grep -q "Date.now() - birthMs"'

printf '\n%d 条红了，%d 条配错对，%d 条一个也没拦，%d 条跳过\n' \
  "$pass" "$mismatched" "$fail" "$skipped"
[ "$fail" -eq 0 ] && [ "$skipped" -eq 0 ] && [ "$mismatched" -eq 0 ]
