# mingli — a computable core for the world's divination systems

[中文](README.zh-CN.md)

Divination systems, implemented as **algorithms**: deterministic casting engines in Rust, an axum service, and a React 19 frontend.

The organizing principle is a strict split between **computing a chart, interpreting it, and talking about it**. This repository only does the first. What a chart *means* is quarantined behind `mingli-interpret` and is always marked as a non-computed artifact.

> 34 crates · 24 leaves (21 time-driven leaves fan out in parallel, 3 word-driven leaves go through `/api/word`) · 434 tests green
> `unsafe_code = "forbid"` · `missing_docs = "deny"` · `clippy::all = "deny"`

---

## The shape of it

```
              LEAVES  —— one leaf per system, each producing a chart
  Bazi  Ziwei  Qimen  Zeri │ Western astrology  Jyotish  Qizheng Siyu
  I Ching  Geomancy  Ifá  Sikidy  Tarot │ Numerology  Wuge │ Maya  Tibetan …
              BRANCHES —— 4 families plus one cross-cutting branch (computational paradigms)
     A cyclic groups / CRT │ B angle quantization │ C sampling / binary │ D hash rings │ ⟂ permutation
              TRUNK  —— shared symbolic components (L2)
     sexagenary cycle │ 64 hexagrams │ Luoshu magic square │ 28 mansions / zodiac │ 16 figures / 256 odu
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
              ROOTS —— mathematical and physical stones
 L1 physical: Julian day · ΔT │ solar longitude → solar terms │ lunar phase → lunisolar calendar
              │ planetary ephemerides │ nutation / precession / ayanamsa
 L0 math:     Cyclic+CRT │ Quantizer │ BinaryLattice+GF(2) │ RingHash │ GroupAction │ SeededSampler
```

Every leaf is an assembly of *which stones from the roots × which symbolic component from the trunk × which branch (family paradigm)*. **Growing a new leaf never touches the roots.**

---

## Determinism labels

Every node carries a label saying who executes it and how far it can be trusted. **The chart is always real; uncertainty is confined to the edges (UND) and to interpretation (INT).**

| Label | Meaning | Executed by | Examples |
|---|---|---|---|
| 🟢 **DET** deterministic | Pure math or physics, reproducible and provable | Rust | Solar terms, sexagenary cycle, hexagram bits |
| 🎲 **STO** stochastic | Genuinely random casting, but the seed is recorded, so it replays | Rust | Family C draws (yarrow, coins, tarot, geomancy) |
| 🟡 **UND** underdetermined | Traditions disagree, an authoritative constant is missing, or a table is unverified | Config switch | Ziqi epoch, which of the 81 numerology tables, midnight rollover, orb conventions |
| 🔮 **INT** interpretation | Turning a chart into prose | LLM | Everything about what a chart "means" |

Each leaf declares its own DET/STO/UND boundaries through `CastingEngine::profile()`. **Where traditions genuinely conflict and no cross-checkable source settles it, the engine returns `None` and marks the point UND rather than inventing an answer.**

---

## Layout

```
crates/
  L0 math roots  mingli-core         finite cyclic groups / CRT / quantizer / GF(2) lattice / hash ring / group action / seeded sampler
  L1 physical    mingli-astro        Julian day · apparent solar longitude and the 24 solar terms · lunar phase and intercalation · sexagenary cycle
                 mingli-ephemeris    planetary ephemerides — geocentric ecliptic longitudes (VSOP87 / ELP-2000)
  L2 trunk       mingli-ganzhi       the sexagenary cycle as a symbol system
                 mingli-gua          the 64-hexagram lattice (Z₂)⁶
                 mingli-luoshu       Luoshu magic square and nine-palace flight
  L3 leaves — family A (cyclic groups / CRT)
                 mingli-maya         the three interlocking Maya calendar counts
                 mingli-pawukon      the Balinese Pawukon calendar
                 mingli-mahabote     Burmese Mahabote natal core numbers
                 mingli-tibetan      cyclic elements of the Tibetan calendar
                 mingli-zeri         cyclic elements of date selection
                 mingli-xiaoliuren   Xiao Liu Ren (Zhuge's roadside divination)
  L3 leaves — family B (angle quantization)
                 mingli-astrology    Western natal charts (Placidus / Koch / Whole Sign / Equal / Porphyry)
                 mingli-jyotish      Vedic astrology (4 ayanamsas · 27 nakshatras · Vimshottari · D-9)
                 mingli-qizhengsiyu  Qizheng Siyu — indigenous Chinese astrology
  L3 leaves — family C (sampling / binary)
                 mingli-yijing       I Ching casting
                 mingli-geomancy     ʿilm al-raml geomancy
                 mingli-sikidy       Malagasy Sikidy
                 mingli-ifa          West African Yoruba Ifá
                 mingli-cartomancy   card draws (Tarot / Lenormand / Futhark — 5 decks)
                 mingli-meihua       Meihua Yishu, several casting methods (the deterministic corner of family C)
  L3 leaves — family D (hash rings)
                 mingli-numerology   Western numerology (Pythagorean + Chaldean)
                 mingli-gematria     Hebrew gematria, 7 methods
                 mingli-abjad        Arabic abjad numerals (Mashriqī / Maghribī orders)
                 mingli-wuge         Japanese-style five-grid name analysis (Kumazaki)
  L3 leaves — cross-cutting ⟂ (permutation / flight)
                 mingli-bazi         Four Pillars — pillars · ten gods · luck cycles · strength · useful god · year/luck overlay
                 mingli-ziwei        Zi Wei Dou Shu — life and body palaces · five-element bureau · 14 majors · 4 auxiliaries · four transformations
                 mingli-liuren       Da Liu Ren
                 mingli-qimen        Qi Men Dun Jia (hour-plate rotating method)
                 mingli-taiyi        Tai Yi Shen Shu
  L3.5 orchestration
                 mingli-engine       treats the tree as a memoized computation DAG, fanning out across all leaves in parallel
                 mingli-analysis     information-theoretic statistics across leaves
                 mingli-interpret    interpretation layer — assembles guard-railed prompts, strictly separated from computation
                 mingli-wasm         wasm bindings for the whole library
services/
  mingli-api/   axum service, :6027
web/            React 19 + Vite frontend, :6026 (proxies /api to :6027 in dev)
```

---

## Running it

```bash
# 1) Backend (:6027)
cargo run -p mingli-api

# 2) Frontend (:6026, separate terminal)
cd web && npm install && npm run dev
# open http://localhost:6026
```

## Tests and cross-checks

```bash
cargo test --workspace     # 434 tests
cargo clippy --workspace   # deny-clean
cargo doc --workspace      # fully documented
```

Every authoritative reference value is **confirmed against multiple independent sources** and lives in a `#[test]` in the relevant crate. For example:

- Day-pillar anchor 2024-01-01 = 甲子 (three sources agree)
- Exact 2024 solar term timestamps (Beijing time; dates exact, times ±15 min under Meeus' low-precision model)
- Lunar New Year 2020–2025 and leap months (leap 2nd month 2023, leap 4th month 2020, including knife-edge true-solar-term cases)
- Full worked example 1990-06-15 14:30 CST → Bazi 庚午 / 壬午 / 辛亥 / 乙未; Ziwei life palace 亥·丁亥, Earth-5 bureau, Zi Wei in 申, 巨门 as the life-palace star
- All 12 Placidus and Koch cusps for Diana (Rodden AA, 1961-07-01 19:45 BST), within 0.05° of pyswisseph
- Lahiri ayanamsa at three epochs against the Swiss Ephemeris `sweph.h` source constant, within ±0.05°
- The classic Jeremiah 25:26 atbash `בבל`(34) ⇌ `ששך`(620)

## API

```bash
curl -X POST http://127.0.0.1:6027/api/bazi -H 'content-type: application/json' \
  -d '{"year":1990,"month":6,"day":15,"hour":14,"minute":30,"tz":8,"gender":"male"}'
```

| Route | What it does |
|---|---|
| `GET  /api/health` | Health check |
| `POST /api/cast` | Parallel fan-out — one input, every system cast at once |
| `POST /api/bazi` · `/api/bazi/overlay-strength` | Four Pillars chart / luck-layer strength overlay |
| `POST /api/ziwei` | Zi Wei Dou Shu chart |
| `POST /api/fortune` | Aggregate fortune at an instant, plus a century-long supply timeline |
| `POST /api/word` | Word-driven leaves (numerology / gematria / abjad / wuge) |
| `POST /api/team` · `/api/team/interpret` | Multi-subject (team) charts |
| `GET  /api/analysis` | Information-theoretic statistics across leaves |
| `GET  /api/intents` · `POST /api/route` | Query model — route an intent to the leaves that answer it |
| `POST /api/interpret` | Interpretation layer (🔮 INT, not a computed result) |

Request fields: `year month day hour` (required), `minute` (default 0), `tz` (default +8), `gender` (`male` / `female`; omit to skip luck cycles). Supported range 1900–2100.
Bind address is overridable with `MINGLI_API_BIND`.

---

## Precision and known simplifications

Honest boundaries come before flattering numbers. Each item below corresponds to a 🟡 UND declaration in the code:

- Astronomy uses Meeus' simplified models: solar longitude to ~0.01° (≈15 min), new moon to a few minutes. **Which day something falls on is reliable**; the minute-level timestamp of a solar term may be off by a few minutes
- Hour pillars use civil clock time by default; apparent solar time correction (longitude offset + a simplified equation of time, ±0.5 min) is **opt-in** via `true_solar_time`
- The late-Zi-hour day rollover in Bazi is handled at civil midnight; the alternative split is not implemented
- For people born in a leap month, Ziwei takes the base month number; the first-half / second-half split is not implemented
- Luck cycle onset uses the 3-days-to-1-year convention without month/day refinement
- Where Da Liu Ren's three-transmission rules diverge across traditions (涉害 / 昴星 / 别责 / 八专), the engine identifies the pattern but returns `None` rather than forcing a reading
- In Qizheng Siyu, Ziqi, the 12 stations, and the classical ecliptic division of the 28 mansions are all marked UND rather than fabricated

## Disclaimer

This is an **algorithms research project**. All output is for research and entertainment only and does not constitute advice of any kind.

## License

MIT — see [`LICENSE`](LICENSE).
