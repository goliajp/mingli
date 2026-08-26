# mingli —— 世界命理/占卜系统的可计算内核

[English](README.md)

[![CI](https://github.com/goliajp/mingli/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/goliajp/mingli/actions/workflows/ci.yml)

　　把全世界的术数当作**算法**来实现：确定性排盘引擎（Rust）+ axum 服务 + React 19 前端。

　　核心原则是**「算 / 释 / 说」三层分离**——本仓库只做「算」：可复现、可校验、可证的纯计算。
「这意味着什么」属于释义层，被显式隔离在 `mingli-interpret` 之后，且永远标记为非计算产物。

> 38 个 crate · 24 片叶（21 片时刻叶走并行 fan-out，4 片字词叶走 `/api/word`，其中一片两边都答）· 8 类问局（HTTP 与 wasm 都接）· 788 个测试全绿
> `unsafe_code = "forbid"` · `missing_docs = "deny"` · `clippy::all = "deny"`

---

## 一张图

```
            叶 LEAVES  —— 每个术数一片叶，产出一张盘 / 一个结果
  八字 紫微 奇门 择日 │ 西洋占星 Jyotish 七政四余 │ 易经 地占 Ifá Sikidy 塔罗 │ 数字学 五格 │ 玛雅 藏历 …
            枝 BRANCHES —— 4 家族 + 1 横切（计算范式）
     A 循环群 / CRT  │  B 角度量化  │  C 抽样 / 二进制  │  D 哈希环  │  ⟂ 飞布 / 置换
            主干 TRUNK  —— 共享符号零件（L2）
        干支 │ 六十四卦 │ 九宫洛书 │ 二十八宿 / 黄道 │ 16 图 / 256 odu
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            根 ROOTS —— 数学石 + 物理石
 物理石 L1：儒略日·ΔT │ 太阳黄经→节气 │ 月相→朔→农历 │ 行星星历 │ 章动 / 岁差 / ayanamsa
 数学石 L0：Cyclic+CRT │ Quantizer │ BinaryLattice+GF(2) │ RingHash │ GroupAction │ SeededSampler
```

　　每片叶子 = 「选根的哪几块石头 × 主干的哪个符号零件 × 哪条枝（家族范式）」的组装。**长新叶不动根。**

　　依赖只向内：每片叶实现 `mingli-contract` 声明的端口，编排层只消费端口、不认识任何叶，装配根是唯一列出全部叶的地方。这条规则由测试守着（`crates/mingli-registry/tests/architecture.rs` 读取每份 manifest，一旦出现朝外的依赖就失败）。

---

## 确定性谱

　　每个节点带一个确定性标签，决定它由谁执行、可信到什么程度。**盘面永远是真的，不确定只活在边缘（UND）和释义（INT）。**

| 标签 | 含义 | 执行者 | 例 |
|---|---|---|---|
| 🟢 **DET** 确定 | 纯数学 / 物理，可复现、可证 | Rust | 节气、干支、卦的二进制、五虎遁 |
| 🎲 **STO** 随机 | 真随机起卦，但种子留痕 → 可复现 | Rust | C 族抽签（蓍草 / 三钱 / 塔罗 / 地占） |
| 🟡 **UND** 欠定 | 流派分歧 / 缺权威常数 / 查表未核 | 配置开关 | 紫炁历元、81 数理版本、子时换日、orb 流派 |
| 🔮 **INT** 释义 | 把盘面翻成人话 | LLM | 所有「这意味着什么」 |

　　每片叶子通过 `CastingEngine::profile()` 显式声明自己的 DET/STO/UND 边界——**流派分歧不臆造，宁可返回 `None` 并标 UND**。

---

## 结构

```
crates/
  L0 数学根    mingli-core         有限循环群 / CRT / 量化器 / GF(2) 格 / 哈希环 / 群作用 / 种子采样
  L1 物理石    mingli-astro        儒略日 · 太阳视黄经与二十四节气 · 月相与农历置闰 · 干支
               mingli-ephemeris    行星星历——日月五星的地心黄道经度（VSOP87 / ELP-2000）· 上升点与中天几何
  L2 端口      mingli-contract     CastingEngine / WordEngine 契约 + 共享查询与声明类型
     主干      mingli-ganzhi       六十干支符号系统
               mingli-gua          六十四卦格 (Z₂)⁶
               mingli-luoshu       洛书幻方与九宫飞布
  L3 叶（A 族 · 循环群 / CRT）
               mingli-maya         玛雅历法的三套互锁计数
               mingli-pawukon      巴厘岛 Pawukon 历
               mingli-mahabote     缅甸 Mahabote 本命核心数
               mingli-tibetan      藏历的循环要素
               mingli-zeri         择日学的循环要素
               mingli-xiaoliuren   小六壬（诸葛马前课）
  L3 叶（B 族 · 角度量化）
               mingli-astrology    西洋占星本命盘（Placidus / Koch / WholeSign / Equal / Porphyry）· 两盘相位
               mingli-jyotish      印度占星（4 派 ayanamsa · 27 nakshatra · Vimshottari · 十六分盘取十四）
               mingli-qizhengsiyu  七政四余——中国本土星占
  L3 叶（C 族 · 抽样 / 二进制）
               mingli-yijing       易经起卦
               mingli-geomancy     地占 ʿilm al-raml
               mingli-sikidy       马达加斯加 Sikidy
               mingli-ifa          西非约鲁巴 Ifá
               mingli-cartomancy   抽牌占卜（塔罗 / Lenormand / Futhark 共 5 种 Deck）
               mingli-meihua       梅花易数·多起卦法（C 族中的确定性子类）
  L3 叶（D 族 · 哈希环）
               mingli-numerology   西洋数字学（Pythagorean + Chaldean）
               mingli-gematria     希伯来 gematria 7 计法
               mingli-abjad        阿拉伯 abjad 数字（Mashriqī / Maghribī 双序）
               mingli-wuge         姓名五格剖象法（熊崎式）
  L3 叶（⟂ 横切 · 飞布 / 置换）
               mingli-bazi         四柱推命——四柱 · 十神 · 大运 · 旺衰 · 用神 · 岁运叠加
               mingli-ziwei        紫微斗数——命宫身宫 · 五行局 · 十四主星 · 四辅星 · 四化
               mingli-liuren       大六壬起课
               mingli-qimen        奇门遁甲（时家转盘法）
               mingli-taiyi        太乙神数
  L4 编排      mingli-engine       共享上下文记忆化 + 并行 fan-out；注册表由外部注入，本层不认识任何叶
               mingli-interpret    释义层——组装带护栏的提示词，与「算」严格分离
  L5 分析      mingli-analysis     跨叶信息论统计
  L6 用例      mingli-app          八类问局各一个用例——命 · 运 · 事 · 择 · 寻 · 合 · 群国 · 号——以及两条交付路共用的入参形状
  L7 装配      mingli-registry     唯一知道「树上有哪些叶」的地方——加新叶在此登记一行
  L8 交付      mingli-wasm         wasm 绑定——八类问局全接，入参与 HTTP 端点同形
services/
  mingli-api/   axum 承接层，:6027
web/            React 19 + Vite 前端，:6026（dev 期 /api 代理到 :6027）
```

---

## 跑起来

```bash
# 1) 后端（:6027）
cargo run -p mingli-api

# 2) 前端（:6026，另开终端）
cd web && bun install && bun run dev
# 打开 http://localhost:6026
```

## 看一眼

```bash
cd web && bun run shots     # 先对 CSS 变量，再用 headless Chromium 走一遍界面
```

　　30 屏（21 片叶 + 横切页 + 各意图页）× 两个视口（1440 与 1024），产物在 `web/e2e/shots/<宽度>/`。
除汇总 console 报错、页面异常与失败请求外，八屏带断言：九宫恰 9 格、分档标题自报的天数等于表里行数、
罗盘 pin 数等于列表行数、某曜的三分盘落宫与本命宫只相隔 0 / 4 / 8 宫。收尾另有一条：无人操作的三秒里
还在发请求，就是有东西在自循环。任一条不过即非零退出。跑之前先对一遍 CSS 变量——用到的有没有来源、
定义的有没有人读。

　　拍屏之前另有一遍：拿 `/api/cast` 的返回逐字段去核 `web/src` 下的全部源码——
后端算出来而前端一处都没提过的字段，就是没人看得见的字段。这条检查立起来之前，已经这么漏过五个。

　　类型检查证明代码编得过，这个证明画面还自洽。

## 只取你要的那几片

　　每套术数是一个独立 crate，装配根按叶 id 逐片开关。不必扛着二十四片。

```bash
cargo add mingli-bazi                     # 单 crate：serde + 三个共享层，不含星历
cargo add mingli-registry --no-default-features --features bazi,yijing   # 两片，走统一端口
```

　　实测（release，wasm32）：

| 装配 | 体积 |
|---|---|
| 一片不装（纯骨架） | 0.53 MB |
| 只要四柱 | 0.57 MB |
| 四柱 + 紫微 | 0.60 MB |
| 只要西洋占星 | 1.32 MB |
| 全部二十四片 | 1.83 MB |

　　一片叶在骨架之上约 0.05 MB；三片带行星星历的叶合计 0.87 MB。
`feature-matrix.sh` 会把每片叶各单独装配一次——某片悄悄拖进另一片，会在那里红，而不是在你的产物里。
它还会把 38 个 crate 各自单独跑一遍测试：`cargo test --workspace` 跑的是**合并后**的 feature 集，
一个 crate 的测试依赖少写了 feature，整仓一起跑照样绿，单独跑才炸。

　　排盘的成本集中在**要走行星星历的那三片**。本机实测：西洋占星约 300 µs、印度占星 250 µs、
七政四余 220 µs；四柱 9 µs、紫微 7 µs；其余十六片各不足 5 µs。守卫验的是**形状**而非微秒数
（后者只属于这台机器）：恰好这三片比中位数那片贵两个数量级——某片叶开始走星历会在这里现形，
某片不再走了同样会。
另有一条守卫：**任一片叶占全树排盘耗时或载荷超过 60% 即红**——它是因为真出过一次才立的。

　　守卫自己也要被验。一条永远绿的守卫和一条真守着东西的守卫，在日常测试里长得一模一样；
分辨它们只有一个办法——把它该拦的东西种回去，看它拦不拦。`guard-probe.sh` 把这件事从
「我当时手工试过」变成一条能重跑的命令：种 82 个已知的错，逐条问该拦它的守卫红没红。
它上一次就抓到一处名不副实——「装配根是唯一列叶的地方」那条，其实并不看释义层。


## 测试 / 校验

```bash
cargo test --workspace     # 788 个测试
cargo clippy --workspace   # deny-clean
cargo doc --workspace      # 全文档
./scripts/coverage.sh      # 低于门槛的文件必须逐个写明理由
./scripts/api-snapshot.sh check snap.txt   # 39 个请求逐字节
./scripts/test-count.sh    # 本文自称的测试数，对回真跑一遍的结果
./scripts/feature-matrix.sh  # 每片叶各单独装配、每个 crate 各单独跑一次测试 + wasm32 + 查依赖图
./scripts/guard-probe.sh   # 种 82 个已知的错，看该拦它的那条守卫拦不拦
```

　　以上连同截图断言，每次 push 都会跑一遍——见顶部徽章。

　　引擎校验的权威参照值均经**多源交叉确认**，全部落在各 crate 的 `#[test]` 里，例如：

- 日干支锚点 2024-01-01 = 甲子（三源互证）
- 2024 二十四节气精确时刻（北京时间，日期精确、时刻 ±15min Meeus 低精度）
- 春节 2020–2025、闰月 2023 闰二月 / 2020 闰四月（含定气刀刃 case）
- 完整样例 1990-06-15 14:30 CST → 八字 庚午 / 壬午 / 辛亥 / 乙未；紫微 命宫亥·丁亥·土五局·紫微申·命宫主星巨门
- Diana（Rodden AA，1961-07-01 19:45 BST）的 Placidus / Koch 全 12 宫头，容差 0.05° vs pyswisseph
- Lahiri ayanamsa 三 epoch 对 Swiss Ephemeris `sweph.h` 源常数，容差 ±0.05°
- 耶利米 25:26 经典 atbash `בבל`(34) ⇌ `ששך`(620)

## API

```bash
curl -X POST http://127.0.0.1:6027/api/bazi -H 'content-type: application/json' \
  -d '{"year":1990,"month":6,"day":15,"hour":14,"minute":30,"tz":8,"gender":"male"}'
```

| 路由 | 说明 |
|---|---|
| `GET  /api/health` | 健康检查 |
| `POST /api/cast` | 全叶并行 fan-out——一次输入，所有术数同时排盘 |
| `POST /api/bazi` · `/api/bazi/overlay-strength` | 四柱盘 / 运层旺衰叠加 |
| `POST /api/ziwei` | 紫微斗数盘 |
| `POST /api/fortune` | 某时刻的岁运聚合 + 百年供给时序 |
| `POST /api/word` | 字词类叶（数字学 / gematria / abjad / 五格） |
| `POST /api/team` · `/api/team/interpret` | 多主体（团队）盘 |
| `GET  /api/analysis` | 跨叶信息论统计 |
| `GET  /api/intents` · `POST /api/route` | 问局模型——按意图路由到叶 |
| `POST /api/event` · `/api/event/interpret` | 占事——问事此刻 + 取机 |
| `POST /api/election` · `/api/election/interpret` | 择吉——扫时窗、逐日分档 |
| `POST /api/locative` · `/api/locative/interpret` | 寻方位 |
| `POST /api/synastry` · `/api/synastry/interpret` | 合盘——两人各给对方什么 |
| `POST /api/mundane` · `/api/mundane/interpret` | 国运——奠基时刻与年度盘 |
| `POST /api/interpret` | 释义层（🔮 INT，非计算产物） |

　　请求字段：`year month day hour`（必填）、`minute`（默认 0）、`tz`（默认 +8）、`gender`（`male` / `female`，也收 `男` / `女`；缺省不算大运，写别的会被拒而不是默默忽略）。支持 1900–2100。
端口可用 `MINGLI_API_BIND` 覆盖。

---

## 精度与已知简化

　　诚实边界优先于好看的数字——以下每一条都在代码里对应一个 🟡 UND 声明：

- 天文采用 Meeus 简化模型：太阳黄经 ~0.01°（≈15min）、朔 ~数分钟。**定到「哪一天」可靠**，节气时刻分钟级可能有几分钟偏差
- 时柱默认按民用钟表时间定时辰；真太阳时校正（经度差 + 简化均时差，±0.5 分钟）为**可选开关** `true_solar_time`
- 八字「晚子时换日」按民用日 00:00 处理，未做夜子时换日分流
- 紫微闰月生人的「生月」取本月数字，未做闰月上 / 下半月分流
- 大运起运按 3 日 = 1 年折算，未细分到月 / 日
- 大六壬三传的涉害 / 昴星 / 别责 / 八专取传流派分歧只判式不强编，返回 `None`
- 七政四余的紫炁、12 次落宫、28 宿分黄道古制均标 UND 不强编

## 免责

　　本仓库是**算法研究项目**。所有输出仅供研究与娱乐，不构成任何形式的建议。

## License

MIT，见 [`LICENSE`](LICENSE)
