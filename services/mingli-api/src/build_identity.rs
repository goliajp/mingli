//! Compiled service identity; no runtime working-tree inspection.
use axum::{extract::Request, middleware::Next, response::Response, Json};
use serde::Serialize;

/// Identity of the source inputs and compiler configuration embedded in this binary.
#[derive(Serialize)]
pub struct BuildIdentity {
    /// Wire protocol version.
    pub schema_version: i32,
    /// Service name.
    pub service: &'static str,
    /// Deterministic source and compiler-configuration identifier, not a binary-file checksum.
    pub build_id: &'static str,
    /// SHA-256 of sorted, length-framed repository build inputs, including Cargo.lock.
    pub source_sha256: &'static str,
    /// Cargo package version.
    pub package_version: &'static str,
}
/// Return the identity compiled into the responding process.
pub async fn identity() -> Json<BuildIdentity> {
    Json(BuildIdentity {
        schema_version: 1,
        service: "mingli-api",
        build_id: env!("MINGLI_BUILD_ID"),
        source_sha256: env!("MINGLI_SOURCE_SHA256"),
        package_version: env!("CARGO_PKG_VERSION"),
    })
}
/// Stamp the actual response, so callers can reject a restart between identity lookup and computation.
pub async fn stamp(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "x-mingli-build-id",
        env!("MINGLI_BUILD_ID")
            .parse()
            .expect("compiled ASCII identity"),
    );
    response
}
