//! 释义提示词的骨架。
//!
//! 六套「吃 JSON」的意图（占事 / 择吉 / 寻方位 / 合盘 / 国运 / 团队）在内容上各不相同，
//! 但**框**是同一个：护栏在前、读法提示、盘面 JSON、可选的尾部提示。
//!
//! 这个框此前在六处各写了一遍，代价已经显形——审计时查出三处漂移：起句的方括号扣在
//! 不同成分上（四处作「【已由确定性引擎算好】的一次 X」、两处作「已由确定性引擎算好的【X】」）、
//! 两套护栏根本没有篇幅上限、合盘那套漏了读法提示的槽位。没有单一落点，改一处不会带着其余五处走。
//!
//! 本命解盘不走这里：它吃的是 [`LeafOutput`] 结构体而非 JSON 串，还要带确定性谱与主体重映射。

/// 一份释义提示词：护栏 → 读法提示 → 盘面 JSON → 尾部提示。
///
/// 渲染顺序即字段顺序，四段之间不额外插入分隔符——各段自带首尾换行，
/// 这样拼出来与手写 `format!` 逐字节相同。
#[derive(Debug, Clone)]
pub struct Prompt {
    guardrail: &'static str,
    hints: Option<&'static str>,
    json_header: &'static str,
    json: String,
    trailer: &'static str,
}

impl Prompt {
    /// 起一份提示词，先放护栏。
    #[must_use]
    pub const fn new(guardrail: &'static str) -> Self {
        Self { guardrail, hints: None, json_header: "", json: String::new(), trailer: "" }
    }

    /// 读法提示，紧接护栏之后。不给则该段为空。
    #[must_use]
    pub const fn hints(mut self, hints: &'static str) -> Self {
        self.hints = Some(hints);
        self
    }

    /// 盘面 JSON 及其抬头。抬头要自带首尾换行（如 `"\nX 结果 JSON：\n"`）。
    #[must_use]
    pub fn json(mut self, header: &'static str, body: &str) -> Self {
        self.json_header = header;
        self.json = body.to_string();
        self
    }

    /// JSON **之后**的尾部——团队合盘的字段语义提示走这里，其余六套只放一个换行。
    #[must_use]
    pub const fn trailer(mut self, trailer: &'static str) -> Self {
        self.trailer = trailer;
        self
    }

    /// 渲染成完整提示词。
    #[must_use]
    pub fn render(&self) -> String {
        let mut s = String::with_capacity(
            self.guardrail.len()
                + self.hints.map_or(0, str::len)
                + self.json_header.len()
                + self.json.len()
                + self.trailer.len(),
        );
        s.push_str(self.guardrail);
        if let Some(h) = self.hints {
            s.push_str(h);
        }
        s.push_str(self.json_header);
        s.push_str(&self.json);
        s.push_str(self.trailer);
        s
    }
}
