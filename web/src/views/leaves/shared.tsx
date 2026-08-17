// 各叶共用的常量与小件。
//
// 每片叶都在自己的文件里，但爻、卦象、地占图形、九宫格这些画法是跨叶的——
// 易经与梅花共用爻与卦象，地占与 Sikidy 共用图形，六壬奇门太乙共用九宫格。
// 放在这里，改一处所有用到的叶一起变。

export const BRANCHES = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥']
export const STEMS = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸']
// 八卦值(0..7)→名/五行（与 mingli-gua TRIGRAM_NAMES 同序：坤震坎兑艮离巽乾）
export const TRIGRAM_NAME = ['坤', '震', '坎', '兑', '艮', '离', '巽', '乾']
export const TRIGRAM_WX = ['土', '木', '水', '金', '土', '火', '木', '金']
export const TRIGRAM_BY_NAME: Record<string, number> = Object.fromEntries(TRIGRAM_NAME.map((n, i) => [n, i]))
export const PLANET_GLYPH: Record<string, string> = {
  太阳: '☉', 太阴: '☽', 月亮: '☽', 水星: '☿', 金星: '♀', 火星: '♂',
  木星: '♃', 土星: '♄', 天王: '♅', 天王星: '♅', 海王: '♆', 海王星: '♆', 冥王: '♇',
}
export const SIGN_NAMES = ['白羊', '金牛', '双子', '巨蟹', '狮子', '处女', '天秤', '天蝎', '射手', '摩羯', '水瓶', '双鱼']
export const SIGN_GLYPH: Record<string, string> = {
  白羊: '♈', 金牛: '♉', 双子: '♊', 巨蟹: '♋', 狮子: '♌', 处女: '♍',
  天秤: '♎', 天蝎: '♏', 射手: '♐', 摩羯: '♑', 水瓶: '♒', 双鱼: '♓',
}
export const ASPECT_SYM: Record<string, string> = { 合: '☌', 冲: '☍', 拱: '△', 刑: '□', 六分: '⚹' }
export const ASPECT_COLOR: Record<string, string> = { 合: '#d4a843', 冲: '#e0584c', 拱: '#5fb06a', 刑: '#e0584c', 六分: '#5fb3bf' }
export const LUOSHU = [[4, 9, 2], [3, 5, 7], [8, 1, 6]]

export function hexLines(v: number): boolean[] { return [0, 1, 2, 3, 4, 5].map((i) => ((v >> i) & 1) === 1) }
export function hexUpper(v: number) { return TRIGRAM_NAME[(v >> 3) & 7] }
export function hexLower(v: number) { return TRIGRAM_NAME[v & 7] }

export function Section({ title, children, wide }: { title: string; children: React.ReactNode; wide?: boolean }) {
  return <div className={`lp-sec${wide ? ' wide' : ''}`}><div className="lp-sec-t">{title}</div>{children}</div>
}
export function Stat({ k, v, hi }: { k: string; v: React.ReactNode; hi?: boolean }) {
  return <div className={`stat${hi ? ' hi' : ''}`}><span className="stat-k">{k}</span><span className="stat-v">{v}</span></div>
}
export function Note({ children }: { children: React.ReactNode }) { return <div className="lp-note">{children}</div> }

export function Yao({ yang, changing }: { yang: boolean; changing?: boolean }) {
  return (
    <div className={`yao${changing ? ' moving' : ''}`}>
      {yang ? <span className="bar" /> : (<><span className="bar half" /><span className="bar half" /></>)}
      {changing && <span className="yao-mk">{yang ? '○' : '×'}</span>}
    </div>
  )
}
export function Hexagram({ lines, moving, label, sub, big }: { lines: boolean[]; moving?: number; label?: string; sub?: string; big?: boolean }) {
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
export function GeoFigure({ value, hi, label, name }: { value: number; hi?: boolean; label?: string; name?: string }) {
  // 第 0 位是第一行（火），画在最上面——位序反了会把 Fortuna Major 画成 Minor
  return (
    <div className="geo-fig-wrap">
      <div className={`geofig${hi ? ' hi' : ''}`} title={name}>
        {[0, 1, 2, 3].map((i) => {
          const single = ((value >> i) & 1) === 1
          return <div className="geo-row" key={i}>{single ? <span className="pip" /> : <><span className="pip" /><span className="pip" /></>}</div>
        })}
      </div>
      {label && <div className="geo-lbl">{label}</div>}
      {name && <div className="geo-name">{name}</div>}
    </div>
  )
}

export function Grid9({ render, head }: { render: (gong: number) => React.ReactNode; head?: string }) {
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

export function Row({ k, v, hi }: { k: string; v: string; hi?: boolean }) {
  return <div className={`crow${hi ? ' hi' : ''}`}><span>{k}</span><b>{v}</b></div>
}

export function fmtDeg(d: number) {
  const r = ((d % 30) + 30) % 30
  const m = Math.floor(r)
  const s = Math.round((r - m) * 60)
  return `${m}°${String(s).padStart(2, '0')}'`
}

export function fmtAge(y: number) {
  const sign = y < 0 ? '-' : ''
  const a = Math.abs(y)
  const yr = Math.floor(a)
  const mo = Math.round((a - yr) * 12)
  return `${sign}${yr}y${mo}m`
}
