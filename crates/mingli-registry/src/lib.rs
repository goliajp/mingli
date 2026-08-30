//! 装配根：把各叶的适配器装配成注册表。
//!
//! 这是整棵树里**唯一**知道「有哪些叶」的地方。叶只认识 [`mingli_contract`] 的契约、
//! 编排层只认识注册表里的 `dyn CastingEngine`，两边互不认识——加一片新叶就是新写一个
//! crate 再来这里登记一行，根、共享层、编排层都不动。
//!
//! feature 开关也落在这里：关掉 `astrology` / `jyotish` / `qizhengsiyu` 即可得到不含
//! VSOP87 星历的轻量构建。

use mingli_contract::{CastingEngine, WordEngine};

/// 各叶的类型化 API，按 feature 转发。
///
/// 门面 crate（`mingli`）从这里取叶，而不是在自己的清单里再列一遍二十四行——
/// 再列一遍就是第二个装配根，漏掉一片不会有任何报错，只会让那片叶从门面上消失。
/// 本模块与 [`registry`] 用的是同一批 `#[cfg]`，两者不可能各说各话。
pub mod leaves {
    /// `bazi`。
    #[cfg(feature = "bazi")]
    pub use mingli_bazi as bazi;
    /// `ziwei`。
    #[cfg(feature = "ziwei")]
    pub use mingli_ziwei as ziwei;
    /// `astrology`。`astrology-lite` 下只有排盘部分，本地星历不在。
    #[cfg(any(feature = "astrology", feature = "astrology-lite", feature = "astrology-thin"))]
    pub use mingli_astrology as astrology;
    /// `jyotish`。
    #[cfg(feature = "jyotish")]
    pub use mingli_jyotish as jyotish;
    /// `qizhengsiyu`。
    #[cfg(feature = "qizhengsiyu")]
    pub use mingli_qizhengsiyu as qizhengsiyu;
    /// `yijing`。
    #[cfg(feature = "yijing")]
    pub use mingli_yijing as yijing;
    /// `geomancy`。
    #[cfg(feature = "geomancy")]
    pub use mingli_geomancy as geomancy;
    /// `sikidy`。
    #[cfg(feature = "sikidy")]
    pub use mingli_sikidy as sikidy;
    /// `ifa`。
    #[cfg(feature = "ifa")]
    pub use mingli_ifa as ifa;
    /// `cartomancy`。
    #[cfg(feature = "cartomancy")]
    pub use mingli_cartomancy as cartomancy;
    /// `meihua`。
    #[cfg(feature = "meihua")]
    pub use mingli_meihua as meihua;
    /// `xiaoliuren`。
    #[cfg(feature = "xiaoliuren")]
    pub use mingli_xiaoliuren as xiaoliuren;
    /// `zeri`。
    #[cfg(feature = "zeri")]
    pub use mingli_zeri as zeri;
    /// `maya`。
    #[cfg(feature = "maya")]
    pub use mingli_maya as maya;
    /// `pawukon`。
    #[cfg(feature = "pawukon")]
    pub use mingli_pawukon as pawukon;
    /// `mahabote`。
    #[cfg(feature = "mahabote")]
    pub use mingli_mahabote as mahabote;
    /// `liuren`。
    #[cfg(feature = "liuren")]
    pub use mingli_liuren as liuren;
    /// `qimen`。
    #[cfg(feature = "qimen")]
    pub use mingli_qimen as qimen;
    /// `taiyi`。
    #[cfg(feature = "taiyi")]
    pub use mingli_taiyi as taiyi;
    /// `tibetan`。
    #[cfg(feature = "tibetan")]
    pub use mingli_tibetan as tibetan;
    /// `numerology`。
    #[cfg(feature = "numerology")]
    pub use mingli_numerology as numerology;
    /// `gematria`。
    #[cfg(feature = "gematria")]
    pub use mingli_gematria as gematria;
    /// `abjad`。
    #[cfg(feature = "abjad")]
    pub use mingli_abjad as abjad;
    /// `wuge`。
    #[cfg(feature = "wuge")]
    pub use mingli_wuge as wuge;
}


/// 已登记的全部**时刻叶**（吃出生/占问时刻，进并行 fan-out）。
///
/// 顺序即 `/api/cast` 与 `/api/health` 的输出顺序，属对外契约的一部分。
#[must_use]
pub fn registry() -> Vec<Box<dyn CastingEngine>> {
    vec![
        #[cfg(feature = "bazi")]
        Box::new(mingli_bazi::BaziEngine),
        #[cfg(feature = "ziwei")]
        Box::new(mingli_ziwei::ZiweiEngine),
        #[cfg(any(feature = "astrology", feature = "astrology-thin"))]
        Box::new(mingli_astrology::AstrologyEngine),
        #[cfg(feature = "jyotish")]
        Box::new(mingli_jyotish::JyotishEngine),
        #[cfg(feature = "qizhengsiyu")]
        Box::new(mingli_qizhengsiyu::QizhengsiyuEngine),
        #[cfg(feature = "yijing")]
        Box::new(mingli_yijing::YijingEngine),
        #[cfg(feature = "geomancy")]
        Box::new(mingli_geomancy::GeomancyEngine),
        #[cfg(feature = "sikidy")]
        Box::new(mingli_sikidy::SikidyEngine),
        #[cfg(feature = "ifa")]
        Box::new(mingli_ifa::IfaEngine),
        #[cfg(feature = "cartomancy")]
        Box::new(mingli_cartomancy::TarotEngine),
        #[cfg(feature = "meihua")]
        Box::new(mingli_meihua::MeihuaEngine),
        #[cfg(feature = "xiaoliuren")]
        Box::new(mingli_xiaoliuren::XiaoliurenEngine),
        #[cfg(feature = "zeri")]
        Box::new(mingli_zeri::ZeriEngine),
        #[cfg(feature = "maya")]
        Box::new(mingli_maya::MayaEngine),
        #[cfg(feature = "pawukon")]
        Box::new(mingli_pawukon::PawukonEngine),
        #[cfg(feature = "mahabote")]
        Box::new(mingli_mahabote::MahaboteEngine),
        #[cfg(feature = "liuren")]
        Box::new(mingli_liuren::LiurenEngine),
        #[cfg(feature = "qimen")]
        Box::new(mingli_qimen::QimenEngine),
        #[cfg(feature = "taiyi")]
        Box::new(mingli_taiyi::TaiyiEngine),
        #[cfg(feature = "tibetan")]
        Box::new(mingli_tibetan::TibetanEngine),
        #[cfg(feature = "numerology")]
        Box::new(mingli_numerology::NumerologyEngine),
    ]
}

/// 已登记的全部**字词叶**（D 族：吃文字或笔画，与时刻无关，不进 fan-out）。
///
/// 数字学同时在这里与 [`registry`] 里各占一行：它的生命灵数由出生日期得（时刻那一半），
/// 姓名三数只吃字（字词这一半）。两条端口各答各的问局，不是同一份东西登记两遍。
#[must_use]
pub fn word_registry() -> Vec<Box<dyn WordEngine>> {
    vec![
        #[cfg(feature = "gematria")]
        Box::new(mingli_gematria::GematriaEngine),
        #[cfg(feature = "abjad")]
        Box::new(mingli_abjad::AbjadEngine),
        #[cfg(feature = "wuge")]
        Box::new(mingli_wuge::WugeEngine),
        #[cfg(feature = "numerology")]
        Box::new(mingli_numerology::NumerologyEngine),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 默认 feature 下的叶顺序。**这是对外契约**——`/api/cast` 与 `/api/health` 按此序输出，
    /// 前端的叶签也照此排。改动这张表就是改动接口，不是内部调整。
    #[cfg(all(feature = "astrology", feature = "jyotish", feature = "qizhengsiyu"))]
    const EXPECTED: [&str; 21] = [
        "bazi", "ziwei", "astrology", "jyotish", "qizhengsiyu", "yijing", "geomancy", "sikidy",
        "ifa", "tarot", "meihua", "xiaoliuren", "zeri", "maya", "pawukon", "mahabote", "liuren",
        "qimen", "taiyi", "tibetan", "numerology",
    ];

    /// 三个星历 feature 全关时的叶顺序——其余十八片的**相对次序不变**，只是少了三片。
    #[cfg(not(any(feature = "astrology", feature = "jyotish", feature = "qizhengsiyu")))]
    const EXPECTED: [&str; 18] = [
        "bazi", "ziwei", "yijing", "geomancy", "sikidy", "ifa", "tarot", "meihua", "xiaoliuren",
        "zeri", "maya", "pawukon", "mahabote", "liuren", "qimen", "taiyi", "tibetan", "numerology",
    ];

    /// 只开 `astrology` 时。
    #[cfg(all(feature = "astrology", not(feature = "jyotish"), not(feature = "qizhengsiyu")))]
    const EXPECTED: [&str; 19] = [
        "bazi", "ziwei", "astrology", "yijing", "geomancy", "sikidy", "ifa", "tarot", "meihua",
        "xiaoliuren", "zeri", "maya", "pawukon", "mahabote", "liuren", "qimen", "taiyi", "tibetan",
        "numerology",
    ];

    /// 顺序锁定，不只是集合。
    ///
    /// 从前只断言「共 21 片、id 唯一」——那挡不住有人调换两行的位置，
    /// 而调换会改变 `/api/cast` 数组的下标，前端按下标取的地方就全错位了。
    #[test]
    fn the_registry_order_is_part_of_the_contract() {
        let ids: Vec<&str> = registry().iter().map(|e| e.id()).collect();
        assert_eq!(ids, EXPECTED, "注册表顺序即 /api/cast 的输出顺序，改它就是改接口");
    }

    /// 三片星历叶由 feature 开关控制，其余十八片在任何组合下都在，且相对次序不变。
    #[test]
    fn the_optional_leaves_are_the_only_thing_features_change() {
        const OPTIONAL: [&str; 3] = ["astrology", "jyotish", "qizhengsiyu"];
        const ALWAYS: [&str; 18] = [
            "bazi", "ziwei", "yijing", "geomancy", "sikidy", "ifa", "tarot", "meihua",
            "xiaoliuren", "zeri", "maya", "pawukon", "mahabote", "liuren", "qimen", "taiyi",
            "tibetan", "numerology",
        ];
        let ids: Vec<&str> = registry().iter().map(|e| e.id()).collect();
        let core: Vec<&str> = ids.iter().copied().filter(|id| !OPTIONAL.contains(id)).collect();
        assert_eq!(core, ALWAYS, "非可选的十八片在任何 feature 组合下都该在，且次序不变");
        assert!(
            ids.len() >= ALWAYS.len() && ids.len() <= ALWAYS.len() + OPTIONAL.len(),
            "叶数只该在 18..=21 之间浮动，实得 {}",
            ids.len()
        );
    }

    #[test]
    fn registry_ids_are_unique_and_nonempty() {
        let reg = registry();
        let mut ids: Vec<&str> = reg.iter().map(|e| e.id()).collect();
        let n = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), n, "叶 id 必须唯一");
        assert!(reg.iter().all(|e| !e.id().is_empty() && !e.name().is_empty()));
    }

    /// 字词叶的顺序同样是契约（`/api/word` 的 system 取值与前端下拉的排序）。
    #[test]
    fn word_registry_covers_the_word_leaves_in_order() {
        let ids: Vec<&str> = word_registry().iter().map(|e| e.id()).collect();
        assert_eq!(ids, ["gematria", "abjad", "wuge", "numerology"]);
    }

    /// 同时长在两条端口上的叶，以及为什么它是同一套术数而非两样东西。
    ///
    /// 这不是豁免清单：要进这张表，两条端口得答的是同一套术数的两种模态，
    /// 且下面那条测试会要求它们的显示名一致——否则目录里会出现两个名字指同一个 id。
    const BOTH_PORTS: &[(&str, &str)] = &[(
        "numerology",
        "生命灵数由出生日期得（时刻端口），姓名三数只吃字（字词端口）；同一套术数的两种输入",
    )];

    /// 两张注册表的 id 不许撞——它们在 `/api/interpret` 与 `/api/word` 两侧共用同一个命名空间。
    ///
    /// 例外是 [`BOTH_PORTS`] 里点名的那几片：撞的不是两片叶，是同一片叶的两条端口。
    /// 对这几片另加一条要求——两侧的显示名必须一样，否则同一个 id 在目录里会有两个名字。
    #[test]
    fn the_two_registries_only_share_an_id_for_a_leaf_that_is_both() {
        let reg = registry();
        for w in word_registry() {
            let Some(c) = reg.iter().find(|e| e.id() == w.id()) else { continue };
            let why = BOTH_PORTS.iter().find(|(id, _)| *id == w.id()).map(|(_, why)| why);
            assert!(
                why.is_some(),
                "字词叶 `{}` 与时刻叶重名。若这确是同一套术数的两条端口，\n\
                 把它连同理由加进 BOTH_PORTS；若不是，换一个 id——\n\
                 两侧共用一个命名空间，重名会让 `/api/interpret` 与 `/api/word` 指向不同的东西。",
                w.id()
            );
            assert_eq!(
                c.name(),
                w.name(),
                "叶 `{}` 的两条端口给了不同的显示名——同一个 id 在目录里就成了两样东西",
                w.id()
            );
        }
    }
}
