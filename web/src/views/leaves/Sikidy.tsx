// Sikidy：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { GeoFigure, Note, Section, Stat } from './shared'

export interface SikidyChart { mothers: number[]; columns: number[]; seer: number; seer_even: boolean }
export function Sikidy({ c }: { c: SikidyChart }) {
  return (
    <div className="lp">
      <Section title="Sikidy 棋盘（16 列）">
        <div className="sik-grid">
          {c.columns.map((v, i) => <GeoFigure key={i} value={v} label={String(i + 1)} hi={i < 4} />)}
        </div>
        <div className="kv-grid"><Stat k="四母（前 4 列）" v="随机起" /><Stat k="创世者 C15" v={c.seer_even ? '偶 ✓' : '奇 ！'} hi /></div>
        <Note>
          十六列里只有前四列（四母）是随机的，其余十二列由它们逐层异或推出——与地占同构，
          故创世者列 C15 恒为偶（GF(2) 奇偶守恒）：若显示为奇，必是算错。
          🟡 第 6 与第 14 列的语义三源三说，本盘两处留空不硬选。
        </Note>
      </Section>
    </div>
  )
}
