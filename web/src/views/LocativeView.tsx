// 寻方位视图：问的此刻 + 取机 + 所寻 → 六壬 / 奇门起课 → 方位候选罗盘 + 各叶原盘 → 「位」。
import { useState } from 'react'
import type { Bearing, Interpretation, Locative } from '../types'
import { fetchLocative, fetchLocativeAdvice } from '../api/client'
import { LeafChart } from '../views/leaves'

const pad = (n: number) => String(n).padStart(2, '0')

function nowFields() {
  const d = new Date()
  return { year: d.getFullYear(), month: d.getMonth() + 1, day: d.getDate(), hour: d.getHours(), minute: d.getMinutes(), tz: 8 }
}

/** 八方在罗盘上的角度（北为 0，顺时针）。 */
const DIR_ANGLE: Record<string, number> = {
  北: 0, 东北: 45, 东: 90, 东南: 135, 南: 180, 西南: 225, 西: 270, 西北: 315,
}

const CATEGORIES = ['寻人', '寻物', '问向', '出行方位'] as const

/** 罗盘：八方标度 + 各候选按方位落点（同方位的稍作扇形错开）。 */
function Compass({ bearings }: { bearings: Bearing[] }) {
  const R = 130, C = 160
  const byDir: Record<string, Bearing[]> = {}
  for (const b of bearings) (byDir[b.direction] ??= []).push(b)
  const pt = (deg: number, r: number) => {
    const a = ((deg - 90) * Math.PI) / 180
    return [C + r * Math.cos(a), C + r * Math.sin(a)] as const
  }
  return (
    <svg className="lc-compass" viewBox="0 0 320 320" role="img" aria-label="方位罗盘">
      <circle cx={C} cy={C} r={R} className="lc-ring" />
      <circle cx={C} cy={C} r={R * 0.55} className="lc-ring inner" />
      {Object.entries(DIR_ANGLE).map(([d, deg]) => {
        const [x, y] = pt(deg, R + 16)
        const [x1, y1] = pt(deg, R - 6)
        const [x2, y2] = pt(deg, R + 4)
        return (
          <g key={d}>
            <line x1={x1} y1={y1} x2={x2} y2={y2} className="lc-tick" />
            <text x={x} y={y} className={`lc-dir${d.length === 1 ? ' major' : ''}`} textAnchor="middle" dominantBaseline="middle">{d}</text>
          </g>
        )
      })}
      {Object.entries(byDir).map(([d, list]) => {
        const base = DIR_ANGLE[d] ?? 0
        return list.map((b, i) => {
          // 同方位的候选沿半径错开，避免叠字
          const r = R * 0.55 + (i % 3) * 22 + 10
          const spread = (i - (list.length - 1) / 2) * 6
          const [x, y] = pt(base + spread, r)
          const cls = b.leaf === 'qimen' ? 'qm' : 'lr'
          return (
            <g key={`${d}-${i}`} className={`lc-pin ${cls}`}>
              <circle cx={x} cy={y} r={3.2} />
              <text x={x} y={y - 8} textAnchor="middle" className="lc-pin-t">{b.element}</text>
            </g>
          )
        })
      })}
      <text x={C} y={C} textAnchor="middle" dominantBaseline="middle" className="lc-center">寻</text>
    </svg>
  )
}

export function LocativeView() {
  const [t, setT] = useState(nowFields)
  const [seed, setSeed] = useState<number | null>(null)
  const [category, setCategory] = useState('')
  const [res, setRes] = useState<Locative | null>(null)
  const [advice, setAdvice] = useState<Interpretation | null>(null)
  const [busy, setBusy] = useState(false)
  const [aBusy, setABusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)

  const body = () => ({ t_ask: t, seed, category: category.trim() || null })
  async function cast() {
    setBusy(true); setErr(null); setAdvice(null)
    try { setRes(await fetchLocative(body())) }
    catch (e) { setErr(e instanceof Error ? e.message : String(e)) }
    finally { setBusy(false) }
  }
  async function ask() {
    setABusy(true)
    try { setAdvice(await fetchLocativeAdvice(body())) }
    catch { setAdvice(null) }
    finally { setABusy(false) }
  }
  const draw = () => setSeed(Math.floor(Math.random() * 0xffff_ffff))

  return (
    <section className="card">
      <div className="lp-sec-t">寻（寻方位）· 问的此刻起六壬 / 奇门课，把落宫翻成方位</div>
      <div className="ev-form">
        <label className="field"><span>年</span><input type="number" value={t.year} style={{ width: 68 }} onChange={(e) => setT({ ...t, year: Number(e.target.value) })} /></label>
        <label className="field"><span>月</span><input type="number" value={t.month} style={{ width: 52 }} onChange={(e) => setT({ ...t, month: Number(e.target.value) })} /></label>
        <label className="field"><span>日</span><input type="number" value={t.day} style={{ width: 52 }} onChange={(e) => setT({ ...t, day: Number(e.target.value) })} /></label>
        <label className="field"><span>时</span><input type="number" value={t.hour} style={{ width: 52 }} onChange={(e) => setT({ ...t, hour: Number(e.target.value) })} /></label>
        <label className="field"><span>分</span><input type="number" value={t.minute} style={{ width: 52 }} onChange={(e) => setT({ ...t, minute: Number(e.target.value) })} /></label>
        <button className="ev-now" onClick={() => setT(nowFields())}>此刻</button>
        <label className="field ev-q"><span>所寻（只入释义，不参与计算）</span>
          <input type="text" value={category} placeholder="如：寻物" list="lc-cats" onChange={(e) => setCategory(e.target.value)} />
          <datalist id="lc-cats">{CATEGORIES.map((c) => <option key={c} value={c} />)}</datalist>
        </label>
      </div>
      <div className="ev-draw">
        <button className="ev-draw-btn" onClick={draw}>取 机</button>
        <span className="ev-seed">{seed === null ? '未取机 · 以问事此刻派生' : `种子 ${seed}`}</span>
        {seed !== null && <button className="ev-clear" onClick={() => setSeed(null)}>清除</button>}
        <button className="go" onClick={() => void cast()} disabled={busy}>{busy ? '起课中…' : '起 课'}</button>
      </div>
      {err && <div className="lp-note" style={{ color: 'var(--shu)' }}>{err}</div>}

      {res && (
        <>
          <div className="ev-meta">
            问于 {res.asked_at.year}-{pad(res.asked_at.month)}-{pad(res.asked_at.day)} {pad(res.asked_at.hour)}:{pad(res.asked_at.minute)}
            {' · '}{res.seed === null ? '未取机' : `取机 ${res.seed}`}
            {res.category && <> · 所寻「{res.category}」</>}
            {' · '}{res.bearings.length} 个方位候选
          </div>
          <div className="lc-top">
            <Compass bearings={res.bearings} />
            <div className="lc-list">
              <table className="el-table">
                <thead><tr><th>要素</th><th>落宫 / 支</th><th>方位</th><th>同宫结构</th></tr></thead>
                <tbody>
                  {res.bearings.map((b, i) => (
                    <tr key={i} className={`lc-row ${b.leaf}`}>
                      <td className="lc-el"><i className="lc-leaf">{b.leaf === 'qimen' ? '奇门' : '六壬'}</i>{b.element}</td>
                      <td className="el-gz">{b.at}</td>
                      <td className="lc-dir-cell">{b.direction}</td>
                      <td className="el-pz">{b.note}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
          <div className="ev-verdict">
            <button className="go interp-btn" onClick={() => void ask()} disabled={aBusy}>
              {aBusy ? '推演中…' : '🔮 往哪个方向（由 LLM 生成）'}
            </button>
            {advice && (
              <>
                <div className="int-text">{advice.text.split('\n').filter((s) => s.trim()).map((para, i) => <p key={i}>{para}</p>)}</div>
                <div className="int-by">{advice.backend} 生成 · 仅供研究与娱乐</div>
              </>
            )}
          </div>
          <details className="lc-raw">
            <summary>各叶原盘（{res.leaves.length} 片）</summary>
            <div className="ev-leaves">
              {res.leaves.map((l) => (
                <section className="ev-leaf" key={l.id}>
                  <div className="ev-leaf-h">{l.name}</div>
                  <LeafChart leaf={l} />
                </section>
              ))}
            </div>
          </details>
          <div className="lp-note">方位由后天八卦九宫（坎北 · 离南 · 震东 · 兑西 + 四维）与十二支方位直接翻出，是确定映射；哪一路取用为吉各家不同，交释义层说明依据。🟡 六壬三传遇流派分歧课式时不出，改列四课上神。</div>
        </>
      )}
    </section>
  )
}
