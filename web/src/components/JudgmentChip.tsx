// 吉凶判读 chip 与其配色。

// 吉凶等级 → 颜色与背景填充（供 JudgmentChip + SVG 分段背景共用）。
export const JUDGMENT_COLOR: Record<string, { fg: string; bg: string }> = {
  大吉: { fg: '#dbe8b8', bg: '#5a7a3a' },
  吉:   { fg: '#cfe1a8', bg: '#7a8f50' },
  平:   { fg: '#b8b6a2', bg: '#5a5a48' },
  凶:   { fg: '#e8b8b0', bg: '#8a4a3a' },
  大凶: { fg: '#f0c8c0', bg: '#a83828' },
}

export const JUDGMENT_FILL: Record<string, string> = {
  大吉: 'rgba(120, 180, 80, 0.18)',
  吉:   'rgba(155, 189, 111, 0.10)',
  平:   'transparent',
  凶:   'rgba(188, 71, 71, 0.10)',
  大凶: 'rgba(188, 71, 71, 0.20)',
}

export function JudgmentChip({ level, score, big = false }: { level: string; score: number; big?: boolean }) {
  const c = JUDGMENT_COLOR[level] ?? { fg: '#aaa', bg: '#444' }
  return (
    <span
      className={`judgment-chip${big ? ' big' : ''}`}
      style={{ background: c.bg, color: c.fg, borderColor: c.fg }}
      title={`净增益 ${score > 0 ? '+' : ''}${score} = 主用神 + 0.5×副用神 − 最高忌神`}
    >
      <b className="judgment-chip-l">{level}</b>
      <small className="judgment-chip-s">{score > 0 ? `+${score}` : score}</small>
    </span>
  )
}
