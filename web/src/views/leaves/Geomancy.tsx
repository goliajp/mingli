// 地占：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { GeoFigure, Note, Section } from './shared'

export interface GeomancyNames { mothers: string[]; daughters: string[]; nieces: string[]; witnesses: string[]; judge: string }
export interface GeomancyChart { mothers: number[]; daughters: number[]; nieces: number[]; witnesses: number[]; judge: number; judge_even: boolean; names: GeomancyNames }
export function Geomancy({ c }: { c: GeomancyChart }) {
  return (
    <div className="lp">
      <Section title="盾牌图（Shield Chart）">
        <div className="shield-pyramid">
          <div className="sp-row">{[...c.daughters].reverse().map((v, i) => <GeoFigure key={'d' + i} value={v} label={`女${4 - i}`} name={c.names.daughters[3 - i]} />)}{[...c.mothers].reverse().map((v, i) => <GeoFigure key={'m' + i} value={v} label={`母${4 - i}`} name={c.names.mothers[3 - i]} hi />)}</div>
          <div className="sp-row">{[...c.nieces].reverse().map((v, i) => <GeoFigure key={'n' + i} value={v} label={`侄${4 - i}`} name={c.names.nieces[3 - i]} />)}</div>
          <div className="sp-row">{[...c.witnesses].reverse().map((v, i) => <GeoFigure key={'w' + i} value={v} label={i === 0 ? '左证' : '右证'} name={c.names.witnesses[1 - i]} />)}</div>
          <div className="sp-row"><GeoFigure value={c.judge} label={`法官 ${c.judge_even ? '✓偶' : '!奇'}`} name={c.names.judge} hi /></div>
        </div>
        <Note>
          自下往上读：四母是唯一随机的一层，四女是四母的转置，四侄由母女两两异或，
          两证再异或得法官。故法官恒为偶图形——这是异或的奇偶守恒定理，不是巧合，
          可当盘面自检：法官若为奇，必是算错。
          🟡 同一图形常有多个并行的阿拉伯名，本盘只出拉丁名，不强选。
        </Note>
      </Section>
    </div>
  )
}
