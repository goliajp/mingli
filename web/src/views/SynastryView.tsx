// 合盘视图：甲乙两人本命 → 各自旺衰用神 + 互相给对方主用神的供给度 → 「配」。
import { useEffect, useState } from 'react'
import type { Interpretation, Synastry, TeamMember } from '../types'
import { fetchSynastry, fetchSynastryAdvice } from '../api/client'
import { WUXING_COLOR } from '../lib/display'

type Person = { name: string; year: number; month: number; day: number; hour: number; minute: number; tz: number; gender: 'male' | 'female' }

const A0: Person = { name: '甲', year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8, gender: 'male' }
const B0: Person = { name: '乙', year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8, gender: 'female' }

function PersonRow({ p, on }: { p: Person; on: (p: Person) => void }) {
  const num = (k: keyof Person, w: number) => (
    <input type="number" value={p[k] as number} style={{ width: w }} onChange={(e) => on({ ...p, [k]: Number(e.target.value) })} />
  )
  return (
    <div className="team-member-row">
      <input className="team-name" type="text" value={p.name} onChange={(e) => on({ ...p, name: e.target.value })} placeholder="称呼" />
      {num('year', 68)}{num('month', 52)}{num('day', 52)}{num('hour', 52)}{num('minute', 52)}
      <select value={p.gender} onChange={(e) => on({ ...p, gender: e.target.value as Person['gender'] })} style={{ width: 56 }}>
        <option value="male">男</option><option value="female">女</option>
      </select>
    </div>
  )
}

/** 一方的命局摘要卡。 */
function Side({ m, gives, to }: { m: TeamMember; gives: number; to: string }) {
  const wx = m.day_master_wuxing
  return (
    <div className="sy-side">
      <div className="sy-name">{m.name}</div>
      <div className="sy-dm">日主 <b style={{ color: WUXING_COLOR[wx] }}>{m.day_master}</b><span className="sy-dm-wx">（{wx}）</span> · {m.strength.level} {m.strength.score}</div>
      <div className="sy-ys">
        主用神 <b style={{ color: WUXING_COLOR[m.yongshen.primary_wuxing] }}>{m.yongshen.primary_wuxing}</b>
        {m.yongshen.secondary_wuxing && <> · 副 <b style={{ color: WUXING_COLOR[m.yongshen.secondary_wuxing] }}>{m.yongshen.secondary_wuxing}</b></>}
        {' · '}忌 {m.yongshen.avoid_wuxing.join(' / ')}
      </div>
      <div className="sy-give">
        <span className="sy-give-l">给 {to} 的供给</span>
        <div className="sy-bar"><i style={{ width: `${Math.min(100, gives)}%` }} /></div>
        <b className="sy-give-n">{gives}%</b>
      </div>
    </div>
  )
}

export function SynastryView() {
  const [a, setA] = useState<Person>(A0)
  const [b, setB] = useState<Person>(B0)
  const [res, setRes] = useState<Synastry | null>(null)
  const [advice, setAdvice] = useState<Interpretation | null>(null)
  const [busy, setBusy] = useState(false)
  const [aBusy, setABusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)

  const body = () => ({ a, b })
  async function calc() {
    setBusy(true); setErr(null); setAdvice(null)
    try { setRes(await fetchSynastry(body())) }
    catch (e) { setErr(e instanceof Error ? e.message : String(e)) }
    finally { setBusy(false) }
  }
  async function ask() {
    setABusy(true)
    try { setAdvice(await fetchSynastryAdvice(body())) }
    catch { setAdvice(null) }
    finally { setABusy(false) }
  }
  useEffect(() => { void calc() }, []) // eslint-disable-line react-hooks/exhaustive-deps

  const gap = res ? Math.abs(res.a_supplies_b - res.b_supplies_a) : 0

  return (
    <section className="card">
      <div className="lp-sec-t">合（合盘）· 两人本命互供用神 · 计算 DET、结论 INT</div>
      <div className="sy-forms team-members">
        <PersonRow p={a} on={setA} />
        <PersonRow p={b} on={setB} />
      </div>
      <div className="ev-draw">
        <button className="ev-now" onClick={() => { setA(B0); setB(A0) }}>甲乙对调</button>
        <button className="go" onClick={() => void calc()} disabled={busy}>{busy ? '合盘中…' : '合 盘'}</button>
      </div>
      {err && <div className="lp-note" style={{ color: 'var(--shu)' }}>{err}</div>}

      {res && (
        <>
          <div className="sy-pair">
            <Side m={res.detail.members[0]} gives={res.a_supplies_b} to={res.b_name} />
            <div className="sy-mid">
              <div className="sy-arrow">⇄</div>
              <div className="sy-verdict-n">
                {gap >= 10 ? '不对称互补' : gap >= 4 ? '略有偏向' : '大致对等'}
              </div>
              <div className="sy-mid-note">差 {gap} 个百分点</div>
            </div>
            <Side m={res.detail.members[1]} gives={res.b_supplies_a} to={res.a_name} />
          </div>
          <div className="sy-team">
            团队五行画像 · 最缺 <b>{res.detail.team_weakest.wuxing} {res.detail.team_weakest.pct}%</b> · 最旺 <b>{res.detail.team_strongest.wuxing} {res.detail.team_strongest.pct}%</b>
          </div>
          <div className="ev-verdict">
            <button className="go interp-btn" onClick={() => void ask()} disabled={aBusy}>
              {aBusy ? '斟酌中…' : '🔮 配（由 LLM 生成）'}
            </button>
            {advice && (
              <>
                <div className="int-text">{advice.text.split('\n').filter((s) => s.trim()).map((para, i) => <p key={i}>{para}</p>)}</div>
                <div className="int-by">{advice.backend} 生成 · 仅供研究与娱乐</div>
              </>
            )}
          </div>
          <div className="lp-note">「供给度」= 对方盘的五行分布里，我主用神所属那一行占多少——它是客观结构指标，与「合不合」是两回事。🟡 合婚 / 合伙契合度可结合互供与日主生克来评估，但不构成关系建议；占星合盘几何相位待加。</div>
        </>
      )}
    </section>
  )
}
