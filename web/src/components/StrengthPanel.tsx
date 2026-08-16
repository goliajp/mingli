// 旺衰量化面板：得令 / 得地 / 得势三栏与五行分布。
import type { Strength } from '../types'
import { WUXING_COLOR } from '../leaves'

// 旺衰量化：得令/得地/得势三栏 0-30 → 综合 0-100 强度条 + 五行力量分布。
export function StrengthPanel({ s, dayWx }: { s: Strength; dayWx: string }) {
  const dayColor = WUXING_COLOR[dayWx] ?? '#888'
  // 五行分布：按命理传统排序 木火土金水
  const wxRows: [string, number][] = [
    ['木', s.wuxing.wood], ['火', s.wuxing.fire], ['土', s.wuxing.earth],
    ['金', s.wuxing.metal], ['水', s.wuxing.water],
  ]
  return (
    <div className="strength-box">
      <div className="strength-hdr">
        <span className="strength-t">日主旺衰<span className="ming-tag">本命</span><span className="neutral-hint">能量量级</span></span>
        <span className="strength-level" style={{ color: dayColor }}>{s.level}</span>
        <span className="strength-score">{s.score}<i>/100</i></span>
      </div>
      <div className="strength-bar">
        <div className="strength-bar-fill" style={{ width: `${s.score}%`, background: dayColor }} />
        <i className="strength-bar-mark" style={{ left: '40%' }} />
        <i className="strength-bar-mark" style={{ left: '60%' }} />
      </div>
      <div className="strength-cols">
        <StrengthCol label="得令" sub="月支长生 + 月支藏干" v={s.got_ling} c={dayColor} />
        <StrengthCol label="得地" sub="年/日/时支通根" v={s.got_di} c={dayColor} />
        <StrengthCol label="得势" sub="干头比劫印" v={s.got_shi} c={dayColor} />
      </div>
      <div className="strength-t" style={{ marginTop: 14 }}>五行力量分布</div>
      <div className="wx-rows">
        {wxRows.map(([n, v]) => (
          <div className="wx-row" key={n}>
            <span className="wx-row-n" style={{ color: WUXING_COLOR[n] }}>{n}</span>
            <div className="wx-row-bar"><i style={{ width: `${v}%`, background: WUXING_COLOR[n] }} /></div>
            <span className="wx-row-v">{v}%</span>
          </div>
        ))}
      </div>
      <div className="lp-note" style={{ paddingTop: 8, fontSize: 14 }}>
        <b>「强 / 弱」≠ 「好 / 坏」</b>：强弱是日主能量量级（像身高体重），本身不构成褒贬。
        命格好坏 = 强弱 × 用神配不配 —— 身强宜抑(食伤/财/官杀泄克)、身弱宜扶(比劫/印帮身);
        「强而有制 / 弱而有助」均属佳格，「强而无制 / 弱而无援」才偏差。**→ 真正的吉凶要看用神 / 喜忌配合。**
        <br />
        🟡 权重表无统一标准（各家月令权重 30%-60% 不一）；本算法显式声明：得令/得地/得势各 0-30，合 0-90 → 0-100；
        「同党」=比劫（同五行）+印星（生我）。量化为辅助判断，非定论。
      </div>
    </div>
  )
}

export function StrengthCol({ label, sub, v, c }: { label: string; sub: string; v: number; c: string }) {
  return (
    <div className="strength-col">
      <div className="strength-col-v" style={{ color: c }}>{v}<i>/30</i></div>
      <div className="strength-col-bar"><i style={{ height: `${(v / 30) * 100}%`, background: c }} /></div>
      <div className="strength-col-l">{label}</div>
      <div className="strength-col-s">{sub}</div>
    </div>
  )
}
