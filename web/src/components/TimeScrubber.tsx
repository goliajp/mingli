// 全局时间拨杆：一根 0–100 岁滑块驱动整棵树的 t 时刻。
import type { DaYun } from '../types'

// 全局时间拨杆（常驻 tabs 上方）：一根 playhead，所有叶子订阅它。
export function TimeScrubber({ age, nowAge, playDate, dayun, onChange, onNow, onBirth }: {
  age: number; nowAge: number; playDate: Date; dayun: DaYun | null
  onChange: (a: number) => void; onNow: () => void; onBirth: () => void
}) {
  const MAX = 100
  const pct = (a: number) => `${(a / MAX) * 100}%`
  const diff = age - nowAge
  const when = Math.abs(diff) < 0.05 ? '此刻' : diff < 0 ? '过去' : '未来'
  return (
    <div className="timebar">
      <div className="timebar-t">时间轴 · 过去 ← 此刻 → 未来　<em>拨动 = 全部系统切到该时刻</em></div>
      <div className="tl-track">
        <div className="tl-future" style={{ left: pct(nowAge) }} />
        {dayun?.pillars.filter((d) => d.start_age <= MAX).map((d) => (
          <i className="tl-tick" key={d.start_age} style={{ left: pct(d.start_age) }}><span>{d.start_age}</span></i>
        ))}
        <i className="tl-now" style={{ left: pct(nowAge) }}><span>今</span></i>
        <i className="tl-play" style={{ left: pct(age) }} />
      </div>
      <input className="tl-range" type="range" min={0} max={MAX} step={0.1} value={age}
        onChange={(e) => onChange(Number(e.target.value))} />
      <div className="tl-read">
        <span className={`tl-when w-${when === '过去' ? 'past' : when === '未来' ? 'future' : 'now'}`}>{when}</span>
        <span className="tl-date">
          {playDate.getFullYear()}-{String(playDate.getMonth() + 1).padStart(2, '0')}-{String(playDate.getDate()).padStart(2, '0')}
          {' '}{String(playDate.getHours()).padStart(2, '0')}:{String(playDate.getMinutes()).padStart(2, '0')}
        </span>
        <span className="tl-age">{Math.floor(age)} 岁</span>
        {Math.abs(diff) >= 0.05 && <span className="tl-rel">（今{diff < 0 ? '前' : '后'} {Math.abs(Math.round(diff))} 年）</span>}
        <span className="tl-jump">
          <button onClick={onBirth}>出生</button>
          <button onClick={onNow}>回到此刻</button>
        </span>
      </div>
    </div>
  )
}
