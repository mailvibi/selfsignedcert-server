use std::{
    net::{IpAddr, SocketAddr},
    process::ExitCode,
};

use axum::{
    Router,
    body::Body,
    extract::Request,
    http::{HeaderValue, StatusCode, header},
    response::Response,
    routing::get,
};
use clap::Parser;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

#[derive(Debug, Parser)]
#[command(
    name = "selfsignedcert-server",
    about = "Serve the embedded Self-Signed Certificate Generator"
)]
struct Cli {
    #[arg(long, default_value = "127.0.0.1", value_name = "IP-ADDRESS")]
    listen: IpAddr,
    #[arg(long, default_value_t = 8080, value_name = "PORT")]
    port: u16,
    #[arg(long, help = "Enable request and startup logging")]
    verbose: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let filter = if cli.verbose {
        EnvFilter::new("info")
    } else {
        EnvFilter::new("warn")
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();
    let address = SocketAddr::new(cli.listen, cli.port);
    let app = app();
    let listener = match TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%address, %error, "unable to bind server socket");
            return ExitCode::from(1);
        }
    };
    tracing::info!(%address, assets = ASSETS.len(), "server listening");
    if let Err(error) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await
    {
        tracing::error!(%error, "server stopped unexpectedly");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .fallback(static_asset)
        .layer(TraceLayer::new_for_http())
}

async fn health() -> Response<Body> {
    response(StatusCode::OK, "text/plain; charset=utf-8", b"OK\n", true)
}

async fn static_asset(request: Request) -> Response<Body> {
    let path = request.uri().path();
    if request.uri().query().is_some() || !safe_path(path) {
        return response(
            StatusCode::NOT_FOUND,
            "text/plain; charset=utf-8",
            b"Not Found\n",
            false,
        );
    }
    let path = if path == "/" { "/index.html" } else { path };
    let Some(asset) = ASSETS.iter().find(|asset| asset.path == path) else {
        return response(
            StatusCode::NOT_FOUND,
            "text/plain; charset=utf-8",
            b"Not Found\n",
            false,
        );
    };
    let mut result = response(
        StatusCode::OK,
        asset.mime,
        asset.bytes,
        path == "/index.html",
    );
    if path != "/index.html" {
        result.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    }
    result
}

fn safe_path(path: &str) -> bool {
    path.len() <= 2048
        && !path.contains('%')
        && !path.contains('\\')
        && !path.contains("//")
        && !path.split('/').any(|part| part == "." || part == "..")
}

fn response(
    status: StatusCode,
    mime: &str,
    bytes: &'static [u8],
    no_cache: bool,
) -> Response<Body> {
    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = status;
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_str(mime).unwrap());
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static(CONTENT_SECURITY_POLICY),
    );
    if no_cache {
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        );
    }
    response
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::{app, safe_path};

    #[test]
    fn rejects_ambiguous_paths() {
        assert!(safe_path("/index.html"));
        assert!(!safe_path("/%2e%2e/etc/passwd"));
        assert!(!safe_path("/a//b"));
        assert!(!safe_path("/a/../b"));
        assert!(!safe_path("/a\\b"));
    }

    #[tokio::test]
    async fn serves_health_and_rejects_unknown_assets() {
        let health = app()
            .oneshot(
                axum::http::Request::get("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(health.status(), axum::http::StatusCode::OK);
        assert_eq!(
            health
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .as_ref(),
            b"OK\n"
        );

        let missing = app()
            .oneshot(
                axum::http::Request::get("/missing.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn serves_frontend_with_hashed_inline_script_policy() {
        for path in ["/", "/index.html"] {
            let response = app()
                .oneshot(axum::http::Request::get(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), axum::http::StatusCode::OK);
            let policy = response
                .headers()
                .get("content-security-policy")
                .unwrap()
                .to_str()
                .unwrap();
            assert!(policy.contains("script-src 'self' 'wasm-unsafe-eval' 'sha256-"));
            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert!(String::from_utf8_lossy(&body).contains("Self-Signed Certificate Generator"));
        }
    }
}
