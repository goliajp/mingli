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
    axum::serve(listener, mingli_api::router())
        .await
        .unwrap_or_else(|e| panic!("服务异常退出：{e}"));
}
