// 小六壬：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { BRANCHES, Section, Stat } from './shared'

export interface XiaoChart {
  month_deity: string
  day_deity: string
  hour_deity: string
  month_pos: number
  day_pos: number
  hour_pos: number
  lunar_month: number
  lunar_day: number
  hour_branch: number
  /** 该神所配之方；落在小吉或空亡时为 null——两者各家不一 / 配「中」而中宫非可面向之方 */
  month_direction: string | null
  day_direction: string | null
  hour_direction: string | null
}
const XIAO_DEITIES = ['大安', '留连', '速喜', '赤口', '小吉', '空亡']
// 与后端 DEITY_DIRECTION 同序：四正各一，小吉与空亡留空
const XIAO_DIR = ['东', '北', '南', '西', null, null]
export function Xiaoliuren({ c }: { c: XiaoChart }) {
  return (
    <div className="lp">
      <Section title="六神掌诀（月 → 日 → 时辰）">
        <div className="deity-ring big">
          {XIAO_DEITIES.map((d, i) => {
            const marks = [c.month_pos === i && '月', c.day_pos === i && '日', c.hour_pos === i && '时'].filter(Boolean)
            return (
              <div className={`deity${c.hour_pos === i ? ' final' : ''}${marks.length ? ' on' : ''}`} key={d}>
                <span className="deity-name">{d}</span>
                {marks.length > 0 && <span className="deity-mark">{marks.join(' · ')}</span>}
                <span className="deity-dir">{XIAO_DIR[i] ?? '—'}</span>
              </div>
            )
          })}
        </div>
      </Section>
      <Section title="三神">
        <div className="kv-grid">
          <Stat k="月神" v={c.month_deity} />
          <Stat k="日神" v={c.day_deity} />
          <Stat k="时神（断）" v={c.hour_deity} hi />
          <Stat k="月神方位" v={c.month_direction ?? '—'} />
          <Stat k="日神方位" v={c.day_direction ?? '—'} />
          <Stat k="时神方位" v={c.hour_direction ?? '—'} hi />
          <Stat k="农历" v={`${c.lunar_month}月${c.lunar_day}日 ${BRANCHES[c.hour_branch]}时`} />
        </div>
        <div className="lp-note">
          方位只出四个：大安东、留连北、速喜南、赤口西。小吉各家三说不一、空亡配「中」而中宫不是可面向之方，
          两者留空。也正因六分之二给不出方位，本系统不答「寻方位」这一类
        </div>
      </Section>
    </div>
  )
}
