// 命格面板。
import type { Pattern } from '../types'
import { WUXING_COLOR } from '../lib/display'
import { STEM_WX } from '../lib/ganzhi'

// 命格：月令藏干透干 → 八正格 / 建禄月刃 / 暗格。是命主结构性属性，不随时间动。
export function PatternPanel({ p }: { p: Pattern }) {
  const isAmbiguous = !p.revealed && !p.is_lu_ren
  return (
    <div className="pattern-box">
      <div className="pattern-hdr">
        <span className="pattern-t">命格<span className="ming-tag">本命</span><span className="neutral-hint">命主结构类型</span></span>
        <span className="pattern-name">{p.name}</span>
      </div>
      <div className="pattern-body">
        <div className="pattern-row">
          <span className="pattern-k">月令藏干</span>
          <span className="pattern-v">
            <b style={{ color: WUXING_COLOR[STEM_WX[p.qi_stem]] ?? 'inherit' }}>{p.qi_stem}</b>
            <i>{p.qi_kind}</i>
            <em>→ 日主十神 {p.ten_god}</em>
          </span>
        </div>
        <div className="pattern-row">
          <span className="pattern-k">取格依据</span>
          <span className="pattern-v">
            {p.source}
            {p.revealed_in && <i className="pattern-tag-on">透干在 {p.revealed_in}</i>}
            {isAmbiguous && <i className="pattern-tag-off">藏而不透</i>}
            {p.is_lu_ren && <i className="pattern-tag-special">禄刃 · 不入八正格</i>}
          </span>
        </div>
      </div>
      <div className="lp-note" style={{ paddingTop: 6, fontSize: 14 }}>
        命格是命主的<b>结构性类型</b>（像血型/星座），本身无优劣。<b>「正官格」不等于「当官」、「七杀格」不等于「凶险」。</b>
        格局好坏 = 格局 × 用神配合；从格/化格/专旺格涉强弱+特殊条件，本算法 🟡 不机械化，留 INT。
      </div>
    </div>
  )
}
