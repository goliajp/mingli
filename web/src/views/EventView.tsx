// 占事视图：问的此刻 + 取机 + 问句 → 卜筮诸叶各出一盘，再交释义层出「断」。
import { useState } from 'react'
import type { EventCast, Interpretation } from '../types'
import { fetchEvent, fetchEventVerdict } from '../api/client'
import { LeafChart } from '../leaves'

const pad = (n: number) => String(n).padStart(2, '0')

/** 取当下时刻，填进「问的此刻」。 */
function nowFields() {
  const d = new Date()
  return {
    year: d.getFullYear(), month: d.getMonth() + 1, day: d.getDate(),
    hour: d.getHours(), minute: d.getMinutes(), tz: 8,
  }
}

export function EventView() {
  const [t, setT] = useState(nowFields)
  const [seed, setSeed] = useState<number | null>(null)
  const [question, setQuestion] = useState('')
  const [res, setRes] = useState<EventCast | null>(null)
  const [verdict, setVerdict] = useState<Interpretation | null>(null)
  const [busy, setBusy] = useState(false)
  const [vBusy, setVBusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)

  const body = () => ({ t_ask: t, seed, question: question.trim() || null })

  async function cast() {
    setBusy(true); setErr(null); setVerdict(null)
    try { setRes(await fetchEvent(body())) }
    catch (e) { setErr(e instanceof Error ? e.message : String(e)) }
    finally { setBusy(false) }
  }
  async function ask() {
    setVBusy(true)
    try { setVerdict(await fetchEventVerdict(body())) }
    catch { setVerdict(null) }
    finally { setVBusy(false) }
  }
  // 取机：摇一次得一个种子，同一时刻同一种子可复现同一盘。
  const draw = () => setSeed(Math.floor(Math.random() * 0xffff_ffff))

  return (
    <section className="card">
      <div className="lp-sec-t">事（占事）· 问的此刻 + 取机 → 卜筮诸叶同时起盘</div>
      <div className="ev-form">
        <label className="field"><span>年</span><input type="number" value={t.year} style={{ width: 68 }} onChange={(e) => setT({ ...t, year: Number(e.target.value) })} /></label>
        <label className="field"><span>月</span><input type="number" value={t.month} style={{ width: 52 }} onChange={(e) => setT({ ...t, month: Number(e.target.value) })} /></label>
        <label className="field"><span>日</span><input type="number" value={t.day} style={{ width: 52 }} onChange={(e) => setT({ ...t, day: Number(e.target.value) })} /></label>
        <label className="field"><span>时</span><input type="number" value={t.hour} style={{ width: 52 }} onChange={(e) => setT({ ...t, hour: Number(e.target.value) })} /></label>
        <label className="field"><span>分</span><input type="number" value={t.minute} style={{ width: 52 }} onChange={(e) => setT({ ...t, minute: Number(e.target.value) })} /></label>
        <button className="ev-now" onClick={() => setT(nowFields())}>此刻</button>
        <label className="field ev-q"><span>问句（只入释义，不参与计算）</span>
          <input type="text" value={question} placeholder="如：此事成否" onChange={(e) => setQuestion(e.target.value)} />
        </label>
      </div>
      <div className="ev-draw">
        <button className="ev-draw-btn" onClick={draw} title="摇一次得一个取机种子；同一时刻 + 同一种子必得同一盘，事后可复核">取 机</button>
        <span className="ev-seed">{seed === null ? '未取机 · 以问事此刻派生' : `种子 ${seed}`}</span>
        {seed !== null && <button className="ev-clear" onClick={() => setSeed(null)}>清除</button>}
        <button className="go" onClick={() => void cast()} disabled={busy}>{busy ? '起盘中…' : '起 盘'}</button>
      </div>
      {err && <div className="lp-note" style={{ color: 'var(--shu)' }}>{err}</div>}

      {res && (
        <>
          <div className="ev-meta">
            问于 {res.asked_at.year}-{pad(res.asked_at.month)}-{pad(res.asked_at.day)} {pad(res.asked_at.hour)}:{pad(res.asked_at.minute)}
            {' · '}{res.seed === null ? '未取机' : `取机 ${res.seed}`}
            {res.question && <> · 所问「{res.question}」</>}
            {' · '}{res.leaves.length} 片卜筮叶同时起盘
          </div>
          <div className="ev-leaves">
            {res.leaves.map((l) => (
              <section className="ev-leaf" key={l.id}>
                <div className="ev-leaf-h">{l.name}</div>
                <LeafChart leaf={l} />
              </section>
            ))}
          </div>
          <div className="ev-verdict">
            <button className="go interp-btn" onClick={() => void ask()} disabled={vBusy}>
              {vBusy ? '断事中…' : '🔮 断（由 LLM 生成）'}
            </button>
            {verdict && (
              <>
                <div className="int-text">{verdict.text.split('\n').filter((s) => s.trim()).map((para, i) => <p key={i}>{para}</p>)}</div>
                <div className="int-by">{verdict.backend} 生成 · 仅供研究与娱乐</div>
              </>
            )}
          </div>
          <div className="lp-note">诸叶同时刻同取机起盘，指向一致处最可断；相左处交由释义层说明分歧。取机种子是可复现凭据，记下它即可复核此盘。</div>
        </>
      )}
    </section>
  )
}
