// 紫微斗数：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import type { Palace, ZiweiChart } from '../../types'
import { lunarStr } from '../../lib/display'
import { Row, Section, Stat } from './shared'

const ZIWEI_GROUP = new Set(['紫微', '天机', '太阳', '武曲', '天同', '廉贞'])
const ZIWEI_AUX = new Set(['文昌', '文曲', '左辅', '右弼'])

const BOARD_POS: Record<string, [number, number]> = {
  巳: [1, 1], 午: [1, 2], 未: [1, 3], 申: [1, 4], 辰: [2, 1], 酉: [2, 4],
  卯: [3, 1], 戌: [3, 4], 寅: [4, 1], 丑: [4, 2], 子: [4, 3], 亥: [4, 4],
}
export function ZiweiView({ c }: { c: ZiweiChart }) {
  const byBranch = (b: string) => c.palaces.find((p) => p.branch === b)
  return (
    <div className="lp">
      <div className="board">
        {Object.keys(BOARD_POS).map((b) => {
          const p = byBranch(b); if (!p) return null
          const [r, col] = BOARD_POS[b]
          return <PalaceCell key={b} p={p} row={r} col={col} />
        })}
        <div className="center">
          <div className="c-title">紫微斗数命盘</div>
          <Row k="农历" v={lunarStr(c.lunar)} />
          <Row k="命宫" v={`${c.ming_ganzhi}（${c.ming_branch}）`} hi />
          <Row k="身宫" v={`${c.shen_branch}宫`} />
          <Row k="五行局" v={c.wuxing_ju} />
          <Row k="紫微" v={`${c.ziwei_branch}宫`} />
          <Row k="天府" v={`${c.tianfu_branch}宫`} />
          <div className="legend"><span><i className="lg zi" />紫微系</span><span><i className="lg fu" />天府系</span><span><i className="lg aux" />辅星</span></div>
        </div>
      </div>
      <Section title={c.sihua.school_id === 'quanshu' ? '四化 · 中州派（王亭之版）' : '四化 · 通行版（中州/三合派）'}>
        <div className="kv-grid">
          <Stat k="化禄" v={`${c.sihua.lu_star}${c.sihua.lu_branch ? ` · ${c.sihua.lu_branch}宫` : ''}`} hi />
          <Stat k="化权" v={`${c.sihua.quan_star}${c.sihua.quan_branch ? ` · ${c.sihua.quan_branch}宫` : ''}`} />
          <Stat k="化科" v={`${c.sihua.ke_star}${c.sihua.ke_branch ? ` · ${c.sihua.ke_branch}宫` : ''}`} />
          <Stat k="化忌" v={`${c.sihua.ji_star}${c.sihua.ji_branch ? ` · ${c.sihua.ji_branch}宫` : ''}`} hi />
        </div>
      </Section>
    </div>
  )
}
function PalaceCell({ p, row, col }: { p: Palace; row: number; col: number }) {
  return (
    <div className={`palace${p.is_ming ? ' ming' : ''}${p.is_shen ? ' shen' : ''}`} style={{ gridRow: row, gridColumn: col }}>
      <div className="p-stars">
        {p.stars.length ? p.stars.map((s) => <span className={`star ${ZIWEI_GROUP.has(s) ? 'zi' : ZIWEI_AUX.has(s) ? 'aux' : 'fu'}`} key={s}>{s}</span>) : <span className="empty">空宫</span>}
      </div>
      <div className="p-foot"><span className="p-name">{p.name}{p.is_shen ? <em>身</em> : null}</span><span className="p-gz">{p.ganzhi}</span></div>
    </div>
  )
}
