pub mod config;
pub mod gui;
pub mod install;
pub mod proxy;
pub mod transform;
pub mod tor;
pub mod warp;
pub mod watch;

pub use config::{AppConfig, FreeModel, ProxyConfig, load_config, save_config};
pub use install::{InstallTarget, run_install, run_uninstall, run_repair, check_node_installed, check_claude_installed, check_opencode_installed, check_github_installed, install_opencode, install_claude_code};
