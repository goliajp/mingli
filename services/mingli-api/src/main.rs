//! 服务入口：装路由、绑端口、跑起来。业务全在 [`mingli_api`] 里。

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_target(false).init();

    // 端口由 port-registry 分配（mingli-api → 6027）；可用 MINGLI_API_BIND 覆盖。
    let addr = std::env::var("MINGLI_API_BIND").unwrap_or_else(|_| "127.0.0.1:6027".to_string());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("绑定 {addr} 失败：{e}"));
    tracing::info!("mingli-api listening on http://{addr}");
    // 释义后端：默认 claude CLI；`MINGLI_INTERPRET_BACKEND=template` 时走离线模板。
    //
    // 这个开关是给**基准比对**留的：离线模板的输出是确定的，于是释义端点的 200 路径
    // 也能纳入逐字节快照；走 CLI 则每次不同，只能比它的 400 路径。
    // 读环境变量是安全的——当初不用它，是因为测试里 `set_var` 在本 crate 编不过（forbid unsafe）。
    let backend = match std::env::var("MINGLI_INTERPRET_BACKEND").as_deref() {
        Ok("template") => mingli_api::backend::Interpret::Offline,
        _ => mingli_api::backend::Interpret::Cli,
    };
    axum::serve(listener, mingli_api::router_with(backend))
        .await
        .unwrap_or_else(|e| panic!("服务异常退出：{e}"));
}
