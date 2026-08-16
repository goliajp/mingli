// 问局意图选择器与待承接占位卡。
import type { IntentSpec } from '../types'

// 问局意图选择器（命/运/事/择/合/群/寻/号 8 chip）。
// 唯一在动的是「问什么」；计算层 21 叶不动，意图决定哪些叶被路由 + 输出形态。
export function IntentBar({ intents, current, onChange }: {
  intents: IntentSpec[] | null
  current: string
  onChange: (id: string) => void
}) {
  if (!intents) return null
  return (
    <section className="intent-bar" title="先选你要问什么。当前已实现 命 / 号 两类，其余 6 类会显示该意图的所需输入原子与默认路由叶。">
      <div className="intent-bar-hint">先选你要 <b>问什么</b> ↓</div>
      <div className="intent-bar-chips">
        {intents.map((s) => {
          const on = current === s.id
          const live = s.status === 'Live'
          return (
            <button
              key={s.id}
              className={`intent-chip${on ? ' on' : ''}${live ? ' live' : ' pending'}`}
              onClick={() => onChange(s.id)}
              title={s.note}
            >
              <span className="intent-chip-name">{s.name_zh}</span>
              <span className="intent-chip-shape">{s.output_shape}</span>
              {!live && <i className="intent-chip-dot">🟡</i>}
              {live && <i className="intent-chip-dot">🟢</i>}
            </button>
          )
        })}
      </div>
    </section>
  )
}

// 非 Natal 意图的占位卡 — 显示所需输入原子 + 默认路由叶 + 算力状态。
export function IntentPendingCard({ spec, onBackToNatal }: {
  spec: IntentSpec | undefined
  onBackToNatal: () => void
}) {
  if (!spec) return null
  return (
    <section className="card intent-pending">
      <header className="intent-pending-head">
        <div className="intent-pending-title">
          <span className="intent-pending-name">{spec.name_zh}</span>
          <span className={`intent-pending-status ${spec.status === 'Live' ? 'live' : 'pending'}`}>
            {spec.status === 'Live' ? '🟢 已上线' : '🟡 待承接'}
          </span>
        </div>
        <div className="intent-pending-shape">输出形态：<b>{spec.output_shape}</b></div>
      </header>
      <div className="intent-pending-grid">
        <div className="intent-pending-cell">
          <div className="intent-pending-cell-l">所需输入原子</div>
          <div className="intent-pending-cell-v">
            {spec.atoms.map((a) => <code key={a} className="atom-chip">{a}</code>)}
          </div>
        </div>
        <div className="intent-pending-cell">
          <div className="intent-pending-cell-l">默认路由叶({spec.default_leaves.length})</div>
          <div className="intent-pending-cell-v">
            {spec.default_leaves.map((l) => <code key={l} className="leaf-chip">{l}</code>)}
          </div>
        </div>
      </div>
      <div className="intent-pending-note">{spec.note}</div>
      <div className="intent-pending-foot">
        <span className="intent-pending-foot-meta">本意图的算力多已在叶里，尚未提供承接 UI</span>
        <button className="back-natal" onClick={onBackToNatal}>← 回「命（本命盘）」</button>
      </div>
    </section>
  )
}
