// 四柱整页：命（出生即定）与运（随时刻而动）两栏对照。
import { useEffect, useState } from 'react'
import { fetchOverlayStrength } from '../api/client'
import type { BaziChart, ChartRequest, OverlayStrength, Pillar } from '../types'
import { WUXING_COLOR } from '../leaves'
import { OverlayStrengthPanel } from '../components/OverlayStrengthPanel'
import { PatternPanel } from '../components/PatternPanel'
import { StrengthPanel } from '../components/StrengthPanel'
import { YongShenPanel } from '../components/YongShenPanel'
import { YunCell } from '../components/YunCell'
import { STEM_WX, pickDayun } from '../lib/ganzhi'

// 四柱宫位象义按主体切（落到视觉）；仅 bazi 叶受影响。
// 顺序 = [年， 月， 日， 时]。
export const SUBJECT_PILLAR_ROLES: Record<'person' | 'company' | 'product' | 'event', [string, string, string, string]> = {
  person: ['祖根', '父母 / 青年', '自身 / 配偶', '子女 / 晚年'],
  company: ['创立根基 / 行业', '团队 / 管理', '主体 / 核心业务', '前景 / 产出'],
  product: ['上市背景', '定位 / 品类', '本体 / 核心特性', '反馈 / 生命周期'],
  event: ['背景 / 大环境', '诱发 / 参与方', '核心走向', '结果 / 后续'],
}

// 八字叶：本命四柱固定 + 运层（= 全局 t 时刻盘的四柱）。
export function BaziNatalYun({ natal, yun, age, form }: { natal: BaziChart; yun?: BaziChart; age: number; form: ChartRequest }) {
  const subject = (form.subject ?? 'person') as 'person' | 'company' | 'product' | 'event'
  const pillarRoles = SUBJECT_PILLAR_ROLES[subject]
  const dayWx = natal.day_master_wuxing
  const natalCols: [string, Pillar][] = [['年', natal.year], ['月', natal.month], ['日', natal.day], ['时', natal.hour]]
  const dyIdx = natal.dayun ? pickDayun(natal.dayun, age) : -1
  const dyActive = natal.dayun && dyIdx >= 0 ? natal.dayun.pillars[dyIdx] : null

  // 岁运叠加旺衰：本命基础上拼大运柱 + 流年柱，问后端要 t 时刻的实际旺衰。
  const liuNianGz = yun?.year.ganzhi
  const dayunGz = dyActive?.ganzhi
  const extras = [dayunGz, liuNianGz].filter((s): s is string => !!s)
  const extrasKey = extras.join(',')
  const [overlay, setOverlay] = useState<OverlayStrength | null>(null)
  useEffect(() => {
    if (extras.length === 0) { setOverlay(null); return }
    let alive = true
    fetchOverlayStrength(form, extras)
      .then((j) => { if (alive) setOverlay(j) })
      .catch(() => { if (alive) setOverlay(null) })
    return () => { alive = false }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [form, extrasKey])
  return (
    <div className="lp">
      <div className="lp-sec ming">
        <div className="lp-sec-t"><span className="my-badge ming">命</span>本命四柱 · 出生即定，拨动时不变</div>
        <div className="pillars natal-fixed">
          {natalCols.map(([name, p], idx) => (
            <div className={`pillar${name === '日' ? ' is-day' : ''}`} key={name}>
              <div className="pname">{name}柱<i className="p12">{p.day_twelve}</i>{subject !== 'person' && <i className="p-role">{pillarRoles[idx]}</i>}</div>
              {subject === 'person' && <div className="p-role-person">{pillarRoles[idx]}</div>}
              <div className="char gan" style={{ color: WUXING_COLOR[p.stem_wuxing] }}>{p.stem}</div>
              <div className="char zhi" style={{ color: WUXING_COLOR[p.branch_wuxing] }}>{p.branch}</div>
              <div className="wx"><i style={{ background: WUXING_COLOR[p.stem_wuxing] }} />{p.stem_wuxing}<i style={{ background: WUXING_COLOR[p.branch_wuxing] }} />{p.branch_wuxing}</div>
              <div className="pnayin">纳音 <b style={{ color: WUXING_COLOR[p.nayin] }}>{p.nayin}</b></div>
              <div className="phidden">
                <div className="ph-t">藏干</div>
                {p.hidden.map((h, i) => (
                  <div className="hs" key={i}>
                    <b style={{ color: WUXING_COLOR[STEM_WX[h.stem]] }}>{h.stem}</b>
                    <i>{h.ten_god}</i>
                  </div>
                ))}
              </div>
              {p.shensha && p.shensha.length > 0 && (
                <div className="pshensha">
                  {p.shensha.map((s) => <span className="ss-chip" key={s}>{s}</span>)}
                </div>
              )}
            </div>
          ))}
        </div>
        <div className="kv-grid" style={{ marginTop: 10 }}>
          <div className="stat hi"><span className="stat-k">日主</span><span className="stat-v">{natal.day_master}（{dayWx}）</span></div>
          {natal.xunkong && <div className="stat"><span className="stat-k">空亡 · 旬空</span><span className="stat-v">{natal.xunkong.join(' ')}</span></div>}
          {natal.three_houses && <>
            <div className="stat"><span className="stat-k">命宫</span><span className="stat-v">{natal.three_houses.ming_gong}</span></div>
            <div className="stat"><span className="stat-k">身宫</span><span className="stat-v">{natal.three_houses.shen_gong}</span></div>
            <div className="stat"><span className="stat-k">胎元</span><span className="stat-v">{natal.three_houses.tai_yuan}</span></div>
          </>}
        </div>
        {natal.pattern && <PatternPanel p={natal.pattern} />}
        {natal.strength && <StrengthPanel s={natal.strength} dayWx={dayWx} />}
        {natal.yongshen && <YongShenPanel y={natal.yongshen} yunWuxing={overlay?.yun.wuxing} />}
      </div>
      <div
        className="lp-sec yun pulse"
        key={`yun-${yun?.year.ganzhi ?? '-'}-${yun?.month.ganzhi ?? '-'}-${yun?.day.ganzhi ?? '-'}-${yun?.hour.ganzhi ?? '-'}-${dyIdx}`}
      >
        <div className="lp-sec-t"><span className="my-badge yun">运</span>运层 · 随顶部时间拨杆而动</div>
        <div className="yun-row">
          {natal.dayun ? (
            dyActive
              ? <YunCell label={`大运 · 第${dyIdx + 1}步`} gz={dyActive.ganzhi} sub={`${dyActive.start_age} 岁起`} dayWx={dayWx} hi />
              : <YunCell label="大运" gz="未起" sub={`${natal.dayun.start_age_years} 岁起运`} />
          ) : (
            <div className="yun-cell"><div className="yun-lbl">大运</div><div className="yun-gz na">—</div><div className="yun-sub">需性别</div></div>
          )}
          {yun ? (
            <>
              <YunCell label="流年" gz={yun.year.ganzhi} dayWx={dayWx} />
              <YunCell label="流月" gz={yun.month.ganzhi} dayWx={dayWx} />
              <YunCell label="流日" gz={yun.day.ganzhi} dayWx={dayWx} />
              <YunCell label="流时" gz={yun.hour.ganzhi} dayWx={dayWx} />
            </>
          ) : (
            <div className="yun-cell"><div className="yun-lbl">流年/月/日/时</div><div className="yun-gz na">…</div></div>
          )}
        </div>
        <div className="lp-note" style={{ paddingTop: 6 }}>
          运层四柱 = 顶部拨杆所指时刻的八字四柱（流年=年柱、流月=月柱、流日=日柱、流时=时柱）；标签为该柱天干对<b>日主</b>的五行生克。
          🟡 大运换步按 start_age（周岁近似）。
        </div>
        {overlay && <OverlayStrengthPanel o={overlay} dayWx={dayWx} extras={{ dayun: dayunGz, liunian: liuNianGz }} />}
      </div>
    </div>
  )
}
