// 视觉自查：用 headless Chromium 跑一遍界面，逐屏截图并收集运行时报错。
//
// 编译通过不等于画面对：类型检查看不到布局塌陷、配色失效、渲染时抛错。
// 这个脚本把「看一眼」变成可重复的一条命令。
//
//   bun run shots            # 需先起好 :6027 后端与 :6026 前端
//   bun run shots -- 奇门     # 只拍某几屏
//
// 产物在 e2e/shots/（已 gitignore），报错汇总打在标准输出。

import { chromium } from 'playwright'
import { mkdir, rm } from 'node:fs/promises'

const BASE = process.env.MINGLI_WEB ?? 'http://127.0.0.1:6026'
const OUT = new URL('./shots/', import.meta.url).pathname

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
    // 占事要先起盘才有内容可看
    const cast = page.getByRole('button', { name: '起 盘', exact: false }).first()
    if (await cast.count()) {
      await cast.click()
      await page.waitForSelector('.ev-leaves', { timeout: 20_000 }).catch(() => problems.push('占事：起盘后等不到 .ev-leaves'))
    }
  }
  try {
    await page.waitForSelector(s.wait, { timeout: 20_000 })
  } catch {
    problems.push(`${s.name}：等不到 ${s.wait}`)
  }
  await page.waitForTimeout(400) // 让过渡动画落定
  await page.screenshot({ path: `${OUT}${s.name}.png`, fullPage: true })
  console.log(`拍下 ${s.name}`)
}

await browser.close()

if (problems.length) {
  console.log(`\n运行时问题 ${problems.length} 条：`)
  for (const p of [...new Set(problems)]) console.log(`  · ${p}`)
  process.exitCode = 1
} else {
  console.log('\n无 console 报错、无失败请求')
}
