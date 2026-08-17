// 巴厘 Pawukon：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { Section } from './shared'

export interface PawukonChart { day: number; urip: number; wuku: string; triwara: string; pancawara: string; sadwara: string; saptawara: string; dasawara: string; dwiwara: string; ekawara: string | null; caturwara: string; astawara: string; sangawara: string }
export function Pawukon({ c }: { c: PawukonChart }) {
  const simple: [string, string, string][] = [['Saptawara', '7', c.saptawara], ['Pancawara', '5', c.pancawara], ['Sadwara', '6', c.sadwara], ['Triwara', '3', c.triwara]]
  const derived: [string, string, string][] = [['Dasawara', '10', c.dasawara], ['Dwiwara', '2', c.dwiwara], ['Ekawara', '1', c.ekawara ?? '—（偶日无）']]
  const stuck: [string, string, string][] = [['Caturwara', '4', c.caturwara], ['Astawara', '8', c.astawara], ['Sangawara', '9', c.sangawara]]
  const grp = (title: string, arr: [string, string, string][]) => (
    <div className="wew-grp"><div className="wew-gt">{title}</div><div className="wewaran">
      {arr.map(([nm, n, v]) => <div className="wew" key={nm}><span className="wn">{nm}<i>{n}</i></span><b>{v}</b></div>)}
    </div></div>
  )
  return (
    <div className="lp">
      <Section title={`Pawukon 第 ${c.day} / 210 日 · Wuku ${c.wuku} · urip ${c.urip}`}>
        {grp('简单 mod 週', simple)}
        {grp('urip 之和派生', derived)}
        {grp('卡日週（n∤210）', stuck)}
      </Section>
    </div>
  )
}
