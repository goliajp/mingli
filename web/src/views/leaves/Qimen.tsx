// 奇门遁甲：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { Grid9, Section, Stat } from './shared'

export interface QimenXun { head_ganzhi: string; head_branch: number; head_yi: string; head_yi_stem: number; xunkong: [string, string] }
export interface QimenChart {
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

export interface QimenPatterns {
  star_fu_yin: boolean
  star_fan_yin: boolean
  gate_fu_yin: boolean
  gate_fan_yin: boolean
  stem_fu_yin_palaces: number[]
  full_fu_yin: boolean
  qi_gates: { palace: number; qi: string; gate: string }[]
  qi_de_shi: { palace: number; qi: string; yi: string; xun_head: string; conflicting: string | null }[]
  stem_patterns: { palace: number; name: string; sky: string; earth: string; classical_class: string }[]
}

export interface QimenSpirits {
  start_palace: number
  spirits: string[]
  spirits_alt: string[]
}

export interface QimenGates {
  zhi_shi_gate: string
  zhi_shi_palace: number
  steps: number
  shift: number
  gates: string[]
}

export interface QimenSky {
  shift: number
  stars: string[]
  stems: string[]
  center_stem: string
  center_palace: number
}
const BRANCH_NAMES = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥']
const YUAN_CN: Record<string, string> = { Upper: '上元', Middle: '中元', Lower: '下元' }
export function Qimen({ c }: { c: QimenChart }) {
  const gongName = (g: number) => `${c.palace[g - 1]}${g}`
  const headGongName = gongName(c.xun_yi_palace)
  const zhiFuGongName = gongName(c.zhi_fu_palace)
  const pat = c.patterns
  const noPattern = !pat.full_fu_yin && !pat.star_fu_yin && !pat.star_fan_yin && !pat.gate_fu_yin
    && !pat.gate_fan_yin && pat.stem_fu_yin_palaces.length === 0 && pat.qi_gates.length === 0
    && pat.qi_de_shi.length === 0 && pat.stem_patterns.length === 0
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

      <Section title="时家盘 · 九宫" wide>
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
            <span className="qm-chip qi" key={'qg' + q.palace}>{q.qi} 合 {q.gate} · {gongName(q.palace)}</span>
          ))}
          {pat.qi_de_shi.map((d) => (
            <span className={`qm-chip qi${d.conflicting ? ' caveat' : ''}`} key={'ds' + d.palace}
              title={d.conflicting
                ? `与「${d.conflicting}」是同一个盘面，非偶然共现；《遁甲演义》判为微疵不吉，须本旬直符同临方可用`
                : `${d.qi} 加 ${d.xun_head}${d.yi}`}>
              三奇得使 · {d.qi}加{d.xun_head}{d.yi} · {gongName(d.palace)}{d.conflicting ? ' ⚠' : ''}
            </span>
          ))}
          {pat.stem_patterns.map((sp) => (
            <span className={`qm-chip ${sp.classical_class === '吉' ? 'good' : 'bad'}`} key={'sp' + sp.palace}
              title={`天盘${sp.sky} 加 地盘${sp.earth}（古籍归「${sp.classical_class}格」）`}>
              {sp.name} · {gongName(sp.palace)}
            </span>
          ))}
          {noPattern && <span className="qm-chip none">本盘无已收格局</span>}
        </div>
      </Section>

      <div className="lp-note">四盘齐全：地盘三奇六仪 · 天盘（九星 + 三奇六仪随值符转）· 人盘八门（值使随时辰数）· 神盘八神；星名后小字是该星在月令下的旺相休囚死。🟡 八神第 5 / 6 位两系称谓相左（白虎 / 玄武 与 勾陈 / 朱雀），位序一致故悬停可见另一系；中宫与天禽寄宫取通行的坤 2。格局收多源无争议的几类：伏吟 / 反吟、三奇合吉门、干加干八格（返首 / 跌穴 / 猖狂 / 逃走 / 夭矫 / 投江 / 荧入白 / 白入荧）与三奇得使；吉凶归类照录古籍分卷。⚠ 标记处表示该得使与一个凶格是**同一个盘面**（如乙加甲午即青龙逃走），《遁甲演义》判为微疵不吉。其余 200+ 条各家出入大，未收。</div>
    </div>
  )
}

/* 按方位摆的太乙九宫：上为南（巽离坤）、中（震中兑）、下为北（艮坎乾）。
   位置与洛书幻方相同，只是宫数换成了太乙自家的配法。 */
