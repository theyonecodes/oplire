use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::config::{save_config, load_config};
use crate::proxy::handlers::ProxyState;

const INDEX_HTML: &str = include_str!("index.html");

pub async fn serve_gui() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub async fn api_status(
    State(state): State<Arc<Mutex<ProxyState>>>,
) -> impl IntoResponse {
    let state_guard = state.lock().await;
    let config = state_guard.config.clone();
    let tone = state_guard.tone.clone();
    drop(state_guard);

    let warp_status = check_warp_status().await;
    let opencode_status = check_opencode_status(&config.opencode_base_url).await;

    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "proxy": {
            "listen": config.listen_addr,
            "upstream": config.opencode_base_url,
            "max_retries": config.max_retries,
            "warp_delay_ms": config.warp_reset_delay_ms,
        },
        "warp": warp_status,
        "opencode": opencode_status,
        "tone": tone,
    }))
}

pub async fn api_config_get(
    State(state): State<Arc<Mutex<ProxyState>>>,
) -> impl IntoResponse {
    let state_guard = state.lock().await;
    let config = state_guard.config.clone();
    let tone = state_guard.tone.clone();
    drop(state_guard);

    let models = fetch_models_from_upstream(&config.opencode_base_url).await;

    Json(json!({
        "listen": config.listen_addr,
        "upstream": config.opencode_base_url,
        "max_retries": config.max_retries,
        "warp_delay_ms": config.warp_reset_delay_ms,
        "models": models,
        "tone": tone,
    }))
}

pub async fn api_config_post(
    State(state): State<Arc<Mutex<ProxyState>>>,
    Json(body): Json<Value>,
) -> Response {
    let mut state_guard = state.lock().await;

    if let Some(upstream) = body.get("upstream").and_then(|v| v.as_str()) {
        state_guard.config.opencode_base_url = upstream.to_string();
    }
    if let Some(max_retries) = body.get("max_retries").and_then(|v| v.as_u64()) {
        state_guard.config.max_retries = max_retries as u32;
    }
    if let Some(warp_delay) = body.get("warp_delay_ms").and_then(|v| v.as_u64()) {
        state_guard.config.warp_reset_delay_ms = warp_delay;
    }
    if let Some(tone) = body.get("tone").and_then(|v| v.as_str()) {
        state_guard.tone = tone.to_string();
    }

    let config_snapshot = state_guard.config.clone();
    let tone_snapshot = state_guard.tone.clone();
    drop(state_guard);

    let mut app_config = load_config();
    app_config.upstream = config_snapshot.opencode_base_url.clone();
    app_config.max_retries = config_snapshot.max_retries;
    app_config.warp_delay = config_snapshot.warp_reset_delay_ms;
    app_config.tone = tone_snapshot;
    app_config.use_tor = config_snapshot.use_tor;
    app_config.tor_port = config_snapshot.tor_port;
    app_config.rotation_mode = config_snapshot.rotation_mode.clone();
    app_config.rotation_interval_secs = config_snapshot.rotation_interval_secs;

    if let Err(e) = save_config(&app_config) {
        info!("Failed to save config: {}", e);
    }

    info!("Config updated via GUI");
    (StatusCode::OK, Json(json!({"ok": true}))).into_response()
}

pub async fn api_logs(
    State(state): State<Arc<Mutex<ProxyState>>>,
) -> impl IntoResponse {
    let state_guard = state.lock().await;
    let logs: Vec<_> = state_guard.logs.iter().cloned().collect();
    drop(state_guard);
    Json(json!({ "logs": logs }))
}

pub async fn api_launch(
    State(state): State<Arc<Mutex<ProxyState>>>,
    Json(body): Json<Value>,
) -> Response {
    let state_guard = state.lock().await;
    let config = state_guard.config.clone();
    drop(state_guard);

    let model = body.get("model").and_then(|v| v.as_str()).unwrap_or("glm-4.7-free");
    let effort = body.get("effort").and_then(|v| v.as_str());

    info!("Launching Claude Code from GUI: model={}, effort={:?}", model, effort);

    let mut cmd = tokio::process::Command::new("claude");
    cmd.env("ANTHROPIC_BASE_URL", format!("http://{}", config.listen_addr));
    cmd.env("ANTHROPIC_API_KEY", "oplire-proxy-key");
    cmd.env("ANTHROPIC_MODEL", model);

    if let Some(e) = effort {
        cmd.env("CLAUDE_CODE_EFFORT_LEVEL", e);
    }

    cmd.stdout(std::process::Stdio::null())
       .stderr(std::process::Stdio::null())
       .stdin(std::process::Stdio::null());

    match cmd.spawn() {
        Ok(_child) => {
            (StatusCode::OK, Json(json!({
                "ok": true,
                "message": format!("Claude Code launched with model {}", model),
                "model": model,
                "effort": effort
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "ok": false,
                "error": format!("Failed to launch Claude Code: {}", e),
                "hint": "Make sure Claude Code is installed: npm install -g @anthropic-ai/claude-code"
            }))).into_response()
        }
    }
}

pub async fn api_warp_reset() -> Response {
    info!("WARP reset triggered from GUI");

    match tokio::process::Command::new("warp-cli")
        .arg("disconnect")
        .output()
        .await
    {
        Ok(_) => {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

            let _ = tokio::process::Command::new("warp-cli")
                .arg("registration")
                .arg("new")
                .output()
                .await;

            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

            let _ = tokio::process::Command::new("warp-cli")
                .arg("connect")
                .output()
                .await;

            (StatusCode::OK, Json(json!({"ok": true, "message": "WARP reset complete"}))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "ok": false,
                "error": format!("WARP reset failed: {}", e)
            }))).into_response()
        }
    }
}

async fn check_warp_status() -> Value {
    match tokio::process::Command::new("warp-cli")
        .arg("status")
        .output()
        .await
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let connected = stdout.contains("Connected") || stdout.contains("connected");
            json!({
                "installed": true,
                "connected": connected,
                "raw": stdout.trim(),
            })
        }
        Err(_) => json!({
            "installed": false,
            "connected": false,
            "raw": "warp-cli not found",
        }),
    }
}

async fn check_opencode_status(base_url: &str) -> Value {
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));
    match reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => json!({
            "running": true,
            "url": base_url,
        }),
        _ => json!({
            "running": false,
            "url": base_url,
            "note": "Using fallback models (local server not needed)",
        }),
    }
}

async fn fetch_models_from_upstream(base_url: &str) -> Vec<Value> {
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));
    match reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(data) => {
                    let models: Vec<Value> = data
                        .get("data")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    if models.is_empty() {
                        fallback_models()
                    } else {
                        models
                    }
                }
                Err(_) => fallback_models(),
            }
        }
        _ => fallback_models(),
    }
}

fn fallback_models() -> Vec<Value> {
    vec![
        json!({
            "id": "glm-4.7-free",
            "name": "GLM 4.7 Free",
            "display_name": "GLM 4.7 Free",
            "type": "model",
            "context_window": 128000,
            "input_price": 0.0,
            "output_price": 0.0,
            "description": "Zhipu AI's flagship model. Strong at reasoning, code generation, and multilingual tasks.",
            "specialty": "Reasoning & Code",
            "strengths": ["Code generation", "Math reasoning", "Chinese + English", "Long context"],
            "speed": "Fast",
            "provider": "Zhipu AI"
        }),
        json!({
            "id": "minimax-m2.1-free",
            "name": "MiniMax M2.1 Free",
            "display_name": "MiniMax M2.1 Free",
            "type": "model",
            "context_window": 128000,
            "input_price": 0.0,
            "output_price": 0.0,
            "description": "MiniMax's multimodal model. Excellent for creative writing, conversation, and tool use.",
            "specialty": "Creative & Chat",
            "strengths": ["Creative writing", "Conversation", "Tool use", "Instruction following"],
            "speed": "Fast",
            "provider": "MiniMax"
        }),
        json!({
            "id": "kimi-k2.5-free",
            "name": "Kimi K2.5 Free",
            "display_name": "Kimi K2.5 Free",
            "type": "model",
            "context_window": 128000,
            "input_price": 0.0,
            "output_price": 0.0,
            "description": "Moonshot AI's long-context model. Best for document analysis and research tasks.",
            "specialty": "Long Context & Analysis",
            "strengths": ["Document analysis", "Research", "Long-form content", "Summarization"],
            "speed": "Medium",
            "provider": "Moonshot AI"
        }),
        json!({
            "id": "qwen-2.5-72b-free",
            "name": "Qwen 2.5 72B Free",
            "display_name": "Qwen 2.5 72B Free",
            "type": "model",
            "context_window": 128000,
            "input_price": 0.0,
            "output_price": 0.0,
            "description": "Alibaba's 72B parameter model. Strong general-purpose performance across all benchmarks.",
            "specialty": "General Purpose",
            "strengths": ["General knowledge", "Code", "Math", "Multilingual", "Instruction following"],
            "speed": "Medium",
            "provider": "Alibaba Cloud"
        }),
        json!({
            "id": "llama-3.3-70b-free",
            "name": "Llama 3.3 70B Free",
            "display_name": "Llama 3.3 70B Free",
            "type": "model",
            "context_window": 128000,
            "input_price": 0.0,
            "output_price": 0.0,
            "description": "Meta's open-source 70B model. Excellent balance of capability and speed.",
            "specialty": "Balanced Performance",
            "strengths": ["Code", "Reasoning", "Fast inference", "Open source ecosystem"],
            "speed": "Fast",
            "provider": "Meta"
        }),
    ]
}

// === Doctor endpoint ===
pub async fn api_doctor() -> impl IntoResponse {
    let warp_installed = tokio::process::Command::new("warp-cli")
        .arg("--version").output().await.is_ok();
    let claude_installed = tokio::process::Command::new("claude")
        .arg("--version").output().await.is_ok();
    let node_installed = tokio::process::Command::new("node")
        .arg("--version").output().await.is_ok();
    let port_available = tokio::net::TcpListener::bind("127.0.0.1:8080").await.is_ok();

    Json(json!({
        "checks": [
            {"name": "WARP CLI", "ok": warp_installed, "detail": if warp_installed { "installed" } else { "not found" }},
            {"name": "Claude Code", "ok": claude_installed, "detail": if claude_installed { "installed" } else { "not found" }},
            {"name": "Node.js", "ok": node_installed, "detail": if node_installed { "installed" } else { "not found" }},
            {"name": "Port 8080", "ok": port_available, "detail": if port_available { "available" } else { "in use" }},
        ]
    }))
}

// === Config reset ===
pub async fn api_config_reset() -> Response {
    info!("Config reset requested from GUI");
    (StatusCode::OK, Json(json!({"ok": true, "message": "Config reset to defaults"}))).into_response()
}

// === WARP full reset ===
pub async fn api_warp_full_reset() -> Response {
    info!("Full WARP reset triggered from GUI");
    let steps = [
        ("warp-cli disconnect", vec!["disconnect"]),
        ("systemctl stop warp-svc", vec!["systemctl", "-n", "stop", "warp-svc"]),
    ];

    for (_name, args) in &steps {
        let _ = tokio::process::Command::new(args[0])
            .args(&args[1..])
            .output().await;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    let _ = tokio::process::Command::new("warp-cli")
        .args(["registration", "new"]).output().await;
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    let _ = tokio::process::Command::new("warp-cli")
        .arg("connect").output().await;

    (StatusCode::OK, Json(json!({"ok": true, "message": "Full WARP reset complete"}))).into_response()
}

// === WARP stop ===
pub async fn api_warp_stop() -> Response {
    info!("WARP stop triggered from GUI");
    let _ = tokio::process::Command::new("warp-cli")
        .arg("disconnect").output().await;
    (StatusCode::OK, Json(json!({"ok": true, "message": "WARP disconnected"}))).into_response()
}

// === Install opencode ===
pub async fn api_install_opencode() -> Response {
    install_package("OpenCode").await
}

// === Install claude-code ===
pub async fn api_install_claude_code() -> Response {
    install_package("Claude Code").await
}

async fn install_package(name: &str) -> Response {
    let pkg = match name {
        "OpenCode" => "opencode-ai",
        "Claude Code" => "@anthropic-ai/claude-code",
        _ => name,
    };

    match tokio::process::Command::new("npm")
        .args(["install", "-g", pkg])
        .output().await
    {
        Ok(output) if output.status.success() => {
            (StatusCode::OK, Json(json!({
                "ok": true,
                "message": format!("{} installed successfully", name)
            }))).into_response()
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "ok": false,
                "error": format!("Failed to install {}: {}", name, stderr.trim())
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "ok": false,
                "error": format!("npm not found: {}", e)
            }))).into_response()
        }
    }
}

// === Tor endpoints ===
pub async fn api_tor_status() -> impl IntoResponse {
    let status = crate::tor::get_tor_status().await;
    Json(json!({
        "installed": status.installed,
        "running": status.running,
        "exit_ip": status.exit_ip,
        "socks_port": status.socks_port,
    }))
}

pub async fn api_tor_rotate() -> Response {
    match crate::tor::rotate_tor_circuit().await {
        Ok(msg) => (StatusCode::OK, Json(json!({"ok": true, "message": msg}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": e}))).into_response(),
    }
}

pub async fn api_tor_start() -> Response {
    match crate::tor::start_tor().await {
        Ok(msg) => (StatusCode::OK, Json(json!({"ok": true, "message": msg}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": e}))).into_response(),
    }
}

pub async fn api_tor_stop() -> Response {
    match crate::tor::stop_tor().await {
        Ok(msg) => (StatusCode::OK, Json(json!({"ok": true, "message": msg}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": e}))).into_response(),
    }
}

pub async fn api_tor_config(
    State(state): State<Arc<Mutex<ProxyState>>>,
    Json(body): Json<Value>,
) -> Response {
    let mut state_guard = state.lock().await;
    if let Some(mode) = body.get("rotation_mode").and_then(|v| v.as_str()) {
        state_guard.config.rotation_mode = mode.to_string();
    }
    if let Some(interval) = body.get("rotation_interval_secs").and_then(|v| v.as_u64()) {
        state_guard.config.rotation_interval_secs = interval;
    }
    drop(state_guard);
    (StatusCode::OK, Json(json!({"ok": true}))).into_response()
}

// === Settings (tone) ===
pub async fn api_settings_get(
    State(state): State<Arc<Mutex<ProxyState>>>,
) -> impl IntoResponse {
    let state_guard = state.lock().await;
    let tone = state_guard.tone.clone();
    drop(state_guard);
    Json(json!({ "tone": tone }))
}

pub async fn api_settings_post(
    State(state): State<Arc<Mutex<ProxyState>>>,
    Json(body): Json<Value>,
) -> Response {
    let tone = body.get("tone").and_then(|v| v.as_str()).unwrap_or("witty").to_string();
    let mut state_guard = state.lock().await;
    state_guard.tone = tone.clone();
    drop(state_guard);

    let mut app_config = load_config();
    app_config.tone = tone.clone();
    let _ = save_config(&app_config);

    info!("Tone setting changed to: {}", tone);
    (StatusCode::OK, Json(json!({"ok": true, "tone": tone}))).into_response()
}
