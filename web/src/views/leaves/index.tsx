// 按叶 id 分发到各叶的整页组件。
//
// 这个 switch 是唯一知道「有哪些叶」的地方——加一片叶 = 加一个文件 + 这里加一行。
import type { BaziChart, CastLeaf, JyotishChart, QizhengsiyuChart, ZiweiChart } from '../../types'

import { BaziView } from './Bazi'
import { ZiweiView } from './Ziwei'
import { Meihua, type MeihuaChart } from './Meihua'
import { Xiaoliuren, type XiaoChart } from './Xiaoliuren'
import { Zeri, type ZeriChart } from './Zeri'
import { Maya, type MayaChart } from './Maya'
import { Pawukon, type PawukonChart } from './Pawukon'
import { Mahabote, type MahaboteChart } from './Mahabote'
import { Tibetan, type TibetanChart } from './Tibetan'
import { Astrology, type AstroChart } from './Astrology'
import { Jyotish } from './Jyotish'
import { Qizhengsiyu } from './Qizhengsiyu'
import { Yijing, type YijingChart } from './Yijing'
import { Geomancy, type GeomancyChart } from './Geomancy'
import { Sikidy, type SikidyChart } from './Sikidy'
import { Ifa, type IfaChart } from './Ifa'
import { Tarot, type TarotChart } from './Tarot'
import { Numerology, type NumChart } from './Numerology'
import { Liuren, type LiurenChart } from './Liuren'
import { Qimen, type QimenChart } from './Qimen'
import { Taiyi, type TaiyiChart } from './Taiyi'

export function LeafChart({ leaf }: { leaf: CastLeaf }) {
  const ch = leaf.chart
  switch (leaf.id) {
    case 'bazi': return <BaziView c={ch as BaziChart} />
    case 'ziwei': return <ZiweiView c={ch as ZiweiChart} />
    case 'astrology': return <Astrology c={ch as AstroChart} />
    case 'jyotish': return <Jyotish c={ch as JyotishChart} />
    case 'qizhengsiyu': return <Qizhengsiyu c={ch as QizhengsiyuChart} />
    case 'yijing': return <Yijing c={ch as YijingChart} />
    case 'geomancy': return <Geomancy c={ch as GeomancyChart} />
    case 'sikidy': return <Sikidy c={ch as SikidyChart} />
    case 'ifa': return <Ifa c={ch as IfaChart} />
    case 'tarot': return <Tarot c={ch as TarotChart} />
    case 'meihua': return <Meihua c={ch as MeihuaChart} />
    case 'xiaoliuren': return <Xiaoliuren c={ch as XiaoChart} />
    case 'zeri': return <Zeri c={ch as ZeriChart} />
    case 'maya': return <Maya c={ch as MayaChart} />
    case 'pawukon': return <Pawukon c={ch as PawukonChart} />
    case 'mahabote': return <Mahabote c={ch as MahaboteChart} />
    case 'liuren': return <Liuren c={ch as LiurenChart} />
    case 'qimen': return <Qimen c={ch as QimenChart} />
    case 'taiyi': return <Taiyi c={ch as TaiyiChart} />
    case 'tibetan': return <Tibetan c={ch as TibetanChart} />
    case 'numerology': return <Numerology c={ch as NumChart} />
    default: return <pre className="jv-fallback">{JSON.stringify(ch, null, 1)}</pre>
  }
}
