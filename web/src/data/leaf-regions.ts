// 叶 → 文化地域分组与配色。

// 术数按文化／地域分组（用户视角）
export const LEAF_REGION: Record<string, string> = {
  bazi: '中华', ziwei: '中华', yijing: '中华', meihua: '中华', xiaoliuren: '中华',
  zeri: '中华', liuren: '中华', qimen: '中华', taiyi: '中华', qizhengsiyu: '中华',
  astrology: '西洋', tarot: '西洋', numerology: '西洋',
  jyotish: '南亚',
  geomancy: '中东',
  ifa: '非洲', sikidy: '非洲',
  maya: '美洲',
  tibetan: '喜马拉雅·东南亚', pawukon: '喜马拉雅·东南亚', mahabote: '喜马拉雅·东南亚',
}

export const REGION_COLOR: Record<string, string> = {
  中华: '#c9a24a', 西洋: '#5fb3bf', 中东: '#a98cff',
  非洲: '#5fb06a', 美洲: '#e0584c', '喜马拉雅·东南亚': '#d98c5f',
  南亚: '#d24a8c',
}

export const regionOf = (id: string) => LEAF_REGION[id] ?? '其他'

export const colorOf = (id: string) => REGION_COLOR[regionOf(id)] ?? '#888'
