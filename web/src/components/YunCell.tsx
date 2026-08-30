// 运层单柱格。
import { WUXING_COLOR, wxRelation } from '../lib/display'
import { gzWuxing } from '../lib/ganzhi'

export function YunCell({ label, gz, sub, dayWx, hi }: {
  label: string; gz: string; sub?: string; dayWx?: string; hi?: boolean
}) {
  const [sWx, bWx] = gzWuxing(gz)
  // 运对日主的五行关系（生我=运助、我克=我用…），仅用五行，不臆造十神。
  const rel = dayWx ? wxRelation(dayWx, sWx) : null
  return (
    <div className={`yun-cell${hi ? ' hi' : ''}`}>
      <div className="yun-lbl">{label}</div>
      <div className="yun-gz">
        <b style={{ color: WUXING_COLOR[sWx] }}>{gz[0]}</b>
        <b style={{ color: WUXING_COLOR[bWx] }}>{gz[1]}</b>
      </div>
      {sub && <div className="yun-sub">{sub}</div>}
      {rel && <div className="yun-rel">{rel}</div>}
    </div>
  )
}
