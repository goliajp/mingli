//! 定局：由节气定阴阳遁与三元局数。
//!
//! 冬至→芒种阳遁、夏至→大雪阴遁；三元由符头地支定。72 局常数表 6 源零冲突，
//! 且满足结构不变量——阳遁「中元 = 上元 + 6、下元 = 上元 + 3」，阴遁「−6 / −3」。

#[cfg(feature = "serde")]
use serde::Serialize;

/// 24 节气名，按 `floor(λ/15)` 索引（春分=0 … 惊蛰=23）。
pub const SOLAR_TERMS: [&str; 24] = [
    "春分", "清明", "谷雨", "立夏", "小满", "芒种", "夏至", "小暑", "大暑", "立秋", "处暑", "白露",
    "秋分", "寒露", "霜降", "立冬", "小雪", "大雪", "冬至", "小寒", "大寒", "立春", "雨水", "惊蛰",
];

/// 各节气的三元局数 `[上元, 中元, 下元]`（1..9），按 [`SOLAR_TERMS`] 同序。
pub const YUAN_JU: [[u8; 3]; 24] = [
    [3, 9, 6], // 春分
    [4, 1, 7], // 清明
    [5, 2, 8], // 谷雨
    [4, 1, 7], // 立夏
    [5, 2, 8], // 小满
    [6, 3, 9], // 芒种
    [9, 3, 6], // 夏至
    [8, 2, 5], // 小暑
    [7, 1, 4], // 大暑
    [2, 5, 8], // 立秋
    [1, 4, 7], // 处暑
    [9, 3, 6], // 白露
    [7, 1, 4], // 秋分
    [6, 9, 3], // 寒露
    [5, 8, 2], // 霜降
    [6, 9, 3], // 立冬
    [5, 8, 2], // 小雪
    [4, 7, 1], // 大雪
    [1, 7, 4], // 冬至
    [2, 8, 5], // 小寒
    [3, 9, 6], // 大寒
    [8, 5, 2], // 立春
    [9, 6, 3], // 雨水
    [1, 7, 4], // 惊蛰
];

/// 三元。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Yuan {
    /// 上元（符头地支 子午卯酉）。
    Upper,
    /// 中元（符头地支 寅申巳亥）。
    Middle,
    /// 下元（符头地支 辰戌丑未）。
    Lower,
}

impl Yuan {
    /// 在 `[上,中,下]` 三元数组中的下标。
    #[must_use]
    pub fn index(self) -> usize {
        match self {
            Yuan::Upper => 0,
            Yuan::Middle => 1,
            Yuan::Lower => 2,
        }
    }
    /// 三元名。
    #[must_use]
    pub fn name(self) -> &'static str {
        ["上元", "中元", "下元"][self.index()]
    }
}

/// 由地支（0..11）定三元：子午卯酉=上元、寅申巳亥=中元、辰戌丑未=下元。
#[must_use]
pub fn yuan_of_branch(branch: u8) -> Yuan {
    match branch % 3 {
        0 => Yuan::Upper,  // 子卯午酉
        2 => Yuan::Middle, // 寅巳申亥
        _ => Yuan::Lower,  // 丑辰未戌
    }
}

/// 节气下标（春分=0 … 惊蛰=23），由太阳视黄经 `floor(λ/15)`。
#[must_use]
pub fn solar_term_index(sun_longitude: f64) -> usize {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "λ/15 ∈ 0..24"
    )]
    let k = (sun_longitude.rem_euclid(360.0) / 15.0).floor() as usize;
    k % 24
}

/// 是否阳遁：冬至→芒种（节气下标 18..24 或 0..6）为阳遁，余为阴遁。
#[must_use]
pub fn is_yang_dun(term_index: usize) -> bool {
    term_index >= 18 || term_index <= 5
}

/// 定局结果：节气、阴阳遁、三元、局数。
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Setup {
    /// 节气下标（春分=0 … 惊蛰=23）。
    pub term_index: usize,
    /// 节气名。
    pub term: &'static str,
    /// 是否阳遁（否则阴遁）。
    pub yang_dun: bool,
    /// 三元。
    pub yuan: Yuan,
    /// 局数 1..9。
    pub ju: u8,
}

/// 由节气下标与三元定局。
#[must_use]
pub fn solar_term_setup(term_index: usize, yuan: Yuan) -> Setup {
    Setup {
        term_index,
        term: SOLAR_TERMS[term_index],
        yang_dun: is_yang_dun(term_index),
        yuan,
        ju: YUAN_JU[term_index][yuan.index()],
    }
}
