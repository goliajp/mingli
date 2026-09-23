// 缅甸 Mahabote：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { Note, Section, Stat } from './shared'

export interface MahaboteChart { planet: string; weekday: string; weekday_index: number; myanmar_year: number }
export function Mahabote({ c }: { c: MahaboteChart }) {
  return (
    <div className="lp">
      <Section title="本命">
        <div className="kv-grid">
          <Stat k="行星（八天週）" v={c.planet} hi />
          <Stat k="出生星期" v={c.weekday} />
          <Stat k="缅历年" v={c.myanmar_year} />
        </div>
        <Note>
          缅历一週八日，周三按午前午后分 Mercury 与 Rahu。
          🟡 本命宫的取法只有单一来源，另一套实现逐日比对多数不合，定下来之前本盘不出本命宫。
        </Note>
      </Section>
    </div>
  )
}
