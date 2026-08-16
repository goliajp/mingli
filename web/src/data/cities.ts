// 出生地城市 → 经纬度与时区。地理常识值，选城市即自动带出坐标。

// 出生地城市 → 纬度/经度/时区（地理坐标，公开常识值；选城市自动换算 Asc/MC 用）
export const REGIONS = ['华北', '华东', '华中', '华南', '西南', '西北', '东北', '港澳台', '海外'] as const

export type Region = typeof REGIONS[number]

export const CITIES: Record<string, { lat: number; lon: number; tz: number; region: Region }> = {
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

export function coordStr(lat?: number, lon?: number, tz?: number): string {
  const la = lat ?? 0, lo = lon ?? 0, t = tz ?? 8
  return `${Math.abs(la).toFixed(2)}°${la >= 0 ? 'N' : 'S'} ${Math.abs(lo).toFixed(2)}°${lo >= 0 ? 'E' : 'W'} · UTC${t >= 0 ? '+' : ''}${t}`
}
