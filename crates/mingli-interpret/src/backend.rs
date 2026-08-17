//! 释义后端与结果类型：[`Interpreter`] 抽象、离线 [`Template`] 兜底、[`Interpretation`] 结果。

/// 释义后端抽象（可替换：离线 [`Template`] / claude CLI / 其它 LLM）。
pub trait Interpreter {
    /// 由提示词产出释义文本。
    ///
    /// # Errors
    /// 后端不可用 / 调用失败时返回错误。
    fn interpret(&self, prompt: &str) -> std::io::Result<String>;
    /// 后端标识（展示用，如 `"claude-cli"` / `"template"`）。
    fn backend(&self) -> &'static str;
}

impl Interpreter for Template {
    fn interpret(&self, prompt: &str) -> std::io::Result<String> {
        // 仅从提示词里回显护栏意图 + 标记这是模板而非 LLM。确定性。
        let has_und = prompt.contains("UND 欠定");
        let mut out = String::from("（模板转述·非 LLM）此盘面各项已由确定性引擎算出；");
        out.push_str("DET 项为确定结果、忠实呈现");
        if has_und {
            out.push_str("，🟡UND 项流派分歧或引擎诚实留空、未予杜撰");
        }
        out.push_str("。仅供研究与娱乐。");
        Ok(out)
    }
    fn backend(&self) -> &'static str {
        "template"
    }
}

/// 离线确定性后端：不调 LLM，忠实把确定性谱转成一段话（无 LLM 时的诚实兜底，且可校验）。
#[derive(Debug, Default)]
pub struct Template;

/// 一次释义的结果（标 🔮INT，非计算）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct Interpretation {
    /// 叶 id。
    pub leaf: String,
    /// 释义文本。
    pub text: String,
    /// 后端标识。
    pub backend: &'static str,
    /// 始终为 INT（提醒前端：这是 LLM/模板生成，非确定计算）。
    pub kind: &'static str,
}
