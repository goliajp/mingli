// 后端拒绝的理由，与后端根本没应答，必须在界面上分得开。
//
// 后端每一次拒绝都带着一句具体的中文理由——「1990 年 2 月只有 28 天」「hour/minute 越界」——
// 那是给用户看的，重试一百次也不会变。而服务没起来是另一回事，重试才有意义。
// 这两种从前被渲染成同一句「服务连接失败，请稍后重试」：输错日期的人被告知这是网络问题，
// 而正确的理由就印在同一行上。
//
// 这支查两件事：
//   一、后端确实为坏输入返回带理由的 400（而不是 500、也不是含糊的一句「请求失败」）；
//   二、前端那句重试提示只由 `describeFailure` 一处产生，且只加给连不上的那种；
//      任何组件若自己拼这句话，就又把两者混回去了。
//
//   node e2e/errors.mjs            # 需先起好 :6027 后端
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'

const API = process.env.MINGLI_API ?? 'http://127.0.0.1:6027'
const RETRY_HINT = '服务连接失败'
let bad = 0

// ── 一、后端为坏输入给出可读的理由 ───────────────────────────────
const CASES = [
  { name: '不存在的日子', body: { year: 1990, month: 2, day: 31, hour: 12, minute: 0, tz: 8 } },
  { name: '越界的钟点', body: { year: 1990, month: 6, day: 15, hour: 25, minute: 0, tz: 8 } },
  { name: '越界的分钟', body: { year: 1990, month: 6, day: 15, hour: 12, minute: 61, tz: 8 } },
  { name: '越界的时区', body: { year: 1990, month: 6, day: 15, hour: 12, minute: 0, tz: 99 } },
  { name: '越界的年份', body: { year: 1200, month: 6, day: 15, hour: 12, minute: 0, tz: 8 } },
]

for (const c of CASES) {
  let res
  try {
    res = await fetch(`${API}/api/bazi`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ ...c.body, gender: 'male' }),
    })
  } catch (e) {
    console.error(`✗ ${API} 连不上：${e.message}`)
    process.exit(2)
  }
  if (res.status !== 400) {
    console.error(`✗ ${c.name}：应答 ${res.status}，坏输入该是 400`)
    bad++
    continue
  }
  const body = await res.json().catch(() => ({}))
  const msg = typeof body.error === 'string' ? body.error : ''
  if (!msg || msg === '请求失败' || msg.length < 4) {
    console.error(`✗ ${c.name}：400 的理由是 ${JSON.stringify(body)}——含糊的理由等于没有理由`)
    bad++
  } else {
    console.log(`  ${c.name} → 400「${msg}」`)
  }
}

// ── 二、重试那句话只有一处出处 ──────────────────────────────────
function walk(dir) {
  const out = []
  for (const name of readdirSync(dir)) {
    const p = join(dir, name)
    if (statSync(p).isDirectory()) out.push(...walk(p))
    else if (/\.(ts|tsx)$/.test(name)) out.push(p)
  }
  return out
}

const files = walk('src')
const holders = files.filter((f) => readFileSync(f, 'utf8').includes(RETRY_HINT))
if (holders.length !== 1 || !holders[0].endsWith('api/client.ts')) {
  console.error(
    `✗ 「${RETRY_HINT}」该只出现在 src/api/client.ts 一处，实际在：\n   ${holders.join('\n   ')}`,
  )
  console.error('   别处自己拼这句话，就等于把「你输错了」与「连不上」又混成一句。')
  bad++
} else {
  console.log(`  「${RETRY_HINT}」只在 ${holders[0]}`)
}

const client = readFileSync('src/api/client.ts', 'utf8')
if (!/kind === 'unreachable'[\s\S]{0,80}服务连接失败/.test(client)) {
  console.error(`✗ client.ts 里那句重试提示没有挂在 unreachable 那一支上`)
  bad++
}

// 拒绝那一路必须原样带着后端的话：客户端不能把它替换成自己的措辞。
if (!/new ApiError\('refused', e\.error/.test(client)) {
  console.error('✗ 4xx 的理由该原样取自后端的 error 字段')
  bad++
}

if (bad > 0) {
  console.error(`\n${bad} 处不合`)
  process.exit(1)
}
console.log('\n两种失败在界面上分得开')
