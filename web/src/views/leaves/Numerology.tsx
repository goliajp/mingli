// 数字学：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { Note, Section } from './shared'

export interface NameNums { system: string; expression: number; soul_urge: number; personality: number }
export interface NumChart {
  life_path: number
  /** 算出这个数用的是哪一派：`component` 分量约化 / `whole_sum` 全数字直加 */
  life_path_method: string
  /** 另一派算出来的数。两派对同一生日常给出不同的值，这正是本叶把它一并算出的理由 */
  life_path_alt: number
  birthday: number
  pythagorean: NameNums | null
  chaldean: NameNums | null
}

// 两派的名字与算法，与叶的 `schools()` 一致
const METHOD: Record<string, { name: string; how: string }> = {
  component: { name: '分量约化', how: '年月日各自约化后求和，再约化' },
  whole_sum: { name: '全数字直加', how: '年月日全部数字平铺相加，再约化' },
}
export function Numerology({ c }: { c: NumChart }) {
  const master = (n: number) => [11, 22, 33].includes(n)
  const here = METHOD[c.life_path_method]
  const other = Object.entries(METHOD).find(([k]) => k !== c.life_path_method)?.[1]
  return (
    <div className="lp">
      <Section title="核心数（出生日期）">
        <div className="num-big">
          <div className="nb"><b className={master(c.life_path) ? 'master' : ''}>{c.life_path}</b><span>生命灵数{master(c.life_path) ? ' · 主数' : ''}</span></div>
          <div className="nb"><b>{c.birthday}</b><span>生日数</span></div>
        </div>
        {here && (
          <div className="num-method">
            <span className="nm-here">{here.name}</span>
            <small>{here.how}</small>
            {other && (
              <span className="nm-alt">
                另一派「{other.name}」得 <b>{c.life_path_alt}</b>
                {c.life_path_alt === c.life_path ? '（同）' : ''}
              </span>
            )}
          </div>
        )}
        <Note>
          生命灵数两派算法不同，同一个生日常得出不同的数。本盘按所选流派出主值，同时把另一派的值并列出来，不代为选边
        </Note>
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
