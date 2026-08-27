// 后端 HTTP 客户端。所有网络访问集中在这里，组件只调具名端点，不碰 fetch 与路径字面。
import type {
  Analysis, BaziChart, CastResponse, ChartRequest, Election, EventCast, FortuneResponse, IntentsResponse,
  Interpretation, Locative, Mundane, OverlayStrength, Synastry, TeamResult, ZiweiChart,
} from '../types'

/** 两种失败要分得开：服务说「你给的不对」，与服务根本没应答。 */
export type ApiFailure = 'refused' | 'unreachable'

/**
 * 一次请求没成的原因。
 *
 * 后端每一次拒绝都带着一句具体的中文理由（「1990 年 2 月只有 28 天」「hour/minute 越界」），
 * 那是**给用户看的**，重试一百次也不会变。而服务没起来是另一回事，重试才有意义。
 * 从前两者都被渲染成同一句「服务连接失败，请稍后重试」，于是输错日期的人被告知
 * 这是网络问题——理由明明就在同一行上。
 */
export class ApiError extends Error {
  readonly kind: ApiFailure
  constructor(kind: ApiFailure, message: string) {
    super(message)
    this.name = 'ApiError'
    this.kind = kind
  }
}

/** 一句可以直接放进界面的话。重试的提示只加给真正连不上的那种。 */
export function describeFailure(e: unknown): string {
  if (e instanceof ApiError) {
    return e.kind === 'unreachable' ? `${e.message}（服务连接失败，请稍后重试）` : e.message
  }
  return e instanceof Error ? e.message : String(e)
}

async function send(path: string, init?: RequestInit): Promise<Response> {
  try {
    return await fetch(path, init)
  } catch (e) {
    // fetch 只在连不上时抛；HTTP 4xx/5xx 是正常返回，走下面那一支。
    throw new ApiError('unreachable', e instanceof Error ? e.message : String(e))
  }
}

async function post<T>(path: string, body: unknown): Promise<T> {
  const res = await send(path, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  })
  if (!res.ok) {
    const e = await res.json().catch(() => ({ error: res.statusText }))
    throw new ApiError('refused', e.error ?? '请求失败')
  }
  return res.json()
}

async function get<T>(path: string): Promise<T> {
  const res = await send(path)
  if (!res.ok) throw new ApiError('refused', res.statusText)
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

/** 占事：问事此刻 + 取机 → 卜筮诸叶各一盘。 */
export const fetchEvent = (body: unknown) => post<EventCast>('/api/event', body)

/** 占事的「断」。 */
export const fetchEventVerdict = (body: unknown) => post<Interpretation>('/api/event/interpret', body)

/** 择吉：扫时窗，按建除分档排序。 */
export const fetchElection = (body: unknown) => post<Election>('/api/election', body)

/** 择吉的「期 / 序」释义。 */
export const fetchElectionAdvice = (body: unknown) => post<Interpretation>('/api/election/interpret', body)

/** 寻方位：问事此刻起课，抽方位候选。 */
export const fetchLocative = (body: unknown) => post<Locative>('/api/locative', body)

/** 寻方位的「位」释义。 */
export const fetchLocativeAdvice = (body: unknown) => post<Interpretation>('/api/locative/interpret', body)

/** 合盘：两人互供用神。 */
export const fetchSynastry = (body: unknown) => post<Synastry>('/api/synastry', body)

/** 合盘的「配」释义。 */
export const fetchSynastryAdvice = (body: unknown) => post<Interpretation>('/api/synastry/interpret', body)

/** 国运：立国盘 + 太乙行宫时间线 + 年度盘。 */
export const fetchMundane = (body: unknown) => post<Mundane>('/api/mundane', body)

/** 国运的「势」释义。 */
export const fetchMundaneAdvice = (body: unknown) => post<Interpretation>('/api/mundane/interpret', body)
