// 塔罗：一整页，按其本系统最专业的排盘样式呈现（不是卡片缩略）。
// 字段与 /api/cast 的 serde 输出一一对应；缺名表处（🟡）忠实留白，不臆造。
import { Section } from './shared'

export interface TarotChart { cards: { index: number; reversed: boolean; name: string; name_zh: string; glyph: string }[]; deck_size: number; reversible: boolean; deck_id?: string }
const DECK_NAMES: Record<string, string> = {
  tarot_full: '塔罗 78',
  tarot_major: '塔罗大阿卡纳 22',
  lenormand: 'Petit Lenormand 36',
  elder_futhark: 'Elder Futhark 卢恩 24',
  younger_futhark: 'Younger Futhark 卢恩 16',
}
export function Tarot({ c }: { c: TarotChart }) {
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
