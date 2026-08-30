// Ifá：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { Note, Section, Stat } from './shared'

export interface IfaChart { index: number; left: number; right: number; left_marks: boolean[]; right_marks: boolean[]; right_name: string; left_name: string; name: string; meji: boolean }
export function Ifa({ c }: { c: IfaChart }) {
  // marks[0] 是顶行；右列为长、画在右侧，与占卜时的摆法一致
  const col = (marks: boolean[], lbl: string, name: string) => (
    <div className="odu-col">
      <div className="odu-marks">{[0, 1, 2, 3].map((i) => <div className="odu-row" key={i}>{marks[i] ? <span className="odu-mark" /> : <><span className="odu-mark" /><span className="odu-mark" /></>}</div>)}</div>
      <div className="odu-lbl">{lbl}</div>
      <div className="odu-name">{name}</div>
    </div>
  )
  return (
    <div className="lp">
      <Section title={`Odu · ${c.name}`}>
        <div className="odu big">{col(c.left_marks, '左（幼）', c.left_name)}{col(c.right_marks, '右（长·先出）', c.right_name)}</div>
        <div className="kv-grid">
          <Stat k="Odu 序" v={`${c.index} / 256`} hi />
          <Stat k="复合名" v={c.name} />
          {c.meji && <Stat k="Méjì" v="左右同形" />}
        </div>
        <Note>
          先读右（长）后读左（幼）——次序要紧：左右颠倒会得到另一个同样像样的 odù。
          单画一竖、双画两竖，各四行合成 256 之一。左右同形者为 Méjì，即十六「主 odù」。
          🟡 十六主 odù 的排序无定本，256 复合名三系拼写不同，故本盘按数值索引、不发经文。
        </Note>
      </Section>
    </div>
  )
}
