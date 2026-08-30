// 与 mingli-api 的 serde 输出一一对应。

export interface ChartRequest {
  year: number
  month: number
  day: number
  hour: number
  minute: number
  tz: number
  gender?: 'male' | 'female'
  latitude?: number
  longitude?: number
  seed?: number
  name?: string
  leaf?: string
  /** 流派选择： key=叶 id， value=流派 id（见各叶 schools）。 */
  schools?: Record<string, string>
  /** 真太阳时：true 则按 longitude+EoT 校正时柱。需要 longitude。 */
  true_solar_time?: boolean
  /** 主体类型(`person`/`company`/`product`/`event`)：仅 /api/interpret 释义层生效。 */
  subject?: 'person' | 'company' | 'product' | 'event'
}

/** 一个流派（对应 engine::SchoolItem）。 */
export interface SchoolItem {
  id: string
  name: string
  default: boolean
  note: string
}

/** 问局意图状态（对应 engine::IntentStatus）。 */
export type IntentStatus = 'Live' | 'Pending'

/** 问局意图规格（对应 engine::IntentSpec）。 */
export interface IntentSpec {
  id: string
  name_zh: string
  atoms: string[]
  default_leaves: string[]
  output_shape: string
  status: IntentStatus
  status_label: string
  note: string
}

/** /api/intents 返回：8 类意图清单 + 当前注册叶集合。 */
export interface IntentsResponse {
  intents: IntentSpec[]
  registered_leaves: { id: string; name: string; family: Family; family_label: string }[]
}

/** 吉凶判读（对应 bazi::Judgment）。 */
export interface Judgment {
  /** 大吉 / 吉 / 平 / 凶 / 大凶 */
  level: string
  /** 净增益分（主用神 + 0.5*副用神 − 最高忌神）。 */
  score: number
  /** 一句话判读。 */
  summary: string
}

/** Fortune：t 时刻运势切片（对应 bazi::FortuneAt）。 */
export interface FortuneAt {
  natal: BaziChart
  t_chart: BaziChart
  age_years: number
  dayun_step: number | null
  dayun_ganzhi: string | null
  flow_year_ganzhi: string
  ming_strength: Strength
  yun_strength: Strength
  delta_score: number
  primary_supply_pct: number
  secondary_supply_pct: number | null
  avoid_supply_pcts: number[]
  judgment: Judgment
}

/** 用神供给时间序列单年点（对应 bazi::FortuneTimelinePoint）。 */
export interface FortuneTimelinePoint {
  age: number
  year: number
  flow_year_ganzhi: string
  dayun_step: number | null
  dayun_ganzhi: string | null
  yun_score: number
  primary_supply_pct: number
  secondary_supply_pct: number | null
  avoid_supply_pct: number
  judgment: Judgment
}

/** /api/fortune 返回。 */
export interface FortuneResponse {
  at: FortuneAt
  timeline: FortuneTimelinePoint[]
  max_age: number
  /** Vimshottari 大运/小运。构建未含 jyotish 时为 null。 */
  dasha: DashaSlice | null
  /** 西洋占星的二次推运（一日一年）。构建未含 astrology 时为 null */
  progression: Progression | null
  ziwei: ZiweiFortune | null
}

/** 紫微那一层的「运」：所问之岁落在哪一步大限，所问之年入哪一宫。 */
export interface ZiweiFortune {
  system: string
  ming_branch: string
  /** 性别缺省时大限出不来（顺逆由「年干阴阳 + 性别」定），此处为 null */
  limit: { step: number; start_age: number; end_age: number; branch: string; palace: string } | null
  annual: { year: number; branch: string; palace: string }
}

/** 一格推运（对应 astrology::progression::ProgressedYear）。 */
export interface ProgressedYear {
  age: number
  planets: { name: string; sign: string; degree: number; longitude: number }[]
  /** 推运星与本命星之间的相位——「运」的着力处 */
  to_natal: { a: string; b: string; kind: string; angle: number }[]
}

/** 二次推运时间线（对应 astrology::progression::Progression）。 */
export interface Progression {
  method: string
  max_age: number
  /** 相邻两格相差几岁 */
  step: number
  years: ProgressedYear[]
}

/** /api/fortune 里的 Vimshottari 切片。 */
export interface DashaSlice {
  system: string
  birth_lord: string
  age_years: number
  /** 目标时刻所在的那一段大运；时刻落在 120 年之外时为 null */
  current: Mahadasha | null
  timeline: Mahadasha[]
}

// /api/interpret LLM 释义（INT，非计算）
export interface Interpretation {
  leaf: string
  text: string
  backend: string
  kind: string
}

// /api/cast 全叶并行排盘的单叶输出（与 engine::LeafOutput 对应）。
export type Family = 'Cyclic' | 'Angular' | 'Sampling' | 'Hashing' | 'CrossCutting'

// 确定性谱（与 engine::DetItem / Determinism 对应）
export type Determinism = 'Det' | 'Sto' | 'Und'
export interface DetItem {
  aspect: string
  status: Determinism
  note: string
}

export interface CastLeaf {
  id: string
  name: string
  family: Family
  family_label: string
  profile: DetItem[]
  schools: SchoolItem[]
  effective_school: string
  chart: unknown
}

export interface CastResponse {
  leaves: CastLeaf[]
}

// /api/analysis 跨叶相关性（信息论 NMI）
export interface LeafStat {
  id: string
  name: string
  family: Family
  feature: string
  entropy: number
  distinct: number
}
export interface Analysis {
  n: number
  leaves: LeafStat[]
  nmi: number[][]
}

export interface HiddenStem {
  stem: string
  ten_god: string
}

export interface Pillar {
  ganzhi: string
  stem: string
  branch: string
  stem_wuxing: string
  branch_wuxing: string
  nayin: string
  ten_god: string
  hidden: HiddenStem[]
  day_twelve: string
  shensha: string[]
}

export interface LuckPillar {
  start_age: number
  ganzhi: string
}

export interface DaYun {
  forward: boolean
  start_age_years: number
  pillars: LuckPillar[]
}

export interface Lunar {
  year: number
  month: number
  leap: boolean
  day: number
}

export interface WuxingPower {
  wood: number
  fire: number
  earth: number
  metal: number
  water: number
}

export interface Strength {
  score: number
  level: string
  got_ling: number
  got_di: number
  got_shi: number
  wuxing: WuxingPower
}

export interface OverlayStrength {
  ming: Strength
  yun: Strength
  delta_score: number
  extras: string[]
}

export interface Pattern {
  name: string
  source: string
  qi_stem: string
  qi_kind: string
  revealed_in?: string | null
  ten_god: string
  revealed: boolean
  is_lu_ren: boolean
}

export interface YongShen {
  method: string
  primary_wuxing: string
  primary_role: string
  secondary_wuxing?: string | null
  secondary_role?: string | null
  avoid_wuxing: string[]
  reasoning: string
}

export interface ThreeHouses {
  ming_gong: string
  shen_gong: string
  tai_yuan: string
}

export interface TeamMember {
  name: string
  day_master: string
  day_master_wuxing: string
  year_gz: string
  month_gz: string
  day_gz: string
  hour_gz: string
  strength: Strength
  yongshen: YongShen
}

export interface TeamResult {
  members: TeamMember[]
  team_wuxing: WuxingPower
  team_weakest: { wuxing: string; pct: number }
  team_strongest: { wuxing: string; pct: number }
  complement_matrix: number[][]
}

export interface BaziChart {
  lunar: Lunar
  year: Pillar
  month: Pillar
  day: Pillar
  hour: Pillar
  day_master: string
  day_master_wuxing: string
  xunkong: [string, string]
  strength: Strength
  pattern: Pattern
  yongshen: YongShen
  three_houses: ThreeHouses
  dayun?: DaYun
}

export interface Palace {
  name: string
  branch: string
  ganzhi: string
  stars: string[]
  is_ming: boolean
  is_shen: boolean
}

export interface JyotishGraha {
  graha: string
  name: string
  sidereal_lon: number
  rasi: number
  rasi_name: string
  nakshatra: number
  nakshatra_name: string
  nakshatra_lord: string
  navamsa: number
  navamsa_name: string
  /** 其余十二个分盘的落宫（分盘 id → rasi 索引 0..11）。D-9 见 navamsa。 */
  vargas: Record<string, number>
}

export interface Mahadasha {
  lord: string
  years: number
  effective_years: number
  start_jd: number
  end_jd: number
  start_age_years: number
  end_age_years: number
  /** 本段内的九步小运 */
  antardashas: Antardasha[]
}

/** 一段大运内的小运（对应 jyotish::Antardasha）。 */
export interface Antardasha {
  lord: string
  years: number
  start_jd: number
  end_jd: number
  start_age_years: number
  end_age_years: number
}

export interface JyotishChart {
  ayanamsa_id: 'lahiri' | 'krishnamurti' | 'raman' | 'fagan_bradley'
  ayanamsa_deg: number
  grahas: JyotishGraha[]
  birth_dasha_lord: string
  mahadashas: Mahadasha[]
  lagna_lon: number | null
  lagna_rasi: number | null
  lagna_rasi_name: string | null
  lagna_navamsa: number | null
  lagna_navamsa_name: string | null
}

export interface Sihua {
  school_id: 'standard' | 'quanshu'
  lu_star: string
  lu_branch: string | null
  quan_star: string
  quan_branch: string | null
  ke_star: string
  ke_branch: string | null
  ji_star: string
  ji_branch: string | null
}

export interface StarPosition {
  star: 'sun' | 'moon' | 'mercury' | 'venus' | 'mars' | 'jupiter' | 'saturn' | 'luohou' | 'jidu' | 'yuebo'
  name: string
  is_qizheng: boolean
  longitude: number
  sign: number
  sign_name: string
  degree_in_sign: number
}

export interface QizhengsiyuChart {
  stars: StarPosition[]
  mansion: number
  mansion_name: string
  day_ganzhi: string
}

export interface ZiweiChart {
  lunar: Lunar
  ming_branch: string
  shen_branch: string
  ming_ganzhi: string
  wuxing_ju: string
  ju_number: number
  ziwei_branch: string
  tianfu_branch: string
  palaces: Palace[]
  sihua: Sihua
  /** 大限盘（十年一宫）。性别缺省时为 null——顺逆由「年干阴阳 + 性别」定，缺一不可 */
  major_limits: MajorLimits | null
}

/** 一步大限（对应 ziwei::limit::MajorLimit）。 */
export interface MajorLimit {
  step: number
  start_age: number
  end_age: number
  branch_index: number
  branch: string
  palace: string
}

/** 大限盘（对应 ziwei::limit::MajorLimits）。 */
export interface MajorLimits {
  /** 起运岁 = 五行局数 */
  start_age: number
  /** 阳男阴女为真 */
  forward: boolean
  steps: MajorLimit[]
}

/** 占事结果（对应 app::event::EventCast）。 */
export interface EventCast {
  asked_at: { year: number; month: number; day: number; hour: number; minute: number; tz: number }
  seed: number | null
  question: string | null
  leaves: CastLeaf[]
}

/** 择吉候选日（对应 app::election::Candidate）。 */
export interface ElectionCandidate {
  year: number
  month: number
  day: number
  day_ganzhi: string
  jianchu: string
  grade: 'Huang' | 'Usable' | 'Hei' | 'Avoid'
  grade_label: string
  mansion: string
  pengzu_gan: string
  pengzu_zhi: string
  tianyi: [string, string]
}

/** 择吉结果（对应 app::election::Election）。 */
export interface Election {
  window_start: { year: number; month: number; day: number; hour: number; minute: number; tz: number }
  window_end: { year: number; month: number; day: number; hour: number; minute: number; tz: number }
  category: string | null
  scanned_days: number
  candidates: ElectionCandidate[]
}

/** 方位候选（对应 app::locative::Bearing）。 */
export interface Bearing {
  leaf: string
  element: string
  at: string
  direction: string
  note: string
}

/** 寻方位结果（对应 app::locative::Locative）。 */
export interface Locative {
  asked_at: { year: number; month: number; day: number; hour: number; minute: number; tz: number }
  seed: number | null
  category: string | null
  bearings: Bearing[]
  leaves: CastLeaf[]
}

/** 合盘结果（对应 app::synastry::Synastry::to_json）。 */
export interface Synastry {
  a_name: string
  b_name: string
  a_supplies_b: number
  b_supplies_a: number
  detail: TeamResult
  aspects: CrossAspects
  /** 印度占星八项合婚。逐项给区间——各家判定表不一 */
  ashtakuta: Ashtakuta
}

/** 八项合婚里的一项（对应 jyotish::kuta::KutaScore）。 */
export interface KutaScore {
  kuta: string
  max_points: number
  /** 得分下界（×10，避开浮点） */
  min_tenths: number
  max_tenths: number
  /** 两源是否给出同一个值 */
  settled: boolean
  basis: string
}

/** 八项合婚（对应 jyotish::kuta::Ashtakuta）。 */
export interface Ashtakuta {
  kutas: KutaScore[]
  total_min_tenths: number
  total_max_tenths: number
  max_points: number
  /** 有几项两源不一致——区间宽度全由它们贡献 */
  unsettled_count: number
}

/** 两张本命盘之间的相位一条（对应 astrology::CrossAspect）。 */
export interface CrossAspect {
  /** 甲盘上的星 */
  a: string
  /** 乙盘上的星 */
  b: string
  /** 相位名（合 / 冲 / 拱 / 刑 / 六分） */
  kind: string
  /** 实测夹角（度） */
  angle: number
}

/** 合盘相位全量。哪些算数、容许度多少属取舍，本层出全量。 */
export interface CrossAspects {
  system: string
  /** 容许度（度） */
  orb: number
  count: number
  list: CrossAspect[]
}

/** 太乙时间线上的一年（对应 app::mundane::YearStep）。 */
export interface YearStep {
  year: number
  age: number
  palace: number
  gua: string
  year_in_palace: number
  sancai: string
  yang_dun: boolean
  enters_palace: boolean
}

/** 国运推演（对应 app::mundane::Mundane）。 */
export interface Mundane {
  founded_at: { year: number; month: number; day: number; hour: number; minute: number; tz: number }
  target_year: number
  founding: CastLeaf[]
  annual: YearStep | null
  timeline: YearStep[]
  span: number
}
