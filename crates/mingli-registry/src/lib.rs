//! 装配根：把各叶的适配器装配成注册表。
//!
//! 这是整棵树里**唯一**知道「有哪些叶」的地方。叶只认识 [`mingli_contract`] 的契约、
//! 编排层只认识注册表里的 `dyn CastingEngine`，两边互不认识——加一片新叶就是新写一个
//! crate 再来这里登记一行，根、共享层、编排层都不动。
//!
//! feature 开关也落在这里：关掉 `astrology` / `jyotish` / `qizhengsiyu` 即可得到不含
//! VSOP87 星历的轻量构建。

use mingli_contract::{CastingEngine, WordEngine};

/// 已登记的全部**时刻叶**（吃出生/占问时刻，进并行 fan-out）。
///
/// 顺序即 `/api/cast` 与 `/api/health` 的输出顺序，属对外契约的一部分。
#[must_use]
pub fn registry() -> Vec<Box<dyn CastingEngine>> {
    vec![
        Box::new(mingli_bazi::BaziEngine),
        Box::new(mingli_ziwei::ZiweiEngine),
        #[cfg(feature = "astrology")]
        Box::new(mingli_astrology::AstrologyEngine),
        #[cfg(feature = "jyotish")]
        Box::new(mingli_jyotish::JyotishEngine),
        #[cfg(feature = "qizhengsiyu")]
        Box::new(mingli_qizhengsiyu::QizhengsiyuEngine),
        Box::new(mingli_yijing::YijingEngine),
        Box::new(mingli_geomancy::GeomancyEngine),
        Box::new(mingli_sikidy::SikidyEngine),
        Box::new(mingli_ifa::IfaEngine),
        Box::new(mingli_cartomancy::TarotEngine),
        Box::new(mingli_meihua::MeihuaEngine),
        Box::new(mingli_xiaoliuren::XiaoliurenEngine),
        Box::new(mingli_zeri::ZeriEngine),
        Box::new(mingli_maya::MayaEngine),
        Box::new(mingli_pawukon::PawukonEngine),
        Box::new(mingli_mahabote::MahaboteEngine),
        Box::new(mingli_liuren::LiurenEngine),
        Box::new(mingli_qimen::QimenEngine),
        Box::new(mingli_taiyi::TaiyiEngine),
        Box::new(mingli_tibetan::TibetanEngine),
        Box::new(mingli_numerology::NumerologyEngine),
    ]
}

/// 已登记的全部**字词叶**（D 族：吃文字或笔画，与时刻无关，不进 fan-out）。
#[must_use]
pub fn word_registry() -> Vec<Box<dyn WordEngine>> {
    vec![
        Box::new(mingli_gematria::GematriaEngine),
        Box::new(mingli_abjad::AbjadEngine),
        Box::new(mingli_wuge::WugeEngine),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_ids_are_unique_and_nonempty() {
        let reg = registry();
        assert_eq!(reg.len(), 21, "默认 feature 下应有 21 片时刻叶");
        let mut ids: Vec<&str> = reg.iter().map(|e| e.id()).collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n, "叶 id 必须唯一");
        assert!(reg.iter().all(|e| !e.id().is_empty() && !e.name().is_empty()));
    }

    #[test]
    fn word_registry_covers_the_three_word_leaves() {
        let ids: Vec<&str> = word_registry().iter().map(|e| e.id()).collect();
        assert_eq!(ids, ["gematria", "abjad", "wuge"]);
    }
}
