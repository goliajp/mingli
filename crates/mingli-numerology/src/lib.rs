//! L3 叶（D 族 / 确定性）：西洋数字学（numerology）。
//!
//! 数字学是「符号 → 数值（查表 φ）→ 求和（幺半群同态）→ 数字根约化」的最纯范例，骨架全在
//! [`mingli_core::ringhash`]。本 crate 提供两套字母表与四个常见数：
//!
//! - **字母表**：Pythagorean（A=1…I=9 循环，见 [`pythagorean`]）与
//!   Chaldean（1..8，9 为神圣不配字母；按振动而非顺序，见 [`chaldean`]）。
//! - **生命灵数（Life Path）**：出生年月日各自约化后求和再约化（保留主数 11/22/33）。
//! - **生日数（Birthday）**：出生「日」约化。
//! - **表达数 / 灵魂数 / 人格数**：姓名全字母 / 元音 / 辅音之和约化。
//!
//! 约化用 [`reduce_with_master`]（遇 11/22/33 停）——主数例外是数字学自家的教义，
//! 不是通用数论，所以住在本 crate 而非 `mingli-core`。
//!
//! 语域注：数本身是确定计算；其「含义」属释义层，本 crate 不下断言。
//! 🟡 欠定项：生命灵数有「分量约化」与「全数字相加」两法（本 crate 用分量约化，多数教材主数法）；
//! Y 算元音还是辅音随流派，本 crate 三说并出，见 [`YRule`]。


#[cfg(feature = "port")]
mod engine;
#[cfg(feature = "port")]
pub use engine::NumerologyEngine;

use mingli_astro::Moment;
use mingli_core::ringhash::{string_sum, sum_digits};
#[cfg(feature = "serde")]
use serde::Serialize;

/// 字母表系统。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum System {
    /// Pythagorean：A=1…I=9 循环。
    Pythagorean,
    /// Chaldean：1..8（9 不配字母）。
    Chaldean,
}

/// Pythagorean 字母值：A=1…I=9，J 起再循环（仅 A-Z / a-z，其余 `None`）。
#[must_use]
pub fn pythagorean(c: char) -> Option<u64> {
    let u = c.to_ascii_uppercase();
    if u.is_ascii_uppercase() {
        Some(((u as u64 - 'A' as u64) % 9) + 1)
    } else {
        None
    }
}

/// 带主数例外的数字根约化：反复取各位之和，遇 11 / 22 / 33 即停。
///
/// 主数（master numbers）不再约化是西洋数字学的通行教义，多源一致。
#[must_use]
pub fn reduce_with_master(n: u64) -> u64 {
    let mut x = n;
    // 至多二十步：u64 最多二十位，而每一步的数位和都严格小于原数（x ≥ 10 时）。
    //
    // 从前这里是没有上限的 `loop`，靠 `x < 10` 收尾。把那个 `<` 改成 `==` 或 `>`，
    // 它就再也收不了——变异扫描里两个超时出自这一处，而超时既不算拦住也不算漏网。
    for _ in 0..20 {
        if matches!(x, 11 | 22 | 33) {
            return x;
        }
        if x < 10 {
            return x;
        }
        x = sum_digits(x);
    }
    unreachable!("u64 至多二十位，逐次取数位和二十步内必落到个位")
}

/// Chaldean 字母值（A..Z，索引 0..26）。1：AIJQY 2：BKR 3：CGLS 4：DMT 5：EHNX 6：UVW 7：OZ 8：FP。
const CHALDEAN: [u64; 26] = [
    1, 2, 3, 4, 5, 8, 3, 5, 1, 1, 2, 3, 4, 5, 7, 8, 1, 2, 3, 4, 6, 6, 6, 5, 1, 7,
];

/// Chaldean 字母值；非 A-Z 返回 `None`。
#[must_use]
pub fn chaldean(c: char) -> Option<u64> {
    let u = c.to_ascii_uppercase();
    if u.is_ascii_uppercase() {
        Some(CHALDEAN[(u as usize) - ('A' as usize)])
    } else {
        None
    }
}

/// 某系统下的字母值。
#[must_use]
pub fn letter_value(c: char, system: System) -> Option<u64> {
    match system {
        System::Pythagorean => pythagorean(c),
        System::Chaldean => chaldean(c),
    }
}

/// 是否是无争议的元音（AEIOU）。Y 的归属看语境，见 [`YRule`] 与 [`vowel_flags`]。
#[must_use]
pub fn is_vowel(c: char) -> bool {
    matches!(c.to_ascii_uppercase(), 'A' | 'E' | 'I' | 'O' | 'U')
}

/// 生命灵数的约化方法（流派分歧）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum LifePathMethod {
    /// **Component（分量约化）**：y/m/d 各约化后求和，再约化（保留主数 11/22/33）。
    /// 当代 Pythagorean 学派多用此法，易识别中间主数。
    Component,
    /// **WholeSum（全数字直加）**：y/m/d 全部数字平铺相加，再约化（保留主数 11/22/33）。
    /// 古典 Chaldean / Kabbalistic 派常用，主数仅出现在最终一次约化。
    WholeSum,
}

/// 生命灵数（按指定流派算）。
#[must_use]
pub fn life_path_with(year: i64, month: u32, day: u32, method: LifePathMethod) -> u64 {
    #[allow(clippy::cast_sign_loss, reason = "出生年取正；下游仅数字根")]
    let yabs = year.unsigned_abs();
    match method {
        LifePathMethod::Component => {
            let y = reduce_with_master(digit_sum_u64(yabs));
            let m = reduce_with_master(u64::from(month));
            let d = reduce_with_master(u64::from(day));
            reduce_with_master(y + m + d)
        }
        LifePathMethod::WholeSum => {
            let s = digit_sum_u64(yabs) + digit_sum_u64(u64::from(month)) + digit_sum_u64(u64::from(day));
            reduce_with_master(s)
        }
    }
}

/// 生命灵数（默认 [`LifePathMethod::Component`]，分量约化法）。
#[must_use]
pub fn life_path(year: i64, month: u32, day: u32) -> u64 {
    life_path_with(year, month, day, LifePathMethod::Component)
}

/// 各位数字之和（多位年份用）。
fn digit_sum_u64(mut n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut s = 0;
    // 同上，至多二十位。`while n > 0` 靠 `n /= 10` 收尾；把它改成 `%=` 就不动了。
    for _ in 0..20 {
        if n == 0 {
            break;
        }
        s += n % 10;
        n /= 10;
    }
    s
}

/// 生日数：出生「日」约化（保留主数）。
#[must_use]
pub fn birthday_number(day: u32) -> u64 {
    reduce_with_master(u64::from(day))
}

/// 表达数（Expression / Destiny）：姓名全字母之和约化。
#[must_use]
pub fn expression(name: &str, system: System) -> u64 {
    reduce_with_master(string_sum(name, |c| letter_value(c, system)))
}

/// Y 归入元音还是辅音的约定。灵魂数与人格数按这条分岔，表达数不受影响。
///
/// 三说的来源强度差得很远，选项按强度排：
///
/// - [`Contextual`](YRule::Contextual)：**4 个独立源**（Hans Decoz / World Numerology、
///   Token Rock、Felicia Bender、Crystal Logic）。Decoz 给了八条按位置的细则，
///   Token Rock 一句话概括为「Y 恒为元音，除非它紧挨着另一个元音」——两者逐例一致，
///   本 crate 实现的就是这一句，它能复现 Decoz 全部八条（含其两条 default）。
/// - [`AfterVowel`](YRule::AfterVowel)：**2 个独立源**（Lyn's Numerology Charts、Astrala）
///   明确主张「Y 跟在元音后面仍算元音」（Clayton / May / Taylor）。
/// - [`Never`](YRule::Never)：**1 个二手转述**（Felicia Bender 引 Juno Jordan
///   《Numerology: The Romance In Your Name》，未取得原书）。这是本 crate 从前的默认。
///
/// 🟡 未实现的部分：前两说都还带一条**按音节**的条款（「该音节里没有别的元音时 Y 算元音」，
/// 如 Bryan 的 Y），要分音节才能判，本 crate 没有音节切分器，故不实现，也不假装实现。
/// Y 恒为元音（Lynn Buess）同样只有一处二手转述，不入选项。
/// W 在 Matthew / Drew / Owen 一类里算元音的说法有 2 源（其中一处只有立场没有规则），
/// 强度不足，本 crate 一律把 W 当辅音。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum YRule {
    /// Y 紧邻另一个元音（前或后）时作辅音，否则作元音。
    Contextual,
    /// 只有后接元音时 Y 才作辅音；跟在元音后面仍作元音。
    AfterVowel,
    /// Y 一律作辅音。
    Never,
}

impl YRule {
    /// 稳定标识（进 JSON）。
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Contextual => "contextual",
            Self::AfterVowel => "after_vowel",
            Self::Never => "never",
        }
    }

    /// 三种约定，按来源强度排。
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Contextual, Self::AfterVowel, Self::Never]
    }
}

/// 逐字符判定「这个位置算不算元音」。返回长度 = `name.chars().count()`。
///
/// AEIOU 恒为元音；Y 按 `rule` 定；其余（含 W、非字母）为非元音。
/// 判邻居时**不跨词**——空格、连字符一类非字母把词断开，所以「Mary Ann」里
/// Mary 的 Y 后面没有字母，算元音。
#[must_use]
pub fn vowel_flags(name: &str, rule: YRule) -> Vec<bool> {
    let chars: Vec<char> = name.chars().collect();
    let mut out = vec![false; chars.len()];
    for i in 0..chars.len() {
        let c = chars[i];
        if is_vowel(c) {
            out[i] = true;
            continue;
        }
        if !c.eq_ignore_ascii_case(&'y') {
            continue;
        }
        let letter_at = |k: usize| chars.get(k).copied().filter(char::is_ascii_alphabetic);
        let prev_is_vowel = i.checked_sub(1).and_then(letter_at).is_some_and(is_vowel);
        let next_is_vowel = letter_at(i + 1).is_some_and(is_vowel);
        out[i] = match rule {
            YRule::Never => false,
            YRule::Contextual => !prev_is_vowel && !next_is_vowel,
            YRule::AfterVowel => !next_is_vowel,
        };
    }
    out
}

fn sum_where(name: &str, system: System, keep: impl Fn(bool, char) -> bool, rule: YRule) -> u64 {
    let flags = vowel_flags(name, rule);
    let total: u64 = name
        .chars()
        .zip(flags)
        .filter(|&(c, is_v)| keep(is_v, c))
        .filter_map(|(c, _)| letter_value(c, system))
        .sum();
    reduce_with_master(total)
}

/// 灵魂数（Soul Urge）：姓名元音之和约化，Y 的归属按 `rule`。
#[must_use]
pub fn soul_urge_with(name: &str, system: System, rule: YRule) -> u64 {
    sum_where(name, system, |is_v, _| is_v, rule)
}

/// 人格数（Personality）：姓名辅音之和约化，Y 的归属按 `rule`。
#[must_use]
pub fn personality_with(name: &str, system: System, rule: YRule) -> u64 {
    sum_where(name, system, |is_v, c| !is_v && c.is_ascii_alphabetic(), rule)
}

/// 灵魂数（Soul Urge）：姓名元音之和约化。
///
/// Y 按来源最强的 [`YRule::Contextual`] 判；另两说的读数在
/// [`NameNumbers::by_y_rule`] 里一并给出。
#[must_use]
pub fn soul_urge(name: &str, system: System) -> u64 {
    soul_urge_with(name, system, YRule::Contextual)
}

/// 人格数（Personality）：姓名辅音之和约化。
///
/// Y 按来源最强的 [`YRule::Contextual`] 判；另两说的读数在
/// [`NameNumbers::by_y_rule`] 里一并给出。
#[must_use]
pub fn personality(name: &str, system: System) -> u64 {
    personality_with(name, system, YRule::Contextual)
}

/// 某一种 Y 归属约定下的灵魂数与人格数。
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct VowelReading {
    /// 约定标识（`"contextual"` / `"after_vowel"` / `"never"`）。
    pub y_rule: &'static str,
    /// 该约定下的灵魂数。
    pub soul_urge: u64,
    /// 该约定下的人格数。
    pub personality: u64,
}

/// 姓名数（某系统下的表达/灵魂/人格）。
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct NameNumbers {
    /// 字母表系统。
    pub system: System,
    /// 表达数（全字母之和，不受 Y 归属影响）。
    pub expression: u64,
    /// 灵魂数（按 [`YRule::Contextual`]）。
    pub soul_urge: u64,
    /// 人格数（按 [`YRule::Contextual`]）。
    pub personality: u64,
    /// 三种 Y 归属约定下的读数并出，按来源强度排；不替调用方选边。
    pub by_y_rule: [VowelReading; 3],
}

/// 由姓名与系统算姓名数。
#[must_use]
pub fn name_numbers(name: &str, system: System) -> NameNumbers {
    let reading = |rule: YRule| VowelReading {
        y_rule: rule.id(),
        soul_urge: soul_urge_with(name, system, rule),
        personality: personality_with(name, system, rule),
    };
    NameNumbers {
        system,
        expression: expression(name, system),
        soul_urge: soul_urge(name, system),
        personality: personality(name, system),
        by_y_rule: YRule::all().map(reading),
    }
}

/// 一次数字学换算的结果。日期数恒有；姓名数在给出姓名时附上（两套系统）。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Cast {
    /// 生命灵数（按当前所选流派 `life_path_method`）。
    pub life_path: u64,
    /// 当前流派标识(`"component"` / `"whole_sum"`)。
    pub life_path_method: &'static str,
    /// 另一流派的生命灵数（对照）；Component 选中时此处 = WholeSum 值，反之亦然。
    pub life_path_alt: u64,
    /// 生日数。
    pub birthday: u64,
    /// 姓名数（Pythagorean），给出姓名时有。
    pub pythagorean: Option<NameNumbers>,
    /// 姓名数（Chaldean），给出姓名时有。
    pub chaldean: Option<NameNumbers>,
}

fn life_path_pair(year: i64, month: u32, day: u32, method: LifePathMethod) -> (u64, u64, &'static str) {
    let main = life_path_with(year, month, day, method);
    let (alt_method, id) = match method {
        LifePathMethod::Component => (LifePathMethod::WholeSum, "component"),
        LifePathMethod::WholeSum => (LifePathMethod::Component, "whole_sum"),
    };
    (main, life_path_with(year, month, day, alt_method), id)
}

/// 在共享上下文 [`Moment`] 上算日期数字学（不含姓名，默认 Component 流派）。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    compute_at_with(m, LifePathMethod::Component)
}

/// 在共享上下文 [`Moment`] 上算日期数字学（指定流派，不含姓名）。
#[must_use]
pub fn compute_at_with(m: &Moment, method: LifePathMethod) -> Cast {
    let (lp, alt, id) = life_path_pair(i64::from(m.year), m.month, m.day, method);
    Cast {
        life_path: lp,
        life_path_method: id,
        life_path_alt: alt,
        birthday: birthday_number(m.day),
        pythagorean: None,
        chaldean: None,
    }
}

/// 在共享上下文上算日期 + 姓名数字学（默认 Component；姓名两套字母表并出）。
#[must_use]
pub fn compute_named(m: &Moment, name: &str) -> Cast {
    compute_named_with(m, name, LifePathMethod::Component)
}

/// 在共享上下文上算日期 + 姓名数字学（指定生命灵数流派；姓名两套字母表并出）。
#[must_use]
pub fn compute_named_with(m: &Moment, name: &str, method: LifePathMethod) -> Cast {
    let (lp, alt, id) = life_path_pair(i64::from(m.year), m.month, m.day, method);
    Cast {
        life_path: lp,
        life_path_method: id,
        life_path_alt: alt,
        birthday: birthday_number(m.day),
        pythagorean: Some(name_numbers(name, System::Pythagorean)),
        chaldean: Some(name_numbers(name, System::Chaldean)),
    }
}

/// 由本地民用日期算（独立入口）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, 12, 0, tz))
}

#[cfg(test)]
mod tests;
