// 印度占星：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import type { JyotishChart } from '../../types'
import { Section, Stat, fmtAge, fmtDeg } from './shared'

const AYANAMSA_LABEL: Record<string, string> = {
  lahiri: 'Lahiri （印度官方）',
  krishnamurti: 'Krishnamurti (KP)',
  raman: 'Raman',
  fagan_bradley: 'Fagan-Bradley',
}
// 宫名跟本叶其余各表一致，用梵文罗马化（对应 jyotish::RASI_NAMES）
const RASI_NAMES = [
  'Mesha', 'Vrishabha', 'Mithuna', 'Karka', 'Simha', 'Kanya',
  'Tula', 'Vrishchika', 'Dhanu', 'Makara', 'Kumbha', 'Meena',
]

// 十二分盘：除数 → 所主。本命 D-1 与九分盘 D-9 另在上方两表。
const VARGA_SUBJECT: [string, string][] = [
  ['d3', '兄弟'], ['d4', '田宅'], ['d7', '子嗣'], ['d10', '事功'],
  ['d12', '父母'], ['d16', '车乘'], ['d20', '修行'], ['d24', '学问'],
  ['d27', '体质'], ['d40', '母系'], ['d45', '父系'], ['d60', '总述'],
]

export function Jyotish({ c }: { c: JyotishChart }) {
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
      <Section title="十六分盘（Ṣoḍaśavarga）· 取十二" wide>
        <div className="lp-note">
          看某一事就看对应分盘上的落宫，不是看本命盘。🟡 D-2 与 D-30 原典未定落宫，本盘不出。
        </div>
        <div className="jy-varga-wrap">
        <table className="jy-graha-table jy-varga">
          <thead>
            <tr>
              <th>曜</th>
              {VARGA_SUBJECT.map(([id, subj]) => (
                <th key={id}>{id.toUpperCase()}<br /><small>{subj}</small></th>
              ))}
            </tr>
          </thead>
          <tbody>
            {c.grahas.map((g) => (
              <tr key={`v-${g.graha}`}>
                <td><b>{g.name}</b></td>
                {VARGA_SUBJECT.map(([id]) => (
                  <td key={id}>{RASI_NAMES[g.vargas?.[id] ?? 0]}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
        </div>
      </Section>
    </div>
  )
}
