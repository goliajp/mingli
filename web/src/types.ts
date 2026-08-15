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
}

export interface Mahadasha {
  lord: string
  years: number
  effective_years: number
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
}
