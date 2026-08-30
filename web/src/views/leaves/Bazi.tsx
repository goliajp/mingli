// 四柱八字：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { useMemo } from 'react'
import type { BaziChart } from '../../types'
import { WUXING, WUXING_COLOR } from '../../lib/display'
import { Section, Stat } from './shared'

export function BaziView({ c }: { c: BaziChart }) {
  const cols: [string, BaziChart['year']][] = [['年', c.year], ['月', c.month], ['日', c.day], ['时', c.hour]]
  const counts = useMemo(() => {
    const m: Record<string, number> = { 木: 0, 火: 0, 土: 0, 金: 0, 水: 0 }
    for (const p of [c.year, c.month, c.day, c.hour]) { m[p.stem_wuxing]++; m[p.branch_wuxing]++ }
    return m
  }, [c])
  return (
    <div className="lp">
      <Section title="四柱">
        <div className="pillars">
          {cols.map(([name, p]) => (
            <div className={`pillar${name === '日' ? ' is-day' : ''}`} key={name}>
              <div className="pname">{name}柱</div>
              <div className="tengod">{p.ten_god}</div>
              <div className="char gan" style={{ color: WUXING_COLOR[p.stem_wuxing] }}>{p.stem}</div>
              <div className="char zhi" style={{ color: WUXING_COLOR[p.branch_wuxing] }}>{p.branch}</div>
              <div className="wx"><i style={{ background: WUXING_COLOR[p.stem_wuxing] }} />{p.stem_wuxing}<i style={{ background: WUXING_COLOR[p.branch_wuxing] }} />{p.branch_wuxing}</div>
            </div>
          ))}
        </div>
        <div className="kv-grid"><Stat k="日主" v={`${c.day_master}（${c.day_master_wuxing}）`} hi /></div>
      </Section>
      <Section title="五行分布">
        <div className="wuxing-bars">
          {WUXING.map((w) => (
            <div className="wb-row" key={w}>
              <span className="wb-name" style={{ color: WUXING_COLOR[w] }}>{w}</span>
              <span className="wb-track"><span className="wb-fill" style={{ width: `${(counts[w] / 8) * 100}%`, background: WUXING_COLOR[w] }} /></span>
              <span className="wb-num">{counts[w]}</span>
            </div>
          ))}
        </div>
      </Section>
      {c.dayun && (
        <Section title={`大运 · ${c.dayun.forward ? '顺行' : '逆行'} · 起运 ${c.dayun.start_age_years} 岁`}>
          <div className="dy-row">
            {c.dayun.pillars.map((d) => <div className="dy-cell" key={d.start_age}><div className="dy-age">{d.start_age}岁</div><div className="dy-gz">{d.ganzhi}</div></div>)}
          </div>
        </Section>
      )}
    </div>
  )
}
