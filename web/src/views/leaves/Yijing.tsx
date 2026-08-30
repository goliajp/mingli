// 易经起卦：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { Hexagram, Section, Stat, hexLines } from './shared'

export interface YijingChart {
  method: string
  primary_upper: string; primary_lower: string; resulting_upper: string; resulting_lower: string
  lines: { value: number; yang: boolean; changing: boolean }[]
  primary: number; resulting: number
  // 64 卦名 + 文王序
  primary_name: string; primary_full_name: string; primary_king_wen: number
  resulting_name: string; resulting_full_name: string; resulting_king_wen: number
}
const YAO_VAL: Record<number, string> = { 6: '老阴 ⚋✕', 7: '少阳 ⚊', 8: '少阴 ⚋', 9: '老阳 ⚊○' }
export function Yijing({ c }: { c: YijingChart }) {
  const primary = c.lines.map((l) => l.yang)
  const movingLine = c.lines.findIndex((l) => l.changing) + 1
  return (
    <div className="lp">
      <Section title="本卦 → 之卦">
        <div className="row-hex big-row">
          <Hexagram lines={primary} moving={movingLine || undefined} label={`本卦 · ${c.primary_full_name}`} sub={`${c.primary_upper}上 ${c.primary_lower}下 · 文王 ${c.primary_king_wen}`} big />
          <span className="hex-arrow">→</span>
          <Hexagram lines={hexLines(c.resulting)} label={`之卦 · ${c.resulting_full_name}`} sub={`${c.resulting_upper}上 ${c.resulting_lower}下 · 文王 ${c.resulting_king_wen}`} big />
        </div>
      </Section>
      <Section title="六爻（自下而上）">
        <div className="yao-list">
          {c.lines.map((l, i) => (
            <div className={`yao-li${l.changing ? ' moving' : ''}`} key={i}><span className="yl-n">{['初', '二', '三', '四', '五', '上'][i]}</span><span className="yl-v">{YAO_VAL[l.value]}</span></div>
          ))}
        </div>
        <div className="kv-grid"><Stat k="起卦法" v={c.method === 'ThreeCoins' ? '三钱法' : c.method} /><Stat k="变爻数" v={c.lines.filter((l) => l.changing).length} hi /></div>
      </Section>
    </div>
  )
}
