// 国运视图：奠基时刻 → 立国盘 + 太乙行宫时间线（三年一宫、廿四年一周）+ 目标年年度盘 → 「势」。
import { useState } from 'react'
import type { Interpretation, Mundane, YearStep } from '../types'
import { fetchMundane, fetchMundaneAdvice } from '../api/client'
import { LeafChart } from '../leaves'

/** 八宫在时间线上的行序：按太乙阳遁顺行 1→2→3→4→6→7→8→9（不入 5）。 */
const PALACE_ROWS = [1, 2, 3, 4, 6, 7, 8, 9]

const SANCAI_CLS: Record<string, string> = { 理天: 'tian', 理地: 'di', 理人: 'ren' }

function Timeline({ steps, target }: { steps: YearStep[]; target: number }) {
  const cols = steps.length
  return (
    <div className="mu-tl" style={{ gridTemplateColumns: `56px repeat(${cols}, minmax(18px, 1fr))` }}>
      <div className="mu-tl-corner" />
      {steps.map((s) => (
        <div key={`h${s.year}`} className={`mu-tl-year${s.year === target ? ' on' : ''}${s.enters_palace ? ' enter' : ''}`} title={`${s.year} · 立国第 ${s.age} 年`}>
          {s.enters_palace || s.year === target ? String(s.year).slice(2) : ''}
        </div>
      ))}
      {PALACE_ROWS.map((p) => (
        <div key={`r${p}`} className="mu-tl-row" style={{ display: 'contents' }}>
          <div className="mu-tl-lab">{steps.find((s) => s.palace === p)?.gua ?? ''}{p}</div>
          {steps.map((s) => (
            <div
              key={`${p}-${s.year}`}
              className={`mu-cell${s.palace === p ? ` on ${SANCAI_CLS[s.sancai] ?? ''}` : ''}${s.year === target ? (s.palace === p ? ' target' : ' target-col') : ''}`}
              title={s.palace === p ? `${s.year} · ${s.gua}${s.palace} 宫第 ${s.year_in_palace} 年 · ${s.sancai}` : `${s.year}`}
            >{s.palace === p ? s.year_in_palace : ''}</div>
          ))}
        </div>
      ))}
    </div>
  )
}

export function MundaneView() {
  const [f, setF] = useState({ year: 1949, month: 10, day: 1, hour: 15, minute: 0, tz: 8 })
  const [geo, setGeo] = useState({ latitude: 39.9, longitude: 116.4 })
  const [target, setTarget] = useState<number>(new Date().getFullYear())
  const [span, setSpan] = useState<24 | 48 | 72>(24)
  const [res, setRes] = useState<Mundane | null>(null)
  const [advice, setAdvice] = useState<Interpretation | null>(null)
  const [busy, setBusy] = useState(false)
  const [aBusy, setABusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)

  const body = () => ({ founded_at: f, latitude: geo.latitude, longitude: geo.longitude, target_year: target, span })
  async function run() {
    setBusy(true); setErr(null); setAdvice(null)
    try { setRes(await fetchMundane(body())) }
    catch (e) { setErr(e instanceof Error ? e.message : String(e)) }
    finally { setBusy(false) }
  }
  async function ask() {
    setABusy(true)
    try { setAdvice(await fetchMundaneAdvice(body())) }
    catch { setAdvice(null) }
    finally { setABusy(false) }
  }
  const num = (k: keyof typeof f, w: number) => (
    <label className="field"><span>{k === 'year' ? '年' : k === 'month' ? '月' : k === 'day' ? '日' : k === 'hour' ? '时' : '分'}</span>
      <input type="number" value={f[k]} style={{ width: w }} onChange={(e) => setF({ ...f, [k]: Number(e.target.value) })} />
    </label>
  )
  const a = res?.annual ?? null

  return (
    <section className="card">
      <div className="lp-sec-t">群 / 国（国运）· 奠基时刻起立国盘 · 沿年份铺太乙行宫 · 目标年年度盘</div>
      <div className="ev-form">
        {num('year', 68)}{num('month', 52)}{num('day', 52)}{num('hour', 52)}{num('minute', 52)}
        <label className="field"><span>纬度</span><input type="number" step="0.01" value={geo.latitude} style={{ width: 76 }} onChange={(e) => setGeo({ ...geo, latitude: Number(e.target.value) })} /></label>
        <label className="field"><span>经度</span><input type="number" step="0.01" value={geo.longitude} style={{ width: 82 }} onChange={(e) => setGeo({ ...geo, longitude: Number(e.target.value) })} /></label>
        <label className="field"><span>目标年</span><input type="number" value={target} style={{ width: 72 }} onChange={(e) => setTarget(Number(e.target.value))} /></label>
        <label className="field"><span>时间线</span>
          <select value={span} onChange={(e) => setSpan(Number(e.target.value) as 24 | 48 | 72)} style={{ width: 96 }}>
            <option value={24}>24 年 · 一周</option><option value={48}>48 年 · 两周</option><option value={72}>72 年 · 三期</option>
          </select>
        </label>
        <button className="go" onClick={() => void run()} disabled={busy}>{busy ? '推演中…' : '推 演'}</button>
      </div>
      {err && <div className="lp-note" style={{ color: 'var(--shu)' }}>{err}</div>}

      {res && (
        <>
          <div className="ev-meta">
            奠基 {res.founded_at.year}-{String(res.founded_at.month).padStart(2, '0')}-{String(res.founded_at.day).padStart(2, '0')}
            {' · '}时间线 {res.span} 年 · 目标年 {res.target_year}
            {a ? <> · 立国第 {a.age} 年</> : <> · 目标年早于奠基，无年度盘</>}
          </div>

          {a && (
            <div className="mu-annual">
              <div className="mu-annual-big">{a.gua}<span>{a.palace}</span></div>
              <div className="mu-annual-body">
                <div className="mu-annual-l">{res.target_year} 年度盘</div>
                <div className="mu-annual-t">太乙居 <b>{a.gua}{a.palace} 宫</b>第 <b>{a.year_in_palace}</b> 年 · <b className={`mu-sc ${SANCAI_CLS[a.sancai] ?? ''}`}>{a.sancai}</b> · {a.yang_dun ? '阳遁' : '阴遁'}</div>
                <div className="mu-annual-n">
                  {a.year_in_palace === 1 ? '换宫之年——新一宫的起点' : a.year_in_palace === 3 ? '本宫末年——明年换宫' : '本宫中段'}
                  {' · '}距周期起点 {a.age % 24} / 24 年
                </div>
              </div>
            </div>
          )}

          <div className="lp-sec-t" style={{ marginTop: 14 }}>太乙行宫时间线 {res.timeline[0]?.year}–{res.timeline[res.timeline.length - 1]?.year} · 每格 = 一年，数字 = 入宫第几年，色 = 三才</div>
          <Timeline steps={res.timeline} target={res.target_year} />
          <div className="mu-legend">
            <span className="mu-key tian">理天</span><span className="mu-key di">理地</span><span className="mu-key ren">理人</span>
            <span className="mu-key-note">列头只标换宫之年与目标年（红框）；窗口按廿四年对齐并总含目标年；三年一宫、不入中五</span>
          </div>

          <div className="ev-verdict">
            <button className="go interp-btn" onClick={() => void ask()} disabled={aBusy}>
              {aBusy ? '推演中…' : '🔮 势（由 LLM 生成）'}
            </button>
            {advice && (
              <>
                <div className="int-text">{advice.text.split('\n').filter((s) => s.trim()).map((para, i) => <p key={i}>{para}</p>)}</div>
                <div className="int-by">{advice.backend} 生成 · 仅供研究与娱乐</div>
              </>
            )}
          </div>

          <details className="lc-raw">
            <summary>立国盘（{res.founding.length} 片）</summary>
            <div className="ev-leaves">
              {res.founding.map((l) => (
                <section className="ev-leaf" key={l.id}>
                  <div className="ev-leaf-h">{l.name}</div>
                  <LeafChart leaf={l} />
                </section>
              ))}
            </div>
          </details>
          <div className="lp-note">这是周期结构的描述，不是对现实政体的预言：太乙三年居一宫、廿四年一周、七十二年三期，宫位由积年确定。🟡 太乙诸神（文昌 / 始击 / 主客等）与宫位吉凶传统引擎未收；立国盘中的奇门 / 占星只作背景。</div>
        </>
      )}
    </section>
  )
}
