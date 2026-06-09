use colored::*;
use std::process::Command;

/// Installation target for the installer
#[derive(Debug, Clone)]
pub enum InstallTarget {
    OpenCode,
    ClaudeCode,
    Node,
    GitHubCLI,
}

fn get_command_name(base: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.cmd", base)
    } else {
        base.to_string()
    }
}

/// Check if Node.js is installed and accessible
pub fn check_node_installed() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Claude Code is installed globally
pub fn check_claude_installed() -> bool {
    Command::new(get_command_name("claude"))
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if OpenCode is installed globally
pub fn check_opencode_installed() -> bool {
    let cmd = get_command_name("opencode");
    Command::new(&cmd)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || Command::new(&cmd)
        .arg("--help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if GitHub CLI is installed
pub fn check_github_installed() -> bool {
    Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the npm global prefix directory
pub fn get_npm_prefix() -> Option<String> {
    Command::new("npm")
        .args(["config", "get", "prefix"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

/// Get the npm global directory (prefix + node_modules)
pub fn get_npm_global_dir() -> Option<String> {
    get_npm_prefix().map(|p| format!("{}/node_modules", p))
}

/// Ensure the npm global directory exists
pub fn ensure_npm_global_dir() -> Result<(), String> {
    if let Some(dir) = get_npm_global_dir() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create npm global dir: {}", e))?;
    }
    Ok(())
}

/// Setup npm prefix to a user-writable directory
pub fn setup_npm_prefix() -> Result<(), String> {
    let prefix = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join("npm-global");

    std::fs::create_dir_all(&prefix)
        .map_err(|e| format!("Failed to create npm prefix: {}", e))?;

    let status = Command::new("npm")
        .args(["config", "set", "prefix", prefix.to_str().unwrap()])
        .status()
        .map_err(|e| format!("Failed to set npm prefix: {}", e))?;

    if !status.success() {
        return Err("Failed to set npm prefix".into());
    }

    // Also add to PATH for current session
    let path = std::env::var("PATH").unwrap_or_default();
    let bin_dir = prefix.to_str().unwrap();
    if !path.contains(bin_dir) {
        std::env::set_var("PATH", format!("{};{}", bin_dir, path));
    }

    Ok(())
}

/// Uninstall OpenCode globally
pub fn uninstall_opencode() -> Result<bool, String> {
    println!("{} Checking for existing OpenCode installation...", "→".dimmed());

    let output = Command::new("npm")
        .args(["ls", "-g", "opencode-ai"])
        .output()
        .map_err(|e| format!("Failed to check npm packages: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    if !stdout.contains("opencode-ai") {
        println!("  {} No existing OpenCode installation found", "✓".green());
        return Ok(false);
    }

    println!("  {} Found existing OpenCode, removing...", "→".dimmed());

    let status = Command::new("npm")
        .args(["uninstall", "-g", "opencode-ai"])
        .status()
        .map_err(|e| format!("Failed to uninstall OpenCode: {}", e))?;

    if status.success() {
        println!("  {} OpenCode uninstalled", "✓".green());
        Ok(true)
    } else {
        Err("Failed to uninstall OpenCode".into())
    }
}

/// Uninstall Claude Code globally
pub fn uninstall_claude() -> Result<bool, String> {
    println!("{} Checking for existing Claude Code installation...", "→".dimmed());

    let output = Command::new("npm")
        .args(["ls", "-g", "@anthropic-ai/claude-code"])
        .output()
        .map_err(|e| format!("Failed to check npm packages: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    if !stdout.contains("@anthropic-ai/claude-code") {
        println!("  {} No existing Claude Code installation found", "✓".green());
        return Ok(false);
    }

    println!("  {} Found existing Claude Code, removing...", "→".dimmed());

    let status = Command::new("npm")
        .args(["uninstall", "-g", "@anthropic-ai/claude-code"])
        .status()
        .map_err(|e| format!("Failed to uninstall Claude Code: {}", e))?;

    if status.success() {
        println!("  {} Claude Code uninstalled", "✓".green());
        Ok(true)
    } else {
        Err("Failed to uninstall Claude Code".into())
    }
}

/// Install OpenCode globally via npm
pub fn install_opencode() -> Result<(), String> {
    println!("\n{}", "=== OpenCode Installation ===".bright_cyan().bold());

    // Step 1: Uninstall existing if present
    uninstall_opencode()?;

    // Step 2: Check Node.js
    if !check_node_installed() {
        return Err("Node.js is required but not installed. Please install Node.js first.".into());
    }

    // Step 3: Setup npm prefix
    println!("{} Setting up npm prefix...", "→".dimmed());
    setup_npm_prefix()?;

    // Step 4: Ensure npm global dir exists
    ensure_npm_global_dir()?;

    // Step 5: Install via npm
    println!("{} Installing opencode-ai via npm...", "→".dimmed());
    let status = Command::new("npm")
        .args(["install", "-g", "opencode-ai"])
        .status()
        .map_err(|e| format!("Failed to run npm install: {}", e))?;

    if !status.success() {
        return Err("npm install failed. Check your network connection and try again.".into());
    }

    // Step 6: Verify installation
    if !check_opencode_installed() {
        return Err("Installation completed but opencode command not found in PATH. Try restarting your terminal.".into());
    }

    println!("\n{} OpenCode installed successfully!", "✓".green().bold());
    print_opencode_post_install();
    Ok(())
}

/// Install Claude Code globally via npm
pub fn install_claude_code() -> Result<(), String> {
    println!("\n{}", "=== Claude Code Installation ===".bright_cyan().bold());

    // Step 1: Uninstall existing if present
    uninstall_claude()?;

    // Step 2: Check Node.js
    if !check_node_installed() {
        return Err("Node.js is required but not installed. Please install Node.js first.".into());
    }

    // Step 3: Setup npm prefix
    println!("{} Setting up npm prefix...", "→".dimmed());
    setup_npm_prefix()?;

    // Step 4: Ensure npm global dir exists
    ensure_npm_global_dir()?;

    // Step 5: Install via npm
    println!("{} Installing @anthropic-ai/claude-code via npm...", "→".dimmed());
    let status = Command::new("npm")
        .args(["install", "-g", "@anthropic-ai/claude-code"])
        .status()
        .map_err(|e| format!("Failed to run npm install: {}", e))?;

    if !status.success() {
        return Err("npm install failed. Check your network connection and try again.".into());
    }

    // Step 6: Verify installation
    if !check_claude_installed() {
        return Err("Installation completed but claude command not found in PATH. Try restarting your terminal.".into());
    }

    println!("\n{} Claude Code installed successfully!", "✓".green().bold());
    print_claude_post_install();
    Ok(())
}

/// Print post-install instructions for OpenCode
fn print_opencode_post_install() {
    println!("\n{}", "Post-install steps:".bright_cyan());
    println!("  1. {}", "Run `opencode` to start the application".white());
    println!("  2. {}", "Set your ANTHROPIC_API_KEY environment variable".white());
    println!("  3. {}", "Or configure it in the app settings".white());
    println!();
}

/// Print post-install instructions for Claude Code
fn print_claude_post_install() {
    println!("\n{}", "Post-install steps:".bright_cyan());
    println!("  1. {}", "Run `claude` to start Claude Code".white());
    println!("  2. {}", "Set your ANTHROPIC_API_KEY environment variable".white());
    println!("  3. {}", "Or use `claude auth` to authenticate".white());
    println!();
}

/// Install Node.js (platform-specific)
pub fn install_node() -> Result<(), String> {
    println!("\n{}", "=== Node.js Installation ===".bright_cyan().bold());

    let platform = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    println!("  Platform: {} ({})", platform, arch);

    match platform {
        "windows" => install_node_windows(),
        "macos" => install_node_macos(),
        "linux" => install_node_linux(),
        _ => Err(format!("Unsupported platform: {}", platform)),
    }
}

fn install_node_windows() -> Result<(), String> {
    println!("{} Downloading Node.js installer for Windows...", "→".dimmed());

    // Use winget if available
    let status = Command::new("winget")
        .args(["install", "--id", "OpenJS.NodeJS.LTS", "--accept-package-agreements", "--accept-source-agreements"])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("{} Node.js installed via winget", "✓".green());
            Ok(())
        }
        _ => {
            // Fallback: direct download
            Err("Please install Node.js manually from https://nodejs.org/ or run: winget install OpenJS.NodeJS.LTS".into())
        }
    }
}

fn install_node_macos() -> Result<(), String> {
    println!("{} Installing Node.js via Homebrew...", "→".dimmed());

    let status = Command::new("brew")
        .args(["install", "node"])
        .status()
        .map_err(|e| format!("Failed to run brew: {}. Please install Homebrew first.", e))?;

    if status.success() {
        println!("{} Node.js installed via Homebrew", "✓".green());
        Ok(())
    } else {
        Err("Failed to install Node.js via Homebrew".into())
    }
}

fn install_node_linux() -> Result<(), String> {
    println!("{} Installing Node.js via package manager...", "→".dimmed());

    // Try apt first
    let status = Command::new("sudo")
        .args(["apt-get", "install", "-y", "nodejs", "npm"])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("{} Node.js installed via apt", "✓".green());
            Ok(())
        }
        _ => {
            // Try yum/dnf
            let status = Command::new("sudo")
                .args(["yum", "install", "-y", "nodejs", "npm"])
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("{} Node.js installed via yum", "✓".green());
                    Ok(())
                }
                _ => Err("Please install Node.js manually from https://nodejs.org/".into()),
            }
        }
    }
}

/// Install GitHub CLI (platform-specific)
pub fn install_github_cli() -> Result<(), String> {
    println!("\n{}", "=== GitHub CLI Installation ===".bright_cyan().bold());

    let platform = std::env::consts::OS;

    match platform {
        "windows" => install_gh_windows(),
        "macos" => install_gh_macos(),
        "linux" => install_gh_linux(),
        _ => Err(format!("Unsupported platform: {}", platform)),
    }
}

fn install_gh_windows() -> Result<(), String> {
    println!("{} Installing GitHub CLI via winget...", "→".dimmed());

    let status = Command::new("winget")
        .args(["install", "--id", "GitHub.cli", "--accept-package-agreements", "--accept-source-agreements"])
        .status()
        .map_err(|e| format!("Failed to run winget: {}", e))?;

    if status.success() {
        println!("{} GitHub CLI installed", "✓".green());
        Ok(())
    } else {
        Err("Failed to install GitHub CLI. Try: winget install GitHub.cli".into())
    }
}

fn install_gh_macos() -> Result<(), String> {
    println!("{} Installing GitHub CLI via Homebrew...", "→".dimmed());

    let status = Command::new("brew")
        .args(["install", "gh"])
        .status()
        .map_err(|e| format!("Failed to run brew: {}", e))?;

    if status.success() {
        println!("{} GitHub CLI installed", "✓".green());
        Ok(())
    } else {
        Err("Failed to install GitHub CLI via Homebrew".into())
    }
}

fn install_gh_linux() -> Result<(), String> {
    println!("{} Installing GitHub CLI...", "→".dimmed());

    // Try apt
    let status = Command::new("sudo")
        .args(["apt-get", "install", "-y", "gh"])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("{} GitHub CLI installed via apt", "✓".green());
            Ok(())
        }
        _ => Err("Please install GitHub CLI manually: https://cli.github.com/".into()),
    }
}

/// Uninstall Node.js (best-effort, platform-specific)
pub fn uninstall_node() -> Result<(), String> {
    println!("\n{}", "=== Node.js Uninstallation ===".bright_cyan().bold());
    Err("Automatic Node.js uninstallation is not supported. Please uninstall manually via your system's package manager.".into())
}

/// Uninstall GitHub CLI (best-effort)
pub fn uninstall_github_cli() -> Result<(), String> {
    println!("\n{}", "=== GitHub CLI Uninstallation ===".bright_cyan().bold());
    Err("Automatic GitHub CLI uninstallation is not supported. Please uninstall manually via your system's package manager.".into())
}

/// Run the install command for the given target
pub fn run_install(target: &InstallTarget) -> Result<(), String> {
    match target {
        InstallTarget::OpenCode => install_opencode(),
        InstallTarget::ClaudeCode => install_claude_code(),
        InstallTarget::Node => install_node(),
        InstallTarget::GitHubCLI => install_github_cli(),
    }
}

/// Run the uninstall command for the given target
pub fn run_uninstall(target: &InstallTarget) -> Result<(), String> {
    match target {
        InstallTarget::OpenCode => {
            uninstall_opencode()?;
            Ok(())
        }
        InstallTarget::ClaudeCode => {
            uninstall_claude()?;
            Ok(())
        }
        InstallTarget::Node => uninstall_node(),
        InstallTarget::GitHubCLI => uninstall_github_cli(),
    }
}

/// Run repair for the given target (uninstall + reinstall)
pub fn run_repair(target: &InstallTarget) -> Result<(), String> {
    println!("\n{}", format!("=== Repairing {:?} ===", target).bright_cyan().bold());
    run_uninstall(target)?;
    run_install(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_target_clone() {
        let target = InstallTarget::OpenCode;
        let cloned = target.clone();
        assert!(matches!(cloned, InstallTarget::OpenCode));
    }

    #[test]
    fn test_check_node_installed_returns_bool() {
        // Just verify it doesn't panic
        let _ = check_node_installed();
    }

    #[test]
    fn test_check_claude_installed_returns_bool() {
        let _ = check_claude_installed();
    }

    #[test]
    fn test_check_opencode_installed_returns_bool() {
        let _ = check_opencode_installed();
    }

    #[test]
    fn test_check_github_installed_returns_bool() {
        let _ = check_github_installed();
    }

    #[test]
    fn test_get_npm_prefix_returns_option() {
        let _ = get_npm_prefix();
    }

    #[test]
    fn test_uninstall_node_returns_error() {
        assert!(uninstall_node().is_err());
    }

    #[test]
    fn test_uninstall_github_cli_returns_error() {
        assert!(uninstall_github_cli().is_err());
    }
}
