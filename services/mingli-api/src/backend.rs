//! 释义后端：claude CLI，以及「阻塞慢 I/O + 失败回退离线模板」这段舞步。

use crate::error::{bad_request, server_error};
use axum::response::{IntoResponse, Response};
use axum::Json;

/// claude CLI 释义后端（外部非确定 I/O，故置于承接层；实现 `mingli_interpret::Interpreter`，可替换）。
pub(crate) struct ClaudeCli;

impl mingli_interpret::Interpreter for ClaudeCli {
    fn interpret(&self, prompt: &str) -> std::io::Result<String> {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut child = Command::new("claude")
            .arg("-p")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .ok_or_else(|| std::io::Error::other("no stdin"))?
            .write_all(prompt.as_bytes())?;
        let out = child.wait_with_output()?;
        if !out.status.success() {
            return Err(std::io::Error::other(
                String::from_utf8_lossy(&out.stderr).to_string(),
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }
    fn backend(&self) -> &'static str {
        "claude-cli"
    }
}

/// 把一次释义跑完并映射成响应。
///
/// 五个意图释义 handler 从前各写了一遍这二十来行，逐对相似度 86–94%——
/// 骨架完全相同，只有调用哪个 `interpret_*` 不同。那是承接层的**机制**
/// （LLM 是阻塞慢 I/O，得移出异步执行器；后端起不来要回退离线模板并诚实标 backend），
/// 不是各意图的差异，所以收在这里。
///
/// `run` 会在阻塞线程上跑，故要求 `Send + 'static`。
pub async fn interpret_blocking<F, E>(run: F) -> Response
where
    F: FnOnce() -> Result<mingli_interpret::Interpretation, E> + Send + 'static,
    E: Send + 'static,
{
    match tokio::task::spawn_blocking(run).await {
        Ok(Ok(interp)) => Json(interp).into_response(),
        // 后端起不来与任务异常终止对调用方是同一件事：这边没给出释义。
        // 文案与拆分前逐字相同——承接层的对外形状不因内部重组而变。
        Ok(Err(_)) | Err(_) => server_error("释义后端不可用"),
    }
}

/// 先算出结果，再交释义——五个意图 handler 共用的两步。
///
/// 算不出来是调用方的问题（400），释义跑不动是我们的问题（500），两者分开。
pub async fn cast_then_interpret<T, C, I, E>(cast: C, interpret: I) -> Response
where
    T: serde::Serialize,
    C: FnOnce() -> Result<T, String>,
    I: FnOnce(String) -> Result<mingli_interpret::Interpretation, E> + Send + 'static,
    E: Send + 'static,
{
    let value = match cast() {
        Ok(v) => v,
        Err(e) => return bad_request(e),
    };
    let json = serde_json::to_string(&value).unwrap_or_default();
    interpret_blocking(move || interpret(json)).await
}
