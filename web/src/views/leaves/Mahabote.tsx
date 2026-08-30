// 缅甸 Mahabote：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { Section, Stat } from './shared'

export interface MahaboteChart { core: number; house: string; planet: string; weekday: string; weekday_index: number; myanmar_year: number }
const MHB_HOUSES = ['Binga', 'Atun', 'Yaza', 'Adipati', 'Marana', 'Thike', 'Puti']
export function Mahabote({ c }: { c: MahaboteChart }) {
  return (
    <div className="lp">
      <Section title="七宫（本命宫高亮）">
        <div className="mhb-houses">
          {MHB_HOUSES.map((h, i) => <div className={`mhb${i === c.core ? ' hi' : ''}`} key={h}><span className="mhb-i">{i}</span><span className="mhb-n">{h}</span></div>)}
        </div>
      </Section>
      <Section title="本命">
        <div className="kv-grid">
          <Stat k="本命宫" v={c.house} hi />
          <Stat k="核心数" v={`${c.core} / 7`} />
          <Stat k="行星（八天週）" v={c.planet} />
          <Stat k="出生星期" v={c.weekday} />
          <Stat k="缅历年" v={c.myanmar_year} />
        </div>
      </Section>
    </div>
  )
}
