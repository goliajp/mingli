// 干支与时间的前端小工具：时辰名、干支五行、按时刻改写请求、大运定位。
import type { ChartRequest, DaYun } from '../types'

export const HOUR_NAMES = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥']

// ============ 八字时间拨杆（命静运动）============
// 本命四柱固定；playhead 时刻 t 的八字四柱 = 该刻的流年/流月/流日/流时。
// 拨动 t：本命不动，运层（大运/流年/月/日/时）实时重算。过去 ← 此刻 → 未来。
export const MS_PER_YEAR = 365.2425 * 86400000

export const STEM_WX: Record<string, string> = {
  甲: '木', 乙: '木', 丙: '火', 丁: '火', 戊: '土', 己: '土', 庚: '金', 辛: '金', 壬: '水', 癸: '水',
}

export const BRANCH_WX: Record<string, string> = {
  子: '水', 亥: '水', 寅: '木', 卯: '木', 巳: '火', 午: '火', 申: '金', 酉: '金',
  辰: '土', 戌: '土', 丑: '土', 未: '土',
}

// 干支字符串 → [天干五行， 地支五行]
export function gzWuxing(gz: string): [string, string] {
  return [STEM_WX[gz[0]] ?? '土', BRANCH_WX[gz[1]] ?? '土']
}

// 把一个真实时刻折成排盘请求（保留性别/经纬/时区/姓名，只换年月日时分）。
export function reqAt(d: Date, base: ChartRequest): ChartRequest {
  return { ...base, year: d.getFullYear(), month: d.getMonth() + 1, day: d.getDate(), hour: d.getHours(), minute: d.getMinutes() }
}

// 按年龄挑当前大运步：最后一个 start_age ≤ age 的步（未起运返回 -1）。
export function pickDayun(dy: DaYun, age: number): number {
  let idx = -1
  for (let i = 0; i < dy.pillars.length; i++) if (dy.pillars[i].start_age <= age) idx = i
  return idx
}
