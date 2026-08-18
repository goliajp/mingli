// 西洋占星：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { ASPECT_COLOR, ASPECT_SYM, PLANET_GLYPH, SIGN_GLYPH, SIGN_NAMES, Section, Stat } from './shared'

export interface AstroChart {
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
export function Astrology({ c }: { c: AstroChart }) {
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
