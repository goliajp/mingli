// 太乙神数：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { Section, Stat } from './shared'

const TAIYI_GRID = [9, 2, 7, 4, 5, 6, 3, 8, 1]
const TAIYI_GUA: Record<number, string> =
  { 1: '乾', 2: '离', 3: '艮', 4: '震', 5: '中', 6: '兑', 7: '坤', 8: '坎', 9: '巽' }

export interface TaiyiMu { position: number; name: string; direction: string; suan: number; da_jiang: number; can_jiang: number }
export interface TaiyiChart {
  year: number; jinian: number; yang_dun: boolean; ju: number; jishen: string
  taiyi: { step: number; palace: number; gua: string; year_in_palace: number; sancai: string }
  wenchang: TaiyiMu; shiji: TaiyiMu
}
export function Taiyi({ c }: { c: TaiyiChart }) {
  // 太乙的九宫配法是乾1 离2 艮3 震4 中5 兑6 坤7 坎8 巽9，与洛书不同
  const mu = (m: TaiyiMu, who: string, cls: string) => (
    <div className={`ty-mu ${cls}`}>
      <div className="ty-mu-h">{who} · {m.name}<i>{m.direction}</i></div>
      <div className="ty-mu-b">算 {m.suan} → 大将 {m.da_jiang} 宫 · 参将 {m.can_jiang} 宫</div>
    </div>
  )
  const at = (g: number) =>
    [g === c.taiyi.palace && '太乙', g === c.wenchang.da_jiang && '主将', g === c.shiji.da_jiang && '客将']
      .filter(Boolean).join(' ')
  return (
    <div className="lp">
      <Section title="太乙行九宫（太乙不入中五，诸将不受此限）">
        {/* 太乙自家的宫数配法（乾1 离2 艮3 震4 中5 兑6 坤7 坎8 巽9）与洛书不同，
            故格子按方位摆而不能套洛书幻方：上为南，左为东。 */}
        <div className="grid9">
          {TAIYI_GRID.map((g) => (
            <div className="g9-pos" key={g}>
              <div className={`g9 ty${g === c.taiyi.palace ? ' taiyi' : ''}${g === 5 ? ' mid' : ''}`}>
                <span className="g9-gong big">{g}</span>
                <span className="g9-stem xs">{TAIYI_GUA[g]}</span>
                {at(g) && <span className="g9-stem sm">{at(g)}</span>}
              </div>
            </div>
          ))}
        </div>
      </Section>
      <Section title="本盘">
        <div className="kv-grid">
          <Stat k="入局" v={`第 ${c.ju} 局`} hi />
          <Stat k="太乙落宫" v={`${c.taiyi.palace}宫 · ${c.taiyi.gua}`} hi />
          <Stat k="三才" v={c.taiyi.sancai} />
          <Stat k="阴阳遁" v={c.yang_dun ? '阳遁' : '阴遁'} />
          <Stat k="计神" v={c.jishen} />
          <Stat k="太乙积年" v={c.jinian.toLocaleString()} />
        </div>
      </Section>
      <Section title="二目与诸将">
        <div className="ty-mus">
          {mu(c.wenchang, '主目 文昌', 'zhu')}
          {mu(c.shiji, '客目 始击', 'ke')}
        </div>
      </Section>
      <div className="lp-note">文昌属主、始击属客，「因主而生客」。算数自目位顺行累加沿途正宫宫数至太乙宫止（间神计一、不累加，终点不计入），再「去十用零」得大将宫，三因得参将宫。🟡「天目」一词在原典里三义并存（种子义即文昌、对举义则指始击、总名义合称二目），故本盘一律只用「文昌 / 始击」。定计目只见《太乙统宗宝鉴》一书，单源未实现。</div>
    </div>
  )
}
