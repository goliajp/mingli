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
# 备份放在仓外：目录类的种错要整棵拷贝，放在 `web/` 里会被 Vite 当成源码去监视，
# 也会在进程被强杀时把一坨 .bak 留在仓库里。每条记 "原路径<TAB>备份路径"。
BACKUP_DIR=$(mktemp -d)
declare -a BACKUPS=()

# 还原时**必须**把 mtime 推到现在，否则种下的错会以编译产物的形式留在 target 里。
#
# 备份是种错之前 `cp` 出来的，它的 mtime 早于「种错之后编出来的那份产物」。
# 直接 mv 回去，源码看上去比产物旧，cargo 判定无需重编——于是**下一次构建仍然用着
# 那份带错的库**。实测过一次：探测跑完、源码明明是好的，`cargo run` 出来的行为却是
# 种错时的样子，害我去查一个根本不存在的逻辑问题。
restore() {
  for entry in "${BACKUPS[@]:-}"; do
    [ -n "$entry" ] || continue
    orig=${entry%%$'\t'*}
    b=${entry#*$'\t'}
    if [ -d "$b" ]; then
      rm -rf "$orig"; mv "$b" "$orig"
      find "$orig" -type f -exec touch {} +
    elif [ -f "$b" ]; then
      mv "$b" "$orig"; touch "$orig"
    fi
  done
  BACKUPS=()
}

# 把种错落到一个文件或一整棵目录上。目录是给前端用的：一个字段名散在
# types.ts、视图、样式里，只改其中一个盖不住——`wired.mjs` 是全 src 搜的。
plant() {
  local target=$1 expr=$2
  local bak="$BACKUP_DIR/$(printf '%s' "$target" | tr / _)"
  if [ -d "$target" ]; then
    cp -R "$target" "$bak"; BACKUPS+=("$target"$'\t'"$bak")
    while IFS= read -r f; do sedi "$expr" "$f"; done < <(
      find "$target" -type f \( -name '*.ts' -o -name '*.tsx' -o -name '*.css' \)
    )
    ! diff -rq "$target" "$bak" >/dev/null 2>&1
  else
    cp "$target" "$bak"; BACKUPS+=("$target"$'\t'"$bak")
    sedi "$expr" "$target" 2>/dev/null || true
    ! cmp -s "$target" "$bak"
  fi
}
# 往 Cargo.toml 里种依赖时，cargo 会顺手改写 Cargo.lock —— 它从不进 BACKUPS，
# 于是一旦中途被打断，锁文件就留着种进去的那条依赖。始终单独兜住它。
cp Cargo.lock "$BACKUP_DIR/Cargo.lock"
trap 'restore; cp "$BACKUP_DIR/Cargo.lock" Cargo.lock; rm -rf "$BACKUP_DIR"' EXIT INT TERM

# probe <组名> <crate> <测试名> <文件> <sed 表达式>
probe() {
  local group=$1 pkg=$2 test=$3 file=$4 expr=$5
  wanted "$group" || return 0

  printf '  %-46s ' "$group"

  # 测试可能住在 tests/ 目录，也可能住在 src/ 里的内联 `#[cfg(test)] mod tests`。
  # 只搜前者会把后者报成「不存在」——加这条注释是因为真漏过一次。
  if ! grep -rq "fn $test\b" crates/*/tests services/*/tests crates/*/src services/*/src 2>/dev/null; then
    printf '⊘ 测试 %s 不存在（改名了？）\n' "$test"
    skipped=$((skipped+1)); return 0
  fi

  if ! plant "$file" "$expr"; then
    printf '⊘ 种下去的错没落地（表达式没匹配上）\n'; restore; skipped=$((skipped+1)); return 0
  fi

  # 清单类的种错（Cargo.toml）本来就可能编译期就被拦下——那是正当的拦法，算红。
  # 源码类的不行：`.rs` 改完编不过，说明**表达式写坏了**，那一跑测的不是守卫是语法。
  # 第一版把两者混为一谈，于是「handler 多加一个字段」那条靠一个没用到的 import 假红了一次。
  local red='test result: FAILED|^error(\[|:)|could not compile'
  case "$file" in
    *.toml) ;;
    *)
      # 先收进变量再判，**不要** `cargo … | grep -q`：本脚本开着 pipefail，
      # 那种写法的退出码取自 cargo 那一端的非零，于是「构建真的挂了」时这个 if 反而不成立——
      # 闸只在构建成功时才可能触发，等于形同虚设。第一版就是这么写的，
      # 三条源码类种错因此带着编译错跑完全程，还被记成了「✓ 红了」。
      local bo
      bo=$(cargo build -p "$pkg" --tests 2>&1 || true)
      if grep -qE '^error\[|^error: |could not compile' <<<"$bo"; then
        restore
        printf '⊘ 种下去的错编译不过（表达式写坏了，这一跑证明不了守卫）\n'
        skipped=$((skipped+1)); return 0
      fi
      ;;
  esac
  local out t0 dt
  t0=$SECONDS
  out=$(cargo test -p "$pkg" "$test" 2>&1 || true)
  dt=$((SECONDS - t0))

  if grep -qE "$red" <<<"$out"; then
    # 「怎么红的」跟「红没红」一样重要：断言红说明守卫真的在看，
    # 编译红说明是构建拦下的（清单类种错的正当拦法，源码类则该在上面的闸就被挡住）
    # 只有 `error[E….]` 与 `could not compile` 才是编译错。cargo 在**测试失败**时
    # 也会打一行 `error: test failed, to rerun pass …`——拿 `^error:` 去判会把断言红
    # 全部认成编译红，第一版正是这么把六条架构探测都标错了
    local how=断言
    grep -qE '^error\[|could not compile' <<<"$out" && how=编译
    restore; printf '✓ 红了（%s · %ss）\n' "$how" "$dt"; pass=$((pass+1)); return 0
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
# probe_script <组名> <命令> <文件> <sed 表达式>
#
# 与 probe 的差别：守卫不是某条 cargo 测试，而是一支脚本；与 probe_cmd 的差别：
# 不要求 dev server，也不在 web 下执行。
probe_script() {
  local group=$1 cmd=$2 file=$3 expr=$4
  wanted "$group" || return 0

  printf '  %-46s ' "$group"
  if ! plant "$file" "$expr"; then
    printf '⊘ 种下去的错没落地（表达式没匹配上）\n'; restore; skipped=$((skipped+1)); return 0
  fi

  local rc=0 t0 dt
  t0=$SECONDS
  eval "$cmd" >/dev/null 2>&1 || rc=$?
  dt=$((SECONDS - t0))
  restore

  if [ "$rc" -ne 0 ]; then
    printf '✓ 红了（脚本 · %ss）\n' "$dt"; pass=$((pass+1))
  else
    printf '✗ 种了错它还是绿的\n'; fail=$((fail+1))
  fi
}

# probe_cmd <组名> <命令> <文件> <sed 表达式> [确认命令] [需要的端口，缺省 "6026 6027"]
#
# 端口做成参数而不是写死两个：`wired.mjs` 与 `errors.mjs` 各自的说明里写的都是
# 「需先起好 :6027 后端」，它们碰不到 vite。写死两个的后果是——只要 dev server 没起，
# 这两条也跟着跳过，而它们本来跑得动。实测这么白跳过了四条里的三条。
probe_cmd() {
  local group=$1 cmd=$2 file=$3 expr=$4 confirm=${5:-} ports=${6:-"6026 6027"}
  wanted "$group" || return 0

  printf '  %-46s ' "$group"
  for port in $ports; do
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

  local rc=0 t0 dt
  t0=$SECONDS
  ( cd web && eval "$cmd" ) >/dev/null 2>&1 || rc=$?
  dt=$((SECONDS - t0))
  restore

  if [ "$rc" -ne 0 ]; then
    printf '✓ 红了（断言 · %ss）\n' "$dt"; pass=$((pass+1))
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
  's|crate::cast(method, effective_seed(m, q))|crate::cast(method, 42)|'

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
  's|plants [0-9][0-9]* known faults|plants 1 known faults|'

probe "自陈：叶里多了个没人问的公开函数" mingli-registry every_public_function_is_reachable_from_something_that_is_not_a_test \
  crates/mingli-ziwei/src/limit.rs \
  's|^/// 某公历年的流年宫：太岁支入宫。|/// 探测用：算了却没人问的那种函数。\n#[must_use]\npub fn annual_palace_unused(ming_branch: u8, year: i32) -> u8 { annual_palace(ming_branch, year).0 }\n\n/// 某公历年的流年宫：太岁支入宫。|'

probe "自陈：认领了「运」却没进运势用例" mingli-app every_leaf_that_claims_the_fortune_intent_shows_up_in_the_fortune_answer \
  crates/mingli-app/src/bazi.rs \
  's|        "ziwei": ziwei_at(b, t),||'

probe "自陈：认领了「合」却没进合盘用例" mingli-app every_leaf_that_claims_the_synastry_intent_shows_up_in_the_synastry_answer \
  crates/mingli-app/src/synastry.rs \
  's|        ashtakuta: ashtakuta_between(a.0, b.0),|        ashtakuta: serde_json::Value::Null,|'

probe "自陈：认领了「字」却不在字词注册表里" mingli-app every_leaf_that_claims_the_onomancy_intent_is_in_the_word_registry \
  crates/mingli-numerology/src/engine.rs \
  '/impl WordEngine/,/^}/s|^        "numerology"$|        "numerology-name"|'

probe "自陈：README 的体积表与脚本对不上" mingli-registry the_wasm_size_table_and_the_script_agree \
  README.md \
  's:| Four Pillars only | 0.57 MB |:| Four Pillars only | 0.77 MB |:'

probe "跨叶：冒出一对没人解释的完全冗余" mingli-analysis the_only_perfectly_redundant_pairs_are_the_ones_we_can_explain \
  crates/mingli-analysis/src/lib.rs \
  's|        ("bazi", "liuren", "两者的主判据都是日支——同一个量，换个名字"),||'

probe "释义：载荷里的非显然单位没人交代" mingli-interpret a_payload_field_with_a_non_obvious_unit_must_be_explained_in_the_same_prompt \
  crates/mingli-interpret/src/guardrails/synastry.rs \
  's|\*\*单位是十分之一分\*\*：`total_min_tenths` / `total_max_tenths` 除以 10 才是分数，|总分见 `total_min_tenths` / `total_max_tenths`，|'

probe "石头：飞布只在某一入中数上出错" mingli-luoshu flying_is_a_permutation_for_every_center_and_direction \
  crates/mingli-core/src/group.rs \
  's|    shift(center - 1, k, 9, forward) + 1|    shift(center - 1, k + i64::from(center == 3), 9, forward) + 1|'

probe "石头：奇门宫号越出九宫" mingli-qimen every_palace_number_on_a_chart_is_in_range \
  crates/mingli-qimen/src/cast.rs \
  's|    let zhi_fu_palace = earth_position_of_stem(&earth, zhi_fu_stem_name);|    let zhi_fu_palace = earth_position_of_stem(\&earth, zhi_fu_stem_name) + 1;|'

probe "石头：农历某岁的月界偏一天" mingli-astro the_lunar_sequence_has_no_seams_across_two_centuries \
  crates/mingli-astro/src/lunar.rs \
  's|    let n_months = nm_cdn.len() - 1;|    if start_year == 1935 \&\& nm_cdn.len() > 4 { nm_cdn[3] += 1; }\n    let n_months = nm_cdn.len() - 1;|'

probe "石头：年干支偏一位" mingli-ganzhi year_pillar \
  crates/mingli-ganzhi/src/cycle.rs \
  's|pub fn year_ganzhi(solar_year: i32) -> GanZhi {|pub fn year_ganzhi(solar_year: i32) -> GanZhi { let solar_year = solar_year + 1;|'

probe "石头：取机发生器换了一套算术" mingli-core splitmix64_matches_the_published_reference_vectors \
  crates/mingli-core/src/sampler.rs \
  's|        z \^ (z >> 31)|        z \& (z >> 31)|'

probe "石头：地占的派生整条塌成常量" mingli-core the_derived_figures_are_pinned_to_concrete_values \
  crates/mingli-core/src/gf2.rs \
  's|^    a \^ b$|    a \& b|'

probe "干支：五虎遁只有庚年是对的" mingli-ganzhi wuhu_dun \
  crates/mingli-ganzhi/src/cycle.rs \
  's|((year_stem % 5) \* 2 + 2)|((year_stem / 5) * 2 + 2)|'

probe "干支：神煞把不该命中的也报上来" mingli-ganzhi a_day_stem_shensha_lookup_names_only_the_ones_that_hit \
  crates/mingli-ganzhi/src/shensha.rs \
  's|if HONGYAN\[day_stem as usize\] == branch|if HONGYAN[day_stem as usize] != branch|'

probe "石头：朔的时刻整体挪了几分钟" mingli-astro the_new_moon_instants_match_two_published_ephemerides \
  crates/mingli-astro/src/moon.rs \
  's|    2451550.09766 + 29.530588861|    2451550.09966 + 29.530588861|'

probe "石头：行星位置不再扣光行时" mingli-ephemeris every_planet_matches_the_positions_jpl_publishes_across_a_century \
  crates/mingli-ephemeris/src/lib.rs \
  's|                tau = LIGHT_TIME_PER_AU \* dist;|                tau = 0.0 * dist;|'

probe "石头：行星理论掉出独立星历的量级" mingli-ephemeris at_j2000_every_planet_matches_an_independent_theory_to_the_arcsecond \
  crates/mingli-ephemeris/src/lib.rs \
  's|            for _ in 0..3 {|            for _ in 0..1 {|'

probe "易经：六爻上下颠倒" mingli-yijing what_is_reported_is_the_hexagram_the_lines_actually_make \
  crates/mingli-yijing/src/lib.rs \
  's|            prim \|= 1 << i;|            prim \|= 1 << (5 - i);|'

probe "易经：上下卦名对调" mingli-yijing what_is_reported_is_the_hexagram_the_lines_actually_make \
  crates/mingli-yijing/src/lib.rs \
  's|        primary_upper: primary.upper().name(),|        primary_upper: primary.lower().name(),|'

probe "易经：变爻掩码错位" mingli-yijing what_is_reported_is_the_hexagram_the_lines_actually_make \
  crates/mingli-yijing/src/lib.rs \
  's|            mask \|= 1 << i;|            mask \|= 1 << (5 - i);|'

probe "缅历：纪元偏一年" mingli-mahabote the_year_number_follows_the_era_epoch_across_two_centuries \
  crates/mingli-mahabote/src/lib.rs \
  's|pub const EPOCH_OFFSET: f64 = 1_954_168.050_623;|pub const EPOCH_OFFSET: f64 = 1_954_533.050_623;|'

probe "缅历：新年挪出四月" mingli-mahabote the_year_number_advances_once_a_year_in_april \
  crates/mingli-mahabote/src/lib.rs \
  's|pub const EPOCH_OFFSET: f64 = 1_954_168.050_623;|pub const EPOCH_OFFSET: f64 = 1_954_268.050_623;|'

probe "缅历：宫名取错核心数" mingli-mahabote what_compute_reports_hangs_together \
  crates/mingli-mahabote/src/lib.rs \
  's|        house: HOUSES\[core\],|        house: HOUSES[(core + 1) % 7],|'

probe "巴厘：八曜正常段落成卡日" mingli-pawukon the_stuck_day_weeks_agree_with_the_reference_closed_form \
  crates/mingli-pawukon/src/lib.rs \
  '92s|if day < 71 {|if day == 71 {|'

probe "玛雅：无名五日全报第一日" mingli-maya haab_wraps_365_and_covers_wayeb \
  crates/mingli-maya/src/lib.rs \
  's|        ((doy - 360) as u8, 18)|        ((doy / 360) as u8, 18)|'

probe "叶的头条数换成另一个量" mingli-registry every_leaf_reports_the_principal_it_is_supposed_to \
  crates/mingli-maya/src/engine.rs \
  's|value: c.tzolkin_number.to_string()|value: c.tzolkin_name.to_string()|'

probe "梅花：下卦不再加时支" mingli-meihua the_source_example_comes_out_of_the_real_entry_point \
  crates/mingli-meihua/src/lib.rs \
  's|    let with_hour = base + u32::from(hb);|    let with_hour = base.wrapping_sub(u32::from(hb));|'

probe "藏历：历日卦塌成常量" mingli-tibetan the_calendar_day_trigram_cycles_with_the_julian_day \
  crates/mingli-tibetan/src/lib.rs \
  's|    amod(jdn + 2, 8)|    { let _ = jdn; 1 }|'

probe "藏历：历日卦偏四位" mingli-tibetan the_calendar_day_trigram_cycles_with_the_julian_day \
  crates/mingli-tibetan/src/lib.rs \
  's|    amod(jdn + 2, 8)|    amod(jdn - 2, 8)|'

probe "择日：等第标签改掉" mingli-zeri day_grades_follow_the_mnemonic \
  crates/mingli-zeri/src/lib.rs \
  's|            DayGrade::Huang => "黄道",|            DayGrade::Huang => "xyzzy",|'

probe "四柱：均时差的号写反" mingli-bazi the_equation_of_time_matches_two_published_tables_at_its_extremes \
  crates/mingli-bazi/src/solar.rs \
  's|    9.87 \* (2.0 \* b).sin() - 7.53 \* b.cos()|    9.87 * (2.0 * b).sin() + 7.53 * b.cos()|'

probe "四柱：均时差的日序基准挪了" mingli-bazi the_equation_of_time_crosses_zero_four_times_a_year \
  crates/mingli-bazi/src/solar.rs \
  's|(n - 81.0) / 365.0|(n + 81.0) / 365.0|'

probe "四柱：经度不再按一度四分钟" mingli-bazi one_degree_of_longitude_is_exactly_four_minutes \
  crates/mingli-bazi/src/solar.rs \
  's|    let geo_correction = (longitude - std_longitude) \* 4.0;|    let geo_correction = (longitude - std_longitude) * 3.0;|'

probe "四柱：起运折算不再除以三" mingli-bazi the_starting_age_is_the_days_to_the_adjacent_jie_divided_by_three \
  crates/mingli-bazi/src/chart.rs \
  's|    let start_age_years = (days / 3.0).max(0.0);|    let start_age_years = (days / 4.0).max(0.0);|'

probe "四柱：起运数到中气而非节" mingli-bazi the_starting_age_is_the_days_to_the_adjacent_jie_divided_by_three \
  crates/mingli-bazi/src/chart.rs \
  's|    let k = ((lam - 15.0) / 30.0).floor();|    let k = ((lam - 30.0) / 30.0).floor();|'

probe "四柱：顺逆搞反" mingli-bazi the_direction_follows_the_year_stem_and_the_gender \
  crates/mingli-bazi/src/chart.rs \
  's|        Gender::Male => year_yang,|        Gender::Male => !year_yang,|'

probe "六壬：八专刚日不再连本位数" mingli-liuren the_three_rare_courses_transmit_the_way_the_books_say \
  crates/mingli-liuren/src/transmission.rs \
  's|            (courses\[0\].up + 2) % 12|            (courses[0].up + 3) % 12|'

probe "六壬：昴星柔日改仰视" mingli-liuren the_three_rare_courses_transmit_the_way_the_books_say \
  crates/mingli-liuren/src/transmission.rs \
  's|        ((9 + 12 - offset) % 12, courses\[0\].up, courses\[2\].up)|        (heaven_plate(9, offset), courses[0].up, courses[2].up)|'

probe "六壬：别责柔日不取支前三合" mingli-liuren the_three_rare_courses_transmit_the_way_the_books_say \
  crates/mingli-liuren/src/transmission.rs \
  's|            (day_branch + 4) % 12|            (day_branch + 3) % 12|'

probe "六壬：昴星末传不再归干" mingli-liuren the_three_rare_courses_transmit_the_way_the_books_say \
  crates/mingli-liuren/src/transmission.rs \
  's|        (heaven_plate(9, offset), courses\[2\].up, courses\[0\].up)|        (heaven_plate(9, offset), courses[0].up, courses[2].up)|'

probe_script "两门：wasm 与 native 算出不同的盘" \
  './scripts/wasm-parity.sh' \
  crates/mingli-meihua/src/lib.rs \
  's|    let base = u32::from(yb) + month + day;|    let base = u32::from(yb) + month + day + u32::from(cfg!(target_arch = "wasm32"));|'

probe_script "两门：两边比的不是同一批输入" \
  './scripts/wasm-parity.sh' \
  scripts/wasm-cast.mjs \
  's|  \[1990, 6, 15, 14, 30\],|  [1991, 6, 15, 14, 30],|'

probe_script "两档：优化改变了排出来的盘" \
  './scripts/profile-parity.sh' \
  crates/mingli-meihua/src/lib.rs \
  's|    let base = u32::from(yb) + month + day;|    let base = u32::from(yb) + month + day + u32::from(cfg!(debug_assertions));|'

probe "占星：Placidus 象限接错" mingli-astrology the_four_quadrants_of_asc1_join_up_without_a_seam \
  crates/mingli-astrology/src/placidus.rs \
  's|        3 => 180.0 + asc2(x1 - 180.0, -f, sine, cose),|        3 => 180.0 + asc2(x1 - 180.0, f, sine, cose),|'

probe "占星：asc2 的快路取值变了" mingli-astrology asc2_quadrant_sanity \
  crates/mingli-astrology/src/placidus.rs \
  's|        out = if sin_x < 0.0 { -90.0 } else { 90.0 };|        out = if sin_x < 0.0 { -90.0 } else { 89.0 };|'

probe_script "装配：类型化出口又拖上了 serde" \
  "bash scripts/leaf-deps.sh yijing" \
  crates/mingli-yijing/Cargo.toml \
  's@^serde = { workspace = true, optional = true }$@serde = { workspace = true }@'

probe_script "装配：单叶档混进了别的叶" \
  "bash scripts/leaf-isolation.sh yijing" \
  crates/mingli-wasm/Cargo.toml \
  's@^yijing = \["mingli-registry/yijing"\]$@yijing = ["mingli-registry/yijing", "mingli-registry/bazi"]@'

probe_script "装配：产物胖了没人拦" \
  "bash scripts/wasm-size.sh chart-solo-yijing" \
  Cargo.toml \
  's@^lto = "thin"$@lto = false@'

probe "六十四卦：八卦符号取错格" mingli-gua every_trigram_has_its_own_name_symbol_and_number \
  crates/mingli-gua/src/lib.rs \
  's@TRIGRAM_SYMBOLS\[(self.0 \& 0b111) as usize\]@TRIGRAM_SYMBOLS[(self.0 | 0b111) as usize]@'

probe "六十四卦：爻位读错" mingli-gua lines_bottom_up \
  crates/mingli-gua/src/lib.rs \
  's@\*slot = (self.0 >> i) \& 1 == 1;@*slot = (self.0 >> 1) \& 1 == 1;@'

probe "六十四卦：卦象字认宽了" mingli-gua nothing_but_those_sixteen_characters_is_accepted \
  crates/mingli-gua/src/lib.rs \
  's@if b0 == 0xE5 \&\& b1 == 0xA4 \&\& b2 == 0xA9@if b0 == 0xE5 \&\& b1 == 0xA4 || b2 == 0xA9@'

probe "六十四卦：纯卦判定认宽了" mingli-gua the_pure_hexagram_test_keys_on_the_character_not_a_byte \
  crates/mingli-gua/src/lib.rs \
  's@if b\[3\] == 0xE4 \&\& b\[4\] == 0xB8 \&\& b\[5\] == 0xBA@if b[3] == 0xE4 || b[4] == 0xB8 \&\& b[5] == 0xBA@'

probe "编排：目录与路由分了岔" mingli-registry the_catalogue_advertises_exactly_what_the_router_delivers \
  crates/mingli-engine/src/lib.rs \
  's@.filter(|e| e.answers().contains(&spec.id))@.filter(|e| e.answers().contains(\&mingli_contract::Intent::Natal))@'

probe "契约：主体展示名被改字" mingli-contract every_arm_of_every_enum_says_something_distinct \
  crates/mingli-contract/src/query.rs \
  's|            Self::Company => "公司/组织",|            Self::Company => "公司",|'

probe "契约：家族标签被改字" mingli-contract every_label_is_exactly_what_the_api_ships \
  crates/mingli-contract/src/declare.rs \
  's|            Family::Angular => "角度量化",|            Family::Angular => "角度",|'

probe "契约：两族撞了同一个标签" mingli-contract every_label_is_exactly_what_the_api_ships \
  crates/mingli-contract/src/declare.rs \
  's|            Family::Hashing => "哈希环",|            Family::Hashing => "角度量化",|'

probe "占星：宫尖归到前一宫" mingli-astrology house_of_basic_assignment \
  crates/mingli-astrology/src/placidus.rs \
  's|        if off < span {|        if off <= span {|'

probe "占星：Porphyry 的 IC 算错" mingli-astrology porphyry_lower_arc_trisects_from_the_ic \
  crates/mingli-astrology/src/placidus.rs \
  's|    let ic = norm360(mc + 180.0);|    let ic = norm360(mc * 180.0);|'

probe "占星：收敛判据恒为零" mingli-astrology signed_diff_deg_is_the_short_way_round_and_keeps_its_sign \
  crates/mingli-astrology/src/placidus.rs \
  's|    let d = (a - b).rem_euclid(360.0);|    let d = 0.0 * (a - b).rem_euclid(360.0);|'

probe "印占：大运的儒略日算错" mingli-jyotish the_julian_days_tile_the_timeline_at_every_year_length \
  crates/mingli-jyotish/src/dasha.rs \
  's|            start_jd: birth_jd_ut + age \* days_per_year,|            start_jd: birth_jd_ut + age * days_per_year * 1.001,|'

probe "印占：子运不再铺满本段" mingli-jyotish the_julian_days_tile_the_timeline_at_every_year_length \
  crates/mingli-jyotish/src/dasha.rs \
  's|        let years = span_years \* sub_years / 120.0;|        let years = span_years * sub_years / 121.0;|'

probe "印占：Tara 相隔数不再从一起数" mingli-jyotish tara_counts_the_stars_between_and_calls_three_of_every_nine_bad \
  crates/mingli-jyotish/src/kuta.rs \
  's|    (to + 27 - from) % 27 + 1|    (to + 27 - from) % 27|'

probe "印占：Tara 的凶位挪了一个" mingli-jyotish tara_counts_the_stars_between_and_calls_three_of_every_nine_bad \
  crates/mingli-jyotish/src/kuta.rs \
  's|    matches!(step % 9, 3 \| 5 \| 7)|    matches!(step % 9, 3 \| 5 \| 8)|'

probe "四柱：春节换岁退错年" mingli-bazi the_spring_festival_year_turns_once_a_year_on_the_first_of_the_first_month \
  crates/mingli-bazi/src/chart.rs \
  's|        YearBreakMethod::SpringFestival => m.lunar.year,|        YearBreakMethod::SpringFestival => if m.lunar.month >= 11 { m.year - 1 } else { m.year },|'

probe "四柱：两派在下半年岔开" mingli-bazi the_two_year_break_schools_only_disagree_early_in_the_year \
  crates/mingli-bazi/src/chart.rs \
  's|        YearBreakMethod::SpringFestival => m.lunar.year,|        YearBreakMethod::SpringFestival => m.lunar.year - 1,|'

probe "六壬：孟仲季三档塌成一档" mingli-liuren the_three_ranks_read_the_ground_position_not_the_god_standing_on_it \
  crates/mingli-liuren/src/transmission.rs \
  's|        2 \| 8 \| 5 \| 11 => 0,|        2 \| 8 \| 5 \| 11 => 1,|'

probe "六壬：涉害改看天盘神" mingli-liuren the_shehai_ladder_narrows_at_every_rung \
  crates/mingli-liuren/src/transmission.rs \
  's|    let best = pool.iter().map(\|c\| meng_zhong_ji(c.down)).min().unwrap_or(2);|    let best = pool.iter().map(\|c\| meng_zhong_ji(c.up)).min().unwrap_or(2);|'

probe "六壬：比用取反了阴阳" mingli-liuren the_shehai_ladder_narrows_at_every_rung \
  crates/mingli-liuren/src/transmission.rs \
  's|filter(\|c\| branch_is_yang(c.up) == yang)|filter(\|c\| branch_is_yang(c.up) != yang)|'

probe "印占：Bhakoot 相隔位次算错" mingli-jyotish bhakoot_agrees_in_both_directions_at_every_pair_of_signs \
  crates/mingli-jyotish/src/kuta.rs \
  's|    let d1 = (gr + 12 - br) % 12 + 1;|    let d1 = (gr * 12 - br) % 12 + 1;|'

probe "印占：未定项数数反了" mingli-jyotish the_unsettled_count_is_exactly_the_items_marked_unsettled \
  crates/mingli-jyotish/src/kuta.rs \
  's|filter(\|k\| !k.settled).count()|filter(\|k\| k.settled).count()|'

probe "印占：主星查表取错位次" mingli-jyotish graha_maitri_reads_the_lords_of_the_two_signs \
  crates/mingli-jyotish/src/kuta.rs \
  's|        .position(\|x\| \*x == lord)|        .position(\|x\| *x != lord)|'

probe "择日：上限那天被多拒了一天" mingli-app window_bounds_are_checked \
  crates/mingli-app/src/election.rs \
  's|    if days > MAX_DAYS {|    if days >= MAX_DAYS {|'

probe "择日：月长表少一档" mingli-app day_stepping_crosses_months_and_leap_years \
  crates/mingli-app/src/lib.rs \
  's|        1 \| 3 \| 5 \| 7 \| 8 \| 10 \| 12 => 31,|        1 \| 3 \| 5 \| 7 \| 8 \| 12 => 31,|'

probe "择日：百年不闰那一支丢了" mingli-app day_stepping_crosses_months_and_leap_years \
  crates/mingli-app/src/lib.rs \
  's|        2 if year % 4 == 0 && (year % 100 != 0 \|\| year % 400 == 0) => 29,|        2 if year % 4 == 0 => 29,|'

probe "入参：时越界只在分也越界时才拦" mingli-app an_hour_or_a_minute_out_of_range_is_refused_rather_than_rolled_over \
  crates/mingli-app/src/lib.rs \
  's|    if hour > 23 \|\| minute > 59 {|    if hour > 23 \&\& minute > 59 {|'

probe "入参：时的上界放宽一格" mingli-app an_hour_or_a_minute_out_of_range_is_refused_rather_than_rolled_over \
  crates/mingli-app/src/lib.rs \
  's|    if hour > 23 \|\| minute > 59 {|    if hour > 24 \|\| minute > 59 {|'

probe "跨叶：常量列的熵写出去变成 -0" mingli-analysis entropy_known \
  crates/mingli-analysis/src/lib.rs \
  's|^        + 0.0$|        - 0.0|'

probe "用例：请求真太阳时却给钟表时" mingli-app true_solar_time_actually_takes_the_corrected_path_when_it_can \
  crates/mingli-app/src/bazi.rs \
  's|        (true, Some(lon)) => mingli_bazi::compute_with_true_solar(input, lon),|        (true, Some(_lon)) => mingli_bazi::compute(input),|'

probe "承接层：释义不说是谁说的" mingli-api the_backend_field_names_whoever_actually_spoke \
  services/mingli-api/src/backend.rs \
  's|            Self::Offline => mingli_interpret::Template.backend(),|            Self::Offline => ClaudeCli.backend(),|'

# 基准取自 HEAD（已提交的那一版），与工作树比——种在工作树上的错正好落在被比的一侧。
probe_script "契约：拒绝的措辞悄悄改了" \
  './scripts/contract-drift.sh' \
  crates/mingli-app/src/lib.rs \
  's|return Err("month 须 1–12".into());|return Err("month 须在 1 到 12 之间".into());|'

# 种的是「[features] 段换了写法」——推导取空，逐叶单装那一整段就会跑零次。
# 守卫要在此处出声，而不是安静地跑完什么也没验。
probe_script "可裁：逐叶名单推空了却不出声" \
  './scripts/feature-matrix.sh' \
  crates/mingli-registry/Cargo.toml \
  's|^\([a-z][a-z0-9_-]*\) = \[|  \1 = [|'

# 覆盖率那支不在 CI 里（llvm-cov 一趟十来分钟），故这条也慢。种的是「判词过期」——
# 文件补上测试爬过门槛后判词没撤，理由随之作废而没人知道。
probe_script "覆盖：判词过期了却还留着" \
  './scripts/coverage.sh' \
  scripts/coverage.sh \
  's|    "crates/mingli-contract/src/declare.rs": "同 intent.rs：`const fn s` 只在编译期求值",|    "crates/mingli-contract/src/declare.rs": "同 intent.rs：`const fn s` 只在编译期求值",\n    "crates/mingli-core/src/gf2.rs": "过期判词",|'

# 种的是「一种数法只匹配到一部分」——非零但错，零产出的下限拦不住它，
# 而 --fix 会把错数直接写回 README。两种数法交叉对账要在此处出声。
probe_script "计数：一种数法坏了却照样写回" \
  './scripts/test-count.sh' \
  scripts/test-count.sh \
  's|ok\\. \[0-9\]+ passed|ok\\. 1[0-9]+ passed|'

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
  's|        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)|        let c = chart(self, m, q);\n        let mut v = serde_json::to_value(\&c).unwrap_or(Value::Null);\n        v["progression"] = serde_json::to_value(crate::progression::progression(m.jde, \&c.planets, 100, 1)).unwrap_or(Value::Null);\n        v|'

probe "成本：一片普通叶开始干重活" mingli-registry the_expensive_leaves_are_exactly_the_ones_that_walk_an_ephemeris \
  crates/mingli-yijing/src/engine.rs \
  's|    crate::cast(method, effective_seed(m, q))|    let mut w = 0f64;\n    for i in 0..40_000u32 { w += f64::from(i).sqrt(); }\n    crate::cast(method, effective_seed(m, q).wrapping_add(u64::from(w < 0.0)))|'

# ── 流派 ──────────────────────────────────────────────────────────
probe "流派：选项收下了却不改盘" mingli-registry every_school_option_actually_changes_the_chart \
  crates/mingli-bazi/src/engine.rs \
  's|"early_sf" => BaziSchool { zi_hour: ZiHourMethod::Early, year_break: YearBreakMethod::SpringFestival },|"early_sf" => BaziSchool::default(),|'

# ── 前端 ──────────────────────────────────────────────────────────
probe_cmd "前端：新字段没有任何一处显示" \
  'node e2e/wired.mjs' \
  web/src \
  's|aspects|aspects_probe_renamed|g' \
  "" 6027

# 种的是「认变量的正则失效」——文件照样扫到，变量一个也认不出，
# 两边都空于是对账天然「平」。守卫要在此处出声，而不是报一纸清白。
probe_cmd "前端：CSS 变量的尺子坏了却报平" \
  'node e2e/css-vars.mjs' \
  web/e2e/css-vars.mjs \
  's|matchAll(/var\\((--\[\\w-\]+)/g)|matchAll(/varX\\((--[\\w-]+)/g)|' \
  "" ""

probe_cmd "前端：等第名两边各说各的" \
  'node e2e/wired.mjs' \
  web/src/views/ElectionView.tsx \
  "s|{ key: 'Huang', label: '黄道', note: '除 · 危 · 定 · 执' },|{ key: 'Huang', label: '黄道日', note: '除 · 危 · 定 · 执' },|" \
  "" 6027

# 种的是「某个字段退化成只写在 types.ts 里」——把界面上用它的那一处换成别的字段。
# （种「把 types.ts 重新算进认领面」是不行的：那是放松判据，放松只会更绿。）
probe_cmd "前端：字段只写在类型里没人用" \
  'node e2e/wired.mjs' \
  web/src/views/leaves/Jyotish.tsx \
  's|c.lagna_navamsa_name|c.lagna_rasi_name|g' \
  "" 6027

probe_cmd "前端：把「你输错了」说成连不上" \
  'node e2e/errors.mjs' \
  web/src/App.tsx \
  's|      {err \&\& <div className="err">⚠ {err}</div>}|      {err \&\& <div className="err">⚠ {err}（服务连接失败，请稍后重试）</div>}|' \
  "" 6027

probe_cmd "前端：内容断言被悄悄摘掉一条" \
  'node e2e/shoot.mjs' \
  web/e2e/shoot.mjs \
  "s|^  '21-数字学': async (page) => {|  '21-数字命理学': async (page) => {|"

probe_cmd "前端：渲染里又读起了时钟" \
  'node e2e/shoot.mjs 30-运势' \
  web/src/hooks/useTimeline.ts \
  's|  const nowAge = Math.max(0, Math.min(MAX_AGE, (nowMs - birthMs) / MS_PER_YEAR))|  const nowAge = Math.max(0, Math.min(MAX_AGE, (Date.now() - birthMs) / MS_PER_YEAR))|' \
  'curl -s http://127.0.0.1:6026/src/hooks/useTimeline.ts | grep -q "Date.now() - birthMs"'

printf '\n%d 条红了，%d 条配错对，%d 条一个也没拦，%d 条跳过\n' \
  "$pass" "$mismatched" "$fail" "$skipped"
[ "$fail" -eq 0 ] && [ "$skipped" -eq 0 ] && [ "$mismatched" -eq 0 ]
