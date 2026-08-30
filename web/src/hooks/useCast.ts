// 「拿盘」这件事：本命三份一起排、跨叶相关懒加载、单叶释义按需生成。
//
// 这些都是副作用，跟版面无关。放在 App 里时，「组装页面」的函数里有一半是 fetch 与
// 它们的时序，读的人得先把两件事在脑子里分开。分出来之后 App 只剩连线。

import { useCallback, useEffect, useState } from 'react'
import { fetchAnalysis, fetchBazi, fetchCast, fetchInterpretation, fetchZiwei, describeFailure } from '../api/client'
import type { Analysis, BaziChart, CastLeaf, ChartRequest, ZiweiChart } from '../types'

/** 一片叶的释义：正文、谁说的、是不是还在等。 */
export interface Interp {
  text: string
  backend: string
  loading?: boolean
}

/** 本命三份盘 + 排盘状态。 */
export function useNatalCast(form: ChartRequest) {
  const [bazi, setBazi] = useState<BaziChart | null>(null)
  const [ziwei, setZiwei] = useState<ZiweiChart | null>(null)
  const [leaves, setLeaves] = useState<CastLeaf[] | null>(null)
  const [err, setErr] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [runId, setRunId] = useState(0) // 每次成功排盘 +1，驱动淡入动画

  const run = useCallback(async () => {
    setLoading(true)
    setErr(null)
    try {
      const [b, z, all] = await Promise.all([fetchBazi(form), fetchZiwei(form), fetchCast(form)])
      setBazi(b)
      setZiwei(z)
      setLeaves(all.leaves)
      setRunId((n) => n + 1)
    } catch (e) {
      // 用 describeFailure 而不是直接取 message：重试的提示只该加给真连不上的那种，
      // 后端拒绝时的理由要原样呈给用户（见 api/client.ts 的 ApiError）。
      setErr(describeFailure(e))
    } finally {
      setLoading(false)
    }
  }, [form])

  useEffect(() => { void run() }, []) // eslint-disable-line react-hooks/exhaustive-deps

  return { bazi, ziwei, leaves, err, setErr, loading, runId, run }
}

/** 跨叶相关分析：网格固定、结果确定，首次要看时才拉。 */
export function useAnalysis(active: boolean) {
  const [analysis, setAnalysis] = useState<Analysis | null>(null)
  useEffect(() => {
    if (active && !analysis) {
      fetchAnalysis().then(setAnalysis).catch(() => {})
    }
  }, [active, analysis])
  return analysis
}

/** 单叶释义：按 id 存，各自独立地转圈。 */
export function useInterpretations(form: ChartRequest) {
  const [interp, setInterp] = useState<Record<string, Interp>>({})
  const generate = useCallback(async (leafId: string) => {
    setInterp((s) => ({ ...s, [leafId]: { text: '', backend: '', loading: true } }))
    try {
      const r = await fetchInterpretation(form, leafId)
      setInterp((s) => ({ ...s, [leafId]: { text: r.text, backend: r.backend } }))
    } catch {
      setInterp((s) => ({ ...s, [leafId]: { text: '释义生成失败，请稍后再试', backend: 'error' } }))
    }
  }, [form])
  return { interp, generate }
}
