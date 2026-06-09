use oplire_reset::{ProxyConfig, AppConfig, FreeModel, load_config, save_config};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_proxy_config_default() {
    let config = ProxyConfig::default();
    assert_eq!(config.listen_addr, "127.0.0.1:8080");
    assert_eq!(config.opencode_base_url, "http://localhost:3000");
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.warp_reset_delay_ms, 5000);
    assert!(!config.use_tor);
    assert_eq!(config.tor_port, 9050);
    assert_eq!(config.rotation_mode, "on_block");
    assert_eq!(config.rotation_interval_secs, 300);
}

#[test]
fn test_free_models() {
    let models = ProxyConfig::free_models();
    assert_eq!(models.len(), 5);
    assert_eq!(models[0].id, "glm-4.7-free");
    assert_eq!(models[1].id, "minimax-m2.1-free");
    assert_eq!(models[2].id, "kimi-k2.5-free");
    assert_eq!(models[3].id, "qwen-2.5-72b-free");
    assert_eq!(models[4].id, "llama-3.3-70b-free");
}

#[test]
fn test_models_response() {
    let response = ProxyConfig::models_response();
    assert_eq!(response.get("object").and_then(|v| v.as_str()), Some("list"));
    let data = response.get("data").and_then(|v| v.as_array()).unwrap();
    assert_eq!(data.len(), 5);
}

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();
    assert_eq!(config.listen, "127.0.0.1:8080");
    assert_eq!(config.upstream, "http://localhost:3000");
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.warp_delay, 5000);
    assert_eq!(config.tone, "witty");
    assert!(!config.use_tor);
    assert_eq!(config.tor_port, 9050);
    assert_eq!(config.rotation_mode, "on_block");
    assert_eq!(config.rotation_interval_secs, 300);
}

#[test]
fn test_config_serialization() {
    let config = AppConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.listen, deserialized.listen);
    assert_eq!(config.upstream, deserialized.upstream);
    assert_eq!(config.max_retries, deserialized.max_retries);
    assert_eq!(config.warp_delay, deserialized.warp_delay);
    assert_eq!(config.tone, deserialized.tone);
    assert_eq!(config.use_tor, deserialized.use_tor);
}

#[test]
fn test_load_save_config() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.json");
    
    let original = AppConfig {
        listen: "127.0.0.1:9090".to_string(),
        upstream: "http://localhost:4000".to_string(),
        max_retries: 5,
        warp_delay: 10000,
        tone: "professional".to_string(),
        use_tor: true,
        tor_port: 9051,
        rotation_mode: "timed".to_string(),
        rotation_interval_secs: 600,
    };
    
    let json = serde_json::to_string_pretty(&original).unwrap();
    fs::write(&config_path, json).unwrap();
    
    let loaded = serde_json::from_str::<AppConfig>(
        &fs::read_to_string(&config_path).unwrap()
    ).unwrap();
    
    assert_eq!(original.listen, loaded.listen);
    assert_eq!(original.upstream, loaded.upstream);
    assert_eq!(original.max_retries, loaded.max_retries);
    assert_eq!(original.warp_delay, loaded.warp_delay);
    assert_eq!(original.tone, loaded.tone);
    assert_eq!(original.use_tor, loaded.use_tor);
    assert_eq!(original.tor_port, loaded.tor_port);
    assert_eq!(original.rotation_mode, loaded.rotation_mode);
    assert_eq!(original.rotation_interval_secs, loaded.rotation_interval_secs);
}

#[test]
fn test_free_model_serialization() {
    let model = FreeModel {
        id: "test-model".to_string(),
        display_name: "Test Model".to_string(),
    };
    let json = serde_json::to_string(&model).unwrap();
    let deserialized: FreeModel = serde_json::from_str(&json).unwrap();
    assert_eq!(model.id, deserialized.id);
    assert_eq!(model.display_name, deserialized.display_name);
}

#[test]
fn test_config_path() {
    let path = oplire_reset::config::config_path();
    assert!(path.ends_with("oplire/config.json"));
}