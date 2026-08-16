//! 团队合盘用例：N 人本命 → 团队五行画像 + N×N 互补矩阵。
//!
//! 计算与呈现分开：[`compute`] 只出结构，两种 JSON 形态（完整盘 / 释义用摘要）
//! 由 [`TeamResult`] 的两个方法给出——原先这段编排在两个 HTTP handler 里各抄了一遍。

use crate::{bazi::birth_input, Birth};
use mingli_bazi::BaziChart;
use serde_json::{json, Value};

/// 团队成员：一份出生输入 + 一个可选称呼。
#[derive(Debug, Clone, Copy)]
pub struct Member<'a> {
    /// 出生输入。
    pub birth: Birth,
    /// 称呼（缺省时用出生年占位）。
    pub name: Option<&'a str>,
}

/// 合盘上限人数（经典合婚 2 人，团队最多 12 人）。
const MAX_MEMBERS: usize = 12;

/// 团队合盘结果。
#[derive(Debug)]
pub struct TeamResult {
    charts: Vec<BaziChart>,
    names: Vec<String>,
    team_wuxing: mingli_bazi::WuxingPower,
    weakest: (String, u32),
    strongest: (String, u32),
    matrix: Vec<Vec<u32>>,
}

/// 算一组成员的合盘。
///
/// # Errors
///
/// 人数为 0 或超过 12 时返回错误说明。
pub fn compute(members: &[Member]) -> Result<TeamResult, String> {
    if members.is_empty() || members.len() > MAX_MEMBERS {
        return Err(format!("members 须 1-{MAX_MEMBERS} 人"));
    }
    let charts: Vec<BaziChart> =
        members.iter().map(|m| mingli_bazi::compute(birth_input(&m.birth))).collect();
    let names = members
        .iter()
        .map(|m| m.name.map_or_else(|| format!("成员 {}", m.birth.year), ToString::to_string))
        .collect();
    let team_wuxing = mingli_bazi::team_wuxing_average(&charts);
    let weakest = mingli_bazi::team_weakest(&team_wuxing);
    let strongest = mingli_bazi::team_strongest(&team_wuxing);
    // 互补矩阵 N×N：M[i][j] = j 对 i 主用神的供给度
    let matrix = charts
        .iter()
        .map(|ci| {
            charts
                .iter()
                .map(|cj| {
                    mingli_bazi::complement_score(&ci.yongshen.primary_wuxing, &cj.strength.wuxing)
                })
                .collect()
        })
        .collect();
    Ok(TeamResult { charts, names, team_wuxing, weakest, strongest, matrix })
}

impl TeamResult {
    /// 完整形态：每人带四柱干支，供前端排盘展示。
    #[must_use]
    pub fn to_json(&self) -> Value {
        self.render(true)
    }

    /// 摘要形态：去掉四柱字面，只留结构结论，供释义层消费。
    #[must_use]
    pub fn to_summary_json(&self) -> Value {
        self.render(false)
    }

    fn render(&self, with_pillars: bool) -> Value {
        let members: Vec<Value> = self
            .names
            .iter()
            .zip(self.charts.iter())
            .map(|(name, c)| {
                let mut v = json!({
                    "name": name,
                    "day_master": c.day_master,
                    "day_master_wuxing": c.day_master_wuxing,
                });
                if with_pillars {
                    v["year_gz"] = json!(c.year.ganzhi);
                    v["month_gz"] = json!(c.month.ganzhi);
                    v["day_gz"] = json!(c.day.ganzhi);
                    v["hour_gz"] = json!(c.hour.ganzhi);
                }
                v["strength"] = json!(c.strength);
                v["yongshen"] = json!(c.yongshen);
                v
            })
            .collect();
        json!({
            "members": members,
            "team_wuxing": self.team_wuxing,
            "team_weakest": { "wuxing": self.weakest.0, "pct": self.weakest.1 },
            "team_strongest": { "wuxing": self.strongest.0, "pct": self.strongest.1 },
            "complement_matrix": self.matrix,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mingli_contract::Gender;

    fn birth(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Birth {
        Birth {
            year,
            month,
            day,
            hour,
            minute,
            tz: 8.0,
            gender: Some(Gender::Male),
            true_solar_time: false,
            longitude: None,
        }
    }

    #[test]
    fn member_count_is_bounded() {
        assert!(compute(&[]).is_err());
        let one = Member { birth: birth(1987, 9, 17, 15, 0), name: None };
        let too_many = vec![one; MAX_MEMBERS + 1];
        assert!(compute(&too_many).is_err());
        assert!(compute(&[one]).is_ok());
    }

    #[test]
    fn matrix_is_square_and_summary_drops_pillars() {
        let ms = [
            Member { birth: birth(1987, 9, 17, 15, 0), name: Some("A") },
            Member { birth: birth(1990, 6, 15, 14, 30), name: None },
        ];
        let r = compute(&ms).expect("双人合盘应成立");
        let full = r.to_json();
        assert_eq!(full["complement_matrix"].as_array().map(Vec::len), Some(2));
        assert_eq!(full["members"][0]["name"], "A");
        // 未命名成员用出生年占位
        assert_eq!(full["members"][1]["name"], "成员 1990");
        assert!(full["members"][0]["year_gz"].is_string());
        assert!(r.to_summary_json()["members"][0]["year_gz"].is_null());
    }
}
