// 择日：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { Section, Stat } from './shared'

export interface ZeriChart {
  month_branch: number; day_branch: number;
  day_stem: number; day_ganzhi_name: string;
  jianchu: string; jianchu_pos: number;
  mansion: string; mansion_index: number;
  pengzu_gan: string; pengzu_zhi: string;
  tianyi_branches: [number, number]; tianyi_names: [string, string];
}
const JIANCHU = ['建', '除', '满', '平', '定', '执', '破', '危', '成', '收', '开', '闭']
const MANSIONS = ['角', '亢', '氐', '房', '心', '尾', '箕', '斗', '牛', '女', '虚', '危', '室', '壁', '奎', '娄', '胃', '昴', '毕', '觜', '参', '井', '鬼', '柳', '星', '张', '翼', '轸']
const LUMINARY = ['木', '金', '土', '日', '月', '火', '水']
const QUADRANT = ['东方苍龙', '北方玄武', '西方白虎', '南方朱雀']
export function Zeri({ c }: { c: ZeriChart }) {
  const lum = LUMINARY[c.mansion_index % 7]
  const weekday = ['日', '月', '火', '水', '木', '金', '土']
  return (
    <div className="lp">
      <Section title="日柱">
        <div className="kv-grid">
          <Stat k="干支" v={c.day_ganzhi_name} hi />
          <Stat k="天乙贵人" v={`${c.tianyi_names[0]}、${c.tianyi_names[1]}`} />
        </div>
      </Section>
      <Section title="建除十二神">
        <div className="strip big">
          {JIANCHU.map((j, i) => <span className={`chip${i === c.jianchu_pos ? ' hi' : ''}`} key={j}>{j}</span>)}
        </div>
      </Section>
      <Section title="二十八宿值日">
        <div className="strip xiu">
          {MANSIONS.map((m, i) => <span className={`xchip${i === c.mansion_index ? ' hi' : ''}`} key={m}>{m}</span>)}
        </div>
        <div className="kv-grid">
          <Stat k="值日宿" v={`${c.mansion}宿`} hi />
          <Stat k="七曜" v={lum} />
          <Stat k="星期" v={weekday.indexOf(lum) >= 0 ? `${['日', '一', '二', '三', '四', '五', '六'][weekday.indexOf(lum)]}` : '—'} />
          <Stat k="所属" v={QUADRANT[Math.floor(c.mansion_index / 7)]} />
        </div>
      </Section>
      <Section title="彭祖百忌">
        <div className="pengzu">
          <div className="pengzu-line">{c.pengzu_gan}</div>
          <div className="pengzu-line">{c.pengzu_zhi}</div>
          <div className="pengzu-src">源：《钦定协纪辨方书》/通胜通行版</div>
        </div>
      </Section>
    </div>
  )
}
