// Sikidy：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { GeoFigure, Section, Stat } from './shared'

export interface SikidyChart { mothers: number[]; columns: number[]; seer: number; seer_even: boolean }
export function Sikidy({ c }: { c: SikidyChart }) {
  return (
    <div className="lp">
      <Section title="Sikidy 棋盘（16 列）">
        <div className="sik-grid">
          {c.columns.map((v, i) => <GeoFigure key={i} value={v} label={String(i + 1)} hi={i < 4} />)}
        </div>
        <div className="kv-grid"><Stat k="四母（前 4 列）" v="随机起" /><Stat k="创世者 C15" v={c.seer_even ? '偶 ✓' : '奇 ！'} hi /></div>
      </Section>
    </div>
  )
}
