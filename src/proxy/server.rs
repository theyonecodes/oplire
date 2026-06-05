use axum::{
    middleware::{self, Next},
    routing::{get, post},
    Router,
};
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::info;

use crate::config::{ProxyConfig, load_config};
use crate::proxy::handlers::{handle_messages, handle_model_detail, handle_models, ProxyState};
use crate::warp::WarpResolver;

async fn log_requests(
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> axum::response::Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();
    info!("→ {} {}", method, uri);
    let response = next.run(request).await;
    let duration = start.elapsed().as_millis() as u64;
    let status = response.status().as_u16();
    info!("← {} {} {} ({}ms)", method, uri, status, duration);
    response
}

pub async fn start_proxy_server(config: ProxyConfig) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let warp_resolver =
        WarpResolver::new(config.max_retries, config.warp_reset_delay_ms);

    let saved_config = load_config();
    let state = Arc::new(Mutex::new(ProxyState {
        config: config.clone(),
        client,
        warp_resolver,
        tone: saved_config.tone,
        logs: VecDeque::new(),
    }));

    let app = Router::new()
        .route("/", get(crate::gui::serve_gui))
        .route("/api/status", get(crate::gui::api_status))
        .route("/api/config", get(crate::gui::api_config_get).post(crate::gui::api_config_post))
        .route("/api/config/reset", post(crate::gui::api_config_reset))
        .route("/api/logs", get(crate::gui::api_logs))
        .route("/api/launch", post(crate::gui::api_launch))
        .route("/api/doctor", get(crate::gui::api_doctor))
        .route("/api/warp/reset", post(crate::gui::api_warp_reset))
        .route("/api/warp/full-reset", post(crate::gui::api_warp_full_reset))
        .route("/api/warp/stop", post(crate::gui::api_warp_stop))
        .route("/api/install/opencode", post(crate::gui::api_install_opencode))
        .route("/api/install/claude-code", post(crate::gui::api_install_claude_code))
        .route("/api/tor/status", get(crate::gui::api_tor_status))
        .route("/api/tor/rotate", post(crate::gui::api_tor_rotate))
        .route("/api/tor/start", post(crate::gui::api_tor_start))
        .route("/api/tor/stop", post(crate::gui::api_tor_stop))
        .route("/api/tor/config", post(crate::gui::api_tor_config))
        .route("/api/settings", get(crate::gui::api_settings_get).post(crate::gui::api_settings_post))
        .route("/api/terminal/ws", get(crate::gui::terminal::handle_ws_upgrade))
        .route("/api/config/advanced", get(crate::gui::api_advanced_config_get).post(crate::gui::api_advanced_config_post))
        .route("/v1/models", get(handle_models))
        .route("/v1/models/{model_id}", get(handle_model_detail))
        .route("/v1/messages", post(handle_messages))
        .route("/v1/chat/completions", post(handle_messages))
        .route("/health", get(|| async { "OK" }))
        .layer(middleware::from_fn(log_requests))
        .with_state(state);

    let addr: SocketAddr = config
        .listen_addr
        .parse()
        .unwrap_or_else(|_| "127.0.0.1:8080".parse().unwrap());

    info!("Starting proxy server on {}", addr);
    info!("OpenCode Zen upstream: {}", config.opencode_base_url);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Proxy server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

