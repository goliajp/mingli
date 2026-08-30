// 七政四余：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import type { QizhengsiyuChart } from '../../types'
import { Section, Stat, fmtDeg } from './shared'

export function Qizhengsiyu({ c }: { c: QizhengsiyuChart }) {
  const qz = c.stars.filter((s) => s.is_qizheng)
  const sy = c.stars.filter((s) => !s.is_qizheng)
  return (
    <div className="lp">
      <Section title={`日柱 · ${c.day_ganzhi} · 28 宿值日 · ${c.mansion_name}`}>
        <div className="kv-grid">
          <Stat k="日柱干支" v={c.day_ganzhi} hi />
          <Stat k="28 宿值日" v={`${c.mansion_name}(idx ${c.mansion})`} hi />
        </div>
      </Section>
      <Section title="七政 · 日月五星地心黄经">
        <table className="jy-graha-table">
          <thead>
            <tr><th>星名</th><th>黄经</th><th>宫名</th><th>宫内度数</th></tr>
          </thead>
          <tbody>
            {qz.map((s) => (
              <tr key={s.star}>
                <td><b>{s.name}</b></td>
                <td>{s.longitude.toFixed(2)}°</td>
                <td>{s.sign_name}</td>
                <td>{fmtDeg(s.degree_in_sign)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Section>
      <Section title="四余 · 罗㬋 / 计都 / 月孛（紫炁 🟡 不入，无天文实体）">
        <table className="jy-graha-table">
          <thead>
            <tr><th>星名</th><th>黄经</th><th>宫名</th><th>宫内度数</th></tr>
          </thead>
          <tbody>
            {sy.map((s) => (
              <tr key={s.star}>
                <td><b>{s.name}</b></td>
                <td>{s.longitude.toFixed(2)}°</td>
                <td>{s.sign_name}</td>
                <td>{fmtDeg(s.degree_in_sign)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Section>
    </div>
  )
}
