// CSS 变量对账：用到的每个 var(--x) 都得有来源，定义的每个 --x 都得有人用。
//
// 拼错一个变量名不会报错，只会静默取到空值——颜色变透明、间距变 0，页面还是渲染出来，
// 截图里也未必看得出来。反过来，样式删掉一段后留下的孤儿变量会让人以为它还在起作用。
// 两个方向都对一遍，才知道这份样式表里没有悬空的名字。
//
//   node e2e/css-vars.mjs
//
// 有一类变量不在 CSS 里定义：组件按数据算出来后内联注入（如按叶的家族色 --fam）。
// 这类从 tsx 里扫 style={{ ['--x']: … }} 认下来，不然对账会把它们误报成拼错。

import { readdir, readFile } from 'node:fs/promises'
import { join } from 'node:path'

const SRC = new URL('../src/', import.meta.url).pathname

/** 递归收集某后缀的文件。 */
async function walk(dir, ext, out = []) {
  for (const e of await readdir(dir, { withFileTypes: true })) {
    const p = join(dir, e.name)
    if (e.isDirectory()) await walk(p, ext, out)
    else if (e.name.endsWith(ext)) out.push(p)
  }
  return out
}

const cssFiles = await walk(SRC, '.css')
const tsxFiles = await walk(SRC, '.tsx')

const used = new Map()    // 变量名 → 用到它的文件
const declared = new Map() // 变量名 → 声明它的文件

for (const f of cssFiles) {
  const text = await readFile(f, 'utf8')
  for (const m of text.matchAll(/var\((--[\w-]+)/g)) {
    used.set(m[1], (used.get(m[1]) ?? new Set()).add(f))
  }
  // 声明要在剔掉 var(...) 之后再找：var(--a, --b) 这种回退写法里的第二个名字不是声明。
  // 另外这里一定要按「声明」逐个匹配而不是按行首——:root 里一行常写好几个
  // （--sumi: …; --sumi2: …; --sumi3: …;），按行首找会漏掉除头一个以外的全部，
  // 于是对账报出一堆「用了没定义」，看着像样式表坏了，其实是这把尺子坏了。
  for (const m of text.replace(/var\([^)]*\)/g, '').matchAll(/(--[\w-]+)\s*:/g)) {
    declared.set(m[1], (declared.get(m[1]) ?? new Set()).add(f))
  }
}

const inline = new Map()
for (const f of tsxFiles) {
  const text = await readFile(f, 'utf8')
  for (const m of text.matchAll(/\[\s*'(--[\w-]+)'/g)) {
    inline.set(m[1], (inline.get(m[1]) ?? new Set()).add(f))
  }
}

const short = (p) => p.slice(SRC.length)
const problems = []

for (const [name, where] of used) {
  if (declared.has(name) || inline.has(name)) continue
  problems.push(`${name} 用在 ${[...where].map(short).join(' / ')}，但哪儿都没定义`)
}
for (const [name, where] of declared) {
  if (used.has(name)) continue
  problems.push(`${name} 定义在 ${[...where].map(short).join(' / ')}，但没人用`)
}
for (const [name, where] of inline) {
  if (used.has(name)) continue
  problems.push(`${name} 由 ${[...where].map(short).join(' / ')} 内联注入，但样式里没人读`)
}

const n = used.size
const src = `${declared.size} 个在样式里、${inline.size} 个由组件内联注入`
if (problems.length) {
  console.log(`CSS 变量对账不平（用到 ${n} 个，${src}）：`)
  for (const p of problems) console.log(`  · ${p}`)
  process.exit(1)
}
console.log(`CSS 变量对账平：用到 ${n} 个，${src}，两个方向都对得上`)
