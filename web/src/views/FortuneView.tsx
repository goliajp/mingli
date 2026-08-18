// 运势视图：t 时刻切片 + 百年用神供给曲线。
import type { FortuneResponse, Pillar } from '../types'
import { WUXING_COLOR } from '../lib/display'
import { JUDGMENT_FILL, JudgmentChip } from '../components/JudgmentChip'
import { YunCell } from '../components/YunCell'
import { gzWuxing } from '../lib/ganzhi'

// Fortune 视图：t 时刻运势切片 + 100 年用神供给曲线。
// 用神喜忌的完整闭环：本命静态喜忌（出生即定） → t 时刻供给度（拨杆动跟动）→ 100 年时序（看一生）。
// 主用神供给度高 = 该时段流年大运得用神之利 = **吉**；忌神供给度高 = **凶**。算法给出 5 等级判读（大吉/吉/平/凶/大凶）。
export function FortuneView({ fortune, age, onBackToNatal }: {
  fortune: FortuneResponse | null
  age: number
  onBackToNatal: () => void
}) {
  if (!fortune) return <section className="card fortune"><div className="fortune-load">运势切片加载中…</div></section>
  const { at, timeline, dasha, progression } = fortune
  const ys = at.natal.yongshen
  const primaryColor = WUXING_COLOR[ys.primary_wuxing] ?? '#888'
  const secondaryColor = ys.secondary_wuxing ? (WUXING_COLOR[ys.secondary_wuxing] ?? '#888') : '#888'
  const avoidColor = '#bc4747'
  const deltaSign = at.delta_score > 0 ? '↑' : at.delta_score < 0 ? '↓' : '→'
  const deltaCls = at.delta_score > 0 ? 'pos' : at.delta_score < 0 ? 'neg' : 'zero'
  const dayWx = at.natal.day_master_wuxing

  // 本命四柱（固定底图）
  const natalCols: [string, Pillar][] = [['年', at.natal.year], ['月', at.natal.month], ['日', at.natal.day], ['时', at.natal.hour]]
  // 运层 5 柱（随 t 动：大运 / 流年 / 流月 / 流日 / 流时）
  const yunLayers: { label: string; gz: string; sub?: string }[] = []
  if (at.dayun_ganzhi) yunLayers.push({ label: '大运', gz: at.dayun_ganzhi, sub: at.dayun_step != null ? `第 ${at.dayun_step + 1}/10 步` : undefined })
  yunLayers.push({ label: '流年', gz: at.flow_year_ganzhi })
  yunLayers.push({ label: '流月', gz: at.t_chart.month.ganzhi })
  yunLayers.push({ label: '流日', gz: at.t_chart.day.ganzhi })
  yunLayers.push({ label: '流时', gz: at.t_chart.hour.ganzhi })

  // 大运 10 步条（横向时间轴）
  const dayunPillars = at.natal.dayun?.pillars ?? []

  // 重大节点：大吉(score≥+15) + 大凶(≤-15)；最多 8 个每类
  const milestones = {
    daji: timeline.filter((p) => p.judgment.level === '大吉').slice(0, 8),
    daxiong: timeline.filter((p) => p.judgment.level === '大凶').slice(0, 8),
  }

  // SVG 折线图参数
  const W = 880, H = 220, padL = 48, padR = 16, padT = 18, padB = 32
  const innerW = W - padL - padR, innerH = H - padT - padB
  const maxAge = timeline.length - 1
  const xOf = (a: number) => padL + (innerW * a) / Math.max(1, maxAge)
  const yOf = (pct: number) => padT + innerH * (1 - Math.min(100, Math.max(0, pct)) / 100)
  const pathOf = (key: 'primary_supply_pct' | 'secondary_supply_pct' | 'avoid_supply_pct') => {
    const pts: string[] = []
    for (const p of timeline) {
      const v = p[key]
      if (v == null) continue
      pts.push(`${pts.length === 0 ? 'M' : 'L'}${xOf(p.age).toFixed(1)},${yOf(v as number).toFixed(1)}`)
    }
    return pts.join(' ')
  }
  // 大运分段竖线（每步起 age）
  const dayunMarks = dayunPillars.map((p) => p.start_age)
  // playhead = 当前 age
  const playX = xOf(Math.min(maxAge, Math.max(0, Math.floor(age))))

  return (
    <section className="card fortune">
      <header className="fortune-head">
        <div className="fortune-title">
          <span className="fortune-name">运 · 八字行运</span>
          <span className="fortune-sub">本命四柱 + t 时刻流年/月/日/时 + 大运段 + 用神供给曲线</span>
        </div>
        <JudgmentChip level={at.judgment.level} score={at.judgment.score} big />
        <button className="back-natal" onClick={onBackToNatal}>← 回「命」</button>
      </header>

      <div className="fortune-judgment-summary">
        <span className="fjs-l">当前 {at.age_years.toFixed(2)} 岁 · 流年 <b style={{color:'#bb6'}}>{at.flow_year_ganzhi}</b></span>
        <span>{at.judgment.summary}</span>
      </div>

      {/* —— 命 / 运 双行四柱对比 —— */}
      <div className="fortune-pillars">
        <div className="fortune-pillars-row ming">
          <div className="fortune-pillars-label">
            <span className="ming-tag">命</span>
            <span className="fortune-pillars-l">本命四柱 · 出生即定</span>
          </div>
          <div className="fortune-pillars-cells">
            {natalCols.map(([lab, p]) => (
              <div className="natal-cell" key={lab}>
                <div className="natal-cell-l">{lab}柱</div>
                <div className="natal-cell-gz">
                  <b style={{ color: WUXING_COLOR[p.stem_wuxing] }}>{p.ganzhi[0]}</b>
                  <b style={{ color: WUXING_COLOR[p.branch_wuxing] }}>{p.ganzhi[1]}</b>
                </div>
                <div className="natal-cell-sub">{p.ten_god} · {p.day_twelve}</div>
                {p.shensha.length > 0 && (
                  <div className="natal-cell-shensha">{p.shensha.join(' · ')}</div>
                )}
              </div>
            ))}
          </div>
        </div>

        <div className="fortune-pillars-row yun">
          <div className="fortune-pillars-label">
            <span className="yun-tag">运</span>
            <span className="fortune-pillars-l">t 时刻 5 柱 · 拨杆动 = 此处跟动</span>
          </div>
          <div className="fortune-pillars-cells">
            {yunLayers.map((y) => (
              <YunCell key={y.label} label={y.label} gz={y.gz} sub={y.sub} dayWx={dayWx} hi={y.label === '大运' || y.label === '流年'} />
            ))}
          </div>
        </div>
      </div>

      {/* —— 大运 10 步条 —— */}
      {dayunPillars.length > 0 && (
        <div className="fortune-dayun-row">
          <div className="fortune-dayun-l">大运十步({at.natal.dayun?.forward ? '顺行' : '逆行'} · 起运 {at.natal.dayun?.start_age_years} 岁)</div>
          <div className="fortune-dayun-strip">
            {dayunPillars.map((p, i) => {
              const isActive = at.dayun_step === i
              const wx = gzWuxing(p.ganzhi)
              return (
                <div key={i} className={`dyx-step${isActive ? ' on' : ''}`}>
                  <div className="dyx-step-age">{p.start_age}+</div>
                  <div className="dyx-step-gz">
                    <b style={{ color: WUXING_COLOR[wx[0]] }}>{p.ganzhi[0]}</b>
                    <b style={{ color: WUXING_COLOR[wx[1]] }}>{p.ganzhi[1]}</b>
                  </div>
                  <div className="dyx-step-i">{i + 1}/10</div>
                </div>
              )
            })}
          </div>
        </div>
      )}

      {/* —— 当前 stat + supplies —— */}
      <div className="fortune-now">
        <div className="fortune-now-row">
          <span className="fortune-now-l">运层旺衰</span>
          <b className="fortune-now-v">{at.yun_strength.score}</b>
          <small className="fortune-now-lvl">({at.yun_strength.level})</small>
          <span className="fortune-now-sep">vs 本命</span>
          <b className="fortune-now-v">{at.ming_strength.score}</b>
          <small className="fortune-now-lvl">({at.ming_strength.level})</small>
          <span className={`fortune-delta ${deltaCls}`}>{deltaSign} {Math.abs(at.delta_score)}</span>
        </div>
        <div className="fortune-now-row supplies">
          <div className="supply-stat" style={{ borderColor: primaryColor }}>
            <div className="supply-stat-l">主用神 · {ys.primary_role}</div>
            <div className="supply-stat-v" style={{ color: primaryColor }}>{ys.primary_wuxing}</div>
            <div className="supply-stat-pct">{at.primary_supply_pct}%</div>
          </div>
          {ys.secondary_wuxing && (
            <div className="supply-stat" style={{ borderColor: secondaryColor }}>
              <div className="supply-stat-l">副用神 · {ys.secondary_role}</div>
              <div className="supply-stat-v" style={{ color: secondaryColor }}>{ys.secondary_wuxing}</div>
              <div className="supply-stat-pct">{at.secondary_supply_pct ?? '—'}%</div>
            </div>
          )}
          {ys.avoid_wuxing.map((w, i) => (
            <div key={w} className="supply-stat avoid" style={{ borderColor: avoidColor }}>
              <div className="supply-stat-l">忌神 · 越低越好</div>
              <div className="supply-stat-v" style={{ color: avoidColor }}>{w}</div>
              <div className="supply-stat-pct">{at.avoid_supply_pcts[i] ?? '—'}%</div>
            </div>
          ))}
        </div>
        <div className="fortune-note">{ys.reasoning} <i>· 主用神供给度高 = 拿到喜用 = <b style={{color:'#9bbd6f'}}>吉</b>；忌神供给度高 = <b style={{color:'#bc4747'}}>凶</b>。</i></div>
      </div>

      {/* —— 重大节点：大吉 / 大凶年份 chip 列 —— */}
      {(milestones.daji.length > 0 || milestones.daxiong.length > 0) && (
        <div className="fortune-milestones">
          <div className="fortune-mile-l">百年大节点 <small>（命局所喜/所忌集中的关键年份）</small></div>
          <div className="fortune-mile-row">
            {milestones.daji.length > 0 && (
              <div className="fortune-mile-group daji">
                <span className="fortune-mile-tag">大吉</span>
                {milestones.daji.map((p) => (
                  <span key={p.age} className="fortune-mile-chip daji" title={p.judgment.summary}>
                    {p.age} 岁 · {p.year} <small>{p.flow_year_ganzhi}</small>
                  </span>
                ))}
              </div>
            )}
            {milestones.daxiong.length > 0 && (
              <div className="fortune-mile-group daxiong">
                <span className="fortune-mile-tag">大凶</span>
                {milestones.daxiong.map((p) => (
                  <span key={p.age} className="fortune-mile-chip daxiong" title={p.judgment.summary}>
                    {p.age} 岁 · {p.year} <small>{p.flow_year_ganzhi}</small>
                  </span>
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {/* —— 另一套「运」：Vimshottari 大运/小运。与上面的大运同名不同物 —— */}
      {dasha?.current && (
        <div className="fortune-dasha">
          <div className="fortune-mile-l">
            Vimshottari 大运 · 小运 <small>（印度占星的运，与四柱大运是两套独立算法，此处并列不合成）</small>
          </div>
          <div className="fd-now">
            出生主星 <b>{dasha.birth_lord}</b> · 现行大运 <b>{dasha.current.lord}</b>
            <span className="fd-span">{dasha.current.start_age_years.toFixed(1)} — {dasha.current.end_age_years.toFixed(1)} 岁</span>
          </div>
          <div className="fd-bar" role="img" aria-label="Vimshottari 120 年大运条">
            {dasha.timeline.map((d) => {
              const on = d.lord === dasha.current?.lord && d.start_age_years === dasha.current.start_age_years
              return (
                <i
                  key={`${d.lord}-${d.start_age_years}`}
                  className={on ? 'fd-seg on' : 'fd-seg'}
                  style={{ flexGrow: d.effective_years }}
                  title={`${d.lord} ${d.start_age_years.toFixed(1)}—${d.end_age_years.toFixed(1)} 岁`}
                >
                  {d.effective_years >= 9 ? d.lord : ''}
                </i>
              )
            })}
          </div>
          <div className="fd-antar">
            {dasha.current.antardashas.map((a) => {
              const on = dasha.age_years >= a.start_age_years && dasha.age_years < a.end_age_years
              return (
                <span key={a.lord} className={on ? 'fd-chip on' : 'fd-chip'}>
                  {a.lord} <small>{a.start_age_years.toFixed(1)}—{a.end_age_years.toFixed(1)}</small>
                </span>
              )
            })}
          </div>
          <div className="fortune-note">
            九主星按固定年数循环 120 年，起点由出生时月亮所在 nakshatra 已行比例定；上方一行是当前大运的九步小运。
            本层只出周期位置，不出吉凶
          </div>
        </div>
      )}

      {/* —— 第三条「运」：西洋占星的二次推运。与上面两条并列，不合成 —— */}
      {progression && progression.years.length > 0 && (
        <div className="fortune-dasha">
          <div className="fortune-mile-l">
            二次推运 · 一日一年 <small>（出生后第 N 日的天象代表第 N 年；每 {progression.step} 年一格）</small>
          </div>
          <div className="fd-antar prog">
            {progression.years.map((y) => {
              const sun = y.planets.find((p) => p.name === '太阳')
              const on = Math.abs(y.age - age) < progression.step / 2
              return (
                <span key={y.age} className={on ? 'fd-chip on' : 'fd-chip'} title={`与本命成角 ${y.to_natal.length} 条`}>
                  {y.age} 岁 <small>{sun ? sun.sign : '—'}</small>
                </span>
              )
            })}
          </div>
          <div className="fortune-note">
            推运太阳约 1°/年、推运月亮约 13°/年（故每两三年换一座），所以日主大势、月主节奏。
            这一条与四柱大运、Vimshottari 并列而不合成——三套各自说各自的时间，合成等于替读者选边
          </div>
        </div>
      )}

      <div className="fortune-chart-wrap">
        <div className="fortune-chart-l">100 年用神供给曲线 · 0—{maxAge} 岁</div>
        <svg className="fortune-chart" viewBox={`0 0 ${W} ${H}`} role="img" aria-label="100 年用神供给度时间序列">
          {/* 吉凶分段背景条（按 timeline[i].judgment.level 着色） */}
          {timeline.map((p, i) => {
            const x1 = xOf(p.age)
            const x2 = i + 1 < timeline.length ? xOf(timeline[i + 1].age) : x1
            const fill = JUDGMENT_FILL[p.judgment.level] ?? 'transparent'
            return <rect key={p.age} x={x1} y={padT} width={Math.max(0, x2 - x1)} height={innerH} fill={fill} />
          })}
          {/* y 轴网格 + 标注 */}
          {[0, 25, 50, 75, 100].map((p) => (
            <g key={p}>
              <line x1={padL} x2={W - padR} y1={yOf(p)} y2={yOf(p)} stroke="#3a3a3a" strokeDasharray={p === 0 || p === 100 ? '0' : '2 4'} strokeWidth={p === 0 ? 1 : 0.5} />
              <text x={padL - 6} y={yOf(p) + 3} textAnchor="end" fontSize="9" fill="#888">{p}%</text>
            </g>
          ))}
          {/* x 轴 + 大运分段竖线 */}
          <line x1={padL} x2={W - padR} y1={H - padB} y2={H - padB} stroke="#666" />
          {dayunMarks.map((a) => (
            <g key={a}>
              <line x1={xOf(a)} x2={xOf(a)} y1={padT} y2={H - padB} stroke="#444" strokeDasharray="2 6" />
              <text x={xOf(a)} y={H - padB + 12} textAnchor="middle" fontSize="9" fill="#777">{a}</text>
            </g>
          ))}
          {/* 忌神供给曲线（填充 → 警示） */}
          <path d={pathOf('avoid_supply_pct')} fill="none" stroke={avoidColor} strokeWidth={1.5} strokeOpacity={0.65} strokeDasharray="3 3" />
          {/* 副用神供给曲线 */}
          {ys.secondary_wuxing && (
            <path d={pathOf('secondary_supply_pct')} fill="none" stroke={secondaryColor} strokeWidth={1.5} strokeOpacity={0.85} />
          )}
          {/* 主用神供给曲线（粗，主角） */}
          <path d={pathOf('primary_supply_pct')} fill="none" stroke={primaryColor} strokeWidth={2.4} />
          {/* playhead */}
          <line x1={playX} x2={playX} y1={padT} y2={H - padB} stroke="#ed8c47" strokeWidth={1.5} />
          <circle cx={playX} cy={yOf(at.primary_supply_pct)} r={4} fill={primaryColor} stroke="#fff" strokeWidth={1} />
        </svg>
        <div className="fortune-chart-legend">
          <span className="fortune-leg" style={{ color: primaryColor }}>● 主用神 {ys.primary_wuxing}</span>
          {ys.secondary_wuxing && <span className="fortune-leg" style={{ color: secondaryColor }}>● 副用神 {ys.secondary_wuxing}</span>}
          <span className="fortune-leg" style={{ color: avoidColor }}>— — 忌神最高</span>
          <span className="fortune-leg" style={{ color: '#777' }}>┊ 大运分段</span>
          <span className="fortune-leg" style={{ color: '#ed8c47' }}>┃ 当前年龄</span>
        </div>
        <div className="fortune-chart-foot">
          算力底层：每年 = （本命四柱 + 当前大运柱 + 流年柱）叠加旺衰 → 五行分布对主/副用神/忌神的占比%。
          <b>主用神长期偏高 + 忌神偏低</b>的区段 = 流年大运扶持有力的<b style={{color:'#9bbd6f'}}>吉运段</b>；反之 = <b style={{color:'#bc4747'}}>不利段</b>需谨慎。仅供研究与娱乐。
        </div>
      </div>
    </section>
  )
}
