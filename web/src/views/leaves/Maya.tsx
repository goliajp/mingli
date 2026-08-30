// 玛雅历：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { Section, Stat } from './shared'

export interface MayaChart { jdn: number; tzolkin_number: number; tzolkin_name: string; tzolkin_round: number; haab_day: number; haab_month: string; long_count: number[] }
const LC_UNITS = ['baktun', 'katun', 'tun', 'winal', 'kin']
export function Maya({ c }: { c: MayaChart }) {
  return (
    <div className="lp">
      <Section title="Long Count（长计历）">
        <div className="maya-lc">{c.long_count.join(' . ')}</div>
        <div className="lc-units">{LC_UNITS.map((u, i) => <span key={u}><b>{c.long_count[i]}</b>{u}</span>)}</div>
      </Section>
      <Section title="两套循环历">
        <div className="maya-cals">
          <div className="maya-cal"><span className="ml">Tzolkʼin · 神圣历</span><b>{c.tzolkin_number} {c.tzolkin_name}</b><i>环位 {c.tzolkin_round} / 260</i></div>
          <div className="maya-cal"><span className="ml">Haab · 太阳历</span><b>{c.haab_day} {c.haab_month}</b><i>365 日</i></div>
        </div>
        <div className="kv-grid"><Stat k="儒略日 JDN" v={c.jdn.toLocaleString()} /></div>
      </Section>
    </div>
  )
}
