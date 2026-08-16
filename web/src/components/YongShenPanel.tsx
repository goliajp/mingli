// 用神 / 喜忌面板，含 t 时刻供给度。
import type { YongShen } from '../types'
import { WUXING_COLOR } from '../leaves'

// 用神 / 喜忌：把旺衰+格局合起来，告诉命主「该补什么 / 该忌什么」。
// 若有运层叠加，旁边附「t 时刻拿到了多少喜用」（本命用神为锚，看运给的够不够）。
export function YongShenPanel({ y, yunWuxing }: { y: YongShen; yunWuxing?: { wood: number; fire: number; earth: number; metal: number; water: number } }) {
  const wxPct = (n: string): number => {
    if (!yunWuxing) return 0
    return ({ '木': yunWuxing.wood, '火': yunWuxing.fire, '土': yunWuxing.earth, '金': yunWuxing.metal, '水': yunWuxing.water } as Record<string, number>)[n] ?? 0
  }
  return (
    <div className="yong-box">
      <div className="yong-hdr">
        <span className="strength-t">用神 · 喜忌<span className="ming-tag">本命</span><span className="neutral-hint">该补 / 该忌 · 是事实题，不是断吉凶</span></span>
        <span className="yong-method">{y.method}</span>
      </div>
      <div className="yong-grid">
        <YongCell kind="primary" label="主用神" wx={y.primary_wuxing} role={y.primary_role} pct={yunWuxing ? wxPct(y.primary_wuxing) : undefined} />
        {y.secondary_wuxing && y.secondary_role && (
          <YongCell kind="secondary" label="副用神" wx={y.secondary_wuxing} role={y.secondary_role} pct={yunWuxing ? wxPct(y.secondary_wuxing) : undefined} />
        )}
        {y.avoid_wuxing.length > 0 && (
          <div className="yong-avoid">
            <div className="yong-avoid-l">忌神</div>
            <div className="yong-avoid-v">
              {y.avoid_wuxing.map((n) => (
                <span key={n} className="yong-avoid-chip" style={{ color: WUXING_COLOR[n], borderColor: WUXING_COLOR[n] }}>
                  <b>{n}</b>
                  {yunWuxing && <i>{wxPct(n)}%</i>}
                </span>
              ))}
            </div>
          </div>
        )}
      </div>
      <div className="yong-reason">{y.reasoning}</div>
      {yunWuxing && (
        <div className="lp-note" style={{ paddingTop: 6, fontSize: 14 }}>
          ↑ 上面百分比 = <b>t 时刻五行分布</b>（本命+大运+流年叠加后）中该五行的实际占比；
          <b>主用神 % 越高 = 当前时段「拿到的喜用」越多</b>；忌神 % 越高 = 当前时段「拿到的克泄」越多。
          这一栏 = 「拨杆 → 是吉是凶」的最后一跳：**用神被供给** = 吉，**忌神被加强** = 凶。
        </div>
      )}
      <div className="lp-note" style={{ paddingTop: 6, fontSize: 14 }}>
        <b>「用神」≠ 必然吉、「忌神」≠ 必然凶。</b>这只是命局结构推出的「该补 / 该忌」的事实结论。
        🟡 取用神有扶抑/调候/通关/病药/格局五法，各家先后顺序不同；从格/化格反扶抑（扶其太过）本算法不覆盖。
      </div>
    </div>
  )
}

export function YongCell({ kind, label, wx, role, pct }: { kind: 'primary' | 'secondary'; label: string; wx: string; role: string; pct?: number }) {
  const c = WUXING_COLOR[wx] ?? 'inherit'
  return (
    <div className={`yong-cell ${kind}`}>
      <div className="yong-cell-l">{label}</div>
      <div className="yong-cell-wx" style={{ color: c }}>{wx}</div>
      <div className="yong-cell-role">{role}</div>
      {pct !== undefined && (
        <div className="yong-cell-pct">
          <div className="yong-cell-bar"><i style={{ width: `${Math.min(pct, 60) * 100 / 60}%`, background: c }} /></div>
          <span className="yong-cell-pct-v" style={{ color: c }}>{pct}<i>%</i></span>
        </div>
      )}
    </div>
  )
}
