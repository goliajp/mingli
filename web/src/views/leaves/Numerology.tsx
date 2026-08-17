// 数字学：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { Note, Section } from './shared'

export interface NameNums { system: string; expression: number; soul_urge: number; personality: number }
export interface NumChart { life_path: number; birthday: number; pythagorean: NameNums | null; chaldean: NameNums | null }
export function Numerology({ c }: { c: NumChart }) {
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
