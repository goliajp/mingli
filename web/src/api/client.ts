// 后端 HTTP 客户端。所有网络访问集中在这里，组件只调具名端点，不碰 fetch 与路径字面。
import type {
  Analysis, BaziChart, CastResponse, ChartRequest, FortuneResponse, IntentsResponse,
  Interpretation, OverlayStrength, TeamResult, ZiweiChart,
} from '../types'

async function post<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(path, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  })
  if (!res.ok) {
    const e = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error(e.error ?? '请求失败')
  }
  return res.json()
}

async function get<T>(path: string): Promise<T> {
  const res = await fetch(path)
  if (!res.ok) throw new Error(res.statusText)
  return res.json()
}

/** 四柱精盘。 */
export const fetchBazi = (req: ChartRequest) => post<BaziChart>('/api/bazi', req)

/** 紫微精盘。 */
export const fetchZiwei = (req: ChartRequest) => post<ZiweiChart>('/api/ziwei', req)

/** 全叶并行排盘。 */
export const fetchCast = (req: ChartRequest) => post<CastResponse>('/api/cast', req)

/** 岁运叠加旺衰：本命 + 外力干支（大运柱 / 流年柱）。 */
export const fetchOverlayStrength = (req: ChartRequest, extras: string[]) =>
  post<OverlayStrength>('/api/bazi/overlay-strength', { ...req, extras })

/** 运势：t 时刻切片 + 百年供给时序。 */
export const fetchFortune = (
  natal: ChartRequest,
  tTarget: { year: number; month: number; day: number; hour: number; minute: number; tz: number },
  timelineMaxAge = 100,
) => post<FortuneResponse>('/api/fortune', { natal, t_target: tTarget, timeline_max_age: timelineMaxAge })

/** 团队合盘。 */
export const fetchTeam = (members: unknown[]) => post<TeamResult>('/api/team', { members })

/** 团队合盘的释义。 */
export const fetchTeamInterpretation = (members: unknown[]) =>
  post<Interpretation>('/api/team/interpret', { members })

/** 字词术数。 */
export const fetchWord = (body: unknown) => post<Record<string, unknown>>('/api/word', body)

/** 单叶释义。 */
export const fetchInterpretation = (req: ChartRequest, leaf: string) =>
  post<Interpretation>('/api/interpret', { ...req, leaf })

/** 跨叶相关性。 */
export const fetchAnalysis = () => get<Analysis>('/api/analysis')

/** 问局意图清单。 */
export const fetchIntents = () => get<IntentsResponse>('/api/intents')
