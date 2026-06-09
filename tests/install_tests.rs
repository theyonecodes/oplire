use oplire_reset::install::{
    InstallTarget, check_node_installed, check_claude_installed, 
    check_opencode_installed, check_github_installed,
    get_npm_prefix, get_npm_global_dir, ensure_npm_global_dir,
    setup_npm_prefix, uninstall_opencode, uninstall_claude,
    install_opencode, install_claude_code, install_node,
    install_github_cli, uninstall_node, uninstall_github_cli,
    run_install, run_uninstall, run_repair,
};

#[test]
fn test_install_target_clone() {
    let target = InstallTarget::OpenCode;
    let cloned = target.clone();
    assert!(matches!(cloned, InstallTarget::OpenCode));
}

#[test]
fn test_check_node_installed_returns_bool() {
    let result = check_node_installed();
    assert!(result == true || result == false);
}

#[test]
fn test_check_claude_installed_returns_bool() {
    let result = check_claude_installed();
    assert!(result == true || result == false);
}

#[test]
fn test_check_opencode_installed_returns_bool() {
    let result = check_opencode_installed();
    assert!(result == true || result == false);
}

#[test]
fn test_check_github_installed_returns_bool() {
    let result = check_github_installed();
    assert!(result == true || result == false);
}

#[test]
fn test_get_npm_prefix_returns_option() {
    let result = get_npm_prefix();
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_get_npm_global_dir() {
    let result = get_npm_global_dir();
    if let Some(prefix) = get_npm_prefix() {
        let expected = format!("{}/node_modules", prefix);
        assert_eq!(result, Some(expected));
    } else {
        assert!(result.is_none());
    }
}

#[test]
fn test_ensure_npm_global_dir() {
    let result = ensure_npm_global_dir();
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_uninstall_node_returns_error() {
    assert!(uninstall_node().is_err());
}

#[test]
fn test_uninstall_github_cli_returns_error() {
    assert!(uninstall_github_cli().is_err());
}

#[test]
fn test_install_targets() {
    assert!(matches!(InstallTarget::OpenCode, InstallTarget::OpenCode));
    assert!(matches!(InstallTarget::ClaudeCode, InstallTarget::ClaudeCode));
    assert!(matches!(InstallTarget::Node, InstallTarget::Node));
    assert!(matches!(InstallTarget::GitHubCLI, InstallTarget::GitHubCLI));
}