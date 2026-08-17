// 小六壬：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { BRANCHES, Section, Stat } from './shared'

export interface XiaoChart { month_deity: string; day_deity: string; hour_deity: string; month_pos: number; day_pos: number; hour_pos: number; lunar_month: number; lunar_day: number; hour_branch: number }
const XIAO_DEITIES = ['大安', '留连', '速喜', '赤口', '小吉', '空亡']
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
          <Stat k="农历" v={`${c.lunar_month}月${c.lunar_day}日 ${BRANCHES[c.hour_branch]}时`} />
        </div>
      </Section>
    </div>
  )
}
