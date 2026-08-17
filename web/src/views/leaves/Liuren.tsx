// 大六壬：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { BRANCHES, STEMS, Stat } from './shared'

export interface LiurenChart { day_stem: number; day_branch: number; hour_branch: number; month_general: number; month_general_name: string; offset: number; heaven: number[]; courses: { down: number; up: number }[]; pattern: string; pattern_label: string; transmission: number[] | null }
export function Liuren({ c }: { c: LiurenChart }) {
  // 天地盘圆环：12 地支地盘均布，天盘在外圈
  const S = 300, cx = S / 2, cy = S / 2
  const ang = (g: number) => (-90 - g * 30) * Math.PI / 180 // 子在上、顺时针
  const pt = (g: number, r: number) => ({ x: cx + r * Math.cos(ang(g)), y: cy + r * Math.sin(ang(g)) })
  return (
    <div className="lp">
      <div className="liuren-wrap">
        <svg className="plate-svg" viewBox={`0 0 ${S} ${S}`} width="300" height="300">
          <circle cx={cx} cy={cy} r={138} className="w-ring" />
          <circle cx={cx} cy={cy} r={104} className="w-ring" />
          <circle cx={cx} cy={cy} r={70} className="w-ring faint" />
          {BRANCHES.map((b, g) => {
            const he = pt(g, 121), ea = pt(g, 87)
            return <g key={g}>
              <text x={he.x} y={he.y} className="pt-h" dominantBaseline="central" textAnchor="middle">{BRANCHES[c.heaven[g]]}</text>
              <text x={ea.x} y={ea.y} className="pt-e" dominantBaseline="central" textAnchor="middle">{b}</text>
            </g>
          })}
          <text x={cx} y={cy - 8} className="plate-c1" dominantBaseline="central" textAnchor="middle">{STEMS[c.day_stem]}{BRANCHES[c.day_branch]}</text>
          <text x={cx} y={cy + 12} className="plate-c2" dominantBaseline="central" textAnchor="middle">{c.pattern_label}</text>
        </svg>
        <div className="liuren-side">
          <div className="kv-grid">
            <Stat k="日干支" v={`${STEMS[c.day_stem]}${BRANCHES[c.day_branch]}`} />
            <Stat k="月将·占时" v={`${c.month_general_name}·${BRANCHES[c.hour_branch]}`} />
            <Stat k="课式" v={c.pattern_label} hi />
          </div>
          <div className="lp-sec-t" style={{ marginTop: 12 }}>四课（右起一→四）</div>
          <div className="courses">
            {[...c.courses].reverse().map((q, i) => (
              <div className="course" key={i}><span className="cu">{BRANCHES[q.up]}</span><span className="cd">{BRANCHES[q.down]}</span></div>
            ))}
          </div>
          <div className="lp-sec-t" style={{ marginTop: 12 }}>三传</div>
          <div className="san-chuan">{c.transmission ? c.transmission.map((t, i) => <span key={i} className="sc-cell">{['初', '中', '末'][i]} {BRANCHES[t]}</span>) : <span className="und">🟡 该课式（{c.pattern_label}）取传流派分歧，未强编</span>}</div>
        </div>
      </div>
    </div>
  )
}
