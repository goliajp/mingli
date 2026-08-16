// 择吉视图：时窗 + 事类 → 逐日择日要素，按建除分档分组，黄道在前。
import { useState } from 'react'
import type { Election, ElectionCandidate, Interpretation } from '../types'
import { fetchElection, fetchElectionAdvice } from '../api/client'

const pad = (n: number) => String(n).padStart(2, '0')

type Ymd = { year: number; month: number; day: number }

/** 今天。 */
function today(): Ymd {
  const d = new Date()
  return { year: d.getFullYear(), month: d.getMonth() + 1, day: d.getDate() }
}

/** 今天起 N 天后。 */
function daysAhead(n: number): Ymd {
  const d = new Date()
  d.setDate(d.getDate() + n)
  return { year: d.getFullYear(), month: d.getMonth() + 1, day: d.getDate() }
}

const CATEGORIES = ['婚嫁', '动土', '开业', '出行', '搬家', '签约', '安葬', '祭祀'] as const

/** 分档的展示顺序与说明。 */
const GRADES: { key: ElectionCandidate['grade']; label: string; note: string }[] = [
  { key: 'Huang', label: '黄道', note: '除 · 危 · 定 · 执' },
  { key: 'Usable', label: '可用', note: '成 · 开' },
  { key: 'Hei', label: '黑道', note: '建 · 满 · 平 · 收' },
  { key: 'Avoid', label: '不可当', note: '破 · 闭' },
]

function DateFields({ label, v, on }: { label: string; v: Ymd; on: (v: Ymd) => void }) {
  return (
    <div className="el-date">
      <span className="el-date-l">{label}</span>
      <input type="number" value={v.year} style={{ width: 68 }} onChange={(e) => on({ ...v, year: Number(e.target.value) })} />
      <i>-</i>
      <input type="number" value={v.month} style={{ width: 48 }} onChange={(e) => on({ ...v, month: Number(e.target.value) })} />
      <i>-</i>
      <input type="number" value={v.day} style={{ width: 48 }} onChange={(e) => on({ ...v, day: Number(e.target.value) })} />
    </div>
  )
}

export function ElectionView() {
  const [start, setStart] = useState<Ymd>(today)
  const [end, setEnd] = useState<Ymd>(() => daysAhead(30))
  const [category, setCategory] = useState('')
  const [res, setRes] = useState<Election | null>(null)
  const [advice, setAdvice] = useState<Interpretation | null>(null)
  const [busy, setBusy] = useState(false)
  const [aBusy, setABusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)

  const body = () => ({
    window_start: { ...start, hour: 12, minute: 0, tz: 8 },
    window_end: { ...end, hour: 12, minute: 0, tz: 8 },
    category: category.trim() || null,
  })

  async function scan() {
    setBusy(true); setErr(null); setAdvice(null)
    try { setRes(await fetchElection(body())) }
    catch (e) { setErr(e instanceof Error ? e.message : String(e)) }
    finally { setBusy(false) }
  }
  async function ask() {
    setABusy(true)
    try { setAdvice(await fetchElectionAdvice(body())) }
    catch { setAdvice(null) }
    finally { setABusy(false) }
  }

  const grouped = res
    ? GRADES.map((g) => ({ ...g, days: res.candidates.filter((c) => c.grade === g.key) }))
    : []

  return (
    <section className="card">
      <div className="lp-sec-t">择（择吉）· 扫一段时窗，逐日出择日要素，按建除十二神通行分档排序</div>
      <div className="el-form">
        <DateFields label="从" v={start} on={setStart} />
        <DateFields label="到" v={end} on={setEnd} />
        <div className="el-quick">
          <button onClick={() => { setStart(today()); setEnd(daysAhead(30)) }}>未来 30 天</button>
          <button onClick={() => { setStart(today()); setEnd(daysAhead(90)) }}>未来 90 天</button>
        </div>
        <label className="field el-cat"><span>所办之事（只入释义，不参与排序）</span>
          <input type="text" value={category} placeholder="如：婚嫁" list="el-cats" onChange={(e) => setCategory(e.target.value)} />
          <datalist id="el-cats">{CATEGORIES.map((c) => <option key={c} value={c} />)}</datalist>
        </label>
        <button className="go" onClick={() => void scan()} disabled={busy}>{busy ? '扫描中…' : '择 日'}</button>
      </div>
      {err && <div className="lp-note" style={{ color: 'var(--shu)' }}>{err}</div>}

      {res && (
        <>
          <div className="el-meta">
            {res.window_start.year}-{pad(res.window_start.month)}-{pad(res.window_start.day)} → {res.window_end.year}-{pad(res.window_end.month)}-{pad(res.window_end.day)}
            {' · '}扫 {res.scanned_days} 天
            {res.category && <> · 所办「{res.category}」</>}
            {' · '}排序只按建除分档，事类宜忌交释义层
          </div>
          <div className="el-groups">
            {grouped.map((g) => (
              <section className={`el-group g-${g.key.toLowerCase()}`} key={g.key}>
                <div className="el-group-h">
                  <b>{g.label}</b><span className="el-group-note">{g.note}</span><span className="el-group-n">{g.days.length} 天</span>
                </div>
                {g.days.length === 0 && <div className="el-empty">时窗内无</div>}
                {g.days.length > 0 && (
                  <table className="el-table">
                    <thead>
                      <tr><th>日期</th><th>日柱</th><th>建除</th><th>宿</th><th>天乙</th><th>彭祖百忌</th></tr>
                    </thead>
                    <tbody>
                      {g.days.map((c) => (
                        <tr key={`${c.year}-${c.month}-${c.day}`}>
                          <td className="el-d">{pad(c.month)}-{pad(c.day)}</td>
                          <td className="el-gz">{c.day_ganzhi}</td>
                          <td className="el-jc">{c.jianchu}</td>
                          <td>{c.mansion}</td>
                          <td className="el-ty">{c.tianyi[0]} {c.tianyi[1]}</td>
                          <td className="el-pz">{c.pengzu_gan} · {c.pengzu_zhi}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </section>
            ))}
          </div>
          <div className="ev-verdict">
            <button className="go interp-btn" onClick={() => void ask()} disabled={aBusy}>
              {aBusy ? '斟酌中…' : '🔮 挑几个日子（由 LLM 生成）'}
            </button>
            {advice && (
              <>
                <div className="int-text">{advice.text.split('\n').filter((s) => s.trim()).map((para, i) => <p key={i}>{para}</p>)}</div>
                <div className="int-by">{advice.backend} 生成 · 仅供研究与娱乐</div>
              </>
            )}
          </div>
          <div className="lp-note">分档出自通行口诀「建满平收黑，除危定执黄，成开皆可用，破闭不可当」（🟡 另有一说把成开并入黄道）。具体事类各宜何神各家出入大，引擎不合成总分，由释义层结合所办之事去说。</div>
        </>
      )}
    </section>
  )
}
