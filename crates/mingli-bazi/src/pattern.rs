//! 命格：月令藏干透干定型——八正格 / 建禄月刃 / 暗格。

use super::*;

/// 命格：月令藏干透干定的「命主类型」。
///
/// **主流通行规则（子平真诠/三命通会一致）**：
/// 1. **建禄 / 月刃**：月令本气 = 日主同五行（同阴阳=建禄、异阴阳=月刃/阳刃），不取八正格。
/// 2. **八正格**（正官/七杀/正财/偏财/正印/偏印/食神/伤官）：月令本气、中气、余气 **按序** 查
///    年/月/时三干头是否「透出」，先透先定；透干所对日主的十神 = 格局名。
/// 3. **暗格**：月令藏干都不透 → 取本气暗藏立格（传统称「八字纯藏」）。
///
/// 格局是命主的**结构性属性**（命理传统的「类型分类」），命主的吉凶倾向 = 格局类型 × 用神扶抑。
///
/// **🟡 流派分歧**：从格/化格/专旺格成立条件分歧大（身极弱+无救助+顺势依附，各家定义不一），
/// 杂气格（辰戌丑未）透干优先级、月令分日用事均有派别，本算法不机械化，留 INT 释义层。
#[derive(Debug, Clone, Serialize)]
pub struct Pattern {
    /// 格局名（正官格/七杀格/正财格/偏财格/正印格/偏印格/食神格/伤官格/建禄格/月刃格/暗 X 格）。
    pub name: String,
    /// 取格依据（中文短句：如「月令本气透干 月柱」「月令暗藏」「月令本气=日主同五行」）。
    pub source: String,
    /// 取格的藏干（月令本/中/余气）字面；禄刃情况为月令本气。
    pub qi_stem: String,
    /// 该藏干所属的气位（本气/中气/余气）。
    pub qi_kind: String,
    /// 透干所在柱（年柱/月柱/时柱）；未透为 None。
    pub revealed_in: Option<String>,
    /// 该藏干相对日主的十神。
    pub ten_god: String,
    /// 是否透干（true=明格、false=暗格或禄刃）。
    pub revealed: bool,
    /// 是否走「禄刃」特殊分支（比肩/劫财不入八正格）。
    pub is_lu_ren: bool,
}

/// 月令定格：按主流通行规则取八正格 / 建禄月刃 / 暗格。
///
/// 入参为本命四柱；不依赖出生时刻——纯符号层运算。
#[must_use]
pub fn determine_pattern(
    year_gz: GanZhi,
    month_gz: GanZhi,
    day_gz: GanZhi,
    hour_gz: GanZhi,
) -> Pattern {
    let dm = day_gz.stem;
    let mb = month_gz.branch;
    let hidden = hidden_stems(mb);
    let main_qi = hidden[0];

    // 特殊分支：月令本气 = 日主同五行 → 建禄（同阴阳） / 月刃（异阴阳）
    if stem_element(main_qi) == stem_element(dm) {
        let same_polarity = (main_qi % 2) == (dm % 2);
        let name = if same_polarity { "建禄格" } else { "月刃格" };
        return Pattern {
            name: name.to_string(),
            source: "月令本气=日主同五行（比劫不入八正格）".to_string(),
            qi_stem: STEMS[main_qi as usize].to_string(),
            qi_kind: "本气".to_string(),
            revealed_in: None,
            ten_god: ten_god(dm, main_qi).to_string(),
            revealed: false,
            is_lu_ren: true,
        };
    }

    // 八正格：按本→中→余 查年/月/时透干。日主自身不算「透」。
    let pillars: [(&str, u8); 3] = [
        ("年柱", year_gz.stem),
        ("月柱", month_gz.stem),
        ("时柱", hour_gz.stem),
    ];
    let qi_names = ["本气", "中气", "余气"];

    for (i, &qi) in hidden.iter().enumerate() {
        if let Some(&(pname, _)) = pillars.iter().find(|&&(_, st)| st == qi) {
            let god = ten_god(dm, qi);
            return Pattern {
                name: format!("{god}格"),
                source: format!("月令{}透干 {}", qi_names[i.min(2)], pname),
                qi_stem: STEMS[qi as usize].to_string(),
                qi_kind: qi_names[i.min(2)].to_string(),
                revealed_in: Some(pname.to_string()),
                ten_god: god.to_string(),
                revealed: true,
                is_lu_ren: false,
            };
        }
    }

    // 全不透：取本气暗藏立格
    let god = ten_god(dm, main_qi);
    Pattern {
        name: format!("暗{god}格"),
        source: "月令本气暗藏（年/月/时三干头无月令藏干）".to_string(),
        qi_stem: STEMS[main_qi as usize].to_string(),
        qi_kind: "本气".to_string(),
        revealed_in: None,
        ten_god: god.to_string(),
        revealed: false,
        is_lu_ren: false,
    }
}
