// 全局时间轴：本命固定，playhead 在动，所有叶订阅同一个时刻。
//
// 「此刻是几岁」「拨到某岁是哪一天」是纯换算；「拨到那里要重排什么」是副作用。
// 两者都跟版面无关，所以一起搬出 App。拖动时的防抖也在这里——它属于「怎么取数」，
// 不属于「怎么摆」。

import { useEffect, useMemo, useState } from 'react'
import { fetchCast, fetchFortune } from '../api/client'
import { MS_PER_YEAR, reqAt } from '../lib/ganzhi'
import type { CastLeaf, ChartRequest, FortuneResponse } from '../types'

/** 时间轴上限：一生按 100 年算。 */
const MAX_AGE = 100

/** 本命 + playhead → 年龄、日期，以及换人时把拨杆收回此刻。 */
export function useTimeline(form: ChartRequest) {
  const [playAge, setPlayAge] = useState<number | null>(null) // null = 跟随此刻
  const birthMs = useMemo(
    () => new Date(form.year, form.month - 1, form.day, form.hour, form.minute).getTime(),
    [form],
  )
  const nowAge = Math.max(0, Math.min(MAX_AGE, (Date.now() - birthMs) / MS_PER_YEAR))
  const age = playAge ?? nowAge
  const playDate = useMemo(() => new Date(birthMs + age * MS_PER_YEAR), [birthMs, age])
  useEffect(() => { setPlayAge(null) }, [birthMs]) // 换人重排 → 拨杆收回此刻
  return { age, nowAge, playAge, setPlayAge, playDate }
}

/** playhead 时刻的全叶盘。拖动会连发，故防抖。 */
export function useLeavesAt(form: ChartRequest, age: number, playDate: Date) {
  const [leavesT, setLeavesT] = useState<CastLeaf[] | null>(null)
  useEffect(() => {
    let alive = true
    const id = setTimeout(() => {
      fetchCast(reqAt(playDate, form))
        .then((r) => { if (alive) setLeavesT(r.leaves) })
        .catch(() => {})
    }, 90)
    return () => { alive = false; clearTimeout(id) }
  }, [age, form]) // eslint-disable-line react-hooks/exhaustive-deps
  return leavesT
}

/** 运势切片 + 一生供给时序。只在看「运」的时候拉。 */
export function useFortune(
  form: ChartRequest,
  age: number,
  playDate: Date,
  active: boolean,
  onError: (msg: string | null) => void,
) {
  const [fortune, setFortune] = useState<FortuneResponse | null>(null)
  useEffect(() => {
    if (!active) return undefined
    let alive = true
    const id = setTimeout(() => {
      fetchFortune(form, {
        year: playDate.getFullYear(), month: playDate.getMonth() + 1, day: playDate.getDate(),
        hour: playDate.getHours(), minute: playDate.getMinutes(), tz: form.tz,
      }).then((r) => { if (alive) { setFortune(r); onError(null) } })
        .catch((e) => { if (alive) onError(e instanceof Error ? e.message : String(e)) })
    }, 120)
    return () => { alive = false; clearTimeout(id) }
  }, [active, form, age, playDate]) // eslint-disable-line react-hooks/exhaustive-deps
  return fortune
}

/** 意图清单：进页面拉一次。 */
export function useIntents() {
  const [intentsList, setIntentsList] = useState<import('../types').IntentSpec[] | null>(null)
  useEffect(() => {
    fetch('/api/intents').then((r) => r.json() as Promise<import('../types').IntentsResponse>)
      .then((r) => setIntentsList(r.intents)).catch(() => {})
  }, [])
  return intentsList
}
