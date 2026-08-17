//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query};
use serde_json::Value;

/// 七政四余（中国本土星占）叶（B 族）。仅 `qizhengsiyu` feature 开启时编译。
/// 10 体黄经（七政 + 罗㬋/计都/月孛三余；紫炁 🟡 不入） + 28 宿值日 + 12 sign 归宫。
#[derive(Debug, Default)]
pub struct QizhengsiyuEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(_e: &QizhengsiyuEngine, m: &Moment, _q: &Query) -> crate::QizhengsiyuChart {
crate::compute_at(m)
}

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
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 当日值宿。
        let c = chart(self, m, q);
        Some(Principal { label: "28 宿值日", value: c.mansion_name.to_string() })
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
            d("十二次·名与顺序", Det, "《汉书·律历志》《晋书·天文志》《旧唐书》《新唐书》《明史》五处一致"),
            d("十二次·对应十二辰", Det, "两条独立证据链：《晋书·天文志上》逐条「於辰在 X」(陈卓/班固传统) 与《旧唐书·天文志下》逐条「X 初起」(一行大衍历传统)——两家度数彼此打架而地支全同；《淮南子·天文训》太阴辰→岁星舍宿经镜像后为第三旁证"),
            d("十二次·整宿归属（近似）", Det, "《淮南子·天文训》(前 2 世纪，经辰镜像) 与《新唐书》一行表头(8 世纪) 逐条一致，等于今日通行表。标为近似：四部原典的度界都让若干宿跨次，整宿归谁只能是约定"),
            d("十二次·度界", Und, "🟡 五系并存且原典层面公开分歧：三统历(汉志/晋志)、费直《周易分野》、蔡邕《月令章句》、大衍历(两唐书)、明历——《晋书》自己就把前三家并列，且受岁差支配不可能收敛。故本叶给对照表但不由黄经推落次。另：汉志与晋志在鹑火/鹑尾分界差一度(张十七/十八 对 张十六/十七)，两版求和都恰 365 度，算术判不了，需点校本校勘记"),
            d("十二次·对应中气", Und, "🟡 只见《汉书·律历志》一处，且用汉代节气次序(立春→惊蛰→雨水→春分、谷雨→清明)；单源不写。中文维基已把它改成今序，照抄会引入错误"),
            d("28 宿距度（古制不等长，按纪元分表）", Det, "汉制两独立原典 27/28 逐宿相同（《淮南子·天文训》约前 139 ·《汉书·律历志下》约 100）；大衍历 724 见《新唐书》卷 028 上，原典自点「其畢、觜觿、參、輿鬼四宿度數與古不同」，《宋史》卷 072 崇天历独立复述；授时历 1280 见《元史》卷 054，《元史》卷 052「至元所測」栏互证。三表的四方小计与周天皆零误差对账"),
            d("宿度随纪元而变（不是常数）", Det, "距度 = 相邻两距星的赤经差，而 dα/dt 含赤纬项，各星速率不同故赤经差必漂移（黄经变化率对所有恒星同为一常数，故黄道距度才近似恒定）。三部正史历法各自明记「考古用古所测」：《新唐书》大衍、《宋史》纪元、《元史》授时。觜宿汉 2 度→唐 1 度→宋半度→元 0.05 度→明变负且觜参次序翻转，乾隆十七年改换距星才复原"),
            d("汉制 ¼ 度余分的归属", Und, "🟡 《淮南子》挂在箕（箕 11¼），《汉书》不挂、循四分历「斗分」传统留给斗（《开元占经》斗 26¼、北方 98¼ 亦然）。两读法整表都收在 365¼，不替调用方选边，两种归属并出"),
            d("明清 360 度制宿度与黄道距度", Und, "🟡 明清改 360 度制（《明史》卷二十五「赤道宿度周天三百六十度，每度六十分」），且觜参次序已翻转，与古度制不可逐行对齐；黄道距度另有完全不同的一套（《后汉书》贾逵黄道铜仪、大衍黄道、授时黄道），同代赤黄两套数差异极大。本叶只收古度制赤道三表，混用会静默出错"),
        ] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到统一契约上：元数据齐备、能出盘、确定性谱已声明、
    /// 有流派时恰有一个默认。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = QizhengsiyuEngine;
        assert!(!e.id().is_empty() && !e.name().is_empty());
        let m = Moment::new(1990, 6, 15, 14, 30, 8.0);
        let q = Query::at(1990, 6, 15, 14, 30, 8.0);
        assert!(!e.cast(&m, &q).is_null(), "每片叶都应产出非空盘面");
        assert!(!e.profile().is_empty(), "每片叶都要显式声明确定性谱");
        let defaults = e.schools().iter().filter(|s| s.default).count();
        assert!(e.schools().is_empty() || defaults == 1, "有流派的叶应恰有一个默认");
        assert!(!e.family().label().is_empty());
    }
}
