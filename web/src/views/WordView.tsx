// 字词术数视图。
import { useEffect, useState } from 'react'
import { fetchWord } from '../api/client'

// 字/词模态术数（D 族：不吃出生时刻，吃文字/笔画）
export function WordView() {
  const [sys, setSys] = useState<'gematria' | 'abjad' | 'wuge' | 'numerology'>('gematria')
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
      setRes(await fetchWord(body))
    } catch { setRes({ error: '请求失败' }) } finally { setBusy(false) }
  }
  const ex = {
    gematria: ['שלום', 'אמת', 'חי'],
    abjad: ['الله', 'بسم', 'محمد'],
    numerology: ['Ada Lovelace', 'Kurt Godel', 'Emmy Noether'],
  } as const

  useEffect(() => { void calc() }, []) // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <section className="card">
      <div className="lp-sec-t">文字术数 · 把文字按字母／笔画值换算并约化（不依赖出生时刻）</div>
      <div className="word-form">
        <label className="field"><span>系统</span>
          <select value={sys} onChange={(e) => { setSys(e.target.value as typeof sys); setRes(null) }} style={{ width: 160 }}>
            <option value="gematria">希伯来 Gematria</option>
            <option value="abjad">阿拉伯 Abjad</option>
            <option value="wuge">姓名五格（笔画）</option>
            <option value="numerology">数字学姓名数</option>
          </select>
        </label>
        {sys !== 'wuge' ? (
          <>
            <label className="field"><span>{sys === 'gematria' ? '希伯来词' : sys === 'abjad' ? '阿拉伯词' : '拉丁字母姓名'}</span>
              <input
                type="text"
                value={text}
                onChange={(e) => setText(e.target.value)}
                dir={sys === 'numerology' ? 'ltr' : 'rtl'}
                style={{ width: 180, fontSize: 18 }}
              />
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
      {sys === 'numerology' && (
        <div className="lp-note">
          🟡 两套字母表并出，不替你选边——两套各有传承且给出不同的数。Y 何时作元音三种约定的读数在接口里全给，
          此处显示的是按上下文判定的那一套。
        </div>
      )}
    </section>
  )
}

export function WordResult({ sys, res }: { sys: string; res: Record<string, unknown> }) {
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
  if (sys === 'numerology') {
    // 两套字母表并出，不替读者选边——两套各有传承且给出不同的数
    const sets: [string, string][] = [['pythagorean', 'Pythagorean（A=1…I=9 循环）'], ['chaldean', 'Chaldean（1..8，9 不配字母）']]
    const rows: [string, string][] = [['expression', '表达数'], ['soul_urge', '灵魂数'], ['personality', '人格数']]
    return (
      <div className="num-name">
        {sets.map(([k, label]) => {
          const n = r[k] as Record<string, number>
          return (
            <div className="gematria-grid" key={k}>
              <div className="g-cell g-head">{label}</div>
              {rows.map(([f, cn]) => (
                <div className="g-cell" key={f}>
                  <div className="g-val">{String(n[f])}</div>
                  <div className="g-cn">{cn}</div>
                </div>
              ))}
            </div>
          )
        })}
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
