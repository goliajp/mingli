//! L3 叶（D 族 / 确定性）：姓名五格剖象法（熊崎式）。
//!
//! 五格把姓名的**康熙笔画**组合成五个数，再各自取五行与 81 数。本 crate 实现**格的推导算法**
//! （以笔画为输入，纯模运算），公式经多源核对：
//!
//! - **天格**：单姓 = 姓 + 1（虚位）；复姓 = 姓各字之和。
//! - **人格**：姓之末字 + 名之首字。
//! - **地格**：单名 = 名 + 1（虚位）；复名 = 名各字之和。
//! - **外格**：单姓单名 = 2；否则 = 总格 − 人格 + 1。
//! - **总格**：姓名全字笔画之和。
//!
//! 每格取**三才五行**（个位：1·2 木、3·4 火、5·6 土、7·8 金、9·0 水）与 **81 数**
//! （[`fold_81`] 归一到 1..=81）。
//!
//! 诚实边界（🟡）：**康熙笔画表**（数千汉字的繁体笔画）属大查表，本 crate **不内置**——笔画由调用方
//! 提供（错一字毒整枝）。**81 数的吉凶判断**亦属查表 + 流派分歧，本 crate 只给 81 数本身、不下吉凶断言。

mod engine;
pub use engine::WugeEngine;

/// 五格数按 `mod-80` 归一到 `1..=81`。
///
/// 81 数体系里 81 与 1 同位，超过 81 者减 80 循环——这是熊崎式的通行约定
/// （🟡 版本有别：另有「减 80 至 ≤80」等变体，本 crate 取通行版）。
#[must_use]
pub const fn fold_81(n: u64) -> u64 {
    if n == 0 {
        0
    } else {
        ((n - 1) % 80) + 1
    }
}
use serde::Serialize;

/// 五行（按格的个位定）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Element {
    /// 木（个位 1、2）。
    Wood,
    /// 火（个位 3、4）。
    Fire,
    /// 土（个位 5、6）。
    Earth,
    /// 金（个位 7、8）。
    Metal,
    /// 水（个位 9、0）。
    Water,
}

impl Element {
    /// 五行名。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Element::Wood => "木",
            Element::Fire => "火",
            Element::Earth => "土",
            Element::Metal => "金",
            Element::Water => "水",
        }
    }
}

/// 由格数的个位定五行。
#[must_use]
pub fn element_of(n: u32) -> Element {
    match n % 10 {
        1 | 2 => Element::Wood,
        3 | 4 => Element::Fire,
        5 | 6 => Element::Earth,
        7 | 8 => Element::Metal,
        _ => Element::Water, // 9、0
    }
}

/// 单格：原值、归一 81 数、五行。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Grid {
    /// 笔画原值。
    pub value: u32,
    /// 归一到 1..=81 的数（81 数）。
    pub number: u32,
    /// 五行。
    pub element: Element,
    /// 五行名。
    pub element_name: &'static str,
}

fn grid(value: u32) -> Grid {
    #[allow(clippy::cast_possible_truncation, reason = "fold_81 结果 ∈ 1..=81")]
    let number = fold_81(u64::from(value)) as u32;
    let element = element_of(value);
    Grid {
        value,
        number,
        element,
        element_name: element.name(),
    }
}

/// 五格剖象结果。
#[derive(Debug, Clone, Serialize)]
pub struct Cast {
    /// 天格。
    pub heaven: Grid,
    /// 人格（主格）。
    pub human: Grid,
    /// 地格。
    pub earth: Grid,
    /// 外格。
    pub outer: Grid,
    /// 总格。
    pub total: Grid,
}

/// 由姓、名各字的康熙笔画算五格。`surname` / `given` 为各字笔画（至少各一字）。
///
/// # Panics
/// 当 `surname` 或 `given` 为空时 panic（姓名至少各一字，属调用契约）。
#[must_use]
pub fn five_grids(surname: &[u32], given: &[u32]) -> Cast {
    assert!(!surname.is_empty() && !given.is_empty(), "姓与名至少各一字");
    let surname_sum: u32 = surname.iter().sum();
    let given_sum: u32 = given.iter().sum();
    let single_surname = surname.len() == 1;
    let single_given = given.len() == 1;

    let heaven = if single_surname { surname_sum + 1 } else { surname_sum };
    let human = surname[surname.len() - 1] + given[0];
    let earth = if single_given { given_sum + 1 } else { given_sum };
    let total = surname_sum + given_sum;
    // 外格 = 天 + 地 − 人。
    //
    // 等价写法是「総 − 人 + 霊数个数」——而**霊数不是公式的一部分**：单姓给天格补一、
    // 单名给地格补一，复姓双名两处都不补。从前这里写作 `総 − 人 + 1`，把补一当成了通式，
    // 于是复姓双名一路多出 1（姓[4,8] 名[6,9] 得 14，实为姓首 4 + 名末 9 = 13）。
    let outer = heaven + earth - human;

    Cast {
        heaven: grid(heaven),
        human: grid(human),
        earth: grid(earth),
        outer: grid(outer),
        total: grid(total),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_81_wraps_at_eighty() {
        assert_eq!(fold_81(0), 0);
        assert_eq!(fold_81(1), 1);
        assert_eq!(fold_81(80), 80);
        assert_eq!(fold_81(81), 1);
        assert_eq!(fold_81(160), 80);
        assert_eq!(fold_81(161), 1);
        // 值域恒在 1..=81（0 除外）
        for n in 1..500u64 {
            assert!((1..=81).contains(&fold_81(n)), "fold_81({n}) 出界");
        }
    }

    #[test]
    fn element_by_last_digit() {
        assert_eq!(element_of(11), Element::Wood); // 个位1
        assert_eq!(element_of(2), Element::Wood);
        assert_eq!(element_of(13), Element::Fire);
        assert_eq!(element_of(24), Element::Fire);
        assert_eq!(element_of(5), Element::Earth);
        assert_eq!(element_of(16), Element::Earth);
        assert_eq!(element_of(7), Element::Metal);
        assert_eq!(element_of(28), Element::Metal);
        assert_eq!(element_of(9), Element::Water);
        assert_eq!(element_of(10), Element::Water); // 个位0
        assert_eq!(Element::Wood.name(), "木");
        assert_eq!(Element::Water.name(), "水");
    }

    #[test]
    fn single_surname_single_given() {
        // 单姓单名：天=姓+1、人=姓+名、地=名+1、外=2、总=姓+名。
        // 例：姓 7 画、名 16 画。天8 人23 地17 外2 总23。
        let c = five_grids(&[7], &[16]);
        assert_eq!(c.heaven.value, 8);
        assert_eq!(c.human.value, 23);
        assert_eq!(c.earth.value, 17);
        assert_eq!(c.outer.value, 2); // 单姓单名固定 2
        assert_eq!(c.total.value, 23);
    }

    #[test]
    fn single_surname_double_given() {
        // 单姓双名：外 = 总 − 人 + 1 = 名末字 + 1。
        // 姓 5、名 [6, 9]。天6 人=5+6=11 地=15 总=20 外=20−11+1=10=名末9+1。
        let c = five_grids(&[5], &[6, 9]);
        assert_eq!(c.heaven.value, 6);
        assert_eq!(c.human.value, 11);
        assert_eq!(c.earth.value, 15);
        assert_eq!(c.total.value, 20);
        assert_eq!(c.outer.value, 10);
        assert_eq!(c.outer.value, given_last_plus_one(&[6, 9])); // = 名末字+1
    }

    fn given_last_plus_one(given: &[u32]) -> u32 {
        given[given.len() - 1] + 1
    }

    #[test]
    fn compound_surname_single_given() {
        // 复姓单名：天=姓和、外=总−人+1=姓首字+1。
        // 姓 [4, 8]、名 10。天12 人=8+10=18 地=11 总=22 外=22−18+1=5=姓首4+1。
        let c = five_grids(&[4, 8], &[10]);
        assert_eq!(c.heaven.value, 12);
        assert_eq!(c.human.value, 18);
        assert_eq!(c.earth.value, 11);
        assert_eq!(c.total.value, 22);
        assert_eq!(c.outer.value, 5);
    }

    #[test]
    fn compound_surname_double_given() {
        // 复姓双名：两处都无霊数，故外格不补一。
        // 姓[4,8] 名[6,9]。天12 人=8+6=14 地=15 总=27 外=天12+地15−人14=13=姓首4+名末9。
        let c = five_grids(&[4, 8], &[6, 9]);
        assert_eq!(c.heaven.value, 12);
        assert_eq!(c.human.value, 14);
        assert_eq!(c.earth.value, 15);
        assert_eq!(c.total.value, 27);
        assert_eq!(c.outer.value, 13);
        assert_eq!(c.outer.value, 4 + 9, "复姓双名的外格即姓首 + 名末");
    }

    #[test]
    fn grid_number_folds_to_81_and_element() {
        // 81 数归一：>81 折回 1..=81；五行随原值个位。
        let c = five_grids(&[50], &[40]); // 总=90 → fold_81(90)=10
        assert_eq!(c.total.value, 90);
        assert_eq!(c.total.number, 10);
        assert!((1..=81).contains(&c.total.number));
        assert_eq!(c.total.element, Element::Water); // 个位0
        assert_eq!(c.total.element_name, "水");
    }

    /// 外格与另一份独立实现逐点相同。
    ///
    /// 第二源：`Getabako/SeimeiHandan`（日文，熊崎式五格，`js/gokaku.js`）。
    /// 它把规矩写作「霊数：一字姓は天格に+1、一字名は地格に+1。総格には含めない。外格の計算には含める」，
    /// 外格取 `天 + 地 − 人`。本仓从前写作 `総 − 人 + 1`，把霊数当成了通式的一部分，
    /// 于是复姓双名一路多出 1——这条比对就是那个 bug 的来处。
    #[test]
    fn the_outer_grid_agrees_with_an_independent_implementation() {
        let reference = |sur: &[u32], giv: &[u32]| -> u32 {
            let (ss, gs): (u32, u32) = (sur.iter().sum(), giv.iter().sum());
            let ten = ss + u32::from(sur.len() == 1);
            let chi = gs + u32::from(giv.len() == 1);
            let jin = sur[sur.len() - 1] + giv[0];
            ten + chi - jin
        };
        let mut checked = 0;
        for a in 1..=20 {
            for b in 1..=20 {
                for shape in 0..4 {
                    let (sur, giv): (Vec<u32>, Vec<u32>) = match shape {
                        0 => (vec![a], vec![b]),
                        1 => (vec![a], vec![b, a]),
                        2 => (vec![a, b], vec![b]),
                        _ => (vec![a, b], vec![b, a]),
                    };
                    assert_eq!(
                        five_grids(&sur, &giv).outer.value,
                        reference(&sur, &giv),
                        "姓{sur:?} 名{giv:?} 的外格与第二源不合"
                    );
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 1600);
    }

    /// 各种姓名长度下外格的封闭式，逐条写明——公式对了，这四条才都对得上。
    #[test]
    fn the_outer_grid_has_a_closed_form_for_each_shape() {
        // 单姓单名：天(a+1) + 地(b+1) − 人(a+b) = 2，与笔画无关
        assert_eq!(five_grids(&[7], &[3]).outer.value, 2);
        assert_eq!(five_grids(&[19], &[11]).outer.value, 2);
        // 单姓双名：= 名末 + 1
        assert_eq!(five_grids(&[5], &[6, 9]).outer.value, 9 + 1);
        // 复姓单名：= 姓首 + 1
        assert_eq!(five_grids(&[4, 8], &[10]).outer.value, 4 + 1);
        // 复姓双名：= 姓首 + 名末（两处都无霊数）
        assert_eq!(five_grids(&[4, 8], &[6, 9]).outer.value, 4 + 9);
    }

    #[test]
    #[should_panic(expected = "姓与名至少各一字")]
    fn empty_panics() {
        let _ = five_grids(&[], &[5]);
    }

    #[test]
    fn all_grids_have_valid_81_numbers() {
        // 性质：任意笔画组合，五格的 81 数皆在 1..=81。
        for s in 1..30u32 {
            for g in 1..30u32 {
                let c = five_grids(&[s], &[g]);
                for grid in [c.heaven, c.human, c.earth, c.outer, c.total] {
                    assert!((1..=81).contains(&grid.number));
                }
            }
        }
    }
}
