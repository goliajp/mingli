// 页面组装：表单 → 请求 → 各视图。计算与展示都在别处，这里只连线。
import { useState } from 'react'
import type { BaziChart, ChartRequest } from './types'
import { LeafChart } from './views/leaves'
import { IntentBar, IntentPendingCard } from './components/IntentBar'
import { ElectionView } from './views/ElectionView'
import { EventView } from './views/EventView'
import { LocativeView } from './views/LocativeView'
import { MundaneView } from './views/MundaneView'
import { SynastryView } from './views/SynastryView'
import { NumField } from './components/NumField'
import { SummaryBar } from './components/SummaryBar'
import { TimeScrubber } from './components/TimeScrubber'
import { CITIES, REGIONS, coordStr } from './data/cities'
import { colorOf, regionOf } from './data/leaf-regions'
import { useAnalysis, useInterpretations, useNatalCast } from './hooks/useCast'
import { useFortune, useIntents, useLeavesAt, useTimeline } from './hooks/useTimeline'
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
  const intentsList = useIntents()

  const { bazi, ziwei, leaves, err, setErr, loading, runId, run } = useNatalCast(form)
  const { age, nowAge, setPlayAge, playDate } = useTimeline(form)
  const leavesT = useLeavesAt(form, age, playDate)
  const fortune = useFortune(form, age, playDate, intent === 'fortune', setErr)
  const analysis = useAnalysis(tab === 'analysis')
  const { interp, generate: genInterp } = useInterpretations(form)

  const set = (k: keyof ChartRequest) => (e: { target: { value: string } }) => {
    const v = e.target.value
    setForm((f) => ({ ...f, [k]: k === 'gender' || k === 'name' ? v : Number(v) }))
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

      {err && <div className="err">⚠ {err}</div>}

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
      {intent === 'locative' && <LocativeView />}
      {intent === 'synastry' && <SynastryView />}
      {intent === 'mundane' && <MundaneView />}

      {intent !== 'natal' && intent !== 'fortune' && intent !== 'event' && intent !== 'election' && intent !== 'locative' && intent !== 'synastry' && intent !== 'mundane' && intentsList && (
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
