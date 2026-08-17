// 视觉自查：用 headless Chromium 跑一遍界面，逐屏截图、断言、收集运行时报错。
//
// 编译通过不等于画面对：类型检查看不到布局塌陷、配色失效、渲染时抛错。
// 而截图也只是「有东西」——图存下来没人逐张看，就等于没看。所以每屏另带断言：
// 该有几格就是几格、该对上的两个数就得对上。断言不过 → 非零退出。
//
//   bun run shots            # 需先起好 :6027 后端与 :6026 前端
//   bun run shots -- 奇门     # 只拍某几屏
//
// 产物在 e2e/shots/（已 gitignore），报错汇总打在标准输出。

import { chromium } from 'playwright'
import { mkdir, rm } from 'node:fs/promises'

const BASE = process.env.MINGLI_WEB ?? 'http://127.0.0.1:6026'
const OUT = new URL('./shots/', import.meta.url).pathname

/** 每屏的断言。拿到 page，抛异常即为不过。 */
const CHECKS = {
  '02-奇门': async (page) => {
    const n = await page.locator('.qm-cell').count()
    if (n !== 9) throw new Error(`九宫应有 9 格，实有 ${n}`)
    // 中五宫不寄星，寄在坤 2；这条写错过一次，格子会空着而不报错
    const mid = await page.locator('.qm-cell').nth(4).innerText()
    if (!mid.includes('天禽寄坤 2')) throw new Error(`中宫应写「天禽寄坤 2」，实为「${mid.replace(/\n/g, ' ')}」`)
  },
  '11-择吉': async (page) => {
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
  '12-寻方位': async (page) => {
    const pins = await page.locator('.lc-pin').count()
    const rows = await page.locator('.lc-list tbody tr').count()
    if (pins !== rows) throw new Error(`罗盘 ${pins} 个点，表格 ${rows} 行——盘上少一个就是漏画一个候选`)
    if (pins === 0) throw new Error('一个方位候选都没有')
  },
  '13-合盘': async (page) => {
    const bars = await page.locator('.sy-give-n').allInnerTexts()
    if (bars.length !== 2) throw new Error(`应有两条供给数，实有 ${bars.length}`)
    const [a, b] = bars.map((t) => Number(t.replace(/\D/g, '')))
    const gap = Number((await page.locator('.sy-mid-note').innerText()).replace(/\D/g, ''))
    if (Math.abs(a - b) !== gap) throw new Error(`两边 ${a}% / ${b}%，中间却写差 ${gap} 个百分点`)
  },
  '14-国运': async (page) => {
    const n = await page.locator('.mu-tl-year').count()
    if (n === 0) throw new Error('时间线一年都没画')
  },
}

/** 每屏：名字 + 怎么切过去 + 切完等什么出现。 */
const SCREENS = [
  { name: '01-首屏-八字', tab: null, wait: '.lp-sec-t, .kv-grid' },
  { name: '02-奇门', tab: '奇门遁甲', wait: '.qm-cell' },
  { name: '03-紫微', tab: '紫微斗数', wait: '.grid9, .lp' },
  { name: '04-西洋占星', tab: '西洋占星', wait: '.lp' },
  { name: '05-大六壬', tab: '大六壬', wait: '.lp' },
  { name: '06-易经', tab: '易经', wait: '.lp' },
  { name: '07-文字术数', tab: '字 文字术数', wait: '.word-form' },
  { name: '08-合盘团队', tab: '合 合盘 / 团队', wait: '.lp-sec-t' },
  { name: '09-相关性', tab: '⊞ 相关性', wait: '.card' },
  // 意图页：先点顶部意图 chip，再等该意图自己的界面
  { name: '10-占事', intent: '事（占事）', wait: '.ev-draw' },
  { name: '11-择吉', intent: '择（择吉）', wait: '.el-form', action: '择 日', result: '.el-groups' },
  { name: '12-寻方位', intent: '寻（寻方位）', wait: '.ev-draw', action: '起 课', result: '.lc-top' },
  { name: '13-合盘', intent: '合（合盘）', wait: '.sy-forms', action: '合 盘', result: '.sy-pair' },
  { name: '14-国运', intent: '群/国（国运）', wait: '.ev-form', action: '推 演', result: '.mu-tl' },
]

const problems = []

const browser = await chromium.launch()
const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, deviceScaleFactor: 2 })

page.on('console', (m) => {
  if (m.type() === 'error') problems.push(`console error: ${m.text()}`)
})
page.on('pageerror', (e) => problems.push(`page error: ${e.message}`))
page.on('requestfailed', (r) => problems.push(`request failed: ${r.url()} — ${r.failure()?.errorText}`))

await rm(OUT, { recursive: true, force: true })
await mkdir(OUT, { recursive: true })

// dev server 的 HMR 长连接让 networkidle 永远不到，改等 DOM + 首屏元素
await page.goto(BASE, { waitUntil: 'domcontentloaded' })
// 首屏自动排盘，等盘出来
await page.waitForSelector('.leaf-tabs', { timeout: 30_000 })

const only = process.argv.slice(2)
for (const s of SCREENS) {
  if (only.length && !only.some((k) => s.name.includes(k))) continue
  const target = s.tab ?? s.intent
  if (target) {
    const btn = page.getByRole('button', { name: target, exact: false }).first()
    if (!(await btn.count())) {
      problems.push(`找不到入口：${target}`)
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
      await page.waitForSelector(result, { timeout: 20_000 }).catch(() => problems.push(`${s.name}：按下「${action}」后等不到 ${result}`))
    }
  }
  try {
    await page.waitForSelector(s.wait, { timeout: 20_000 })
  } catch {
    problems.push(`${s.name}：等不到 ${s.wait}`)
  }
  await page.waitForTimeout(400) // 让过渡动画落定
  await page.screenshot({ path: `${OUT}${s.name}.png`, fullPage: true })
  const check = CHECKS[s.name]
  if (check) {
    try {
      await check(page)
      console.log(`拍下 ${s.name} ✓`)
    } catch (e) {
      problems.push(`${s.name}：${e.message}`)
      console.log(`拍下 ${s.name} ✗ ${e.message}`)
    }
  } else {
    console.log(`拍下 ${s.name}`)
  }
}

await browser.close()

if (problems.length) {
  console.log(`\n运行时问题 ${problems.length} 条：`)
  for (const p of [...new Set(problems)]) console.log(`  · ${p}`)
  process.exitCode = 1
} else {
  console.log('\n无 console 报错、无失败请求')
}
