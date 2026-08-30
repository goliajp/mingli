// 后端算出来的字段，前端总得有一处认得它。
//
// 「算出来但看不见」在这个仓库里出过五次（印度占星的十二分盘、藏历的历日卦、
// 运势的大运段、合盘的两盘相位、数字学的另一派生命灵数）。共同点是：
// 后端一直在算、JSON 一直在发，只是没人在前端接。类型检查看不见这件事，
// 截图也看不见——屏幕上少一块，跟本来就没有这块，长得一模一样。
//
// 判据：/api/cast 每片叶盘面上的每个顶层字段，都要在 web/src 的某处源码里**被用到**。
// 不必渲染成什么样子，但至少得被认领。EXCUSED 里的另说，且要写明为什么。
//
// 「用到」不含 types.ts。那是 DTO 层——字段写进类型定义只说明它到了浏览器，
// 不说明有谁看它一眼。从前把 types.ts 也算作认领，于是「算出来、类型也写了、
// 没人渲染」这一类恰好躲过去：实测收紧后当场浮出两个（紫微的局数、择日的等第名）。
//
// 豁免项本身也要还成立：一条不再对应任何字段的豁免是死条目，
// 留着只会在日后遮住一个同名的新缺口。
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
  ['ziwei', 'ju_number', '五行局的数；界面写的是 `wuxing_ju` 那个名（「土五局」），数就在名里'],
  ['jyotish', 'lagna_rasi', '上升所在宫的序号；界面写的是 `lagna_rasi_name`，同一件事的名'],
  ['jyotish', 'lagna_navamsa', '上升在 D-9 的宫序号；界面写的是 `lagna_navamsa_name`'],
  [
    'zeri',
    'grade_label',
    '等第的中文名；ElectionView 自带一张表，因为它还要按四档排序并各带一句建除口诀，'
      + '空档也得列出来。两份措辞由下面 GRADE_LABELS 那条对账守着',
  ],
]

/** 后端发来的等第名，必须与前端那张表里的字字相同。 */
const GRADE_TABLE = 'src/views/ElectionView.tsx'

async function sources(dir, skip = null) {
  let out = ''
  for (const e of await readdir(dir, { withFileTypes: true })) {
    const p = join(dir, e.name)
    if (e.isDirectory()) out += await sources(p, skip)
    else if (/\.tsx?$/.test(e.name) && !(skip && p.endsWith(skip))) out += await readFile(p, 'utf8')
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

// /api/cast 之外还有几条各自出结构的端点，它们的顶层字段同样得有人接。
// 合盘的 ashtakuta 就是从这个盲区里漏过一次的——它不在任何一片叶的盘面上，
// 于是只扫 /api/cast 的话看不见。
const EXTRA = [
  ['/api/synastry', {
    a: { year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8, gender: 'male' },
    b: { year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8, gender: 'female' },
  }],
]


// 认「整个标识符」，不认子串。
//
// 原先用的是 `src.includes(k)`，于是字段 `sign` 会被源码里任何一个 `design` / `assign`
// 认领，`age` 会被 `message` 认领——报的是「前端认得这个字段」，实际前端从没提过它。
// 现在 217 个字段里还没有一个是这么蒙过去的（改这一行时逐个验过），所以这不是修 bug，
// 是把一条迟早会被踩中的假绿路径堵掉。
const shown = (src, k) =>
  new RegExp(`(^|[^A-Za-z0-9_])${k.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}([^A-Za-z0-9_]|$)`).test(src)

// 认领面不含 types.ts：字段写进类型定义只说明它到了浏览器，不说明有谁看它一眼。
const srcRoot = new URL('../src/', import.meta.url).pathname
const src = await sources(srcRoot, 'types.ts')
const gaps = []
let fields = 0
for (const leaf of leaves) {
  const chart = leaf.chart
  if (!chart || typeof chart !== 'object' || Array.isArray(chart)) continue
  for (const k of Object.keys(chart)) {
    fields++
    if (shown(src, k)) continue
    if (EXCUSED.some(([id, f]) => (id === '*' || id === leaf.id) && f === k)) continue
    gaps.push(`${leaf.id} · ${k}`)
  }
}

for (const [path, payload] of EXTRA) {
  const r = await fetch(`${API}${path}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(payload),
  })
  if (!r.ok) {
    console.error(`✗ ${path} 返回 ${r.status}`)
    process.exit(1)
  }
  const body = await r.json()
  for (const k of Object.keys(body)) {
    fields++
    if (!shown(src, k)) gaps.push(`${path} · ${k}`)
  }
}

// 豁免项本身还成立吗——一条不再对应任何字段的豁免是死条目，
// 留着只会在日后遮住一个同名的新缺口。
const present = new Set()
for (const leaf of leaves) {
  for (const k of Object.keys(leaf.chart ?? {})) {
    present.add(`${leaf.id}·${k}`)
    present.add(`*·${k}`)
  }
}
for (const [id, f, why] of EXCUSED) {
  if (!present.has(`${id}·${f}`)) {
    gaps.push(`豁免已失效：${id} · ${f}（理由写着「${why}」，但盘面上已无此字段）`)
  }
}

// 择日的等第名在两处各写了一份：后端算好发过来，前端另有一张表（它还要排序与注解）。
// 重复本身是有理由的，但两边的字必须一样，否则界面上的词与释义层收到的词会各说各的。
{
  const zeri = leaves.find((l) => l.id === 'zeri')
  const label = zeri?.chart?.grade_label
  const table = await readFile(new URL(`../${GRADE_TABLE}`, import.meta.url).pathname, 'utf8')
  // 不能用上面那个 `shown`：它的边界字符类是 [A-Za-z0-9_]，对中文不起作用——
  // 把「黄道」改成「黄道日」照样算命中。改成把表里的标签逐个解析出来做整串比对。
  const labels = [...table.matchAll(/label:\s*'([^']+)'/g)].map((m) => m[1])
  if (labels.length !== 4) {
    gaps.push(`${GRADE_TABLE} 里解析出 ${labels.length} 个等第名，应为四个——这条对账失效了`)
  }
  if (typeof label !== 'string' || !label) {
    gaps.push('zeri · grade_label 没发过来，等第名的对账无从做起')
  } else if (!labels.includes(label)) {
    gaps.push(
      `等第名对不上：后端发「${label}」，${GRADE_TABLE} 里那四个是 ${labels.map((x) => `「${x}」`).join('')}`,
    )
  }
}

if (gaps.length) {
  console.error(`✗ 后端算出来、前端一处都没有的字段 ${gaps.length} 个：`)
  for (const g of gaps) console.error(`    ${g}`)
  console.error('  要么在界面上接住它，要么写进本脚本的 EXCUSED 并说明为什么不必接')
  process.exit(1)
}
console.log(`✓ ${leaves.length} 片叶 ${fields} 个字段，前端都认得（另有 ${EXCUSED.length} 条写明理由的例外）`)
