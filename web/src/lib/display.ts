// 展示层共用的五行配色与写法。
//
// 不只叶页在用：旺衰面板、用神面板、大运格、概要条也都要同一套颜色，
// 所以它住在 lib 而不是某一片叶旁边。
import type { Lunar } from '../types'

export const WUXING = ['木', '火', '土', '金', '水'] as const
export const WUXING_COLOR: Record<string, string> = {
  木: '#5fb06a', 火: '#e0584c', 土: '#d4a843', 金: '#cfd6dc', 水: '#5b9bd6',
}

// 五行生克关系（a 视角对 b）
export function wxRelation(a: string, b: string): string {
  if (a === b) return '比和'
  const gen: Record<string, string> = { 木: '火', 火: '土', 土: '金', 金: '水', 水: '木' }
  const ctl: Record<string, string> = { 木: '土', 土: '水', 水: '火', 火: '金', 金: '木' }
  if (gen[a] === b) return `${a}生${b}`
  if (gen[b] === a) return `${b}生${a}`
  if (ctl[a] === b) return `${a}克${b}`
  return `${b}克${a}`
}

export function lunarStr(l: Lunar) {
  const m = ['', '正', '二', '三', '四', '五', '六', '七', '八', '九', '十', '冬', '腊']
  const d = ['', '初一', '初二', '初三', '初四', '初五', '初六', '初七', '初八', '初九', '初十',
    '十一', '十二', '十三', '十四', '十五', '十六', '十七', '十八', '十九', '二十',
    '廿一', '廿二', '廿三', '廿四', '廿五', '廿六', '廿七', '廿八', '廿九', '三十']
  return `${l.year}年${l.leap ? '闰' : ''}${m[l.month]}月${d[l.day]}`
}
