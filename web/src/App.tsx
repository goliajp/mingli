// 页面组装：表单 → 请求 → 各视图。计算与展示都在别处，这里只连线。
import { useEffect, useMemo, useState } from 'react'
import type { Analysis, BaziChart, CastLeaf, ChartRequest, FortuneResponse, IntentSpec, IntentsResponse, ZiweiChart } from './types'
import { LeafChart } from './leaves'
import { fetchAnalysis, fetchBazi, fetchCast, fetchFortune, fetchInterpretation, fetchZiwei } from './api/client'
import { IntentBar, IntentPendingCard } from './components/IntentBar'
import { ElectionView } from './views/ElectionView'
import { EventView } from './views/EventView'
import { NumField } from './components/NumField'
import { SummaryBar } from './components/SummaryBar'
import { TimeScrubber } from './components/TimeScrubber'
import { CITIES, REGIONS, coordStr } from './data/cities'
import { colorOf, regionOf } from './data/leaf-regions'
import { MS_PER_YEAR, reqAt } from './lib/ganzhi'
import { AnalysisView } from './views/AnalysisView'
import { BaziNatalYun } from './views/BaziNatalYun'
import { FortuneView } from './views/FortuneView'
import { TeamView } from './views/TeamView'
import { WordView } from './views/WordView'

export default function App() {
  const [form, setForm] = useState<ChartRequest>({
    year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8, gender: 'male',
    latitude: 28.23, longitude: 112.94, name: 'LI HAO',
  })
  const [city, setCity] = useState('长沙')
  const [tab, setTab] = useState<string>('bazi') // 当前叶 id
  // 问局意图（命/运/事/择/合/群/寻/号），默认 natal（本命盘）。
  const [intent, setIntent] = useState<string>('natal')
  const [intentsList, setIntentsList] = useState<IntentSpec[] | null>(null)
  useEffect(() => {
    fetch('/api/intents').then((r) => r.json() as Promise<IntentsResponse>)
      .then((r) => setIntentsList(r.intents)).catch(() => {})
  }, [])
  // Fortune：t 时刻运势切片 + 100 年用神供给时间序列。当 intent='fortune' 时按需 fetch。
  const [fortune, setFortune] = useState<FortuneResponse | null>(null)
  const [bazi, setBazi] = useState<BaziChart | null>(null)
  const [ziwei, setZiwei] = useState<ZiweiChart | null>(null)
  const [leaves, setLeaves] = useState<CastLeaf[] | null>(null)
  const [leavesT, setLeavesT] = useState<CastLeaf[] | null>(null) // t 时刻全叶盘（随全局拨杆动）
  const [playAge, setPlayAge] = useState<number | null>(null)      // 全局 playhead 年龄；null=跟随此刻
  const [analysis, setAnalysis] = useState<Analysis | null>(null)
  const [interp, setInterp] = useState<Record<string, { text: string; backend: string; loading?: boolean }>>({})
  const [err, setErr] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [runId, setRunId] = useState(0) // 每次成功排盘 +1，驱动淡入动画

  async function run() {
    setLoading(true)
    setErr(null)
    try {
      const [b, z, all] = await Promise.all([
        fetchBazi(form),
        fetchZiwei(form),
        fetchCast(form),
      ])
      setBazi(b)
      setZiwei(z)
      setLeaves(all.leaves)
      setRunId((n) => n + 1)
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { void run() }, []) // eslint-disable-line react-hooks/exhaustive-deps

  // 全局时间轴：本命(form)固定；playhead t 驱动「t 时刻全叶盘」leavesT —— 所有叶子跟着同一个时间动。
  const birthMs = useMemo(() => new Date(form.year, form.month - 1, form.day, form.hour, form.minute).getTime(), [form])
  const nowAge = Math.max(0, Math.min(100, (Date.now() - birthMs) / MS_PER_YEAR))
  const age = playAge ?? nowAge
  const playDate = useMemo(() => new Date(birthMs + age * MS_PER_YEAR), [birthMs, age])
  useEffect(() => { setPlayAge(null) }, [birthMs]) // 换人重排→拨杆收回此刻
  // Fortune 视图时，playhead 拨动 → 重新 fetch 运势切片 + 时序（后端缓存 timeline 可在 100 年内做但当前每次重算 — natal 不变时 timeline 实际相同）。
  useEffect(() => {
    if (intent !== 'fortune') return
    let alive = true
    const id = setTimeout(() => {
      fetchFortune(form, {
        year: playDate.getFullYear(), month: playDate.getMonth() + 1, day: playDate.getDate(),
        hour: playDate.getHours(), minute: playDate.getMinutes(), tz: form.tz,
      }).then((r) => { if (alive) { setFortune(r); setErr(null) } })
        .catch((e) => { if (alive) setErr(e instanceof Error ? e.message : String(e)) })
    }, 120)
    return () => { alive = false; clearTimeout(id) }
  }, [intent, form, age, playDate]) // eslint-disable-line react-hooks/exhaustive-deps
  useEffect(() => {
    let alive = true
    const id = setTimeout(() => {
      fetchCast(reqAt(playDate, form))
        .then((r) => { if (alive) setLeavesT(r.leaves) })
        .catch(() => {})
    }, 90) // 拖动防抖
    return () => { alive = false; clearTimeout(id) }
  }, [age, form]) // eslint-disable-line react-hooks/exhaustive-deps

  // 跨叶相关分析：固定网格、结果确定，首次打开懒加载（后端缓存）。
  useEffect(() => {
    if (tab === 'analysis' && !analysis) {
      fetchAnalysis().then(setAnalysis).catch(() => {})
    }
  }, [tab, analysis])

  const set = (k: keyof ChartRequest) => (e: { target: { value: string } }) => {
    const v = e.target.value
    setForm((f) => ({ ...f, [k]: k === 'gender' || k === 'name' ? v : Number(v) }))
  }

  async function genInterp(leafId: string) {
    setInterp((s) => ({ ...s, [leafId]: { text: '', backend: '', loading: true } }))
    try {
      const r = await fetchInterpretation(form, leafId)
      setInterp((s) => ({ ...s, [leafId]: { text: r.text, backend: r.backend } }))
    } catch {
      setInterp((s) => ({ ...s, [leafId]: { text: '释义生成失败，请稍后再试', backend: 'error' } }))
    }
  }

  return (
    <div className="wrap">
      <header className="head">
        <div>
          <div className="brand">命理 <b>MINGLI</b></div>
          <div className="tagline">跨文化术数排盘 · 21 种体系 · 同一生辰一键起盘</div>
        </div>
        <div className="stack">Rust 引擎 · React 界面</div>
      </header>

      <IntentBar intents={intentsList} current={intent} onChange={setIntent} />

      <section className="form">
        <NumField label="年" v={form.year} on={set('year')} w={70} />
        <NumField label="月" v={form.month} on={set('month')} />
        <NumField label="日" v={form.day} on={set('day')} />
        <NumField label="时" v={form.hour} on={set('hour')} />
        <NumField label="分" v={form.minute} on={set('minute')} />
        <label className="field">
          <span>性别</span>
          <select value={form.gender} onChange={set('gender')}>
            <option value="male">男</option>
            <option value="female">女</option>
          </select>
        </label>
        <label className="field" title="出生地：占星上升点/中天(Asc/MC)取决于出生时刻+出生地经纬度；时区亦为出生地时区">
          <span>出生地</span>
          <select value={city} style={{ width: 92 }} onChange={(e) => {
            const name = e.target.value
            const c = CITIES[name]
            setCity(name)
            setForm((f) => ({ ...f, latitude: c.lat, longitude: c.lon, tz: c.tz }))
          }}>
            {REGIONS.map((r) => (
              <optgroup label={r} key={r}>
                {Object.keys(CITIES).filter((n) => CITIES[n].region === r).map((n) => <option key={n} value={n}>{n}</option>)}
              </optgroup>
            ))}
          </select>
        </label>
        <div className="field coords">
          <span>经纬·时区</span>
          <div className="coords-v">{coordStr(form.latitude, form.longitude, form.tz)}</div>
        </div>
        <label className="field">
          <span>姓名</span>
          <input type="text" value={form.name ?? ''} onChange={set('name')} style={{ width: 130 }} />
        </label>
        <label className="field" title="开启 = 按出生地经度 + 均时差 EoT 校正钟表时（真太阳时）。跨时辰边界时，时柱会变。">
          <span>真太阳时</span>
          <select value={form.true_solar_time ? 'on' : 'off'} style={{ width: 64 }} onChange={(e) => setForm((f) => ({ ...f, true_solar_time: e.target.value === 'on' }))}>
            <option value="off">钟表</option>
            <option value="on">真太阳</option>
          </select>
        </label>
        <label className="field" title="主体类型：换释义层象义映射，计算层不变。人=默认，公司/产品/事件 = 物有八字的择日逆运算。">
          <span>主体</span>
          <select value={form.subject ?? 'person'} style={{ width: 78 }} onChange={(e) => setForm((f) => ({ ...f, subject: e.target.value as 'person' | 'company' | 'product' | 'event' }))}>
            <option value="person">人</option>
            <option value="company">公司</option>
            <option value="product">物/产品</option>
            <option value="event">事件</option>
          </select>
        </label>
        <button className="go" onClick={() => void run()} disabled={loading}>
          {loading ? '排盘中…' : '排 盘'}
        </button>
      </section>

      {err && <div className="err">⚠ {err}（服务连接失败，请稍后重试）</div>}

      {intent === 'fortune' && bazi && (
        <div className="result" key={`fortune-${runId}`}>
          <SummaryBar bazi={bazi} ziwei={ziwei} form={form} />
          <TimeScrubber
            age={age} nowAge={nowAge} playDate={playDate} dayun={bazi.dayun ?? null}
            onChange={setPlayAge} onNow={() => setPlayAge(null)} onBirth={() => setPlayAge(0)}
          />
          <FortuneView fortune={fortune} age={age} onBackToNatal={() => setIntent('natal')} />
        </div>
      )}

      {intent === 'event' && <EventView />}
      {intent === 'election' && <ElectionView />}

      {intent !== 'natal' && intent !== 'fortune' && intent !== 'event' && intent !== 'election' && intentsList && (
        <IntentPendingCard spec={intentsList.find((s) => s.id === intent)} onBackToNatal={() => setIntent('natal')} />
      )}

      {intent === 'natal' && bazi && (
        <div className="result" key={runId}>
          <SummaryBar bazi={bazi} ziwei={ziwei} form={form} />

          <TimeScrubber
            age={age} nowAge={nowAge} playDate={playDate} dayun={bazi.dayun ?? null}
            onChange={setPlayAge} onNow={() => setPlayAge(null)} onBirth={() => setPlayAge(0)}
          />

          {leaves && (
            <div className="tabs leaf-tabs">
              {leaves.map((l) => (
                <button
                  key={l.id}
                  className={`leaf-tab${tab === l.id ? ' on' : ''}`}
                  style={{ ['--fam' as string]: colorOf(l.id) }}
                  onClick={() => setTab(l.id)}
                >
                  <i className="tab-dot" />{l.name}
                </button>
              ))}
              <button className={`leaf-tab xleaf${tab === 'analysis' ? ' on' : ''}`} onClick={() => setTab('analysis')}>⊞ 相关性</button>
              <button className={`leaf-tab xleaf${tab === 'word' ? ' on' : ''}`} onClick={() => setTab('word')}>字 文字术数</button>
              <button className={`leaf-tab xleaf${tab === 'team' ? ' on' : ''}`} onClick={() => setTab('team')}>合 合盘 / 团队</button>
            </div>
          )}

          {tab === 'analysis' && <AnalysisView analysis={analysis} />}
          {tab === 'word' && <WordView />}
          {tab === 'team' && <TeamView />}

          {tab === 'bazi' && (
            <section className="card leaf-solo" style={{ borderTopColor: colorOf('bazi') }}>
              <div className="solo-head">
                <span className="solo-name">八字 · 命静运动</span>
                <span className="solo-fam" style={{ color: colorOf('bazi') }}>中华</span>
              </div>
              <BaziNatalYun natal={bazi} yun={(leavesT ?? leaves)?.find((l) => l.id === 'bazi')?.chart as BaziChart | undefined} age={age} form={form} />
            </section>
          )}

          {leaves && (() => {
            // 只在 tab 确实指向某片叶时渲染。此前对未知 tab 兜底到 leaves[0]，
            // 于是「合盘 / 团队」页脚下会多冒出一整块八字。
            const l = leaves.find((x) => x.id === tab)
            if (!l || l.id === 'bazi') return null
            const color = colorOf(l.id)
            const lT = (leavesT ?? leaves).find((x) => x.id === l.id) ?? l // 该叶在 t 时刻的盘
            return (
              <section className="card leaf-solo" style={{ borderTopColor: color }}>
                <div className="solo-head">
                  <span className="solo-name">{l.name}</span>
                  <span className="solo-fam" style={{ color }}>{regionOf(l.id)}</span>
                  {l.schools.length > 0 && (
                    <label className="solo-school" title="切换流派后自动重排该输入下所有叶子（含本叶）">
                      <span className="solo-school-l">流派</span>
                      <select
                        value={form.schools?.[l.id] ?? l.effective_school}
                        onChange={(e) => {
                          const v = e.target.value
                          setForm((f) => ({ ...f, schools: { ...(f.schools ?? {}), [l.id]: v } }))
                          // 异步重排，无需 await（用户感知到加载态）。
                          setTimeout(() => { void run() }, 0)
                        }}
                      >
                        {l.schools.map((s) => (
                          <option key={s.id} value={s.id} title={s.note}>{s.name}</option>
                        ))}
                      </select>
                    </label>
                  )}
                </div>
                <LeafChart leaf={lT} />
                <div className="int-panel">
                  <div className="lp-sec-t">🔮 文字释义 · 由语言模型生成，仅供参考</div>
                  {(() => {
                    const cur = interp[l.id]
                    if (!cur) return <button className="int-btn" onClick={() => void genInterp(l.id)}>生成释义</button>
                    if (cur.loading) return <div className="lp-note">释义生成中…（约 10 秒）</div>
                    return (
                      <div className="int-text">
                        {cur.text.split('\n').filter((s) => s.trim()).map((para, i) => <p key={i}>{para}</p>)}
                        <div className="int-by">{cur.backend} 生成 · 仅供研究与娱乐</div>
                      </div>
                    )
                  })()}
                </div>
              </section>
            )
          })()}
        </div>
      )}

      <footer className="foot">
        引擎依权威排盘工具与公认历法值校验（节气·朔·干支·闰月·历元）。结果仅供研究与娱乐。
      </footer>
    </div>
  )
}
