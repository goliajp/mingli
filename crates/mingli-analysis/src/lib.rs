//! L3.5 跨叶分析：对引擎的并行 fan-out 输出做信息论统计。
//!
//! 思路：单一输入下每片叶都是确定的，故「相关性」只在**输入分布**上才有意义——取一组时刻样本，
//! 每片叶产出一个分类特征，再算两两**归一化互信息 NMI**：
//!
//! - **A / ⟂ / B 族**同吃共享天文历法层（干支/节气/黄经），故同源量高度相关——极端如 `bazi` 的日支
//!   与 `liuren` 的日支是同一个量，`NMI = 1`。
//! - **C 族**经 `core::sampler`（SplitMix64）把时刻种子雪崩化，输出与历法结构**去相关**，对历法特征
//!   的 NMI 接近 0。
//!
//! 数学部分（[`entropy`] / [`mutual_information`] / [`nmi`]）是纯函数石头，以已知分布精确校验
//! （公平币 H=1 bit、独立变量 MI=0、同变量 MI=H）。
//!
//! 诚实注：互信息的有限样本估计**正偏**（样本越稀偏高），故本层用 NMI + 充足样本，且「C 族低相关」
//! 这类结论是保守的（真值更低）。

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::similar_names,
    clippy::float_cmp,
    reason = "信息论计数→f64 精度损失可忽略；px/py 等为数学惯用命名；intern 编码窄化受控；对 0.0/identity 的精确比较是有意为之"
)]

use mingli_contract::{CastingEngine, Family, Gender, Query};
use mingli_engine::cast_all_detailed;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

// 计数表用 BTreeMap 而非 HashMap：浮点加法不结合，`a + b + c` 换个次序就换个末位。
// HashMap 的遍历次序由每进程随机播种的 hasher 决定，于是同一份输入换个进程跑出的
// 熵与互信息在最低位上不一样——`/api/analysis` 的 NMI 矩阵因此两次运行不逐字节相同。
// 按键有序遍历把求和次序钉死，结果才是「可复现」的。

// ===================== 信息论（石头） =====================

/// 香农熵（bit / log₂）。输入为分类样本的整数编码。
#[must_use]
pub fn entropy(xs: &[i64]) -> f64 {
    let n = xs.len() as f64;
    if n == 0.0 {
        return 0.0;
    }
    let mut c: BTreeMap<i64, u64> = BTreeMap::new();
    for &x in xs {
        *c.entry(x).or_insert(0) += 1;
    }
    -c.values()
        .map(|&k| {
            let p = k as f64 / n;
            p * p.log2()
        })
        .sum::<f64>()
}

/// 互信息 `I(X;Y)`（bit）。`xs`/`ys` 等长，按位置配对。
#[must_use]
pub fn mutual_information(xs: &[i64], ys: &[i64]) -> f64 {
    let n = xs.len() as f64;
    if n == 0.0 || xs.len() != ys.len() {
        return 0.0;
    }
    let mut px: BTreeMap<i64, u64> = BTreeMap::new();
    let mut py: BTreeMap<i64, u64> = BTreeMap::new();
    let mut pxy: BTreeMap<(i64, i64), u64> = BTreeMap::new();
    for (&x, &y) in xs.iter().zip(ys) {
        *px.entry(x).or_insert(0) += 1;
        *py.entry(y).or_insert(0) += 1;
        *pxy.entry((x, y)).or_insert(0) += 1;
    }
    let mut mi = 0.0;
    for (&(x, y), &kxy) in &pxy {
        let p_xy = kxy as f64 / n;
        let p_x = px[&x] as f64 / n;
        let p_y = py[&y] as f64 / n;
        mi += p_xy * (p_xy / (p_x * p_y)).log2();
    }
    mi.max(0.0)
}

/// 归一化互信息 `NMI = I(X;Y) / √(H(X)·H(Y))`，值域 `[0,1]`。任一边熵为 0（常量）则 0。
#[must_use]
pub fn nmi(xs: &[i64], ys: &[i64]) -> f64 {
    let hx = entropy(xs);
    let hy = entropy(ys);
    if hx <= 0.0 || hy <= 0.0 {
        return 0.0;
    }
    (mutual_information(xs, ys) / (hx * hy).sqrt()).clamp(0.0, 1.0)
}

// ===================== 逐叶特征 =====================

/// 取某叶 JSON 盘里一个**低基数分类特征**（用于跨叶 NMI）。NMI 对重标号不变，故编码任意。
#[must_use]
pub fn feature(id: &str, c: &Value) -> Option<String> {
    let s = |v: &Value| v.as_str().map(str::to_string);
    let n = |v: &Value| v.as_i64().map(|x| x.to_string());
    match id {
        "bazi" => s(&c["day"]["branch"]),       // 日支(12)
        "ziwei" => s(&c["ming_branch"]),         // 命宫支(12)
        "astrology" => s(&c["planets"][0]["sign"]), // 太阳座(12)
        "jyotish" => s(&c["grahas"][1]["nakshatra_name"]), // 月亮 nakshatra(27)
        "qizhengsiyu" => s(&c["mansion_name"]),  // 28 宿值日
        "meihua" => s(&c["primary_upper"]),      // 上卦(8)
        "xiaoliuren" => n(&c["hour_pos"]),       // 时神位(6)
        "zeri" => s(&c["jianchu"]),              // 建除(12)
        "maya" => n(&c["tzolkin_number"]),       // 13
        "pawukon" => s(&c["pancawara"]),         // 5
        "mahabote" => s(&c["house"]),            // 7
        "tibetan" => s(&c["animal"]),            // 生肖(12)
        "qimen" => n(&c["setup"]["ju"]),         // 局(9)
        "taiyi" => n(&c["taiyi"]["palace"]),     // 八宫(8)
        "numerology" => n(&c["life_path"]),      // 生命数(~11)
        "liuren" => n(&c["day_branch"]),         // 日支(12)——与 bazi 同源
        "yijing" => s(&c["primary_lower"]),      // C：下卦(8)
        "geomancy" => n(&c["judge"]),            // C：法官(16)
        "sikidy" => n(&c["seer"]),               // C：创世者(16)
        "ifa" => n(&c["left"]),                  // C：左 figure(16)
        "tarot" => c["cards"][0]["index"].as_i64().map(|i| (i % 13).to_string()), // C：粗化(13)
        _ => None,
    }
}

/// 该叶所取特征的中文说明（展示用）。
#[must_use]
pub fn feature_label(id: &str) -> &'static str {
    match id {
        "bazi" | "liuren" => "日支", "ziwei" => "命宫支", "astrology" => "太阳星座",
        "jyotish" => "月宿(nakshatra)", "qizhengsiyu" => "28 宿值日", "meihua" => "上卦",
        "xiaoliuren" => "时神位", "zeri" => "建除", "maya" => "Tzolkʼin 数", "pawukon" => "Pancawara",
        "mahabote" => "本命宫", "tibetan" => "生肖", "qimen" => "局数", "taiyi" => "太乙宫",
        "numerology" => "生命灵数", "yijing" => "下卦",
        "geomancy" => "法官", "sikidy" => "创世者", "ifa" => "左 figure", "tarot" => "首牌（粗化）", _ => "",
    }
}

// ===================== 跨叶分析 =====================

/// 单叶在样本上的统计。
#[derive(Debug, Clone, Serialize)]
pub struct LeafStat {
    /// 叶 id。
    pub id: String,
    /// 显示名。
    pub name: String,
    /// 家族。
    pub family: Family,
    /// 所取特征说明。
    pub feature: &'static str,
    /// 样本熵（bit）。
    pub entropy: f64,
    /// 不同取值数。
    pub distinct: usize,
}

/// 跨叶分析结果：每叶统计 + 两两 NMI 矩阵（与 `leaves` 同序）。
#[derive(Debug, Clone, Serialize)]
pub struct Analysis {
    /// 样本数。
    pub n: usize,
    /// 每叶统计。
    pub leaves: Vec<LeafStat>,
    /// `nmi[i][j]` = 第 i、j 叶特征的归一化互信息。
    pub nmi: Vec<Vec<f64>>,
}

/// 在一组查询样本上做跨叶分析。
#[must_use]
pub fn cross_leaf(reg: &[Box<dyn CastingEngine>], queries: &[Query]) -> Analysis {
    let detailed: Vec<_> = queries.iter().map(|q| cast_all_detailed(reg, q)).collect();
    if detailed.is_empty() {
        return Analysis { n: 0, leaves: vec![], nmi: vec![] };
    }
    let order = &detailed[0];
    let k = order.len();
    // 每叶一列：把分类特征 intern 成整数编码。
    let mut cols: Vec<Vec<i64>> = vec![Vec::with_capacity(detailed.len()); k];
    let mut interns: Vec<HashMap<String, i64>> = vec![HashMap::new(); k];
    for row in &detailed {
        for (li, leaf) in row.iter().enumerate() {
            let f = feature(leaf.id, &leaf.chart).unwrap_or_else(|| "∅".to_string());
            let map = &mut interns[li];
            let next = map.len() as i64;
            let code = *map.entry(f).or_insert(next);
            cols[li].push(code);
        }
    }
    let leaves: Vec<LeafStat> = order
        .iter()
        .enumerate()
        .map(|(li, leaf)| LeafStat {
            id: leaf.id.to_string(),
            name: leaf.name.to_string(),
            family: leaf.family,
            feature: feature_label(leaf.id),
            entropy: entropy(&cols[li]),
            distinct: interns[li].len(),
        })
        .collect();
    let mut mat = vec![vec![0.0; k]; k];
    for i in 0..k {
        mat[i][i] = if leaves[i].entropy > 0.0 { 1.0 } else { 0.0 };
        for j in (i + 1)..k {
            let v = nmi(&cols[i], &cols[j]);
            mat[i][j] = v;
            mat[j][i] = v;
        }
    }
    Analysis { n: detailed.len(), leaves, nmi: mat }
}

/// 生成采样网格：`start..=end` 年 × 12 月 × 每月 9、24 日（午时，北京坐标）。
#[must_use]
pub fn sample_grid(start_year: i32, end_year: i32) -> Vec<Query> {
    let mut v = Vec::new();
    for year in start_year..=end_year {
        for month in 1..=12u32 {
            for day in [9u32, 24] {
                v.push(Query {
                    year,
                    month,
                    day,
                    hour: 12,
                    minute: 0,
                    tz: 8.0,
                    gender: Some(Gender::Male),
                    latitude: Some(39.9),
                    longitude: Some(116.4),
                    seed: None,
                    name: Some("样本".to_string()),
                    schools: std::collections::BTreeMap::new(),
                });
            }
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use mingli_registry::registry;

    #[test]
    fn a_count_over_the_same_data_lands_on_the_same_bits() {
        // 这条不是「值对不对」，是「同一份输入换个进程跑还是不是同一个数」。
        //
        // 熵与互信息都是一串浮点相加，而浮点加法不结合：换个求和次序就换个末位。
        // 计数表若用 HashMap，遍历次序随每进程的随机 hasher 变，于是同样的输入
        // 每跑一次 NMI 矩阵就在末位上抖一下——`/api/analysis` 曾经就是这样，
        // 同一个二进制连跑两遍 body 的 md5 都不同。
        //
        // 所以这里钉的是**位模式**：近似比较（`< 1e-9`）看不见这种抖动，
        // 而正是这种抖动让「可复现」这句话不成立。数值本身由下面几条已有用例把关。
        let xs = [3_i64, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9, 3, 2, 3, 8, 4];
        let ys = [1_i64, 1, 2, 3, 5, 8, 3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7];
        assert_eq!(entropy(&xs).to_bits(), 0x4008_5F1B_9754_E0A2);
        assert_eq!(mutual_information(&xs, &ys).to_bits(), 0x4000_11CE_AB0F_4C06);
        assert_eq!(nmi(&xs, &ys).to_bits(), 0x3FE5_5183_AEF9_199D);
    }

    #[test]
    fn entropy_known() {
        assert!((entropy(&[0, 0, 1, 1]) - 1.0).abs() < 1e-9); // 公平币 1 bit
        assert_eq!(entropy(&[5, 5, 5]), 0.0); // 常量 0
        assert!((entropy(&[0, 1, 2, 3]) - 2.0).abs() < 1e-9); // 4 等概 = 2 bit
        assert_eq!(entropy(&[]), 0.0);
    }

    #[test]
    fn mi_known() {
        // 同变量：MI = H。
        let x = [0, 1, 2, 3, 0, 1, 2, 3];
        assert!((mutual_information(&x, &x) - entropy(&x)).abs() < 1e-9);
        // 独立：x=[0,0,1,1]， y=[0,1,0,1] 联合均匀 → MI=0。
        assert!(mutual_information(&[0, 0, 1, 1], &[0, 1, 0, 1]).abs() < 1e-9);
        // NMI：同变量=1，独立=0。
        assert!((nmi(&x, &x) - 1.0).abs() < 1e-9);
        assert_eq!(nmi(&[0, 0, 1, 1], &[0, 1, 0, 1]), 0.0);
        // 常量边 → NMI 0。
        assert_eq!(nmi(&[7, 7, 7], &[0, 1, 2]), 0.0);
        // 长度不等 → MI 0。
        assert_eq!(mutual_information(&[0, 1], &[0]), 0.0);
    }

    #[test]
    fn feature_extraction_smoke() {
        let q = sample_grid(1990, 1990);
        let leaves = cast_all_detailed(&registry(), &q[0]);
        for leaf in &leaves {
            let f = feature(leaf.id, &leaf.chart);
            assert!(f.is_some(), "{} 应能取到特征", leaf.id);
            assert!(!feature_label(leaf.id).is_empty());
        }
    }

    #[test]
    fn edge_cases() {
        // 空样本 → 空分析。
        let a = cross_leaf(&registry(), &[]);
        assert_eq!(a.n, 0);
        assert!(a.leaves.is_empty() && a.nmi.is_empty());
        // 未知叶 id → 无特征 / 空标签。
        assert_eq!(feature("nope", &serde_json::json!({})), None);
        assert_eq!(feature_label("nope"), "");
    }

    #[test]
    fn cross_leaf_validates_thesis() {
        // 30 年 × 12 月 × 2 日 = 720 样本。
        let a = cross_leaf(&registry(), &sample_grid(1980, 2009));
        assert_eq!(a.n, 720);
        let idx = |id: &str| a.leaves.iter().position(|l| l.id == id).unwrap();
        let (bazi, liuren) = (idx("bazi"), idx("liuren"));
        let yijing = idx("yijing");
        let geomancy = idx("geomancy");

        // 对角 = 1（自相关）。
        for i in 0..a.leaves.len() {
            assert!((a.nmi[i][i] - 1.0).abs() < 1e-9);
        }
        // A/⟂ 同源：bazi 日支 ≡ liuren 日支 → NMI = 1。
        assert!(a.nmi[bazi][liuren] > 0.999, "同源日支应 NMI≈1，实得 {}", a.nmi[bazi][liuren]);
        // C 族经 PRNG 与历法去相关：yijing/geomancy 对 bazi 日支 NMI 远低于同源。
        assert!(a.nmi[bazi][yijing] < 0.3, "C(yijing) 应与日支低相关，实得 {}", a.nmi[bazi][yijing]);
        assert!(a.nmi[bazi][geomancy] < 0.3, "C(geomancy) 应与日支低相关，实得 {}", a.nmi[bazi][geomancy]);
        // 且 C-历法 显著低于 同源对。
        assert!(a.nmi[bazi][yijing] < a.nmi[bazi][liuren]);
    }
}
