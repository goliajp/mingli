// 梅花易数：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { WUXING_COLOR, wxRelation } from '../../lib/display'
import { BRANCHES, Hexagram, Section, Stat, TRIGRAM_BY_NAME, TRIGRAM_WX, hexLines, hexLower, hexUpper } from './shared'

export interface MeihuaChart {
  method_id: 'time' | 'numbers'
  primary: number; mutual: number; changed: number; moving_line: number
  primary_upper: string; primary_lower: string; changed_upper: string; changed_lower: string
  hour_branch: number
  // 时间法字段
  year_branch?: number | null; month?: number | null; day?: number | null
  // 数字法字段
  numbers?: [number, number] | null
  // 64 卦名 + 文王序
  primary_name: string; primary_full_name: string; primary_king_wen: number
  mutual_name: string; mutual_full_name: string; mutual_king_wen: number
  changed_name: string; changed_full_name: string; changed_king_wen: number
}
export function Meihua({ c }: { c: MeihuaChart }) {
  // 体用：动爻在下卦(1-3)→下卦为用、上卦为体；在上卦(4-6)→上卦为用、下卦为体
  const movingInLower = c.moving_line <= 3
  const yongName = movingInLower ? c.primary_lower : c.primary_upper
  const tiName = movingInLower ? c.primary_upper : c.primary_lower
  const tiWx = TRIGRAM_WX[TRIGRAM_BY_NAME[tiName]]
  const yongWx = TRIGRAM_WX[TRIGRAM_BY_NAME[yongName]]
  const isNumbers = c.method_id === 'numbers'
  return (
    <div className="lp">
      <Section title="本卦 · 互卦 · 之卦">
        <div className="row-hex big-row">
          <Hexagram lines={hexLines(c.primary)} moving={c.moving_line} label={`本卦 · ${c.primary_full_name}`} sub={`${c.primary_upper}上 ${c.primary_lower}下 · 文王 ${c.primary_king_wen}`} big />
          <Hexagram lines={hexLines(c.mutual)} label={`互卦 · ${c.mutual_full_name}`} sub={`${hexUpper(c.mutual)}上 ${hexLower(c.mutual)}下 · 文王 ${c.mutual_king_wen}`} big />
          <span className="hex-arrow">→</span>
          <Hexagram lines={hexLines(c.changed)} label={`之卦 · ${c.changed_full_name}`} sub={`${c.changed_upper}上 ${c.changed_lower}下 · 文王 ${c.changed_king_wen}`} big />
        </div>
      </Section>
      <Section title="体用五行">
        <div className="kv-grid">
          <Stat k="体卦" v={<span style={{ color: WUXING_COLOR[tiWx] }}>{tiName}（{tiWx}）</span>} hi />
          <Stat k="用卦" v={<span style={{ color: WUXING_COLOR[yongWx] }}>{yongName}（{yongWx}）</span>} />
          <Stat k="动爻" v={`第 ${c.moving_line} 爻`} />
          <Stat k="体用关系" v={wxRelation(yongWx, tiWx)} hi />
        </div>
      </Section>
      <Section title={isNumbers ? '数字（报数）法 · 起卦来源' : '时间起卦法 · 起卦来源'}>
        <div className="kv-grid">
          {isNumbers && c.numbers ? (
            <>
              <Stat k="首数 （上卦）" v={`${c.numbers[0]} → mod 8`} />
              <Stat k="次数 （下卦）" v={`${c.numbers[1]} → mod 8`} />
              <Stat k="时辰" v={`${BRANCHES[c.hour_branch - 1] ?? c.hour_branch}时`} />
              <Stat k="动爻公式" v="（首+次+时辰） mod 6" />
            </>
          ) : (
            <>
              <Stat k="年支" v={`${BRANCHES[(c.year_branch ?? 1) - 1] ?? c.year_branch}（${c.year_branch}）`} />
              <Stat k="农历月" v={`${c.month}`} />
              <Stat k="农历日" v={`${c.day}`} />
              <Stat k="时辰" v={`${BRANCHES[c.hour_branch - 1] ?? c.hour_branch}时`} />
            </>
          )}
        </div>
      </Section>
    </div>
  )
}
