// 岁运叠加旺衰面板：运层与本命的对照。
import type { OverlayStrength } from '../types'
import { WUXING_COLOR } from '../lib/display'

// 岁运叠加旺衰：大运柱 + 流年柱 拼到本命之上 → t 时刻的实际旺衰。
// 拨杆动时这条会随之脉动，与本命旺衰条形成「命底·运面」对照。
export function OverlayStrengthPanel({ o, dayWx, extras }: {
  o: OverlayStrength; dayWx: string; extras: { dayun?: string; liunian?: string }
}) {
  const dayColor = WUXING_COLOR[dayWx] ?? '#888'
  const d = o.delta_score
  const dir = d > 0 ? '增强' : d < 0 ? '减弱' : '持平'
  const dirCls = d > 0 ? 'up' : d < 0 ? 'down' : 'flat'
  return (
    <div className="overlay-box">
      <div className="strength-hdr">
        <span className="strength-t">岁运叠加旺衰<span className="yun-tag">运</span><span className="neutral-hint">本命 + 大运 + 流年</span></span>
        <span className="strength-level" style={{ color: dayColor }}>{o.yun.level}</span>
        <span className="strength-score">{o.yun.score}<i>/100</i></span>
        <span className={`delta-pill ${dirCls}`}>{d >= 0 ? '+' : ''}{d} · 较本命{dir}</span>
      </div>
      <div className="overlay-extras">
        {extras.dayun && <span className="overlay-src"><b>大运</b>{extras.dayun}</span>}
        {extras.liunian && <span className="overlay-src"><b>流年</b>{extras.liunian}</span>}
      </div>
      <div className="strength-bar">
        <div className="strength-bar-fill" style={{ width: `${o.yun.score}%`, background: dayColor, opacity: .82 }} />
        <i className="overlay-natal-mark" style={{ left: `${o.ming.score}%` }} title={`本命 ${o.ming.score}`} />
      </div>
      <div className="overlay-trio">
        <OverlayTrio label="得令" v={o.yun.got_ling} base={o.ming.got_ling} c={dayColor} />
        <OverlayTrio label="得地" v={o.yun.got_di} base={o.ming.got_di} c={dayColor} />
        <OverlayTrio label="得势" v={o.yun.got_shi} base={o.ming.got_shi} c={dayColor} />
      </div>
      <div className="lp-note" style={{ paddingTop: 8, fontSize: 14 }}>
        <b>「增强 / 减弱」≠ 「转好 / 转坏」</b>：这只是 t 时刻日主能量相对本命的偏移。
        若本命身弱，流年「增强」可能正合「弱者宜扶」是吉；若本命身强已无制，流年再「增强」反而是「强而无制」的偏差 —— 同样的 ↑，吉凶方向相反。<br />
        本命底图固定，大运柱 + 流年柱「外力」拼入得地/得势/五行分布，得令永远取本命月支（月令出生即定）；旺衰条上的小标记是本命基准位，条本身的填充长度是叠加后值。
        🟡 岁运折扣权重无统一标准；真正的吉凶要看用神 / 喜忌配合。
      </div>
    </div>
  )
}

export function OverlayTrio({ label, v, base, c }: { label: string; v: number; base: number; c: string }) {
  const d = v - base
  return (
    <div className="overlay-trio-cell">
      <div className="overlay-trio-l">{label}</div>
      <div className="overlay-trio-v" style={{ color: c }}>{v}<i>/30</i></div>
      <div className={`overlay-trio-d ${d > 0 ? 'up' : d < 0 ? 'down' : 'flat'}`}>
        {d >= 0 ? '+' : ''}{d}<i>本命 {base}</i>
      </div>
    </div>
  )
}
