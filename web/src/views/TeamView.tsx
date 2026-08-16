// 合盘 / 团队视图。
import { useEffect, useState } from 'react'
import { fetchTeam, fetchTeamInterpretation } from '../api/client'
import type { Interpretation, TeamResult } from '../types'
import { WUXING_COLOR } from '../leaves'

// 跨叶相关性热力图
// 合盘 / 团队：N 人输入 → 团队五行画像 + N×N 互补矩阵。
// 默认 2 人对照（经典合婚）；可加成员到 12 人（团队五行画像）。
export function TeamView() {
  const [members, setMembers] = useState([
    { name: 'A', year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8, gender: 'male' as const },
    { name: 'B', year: 1992, month: 3, day: 22, hour: 9, minute: 0, tz: 8, gender: 'female' as const },
  ])
  const [res, setRes] = useState<TeamResult | null>(null)
  const [busy, setBusy] = useState(false)
  const [interp, setInterp] = useState<Interpretation | null>(null)
  const [interpBusy, setInterpBusy] = useState(false)
  async function calc() {
    setBusy(true); setInterp(null)
    try {
      setRes(await fetchTeam(members))
    } catch { setRes(null) } finally { setBusy(false) }
  }
  async function interpretTeam() {
    setInterpBusy(true)
    try {
      setInterp(await fetchTeamInterpretation(members))
    } catch { setInterp(null) } finally { setInterpBusy(false) }
  }
  useEffect(() => { void calc() }, []) // eslint-disable-line react-hooks/exhaustive-deps
  const upd = (i: number, k: keyof typeof members[number], v: string | number) => {
    setMembers((arr) => arr.map((m, j) => j === i ? { ...m, [k]: typeof m[k] === 'number' ? Number(v) : v } : m))
  }
  const add = () => members.length < 12 && setMembers([...members, { name: `成员${members.length + 1}`, year: 1990, month: 1, day: 1, hour: 12, minute: 0, tz: 8, gender: 'male' as const }])
  const rm = (i: number) => members.length > 1 && setMembers(members.filter((_, j) => j !== i))
  return (
    <section className="card">
      <div className="lp-sec-t">合盘 / 团队 · N 人五行画像 + 互补矩阵（计算 DET、结论 INT）</div>
      <div className="team-members">
        {members.map((m, i) => (
          <div className="team-member-row" key={i}>
            <input className="team-name" type="text" value={m.name} onChange={(e) => upd(i, 'name', e.target.value)} placeholder="名称" />
            <input type="number" value={m.year} onChange={(e) => upd(i, 'year', e.target.value)} style={{ width: 68 }} />
            <input type="number" value={m.month} onChange={(e) => upd(i, 'month', e.target.value)} style={{ width: 52 }} />
            <input type="number" value={m.day} onChange={(e) => upd(i, 'day', e.target.value)} style={{ width: 52 }} />
            <input type="number" value={m.hour} onChange={(e) => upd(i, 'hour', e.target.value)} style={{ width: 52 }} />
            <input type="number" value={m.minute} onChange={(e) => upd(i, 'minute', e.target.value)} style={{ width: 52 }} />
            <select value={m.gender} onChange={(e) => upd(i, 'gender', e.target.value)} style={{ width: 56 }}>
              <option value="male">男</option><option value="female">女</option>
            </select>
            <button className="team-rm" onClick={() => rm(i)} disabled={members.length <= 1}>×</button>
          </div>
        ))}
        <div className="team-actions">
          <button onClick={add} disabled={members.length >= 12}>+ 添加成员</button>
          <button className="go" onClick={() => void calc()} disabled={busy}>{busy ? '合盘中…' : '合 盘'}</button>
        </div>
      </div>
      {res && (
        <div style={{ marginTop: 18 }}>
          <div className="lp-sec-t">团队五行画像</div>
          <div className="wx-rows">
            {(['木', '火', '土', '金', '水'] as const).map((n) => {
              const v = ({ '木': res.team_wuxing.wood, '火': res.team_wuxing.fire, '土': res.team_wuxing.earth, '金': res.team_wuxing.metal, '水': res.team_wuxing.water } as Record<string, number>)[n] ?? 0
              return (
                <div className="wx-row" key={n}>
                  <span className="wx-row-n" style={{ color: WUXING_COLOR[n] }}>{n}</span>
                  <div className="wx-row-bar"><i style={{ width: `${v}%`, background: WUXING_COLOR[n] }} /></div>
                  <span className="wx-row-v">{v}%</span>
                </div>
              )
            })}
          </div>
          <div className="kv-grid" style={{ marginTop: 8 }}>
            <div className="stat"><span className="stat-k">团队最缺</span><span className="stat-v" style={{ color: WUXING_COLOR[res.team_weakest.wuxing] }}>{res.team_weakest.wuxing} {res.team_weakest.pct}%</span></div>
            <div className="stat"><span className="stat-k">团队最旺</span><span className="stat-v" style={{ color: WUXING_COLOR[res.team_strongest.wuxing] }}>{res.team_strongest.wuxing} {res.team_strongest.pct}%</span></div>
          </div>
          <div className="lp-sec-t" style={{ marginTop: 14 }}>互补矩阵 · 行 = 用神所属者，列 = 供给者（% 越高 = 列对行帮越大）</div>
          <table className="team-matrix">
            <thead>
              <tr><th></th>{res.members.map((m, j) => <th key={j}>{m.name}</th>)}</tr>
            </thead>
            <tbody>
              {res.members.map((mi, i) => (
                <tr key={i}>
                  <th>{mi.name}<i className="m-yong" style={{ color: WUXING_COLOR[mi.yongshen.primary_wuxing] }}>{mi.yongshen.primary_wuxing}</i></th>
                  {res.complement_matrix[i].map((v, j) => {
                    const intensity = Math.min(v, 50) / 50
                    return (
                      <td key={j} style={{ background: `rgba(95, 179, 191, ${intensity * 0.55})` }}>
                        {v}<i>%</i>
                      </td>
                    )
                  })}
                </tr>
              ))}
            </tbody>
          </table>
          <div className="lp-note" style={{ paddingTop: 10, fontSize: 14 }}>
            <b>「互补度」≠ 「合不合」</b>。这里只给客观指标（j 的盘里有多少 i 的用神），不下「适合 / 不适合」结论。
            🟡 团队五行均衡度有传统依据但无统一量化标准；合婚/合伙契合度可结合主用神互补来评估，仅供研究与娱乐。
          </div>
          <div className="team-interp">
            <button className="go interp-btn" onClick={() => void interpretTeam()} disabled={interpBusy}>
              {interpBusy ? '释义生成中…' : '🔮 团队释义（由 LLM 生成）'}
            </button>
            {interp && (
              <div className="int-block">
                <div className="int-text">{interp.text.split('\n').filter((s) => s.trim()).map((para, i) => <p key={i}>{para}</p>)}</div>
                <div className="int-by">🔮 INT · backend = {interp.backend} · 仅供研究与娱乐</div>
              </div>
            )}
          </div>
        </div>
      )}
    </section>
  )
}
