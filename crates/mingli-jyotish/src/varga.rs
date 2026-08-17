//! 分盘（varga）：把一宫三十度切成 n 份，每份再映回十二宫。
//!
//! 十六分盘（Ṣoḍaśavarga）里，本盘 D-1 与九分盘 D-9 另有出处（见 [`crate::navamsa_of`]），
//! 这里收其余十二盘。每盘只有两件事要定：**每份多宽**，与**第一份从哪一宫起、之后怎么数**。
//!
//! 起宫的规矩分四类，本身就是这套体系的结构：
//!
//! - **固定跳步**：D-3 每份跳四宫（本宫 / 第 5 / 第 9），D-4 每份跳三宫（本宫 / 第 4 / 第 7 / 第 10）
//! - **奇偶分起**：D-7 奇宫自本宫起、偶宫自第 7 宫起；D-10 偶宫自第 9 宫起；
//!   D-24 奇宫自狮子、偶宫自巨蟹；D-40 奇宫自白羊、偶宫自天秤
//! - **三性分起**（动 / 固定 / 双体）：D-16 与 D-45 自白羊 / 狮子 / 射手；D-20 自白羊 / 射手 / 狮子
//! - **四大分起**（火 / 土 / 风 / 水）：D-27 自白羊 / 巨蟹 / 天秤 / 摩羯
//!
//! 另有两盘不在此列：D-2 的落宫原典未指定、D-30 的偶宫弧长梵文两可，
//! 两处各家分歧且无多源可依，见本叶 `profile()` 的 🟡 条目。
//!
//! **来源**：以上每一条都由两个彼此独立的开源实现逐条对照确认——
//! `kunjara/jyotish`（PHP，每盘只实现一法，即 Parasara 传统）与
//! `naturalstupid/PyJHora`（Python，每盘并列 3–6 法，此处取其 Parasara 默认）。
//! 两份在十二盘上全部一致（其一用 1 起宫号、其一用 0 起，换算后逐条相同）。
//!
//! **不取的诸法**：Parivritti cyclic / even-reverse、Somanatha alternate、Jaganatha 等
//! 另有传承，PyJHora 每盘并列 3–6 种。本叶只出 Parasara 一系并在 profile 里声明其余，
//! 不静默选边。

use serde::Serialize;

/// 一个分盘。
///
/// 判别式即除数：`Varga::D7 as u32 == 7`，于是「每份多宽」就是 `30 / n`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Varga {
    /// D-3 drekkāṇa：兄弟姊妹。
    D3 = 3,
    /// D-4 chaturthāṃśa：田宅。
    D4 = 4,
    /// D-7 saptāṃśa：子嗣。
    D7 = 7,
    /// D-10 daśāṃśa：事功。
    D10 = 10,
    /// D-12 dvādaśāṃśa：父母。
    D12 = 12,
    /// D-16 ṣoḍaśāṃśa：车乘与安适。
    D16 = 16,
    /// D-20 viṃśāṃśa：修行。
    D20 = 20,
    /// D-24 chaturviṃśāṃśa：学问。
    D24 = 24,
    /// D-27 bhāṃśa / nakṣatrāṃśa：体质与强弱。
    D27 = 27,
    /// D-40 khavedāṃśa：母系所传。
    D40 = 40,
    /// D-45 akṣavedāṃśa：父系所传。
    D45 = 45,
    /// D-60 ṣaṣṭyāṃśa：总述。
    D60 = 60,
}

/// 本模块收的全部分盘，按除数升序。
pub const ALL: [Varga; 12] = [
    Varga::D3,
    Varga::D4,
    Varga::D7,
    Varga::D10,
    Varga::D12,
    Varga::D16,
    Varga::D20,
    Varga::D24,
    Varga::D27,
    Varga::D40,
    Varga::D45,
    Varga::D60,
];

impl Varga {
    /// 除数 n：一宫切成几份。
    #[must_use]
    pub fn divisor(self) -> u32 {
        self as u32
    }

    /// 稳定 id（`"d3"`…`"d60"`）。
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::D3 => "d3",
            Self::D4 => "d4",
            Self::D7 => "d7",
            Self::D10 => "d10",
            Self::D12 => "d12",
            Self::D16 => "d16",
            Self::D20 => "d20",
            Self::D24 => "d24",
            Self::D27 => "d27",
            Self::D40 => "d40",
            Self::D45 => "d45",
            Self::D60 => "d60",
        }
    }

    /// 梵文名（IAST 罗马化）。
    #[must_use]
    pub fn sanskrit_name(self) -> &'static str {
        match self {
            Self::D3 => "drekkāṇa",
            Self::D4 => "chaturthāṃśa",
            Self::D7 => "saptāṃśa",
            Self::D10 => "daśāṃśa",
            Self::D12 => "dvādaśāṃśa",
            Self::D16 => "ṣoḍaśāṃśa",
            Self::D20 => "viṃśāṃśa",
            Self::D24 => "chaturviṃśāṃśa",
            Self::D27 => "bhāṃśa",
            Self::D40 => "khavedāṃśa",
            Self::D45 => "akṣavedāṃśa",
            Self::D60 => "ṣaṣṭyāṃśa",
        }
    }

    /// 本盘主判何事（一词，取通行说法）。
    #[must_use]
    pub fn subject(self) -> &'static str {
        match self {
            Self::D3 => "兄弟姊妹",
            Self::D4 => "田宅",
            Self::D7 => "子嗣",
            Self::D10 => "事功",
            Self::D12 => "父母",
            Self::D16 => "车乘·安适",
            Self::D20 => "修行",
            Self::D24 => "学问",
            Self::D27 => "体质",
            Self::D40 => "母系",
            Self::D45 => "父系",
            Self::D60 => "总述",
        }
    }
}

/// 十二宫的三性：动（chara）/ 固定（sthira）/ 双体（dvisva），自白羊起每三宫一轮。
fn quality(rasi: usize) -> usize {
    rasi % 3
}

/// 十二宫的四大：火 / 土 / 风 / 水，自白羊起每四宫一轮。
fn element(rasi: usize) -> usize {
    rasi % 4
}

/// 某恒星黄经在某分盘上落哪一宫（0=白羊 … 11=双鱼）。
///
/// 三步：本宫、份序、起宫。份序 `part` = 宫内度数整除份宽；
/// 起宫由本盘的规矩定（见模块说明的四类）；落宫 = 起宫 + 份序，模十二。
#[must_use]
pub fn varga_rasi(varga: Varga, sidereal_lon: f64) -> usize {
    let lon = sidereal_lon.rem_euclid(360.0);
    let rasi = (lon / 30.0).floor() as usize % 12;
    let degree = lon - (rasi as f64) * 30.0;
    let n = f64::from(varga.divisor());
    let part = ((degree * n) / 30.0).floor() as usize;
    let odd = rasi.is_multiple_of(2); // 0 起算：白羊是第一宫，属奇

    let base = match varga {
        // 固定跳步：份序每进一格，起宫跳 step 宫
        Varga::D3 => return (rasi + part * 4) % 12,
        Varga::D4 => return (rasi + part * 3) % 12,
        // 奇偶分起
        Varga::D7 => rasi + if odd { 0 } else { 6 },
        Varga::D10 => rasi + if odd { 0 } else { 8 },
        Varga::D24 => {
            if odd {
                4 // 狮子
            } else {
                3 // 巨蟹
            }
        }
        Varga::D40 => {
            if odd {
                0 // 白羊
            } else {
                6 // 天秤
            }
        }
        // 自本宫起顺数
        Varga::D12 | Varga::D60 => rasi,
        // 三性分起
        Varga::D16 | Varga::D45 => match quality(rasi) {
            0 => 0,  // 动 → 白羊
            1 => 4,  // 固定 → 狮子
            _ => 8,  // 双体 → 射手
        },
        Varga::D20 => match quality(rasi) {
            0 => 0,  // 动 → 白羊
            1 => 8,  // 固定 → 射手
            _ => 4,  // 双体 → 狮子
        },
        // 四大分起
        Varga::D27 => match element(rasi) {
            0 => 0,  // 火 → 白羊
            1 => 3,  // 土 → 巨蟹
            2 => 6,  // 风 → 天秤
            _ => 9,  // 水 → 摩羯
        },
    };
    (base + part) % 12
}

/// 一个天体在全部十二个分盘上的落宫。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VargaPositions {
    /// 分盘 id → 落宫索引（0=白羊）。
    pub rasi: std::collections::BTreeMap<&'static str, usize>,
}

/// 算一个恒星黄经在本模块全部分盘上的落宫。
#[must_use]
pub fn all_vargas(sidereal_lon: f64) -> VargaPositions {
    VargaPositions {
        rasi: ALL.iter().map(|v| (v.id(), varga_rasi(*v, sidereal_lon))).collect(),
    }
}
