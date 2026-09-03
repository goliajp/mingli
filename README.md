# mingli — a computable core for the world's divination systems

[中文](README.zh-CN.md)

[![CI](https://github.com/goliajp/mingli/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/goliajp/mingli/actions/workflows/ci.yml)

Divination systems, implemented as **algorithms**: deterministic casting engines in Rust, an axum service, and a React 19 frontend.

The organizing principle is a strict split between **computing a chart, interpreting it, and talking about it**. This repository only does the first. What a chart *means* is quarantined behind `mingli-interpret` and is always marked as a non-computed artifact.

> 39 crates · 24 leaves (21 time-driven leaves fan out in parallel, 4 word-driven leaves go through `/api/word`, one of them both) · 8 intents, over HTTP and wasm alike · 844 tests green
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

Dependencies point strictly inward. Each leaf implements the ports declared in `mingli-contract`; the orchestrator consumes those ports and never names a leaf; the composition root is the only place that enumerates them. That rule is enforced by a test (`crates/mingli-registry/tests/architecture.rs`) which reads every manifest and fails on an outward edge.

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
                 mingli-ephemeris    planetary ephemerides — geocentric ecliptic longitudes (VSOP87 / ELP-2000) · ascendant and midheaven geometry
  L2 ports       mingli-contract     the CastingEngine / WordEngine ports plus the shared query and declaration types
     trunk       mingli-ganzhi       the sexagenary cycle as a symbol system
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
                 mingli-astrology    Western natal charts (Placidus / Koch / Whole Sign / Equal / Porphyry) · cross-chart aspects
                 mingli-jyotish      Vedic astrology (4 ayanamsas · 27 nakshatras · Vimshottari · 14 of the 16 divisional charts)
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
  L4 orchestration
                 mingli-engine       memoized shared context + parallel fan-out over an injected registry; knows no leaf
                 mingli-interpret    interpretation layer — assembles guard-railed prompts, strictly separated from computation
  L5 analysis    mingli-analysis     information-theoretic statistics across leaves
  L6 use cases   mingli-app          one use case per intent — natal · fortune · event · election · locative · synastry · mundane · word — and the input shapes both delivery paths share
  L7 assembly    mingli-registry     the one place that knows which leaves exist — add a leaf by registering one line here
  L8 delivery    mingli-wasm         wasm bindings — all eight intents, taking the same request bodies the HTTP endpoints take
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
cd web && bun install && bun run dev
# open http://localhost:6026
```

## Seeing it

```bash
cd web && bun run shots     # reconcile the CSS variables, then walk the UI in headless Chromium
```

30 screens — all twenty-one leaves, the cross-cutting tabs, every intent — at 1440 and again at
1024, landing in `web/e2e/shots/<width>/`. Console errors, page exceptions and failed requests are
collected, and eight screens assert on what they rendered: the nine-palace grid has nine cells,
each group heading's count matches the rows beneath it, the compass plots as many pins as the table
has rows, a graha's third-divisional sign sits 0, 4 or 8 signs from its natal one. A final check
counts requests over three idle seconds — a page that keeps talking to the backend while nobody
touches it is looping. Any of that failing exits non-zero.

Before the screens, one more pass compares what `/api/cast` returns against every
source file under `web/src`: a field the backend computes that no file so much as
names is a field nobody can see. Five had slipped through that way before the
check existed.

A type check proves the code compiles. This proves the page still agrees with itself.

## Taking only what you need

Each system is its own crate, and the registry gates each one behind a feature named
after it. Nothing forces you to carry twenty-four.

```bash
cargo add mingli-bazi                     # one crate: serde plus three shared layers, no ephemeris
cargo add mingli-registry --no-default-features --features bazi,yijing   # two systems behind the port
```

Measured, release wasm32:

| npm package | Build | Module | gzipped |
|---|---|---:|---:|
| `mingli-wasm-astrology-thin` | Natal charts, truncated ephemeris built in | 230 KB | 120 KB |
| `mingli-wasm-yijing` | Yi Jing only | 156 KB | 72 KB |
| `mingli-wasm-astrology-lite` | Natal charts, you supply the positions | 162 KB | 73 KB |
| `mingli-wasm-bazi` | Four Pillars only | 195 KB | 89 KB |
| `mingli-wasm-chinese` | The ten Chinese systems | 342 KB | 143 KB |
| `mingli-wasm-chart` | All twenty-four, charts only | 1238 KB | 713 KB |
| `mingli-wasm` | All twenty-four plus use cases | 1444 KB | 790 KB |


### Bring your own ephemeris

VSOP87D's constant tables are about 780 KB of a browser bundle that computes
planetary positions — around ninety percent of it — and evaluating the series is
97% of the time it takes to build a chart. If the host already has an ephemeris
(a browser has a fifty-kilobyte JavaScript one, thirty times faster than ours),
hand over the nine longitudes instead:

```js
import init, { astrology_with } from 'mingli-wasm-astrology-lite';
await init();
// Order: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune
const lons = [83.91, 342.30, 65.30, 48.51, 10.88, 105.84, 294.05, 277.32, 283.61];
const chart = JSON.parse(astrology_with(JSON.stringify(birth), JSON.stringify(lons)));
```

Ascendant, midheaven, houses, the chosen cusp system, aspects and sign
placements are still computed here; only where the numbers came from moves out.
In Rust it is `mingli_astrology::compute_at_with`, with the leaf's `ephemeris`
feature off. Same code path measured both ways: 857,633 bytes computing
locally, 79,863 taking them from the caller.

A system costs about 0.05 MB on top of the skeleton; the three that carry planetary
ephemerides cost 0.87 MB between them. `feature-matrix.sh` builds every leaf on its
own, so a system that quietly drags in another fails there rather than in your bundle.
It also runs each of the 39 crates' tests in isolation: `cargo test --workspace` compiles
with the *union* of everyone's features, so a crate whose own test dependencies are short
a feature still passes there, and only fails when run alone.

Casting is dominated by the three leaves that walk a planetary ephemeris. On this
machine one chart takes roughly 300 µs for Western astrology, 250 µs for Jyotish and
220 µs for the Seven Luminaries; 9 µs for Four Pillars and 7 µs for Zi Wei; and under
5 µs each for the other sixteen. A guard checks the shape rather than the microseconds,
which belong to this machine: exactly those three must cost two orders of magnitude
more than the median leaf, so a system that starts walking an ephemeris shows up as
one, and one that stops shows up too.
A guard fails if any single leaf takes more than 60% of the whole tree's casting time
or payload -- it exists because one did, once.

Guards need guarding too. A test that can never fail and a test that is really holding
something up look identical on a green run; the only way to tell them apart is to put the
fault back and see whether it gets caught. `guard-probe.sh` turns that from something
someone once did by hand into a command anyone can re-run: it plants 122 known faults
and asks, for each, whether the guard that should catch it goes red. It has already found
one guard that did not do what its name said -- "the composition root is the only place
that lists leaves" never looked at the interpretation layer at all.


## Tests and cross-checks

```bash
cargo test --workspace     # 844 tests
cargo clippy --workspace   # deny-clean
cargo doc --workspace      # fully documented
./scripts/coverage.sh      # 98%+ regions; every file below the line has a written reason
./scripts/api-snapshot.sh check snap.txt   # 43 requests, byte for byte
./scripts/test-count.sh    # the count in this README, against a real run
./scripts/feature-matrix.sh  # every leaf built alone, every crate tested alone, wasm32, one dependency-graph check
./scripts/guard-probe.sh   # plants 122 known faults, checks the guard that should catch each one does
```

All of the above, plus the screenshot pass, run on every push — see the badge at the top.

One check is deliberately not among them:

```bash
./scripts/mutants.sh mingli-astro   # break every spot in a crate, one at a time; see which breaks nothing
```

`guard-probe.sh` plants faults we chose, and asks whether the guard meant to
catch each one does. This asks the opposite question — is there anywhere in a
crate that nobody is watching — and it answers by breaking every spot in turn.
A run takes hours, so it stays a hand tool rather than a gate. What it found
here is the kind of thing a passing suite hides: coefficients too small to
observe over the dates this project supports, loops that answered a wrong input
by never returning, and tolerances wider than the wobble they were meant to
catch.

Not every survivor is a gap. Some are equivalent mutants — `+ 180` and `- 180`
agree under a mod 360, a quadrant's formula turns out to equal its neighbour's.
Proving that costs more than the scan does, so the conclusions are kept in
`scripts/mutants-known.txt` with their reasons, and each run reconciles against
it: only survivors nobody has explained yet are reported, and entries that stop
appearing are flagged so the list cannot quietly go stale.

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
| `POST /api/event` · `/api/event/interpret` | Divination for a question asked at an instant |
| `POST /api/election` · `/api/election/interpret` | Scan a window and rank the days in it |
| `POST /api/locative` · `/api/locative/interpret` | Which direction to face |
| `POST /api/synastry` · `/api/synastry/interpret` | Two people — what each supplies the other |
| `POST /api/mundane` · `/api/mundane/interpret` | A polity's founding moment and its year charts |
| `POST /api/interpret` | Interpretation layer (🔮 INT, not a computed result) |

Request fields: `year month day hour` (required), `minute` (default 0), `tz` (default +8), `gender` (`male` / `female`, or `男` / `女`; omit to skip luck cycles — anything else is rejected rather than quietly ignored). Supported range 1900–2100.
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
