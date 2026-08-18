// 藏历循环：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { Section, Stat } from './shared'

export interface TibetanChart { year: number; animal: string; element: string; male: boolean; mewa: number; mewa_color: string; sexagenary: number; rabjung: number; year_in_rabjung: number; day_parkha: number; day_parkha_name: string }
const MEWA_HEX: Record<string, string> = { White: '#e9e2d0', Black: '#2a2620', Blue: '#3a6ea5', Green: '#5fb06a', Yellow: '#d4a843', Red: '#d8392e', Maroon: '#7a2e2a' }
export function Tibetan({ c }: { c: TibetanChart }) {
  return (
    <div className="lp">
      <Section title="年柱（六十周期）">
        <div className="tib-big">{c.element} · {c.male ? '阳' : '阴'} · {c.animal}</div>
        <div className="kv-grid">
          <Stat k="六十周期位" v={`第 ${c.sexagenary} / 60`} />
          <Stat k="绕迥 rabjung" v={`第 ${c.rabjung} 期`} />
          <Stat k="期内年" v={`第 ${c.year_in_rabjung}`} />
          <Stat k="历日卦 parkha" v={`${c.day_parkha_name}（第 ${c.day_parkha}）`} />
        </div>
      </Section>
      <Section title="年 Mewa（九宫 sme ba）">
        <div className="mewa-show">
          <span className="mewa-big" style={{ background: MEWA_HEX[c.mewa_color] ?? '#888', color: c.mewa_color === 'White' || c.mewa_color === 'Yellow' ? '#100d0a' : '#fff' }}>{c.mewa}</span>
          <span className="mewa-cn">{c.mewa_color}</span>
        </div>
      </Section>
    </div>
  )
}
