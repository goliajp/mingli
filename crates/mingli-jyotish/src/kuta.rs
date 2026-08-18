//! Ashtakuta（八项合婚）：以两人月宿与月宫比对，满分 36。
//!
//! # 一件必须先说清的事：36 分不是一个确定的数
//!
//! 八项的**名目与权重**多源一致（Varna 1、Vashya 2、Tara 3、Yoni 4、Graha Maitri 5、
//! Gana 6、Bhakoot 7、Nadi 8，合 36），但**逐项的判定表各家并不相同**。
//! 本叶取两份互相独立的公布表逐格对过（Saravali 的 Asta Koota 分项页
//! <https://saravali.github.io/astrology/koota_nadi.html> 一系，与
//! freehoroscopesonline 的同名分项页），结果是：
//!
//! | 表 | 两源比对 |
//! |---|---|
//! | Varna 4×4 | 16/16 全同 |
//! | Gana 3×3 | 互为转置——内容相同，但两家都自称「行=新娘」，方向说法有出入 |
//! | Nadi、Bhakoot、八项权重 | 全同 |
//! | 27 宿 → 14 兽 | 逐条全同 |
//! | **Vashya 5×5** | **8/25 不同** |
//! | **Yoni 14×14** | 结构同（对角恒 4、同样那 14 个零格），**中段 72/196 不同**（69 格差 1） |
//!
//! 于是本叶**不出一个总分，出一个区间**：每项给 `(min, max)`，两源一致时二者相等。
//! 这不是取巧——把两派中的一派静默选下来，得到的那个「36 分制得几分」会随选谁而变，
//! 而读的人无从知道。区间把这件事摆在明处：区间宽度就是「各家分歧对结论的影响有多大」。
//!
//! # 各项判据来源
//!
//! - **Varna**：月宫五行定四姓（水=婆罗门、火=刹帝利、风=吠舍、土=首陀罗），
//!   男姓不低于女姓得 1 分，否则 0。两源同表
//! - **Vashya**：月宫分五类，两源的类归属相同而相性矩阵有出入，故出区间
//! - **Tara**：两人月宿相隔数各除以 9，余数落在 Vipat(3)/Pratyak(5)/Vadha(7) 为凶。
//!   两向皆吉 3 分、一吉一凶 1.5、皆凶 0
//! - **Yoni**：27 宿配 14 兽，同兽 4、死敌 0，中段两源不一故出区间
//! - **Graha Maitri**：两人月宫主星的天然友敌。友友 5、友中 4、中中 3、友敌 2、中敌 1、敌敌 0
//! - **Gana**：27 宿配三性（天/人/罗刹）
//! - **Bhakoot**：两人月宫相隔位次，1/3/4/7/10/11 位为吉得 7，2/5/6/8/9/12 位为凶得 0
//! - **Nadi**：27 宿配三脉（Adi/Madhya/Antya），**同脉得 0、异脉得满 8**

use serde::Serialize;

/// 一项的得分。两源一致时 `min == max`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KutaScore {
    /// 项名。
    pub kuta: &'static str,
    /// 本项满分。
    pub max_points: u32,
    /// 得分下界（×10，避免浮点：Vashya 有 0.5 分档）。
    pub min_tenths: u32,
    /// 得分上界（×10）。两源一致时等于 `min_tenths`。
    pub max_tenths: u32,
    /// 两源是否给出同一个值。
    pub settled: bool,
    /// 判据说明（本项据什么得此分）。
    pub basis: String,
}

/// 八项合起来的结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Ashtakuta {
    /// 八项逐项。
    pub kutas: Vec<KutaScore>,
    /// 总分下界（×10，满分 360）。
    pub total_min_tenths: u32,
    /// 总分上界（×10）。
    pub total_max_tenths: u32,
    /// 满分（恒 36）。
    pub max_points: u32,
    /// 八项里有几项两源不一致——区间宽度全由它们贡献。
    pub unsettled_count: u32,
}

/// 27 宿配三性：0=Deva 天、1=Manushya 人、2=Rakshasa 罗刹。
///
/// 两源逐宿相同（Saravali `koota_gana`、freehoroscopesonline `ganakoota`）。
pub const GANA: [u8; 27] = [
    0, 1, 2, 1, 0, 1, 0, 0, 2, // Ashwini..Ashlesha
    2, 1, 1, 0, 2, 0, 2, 0, 2, // Magha..Jyeshtha
    2, 1, 1, 0, 2, 2, 1, 1, 0, // Mula..Revati
];

/// 27 宿配三脉：0=Adi 风、1=Madhya 胆、2=Antya 痰。两源逐宿相同。
pub const NADI: [u8; 27] = [
    0, 1, 2, 2, 1, 0, 0, 1, 2, // Ashwini..Ashlesha
    2, 1, 0, 0, 1, 2, 2, 1, 0, // Magha..Jyeshtha
    0, 1, 2, 2, 1, 0, 0, 1, 2, // Mula..Revati
];

/// 14 兽名（索引即 [`YONI`] 的取值）。
pub const YONI_ANIMALS: [&str; 14] = [
    "Horse", "Elephant", "Sheep", "Serpent", "Dog", "Cat", "Rat",
    "Cow", "Buffalo", "Tiger", "Deer", "Monkey", "Mongoose", "Lion",
];

/// 27 宿配 14 兽。两源逐宿相同。
pub const YONI: [u8; 27] = [
    0, 1, 2, 3, 3, 4, 5, 2, 5, // Ashwini..Ashlesha
    6, 6, 7, 8, 9, 8, 9, 10, 10, // Magha..Jyeshtha
    4, 11, 12, 11, 13, 0, 13, 7, 1, // Mula..Revati
];

/// 死敌兽对：这七对两源完全一致，各得 0 分。
pub const YONI_SWORN_ENEMIES: [(u8, u8); 7] =
    [(0, 8), (1, 13), (2, 11), (3, 12), (4, 10), (5, 6), (7, 9)];

/// 12 宫主星（0=白羊…11=双鱼）：火星/金星/水星/月亮/太阳/水星/金星/火星/木星/土星/土星/木星。
pub const RASI_LORD: [&str; 12] = [
    "Mars", "Venus", "Mercury", "Moon", "Sun", "Mercury",
    "Venus", "Mars", "Jupiter", "Saturn", "Saturn", "Jupiter",
];

/// 七曜天然友敌下的 Graha Maitri 得分（行=女方宫主，列=男方宫主）。
///
/// 序：日月火水木金土。表出自 freehoroscopesonline 的 `grahamaitrikoota`，
/// 其数值与 Saravali 给的分档（友友 5 / 友中 4 / 中中 3 / 友敌 2 / 中敌 1 / 敌敌 0）相合，
/// 且与 BPHS 的推导规则（自本星曜庙位起第 2/4/5/8/9/12 宫之主为友、余为敌、
/// 一友一敌为中）方向一致——两处独立地说同一件事。
pub const GRAHA_MAITRI: [[u32; 7]; 7] = [
    [50, 50, 50, 40, 50, 0, 0],    // Sun
    [50, 50, 40, 10, 40, 5, 5],    // Moon
    [50, 40, 50, 5, 50, 30, 5],    // Mars
    [40, 10, 5, 50, 5, 50, 40],    // Mercury
    [50, 40, 50, 5, 50, 5, 30],    // Jupiter
    [0, 5, 30, 50, 5, 50, 50],     // Venus
    [0, 5, 5, 40, 30, 50, 50],     // Saturn
];

/// 月宫配四姓：0=婆罗门（水）、1=刹帝利（火）、2=吠舍（风）、3=首陀罗（土）。两源同表。
pub const VARNA: [u8; 12] = [1, 3, 2, 0, 1, 3, 2, 0, 1, 3, 2, 0];

/// 月宫配五类：0=四足、1=人、2=水生、3=狮（Vanachara）、4=虫（Keeta）。
///
/// 摩羯前半属四足、后半属水生——本叶按整宫取四足，并在 `profile()` 声明这一处简化。
pub const VASHYA: [u8; 12] = [0, 0, 1, 2, 3, 1, 1, 4, 0, 0, 1, 2];

/// Vashya 相性：两源的取值（×10）。二者不同处即出区间。
const VASHYA_A: [[u32; 5]; 5] = [
    [20, 0, 0, 5, 0],
    [10, 20, 10, 5, 10],
    [5, 10, 20, 10, 10],
    [0, 0, 0, 20, 0],
    [10, 10, 10, 0, 20],
];
const VASHYA_B: [[u32; 5]; 5] = [
    [20, 10, 10, 15, 10],
    [10, 20, 15, 0, 10],
    [10, 15, 20, 10, 10],
    [0, 0, 0, 20, 0],
    [10, 10, 10, 0, 20],
];

/// Gana 相性（行=女、列=男，×10）。两源内容相同，只是行列方向说法有出入，
/// 故此处取一致的那个读法：天/人相配得满，罗刹与天人相配失分。
const GANA_MATRIX: [[u32; 3]; 3] = [
    [60, 60, 0],  // 女 Deva
    [50, 60, 0],  // 女 Manushya
    [10, 0, 60],  // 女 Rakshasa
];

fn yoni_bounds(a: u8, b: u8) -> (u32, u32) {
    if a == b {
        return (40, 40); // 同兽：两源皆 4
    }
    let sworn = YONI_SWORN_ENEMIES
        .iter()
        .any(|&(x, y)| (x == a && y == b) || (x == b && y == a));
    if sworn {
        return (0, 0); // 死敌：两源皆 0，且七对完全一致
    }
    (10, 30) // 中段：两源在 72/196 格上不一，只定得下「在 1..3 之间」
}

/// 两人相隔的宿数（1 起），用于 Tara。
fn tara_step(from: usize, to: usize) -> usize {
    (to + 27 - from) % 27 + 1
}

fn tara_bad(step: usize) -> bool {
    // 除以 9 的余数落在 Vipat(3) / Pratyak(5) / Vadha(7) 为凶；余 0 视作第 9 位（吉）
    matches!(step % 9, 3 | 5 | 7)
}

fn score(kuta: &'static str, max_points: u32, lo: u32, hi: u32, basis: String) -> KutaScore {
    KutaScore { kuta, max_points, min_tenths: lo, max_tenths: hi, settled: lo == hi, basis }
}

/// 算八项。`bride`/`groom` 各给 (月宿序 0..27, 月宫序 0..12)。
#[must_use]
pub fn ashtakuta(bride: (usize, usize), groom: (usize, usize)) -> Ashtakuta {
    let (bn, br) = bride;
    let (gn, gr) = groom;
    let mut kutas = Vec::with_capacity(8);

    // 1 Varna：男姓不低于女姓则 1 分
    let (vb, vg) = (VARNA[br], VARNA[gr]);
    kutas.push(score(
        "Varna",
        1,
        if vg <= vb { 10 } else { 0 },
        if vg <= vb { 10 } else { 0 },
        format!("女属第 {vb} 姓、男属第 {vg} 姓（0 婆罗门…3 首陀罗），男不低于女即得分"),
    ));

    // 2 Vashya：两源矩阵有出入 → 区间
    let (a, b) = (VASHYA_A[usize::from(VASHYA[br])][usize::from(VASHYA[gr])],
                  VASHYA_B[usize::from(VASHYA[br])][usize::from(VASHYA[gr])]);
    kutas.push(score(
        "Vashya",
        2,
        a.min(b),
        a.max(b),
        format!("女宫属第 {} 类、男宫属第 {} 类；两源此格作 {a}/10 与 {b}/10", VASHYA[br], VASHYA[gr]),
    ));

    // 3 Tara：两向各判吉凶
    let (s1, s2) = (tara_step(bn, gn), tara_step(gn, bn));
    let bad = u32::from(tara_bad(s1)) + u32::from(tara_bad(s2));
    let t = match bad {
        0 => 30,
        1 => 15,
        _ => 0,
    };
    kutas.push(score("Tara", 3, t, t, format!("两向相隔 {s1} / {s2} 宿，凶向 {bad} 个")));

    // 4 Yoni：同兽 4、死敌 0，中段两源不一 → 区间
    let (ylo, yhi) = yoni_bounds(YONI[bn], YONI[gn]);
    kutas.push(score(
        "Yoni",
        4,
        ylo,
        yhi,
        format!("女宿属{}、男宿属{}", YONI_ANIMALS[usize::from(YONI[bn])], YONI_ANIMALS[usize::from(YONI[gn])]),
    ));

    // 5 Graha Maitri：两宫主星的天然友敌
    let idx = |lord: &str| ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"]
        .iter()
        .position(|x| *x == lord)
        .unwrap_or(0);
    let g = GRAHA_MAITRI[idx(RASI_LORD[br])][idx(RASI_LORD[gr])];
    kutas.push(score(
        "Graha Maitri",
        5,
        g,
        g,
        format!("女宫主 {}、男宫主 {}", RASI_LORD[br], RASI_LORD[gr]),
    ));

    // 6 Gana
    let gm = GANA_MATRIX[usize::from(GANA[bn])][usize::from(GANA[gn])];
    kutas.push(score("Gana", 6, gm, gm, format!("女宿第 {} 性、男宿第 {} 性（0 天 1 人 2 罗刹）", GANA[bn], GANA[gn])));

    // 7 Bhakoot：两宫相隔位次
    let d1 = (gr + 12 - br) % 12 + 1;
    let d2 = (br + 12 - gr) % 12 + 1;
    let ok = |d: usize| matches!(d, 1 | 3 | 4 | 7 | 10 | 11);
    let bh = if ok(d1) && ok(d2) { 70 } else { 0 };
    kutas.push(score("Bhakoot", 7, bh, bh, format!("两宫相隔 {d1} / {d2} 位")));

    // 8 Nadi：同脉 0、异脉满分
    let nd = if NADI[bn] == NADI[gn] { 0 } else { 80 };
    kutas.push(score("Nadi", 8, nd, nd, format!("女宿第 {} 脉、男宿第 {} 脉", NADI[bn], NADI[gn])));

    let total_min_tenths = kutas.iter().map(|k| k.min_tenths).sum();
    let total_max_tenths = kutas.iter().map(|k| k.max_tenths).sum();
    let unsettled_count = u32::try_from(kutas.iter().filter(|k| !k.settled).count()).unwrap_or(0);
    Ashtakuta { kutas, total_min_tenths, total_max_tenths, max_points: 36, unsettled_count }
}
