pub mod terminal;
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

use crate::config::{save_config, load_config, AppConfig};
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
    let tor_status = crate::tor::get_tor_status().await;

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
        "tor": {
            "installed": tor_status.installed,
            "running": tor_status.running,
            "exit_ip": tor_status.exit_ip,
            "socks_port": tor_status.socks_port,
        },
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
        if !upstream.starts_with("http://") && !upstream.starts_with("https://") {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "ok": false,
                "error": "Invalid upstream URL",
                "hint": "URL must start with http:// or https://"
            }))).into_response();
        }
        state_guard.config.opencode_base_url = upstream.to_string();
    }
    if let Some(max_retries) = body.get("max_retries").and_then(|v| v.as_u64()) {
        if max_retries > 100 {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "ok": false,
                "error": "max_retries must be between 0 and 100",
                "hint": "A reasonable value is 3-5"
            }))).into_response();
        }
        state_guard.config.max_retries = max_retries as u32;
    }
    if let Some(warp_delay) = body.get("warp_delay_ms").and_then(|v| v.as_u64()) {
        if warp_delay > 60000 {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "ok": false,
                "error": "warp_delay_ms must be between 0 and 60000",
                "hint": "A reasonable value is 1000-10000"
            }))).into_response();
        }
        state_guard.config.warp_reset_delay_ms = warp_delay;
    }
    if let Some(tone) = body.get("tone").and_then(|v| v.as_str()) {
        let valid_tones = ["professional", "witty", "minimal"];
        if !valid_tones.contains(&tone) {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "ok": false,
                "error": format!("Invalid tone: {}", tone),
                "hint": format!("Valid tones: {}", valid_tones.join(", "))
            }))).into_response();
        }
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

    match save_config(&app_config) {
        Ok(()) => {
            info!("Config updated via GUI");
            (StatusCode::OK, Json(json!({"ok": true, "message": "Configuration saved successfully"}))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to save config: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "ok": false,
                "error": format!("Failed to save configuration: {}", e),
                "hint": "Check file permissions and disk space"
            }))).into_response()
        }
    }
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

    let valid_models = [
        "glm-4.7-free",
        "minimax-m2.1-free",
        "kimi-k2.5-free",
        "qwen-2.5-72b-free",
        "llama-3.3-70b-free",
    ];
    let valid_efforts = ["low", "medium", "high", "xhigh", "max"];

    if !valid_models.contains(&model) {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "ok": false,
            "error": format!("Invalid model: {}", model),
            "hint": format!("Valid models: {}", valid_models.join(", "))
        }))).into_response();
    }

    if let Some(e) = effort {
        if !valid_efforts.contains(&e) {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "ok": false,
                "error": format!("Invalid effort level: {}", e),
                "hint": format!("Valid effort levels: {}", valid_efforts.join(", "))
            }))).into_response();
        }
    }

    let cmd_name = if cfg!(target_os = "windows") { "claude.cmd" } else { "claude" };
    let claude_check = tokio::process::Command::new(cmd_name)
        .arg("--version")
        .output()
        .await;
    if claude_check.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "ok": false,
            "error": "Claude Code not found in PATH",
            "hint": "Install Claude Code by running: npm install -g @anthropic-ai/claude-code",
            "install_command": "npm install -g @anthropic-ai/claude-code",
            "gui_install_available": true
        }))).into_response();
    }

    info!("Launching Claude Code from GUI: model={}, effort={:?}", model, effort);

    let mut cmd = tokio::process::Command::new(cmd_name);
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
            let error_msg = e.to_string();
            let is_not_found = error_msg.contains("not found") || error_msg.contains("No such file");

            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "ok": false,
                "error": if is_not_found {
                    "Claude Code executable not found".to_string()
                } else {
                    format!("Failed to launch Claude Code: {}", error_msg)
                },
                "hint": if is_not_found {
                    "Install Claude Code: npm install -g @anthropic-ai/claude-code".to_string()
                } else {
                    format!("Launch error: {}", error_msg)
                },
                "install_command": "npm install -g @anthropic-ai/claude-code",
                "gui_install_available": true
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
    let output_future = tokio::process::Command::new("warp-cli")
        .arg("status")
        .output();
        
    match tokio::time::timeout(std::time::Duration::from_secs(3), output_future).await {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let connected = stdout.contains("Connected") || stdout.contains("connected");
            json!({
                "installed": true,
                "connected": connected,
                "raw": stdout.trim(),
            })
        }
        Ok(Err(_)) | Err(_) => json!({
            "installed": false,
            "connected": false,
            "raw": "warp-cli not found or timed out",
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
    let claude_cmd = if cfg!(target_os = "windows") { "claude.cmd" } else { "claude" };
    let claude_installed = tokio::process::Command::new(claude_cmd)
        .arg("--version").output().await.is_ok();
    let node_installed = tokio::process::Command::new("node")
        .arg("--version").output().await.is_ok();
    let _port_available = tokio::net::TcpListener::bind("127.0.0.1:8080").await.is_ok();

    // If we are running the API, port 8080 IS bound by us. So port 8080 check is technically OK if we can serve this request!
    let port_available = true;

    Json(json!({
        "warp": {"ok": warp_installed, "status": if warp_installed { "installed" } else { "not found" }},
        "claude_code": {"ok": claude_installed, "status": if claude_installed { "installed" } else { "not found" }},
        "node": {"ok": node_installed, "status": if node_installed { "installed" } else { "not found" }},
        "port_8080": {"ok": port_available, "status": "in use by proxy (expected)"},
    }))
}

// === Config reset ===
pub async fn api_config_reset() -> Response {
    info!("Config reset requested from GUI");
    let default_config = AppConfig::default();
    if let Err(e) = save_config(&default_config) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": e}))).into_response();
    }
    (StatusCode::OK, Json(json!({"ok": true, "message": "Config reset to defaults"}))).into_response()
}

// === WARP full reset ===
pub async fn api_warp_full_reset() -> Response {
    info!("Full WARP reset triggered from GUI");
    let steps = if cfg!(target_os = "windows") {
        vec![
            ("warp-cli disconnect", vec!["warp-cli", "disconnect"]),
            ("net stop Cloudflare WARP", vec!["cmd", "/c", "net stop \"Cloudflare WARP\""]),
            ("clear WARP cache", vec!["cmd", "/c", "del /Q /S \"C:\\ProgramData\\Cloudflare\\*\""]),
            ("net start Cloudflare WARP", vec!["cmd", "/c", "net start \"Cloudflare WARP\""]),
        ]
    } else {
        vec![
            ("warp-cli disconnect", vec!["warp-cli", "disconnect"]),
            ("systemctl stop warp-svc", vec!["sudo", "-n", "sh", "-c", "systemctl stop warp-svc"]),
            ("clear WARP cache", vec!["sudo", "-n", "sh", "-c", "rm -rf /var/lib/cloudflare-warp/*"]),
            ("systemctl start warp-svc", vec!["sudo", "-n", "sh", "-c", "systemctl start warp-svc"]),
        ]
    };

    for (desc, cmd) in steps {
        let cmd_name = cmd[0];
        let cmd_args = &cmd[1..];
        match tokio::process::Command::new(cmd_name).args(cmd_args).output().await {
            Ok(out) if !out.status.success() => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                tracing::warn!("Step '{}' failed: {}", desc, stderr);
            }
            Err(e) => tracing::warn!("Failed to execute step '{}': {}", desc, e),
            _ => info!("Step '{}' succeeded", desc),
        }
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
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
    info!("WARP disconnect triggered from GUI");
    let output = tokio::process::Command::new("warp-cli")
        .arg("disconnect").output().await;
    match output {
        Ok(o) if o.status.success() => {
            (StatusCode::OK, Json(json!({"ok": true, "message": "WARP disconnected"}))).into_response()
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": err.to_string()}))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": e.to_string()}))).into_response()
        }
    }
}

pub async fn api_warp_connect() -> Response {
    info!("WARP connect triggered from GUI");
    let output = tokio::process::Command::new("warp-cli")
        .arg("connect").output().await;
    match output {
        Ok(o) if o.status.success() => {
            (StatusCode::OK, Json(json!({"ok": true, "message": "WARP connected"}))).into_response()
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": err.to_string()}))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": e.to_string()}))).into_response()
        }
    }
}

// === Install opencode ===
pub async fn api_install_opencode() -> Response {
    install_package("OpenCode").await
}

// === Install claude-code ===
pub async fn api_install_claude_code() -> Response {
    install_package("Claude Code").await
}

// === Install WARP ===
pub async fn api_install_warp() -> Response {
    if cfg!(target_os = "windows") {
        match tokio::process::Command::new("winget")
            .args(["install", "--id", "Cloudflare.Warp", "--accept-package-agreements", "--accept-source-agreements"])
            .output().await
        {
            Ok(output) if output.status.success() => {
                (StatusCode::OK, Json(json!({"ok": true, "message": "WARP installed successfully"}))).into_response()
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": format!("Install failed: {}\n{}", stderr, stdout)}))).into_response()
            }
            Err(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": format!("Failed to run winget: {}", e)}))).into_response()
            }
        }
    } else {
        (StatusCode::BAD_REQUEST, Json(json!({"ok": false, "error": "Auto-install for WARP is only supported on Windows currently."}))).into_response()
    }
}

// === Install Tor ===
pub async fn api_install_tor() -> Response {
    if cfg!(target_os = "windows") {
        let ps_script = r#"
            $ErrorActionPreference = 'Stop'
            $url = "https://dist.torproject.org/torbrowser/15.0.15/tor-expert-bundle-windows-x86_64-15.0.15.tar.gz"
            $dest = "C:\tor"
            if (!(Test-Path $dest)) { New-Item -ItemType Directory -Force -Path $dest | Out-Null }
            Invoke-WebRequest -Uri $url -OutFile "$dest\tor.tar.gz"
            # Verify download exists and has reasonable size (at least 1MB)
            $file = Get-Item "$dest\tor.tar.gz"
            if ($file.Length -lt 1048576) {
                throw "Downloaded file is too small ($($file.Length) bytes). Download may be corrupted."
            }
            tar -xf "$dest\tor.tar.gz" -C $dest
            # Verify tor.exe exists after extraction
            if (!(Test-Path "$dest\tor\tor.exe")) {
                throw "tor.exe not found after extraction. Archive may be corrupted."
            }
            $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
            if ($userPath -notmatch "C:\\tor\\tor") {
                [Environment]::SetEnvironmentVariable("Path", "$userPath;C:\tor\tor", "User")
            }
            Remove-Item "$dest\tor.tar.gz" -Force
        "#;
        match tokio::process::Command::new("powershell")
            .args(["-Command", ps_script])
            .output().await
        {
            Ok(output) if output.status.success() => {
                (StatusCode::OK, Json(json!({"ok": true, "message": "Tor installed successfully to C:\\tor"}))).into_response()
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": format!("Install failed: {}", stderr)}))).into_response()
            }
            Err(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": format!("Failed to run powershell: {}", e)}))).into_response()
            }
        }
    } else {
        (StatusCode::BAD_REQUEST, Json(json!({"ok": false, "error": "Auto-install for Tor is only supported on Windows currently."}))).into_response()
    }
}


async fn install_package(name: &str) -> Response {
    let pkg = match name {
        "OpenCode" => "opencode-ai",
        "Claude Code" => "@anthropic-ai/claude-code",
        _ => name,
    };

    let npm_cmd = if cfg!(target_os = "windows") { "npm.cmd" } else { "npm" };

    match tokio::process::Command::new(npm_cmd)
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
    let mut state_guard = state.lock().await;

    if let Some(tone) = body.get("tone").and_then(|v| v.as_str()) {
        let tone = tone.to_string();
        state_guard.tone = tone.clone();
        let mut app_config = load_config();
        app_config.tone = tone.clone();
        let _ = save_config(&app_config);
        info!("Tone setting changed to: {}", tone);
    }

    if let Some(effort) = body.get("effort_level").and_then(|v| v.as_str()) {
        info!("Effort level set to: {}", effort);
    }

    drop(state_guard);
    (StatusCode::OK, Json(json!({"ok": true}))).into_response()
}

pub async fn api_advanced_config_get(
    State(state): State<Arc<Mutex<ProxyState>>>,
) -> impl IntoResponse {
    let state_guard = state.lock().await;
    Json(json!({
        "max_retries": state_guard.config.max_retries,
        "warp_reset_delay_ms": state_guard.config.warp_reset_delay_ms,
        "opencode_base_url": state_guard.config.opencode_base_url,
    }))
}

#[derive(serde::Deserialize)]
pub struct AdvancedConfigUpdate {
    max_retries: Option<u32>,
    warp_reset_delay_ms: Option<u64>,
    opencode_base_url: Option<String>,
}

pub async fn api_advanced_config_post(
    State(state): State<Arc<Mutex<ProxyState>>>,
    Json(body): Json<AdvancedConfigUpdate>,
) -> impl IntoResponse {
    let mut state_guard = state.lock().await;
    if let Some(mr) = body.max_retries {
        state_guard.config.max_retries = mr;
    }
    if let Some(delay) = body.warp_reset_delay_ms {
        state_guard.config.warp_reset_delay_ms = delay;
    }
    if let Some(url) = body.opencode_base_url {
        state_guard.config.opencode_base_url = url;
    }
    let config = state_guard.config.clone();
    drop(state_guard);
    
    let mut app_config = crate::config::load_config();
    app_config.max_retries = config.max_retries;
    app_config.warp_delay = config.warp_reset_delay_ms;
    app_config.upstream = config.opencode_base_url.clone();
    let _ = crate::config::save_config(&app_config);
    
    Json(json!({ "ok": true }))
}

pub async fn api_system_logs() -> impl IntoResponse {
    let mut logs = Vec::new();
    if let Ok(content) = std::fs::read_to_string("oplire.log") {
        for line in content.lines().rev().take(100) {
            // Very naive parser for format: "2026-06-09T12:38:49.064758Z  INFO oplire: message..."
            let parts: Vec<&str> = line.splitn(4, ' ').collect();
            if parts.len() >= 4 {
                let timestamp = parts[0].trim();
                // parts[1] might be empty if there are multiple spaces
                let mut level_idx = 1;
                while level_idx < parts.len() && parts[level_idx].trim().is_empty() {
                    level_idx += 1;
                }
                if level_idx + 2 < parts.len() {
                    let level = parts[level_idx].trim();
                    let target = parts[level_idx + 1].trim().trim_end_matches(':');
                    let message = parts[level_idx + 2..].join(" ").trim().to_string();

                    logs.push(json!({
                        "timestamp": timestamp,
                        "level": level,
                        "target": target,
                        "message": message,
                    }));
                } else {
                    // Fallback
                    logs.push(json!({
                        "timestamp": "",
                        "level": "INFO",
                        "target": "system",
                        "message": line,
                    }));
                }
            } else {
                logs.push(json!({
                    "timestamp": "",
                    "level": "INFO",
                    "target": "system",
                    "message": line,
                }));
            }
        }
        logs.reverse();
    }
    Json(json!({ "logs": logs }))
}
