//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// 七政四余（中国本土星占）叶（B 族）。仅 `qizhengsiyu` feature 开启时编译。
/// 10 体黄经（七政 + 罗㬋/计都/月孛三余；紫炁 🟡 不入） + 28 宿值日 + 12 sign 归宫。
#[derive(Debug, Default)]
pub struct QizhengsiyuEngine;

impl CastingEngine for QizhengsiyuEngine {
    fn id(&self) -> &'static str {
        "qizhengsiyu"
    }
    fn name(&self) -> &'static str {
        "七政四余"
    }
    fn family(&self) -> Family {
        Family::Angular
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(crate::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("七政地心黄经（日月水金火木土）", Det, "VSOP87 行星 + ELP-2000/82 月亮(apparent)，太阳校验 Meeus 0.02°"),
            d("罗㬋（月平升交点）", Det, "Meeus AA 第 47 章 eq 47.7 精确五项 + SOFA J2000 校验 0.14″；通行近代取升交点"),
            d("计都 = 罗㬋 + 180°", Det, "汤若望《时宪历》后通行近代/印度对位"),
            d("月孛（月平远地点）", Det, "Meeus AA p.343 月平近地点 + 180°，PyMeeus/NASA GSFC/soniakeys 三源系数字符级一致"),
            d("沈括古版四余流派", Und, "🟡 沈括《梦溪笔谈》古法定义在二手源中相互矛盾：某源称罗=升交点/计=月远地点；另源称罗=降交点。原典考证未稳，本叶不提供沈括版流派，只走通行近代版"),
            d("28 宿值日", Det, "(JDN+11) mod 28，跨 5 锚点 341 年交叉验证（沿 zeri::mansion）"),
            d("12 sign 归宫（30° 等分）", Det, "回归黄道整宫归宫，天文公认无歧义"),
            d("紫炁", Und, "🟡 无天文实体；中文维基明文「找不著对应的天文现象」；五种互不兼容定义（虚星/月近地点/月轨中点/木余气/天狼星）无可代入时间公式，swisseph 等主流库均不提供"),
            d("12 次落宫（星纪/玄枵/...）", Und, "🟡 古籍三源分歧实质：《尔雅》给标志宿、《汉书·律历志》按度数（多宿跨次）、通行表整宿归一次；不强编"),
            d("28 宿分黄道（古制不等长）", Und, "🟡 每宿距度由观测得 + 岁差校正涉大查表；本叶只做值日 （JDN 周期）"),
        ] }
    }
}
