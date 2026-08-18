// 视觉自查：用 headless Chromium 跑一遍界面，逐屏截图、断言、收集运行时报错。
//
// 编译通过不等于画面对：类型检查看不到布局塌陷、配色失效、渲染时抛错。
// 而截图也只是「有东西」——图存下来没人逐张看，就等于没看。所以每屏另带断言：
// 该有几格就是几格、该对上的两个数就得对上。断言不过 → 非零退出。
//
// 两个视口各跑一遍：1440 宽（常态）与 1024 宽（窄屏）。窄屏是另一套布局，
// 只在宽屏拍，等于那套布局从来没人看过。
//
//   bun run shots            # 需先起好 :6027 后端与 :6026 前端
//   bun run shots -- 奇门     # 只拍某几屏
//
// 产物在 e2e/shots/<宽度>/（已 gitignore），报错汇总打在标准输出。

import { chromium } from 'playwright'
import { mkdir, rm } from 'node:fs/promises'

const BASE = process.env.MINGLI_WEB ?? 'http://127.0.0.1:6026'
const OUT = new URL('./shots/', import.meta.url).pathname

/** 每屏的断言。拿到 page，抛异常即为不过。 */
const CHECKS = {
  '02-紫微斗数': async (page) => {
    // 大限十二步走满一轮，起运岁即五行局数，且相邻两步差十年
    const cells = await page.locator('.zw-limit b').allInnerTexts()
    if (cells.length !== 12) throw new Error(`大限应有 12 步，实有 ${cells.length}`)
    const spans = cells.map((t) => t.split('–').map(Number))
    for (const [lo, hi] of spans) {
      if (hi - lo !== 9) throw new Error(`每步应含十年（首尾各含），实得 ${lo}–${hi}`)
    }
    for (let i = 1; i < spans.length; i++) {
      if (spans[i][0] - spans[i - 1][0] !== 10) throw new Error(`相邻两步应差十岁：${spans[i - 1][0]} → ${spans[i][0]}`)
    }
    const head = await page.locator('.zw-limits .lp-sec-t').innerText()
    const ju = Number(head.match(/(\d+) 岁起运/)?.[1])
    if (spans[0][0] !== ju) throw new Error(`首步应自 ${ju} 岁起，实自 ${spans[0][0]}`)
  },

  '21-数字学': async (page) => {
    // 生命灵数两派算法不同，本盘出主值并把另一派并列——两个数都得在屏幕上
    const here = await page.locator('.nm-here').innerText()
    if (!['分量约化', '全数字直加'].includes(here.trim())) throw new Error(`算法名写的是「${here}」，不是两派之一`)
    const alt = await page.locator('.nm-alt').innerText()
    const n = alt.match(/得\s*(\d+)/)?.[1]
    if (!n) throw new Error(`另一派的数没写出来：「${alt}」`)
    const main = (await page.locator('.num-big .nb b').first().innerText()).trim()
    if (!/^\d+$/.test(main)) throw new Error(`主值不是个数：「${main}」`)
  },
  '04-印度占星': async (page) => {
    // 分盘表：每曜一行、每盘一列。列数写死 12（本盘 D-1 与九分盘 D-9 在上面两表，不在此）
    const cols = await page.locator('.jy-varga thead th').count()
    if (cols !== 13) throw new Error(`分盘表应有 1 + 12 列，实有 ${cols}`)
    const rows = await page.locator('.jy-varga tbody tr').count()
    const grahas = await page.locator('.jy-graha-table').first().locator('tbody tr').count()
    if (rows !== grahas) throw new Error(`九曜表 ${grahas} 行，分盘表 ${rows} 行`)
    // D-3 是独立可验的：三分盘落宫只可能是本宫 / 第 5 / 第 9，与九曜表的 Rasi 对得上
    const rasi = await page.locator('.jy-graha-table').first().locator('tbody tr td:nth-child(3)').allInnerTexts()
    const d3 = await page.locator('.jy-varga tbody tr td:nth-child(2)').allInnerTexts()
    const ORDER = ['Mesha', 'Vrishabha', 'Mithuna', 'Karka', 'Simha', 'Kanya',
      'Tula', 'Vrishchika', 'Dhanu', 'Makara', 'Kumbha', 'Meena']
    for (const [i, r] of rasi.entries()) {
      const from = ORDER.indexOf(r.trim())
      const to = ORDER.indexOf(d3[i].trim())
      if (from < 0 || to < 0) throw new Error(`第 ${i + 1} 行宫名认不出：本命「${r}」/ D-3「${d3[i]}」`)
      const step = (to - from + 12) % 12
      if (step !== 0 && step !== 4 && step !== 8) {
        throw new Error(`第 ${i + 1} 行 D-3 从「${ORDER[from]}」落到「${ORDER[to]}」，跳了 ${step} 宫——三分盘只能跳 0/4/8`)
      }
    }
  },
  '20-藏历循环': async (page) => {
    const PARKHA = ['Li', 'Khon', 'Da', 'Khen', 'Kham', 'Gin', 'Zin', 'Zon']
    const t = await page.getByText('历日卦 parkha').locator('..').innerText()
    if (!PARKHA.some((k) => t.includes(k))) throw new Error(`历日卦写的是「${t.replace(/\n/g, ' ')}」，不在八卦名里`)
  },
  '30-运势': async (page) => {
    // Vimshottari 九主星共 120 年；当前段的小运也是九步
    const segs = await page.locator('.fd-seg').count()
    if (segs !== 9) throw new Error(`大运条应有 9 段，实有 ${segs}`)
    const chips = await page.locator('.fd-antar:not(.prog) .fd-chip').count()
    if (chips !== 9) throw new Error(`当前大运的小运应有 9 步，实有 ${chips}`)
    const on = await page.locator('.fd-antar:not(.prog) .fd-chip.on').count()
    if (on !== 1) throw new Error(`当前小运应恰好高亮 1 步，实有 ${on}`)
    // 推运每五年一格，0..100 共 21 格
    const prog = await page.locator('.fd-antar.prog .fd-chip').count()
    if (prog !== 21) throw new Error(`推运应有 21 格（0..100 每五年），实有 ${prog}`)
    // 第三条时间线：二次推运。三套各自说各自的时间，缺一条就是少了一路
    const lines = await page.locator('.fortune-mile-l').allInnerTexts()
    if (!lines.some((t) => t.includes('二次推运'))) {
      throw new Error(`运势屏应有三条时间线（大运 / Vimshottari / 二次推运），实见「${lines.join(' | ')}」`)
    }
  },
  '18-奇门遁甲': async (page) => {
    const n = await page.locator('.qm-cell').count()
    if (n !== 9) throw new Error(`九宫应有 9 格，实有 ${n}`)
    // 中五宫不寄星，寄在坤 2；这条写错过一次，格子会空着而不报错
    const mid = await page.locator('.qm-cell').nth(4).innerText()
    if (!mid.includes('天禽寄坤 2')) throw new Error(`中宫应写「天禽寄坤 2」，实为「${mid.replace(/\n/g, ' ')}」`)
  },
  '26-择吉': async (page) => {
    const heads = await page.locator('.el-group-h b').allInnerTexts()
    for (const g of ['黄道', '可用', '黑道', '不可当']) {
      if (!heads.includes(g)) throw new Error(`择吉四档缺「${g}」，实有 ${heads.join(' / ')}`)
    }
    // 每档自报的天数要等于它表里的行数
    for (const [i, g] of heads.entries()) {
      const said = Number((await page.locator('.el-group-n').nth(i).innerText()).replace(/\D/g, ''))
      const rows = await page.locator('.el-group').nth(i).locator('tbody tr').count()
      if (said !== rows) throw new Error(`「${g}」自报 ${said} 天，表里 ${rows} 行`)
    }
  },
  '27-寻方位': async (page) => {
    const pins = await page.locator('.lc-pin').count()
    const rows = await page.locator('.lc-list tbody tr').count()
    if (pins !== rows) throw new Error(`罗盘 ${pins} 个点，表格 ${rows} 行——盘上少一个就是漏画一个候选`)
    if (pins === 0) throw new Error('一个方位候选都没有')
  },
  '28-合盘': async (page) => {
    const bars = await page.locator('.sy-give-n').allInnerTexts()
    if (bars.length !== 2) throw new Error(`应有两条供给数，实有 ${bars.length}`)
    const [a, b] = bars.map((t) => Number(t.replace(/\D/g, '')))
    const gap = Number((await page.locator('.sy-mid-note').innerText()).replace(/\D/g, ''))
    if (Math.abs(a - b) !== gap) throw new Error(`两边 ${a}% / ${b}%，中间却写差 ${gap} 个百分点`)
    // 八项合婚：八行，且总分等于各项之和
    const rows = await page.locator('.sy-kuta tbody tr').count()
    if (rows !== 8) throw new Error(`Ashtakuta 应有八项，实有 ${rows}`)
    const cells = await page.locator('.sy-kuta tbody tr td:nth-child(2)').allInnerTexts()
    const lows = cells.map((t) => Number(t.trim().split('–')[0].replace('≈', '')))
    const sum = lows.reduce((a, b) => a + b, 0)
    const head = await page.locator('.sy-kuta-l b').innerText()
    const headLow = Number(head.trim().split('–')[0])
    if (Math.abs(sum - headLow) > 0.05) throw new Error(`八项下界之和 ${sum}，标题却写 ${headLow}`)
    // 满分列合计恒为 36
    const maxes = await page.locator('.sy-kuta tbody tr td:nth-child(3)').allInnerTexts()
    const total = maxes.reduce((a, t) => a + Number(t), 0)
    if (total !== 36) throw new Error(`八项满分之和应为 36，实为 ${total}`)

    // 相位：自报几条就得画几条
    const said = Number((await page.locator('.sy-asp-l small').innerText()).match(/(\d+) 条/)?.[1])
    const drawn = await page.locator('.sy-asp').count()
    if (said !== drawn) throw new Error(`相位自报 ${said} 条，画出 ${drawn} 条`)
  },
  '29-国运': async (page) => {
    const n = await page.locator('.mu-tl-year').count()
    if (n === 0) throw new Error('时间线一年都没画')
  },
}

/** 每屏：名字 + 怎么切过去 + 切完等什么出现。 */
/** 21 片叶：tab 上的名字 + 切过去等什么。缺省等 `.lp`（叶整页的外壳）。 */
const LEAVES = [
  ['四柱八字', '.lp-sec-t, .kv-grid'], ['紫微斗数', '.grid9, .lp'], ['西洋占星', '.astro-wheel, .lp'],
  ['印度占星'], ['七政四余'], ['易经起卦'], ['地占'], ['Sikidy'], ['Ifá'], ['塔罗'],
  ['梅花易数'], ['小六壬'], ['择日'], ['玛雅历'], ['巴厘Pawukon'], ['缅甸Mahabote'],
  ['大六壬'], ['奇门遁甲', '.qm-cell'], ['太乙神数'], ['藏历循环'], ['数字学'],
]

const SCREENS = [
  // 首屏已经停在八字上，不用点 tab
  { name: '01-四柱八字', tab: null, wait: '.lp-sec-t, .kv-grid' },
  ...LEAVES.slice(1).map(([tab, wait], i) => ({
    name: `${String(i + 2).padStart(2, '0')}-${tab}`,
    tab,
    wait: wait ?? '.lp',
  })),
  { name: '22-文字术数', tab: '字 文字术数', wait: '.word-form' },
  { name: '23-合盘团队', tab: '合 合盘 / 团队', wait: '.lp-sec-t' },
  { name: '24-相关性', tab: '⊞ 相关性', wait: '.card' },
  // 意图页：先点顶部意图 chip，再等该意图自己的界面
  { name: '25-占事', intent: '事（占事）', wait: '.ev-draw' },
  { name: '26-择吉', intent: '择（择吉）', wait: '.el-form', action: '择 日', result: '.el-groups' },
  { name: '27-寻方位', intent: '寻（寻方位）', wait: '.ev-draw', action: '起 课', result: '.lc-top' },
  { name: '28-合盘', intent: '合（合盘）', wait: '.sy-forms', action: '合 盘', result: '.sy-pair' },
  { name: '29-国运', intent: '群/国（国运）', wait: '.ev-form', action: '推 演', result: '.mu-tl' },
  { name: '30-运势', intent: '运（运势/流年/大运）', wait: '.fortune-chart' },
]

const problems = []
const only = process.argv.slice(2)

// README 自称拍多少屏。屏表是由叶表切片拼出来的，只有这里算得准，所以由这里来对。
// 认的是「N screens」/「N 屏」这句话本身，不是「文中出现过 N」——后者会被别处的数字蒙混过去。
if (!only.length) {
  const { readFile } = await import('node:fs/promises')
  for (const [f, re] of [
    ['../../README.md', /(\d+) screens/],
    ['../../README.zh-CN.md', /(\d+) 屏/],
  ]) {
    const said = (await readFile(new URL(f, import.meta.url), 'utf8')).match(re)?.[1]
    if (said === undefined) problems.push(`${f.slice(6)} 里找不到「N 屏」那句话——句式改过了，本处要跟着改`)
    else if (Number(said) !== SCREENS.length) problems.push(`${f.slice(6)} 自称 ${said} 屏，实为 ${SCREENS.length} 屏`)
  }
}

/** 常态宽屏 + 窄屏。窄屏那套 grid 换列数，只有真按那个宽度渲染才看得见。 */
const VIEWPORTS = [
  { tag: '1440', width: 1440, height: 1000 },
  { tag: '1024', width: 1024, height: 900 },
]

const browser = await chromium.launch()
await rm(OUT, { recursive: true, force: true })

for (const vp of VIEWPORTS) {
  const dir = `${OUT}${vp.tag}/`
  await mkdir(dir, { recursive: true })
  const page = await browser.newPage({
    viewport: { width: vp.width, height: vp.height },
    deviceScaleFactor: 2,
  })
  const note = (m) => problems.push(`[${vp.tag}] ${m}`)
  page.on('console', (m) => { if (m.type() === 'error') note(`console error: ${m.text()}`) })
  page.on('pageerror', (e) => note(`page error: ${e.message}`))
  page.on('requestfailed', (r) => note(`request failed: ${r.url()} — ${r.failure()?.errorText}`))
  // 界面静止时后端调用也该静止。曾有一处在渲染里读时钟，使全叶排盘以近 10 次/秒
  // 自循环重发；画面看起来正常，只有数请求才看得见。
  let apiCalls = 0
  page.on('request', (r) => { if (r.url().includes('/api/')) apiCalls++ })

  // dev server 的 HMR 长连接让 networkidle 永远不到，改等 DOM + 首屏元素
  await page.goto(BASE, { waitUntil: 'domcontentloaded' })
  // 首屏自动排盘，等盘出来
  await page.waitForSelector('.leaf-tabs', { timeout: 30_000 })

  console.log(`\n—— ${vp.width}×${vp.height} ——`)
  for (const s of SCREENS) {
    if (only.length && !only.some((k) => s.name.includes(k))) continue
    const target = s.tab ?? s.intent
    if (target) {
      // 一定要限定在各自的容器里找：叶名与意图 chip 的文案会撞
      // （意图「号（数字学生命灵数/姓名值/五格）」里含「数字学」，全页找会点中它，
      //  于是整个页面切到那个意图，后面几屏跟着一起找不到入口）
      const scope = page.locator(s.tab ? '.leaf-tabs' : '.intent-bar-chips')
      const btn = scope.getByRole('button', { name: target, exact: false }).first()
      if (!(await btn.count())) {
        note(`找不到入口：${target}`)
        continue
      }
      await btn.click()
    }
    if (s.intent) {
      // 意图页要先按下动作按钮才有内容可看（占事「起盘」、择吉「择日」）
      const action = s.action ?? '起 盘'
      const result = s.result ?? '.ev-leaves'
      const btn = page.getByRole('button', { name: action, exact: false }).first()
      if (await btn.count()) {
        await btn.click()
        await page.waitForSelector(result, { timeout: 20_000 })
          .catch(() => note(`${s.name}：按下「${action}」后等不到 ${result}`))
      }
    }
    try {
      await page.waitForSelector(s.wait, { timeout: 20_000 })
    } catch {
      note(`${s.name}：等不到 ${s.wait}`)
    }
    await page.waitForTimeout(400) // 让过渡动画落定
    await page.screenshot({ path: `${dir}${s.name}.png`, fullPage: true })
    // 断言问的是「页面自己前后一致吗」，与宽度无关，所以两个视口都跑
    const check = CHECKS[s.name]
    if (check) {
      try {
        await check(page)
        console.log(`拍下 ${s.name} ✓`)
      } catch (e) {
        note(`${s.name}：${e.message}`)
        console.log(`拍下 ${s.name} ✗ ${e.message}`)
      }
    } else {
      console.log(`拍下 ${s.name}`)
    }
  }
  // —— 收尾：没人操作的三秒里，还在发几个请求？ ——
  await page.waitForTimeout(1200)
  const before = apiCalls
  await page.waitForTimeout(3000)
  const idle = apiCalls - before
  if (idle > 2) note(`界面静止的 3 秒里发了 ${idle} 次 /api 调用——有东西在自循环`)
  else console.log(`静止 3 秒 · /api 调用 ${idle} 次`)

  await page.close()
}

await browser.close()

if (problems.length) {
  console.log(`\n运行时问题 ${problems.length} 条：`)
  for (const p of [...new Set(problems)]) console.log(`  · ${p}`)
  process.exitCode = 1
} else {
  console.log('\n两个视口 · 全部屏：无 console 报错、无失败请求、断言全过')
}
