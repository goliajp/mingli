// 后端算出来的字段，前端总得有一处认得它。
//
// 「算出来但看不见」在这个仓库里出过五次（印度占星的十二分盘、藏历的历日卦、
// 运势的大运段、合盘的两盘相位、数字学的另一派生命灵数）。共同点是：
// 后端一直在算、JSON 一直在发，只是没人在前端接。类型检查看不见这件事，
// 截图也看不见——屏幕上少一块，跟本来就没有这块，长得一模一样。
//
// 判据：/api/cast 每片叶盘面上的每个顶层字段，都要在 web/src 的某处源码里出现。
// 不必渲染成什么样子，但至少得被认领。EXCUSED 里的另说，且要写明为什么。
//
//   node e2e/wired.mjs            # 需先起好 :6027 后端

import { readdir, readFile } from 'node:fs/promises'
import { join } from 'node:path'

const API = process.env.MINGLI_API ?? 'http://127.0.0.1:6027'

/** 不必出现在前端的字段，与不必的理由。 */
const EXCUSED = [
  ['*', 'input', '入参回显，前端本来就有这份表单数据'],
  ['*', 'lunar', '入参时刻的农历写法，同为回显'],
  ['yijing', 'changing_mask', '变爻的位掩码；界面按 `lines[].changing` 逐爻画，是同一件事的另一种写法'],
]

async function sources(dir) {
  let out = ''
  for (const e of await readdir(dir, { withFileTypes: true })) {
    const p = join(dir, e.name)
    if (e.isDirectory()) out += await sources(p)
    else if (/\.tsx?$/.test(e.name)) out += await readFile(p, 'utf8')
  }
  return out
}

const body = {
  year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8,
  gender: 'male', latitude: 31.23, longitude: 121.47, seed: 7, name: 'Ada',
}
const res = await fetch(`${API}/api/cast`, {
  method: 'POST',
  headers: { 'content-type': 'application/json' },
  body: JSON.stringify(body),
})
if (!res.ok) {
  console.error(`✗ ${API}/api/cast 返回 ${res.status}——后端没起来的话这一遍等于没测`)
  process.exit(1)
}
const { leaves } = await res.json()
if (!leaves?.length) {
  console.error('✗ 一片叶都没拿到，判据失效')
  process.exit(1)
}

const src = await sources(new URL('../src/', import.meta.url).pathname)
const gaps = []
let fields = 0
for (const leaf of leaves) {
  const chart = leaf.chart
  if (!chart || typeof chart !== 'object' || Array.isArray(chart)) continue
  for (const k of Object.keys(chart)) {
    fields++
    if (src.includes(k)) continue
    if (EXCUSED.some(([id, f]) => (id === '*' || id === leaf.id) && f === k)) continue
    gaps.push(`${leaf.id} · ${k}`)
  }
}

if (gaps.length) {
  console.error(`✗ 后端算出来、前端一处都没有的字段 ${gaps.length} 个：`)
  for (const g of gaps) console.error(`    ${g}`)
  console.error('  要么在界面上接住它，要么写进本脚本的 EXCUSED 并说明为什么不必接')
  process.exit(1)
}
console.log(`✓ ${leaves.length} 片叶 ${fields} 个字段，前端都认得（另有 ${EXCUSED.length} 条写明理由的例外）`)
