//! 合盘用例：两人本命 → 互补结构。
//!
//! 团队合盘（[`crate::team`]）已能对 N 人出五行画像与 N×N 互补矩阵；合盘就是它的双人特例，
//! 但输出形态不同——「配」要的是**甲乙两方各自的用神、对方给了多少**这两句话，
//! 而不是一张矩阵。这里把双人特例包成正式路径，并把 2×2 矩阵摊平成互供两个数。

use crate::team::{self, Member};
use crate::Birth;
#[cfg(feature = "astrology")]
use mingli_astro::Moment;
use serde::Serialize;
use serde_json::{json, Value};

/// 一次合盘的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Synastry {
    /// 甲方名。
    pub a_name: String,
    /// 乙方名。
    pub b_name: String,
    /// 甲方给乙方主用神的供给度（%）：乙的用神五行在甲盘里占多少。
    pub a_supplies_b: u32,
    /// 乙方给甲方主用神的供给度（%）。
    pub b_supplies_a: u32,
    /// 团队合盘的完整结构（两人的旺衰 / 用神 / 五行画像 / 2×2 矩阵），供释义层与前端。
    pub detail: Value,
    /// 两人本命盘之间的占星相位（几何事实，不含取舍）。
    pub aspects: Value,
}

/// 合盘：两人本命各排一盘，取互补矩阵的两个非对角元。
///
/// # Errors
///
/// 底层团队合盘失败时（不会发生于双人）原样返回说明。
pub fn compute(a: (&Birth, Option<&str>), b: (&Birth, Option<&str>)) -> Result<Synastry, String> {
    let members = [
        Member { birth: *a.0, name: Some(a.1.unwrap_or("甲")) },
        Member { birth: *b.0, name: Some(b.1.unwrap_or("乙")) },
    ];
    let r = team::compute(&members)?;
    let detail = r.to_json();
    // complement_matrix[i][j] = j 对 i 主用神的供给度 → 甲供乙 = m[1][0]，乙供甲 = m[0][1]
    let m = &detail["complement_matrix"];
    let cell = |i: usize, j: usize| u32::try_from(m[i][j].as_u64().unwrap_or(0)).unwrap_or(0);
    Ok(Synastry {
        a_name: a.1.unwrap_or("甲").to_string(),
        b_name: b.1.unwrap_or("乙").to_string(),
        a_supplies_b: cell(1, 0),
        b_supplies_a: cell(0, 1),
        detail,
        aspects: cross_aspects_between(a.0, b.0),
    })
}

/// 两人本命盘之间的相位（只出几何，不出取舍）。
///
/// 四柱那一路给的是「互供用神」——一个量化的供给度；占星这一路给的是「两盘之间成了哪些角」——
/// 一组结构事实。两者都是「配」，说的却不是同一件事，故并列而不合成。
///
/// 哪些相位算数、容许度多少、哪些星入合盘，各家出入很大，那属取舍不属计算，
/// 本层按默认容许度出全量，选哪些交释义层。
#[cfg(feature = "astrology")]
fn cross_aspects_between(a: &Birth, b: &Birth) -> Value {
    let chart = |x: &Birth| {
        let m = Moment::new(x.year, x.month, x.day, x.hour, x.minute, x.tz);
        mingli_astrology::compute_at(&m, None, mingli_astrology::HouseSystem::Placidus)
    };
    let (ca, cb) = (chart(a), chart(b));
    let list = mingli_astrology::cross_aspects(&ca.planets, &cb.planets, mingli_astrology::DEFAULT_ORB);
    json!({
        "system": "astrology",
        "orb": mingli_astrology::DEFAULT_ORB,
        "count": list.len(),
        "list": list,
    })
}

/// 关掉 `astrology` feature 时的桩：这套算力不在本次构建里。
#[cfg(not(feature = "astrology"))]
fn cross_aspects_between(_a: &Birth, _b: &Birth) -> Value {
    Value::Null
}

impl Synastry {
    /// 序列化成对外 JSON：互供两数置顶，团队结构随后。
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "a_name": self.a_name,
            "b_name": self.b_name,
            "a_supplies_b": self.a_supplies_b,
            "b_supplies_a": self.b_supplies_a,
            "detail": self.detail,
            "aspects": self.aspects,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mingli_contract::Gender;

    fn birth(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Birth {
        Birth { year, month, day, hour, minute, tz: 8.0, gender: Some(Gender::Male), true_solar_time: false, longitude: None }
    }

    #[test]
    fn mutual_supply_is_read_off_the_two_off_diagonals() {
        // 1987 长沙男 A + 1990 长沙男 B：既有 oracle 矩阵 [[11,18],[23,22]]
        // → 乙供甲 = m[0][1] = 18，甲供乙 = m[1][0] = 23
        let a = birth(1987, 9, 17, 15, 0);
        let b = birth(1990, 6, 15, 14, 30);
        let s = compute((&a, Some("A")), (&b, Some("B"))).expect("双人合盘应成立");
        assert_eq!((s.b_supplies_a, s.a_supplies_b), (18, 23));
        assert_eq!((s.a_name.as_str(), s.b_name.as_str()), ("A", "B"));
        assert_eq!(s.detail["complement_matrix"][0][1], 18);
    }

    #[test]
    fn names_default_to_jia_yi_and_json_puts_supply_first() {
        let a = birth(1987, 9, 17, 15, 0);
        let b = birth(1990, 6, 15, 14, 30);
        let s = compute((&a, None), (&b, None)).expect("应成立");
        let v = s.to_json();
        assert_eq!((v["a_name"].as_str(), v["b_name"].as_str()), (Some("甲"), Some("乙")));
        assert!(v["a_supplies_b"].is_u64() && v["b_supplies_a"].is_u64());
        assert!(v["detail"]["team_wuxing"].is_object());
    }

    #[test]
    fn swapping_the_pair_swaps_the_two_numbers() {
        let a = birth(1987, 9, 17, 15, 0);
        let b = birth(1990, 6, 15, 14, 30);
        let ab = compute((&a, None), (&b, None)).expect("应成立");
        let ba = compute((&b, None), (&a, None)).expect("应成立");
        assert_eq!((ab.a_supplies_b, ab.b_supplies_a), (ba.b_supplies_a, ba.a_supplies_b));
    }
}
