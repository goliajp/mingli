// 顶部命局摘要行。
import { Fragment } from 'react'
import type { BaziChart, ChartRequest, ZiweiChart } from '../types'
import { WUXING_COLOR, lunarStr } from '../lib/display'
import { HOUR_NAMES } from '../lib/ganzhi'

export function SummaryBar({ bazi, ziwei, form }: { bazi: BaziChart; ziwei: ZiweiChart | null; form: ChartRequest }) {
  const bz = [bazi.year, bazi.month, bazi.day, bazi.hour]
  const hourName = HOUR_NAMES[Math.floor(((form.hour + 1) % 24) / 2)]
  return (
    <section className="summary">
      <div className="sm-left">
        <div className="sm-row">
          <span className="sm-k">公历</span>
          {form.year}-{String(form.month).padStart(2, '0')}-{String(form.day).padStart(2, '0')} {String(form.hour).padStart(2, '0')}:{String(form.minute).padStart(2, '0')}
          <span className="sm-sep">·</span>{hourName}时<span className="sm-sep">·</span>UTC{form.tz >= 0 ? '+' : ''}{form.tz}
        </div>
        <div className="sm-row"><span className="sm-k">农历</span>{lunarStr(bazi.lunar)}</div>
        {ziwei && <div className="sm-row"><span className="sm-k">命宫</span>{ziwei.ming_ganzhi}（{ziwei.ming_branch}）<span className="sm-sep">·</span>{ziwei.wuxing_ju}</div>}
        {bazi.strength && bazi.pattern && bazi.yongshen && (
          <div className="sm-row">
            <span className="sm-k">命局</span>
            <b style={{ color: WUXING_COLOR[bazi.day_master_wuxing] }}>{bazi.day_master}</b>·{bazi.strength.level}
            <span className="sm-sep">·</span>{bazi.pattern.name}
            <span className="sm-sep">·</span>喜<b style={{ color: WUXING_COLOR[bazi.yongshen.primary_wuxing] }}>{bazi.yongshen.primary_wuxing}</b>
            {bazi.yongshen.secondary_wuxing && <>/<b style={{ color: WUXING_COLOR[bazi.yongshen.secondary_wuxing] }}>{bazi.yongshen.secondary_wuxing}</b></>}
            {bazi.yongshen.avoid_wuxing.length > 0 && <>
              <span className="sm-sep">·</span>忌
              {bazi.yongshen.avoid_wuxing.map((n, i) => (
                <Fragment key={n}>{i > 0 && '/'}<b style={{ color: WUXING_COLOR[n] }}>{n}</b></Fragment>
              ))}
            </>}
          </div>
        )}
      </div>
      <div className="sm-right">
        <div className="bazi-big">
          {bz.map((p, i) => (
            <span className="bb-col" key={i}>
              <b style={{ color: WUXING_COLOR[p.stem_wuxing] }}>{p.stem}</b>
              <b style={{ color: WUXING_COLOR[p.branch_wuxing] }}>{p.branch}</b>
            </span>
          ))}
        </div>
        <div className="bazi-label">年　月　日　时</div>
      </div>
    </section>
  )
}
