import { Fragment, useEffect, useMemo, useState } from 'react'
import type { Analysis, BaziChart, CastLeaf, CastResponse, ChartRequest, DaYun, FortuneResponse, IntentSpec, IntentsResponse, Interpretation, OverlayStrength, Pattern, Pillar, Strength, TeamResult, YongShen, ZiweiChart } from './types'
import { LeafChart, WUXING_COLOR, lunarStr, wxRelation } from './leaves'

const HOUR_NAMES = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥']

// 出生地城市 → 纬度/经度/时区（地理坐标，公开常识值；选城市自动换算 Asc/MC 用）
const REGIONS = ['华北', '华东', '华中', '华南', '西南', '西北', '东北', '港澳台', '海外'] as const
type Region = typeof REGIONS[number]
const CITIES: Record<string, { lat: number; lon: number; tz: number; region: Region }> = {
  北京: { lat: 39.90, lon: 116.41, tz: 8, region: '华北' }, 天津: { lat: 39.13, lon: 117.20, tz: 8, region: '华北' },
  石家庄: { lat: 38.04, lon: 114.51, tz: 8, region: '华北' }, 太原: { lat: 37.87, lon: 112.55, tz: 8, region: '华北' },
  呼和浩特: { lat: 40.84, lon: 111.75, tz: 8, region: '华北' },
  上海: { lat: 31.23, lon: 121.47, tz: 8, region: '华东' }, 南京: { lat: 32.06, lon: 118.80, tz: 8, region: '华东' },
  杭州: { lat: 30.27, lon: 120.16, tz: 8, region: '华东' }, 苏州: { lat: 31.30, lon: 120.58, tz: 8, region: '华东' },
  无锡: { lat: 31.49, lon: 120.31, tz: 8, region: '华东' }, 宁波: { lat: 29.87, lon: 121.55, tz: 8, region: '华东' },
  温州: { lat: 28.00, lon: 120.70, tz: 8, region: '华东' }, 合肥: { lat: 31.82, lon: 117.23, tz: 8, region: '华东' },
  济南: { lat: 36.65, lon: 117.00, tz: 8, region: '华东' }, 青岛: { lat: 36.07, lon: 120.38, tz: 8, region: '华东' },
  福州: { lat: 26.07, lon: 119.30, tz: 8, region: '华东' }, 厦门: { lat: 24.48, lon: 118.09, tz: 8, region: '华东' },
  南昌: { lat: 28.68, lon: 115.86, tz: 8, region: '华东' },
  武汉: { lat: 30.59, lon: 114.31, tz: 8, region: '华中' }, 长沙: { lat: 28.23, lon: 112.94, tz: 8, region: '华中' },
  郑州: { lat: 34.75, lon: 113.62, tz: 8, region: '华中' },
  广州: { lat: 23.13, lon: 113.26, tz: 8, region: '华南' }, 深圳: { lat: 22.54, lon: 114.06, tz: 8, region: '华南' },
  东莞: { lat: 23.02, lon: 113.75, tz: 8, region: '华南' }, 佛山: { lat: 23.02, lon: 113.12, tz: 8, region: '华南' },
  南宁: { lat: 22.82, lon: 108.32, tz: 8, region: '华南' }, 海口: { lat: 20.04, lon: 110.32, tz: 8, region: '华南' },
  重庆: { lat: 29.56, lon: 106.55, tz: 8, region: '西南' }, 成都: { lat: 30.66, lon: 104.07, tz: 8, region: '西南' },
  贵阳: { lat: 26.65, lon: 106.63, tz: 8, region: '西南' }, 昆明: { lat: 25.04, lon: 102.71, tz: 8, region: '西南' },
  拉萨: { lat: 29.65, lon: 91.14, tz: 8, region: '西南' },
  西安: { lat: 34.34, lon: 108.94, tz: 8, region: '西北' }, 兰州: { lat: 36.06, lon: 103.83, tz: 8, region: '西北' },
  西宁: { lat: 36.62, lon: 101.78, tz: 8, region: '西北' }, 银川: { lat: 38.49, lon: 106.23, tz: 8, region: '西北' },
  乌鲁木齐: { lat: 43.83, lon: 87.62, tz: 8, region: '西北' },
  沈阳: { lat: 41.80, lon: 123.43, tz: 8, region: '东北' }, 大连: { lat: 38.91, lon: 121.61, tz: 8, region: '东北' },
  长春: { lat: 43.82, lon: 125.32, tz: 8, region: '东北' }, 哈尔滨: { lat: 45.80, lon: 126.53, tz: 8, region: '东北' },
  香港: { lat: 22.32, lon: 114.17, tz: 8, region: '港澳台' }, 澳门: { lat: 22.20, lon: 113.54, tz: 8, region: '港澳台' },
  台北: { lat: 25.03, lon: 121.57, tz: 8, region: '港澳台' }, 高雄: { lat: 22.63, lon: 120.30, tz: 8, region: '港澳台' },
  东京: { lat: 35.68, lon: 139.65, tz: 9, region: '海外' }, 大阪: { lat: 34.69, lon: 135.50, tz: 9, region: '海外' },
  首尔: { lat: 37.57, lon: 126.98, tz: 9, region: '海外' }, 新加坡: { lat: 1.35, lon: 103.82, tz: 8, region: '海外' },
  曼谷: { lat: 13.76, lon: 100.50, tz: 7, region: '海外' }, 吉隆坡: { lat: 3.14, lon: 101.69, tz: 8, region: '海外' },
  纽约: { lat: 40.71, lon: -74.01, tz: -5, region: '海外' }, 洛杉矶: { lat: 34.05, lon: -118.24, tz: -8, region: '海外' },
  旧金山: { lat: 37.77, lon: -122.42, tz: -8, region: '海外' }, 温哥华: { lat: 49.28, lon: -123.12, tz: -8, region: '海外' },
  伦敦: { lat: 51.51, lon: -0.13, tz: 0, region: '海外' }, 巴黎: { lat: 48.86, lon: 2.35, tz: 1, region: '海外' },
  柏林: { lat: 52.52, lon: 13.40, tz: 1, region: '海外' }, 莫斯科: { lat: 55.76, lon: 37.62, tz: 3, region: '海外' },
  迪拜: { lat: 25.20, lon: 55.27, tz: 4, region: '海外' }, 悉尼: { lat: -33.87, lon: 151.21, tz: 10, region: '海外' },
}

// 术数按文化／地域分组（用户视角）
const LEAF_REGION: Record<string, string> = {
  bazi: '中华', ziwei: '中华', yijing: '中华', meihua: '中华', xiaoliuren: '中华',
  zeri: '中华', liuren: '中华', qimen: '中华', taiyi: '中华', qizhengsiyu: '中华',
  astrology: '西洋', tarot: '西洋', numerology: '西洋',
  jyotish: '南亚',
  geomancy: '中东',
  ifa: '非洲', sikidy: '非洲',
  maya: '美洲',
  tibetan: '喜马拉雅·东南亚', pawukon: '喜马拉雅·东南亚', mahabote: '喜马拉雅·东南亚',
}
const REGION_COLOR: Record<string, string> = {
  中华: '#c9a24a', 西洋: '#5fb3bf', 中东: '#a98cff',
  非洲: '#5fb06a', 美洲: '#e0584c', '喜马拉雅·东南亚': '#d98c5f',
  南亚: '#d24a8c',
}
const regionOf = (id: string) => LEAF_REGION[id] ?? '其他'
const colorOf = (id: string) => REGION_COLOR[regionOf(id)] ?? '#888'

function coordStr(lat?: number, lon?: number, tz?: number): string {
  const la = lat ?? 0, lo = lon ?? 0, t = tz ?? 8
  return `${Math.abs(la).toFixed(2)}°${la >= 0 ? 'N' : 'S'} ${Math.abs(lo).toFixed(2)}°${lo >= 0 ? 'E' : 'W'} · UTC${t >= 0 ? '+' : ''}${t}`
}

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
        post<BaziChart>('/api/bazi', form),
        post<ZiweiChart>('/api/ziwei', form),
        post<CastResponse>('/api/cast', form),
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
      post<FortuneResponse>('/api/fortune', {
        natal: form,
        t_target: { year: playDate.getFullYear(), month: playDate.getMonth() + 1, day: playDate.getDate(), hour: playDate.getHours(), minute: playDate.getMinutes(), tz: form.tz },
        timeline_max_age: 100,
      }).then((r) => { if (alive) { setFortune(r); setErr(null) } })
        .catch((e) => { if (alive) setErr(e instanceof Error ? e.message : String(e)) })
    }, 120)
    return () => { alive = false; clearTimeout(id) }
  }, [intent, form, age, playDate]) // eslint-disable-line react-hooks/exhaustive-deps
  useEffect(() => {
    let alive = true
    const id = setTimeout(() => {
      post<CastResponse>('/api/cast', reqAt(playDate, form))
        .then((r) => { if (alive) setLeavesT(r.leaves) })
        .catch(() => {})
    }, 90) // 拖动防抖
    return () => { alive = false; clearTimeout(id) }
  }, [age, form]) // eslint-disable-line react-hooks/exhaustive-deps

  // 跨叶相关分析：固定网格、结果确定，首次打开懒加载（后端缓存）。
  useEffect(() => {
    if (tab === 'analysis' && !analysis) {
      fetch('/api/analysis').then((r) => r.json()).then(setAnalysis).catch(() => {})
    }
  }, [tab, analysis])

  const set = (k: keyof ChartRequest) => (e: { target: { value: string } }) => {
    const v = e.target.value
    setForm((f) => ({ ...f, [k]: k === 'gender' || k === 'name' ? v : Number(v) }))
  }

  async function genInterp(leafId: string) {
    setInterp((s) => ({ ...s, [leafId]: { text: '', backend: '', loading: true } }))
    try {
      const r = await post<Interpretation>('/api/interpret', { ...form, leaf: leafId })
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

      {intent !== 'natal' && intent !== 'fortune' && intentsList && (
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

          {tab !== 'analysis' && tab !== 'word' && tab !== 'bazi' && leaves && (() => {
            const l = leaves.find((x) => x.id === tab) ?? leaves[0]
            if (!l) return null
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

// ============ 八字时间拨杆（命静运动）============
// 本命四柱固定；playhead 时刻 t 的八字四柱 = 该刻的流年/流月/流日/流时。
// 拨动 t：本命不动，运层（大运/流年/月/日/时）实时重算。过去 ← 此刻 → 未来。
const MS_PER_YEAR = 365.2425 * 86400000
const STEM_WX: Record<string, string> = {
  甲: '木', 乙: '木', 丙: '火', 丁: '火', 戊: '土', 己: '土', 庚: '金', 辛: '金', 壬: '水', 癸: '水',
}
const BRANCH_WX: Record<string, string> = {
  子: '水', 亥: '水', 寅: '木', 卯: '木', 巳: '火', 午: '火', 申: '金', 酉: '金',
  辰: '土', 戌: '土', 丑: '土', 未: '土',
}
// 干支字符串 → [天干五行， 地支五行]
function gzWuxing(gz: string): [string, string] {
  return [STEM_WX[gz[0]] ?? '土', BRANCH_WX[gz[1]] ?? '土']
}
// 把一个真实时刻折成排盘请求（保留性别/经纬/时区/姓名，只换年月日时分）。
function reqAt(d: Date, base: ChartRequest): ChartRequest {
  return { ...base, year: d.getFullYear(), month: d.getMonth() + 1, day: d.getDate(), hour: d.getHours(), minute: d.getMinutes() }
}
// 按年龄挑当前大运步：最后一个 start_age ≤ age 的步（未起运返回 -1）。
function pickDayun(dy: DaYun, age: number): number {
  let idx = -1
  for (let i = 0; i < dy.pillars.length; i++) if (dy.pillars[i].start_age <= age) idx = i
  return idx
}

function YunCell({ label, gz, sub, dayWx, hi }: {
  label: string; gz: string; sub?: string; dayWx?: string; hi?: boolean
}) {
  const [sWx, bWx] = gzWuxing(gz)
  // 运对日主的五行关系（生我=运助、我克=我用…），仅用五行，不臆造十神。
  const rel = dayWx ? wxRelation(dayWx, sWx) : null
  return (
    <div className={`yun-cell${hi ? ' hi' : ''}`}>
      <div className="yun-lbl">{label}</div>
      <div className="yun-gz">
        <b style={{ color: WUXING_COLOR[sWx] }}>{gz[0]}</b>
        <b style={{ color: WUXING_COLOR[bWx] }}>{gz[1]}</b>
      </div>
      {sub && <div className="yun-sub">{sub}</div>}
      {rel && <div className="yun-rel">{rel}</div>}
    </div>
  )
}

// 全局时间拨杆（常驻 tabs 上方）：一根 playhead，所有叶子订阅它。
function TimeScrubber({ age, nowAge, playDate, dayun, onChange, onNow, onBirth }: {
  age: number; nowAge: number; playDate: Date; dayun: DaYun | null
  onChange: (a: number) => void; onNow: () => void; onBirth: () => void
}) {
  const MAX = 100
  const pct = (a: number) => `${(a / MAX) * 100}%`
  const diff = age - nowAge
  const when = Math.abs(diff) < 0.05 ? '此刻' : diff < 0 ? '过去' : '未来'
  return (
    <div className="timebar">
      <div className="timebar-t">时间轴 · 过去 ← 此刻 → 未来　<em>拨动 = 全部系统切到该时刻</em></div>
      <div className="tl-track">
        <div className="tl-future" style={{ left: pct(nowAge) }} />
        {dayun?.pillars.filter((d) => d.start_age <= MAX).map((d) => (
          <i className="tl-tick" key={d.start_age} style={{ left: pct(d.start_age) }}><span>{d.start_age}</span></i>
        ))}
        <i className="tl-now" style={{ left: pct(nowAge) }}><span>今</span></i>
        <i className="tl-play" style={{ left: pct(age) }} />
      </div>
      <input className="tl-range" type="range" min={0} max={MAX} step={0.1} value={age}
        onChange={(e) => onChange(Number(e.target.value))} />
      <div className="tl-read">
        <span className={`tl-when w-${when === '过去' ? 'past' : when === '未来' ? 'future' : 'now'}`}>{when}</span>
        <span className="tl-date">
          {playDate.getFullYear()}-{String(playDate.getMonth() + 1).padStart(2, '0')}-{String(playDate.getDate()).padStart(2, '0')}
          {' '}{String(playDate.getHours()).padStart(2, '0')}:{String(playDate.getMinutes()).padStart(2, '0')}
        </span>
        <span className="tl-age">{Math.floor(age)} 岁</span>
        {Math.abs(diff) >= 0.05 && <span className="tl-rel">（今{diff < 0 ? '前' : '后'} {Math.abs(Math.round(diff))} 年）</span>}
        <span className="tl-jump">
          <button onClick={onBirth}>出生</button>
          <button onClick={onNow}>回到此刻</button>
        </span>
      </div>
    </div>
  )
}

// 四柱宫位象义按主体切（落到视觉）；仅 bazi 叶受影响。
// 顺序 = [年， 月， 日， 时]。
const SUBJECT_PILLAR_ROLES: Record<'person' | 'company' | 'product' | 'event', [string, string, string, string]> = {
  person: ['祖根', '父母 / 青年', '自身 / 配偶', '子女 / 晚年'],
  company: ['创立根基 / 行业', '团队 / 管理', '主体 / 核心业务', '前景 / 产出'],
  product: ['上市背景', '定位 / 品类', '本体 / 核心特性', '反馈 / 生命周期'],
  event: ['背景 / 大环境', '诱发 / 参与方', '核心走向', '结果 / 后续'],
}

// 八字叶：本命四柱固定 + 运层（= 全局 t 时刻盘的四柱）。
function BaziNatalYun({ natal, yun, age, form }: { natal: BaziChart; yun?: BaziChart; age: number; form: ChartRequest }) {
  const subject = (form.subject ?? 'person') as 'person' | 'company' | 'product' | 'event'
  const pillarRoles = SUBJECT_PILLAR_ROLES[subject]
  const dayWx = natal.day_master_wuxing
  const natalCols: [string, Pillar][] = [['年', natal.year], ['月', natal.month], ['日', natal.day], ['时', natal.hour]]
  const dyIdx = natal.dayun ? pickDayun(natal.dayun, age) : -1
  const dyActive = natal.dayun && dyIdx >= 0 ? natal.dayun.pillars[dyIdx] : null

  // 岁运叠加旺衰：本命基础上拼大运柱 + 流年柱，问后端要 t 时刻的实际旺衰。
  const liuNianGz = yun?.year.ganzhi
  const dayunGz = dyActive?.ganzhi
  const extras = [dayunGz, liuNianGz].filter((s): s is string => !!s)
  const extrasKey = extras.join(',')
  const [overlay, setOverlay] = useState<OverlayStrength | null>(null)
  useEffect(() => {
    if (extras.length === 0) { setOverlay(null); return }
    let alive = true
    fetch('/api/bazi/overlay-strength', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ ...form, extras }),
    })
      .then((r) => r.ok ? r.json() : null)
      .then((j) => { if (alive && j) setOverlay(j as OverlayStrength) })
      .catch(() => { if (alive) setOverlay(null) })
    return () => { alive = false }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [form, extrasKey])
  return (
    <div className="lp">
      <div className="lp-sec ming">
        <div className="lp-sec-t"><span className="my-badge ming">命</span>本命四柱 · 出生即定，拨动时不变</div>
        <div className="pillars natal-fixed">
          {natalCols.map(([name, p], idx) => (
            <div className={`pillar${name === '日' ? ' is-day' : ''}`} key={name}>
              <div className="pname">{name}柱<i className="p12">{p.day_twelve}</i>{subject !== 'person' && <i className="p-role">{pillarRoles[idx]}</i>}</div>
              {subject === 'person' && <div className="p-role-person">{pillarRoles[idx]}</div>}
              <div className="char gan" style={{ color: WUXING_COLOR[p.stem_wuxing] }}>{p.stem}</div>
              <div className="char zhi" style={{ color: WUXING_COLOR[p.branch_wuxing] }}>{p.branch}</div>
              <div className="wx"><i style={{ background: WUXING_COLOR[p.stem_wuxing] }} />{p.stem_wuxing}<i style={{ background: WUXING_COLOR[p.branch_wuxing] }} />{p.branch_wuxing}</div>
              <div className="pnayin">纳音 <b style={{ color: WUXING_COLOR[p.nayin] }}>{p.nayin}</b></div>
              <div className="phidden">
                <div className="ph-t">藏干</div>
                {p.hidden.map((h, i) => (
                  <div className="hs" key={i}>
                    <b style={{ color: WUXING_COLOR[STEM_WX[h.stem]] }}>{h.stem}</b>
                    <i>{h.ten_god}</i>
                  </div>
                ))}
              </div>
              {p.shensha && p.shensha.length > 0 && (
                <div className="pshensha">
                  {p.shensha.map((s) => <span className="ss-chip" key={s}>{s}</span>)}
                </div>
              )}
            </div>
          ))}
        </div>
        <div className="kv-grid" style={{ marginTop: 10 }}>
          <div className="stat hi"><span className="stat-k">日主</span><span className="stat-v">{natal.day_master}（{dayWx}）</span></div>
          {natal.xunkong && <div className="stat"><span className="stat-k">空亡 · 旬空</span><span className="stat-v">{natal.xunkong.join(' ')}</span></div>}
          {natal.three_houses && <>
            <div className="stat"><span className="stat-k">命宫</span><span className="stat-v">{natal.three_houses.ming_gong}</span></div>
            <div className="stat"><span className="stat-k">身宫</span><span className="stat-v">{natal.three_houses.shen_gong}</span></div>
            <div className="stat"><span className="stat-k">胎元</span><span className="stat-v">{natal.three_houses.tai_yuan}</span></div>
          </>}
        </div>
        {natal.pattern && <PatternPanel p={natal.pattern} />}
        {natal.strength && <StrengthPanel s={natal.strength} dayWx={dayWx} />}
        {natal.yongshen && <YongShenPanel y={natal.yongshen} yunWuxing={overlay?.yun.wuxing} />}
      </div>
      <div
        className="lp-sec yun pulse"
        key={`yun-${yun?.year.ganzhi ?? '-'}-${yun?.month.ganzhi ?? '-'}-${yun?.day.ganzhi ?? '-'}-${yun?.hour.ganzhi ?? '-'}-${dyIdx}`}
      >
        <div className="lp-sec-t"><span className="my-badge yun">运</span>运层 · 随顶部时间拨杆而动</div>
        <div className="yun-row">
          {natal.dayun ? (
            dyActive
              ? <YunCell label={`大运 · 第${dyIdx + 1}步`} gz={dyActive.ganzhi} sub={`${dyActive.start_age} 岁起`} dayWx={dayWx} hi />
              : <YunCell label="大运" gz="未起" sub={`${natal.dayun.start_age_years} 岁起运`} />
          ) : (
            <div className="yun-cell"><div className="yun-lbl">大运</div><div className="yun-gz na">—</div><div className="yun-sub">需性别</div></div>
          )}
          {yun ? (
            <>
              <YunCell label="流年" gz={yun.year.ganzhi} dayWx={dayWx} />
              <YunCell label="流月" gz={yun.month.ganzhi} dayWx={dayWx} />
              <YunCell label="流日" gz={yun.day.ganzhi} dayWx={dayWx} />
              <YunCell label="流时" gz={yun.hour.ganzhi} dayWx={dayWx} />
            </>
          ) : (
            <div className="yun-cell"><div className="yun-lbl">流年/月/日/时</div><div className="yun-gz na">…</div></div>
          )}
        </div>
        <div className="lp-note" style={{ paddingTop: 6 }}>
          运层四柱 = 顶部拨杆所指时刻的八字四柱（流年=年柱、流月=月柱、流日=日柱、流时=时柱）；标签为该柱天干对<b>日主</b>的五行生克。
          🟡 大运换步按 start_age（周岁近似）。
        </div>
        {overlay && <OverlayStrengthPanel o={overlay} dayWx={dayWx} extras={{ dayun: dayunGz, liunian: liuNianGz }} />}
      </div>
    </div>
  )
}

// 岁运叠加旺衰：大运柱 + 流年柱 拼到本命之上 → t 时刻的实际旺衰。
// 拨杆动时这条会随之脉动，与本命旺衰条形成「命底·运面」对照。
function OverlayStrengthPanel({ o, dayWx, extras }: {
  o: OverlayStrength; dayWx: string; extras: { dayun?: string; liunian?: string }
}) {
  const dayColor = WUXING_COLOR[dayWx] ?? '#888'
  const d = o.delta_score
  const dir = d > 0 ? '增强' : d < 0 ? '减弱' : '持平'
  const dirCls = d > 0 ? 'up' : d < 0 ? 'down' : 'flat'
  return (
    <div className="overlay-box">
      <div className="strength-hdr">
        <span className="strength-t">岁运叠加旺衰<span className="yun-tag">运</span><span className="neutral-hint">本命 + 大运 + 流年</span></span>
        <span className="strength-level" style={{ color: dayColor }}>{o.yun.level}</span>
        <span className="strength-score">{o.yun.score}<i>/100</i></span>
        <span className={`delta-pill ${dirCls}`}>{d >= 0 ? '+' : ''}{d} · 较本命{dir}</span>
      </div>
      <div className="overlay-extras">
        {extras.dayun && <span className="overlay-src"><b>大运</b>{extras.dayun}</span>}
        {extras.liunian && <span className="overlay-src"><b>流年</b>{extras.liunian}</span>}
      </div>
      <div className="strength-bar">
        <div className="strength-bar-fill" style={{ width: `${o.yun.score}%`, background: dayColor, opacity: .82 }} />
        <i className="overlay-natal-mark" style={{ left: `${o.ming.score}%` }} title={`本命 ${o.ming.score}`} />
      </div>
      <div className="overlay-trio">
        <OverlayTrio label="得令" v={o.yun.got_ling} base={o.ming.got_ling} c={dayColor} />
        <OverlayTrio label="得地" v={o.yun.got_di} base={o.ming.got_di} c={dayColor} />
        <OverlayTrio label="得势" v={o.yun.got_shi} base={o.ming.got_shi} c={dayColor} />
      </div>
      <div className="lp-note" style={{ paddingTop: 8, fontSize: 14 }}>
        <b>「增强 / 减弱」≠ 「转好 / 转坏」</b>：这只是 t 时刻日主能量相对本命的偏移。
        若本命身弱，流年「增强」可能正合「弱者宜扶」是吉；若本命身强已无制，流年再「增强」反而是「强而无制」的偏差 —— 同样的 ↑，吉凶方向相反。<br />
        本命底图固定，大运柱 + 流年柱「外力」拼入得地/得势/五行分布，得令永远取本命月支（月令出生即定）；旺衰条上的小标记是本命基准位，条本身的填充长度是叠加后值。
        🟡 岁运折扣权重无统一标准；真正的吉凶要看用神 / 喜忌配合。
      </div>
    </div>
  )
}

function OverlayTrio({ label, v, base, c }: { label: string; v: number; base: number; c: string }) {
  const d = v - base
  return (
    <div className="overlay-trio-cell">
      <div className="overlay-trio-l">{label}</div>
      <div className="overlay-trio-v" style={{ color: c }}>{v}<i>/30</i></div>
      <div className={`overlay-trio-d ${d > 0 ? 'up' : d < 0 ? 'down' : 'flat'}`}>
        {d >= 0 ? '+' : ''}{d}<i>本命 {base}</i>
      </div>
    </div>
  )
}

// 用神 / 喜忌：把旺衰+格局合起来，告诉命主「该补什么 / 该忌什么」。
// 若有运层叠加，旁边附「t 时刻拿到了多少喜用」（本命用神为锚，看运给的够不够）。
function YongShenPanel({ y, yunWuxing }: { y: YongShen; yunWuxing?: { wood: number; fire: number; earth: number; metal: number; water: number } }) {
  const wxPct = (n: string): number => {
    if (!yunWuxing) return 0
    return ({ '木': yunWuxing.wood, '火': yunWuxing.fire, '土': yunWuxing.earth, '金': yunWuxing.metal, '水': yunWuxing.water } as Record<string, number>)[n] ?? 0
  }
  return (
    <div className="yong-box">
      <div className="yong-hdr">
        <span className="strength-t">用神 · 喜忌<span className="ming-tag">本命</span><span className="neutral-hint">该补 / 该忌 · 是事实题，不是断吉凶</span></span>
        <span className="yong-method">{y.method}</span>
      </div>
      <div className="yong-grid">
        <YongCell kind="primary" label="主用神" wx={y.primary_wuxing} role={y.primary_role} pct={yunWuxing ? wxPct(y.primary_wuxing) : undefined} />
        {y.secondary_wuxing && y.secondary_role && (
          <YongCell kind="secondary" label="副用神" wx={y.secondary_wuxing} role={y.secondary_role} pct={yunWuxing ? wxPct(y.secondary_wuxing) : undefined} />
        )}
        {y.avoid_wuxing.length > 0 && (
          <div className="yong-avoid">
            <div className="yong-avoid-l">忌神</div>
            <div className="yong-avoid-v">
              {y.avoid_wuxing.map((n) => (
                <span key={n} className="yong-avoid-chip" style={{ color: WUXING_COLOR[n], borderColor: WUXING_COLOR[n] }}>
                  <b>{n}</b>
                  {yunWuxing && <i>{wxPct(n)}%</i>}
                </span>
              ))}
            </div>
          </div>
        )}
      </div>
      <div className="yong-reason">{y.reasoning}</div>
      {yunWuxing && (
        <div className="lp-note" style={{ paddingTop: 6, fontSize: 14 }}>
          ↑ 上面百分比 = <b>t 时刻五行分布</b>（本命+大运+流年叠加后）中该五行的实际占比；
          <b>主用神 % 越高 = 当前时段「拿到的喜用」越多</b>；忌神 % 越高 = 当前时段「拿到的克泄」越多。
          这一栏 = 「拨杆 → 是吉是凶」的最后一跳：**用神被供给** = 吉，**忌神被加强** = 凶。
        </div>
      )}
      <div className="lp-note" style={{ paddingTop: 6, fontSize: 14 }}>
        <b>「用神」≠ 必然吉、「忌神」≠ 必然凶。</b>这只是命局结构推出的「该补 / 该忌」的事实结论。
        🟡 取用神有扶抑/调候/通关/病药/格局五法，各家先后顺序不同；从格/化格反扶抑（扶其太过）本算法不覆盖。
      </div>
    </div>
  )
}

function YongCell({ kind, label, wx, role, pct }: { kind: 'primary' | 'secondary'; label: string; wx: string; role: string; pct?: number }) {
  const c = WUXING_COLOR[wx] ?? 'inherit'
  return (
    <div className={`yong-cell ${kind}`}>
      <div className="yong-cell-l">{label}</div>
      <div className="yong-cell-wx" style={{ color: c }}>{wx}</div>
      <div className="yong-cell-role">{role}</div>
      {pct !== undefined && (
        <div className="yong-cell-pct">
          <div className="yong-cell-bar"><i style={{ width: `${Math.min(pct, 60) * 100 / 60}%`, background: c }} /></div>
          <span className="yong-cell-pct-v" style={{ color: c }}>{pct}<i>%</i></span>
        </div>
      )}
    </div>
  )
}

// 命格：月令藏干透干 → 八正格 / 建禄月刃 / 暗格。是命主结构性属性，不随时间动。
function PatternPanel({ p }: { p: Pattern }) {
  const isAmbiguous = !p.revealed && !p.is_lu_ren
  return (
    <div className="pattern-box">
      <div className="pattern-hdr">
        <span className="pattern-t">命格<span className="ming-tag">本命</span><span className="neutral-hint">命主结构类型</span></span>
        <span className="pattern-name">{p.name}</span>
      </div>
      <div className="pattern-body">
        <div className="pattern-row">
          <span className="pattern-k">月令藏干</span>
          <span className="pattern-v">
            <b style={{ color: WUXING_COLOR[STEM_WX[p.qi_stem]] ?? 'inherit' }}>{p.qi_stem}</b>
            <i>{p.qi_kind}</i>
            <em>→ 日主十神 {p.ten_god}</em>
          </span>
        </div>
        <div className="pattern-row">
          <span className="pattern-k">取格依据</span>
          <span className="pattern-v">
            {p.source}
            {p.revealed_in && <i className="pattern-tag-on">透干在 {p.revealed_in}</i>}
            {isAmbiguous && <i className="pattern-tag-off">藏而不透</i>}
            {p.is_lu_ren && <i className="pattern-tag-special">禄刃 · 不入八正格</i>}
          </span>
        </div>
      </div>
      <div className="lp-note" style={{ paddingTop: 6, fontSize: 14 }}>
        命格是命主的<b>结构性类型</b>（像血型/星座），本身无优劣。<b>「正官格」不等于「当官」、「七杀格」不等于「凶险」。</b>
        格局好坏 = 格局 × 用神配合；从格/化格/专旺格涉强弱+特殊条件，本算法 🟡 不机械化，留 INT。
      </div>
    </div>
  )
}

// 旺衰量化：得令/得地/得势三栏 0-30 → 综合 0-100 强度条 + 五行力量分布。
function StrengthPanel({ s, dayWx }: { s: Strength; dayWx: string }) {
  const dayColor = WUXING_COLOR[dayWx] ?? '#888'
  // 五行分布：按命理传统排序 木火土金水
  const wxRows: [string, number][] = [
    ['木', s.wuxing.wood], ['火', s.wuxing.fire], ['土', s.wuxing.earth],
    ['金', s.wuxing.metal], ['水', s.wuxing.water],
  ]
  return (
    <div className="strength-box">
      <div className="strength-hdr">
        <span className="strength-t">日主旺衰<span className="ming-tag">本命</span><span className="neutral-hint">能量量级</span></span>
        <span className="strength-level" style={{ color: dayColor }}>{s.level}</span>
        <span className="strength-score">{s.score}<i>/100</i></span>
      </div>
      <div className="strength-bar">
        <div className="strength-bar-fill" style={{ width: `${s.score}%`, background: dayColor }} />
        <i className="strength-bar-mark" style={{ left: '40%' }} />
        <i className="strength-bar-mark" style={{ left: '60%' }} />
      </div>
      <div className="strength-cols">
        <StrengthCol label="得令" sub="月支长生 + 月支藏干" v={s.got_ling} c={dayColor} />
        <StrengthCol label="得地" sub="年/日/时支通根" v={s.got_di} c={dayColor} />
        <StrengthCol label="得势" sub="干头比劫印" v={s.got_shi} c={dayColor} />
      </div>
      <div className="strength-t" style={{ marginTop: 14 }}>五行力量分布</div>
      <div className="wx-rows">
        {wxRows.map(([n, v]) => (
          <div className="wx-row" key={n}>
            <span className="wx-row-n" style={{ color: WUXING_COLOR[n] }}>{n}</span>
            <div className="wx-row-bar"><i style={{ width: `${v}%`, background: WUXING_COLOR[n] }} /></div>
            <span className="wx-row-v">{v}%</span>
          </div>
        ))}
      </div>
      <div className="lp-note" style={{ paddingTop: 8, fontSize: 14 }}>
        <b>「强 / 弱」≠ 「好 / 坏」</b>：强弱是日主能量量级（像身高体重），本身不构成褒贬。
        命格好坏 = 强弱 × 用神配不配 —— 身强宜抑(食伤/财/官杀泄克)、身弱宜扶(比劫/印帮身);
        「强而有制 / 弱而有助」均属佳格，「强而无制 / 弱而无援」才偏差。**→ 真正的吉凶要看用神 / 喜忌配合。**
        <br />
        🟡 权重表无统一标准（各家月令权重 30%-60% 不一）；本算法显式声明：得令/得地/得势各 0-30，合 0-90 → 0-100；
        「同党」=比劫（同五行）+印星（生我）。量化为辅助判断，非定论。
      </div>
    </div>
  )
}

function StrengthCol({ label, sub, v, c }: { label: string; sub: string; v: number; c: string }) {
  return (
    <div className="strength-col">
      <div className="strength-col-v" style={{ color: c }}>{v}<i>/30</i></div>
      <div className="strength-col-bar"><i style={{ height: `${(v / 30) * 100}%`, background: c }} /></div>
      <div className="strength-col-l">{label}</div>
      <div className="strength-col-s">{sub}</div>
    </div>
  )
}

// 问局意图选择器（命/运/事/择/合/群/寻/号 8 chip）。
// 唯一在动的是「问什么」；计算层 21 叶不动，意图决定哪些叶被路由 + 输出形态。
function IntentBar({ intents, current, onChange }: {
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
function IntentPendingCard({ spec, onBackToNatal }: {
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

// 吉凶等级 → 颜色与背景填充（供 JudgmentChip + SVG 分段背景共用）。
const JUDGMENT_COLOR: Record<string, { fg: string; bg: string }> = {
  大吉: { fg: '#dbe8b8', bg: '#5a7a3a' },
  吉:   { fg: '#cfe1a8', bg: '#7a8f50' },
  平:   { fg: '#b8b6a2', bg: '#5a5a48' },
  凶:   { fg: '#e8b8b0', bg: '#8a4a3a' },
  大凶: { fg: '#f0c8c0', bg: '#a83828' },
}
const JUDGMENT_FILL: Record<string, string> = {
  大吉: 'rgba(120, 180, 80, 0.18)',
  吉:   'rgba(155, 189, 111, 0.10)',
  平:   'transparent',
  凶:   'rgba(188, 71, 71, 0.10)',
  大凶: 'rgba(188, 71, 71, 0.20)',
}

function JudgmentChip({ level, score, big = false }: { level: string; score: number; big?: boolean }) {
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

// Fortune 视图：t 时刻运势切片 + 100 年用神供给曲线。
// 用神喜忌的完整闭环：本命静态喜忌（出生即定） → t 时刻供给度（拨杆动跟动）→ 100 年时序（看一生）。
// 主用神供给度高 = 该时段流年大运得用神之利 = **吉**；忌神供给度高 = **凶**。算法给出 5 等级判读（大吉/吉/平/凶/大凶）。
function FortuneView({ fortune, age, onBackToNatal }: {
  fortune: FortuneResponse | null
  age: number
  onBackToNatal: () => void
}) {
  if (!fortune) return <section className="card fortune"><div className="fortune-load">运势切片加载中…</div></section>
  const { at, timeline } = fortune
  const ys = at.natal.yongshen
  const primaryColor = WUXING_COLOR[ys.primary_wuxing] ?? '#888'
  const secondaryColor = ys.secondary_wuxing ? (WUXING_COLOR[ys.secondary_wuxing] ?? '#888') : '#888'
  const avoidColor = '#bc4747'
  const deltaSign = at.delta_score > 0 ? '↑' : at.delta_score < 0 ? '↓' : '→'
  const deltaCls = at.delta_score > 0 ? 'pos' : at.delta_score < 0 ? 'neg' : 'zero'
  const dayWx = at.natal.day_master_wuxing

  // 本命四柱（固定底图）
  const natalCols: [string, Pillar][] = [['年', at.natal.year], ['月', at.natal.month], ['日', at.natal.day], ['时', at.natal.hour]]
  // 运层 5 柱（随 t 动：大运 / 流年 / 流月 / 流日 / 流时）
  const yunLayers: { label: string; gz: string; sub?: string }[] = []
  if (at.dayun_ganzhi) yunLayers.push({ label: '大运', gz: at.dayun_ganzhi, sub: at.dayun_step != null ? `第 ${at.dayun_step + 1}/10 步` : undefined })
  yunLayers.push({ label: '流年', gz: at.flow_year_ganzhi })
  yunLayers.push({ label: '流月', gz: at.t_chart.month.ganzhi })
  yunLayers.push({ label: '流日', gz: at.t_chart.day.ganzhi })
  yunLayers.push({ label: '流时', gz: at.t_chart.hour.ganzhi })

  // 大运 10 步条（横向时间轴）
  const dayunPillars = at.natal.dayun?.pillars ?? []

  // 重大节点：大吉(score≥+15) + 大凶(≤-15)；最多 8 个每类
  const milestones = {
    daji: timeline.filter((p) => p.judgment.level === '大吉').slice(0, 8),
    daxiong: timeline.filter((p) => p.judgment.level === '大凶').slice(0, 8),
  }

  // SVG 折线图参数
  const W = 880, H = 220, padL = 48, padR = 16, padT = 18, padB = 32
  const innerW = W - padL - padR, innerH = H - padT - padB
  const maxAge = timeline.length - 1
  const xOf = (a: number) => padL + (innerW * a) / Math.max(1, maxAge)
  const yOf = (pct: number) => padT + innerH * (1 - Math.min(100, Math.max(0, pct)) / 100)
  const pathOf = (key: 'primary_supply_pct' | 'secondary_supply_pct' | 'avoid_supply_pct') => {
    const pts: string[] = []
    for (const p of timeline) {
      const v = p[key]
      if (v == null) continue
      pts.push(`${pts.length === 0 ? 'M' : 'L'}${xOf(p.age).toFixed(1)},${yOf(v as number).toFixed(1)}`)
    }
    return pts.join(' ')
  }
  // 大运分段竖线（每步起 age）
  const dayunMarks = dayunPillars.map((p) => p.start_age)
  // playhead = 当前 age
  const playX = xOf(Math.min(maxAge, Math.max(0, Math.floor(age))))

  return (
    <section className="card fortune">
      <header className="fortune-head">
        <div className="fortune-title">
          <span className="fortune-name">运 · 八字行运</span>
          <span className="fortune-sub">本命四柱 + t 时刻流年/月/日/时 + 大运段 + 用神供给曲线</span>
        </div>
        <JudgmentChip level={at.judgment.level} score={at.judgment.score} big />
        <button className="back-natal" onClick={onBackToNatal}>← 回「命」</button>
      </header>

      <div className="fortune-judgment-summary">
        <span className="fjs-l">当前 {at.age_years.toFixed(2)} 岁 · 流年 <b style={{color:'#bb6'}}>{at.flow_year_ganzhi}</b></span>
        <span>{at.judgment.summary}</span>
      </div>

      {/* —— 命 / 运 双行四柱对比 —— */}
      <div className="fortune-pillars">
        <div className="fortune-pillars-row ming">
          <div className="fortune-pillars-label">
            <span className="ming-tag">命</span>
            <span className="fortune-pillars-l">本命四柱 · 出生即定</span>
          </div>
          <div className="fortune-pillars-cells">
            {natalCols.map(([lab, p]) => (
              <div className="natal-cell" key={lab}>
                <div className="natal-cell-l">{lab}柱</div>
                <div className="natal-cell-gz">
                  <b style={{ color: WUXING_COLOR[p.stem_wuxing] }}>{p.ganzhi[0]}</b>
                  <b style={{ color: WUXING_COLOR[p.branch_wuxing] }}>{p.ganzhi[1]}</b>
                </div>
                <div className="natal-cell-sub">{p.ten_god} · {p.day_twelve}</div>
                {p.shensha.length > 0 && (
                  <div className="natal-cell-shensha">{p.shensha.join(' · ')}</div>
                )}
              </div>
            ))}
          </div>
        </div>

        <div className="fortune-pillars-row yun">
          <div className="fortune-pillars-label">
            <span className="yun-tag">运</span>
            <span className="fortune-pillars-l">t 时刻 5 柱 · 拨杆动 = 此处跟动</span>
          </div>
          <div className="fortune-pillars-cells">
            {yunLayers.map((y) => (
              <YunCell key={y.label} label={y.label} gz={y.gz} sub={y.sub} dayWx={dayWx} hi={y.label === '大运' || y.label === '流年'} />
            ))}
          </div>
        </div>
      </div>

      {/* —— 大运 10 步条 —— */}
      {dayunPillars.length > 0 && (
        <div className="fortune-dayun-row">
          <div className="fortune-dayun-l">大运十步({at.natal.dayun?.forward ? '顺行' : '逆行'} · 起运 {at.natal.dayun?.start_age_years} 岁)</div>
          <div className="fortune-dayun-strip">
            {dayunPillars.map((p, i) => {
              const isActive = at.dayun_step === i
              const wx = gzWuxing(p.ganzhi)
              return (
                <div key={i} className={`dyx-step${isActive ? ' on' : ''}`}>
                  <div className="dyx-step-age">{p.start_age}+</div>
                  <div className="dyx-step-gz">
                    <b style={{ color: WUXING_COLOR[wx[0]] }}>{p.ganzhi[0]}</b>
                    <b style={{ color: WUXING_COLOR[wx[1]] }}>{p.ganzhi[1]}</b>
                  </div>
                  <div className="dyx-step-i">{i + 1}/10</div>
                </div>
              )
            })}
          </div>
        </div>
      )}

      {/* —— 当前 stat + supplies —— */}
      <div className="fortune-now">
        <div className="fortune-now-row">
          <span className="fortune-now-l">运层旺衰</span>
          <b className="fortune-now-v">{at.yun_strength.score}</b>
          <small className="fortune-now-lvl">({at.yun_strength.level})</small>
          <span className="fortune-now-sep">vs 本命</span>
          <b className="fortune-now-v">{at.ming_strength.score}</b>
          <small className="fortune-now-lvl">({at.ming_strength.level})</small>
          <span className={`fortune-delta ${deltaCls}`}>{deltaSign} {Math.abs(at.delta_score)}</span>
        </div>
        <div className="fortune-now-row supplies">
          <div className="supply-stat" style={{ borderColor: primaryColor }}>
            <div className="supply-stat-l">主用神 · {ys.primary_role}</div>
            <div className="supply-stat-v" style={{ color: primaryColor }}>{ys.primary_wuxing}</div>
            <div className="supply-stat-pct">{at.primary_supply_pct}%</div>
          </div>
          {ys.secondary_wuxing && (
            <div className="supply-stat" style={{ borderColor: secondaryColor }}>
              <div className="supply-stat-l">副用神 · {ys.secondary_role}</div>
              <div className="supply-stat-v" style={{ color: secondaryColor }}>{ys.secondary_wuxing}</div>
              <div className="supply-stat-pct">{at.secondary_supply_pct ?? '—'}%</div>
            </div>
          )}
          {ys.avoid_wuxing.map((w, i) => (
            <div key={w} className="supply-stat avoid" style={{ borderColor: avoidColor }}>
              <div className="supply-stat-l">忌神 · 越低越好</div>
              <div className="supply-stat-v" style={{ color: avoidColor }}>{w}</div>
              <div className="supply-stat-pct">{at.avoid_supply_pcts[i] ?? '—'}%</div>
            </div>
          ))}
        </div>
        <div className="fortune-note">{ys.reasoning} <i>· 主用神供给度高 = 拿到喜用 = <b style={{color:'#9bbd6f'}}>吉</b>；忌神供给度高 = <b style={{color:'#bc4747'}}>凶</b>。</i></div>
      </div>

      {/* —— 重大节点：大吉 / 大凶年份 chip 列 —— */}
      {(milestones.daji.length > 0 || milestones.daxiong.length > 0) && (
        <div className="fortune-milestones">
          <div className="fortune-mile-l">百年大节点 <small>（命局所喜/所忌集中的关键年份）</small></div>
          <div className="fortune-mile-row">
            {milestones.daji.length > 0 && (
              <div className="fortune-mile-group daji">
                <span className="fortune-mile-tag">大吉</span>
                {milestones.daji.map((p) => (
                  <span key={p.age} className="fortune-mile-chip daji" title={p.judgment.summary}>
                    {p.age} 岁 · {p.year} <small>{p.flow_year_ganzhi}</small>
                  </span>
                ))}
              </div>
            )}
            {milestones.daxiong.length > 0 && (
              <div className="fortune-mile-group daxiong">
                <span className="fortune-mile-tag">大凶</span>
                {milestones.daxiong.map((p) => (
                  <span key={p.age} className="fortune-mile-chip daxiong" title={p.judgment.summary}>
                    {p.age} 岁 · {p.year} <small>{p.flow_year_ganzhi}</small>
                  </span>
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      <div className="fortune-chart-wrap">
        <div className="fortune-chart-l">100 年用神供给曲线 · 0—{maxAge} 岁</div>
        <svg className="fortune-chart" viewBox={`0 0 ${W} ${H}`} role="img" aria-label="100 年用神供给度时间序列">
          {/* 吉凶分段背景条（按 timeline[i].judgment.level 着色） */}
          {timeline.map((p, i) => {
            const x1 = xOf(p.age)
            const x2 = i + 1 < timeline.length ? xOf(timeline[i + 1].age) : x1
            const fill = JUDGMENT_FILL[p.judgment.level] ?? 'transparent'
            return <rect key={p.age} x={x1} y={padT} width={Math.max(0, x2 - x1)} height={innerH} fill={fill} />
          })}
          {/* y 轴网格 + 标注 */}
          {[0, 25, 50, 75, 100].map((p) => (
            <g key={p}>
              <line x1={padL} x2={W - padR} y1={yOf(p)} y2={yOf(p)} stroke="#3a3a3a" strokeDasharray={p === 0 || p === 100 ? '0' : '2 4'} strokeWidth={p === 0 ? 1 : 0.5} />
              <text x={padL - 6} y={yOf(p) + 3} textAnchor="end" fontSize="9" fill="#888">{p}%</text>
            </g>
          ))}
          {/* x 轴 + 大运分段竖线 */}
          <line x1={padL} x2={W - padR} y1={H - padB} y2={H - padB} stroke="#666" />
          {dayunMarks.map((a) => (
            <g key={a}>
              <line x1={xOf(a)} x2={xOf(a)} y1={padT} y2={H - padB} stroke="#444" strokeDasharray="2 6" />
              <text x={xOf(a)} y={H - padB + 12} textAnchor="middle" fontSize="9" fill="#777">{a}</text>
            </g>
          ))}
          {/* 忌神供给曲线（填充 → 警示） */}
          <path d={pathOf('avoid_supply_pct')} fill="none" stroke={avoidColor} strokeWidth={1.5} strokeOpacity={0.65} strokeDasharray="3 3" />
          {/* 副用神供给曲线 */}
          {ys.secondary_wuxing && (
            <path d={pathOf('secondary_supply_pct')} fill="none" stroke={secondaryColor} strokeWidth={1.5} strokeOpacity={0.85} />
          )}
          {/* 主用神供给曲线（粗，主角） */}
          <path d={pathOf('primary_supply_pct')} fill="none" stroke={primaryColor} strokeWidth={2.4} />
          {/* playhead */}
          <line x1={playX} x2={playX} y1={padT} y2={H - padB} stroke="#ed8c47" strokeWidth={1.5} />
          <circle cx={playX} cy={yOf(at.primary_supply_pct)} r={4} fill={primaryColor} stroke="#fff" strokeWidth={1} />
        </svg>
        <div className="fortune-chart-legend">
          <span className="fortune-leg" style={{ color: primaryColor }}>● 主用神 {ys.primary_wuxing}</span>
          {ys.secondary_wuxing && <span className="fortune-leg" style={{ color: secondaryColor }}>● 副用神 {ys.secondary_wuxing}</span>}
          <span className="fortune-leg" style={{ color: avoidColor }}>— — 忌神最高</span>
          <span className="fortune-leg" style={{ color: '#777' }}>┊ 大运分段</span>
          <span className="fortune-leg" style={{ color: '#ed8c47' }}>┃ 当前年龄</span>
        </div>
        <div className="fortune-chart-foot">
          算力底层：每年 = （本命四柱 + 当前大运柱 + 流年柱）叠加旺衰 → 五行分布对主/副用神/忌神的占比%。
          <b>主用神长期偏高 + 忌神偏低</b>的区段 = 流年大运扶持有力的<b style={{color:'#9bbd6f'}}>吉运段</b>；反之 = <b style={{color:'#bc4747'}}>不利段</b>需谨慎。仅供研究与娱乐。
        </div>
      </div>
    </section>
  )
}

function NumField({ label, v, on, w = 58 }: {
  label: string; v: number; on: (e: { target: { value: string } }) => void; w?: number
}) {
  return (
    <label className="field">
      <span>{label}</span>
      <input type="number" value={v} onChange={on} style={{ width: w }} />
    </label>
  )
}

// 字/词模态术数（D 族：不吃出生时刻，吃文字/笔画）
function WordView() {
  const [sys, setSys] = useState<'gematria' | 'abjad' | 'wuge'>('gematria')
  const [text, setText] = useState('שלום')
  const [surname, setSurname] = useState('7')
  const [given, setGiven] = useState('16,9')
  const [res, setRes] = useState<Record<string, unknown> | null>(null)
  const [busy, setBusy] = useState(false)

  const parse = (s: string) => s.split(/[,，\s]+/).filter(Boolean).map(Number).filter((n) => n > 0)
  async function calc() {
    setBusy(true); setRes(null)
    const body = sys === 'wuge'
      ? { system: 'wuge', surname: parse(surname), given: parse(given) }
      : { system: sys, text }
    try {
      const r = await fetch('/api/word', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) })
      setRes(await r.json())
    } catch { setRes({ error: '请求失败' }) } finally { setBusy(false) }
  }
  const ex = { gematria: ['שלום', 'אמת', 'חי'], abjad: ['الله', 'بسم', 'محمد'] } as const

  useEffect(() => { void calc() }, []) // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <section className="card">
      <div className="lp-sec-t">文字术数 · 把文字按字母／笔画值换算并约化（不依赖出生时刻）</div>
      <div className="word-form">
        <label className="field"><span>系统</span>
          <select value={sys} onChange={(e) => { setSys(e.target.value as 'gematria' | 'abjad' | 'wuge'); setRes(null) }} style={{ width: 150 }}>
            <option value="gematria">希伯来 Gematria</option>
            <option value="abjad">阿拉伯 Abjad</option>
            <option value="wuge">姓名五格（笔画）</option>
          </select>
        </label>
        {sys !== 'wuge' ? (
          <>
            <label className="field"><span>{sys === 'gematria' ? '希伯来词' : '阿拉伯词'}</span>
              <input type="text" value={text} onChange={(e) => setText(e.target.value)} dir="rtl" style={{ width: 180, fontSize: 18 }} />
            </label>
            <div className="word-ex">{ex[sys].map((w) => <button key={w} className="exchip" onClick={() => setText(w)}>{w}</button>)}</div>
          </>
        ) : (
          <>
            <label className="field"><span>姓·笔画（逗号分）</span><input type="text" value={surname} onChange={(e) => setSurname(e.target.value)} style={{ width: 90 }} /></label>
            <label className="field"><span>名·笔画（逗号分）</span><input type="text" value={given} onChange={(e) => setGiven(e.target.value)} style={{ width: 110 }} /></label>
          </>
        )}
        <button className="go" onClick={() => void calc()} disabled={busy} style={{ marginLeft: 12 }}>{busy ? '…' : '计 算'}</button>
      </div>
      {res && <WordResult sys={sys} res={res} />}
      {sys === 'wuge' && <div className="lp-note">🟡 请填各字笔画（不内置康熙笔画表）；81 数理吉凶可参考传统熊崎健翁数理表自行查阅。</div>}
    </section>
  )
}
function WordResult({ sys, res }: { sys: string; res: Record<string, unknown> }) {
  if (res.error) return <div className="err">⚠ {String(res.error)}</div>
  const r = res.result as Record<string, unknown>
  if (sys === 'gematria') {
    const cells: [string, string, string][] = [
      ['hechrachi', '标准值', 'Hechrachi'],
      ['gadol', '大值', 'Gadol'],
      ['siduri', '序数', 'Siduri'],
      ['katan', '小值', 'Katan'],
      ['katan_mispari', '数字根', 'Katan Mispari'],
      ['atbash', '换码 AtBash', 'i ↔ 23−i'],
      ['albam', '换码 AlBam', '1..11 ↔ 12..22'],
    ]
    return (
      <div className="gematria-grid">
        {cells.map(([k, cn, en]) => (
          <div className="g-cell" key={k}>
            <div className="g-val">{String(r[k])}</div>
            <div className="g-cn">{cn}</div>
            <div className="g-en">{en}</div>
          </div>
        ))}
      </div>
    )
  }
  if (sys === 'abjad') {
    return (
      <div className="gematria-grid">
        <div className="g-cell"><div className="g-val">{String(r.mashriqi)}</div><div className="g-cn">东方序</div><div className="g-en">Mashriqī</div></div>
        <div className="g-cell"><div className="g-val">{String(r.maghribi)}</div><div className="g-cn">西方序</div><div className="g-en">Maghribī</div></div>
      </div>
    )
  }
  // wuge
  const grids: [string, string][] = [['天格', 'heaven'], ['人格', 'human'], ['地格', 'earth'], ['外格', 'outer'], ['总格', 'total']]
  return (
    <div className="wuge-grids">
      {grids.map(([cn, k]) => {
        const g = r[k] as { value: number; number: number; element_name: string }
        return <div className="wg-cell" key={k}><div className="wg-name">{cn}</div><div className="wg-val">{g.value}</div><div className="wg-sub">{g.number}数 · {g.element_name}</div></div>
      })}
    </div>
  )
}

// 跨叶相关性热力图
// 合盘 / 团队：N 人输入 → 团队五行画像 + N×N 互补矩阵。
// 默认 2 人对照（经典合婚）；可加成员到 12 人（团队五行画像）。
function TeamView() {
  const [members, setMembers] = useState([
    { name: 'A', year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8, gender: 'male' as const },
    { name: 'B', year: 1992, month: 3, day: 22, hour: 9, minute: 0, tz: 8, gender: 'female' as const },
  ])
  const [res, setRes] = useState<TeamResult | null>(null)
  const [busy, setBusy] = useState(false)
  const [interp, setInterp] = useState<Interpretation | null>(null)
  const [interpBusy, setInterpBusy] = useState(false)
  async function calc() {
    setBusy(true); setInterp(null)
    try {
      const r = await fetch('/api/team', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ members }) })
      setRes(await r.json())
    } catch { setRes(null) } finally { setBusy(false) }
  }
  async function interpretTeam() {
    setInterpBusy(true)
    try {
      const r = await fetch('/api/team/interpret', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ members }) })
      setInterp(await r.json())
    } catch { setInterp(null) } finally { setInterpBusy(false) }
  }
  useEffect(() => { void calc() }, []) // eslint-disable-line react-hooks/exhaustive-deps
  const upd = (i: number, k: keyof typeof members[number], v: string | number) => {
    setMembers((arr) => arr.map((m, j) => j === i ? { ...m, [k]: typeof m[k] === 'number' ? Number(v) : v } : m))
  }
  const add = () => members.length < 12 && setMembers([...members, { name: `成员${members.length + 1}`, year: 1990, month: 1, day: 1, hour: 12, minute: 0, tz: 8, gender: 'male' as const }])
  const rm = (i: number) => members.length > 1 && setMembers(members.filter((_, j) => j !== i))
  return (
    <section className="card">
      <div className="lp-sec-t">合盘 / 团队 · N 人五行画像 + 互补矩阵（计算 DET、结论 INT）</div>
      <div className="team-members">
        {members.map((m, i) => (
          <div className="team-member-row" key={i}>
            <input className="team-name" type="text" value={m.name} onChange={(e) => upd(i, 'name', e.target.value)} placeholder="名称" />
            <input type="number" value={m.year} onChange={(e) => upd(i, 'year', e.target.value)} style={{ width: 64 }} />
            <input type="number" value={m.month} onChange={(e) => upd(i, 'month', e.target.value)} style={{ width: 44 }} />
            <input type="number" value={m.day} onChange={(e) => upd(i, 'day', e.target.value)} style={{ width: 44 }} />
            <input type="number" value={m.hour} onChange={(e) => upd(i, 'hour', e.target.value)} style={{ width: 44 }} />
            <input type="number" value={m.minute} onChange={(e) => upd(i, 'minute', e.target.value)} style={{ width: 44 }} />
            <select value={m.gender} onChange={(e) => upd(i, 'gender', e.target.value)} style={{ width: 56 }}>
              <option value="male">男</option><option value="female">女</option>
            </select>
            <button className="team-rm" onClick={() => rm(i)} disabled={members.length <= 1}>×</button>
          </div>
        ))}
        <div className="team-actions">
          <button onClick={add} disabled={members.length >= 12}>+ 添加成员</button>
          <button className="go" onClick={() => void calc()} disabled={busy}>{busy ? '合盘中…' : '合 盘'}</button>
        </div>
      </div>
      {res && (
        <div style={{ marginTop: 18 }}>
          <div className="lp-sec-t">团队五行画像</div>
          <div className="wx-rows">
            {(['木', '火', '土', '金', '水'] as const).map((n) => {
              const v = ({ '木': res.team_wuxing.wood, '火': res.team_wuxing.fire, '土': res.team_wuxing.earth, '金': res.team_wuxing.metal, '水': res.team_wuxing.water } as Record<string, number>)[n] ?? 0
              return (
                <div className="wx-row" key={n}>
                  <span className="wx-row-n" style={{ color: WUXING_COLOR[n] }}>{n}</span>
                  <div className="wx-row-bar"><i style={{ width: `${v}%`, background: WUXING_COLOR[n] }} /></div>
                  <span className="wx-row-v">{v}%</span>
                </div>
              )
            })}
          </div>
          <div className="kv-grid" style={{ marginTop: 8 }}>
            <div className="stat"><span className="stat-k">团队最缺</span><span className="stat-v" style={{ color: WUXING_COLOR[res.team_weakest.wuxing] }}>{res.team_weakest.wuxing} {res.team_weakest.pct}%</span></div>
            <div className="stat"><span className="stat-k">团队最旺</span><span className="stat-v" style={{ color: WUXING_COLOR[res.team_strongest.wuxing] }}>{res.team_strongest.wuxing} {res.team_strongest.pct}%</span></div>
          </div>
          <div className="lp-sec-t" style={{ marginTop: 14 }}>互补矩阵 · 行 = 用神所属者，列 = 供给者（% 越高 = 列对行帮越大）</div>
          <table className="team-matrix">
            <thead>
              <tr><th></th>{res.members.map((m, j) => <th key={j}>{m.name}</th>)}</tr>
            </thead>
            <tbody>
              {res.members.map((mi, i) => (
                <tr key={i}>
                  <th>{mi.name}<i className="m-yong" style={{ color: WUXING_COLOR[mi.yongshen.primary_wuxing] }}>{mi.yongshen.primary_wuxing}</i></th>
                  {res.complement_matrix[i].map((v, j) => {
                    const intensity = Math.min(v, 50) / 50
                    return (
                      <td key={j} style={{ background: `rgba(95, 179, 191, ${intensity * 0.55})` }}>
                        {v}<i>%</i>
                      </td>
                    )
                  })}
                </tr>
              ))}
            </tbody>
          </table>
          <div className="lp-note" style={{ paddingTop: 10, fontSize: 14 }}>
            <b>「互补度」≠ 「合不合」</b>。这里只给客观指标（j 的盘里有多少 i 的用神），不下「适合 / 不适合」结论。
            🟡 团队五行均衡度有传统依据但无统一量化标准；合婚/合伙契合度可结合主用神互补来评估，仅供研究与娱乐。
          </div>
          <div className="team-interp">
            <button className="go interp-btn" onClick={() => void interpretTeam()} disabled={interpBusy}>
              {interpBusy ? '释义生成中…' : '🔮 团队释义（由 LLM 生成）'}
            </button>
            {interp && (
              <div className="int-block">
                <div className="int-text">{interp.text.split('\n').filter((s) => s.trim()).map((para, i) => <p key={i}>{para}</p>)}</div>
                <div className="int-by">🔮 INT · backend = {interp.backend} · 仅供研究与娱乐</div>
              </div>
            )}
          </div>
        </div>
      )}
    </section>
  )
}

function AnalysisView({ analysis }: { analysis: Analysis | null }) {
  if (!analysis) {
    return <section className="card"><div className="lp-note">相关性分析计算中…（首次约 2 秒）</div></section>
  }
  const { leaves, nmi, n } = analysis
  return (
    <section className="card">
      <div className="lp-sec-t">各术数之间的相关性（{n} 组随机生辰样本）</div>
      <div className="all-note">
        在大量随机生辰下，比较任意两种术数的某个特征是否同步变化（亮＝高度相关、暗＝各自独立）。
        基于天文历法的术数必然一致（<b>八字日支 ≡ 大六壬日支 ＝ 1.00</b>）；
        随机起卦类与生辰无关、彼此独立（≈0）。🟡 有限样本下相关度估计偏高，故「低相关」是保守结论。
      </div>
      <div className="heat" style={{ gridTemplateColumns: `132px repeat(${leaves.length}, 1fr)` }}>
        <div className="hc corner" />
        {leaves.map((l) => <div className="hc col" key={l.id} style={{ color: colorOf(l.id) }}>{l.name}</div>)}
        {leaves.map((row, i) => (
          <Fragment key={row.id}>
            <div className="hc rowh" style={{ color: colorOf(row.id) }}>{row.name}<i>{row.feature}</i></div>
            {leaves.map((col, j) => {
              const v = nmi[i][j]
              const txt = i === j ? '1' : v >= 0.12 ? v.toFixed(2).replace(/^0/, '') : ''
              return <div className="hcell" key={col.id} title={`${row.name} × ${col.name} = ${v.toFixed(3)}`} style={{ background: `rgba(201,162,74,${Math.max(0.02, v)})` }}>{txt}</div>
            })}
          </Fragment>
        ))}
      </div>
    </section>
  )
}

function SummaryBar({ bazi, ziwei, form }: { bazi: BaziChart; ziwei: ZiweiChart | null; form: ChartRequest }) {
  const bz = [bazi.year, bazi.month, bazi.day, bazi.hour]
  const hourName = HOUR_NAMES[Math.floor(((form.hour + 1) % 24) / 2)]
  return (
    <section className="summary">
      <div className="sm-left">
        <div className="sm-row">
          <span className="sm-k">公历</span>
          {form.year}-{String(form.month).padStart(2, '0')}-{String(form.day).padStart(2, '0')} {String(form.hour).padStart(2, '0')}:{String(form.minute).padStart(2, '0')}
          <span className="sm-sep">·</span>{hourName}时<span className="sm-sep">·</span>UTC{form.tz >= 0 ? '+' : ''}{form.tz}
        </div>
        <div className="sm-row"><span className="sm-k">农历</span>{lunarStr(bazi.lunar)}</div>
        {ziwei && <div className="sm-row"><span className="sm-k">命宫</span>{ziwei.ming_ganzhi}（{ziwei.ming_branch}）<span className="sm-sep">·</span>{ziwei.wuxing_ju}</div>}
        {bazi.strength && bazi.pattern && bazi.yongshen && (
          <div className="sm-row">
            <span className="sm-k">命局</span>
            <b style={{ color: WUXING_COLOR[bazi.day_master_wuxing] }}>{bazi.day_master}</b>·{bazi.strength.level}
            <span className="sm-sep">·</span>{bazi.pattern.name}
            <span className="sm-sep">·</span>喜<b style={{ color: WUXING_COLOR[bazi.yongshen.primary_wuxing] }}>{bazi.yongshen.primary_wuxing}</b>
            {bazi.yongshen.secondary_wuxing && <>/<b style={{ color: WUXING_COLOR[bazi.yongshen.secondary_wuxing] }}>{bazi.yongshen.secondary_wuxing}</b></>}
            {bazi.yongshen.avoid_wuxing.length > 0 && <>
              <span className="sm-sep">·</span>忌
              {bazi.yongshen.avoid_wuxing.map((n, i) => (
                <Fragment key={n}>{i > 0 && '/'}<b style={{ color: WUXING_COLOR[n] }}>{n}</b></Fragment>
              ))}
            </>}
          </div>
        )}
      </div>
      <div className="sm-right">
        <div className="bazi-big">
          {bz.map((p, i) => (
            <span className="bb-col" key={i}>
              <b style={{ color: WUXING_COLOR[p.stem_wuxing] }}>{p.stem}</b>
              <b style={{ color: WUXING_COLOR[p.branch_wuxing] }}>{p.branch}</b>
            </span>
          ))}
        </div>
        <div className="bazi-label">年　月　日　时</div>
      </div>
    </section>
  )
}

