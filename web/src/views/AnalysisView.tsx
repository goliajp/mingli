// 跨叶相关性热力图。
import { Fragment } from 'react'
import type { Analysis } from '../types'
import { colorOf } from '../data/leaf-regions'

export function AnalysisView({ analysis }: { analysis: Analysis | null }) {
  if (!analysis) {
    return <section className="card"><div className="lp-note">相关性分析计算中…（首次约 2 秒）</div></section>
  }
  const { leaves, nmi, n } = analysis
  return (
    <section className="card">
      <div className="lp-sec-t">各术数之间的相关性（{n} 组随机生辰样本）</div>
      <div className="all-note">
        在大量随机生辰下，比较任意两种术数的某个特征是否同步变化（亮＝高度相关、暗＝各自独立）。
        基于天文历法的术数必然一致（<b>八字日支 ≡ 大六壬日支 ＝ 1.00</b>）；
        随机起卦类与生辰无关、彼此独立（≈0）。🟡 有限样本下相关度估计偏高，故「低相关」是保守结论。
      </div>
      <div className="heat" style={{ gridTemplateColumns: `132px repeat(${leaves.length}, 1fr)` }}>
        <div className="hc corner" />
        {leaves.map((l) => <div className="hc col" key={l.id} style={{ color: colorOf(l.id) }}>{l.name}</div>)}
        {leaves.map((row, i) => (
          <Fragment key={row.id}>
            <div className="hc rowh" style={{ color: colorOf(row.id) }}>{row.name}<i>{row.feature}</i></div>
            {leaves.map((col, j) => {
              const v = nmi[i][j]
              const txt = i === j ? '1' : v >= 0.12 ? v.toFixed(2).replace(/^0/, '') : ''
              return <div className="hcell" key={col.id} title={`${row.name} × ${col.name} = ${v.toFixed(3)}`} style={{ background: `rgba(201,162,74,${Math.max(0.02, v)})` }}>{txt}</div>
            })}
          </Fragment>
        ))}
      </div>
    </section>
  )
}
