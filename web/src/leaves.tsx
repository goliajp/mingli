// 每片叶 = 一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 mingli-api /api/cast 的 serde 输出一一对应；缺名表处(🟡)忠实留白，不臆造。
import { useMemo } from 'react'
import type { BaziChart, CastLeaf, JyotishChart, Lunar, Palace, QizhengsiyuChart, ZiweiChart } from './types'

// ============ 共享常量 ============
export const WUXING = ['木', '火', '土', '金', '水'] as const
export const WUXING_COLOR: Record<string, string> = {
  木: '#5fb06a', 火: '#e0584c', 土: '#d4a843', 金: '#cfd6dc', 水: '#5b9bd6',
}
export const ZIWEI_GROUP = new Set(['紫微', '天机', '太阳', '武曲', '天同', '廉贞'])
export const ZIWEI_AUX = new Set(['文昌', '文曲', '左辅', '右弼'])
const BRANCHES = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥']
const STEMS = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸']
// 八卦值(0..7)→名/五行（与 mingli-gua TRIGRAM_NAMES 同序：坤震坎兑艮离巽乾）
const TRIGRAM_NAME = ['坤', '震', '坎', '兑', '艮', '离', '巽', '乾']
const TRIGRAM_WX = ['土', '木', '水', '金', '土', '火', '木', '金']
const TRIGRAM_BY_NAME: Record<string, number> = Object.fromEntries(TRIGRAM_NAME.map((n, i) => [n, i]))
const PLANET_GLYPH: Record<string, string> = {
  太阳: '☉', 太阴: '☽', 月亮: '☽', 水星: '☿', 金星: '♀', 火星: '♂',
  木星: '♃', 土星: '♄', 天王: '♅', 天王星: '♅', 海王: '♆', 海王星: '♆', 冥王: '♇',
}
const SIGN_NAMES = ['白羊', '金牛', '双子', '巨蟹', '狮子', '处女', '天秤', '天蝎', '射手', '摩羯', '水瓶', '双鱼']
const SIGN_GLYPH: Record<string, string> = {
  白羊: '♈', 金牛: '♉', 双子: '♊', 巨蟹: '♋', 狮子: '♌', 处女: '♍',
  天秤: '♎', 天蝎: '♏', 射手: '♐', 摩羯: '♑', 水瓶: '♒', 双鱼: '♓',
}
const ASPECT_SYM: Record<string, string> = { 合: '☌', 冲: '☍', 拱: '△', 刑: '□', 六分: '⚹' }
const ASPECT_COLOR: Record<string, string> = { 合: '#d4a843', 冲: '#e0584c', 拱: '#5fb06a', 刑: '#e0584c', 六分: '#5fb3bf' }
const LUOSHU = [[4, 9, 2], [3, 5, 7], [8, 1, 6]]

export function lunarStr(l: Lunar) {
  const m = ['', '正', '二', '三', '四', '五', '六', '七', '八', '九', '十', '冬', '腊']
  const d = ['', '初一', '初二', '初三', '初四', '初五', '初六', '初七', '初八', '初九', '初十',
    '十一', '十二', '十三', '十四', '十五', '十六', '十七', '十八', '十九', '二十',
    '廿一', '廿二', '廿三', '廿四', '廿五', '廿六', '廿七', '廿八', '廿九', '三十']
  return `${l.year}年${l.leap ? '闰' : ''}${m[l.month]}月${d[l.day]}`
}
function hexLines(v: number): boolean[] { return [0, 1, 2, 3, 4, 5].map((i) => ((v >> i) & 1) === 1) }
function hexUpper(v: number) { return TRIGRAM_NAME[(v >> 3) & 7] }
function hexLower(v: number) { return TRIGRAM_NAME[v & 7] }
// 五行生克关系（a 视角对 b）
export function wxRelation(a: string, b: string): string {
  if (a === b) return '比和'
  const gen: Record<string, string> = { 木: '火', 火: '土', 土: '金', 金: '水', 水: '木' }
  const ctl: Record<string, string> = { 木: '土', 土: '水', 水: '火', 火: '金', 金: '木' }
  if (gen[a] === b) return `${a}生${b}`
  if (gen[b] === a) return `${b}生${a}`
  if (ctl[a] === b) return `${a}克${b}`
  return `${b}克${a}`
}

// ============ 通用小件 ============
function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return <div className="lp-sec"><div className="lp-sec-t">{title}</div>{children}</div>
}
function Stat({ k, v, hi }: { k: string; v: React.ReactNode; hi?: boolean }) {
  return <div className={`stat${hi ? ' hi' : ''}`}><span className="stat-k">{k}</span><span className="stat-v">{v}</span></div>
}
function Note({ children }: { children: React.ReactNode }) { return <div className="lp-note">{children}</div> }

function Yao({ yang, changing }: { yang: boolean; changing?: boolean }) {
  return (
    <div className={`yao${changing ? ' moving' : ''}`}>
      {yang ? <span className="bar" /> : (<><span className="bar half" /><span className="bar half" /></>)}
      {changing && <span className="yao-mk">{yang ? '○' : '×'}</span>}
    </div>
  )
}
function Hexagram({ lines, moving, label, sub, big }: { lines: boolean[]; moving?: number; label?: string; sub?: string; big?: boolean }) {
  return (
    <div className="hexbox">
      <div className={`hex${big ? ' big' : ''}`}>
        {[5, 4, 3, 2, 1, 0].map((i) => <Yao key={i} yang={lines[i]} changing={moving === i + 1} />)}
      </div>
      {label && <div className="hex-label">{label}</div>}
      {sub && <div className="hex-sub">{sub}</div>}
    </div>
  )
}
function GeoFigure({ value, hi, label }: { value: number; hi?: boolean; label?: string }) {
  return (
    <div className="geo-fig-wrap">
      <div className={`geofig${hi ? ' hi' : ''}`}>
        {[3, 2, 1, 0].map((i) => {
          const single = ((value >> i) & 1) === 1
          return <div className="geo-row" key={i}>{single ? <span className="pip" /> : <><span className="pip" /><span className="pip" /></>}</div>
        })}
      </div>
      {label && <div className="geo-lbl">{label}</div>}
    </div>
  )
}

// ============ A 族 ============
export function BaziView({ c }: { c: BaziChart }) {
  const cols: [string, BaziChart['year']][] = [['年', c.year], ['月', c.month], ['日', c.day], ['时', c.hour]]
  const counts = useMemo(() => {
    const m: Record<string, number> = { 木: 0, 火: 0, 土: 0, 金: 0, 水: 0 }
    for (const p of [c.year, c.month, c.day, c.hour]) { m[p.stem_wuxing]++; m[p.branch_wuxing]++ }
    return m
  }, [c])
  return (
    <div className="lp">
      <Section title="四柱">
        <div className="pillars">
          {cols.map(([name, p]) => (
            <div className={`pillar${name === '日' ? ' is-day' : ''}`} key={name}>
              <div className="pname">{name}柱</div>
              <div className="tengod">{p.ten_god}</div>
              <div className="char gan" style={{ color: WUXING_COLOR[p.stem_wuxing] }}>{p.stem}</div>
              <div className="char zhi" style={{ color: WUXING_COLOR[p.branch_wuxing] }}>{p.branch}</div>
              <div className="wx"><i style={{ background: WUXING_COLOR[p.stem_wuxing] }} />{p.stem_wuxing}<i style={{ background: WUXING_COLOR[p.branch_wuxing] }} />{p.branch_wuxing}</div>
            </div>
          ))}
        </div>
        <div className="kv-grid"><Stat k="日主" v={`${c.day_master}（${c.day_master_wuxing}）`} hi /></div>
      </Section>
      <Section title="五行分布">
        <div className="wuxing-bars">
          {WUXING.map((w) => (
            <div className="wb-row" key={w}>
              <span className="wb-name" style={{ color: WUXING_COLOR[w] }}>{w}</span>
              <span className="wb-track"><span className="wb-fill" style={{ width: `${(counts[w] / 8) * 100}%`, background: WUXING_COLOR[w] }} /></span>
              <span className="wb-num">{counts[w]}</span>
            </div>
          ))}
        </div>
      </Section>
      {c.dayun && (
        <Section title={`大运 · ${c.dayun.forward ? '顺行' : '逆行'} · 起运 ${c.dayun.start_age_years} 岁`}>
          <div className="dy-row">
            {c.dayun.pillars.map((d) => <div className="dy-cell" key={d.start_age}><div className="dy-age">{d.start_age}岁</div><div className="dy-gz">{d.ganzhi}</div></div>)}
          </div>
        </Section>
      )}
    </div>
  )
}

const BOARD_POS: Record<string, [number, number]> = {
  巳: [1, 1], 午: [1, 2], 未: [1, 3], 申: [1, 4], 辰: [2, 1], 酉: [2, 4],
  卯: [3, 1], 戌: [3, 4], 寅: [4, 1], 丑: [4, 2], 子: [4, 3], 亥: [4, 4],
}
export function ZiweiView({ c }: { c: ZiweiChart }) {
  const byBranch = (b: string) => c.palaces.find((p) => p.branch === b)
  return (
    <div className="lp">
      <div className="board">
        {Object.keys(BOARD_POS).map((b) => {
          const p = byBranch(b); if (!p) return null
          const [r, col] = BOARD_POS[b]
          return <PalaceCell key={b} p={p} row={r} col={col} />
        })}
        <div className="center">
          <div className="c-title">紫微斗数命盘</div>
          <Row k="农历" v={lunarStr(c.lunar)} />
          <Row k="命宫" v={`${c.ming_ganzhi}（${c.ming_branch}）`} hi />
          <Row k="身宫" v={`${c.shen_branch}宫`} />
          <Row k="五行局" v={c.wuxing_ju} />
          <Row k="紫微" v={`${c.ziwei_branch}宫`} />
          <Row k="天府" v={`${c.tianfu_branch}宫`} />
          <div className="legend"><span><i className="lg zi" />紫微系</span><span><i className="lg fu" />天府系</span><span><i className="lg aux" />辅星</span></div>
        </div>
      </div>
      <Section title={c.sihua.school_id === 'quanshu' ? '四化 · 全书本（王亭之版）' : '四化 · 通行版（中州/三合派）'}>
        <div className="kv-grid">
          <Stat k="化禄" v={`${c.sihua.lu_star}${c.sihua.lu_branch ? ` · ${c.sihua.lu_branch}宫` : ''}`} hi />
          <Stat k="化权" v={`${c.sihua.quan_star}${c.sihua.quan_branch ? ` · ${c.sihua.quan_branch}宫` : ''}`} />
          <Stat k="化科" v={`${c.sihua.ke_star}${c.sihua.ke_branch ? ` · ${c.sihua.ke_branch}宫` : ''}`} />
          <Stat k="化忌" v={`${c.sihua.ji_star}${c.sihua.ji_branch ? ` · ${c.sihua.ji_branch}宫` : ''}`} hi />
        </div>
      </Section>
    </div>
  )
}
function PalaceCell({ p, row, col }: { p: Palace; row: number; col: number }) {
  return (
    <div className={`palace${p.is_ming ? ' ming' : ''}${p.is_shen ? ' shen' : ''}`} style={{ gridRow: row, gridColumn: col }}>
      <div className="p-stars">
        {p.stars.length ? p.stars.map((s) => <span className={`star ${ZIWEI_GROUP.has(s) ? 'zi' : ZIWEI_AUX.has(s) ? 'aux' : 'fu'}`} key={s}>{s}</span>) : <span className="empty">空宫</span>}
      </div>
      <div className="p-foot"><span className="p-name">{p.name}{p.is_shen ? <em>身</em> : null}</span><span className="p-gz">{p.ganzhi}</span></div>
    </div>
  )
}
function Row({ k, v, hi }: { k: string; v: string; hi?: boolean }) {
  return <div className={`crow${hi ? ' hi' : ''}`}><span>{k}</span><b>{v}</b></div>
}

interface MeihuaChart {
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
function Meihua({ c }: { c: MeihuaChart }) {
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

interface XiaoChart { month_deity: string; day_deity: string; hour_deity: string; month_pos: number; day_pos: number; hour_pos: number; lunar_month: number; lunar_day: number; hour_branch: number }
const XIAO_DEITIES = ['大安', '留连', '速喜', '赤口', '小吉', '空亡']
function Xiaoliuren({ c }: { c: XiaoChart }) {
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

interface ZeriChart {
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
function Zeri({ c }: { c: ZeriChart }) {
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

interface MayaChart { jdn: number; tzolkin_number: number; tzolkin_name: string; tzolkin_round: number; haab_day: number; haab_month: string; long_count: number[] }
const LC_UNITS = ['baktun', 'katun', 'tun', 'winal', 'kin']
function Maya({ c }: { c: MayaChart }) {
  return (
    <div className="lp">
      <Section title="Long Count（长计历）">
        <div className="maya-lc">{c.long_count.join(' . ')}</div>
        <div className="lc-units">{LC_UNITS.map((u, i) => <span key={u}><b>{c.long_count[i]}</b>{u}</span>)}</div>
      </Section>
      <Section title="两套循环历">
        <div className="maya-cals">
          <div className="maya-cal"><span className="ml">Tzolkʼin · 神圣历</span><b>{c.tzolkin_number} {c.tzolkin_name}</b><i>环位 {c.tzolkin_round} / 260</i></div>
          <div className="maya-cal"><span className="ml">Haab · 太阳历</span><b>{c.haab_day} {c.haab_month}</b><i>365 日</i></div>
        </div>
        <div className="kv-grid"><Stat k="儒略日 JDN" v={c.jdn.toLocaleString()} /></div>
      </Section>
    </div>
  )
}

interface PawukonChart { day: number; urip: number; wuku: string; triwara: string; pancawara: string; sadwara: string; saptawara: string; dasawara: string; dwiwara: string; ekawara: string | null; caturwara: string; astawara: string; sangawara: string }
function Pawukon({ c }: { c: PawukonChart }) {
  const simple: [string, string, string][] = [['Saptawara', '7', c.saptawara], ['Pancawara', '5', c.pancawara], ['Sadwara', '6', c.sadwara], ['Triwara', '3', c.triwara]]
  const derived: [string, string, string][] = [['Dasawara', '10', c.dasawara], ['Dwiwara', '2', c.dwiwara], ['Ekawara', '1', c.ekawara ?? '—（偶日无）']]
  const stuck: [string, string, string][] = [['Caturwara', '4', c.caturwara], ['Astawara', '8', c.astawara], ['Sangawara', '9', c.sangawara]]
  const grp = (title: string, arr: [string, string, string][]) => (
    <div className="wew-grp"><div className="wew-gt">{title}</div><div className="wewaran">
      {arr.map(([nm, n, v]) => <div className="wew" key={nm}><span className="wn">{nm}<i>{n}</i></span><b>{v}</b></div>)}
    </div></div>
  )
  return (
    <div className="lp">
      <Section title={`Pawukon 第 ${c.day} / 210 日 · Wuku ${c.wuku} · urip ${c.urip}`}>
        {grp('简单 mod 週', simple)}
        {grp('urip 之和派生', derived)}
        {grp('卡日週（n∤210）', stuck)}
      </Section>
    </div>
  )
}

interface MahaboteChart { core: number; house: string; planet: string; weekday: string; weekday_index: number; myanmar_year: number }
const MHB_HOUSES = ['Binga', 'Atun', 'Yaza', 'Adipati', 'Marana', 'Thike', 'Puti']
function Mahabote({ c }: { c: MahaboteChart }) {
  return (
    <div className="lp">
      <Section title="七宫（本命宫高亮）">
        <div className="mhb-houses">
          {MHB_HOUSES.map((h, i) => <div className={`mhb${i === c.core ? ' hi' : ''}`} key={h}><span className="mhb-i">{i}</span><span className="mhb-n">{h}</span></div>)}
        </div>
      </Section>
      <Section title="本命">
        <div className="kv-grid">
          <Stat k="本命宫" v={c.house} hi />
          <Stat k="核心数" v={`${c.core} / 7`} />
          <Stat k="行星（八天週）" v={c.planet} />
          <Stat k="出生星期" v={c.weekday} />
          <Stat k="缅历年" v={c.myanmar_year} />
        </div>
      </Section>
    </div>
  )
}

interface TibetanChart { year: number; animal: string; element: string; male: boolean; mewa: number; mewa_color: string; sexagenary: number; rabjung: number; year_in_rabjung: number }
const MEWA_HEX: Record<string, string> = { White: '#e9e2d0', Black: '#2a2620', Blue: '#3a6ea5', Green: '#5fb06a', Yellow: '#d4a843', Red: '#d8392e', Maroon: '#7a2e2a' }
function Tibetan({ c }: { c: TibetanChart }) {
  return (
    <div className="lp">
      <Section title="年柱（六十周期）">
        <div className="tib-big">{c.element} · {c.male ? '阳' : '阴'} · {c.animal}</div>
        <div className="kv-grid">
          <Stat k="六十周期位" v={`第 ${c.sexagenary} / 60`} />
          <Stat k="绕迥 rabjung" v={`第 ${c.rabjung} 期`} />
          <Stat k="期内年" v={`第 ${c.year_in_rabjung}`} />
        </div>
      </Section>
      <Section title="年 Mewa（九宫 sme ba）">
        <div className="mewa-show">
          <span className="mewa-big" style={{ background: MEWA_HEX[c.mewa_color] ?? '#888', color: c.mewa_color === 'White' || c.mewa_color === 'Yellow' ? '#100d0a' : '#fff' }}>{c.mewa}</span>
          <span className="mewa-cn">{c.mewa_color}</span>
        </div>
      </Section>
    </div>
  )
}

// ============ B 族：占星圆形星盘 ============
interface AstroChart {
  angles?: { asc_sign: string; asc_degree: number; mc_sign: string; mc_degree: number; ascendant: number; midheaven: number }
  aspects: { a: string; b: string; kind: string; angle: number }[]
  houses?: { number: number; sign: string; planets: string[] }[]
  cusp_system?: string
  cusp_houses?: { number: number; cusp_longitude: number; cusp_sign: string; cusp_degree: number; planets: string[] }[]
  planets: { name: string; sign: string; degree: number; longitude: number; house: number }[]
}
const CUSP_SYSTEM_LABEL: Record<string, string> = {
  placidus: 'Placidus 半弧三分',
  koch: 'Koch 等赤经四分',
  whole_sign: '整宫制 Whole Sign',
  equal: 'Equal 等宫',
  porphyry: 'Porphyry 黄道三分',
}
function Astrology({ c }: { c: AstroChart }) {
  const S = 360, cx = S / 2, cy = S / 2, rOuter = 168, rSign = 144, rPlanet = 116, rAspect = 96
  const ascLon = c.angles?.ascendant ?? 0
  // 黄经 → 屏幕角（Asc 置左=180°，黄经增大逆时针）
  const scr = (lon: number) => (180 + (lon - ascLon)) * Math.PI / 180
  const pt = (lon: number, r: number) => ({ x: cx + r * Math.cos(scr(lon)), y: cy - r * Math.sin(scr(lon)) })
  return (
    <div className="lp">
      <div className="astro-wrap">
        <svg className="wheel" viewBox={`0 0 ${S} ${S}`} width="360" height="360">
          <circle cx={cx} cy={cy} r={rOuter} className="w-ring" />
          <circle cx={cx} cy={cy} r={rSign} className="w-ring" />
          <circle cx={cx} cy={cy} r={rAspect} className="w-ring faint" />
          {SIGN_NAMES.map((_, i) => {
            const a = scr(i * 30)
            const p1 = { x: cx + rSign * Math.cos(a), y: cy - rSign * Math.sin(a) }
            const p2 = { x: cx + rOuter * Math.cos(a), y: cy - rOuter * Math.sin(a) }
            const gp = pt(i * 30 + 15, (rSign + rOuter) / 2)
            return <g key={i}>
              <line x1={p1.x} y1={p1.y} x2={p2.x} y2={p2.y} className="w-spoke" />
              <text x={gp.x} y={gp.y} className="w-sign" dominantBaseline="central" textAnchor="middle">{SIGN_GLYPH[SIGN_NAMES[i]]}</text>
            </g>
          })}
          {/* 相位线 */}
          {c.aspects.map((asp, i) => {
            const pa = c.planets.find((p) => p.name === asp.a), pb = c.planets.find((p) => p.name === asp.b)
            if (!pa || !pb) return null
            const A = pt(pa.longitude, rAspect), B = pt(pb.longitude, rAspect)
            return <line key={i} x1={A.x} y1={A.y} x2={B.x} y2={B.y} stroke={ASPECT_COLOR[asp.kind] ?? '#888'} strokeWidth={1} opacity={0.5} />
          })}
          {/* 中间宫尖（11/12/2/3 + 对宫）细虚线 */}
          {c.cusp_houses?.filter((h) => ![1, 4, 7, 10].includes(h.number)).map((h) => {
            const p = pt(h.cusp_longitude, rSign)
            return <g key={`cusp-${h.number}`}>
              <line x1={cx} y1={cy} x2={p.x} y2={p.y} stroke="#999" strokeWidth={0.5} strokeDasharray="2,3" opacity={0.55} />
              <text x={pt(h.cusp_longitude, rSign - 8).x} y={pt(h.cusp_longitude, rSign - 8).y} className="w-cusp-num" fontSize={7} fill="#888" dominantBaseline="central" textAnchor="middle">{h.number}</text>
            </g>
          })}
          {/* Asc / MC */}
          {c.angles && <>
            <line x1={cx} y1={cy} x2={pt(c.angles.ascendant, rOuter).x} y2={pt(c.angles.ascendant, rOuter).y} className="w-axis" />
            <line x1={cx} y1={cy} x2={pt(c.angles.midheaven, rOuter).x} y2={pt(c.angles.midheaven, rOuter).y} className="w-axis" />
            <text x={pt(c.angles.ascendant, rOuter + 9).x} y={pt(c.angles.ascendant, rOuter + 9).y} className="w-ax-lbl" dominantBaseline="central" textAnchor="middle">Asc</text>
            <text x={pt(c.angles.midheaven, rOuter + 9).x} y={pt(c.angles.midheaven, rOuter + 9).y} className="w-ax-lbl" dominantBaseline="central" textAnchor="middle">MC</text>
          </>}
          {/* 行星 */}
          {c.planets.map((p) => {
            const g = pt(p.longitude, rPlanet)
            return <g key={p.name}>
              <text x={g.x} y={g.y} className="w-planet" dominantBaseline="central" textAnchor="middle">{PLANET_GLYPH[p.name] ?? '·'}</text>
            </g>
          })}
        </svg>
        <div className="astro-side">
          {c.angles && <div className="kv-grid">
            <Stat k="上升 Asc" v={`${SIGN_GLYPH[c.angles.asc_sign] ?? ''} ${c.angles.asc_sign} ${c.angles.asc_degree.toFixed(1)}°`} hi />
            <Stat k="中天 MC" v={`${SIGN_GLYPH[c.angles.mc_sign] ?? ''} ${c.angles.mc_sign} ${c.angles.mc_degree.toFixed(1)}°`} hi />
          </div>}
          <div className="astro-grid">
            {c.planets.map((p) => (
              <div className="astro-row" key={p.name}>
                <span className="ag glyph">{PLANET_GLYPH[p.name] ?? '·'}</span>
                <span className="ag pn">{p.name}</span>
                <span className="ag sign">{SIGN_GLYPH[p.sign] ?? ''} {p.sign}</span>
                <span className="ag deg">{p.degree.toFixed(1)}°</span>
                <span className="ag hs">{p.house}宫</span>
              </div>
            ))}
          </div>
        </div>
      </div>
      <Section title={`相位 · ${c.aspects.length}`}>
        <div className="aspects">
          {c.aspects.map((a, i) => <span className="aspect" key={i}>{a.a}<b style={{ color: ASPECT_COLOR[a.kind] }}>{ASPECT_SYM[a.kind] ?? a.kind}</b>{a.b}<i>{a.angle.toFixed(0)}°</i></span>)}
        </div>
      </Section>
      {c.cusp_houses && <Section title={`十二宫 · ${CUSP_SYSTEM_LABEL[c.cusp_system ?? ''] ?? c.cusp_system ?? '分宫'}`}>
        <div className="astro-grid">
          {c.cusp_houses.map((h) => (
            <div className="astro-row" key={h.number}>
              <span className="ag pn">{h.number}宫</span>
              <span className="ag sign">{SIGN_GLYPH[h.cusp_sign] ?? ''} {h.cusp_sign}</span>
              <span className="ag deg">{h.cusp_degree.toFixed(1)}°</span>
              <span className="ag hs">{h.planets.join('·') || '—'}</span>
            </div>
          ))}
        </div>
      </Section>}
    </div>
  )
}

// ============ Jyotish （印度占星） ============
const AYANAMSA_LABEL: Record<string, string> = {
  lahiri: 'Lahiri （印度官方）',
  krishnamurti: 'Krishnamurti (KP)',
  raman: 'Raman',
  fagan_bradley: 'Fagan-Bradley',
}
function fmtDeg(d: number) {
  const r = ((d % 30) + 30) % 30
  const m = Math.floor(r)
  const s = Math.round((r - m) * 60)
  return `${m}°${String(s).padStart(2, '0')}'`
}
function fmtAge(y: number) {
  const sign = y < 0 ? '-' : ''
  const a = Math.abs(y)
  const yr = Math.floor(a)
  const mo = Math.round((a - yr) * 12)
  return `${sign}${yr}y${mo}m`
}
function Jyotish({ c }: { c: JyotishChart }) {
  return (
    <div className="lp">
      <Section title={`Ayanamsa · ${AYANAMSA_LABEL[c.ayanamsa_id] ?? c.ayanamsa_id} · ${c.ayanamsa_deg.toFixed(4)}°`}>
        <div className="kv-grid">
          <Stat k="出生 Mahadasha 主星" v={c.birth_dasha_lord} hi />
          {c.lagna_rasi_name && c.lagna_lon !== null && (
            <Stat k="Lagna （上升）" v={`${c.lagna_rasi_name} ${fmtDeg(c.lagna_lon)}`} hi />
          )}
          {c.lagna_navamsa_name && (
            <Stat k="Lagna · D-9 Navamsa" v={c.lagna_navamsa_name} />
          )}
        </div>
      </Section>
      <Section title="九曜 (Navagraha) · D-1 · D-9">
        <table className="jy-graha-table">
          <thead>
            <tr><th>行星</th><th>恒星黄经</th><th>Rasi</th><th>Nakshatra</th><th>Lord</th><th>Navamsa (D-9)</th></tr>
          </thead>
          <tbody>
            {c.grahas.map((g) => (
              <tr key={g.graha}>
                <td><b>{g.name}</b></td>
                <td>{fmtDeg(g.sidereal_lon)}</td>
                <td>{g.rasi_name}</td>
                <td>{g.nakshatra_name}</td>
                <td>{g.nakshatra_lord}</td>
                <td>{g.navamsa_name}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Section>
      <Section title="Vimshottari Mahadasha · 120 年 timeline">
        <table className="jy-graha-table">
          <thead>
            <tr><th>主星</th><th>持续</th><th>起 （出生后）</th><th>止 （出生后）</th></tr>
          </thead>
          <tbody>
            {c.mahadashas.map((d, i) => (
              <tr key={`${d.lord}-${i}`} className={i === 0 ? 'jy-md-now' : ''}>
                <td><b>{d.lord}</b>{i === 0 && <em> · birth</em>}</td>
                <td>{d.effective_years.toFixed(2)}y</td>
                <td>{fmtAge(d.start_age_years)}</td>
                <td>{fmtAge(d.end_age_years)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Section>
    </div>
  )
}

// ============ 七政四余 （中国本土星占） ============
function Qizhengsiyu({ c }: { c: QizhengsiyuChart }) {
  const qz = c.stars.filter((s) => s.is_qizheng)
  const sy = c.stars.filter((s) => !s.is_qizheng)
  return (
    <div className="lp">
      <Section title={`日柱 · ${c.day_ganzhi} · 28 宿值日 · ${c.mansion_name}`}>
        <div className="kv-grid">
          <Stat k="日柱干支" v={c.day_ganzhi} hi />
          <Stat k="28 宿值日" v={`${c.mansion_name}(idx ${c.mansion})`} hi />
        </div>
      </Section>
      <Section title="七政 · 日月五星地心黄经">
        <table className="jy-graha-table">
          <thead>
            <tr><th>星名</th><th>黄经</th><th>宫名</th><th>宫内度数</th></tr>
          </thead>
          <tbody>
            {qz.map((s) => (
              <tr key={s.star}>
                <td><b>{s.name}</b></td>
                <td>{s.longitude.toFixed(2)}°</td>
                <td>{s.sign_name}</td>
                <td>{fmtDeg(s.degree_in_sign)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Section>
      <Section title="四余 · 罗㬋 / 计都 / 月孛（紫炁 🟡 不入，无天文实体）">
        <table className="jy-graha-table">
          <thead>
            <tr><th>星名</th><th>黄经</th><th>宫名</th><th>宫内度数</th></tr>
          </thead>
          <tbody>
            {sy.map((s) => (
              <tr key={s.star}>
                <td><b>{s.name}</b></td>
                <td>{s.longitude.toFixed(2)}°</td>
                <td>{s.sign_name}</td>
                <td>{fmtDeg(s.degree_in_sign)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Section>
    </div>
  )
}

// ============ C 族 ============
interface YijingChart {
  method: string
  primary_upper: string; primary_lower: string; resulting_upper: string; resulting_lower: string
  lines: { value: number; yang: boolean; changing: boolean }[]
  primary: number; resulting: number
  // 64 卦名 + 文王序
  primary_name: string; primary_full_name: string; primary_king_wen: number
  resulting_name: string; resulting_full_name: string; resulting_king_wen: number
}
const YAO_VAL: Record<number, string> = { 6: '老阴 ⚋✕', 7: '少阳 ⚊', 8: '少阴 ⚋', 9: '老阳 ⚊○' }
function Yijing({ c }: { c: YijingChart }) {
  const primary = c.lines.map((l) => l.yang)
  const movingLine = c.lines.findIndex((l) => l.changing) + 1
  return (
    <div className="lp">
      <Section title="本卦 → 之卦">
        <div className="row-hex big-row">
          <Hexagram lines={primary} moving={movingLine || undefined} label={`本卦 · ${c.primary_full_name}`} sub={`${c.primary_upper}上 ${c.primary_lower}下 · 文王 ${c.primary_king_wen}`} big />
          <span className="hex-arrow">→</span>
          <Hexagram lines={hexLines(c.resulting)} label={`之卦 · ${c.resulting_full_name}`} sub={`${c.resulting_upper}上 ${c.resulting_lower}下 · 文王 ${c.resulting_king_wen}`} big />
        </div>
      </Section>
      <Section title="六爻（自下而上）">
        <div className="yao-list">
          {c.lines.map((l, i) => (
            <div className={`yao-li${l.changing ? ' moving' : ''}`} key={i}><span className="yl-n">{['初', '二', '三', '四', '五', '上'][i]}</span><span className="yl-v">{YAO_VAL[l.value]}</span></div>
          ))}
        </div>
        <div className="kv-grid"><Stat k="起卦法" v={c.method === 'ThreeCoins' ? '三钱法' : c.method} /><Stat k="变爻数" v={c.lines.filter((l) => l.changing).length} hi /></div>
      </Section>
    </div>
  )
}

interface GeomancyChart { mothers: number[]; daughters: number[]; nieces: number[]; witnesses: number[]; judge: number; judge_even: boolean }
function Geomancy({ c }: { c: GeomancyChart }) {
  return (
    <div className="lp">
      <Section title="盾牌图（Shield Chart）">
        <div className="shield-pyramid">
          <div className="sp-row">{[...c.daughters].reverse().map((v, i) => <GeoFigure key={'d' + i} value={v} label={`女${4 - i}`} />)}{[...c.mothers].reverse().map((v, i) => <GeoFigure key={'m' + i} value={v} label={`母${4 - i}`} hi />)}</div>
          <div className="sp-row">{[...c.nieces].reverse().map((v, i) => <GeoFigure key={'n' + i} value={v} label={`侄${4 - i}`} />)}</div>
          <div className="sp-row">{[...c.witnesses].reverse().map((v, i) => <GeoFigure key={'w' + i} value={v} label={i === 0 ? '左证' : '右证'} />)}</div>
          <div className="sp-row"><GeoFigure value={c.judge} label={`法官 ${c.judge_even ? '✓偶' : '!奇'}`} hi /></div>
        </div>
      </Section>
    </div>
  )
}

interface SikidyChart { mothers: number[]; columns: number[]; seer: number; seer_even: boolean }
function Sikidy({ c }: { c: SikidyChart }) {
  return (
    <div className="lp">
      <Section title="Sikidy 棋盘（16 列）">
        <div className="sik-grid">
          {c.columns.map((v, i) => <GeoFigure key={i} value={v} label={String(i + 1)} hi={i < 4} />)}
        </div>
        <div className="kv-grid"><Stat k="四母（前 4 列）" v="随机起" /><Stat k="创世者 C15" v={c.seer_even ? '偶 ✓' : '奇 ！'} hi /></div>
      </Section>
    </div>
  )
}

interface IfaChart { index: number; left: number; right: number; left_marks: boolean[]; right_marks: boolean[] }
function Ifa({ c }: { c: IfaChart }) {
  const col = (marks: boolean[], lbl: string) => (
    <div className="odu-col"><div className="odu-marks">{[0, 1, 2, 3].map((i) => <div className="odu-row" key={i}>{marks[i] ? <span className="odu-mark" /> : <><span className="odu-mark" /><span className="odu-mark" /></>}</div>)}</div><div className="odu-lbl">{lbl}</div></div>
  )
  return (
    <div className="lp">
      <Section title="Odu（双 figure）">
        <div className="odu big">{col(c.left_marks, `左 #${c.left}`)}{col(c.right_marks, `右 #${c.right}`)}</div>
        <div className="kv-grid"><Stat k="Odu 序" v={`${c.index} / 256`} hi /></div>
      </Section>
    </div>
  )
}

interface TarotChart { cards: { index: number; reversed: boolean; name: string; name_zh: string; glyph: string }[]; deck_size: number; reversible: boolean; deck_id?: string }
const DECK_NAMES: Record<string, string> = {
  tarot_full: '塔罗 78',
  tarot_major: '塔罗大阿卡纳 22',
  lenormand: 'Petit Lenormand 36',
  elder_futhark: 'Elder Futhark 卢恩 24',
  younger_futhark: 'Younger Futhark 卢恩 16',
}
function Tarot({ c }: { c: TarotChart }) {
  const labels = ['过去', '现在', '未来']
  const deckName = c.deck_id ? (DECK_NAMES[c.deck_id] ?? `Deck ${c.deck_id}`) : '塔罗 78'
  return (
    <div className="lp">
      <Section title={`牌阵 · ${deckName}（过去 · 现在 · 未来）`}>
        <div className="tarot big">
          {c.cards.map((card, i) => (
            <div className={`tcard${card.reversed ? ' rev' : ''}`} key={i}>
              <div className="tc-pos">{labels[i] ?? `#${i + 1}`}</div>
              <div className="tc-num">#{card.index}{card.glyph && <span style={{ marginLeft: 6, fontSize: '1.3em' }}>{card.glyph}</span>}</div>
              {card.name && <div className="tc-name" style={{ fontWeight: 600, marginTop: 4 }}>{card.name}</div>}
              {card.name_zh && <div className="tc-name-zh" style={{ color: '#888' }}>{card.name_zh}</div>}
              {c.reversible ? <div className="tc-ori">{card.reversed ? '逆位 ↺' : '正位 ↑'}</div> : <div className="tc-ori">不计逆位</div>}
            </div>
          ))}
        </div>
      </Section>
    </div>
  )
}

// ============ D 族 ============
interface NameNums { system: string; expression: number; soul_urge: number; personality: number }
interface NumChart { life_path: number; birthday: number; pythagorean: NameNums | null; chaldean: NameNums | null }
function Numerology({ c }: { c: NumChart }) {
  const master = (n: number) => [11, 22, 33].includes(n)
  return (
    <div className="lp">
      <Section title="核心数（出生日期）">
        <div className="num-big">
          <div className="nb"><b className={master(c.life_path) ? 'master' : ''}>{c.life_path}</b><span>生命灵数{master(c.life_path) ? ' · 主数' : ''}</span></div>
          <div className="nb"><b>{c.birthday}</b><span>生日数</span></div>
        </div>
      </Section>
      {(c.pythagorean || c.chaldean) ? (
        <Section title="姓名数（两套字母表）">
          <table className="num-tbl"><thead><tr><th></th><th>表达 Expression</th><th>灵魂 Soul</th><th>人格 Personality</th></tr></thead><tbody>
            {c.pythagorean && <tr><td>Pythagorean</td><td>{c.pythagorean.expression}</td><td>{c.pythagorean.soul_urge}</td><td>{c.pythagorean.personality}</td></tr>}
            {c.chaldean && <tr><td>Chaldean</td><td>{c.chaldean.expression}</td><td>{c.chaldean.soul_urge}</td><td>{c.chaldean.personality}</td></tr>}
          </tbody></table>
        </Section>
      ) : <Note>填入「姓名」以计算表达／灵魂／人格数</Note>}
    </div>
  )
}

// ============ ⟂ 横切 ============
interface LiurenChart { day_stem: number; day_branch: number; hour_branch: number; month_general: number; month_general_name: string; offset: number; heaven: number[]; courses: { down: number; up: number }[]; pattern: string; transmission: number[] | null }
const LIUREN_PATTERN: Record<string, string> = {
  ZhongShen: '重审', YuanShou: '元首', BiYong: '比用', SheHai: '涉害', HaoShi: '蒿矢', TanShe: '弹射',
  MaoXing: '昴星', BieZe: '别责', BaZhuan: '八专', FuYin: '伏吟', FanYin: '返吟',
}
function Liuren({ c }: { c: LiurenChart }) {
  // 天地盘圆环：12 地支地盘均布，天盘在外圈
  const S = 300, cx = S / 2, cy = S / 2
  const ang = (g: number) => (-90 - g * 30) * Math.PI / 180 // 子在上、顺时针
  const pt = (g: number, r: number) => ({ x: cx + r * Math.cos(ang(g)), y: cy + r * Math.sin(ang(g)) })
  return (
    <div className="lp">
      <div className="liuren-wrap">
        <svg className="plate-svg" viewBox={`0 0 ${S} ${S}`} width="300" height="300">
          <circle cx={cx} cy={cy} r={138} className="w-ring" />
          <circle cx={cx} cy={cy} r={104} className="w-ring" />
          <circle cx={cx} cy={cy} r={70} className="w-ring faint" />
          {BRANCHES.map((b, g) => {
            const he = pt(g, 121), ea = pt(g, 87)
            return <g key={g}>
              <text x={he.x} y={he.y} className="pt-h" dominantBaseline="central" textAnchor="middle">{BRANCHES[c.heaven[g]]}</text>
              <text x={ea.x} y={ea.y} className="pt-e" dominantBaseline="central" textAnchor="middle">{b}</text>
            </g>
          })}
          <text x={cx} y={cy - 8} className="plate-c1" dominantBaseline="central" textAnchor="middle">{STEMS[c.day_stem]}{BRANCHES[c.day_branch]}</text>
          <text x={cx} y={cy + 12} className="plate-c2" dominantBaseline="central" textAnchor="middle">{LIUREN_PATTERN[c.pattern] ?? c.pattern}</text>
        </svg>
        <div className="liuren-side">
          <div className="kv-grid">
            <Stat k="日干支" v={`${STEMS[c.day_stem]}${BRANCHES[c.day_branch]}`} />
            <Stat k="月将·占时" v={`${c.month_general_name}·${BRANCHES[c.hour_branch]}`} />
            <Stat k="课式" v={LIUREN_PATTERN[c.pattern] ?? c.pattern} hi />
          </div>
          <div className="lp-sec-t" style={{ marginTop: 12 }}>四课（右起一→四）</div>
          <div className="courses">
            {[...c.courses].reverse().map((q, i) => (
              <div className="course" key={i}><span className="cu">{BRANCHES[q.up]}</span><span className="cd">{BRANCHES[q.down]}</span></div>
            ))}
          </div>
          <div className="lp-sec-t" style={{ marginTop: 12 }}>三传</div>
          <div className="san-chuan">{c.transmission ? c.transmission.map((t, i) => <span key={i} className="sc-cell">{['初', '中', '末'][i]} {BRANCHES[t]}</span>) : <span className="und">🟡 该课式（{LIUREN_PATTERN[c.pattern]}）取传流派分歧，未强编</span>}</div>
        </div>
      </div>
    </div>
  )
}

function Grid9({ render, head }: { render: (gong: number) => React.ReactNode; head?: string }) {
  const pos: Record<number, [number, number]> = {}
  for (let r = 0; r < 3; r++) for (let col = 0; col < 3; col++) pos[LUOSHU[r][col]] = [r, col]
  return (
    <>
      {head && <div className="qm-head">{head}</div>}
      <div className="grid9">
        {Array.from({ length: 9 }, (_, k) => k + 1).map((gong) => {
          const [r, col] = pos[gong]
          return <div className="g9-pos" key={gong} style={{ gridRow: r + 1, gridColumn: col + 1 }}>{render(gong)}</div>
        })}
      </div>
    </>
  )
}

interface QimenXun { head_ganzhi: string; head_branch: number; head_yi: string; head_yi_stem: number; xunkong: [string, string] }
interface QimenChart {
  setup: { ju: number; term: string; yang_dun: boolean; yuan: string }
  fu_tou_branch: number
  time_ganzhi: string
  time_stem: number
  time_branch: number
  xun: QimenXun
  earth: string[]
  palace: string[]
  xun_yi_palace: number
  zhi_fu_stem: number
  zhi_fu_stem_name: string
  zhi_fu_palace: number
  zhi_fu_xing: string
  jiuxing_earth: string[]
  sky: QimenSky
  gates: QimenGates
  spirits: QimenSpirits
  month_branch: number
  month_element: string
  star_vigor: string[]
  patterns: QimenPatterns
}

interface QimenPatterns {
  star_fu_yin: boolean
  star_fan_yin: boolean
  gate_fu_yin: boolean
  gate_fan_yin: boolean
  stem_fu_yin_palaces: number[]
  full_fu_yin: boolean
  qi_gates: { palace: number; qi: string; gate: string }[]
}

interface QimenSpirits {
  start_palace: number
  spirits: string[]
  spirits_alt: string[]
}

interface QimenGates {
  zhi_shi_gate: string
  zhi_shi_palace: number
  steps: number
  shift: number
  gates: string[]
}

interface QimenSky {
  shift: number
  stars: string[]
  stems: string[]
  center_stem: string
  center_palace: number
}
const BRANCH_NAMES = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥']
const YUAN_CN: Record<string, string> = { Upper: '上元', Middle: '中元', Lower: '下元' }
function Qimen({ c }: { c: QimenChart }) {
  const gongName = (g: number) => `${c.palace[g - 1]}${g}`
  const headGongName = gongName(c.xun_yi_palace)
  const zhiFuGongName = gongName(c.zhi_fu_palace)
  const pat = c.patterns
  const noPattern = !pat.full_fu_yin && !pat.star_fu_yin && !pat.star_fan_yin && !pat.gate_fu_yin
    && !pat.gate_fan_yin && pat.stem_fu_yin_palaces.length === 0 && pat.qi_gates.length === 0
  return (
    <div className="lp">
      <Section title={`${c.setup.yang_dun ? '阳遁' : '阴遁'} ${c.setup.ju} 局 · ${c.setup.term} · ${YUAN_CN[c.setup.yuan] ?? c.setup.yuan}`}>
        <div className="kv-grid">
          <Stat k="占事时柱" v={c.time_ganzhi} hi />
          <Stat k="所在旬" v={`${c.xun.head_ganzhi} 旬`} />
          <Stat k="旬遁六仪" v={c.xun.head_yi} />
          <Stat k="旬空" v={`${c.xun.xunkong[0]} ${c.xun.xunkong[1]}`} />
          <Stat k="值符干" v={`${c.zhi_fu_stem_name}${c.time_stem === 0 ? ' (甲遁仪)' : ''}`} hi />
          <Stat k="值符宫" v={zhiFuGongName} hi />
          <Stat k="值符星（本旬）" v={c.zhi_fu_xing} />
          <Stat k="值使门" v={c.gates.zhi_shi_gate} hi />
          <Stat k="值使落宫" v={gongName(c.gates.zhi_shi_palace)} hi />
          <Stat k="月令" v={`${c.month_element}（${BRANCH_NAMES[c.month_branch]}月）`} />
        </div>
      </Section>

      <Section title="时家盘 · 九宫">
        <Grid9 head={`每宫自上而下：八神 · 天盘星（后缀旺衰） · 八门 · 天盘干／地盘干。值符星 ${c.zhi_fu_xing} 自旬首宫 ${headGongName} 转到 ${zhiFuGongName}（转 ${c.sky.shift} 格）；值使 ${c.gates.zhi_shi_gate} 落 ${gongName(c.gates.zhi_shi_palace)}（转 ${c.gates.shift} 格）；八神自值符宫${c.setup.yang_dun ? '顺' : '逆'}布`}
          render={(g) => {
            const k = g - 1
            const center = g === 5
            const cls = ['qm-cell']
            if (g === c.zhi_fu_palace) cls.push('is-zhifu')
            if (g === c.gates.zhi_shi_palace) cls.push('is-zhishi')
            if (g === c.xun_yi_palace) cls.push('is-yi')
            return (
              <div className={cls.join(' ')}>
                <div className="qm-r1">
                  <span className="qm-shen" title={c.spirits.spirits_alt[k] !== c.spirits.spirits[k] ? `另一系作「${c.spirits.spirits_alt[k]}」` : undefined}>
                    {c.spirits.spirits[k] || (center ? '' : '—')}
                  </span>
                  <span className="qm-gong">{c.palace[k]}{g}</span>
                </div>
                <div className="qm-r2" title={`原配 ${c.jiuxing_earth[k]}`}>
                  {center ? '天禽寄坤 2' : (c.sky.stars[k] || '—')}
                  {c.star_vigor[k] && <b className="qm-vigor">{c.star_vigor[k]}</b>}
                </div>
                <div className="qm-r3">{c.gates.gates[k] || (center ? '' : '—')}</div>
                <div className="qm-r4">
                  <b className="qm-sky-stem">{c.sky.stems[k] || '—'}</b>
                  {g === c.sky.center_palace && <b className="qm-lodged" title="中宫寄干，随坤 2 同转">{c.sky.center_stem}</b>}
                  <i className="qm-sep">／</i>
                  <span className="qm-earth-stem">{c.earth[k]}</span>
                </div>
              </div>
            )
          }} />
        <div className="qm-legend">
          <span className="qm-key is-zhifu">值符宫</span>
          <span className="qm-key is-zhishi">值使宫</span>
          <span className="qm-key is-yi">旬首宫</span>
          <span className="qm-key-note">天盘干在上、地盘干在下；星名悬停可见其原配宫位</span>
        </div>
      </Section>

      <Section title="盘面格局（结构事实，吉凶判读交释义层）">
        <div className="qm-pat">
          {pat.full_fu_yin && <span className="qm-chip on">全盘伏吟</span>}
          {pat.star_fu_yin && <span className="qm-chip">星伏吟</span>}
          {pat.star_fan_yin && <span className="qm-chip">星反吟</span>}
          {pat.gate_fu_yin && <span className="qm-chip">门伏吟</span>}
          {pat.gate_fan_yin && <span className="qm-chip">门反吟</span>}
          {pat.stem_fu_yin_palaces.length > 0 &&
            <span className="qm-chip">干伏吟 · {pat.stem_fu_yin_palaces.map(gongName).join(' ')}</span>}
          {pat.qi_gates.map((q) => (
            <span className="qm-chip qi" key={q.palace}>{q.qi} 临 {q.gate} · {gongName(q.palace)}</span>
          ))}
          {noPattern && <span className="qm-chip none">本盘无已收格局</span>}
        </div>
      </Section>

      <div className="lp-note">四盘齐全：地盘三奇六仪 · 天盘（九星 + 三奇六仪随值符转）· 人盘八门（值使随时辰数）· 神盘八神；星名后小字是该星在月令下的旺相休囚死。🟡 八神第 5 / 6 位两系称谓相左（白虎 / 玄武 与 勾陈 / 朱雀），位序一致故悬停可见另一系；中宫与天禽寄宫取通行的坤 2。格局只收无争议的伏吟 / 反吟 / 三奇临吉门，其余 200+ 条各家出入大，未收。</div>
    </div>
  )
}

interface TaiyiChart { year: number; jinian: number; yang_dun: boolean; taiyi: { step: number; palace: number; gua: string; year_in_palace: number; sancai: string } }
function Taiyi({ c }: { c: TaiyiChart }) {
  return (
    <div className="lp">
      <Section title="太乙行九宫（不入中五）">
        <Grid9 render={(g) => <div className={`g9 ty${g === c.taiyi.palace ? ' taiyi' : ''}${g === 5 ? ' mid' : ''}`}><span className="g9-gong big">{g}</span>{g === c.taiyi.palace && <span className="g9-stem sm">太乙</span>}</div>} />
      </Section>
      <Section title="本盘">
        <div className="kv-grid">
          <Stat k="太乙落宫" v={`${c.taiyi.palace}宫 · ${c.taiyi.gua}`} hi />
          <Stat k="三才" v={c.taiyi.sancai} />
          <Stat k="阴阳遁" v={c.yang_dun ? '阳遁' : '阴遁'} />
          <Stat k="太乙积年" v={c.jinian.toLocaleString()} />
        </div>
      </Section>
    </div>
  )
}

// ============ 派发器 ============
export function LeafChart({ leaf }: { leaf: CastLeaf }) {
  const ch = leaf.chart
  switch (leaf.id) {
    case 'bazi': return <BaziView c={ch as BaziChart} />
    case 'ziwei': return <ZiweiView c={ch as ZiweiChart} />
    case 'astrology': return <Astrology c={ch as AstroChart} />
    case 'jyotish': return <Jyotish c={ch as JyotishChart} />
    case 'qizhengsiyu': return <Qizhengsiyu c={ch as QizhengsiyuChart} />
    case 'yijing': return <Yijing c={ch as YijingChart} />
    case 'geomancy': return <Geomancy c={ch as GeomancyChart} />
    case 'sikidy': return <Sikidy c={ch as SikidyChart} />
    case 'ifa': return <Ifa c={ch as IfaChart} />
    case 'tarot': return <Tarot c={ch as TarotChart} />
    case 'meihua': return <Meihua c={ch as MeihuaChart} />
    case 'xiaoliuren': return <Xiaoliuren c={ch as XiaoChart} />
    case 'zeri': return <Zeri c={ch as ZeriChart} />
    case 'maya': return <Maya c={ch as MayaChart} />
    case 'pawukon': return <Pawukon c={ch as PawukonChart} />
    case 'mahabote': return <Mahabote c={ch as MahaboteChart} />
    case 'liuren': return <Liuren c={ch as LiurenChart} />
    case 'qimen': return <Qimen c={ch as QimenChart} />
    case 'taiyi': return <Taiyi c={ch as TaiyiChart} />
    case 'tibetan': return <Tibetan c={ch as TibetanChart} />
    case 'numerology': return <Numerology c={ch as NumChart} />
    default: return <pre className="jv-fallback">{JSON.stringify(ch, null, 1)}</pre>
  }
}
