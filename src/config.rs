use std::fs;
use std::path::PathBuf;

/// Proxy configuration for the Anthropic ↔ OpenCode Zen bridge.
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// Address to listen on (default: 127.0.0.1:8080)
    pub listen_addr: String,
    /// OpenCode Zen API base URL (default: http://localhost:3000)
    pub opencode_base_url: String,
    /// API key for OpenCode Zen (if required)
    pub opencode_api_key: Option<String>,
    /// Maximum retry attempts after WARP reset
    pub max_retries: u32,
    /// Delay between WARP reset steps (milliseconds)
    pub warp_reset_delay_ms: u64,
    /// Use Tor for routing instead of direct connection
    pub use_tor: bool,
    /// Tor SOCKS5 port (default: 9050)
    pub tor_port: u16,
    /// Tor rotation mode: "on_block" or "timed"
    pub rotation_mode: String,
    /// Tor rotation interval in seconds (for timed mode)
    pub rotation_interval_secs: u64,
}

/// A free model available on OpenCode Zen.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FreeModel {
    /// Model identifier used by OpenCode Zen
    pub id: String,
    /// Display name shown in Claude Code
    pub display_name: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:8080".to_string(),
            opencode_base_url: "http://localhost:3000".to_string(),
            opencode_api_key: None,
            max_retries: 3,
            warp_reset_delay_ms: 5000,
            use_tor: false,
            tor_port: 9050,
            rotation_mode: "on_block".to_string(),
            rotation_interval_secs: 300,
        }
    }
}

impl ProxyConfig {
    /// Returns the list of free models mapped to Anthropic schema.
    pub fn free_models() -> Vec<FreeModel> {
        vec![
            FreeModel {
                id: "glm-4.7-free".to_string(),
                display_name: "GLM 4.7 Free".to_string(),
            },
            FreeModel {
                id: "minimax-m2.1-free".to_string(),
                display_name: "MiniMax M2.1 Free".to_string(),
            },
            FreeModel {
                id: "kimi-k2.5-free".to_string(),
                display_name: "Kimi K2.5 Free".to_string(),
            },
            FreeModel {
                id: "qwen-2.5-72b-free".to_string(),
                display_name: "Qwen 2.5 72B Free".to_string(),
            },
            FreeModel {
                id: "llama-3.3-70b-free".to_string(),
                display_name: "Llama 3.3 70B Free".to_string(),
            },
        ]
    }

    /// Build Anthropic-compatible models response JSON.
    pub fn models_response() -> serde_json::Value {
        let models: Vec<serde_json::Value> = Self::free_models()
            .iter()
            .map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "object": "model",
                    "created": 0,
                    "owned_by": "opencode-zen"
                })
            })
            .collect();

        serde_json::json!({
            "object": "list",
            "data": models
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub listen: String,
    pub upstream: String,
    pub max_retries: u32,
    pub warp_delay: u64,
    #[serde(default = "default_tone")]
    pub tone: String,
    #[serde(default)]
    pub use_tor: bool,
    #[serde(default = "default_tor_port")]
    pub tor_port: u16,
    #[serde(default = "default_rotation_mode")]
    pub rotation_mode: String,
    #[serde(default = "default_rotation_interval")]
    pub rotation_interval_secs: u64,
}

fn default_tone() -> String { "witty".to_string() }
fn default_tor_port() -> u16 { 9050 }
fn default_rotation_mode() -> String { "on_block".to_string() }
fn default_rotation_interval() -> u64 { 300 }

pub fn config_path() -> PathBuf {
    let mut path = dirs_config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("oplire");
    path.push("config.json");
    path
}

fn dirs_config_dir() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        if let Ok(home) = std::env::var("HOME") {
            let mut p = PathBuf::from(home);
            p.push("Library");
            p.push("Application Support");
            return Some(p);
        }
    }
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg));
    }
    if let Ok(home) = std::env::var("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".config");
        return Some(p);
    }
    None
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    AppConfig::default()
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            listen: "127.0.0.1:8080".to_string(),
            upstream: "http://localhost:3000".to_string(),
            max_retries: 3,
            warp_delay: 5000,
            tone: "witty".to_string(),
            use_tor: false,
            tor_port: 9050,
            rotation_mode: "on_block".to_string(),
            rotation_interval_secs: 300,
        }
    }
}
