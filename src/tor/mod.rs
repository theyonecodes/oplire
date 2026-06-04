use std::time::Duration;
use tracing::info;

pub const DEFAULT_SOCKS_PORT: u16 = 9050;
pub const DEFAULT_CONTROL_PORT: u16 = 9051;

#[derive(Debug, Clone)]
pub struct TorStatus {
    pub installed: bool,
    pub running: bool,
    pub exit_ip: Option<String>,
    pub socks_port: u16,
}

pub async fn check_tor_installed() -> bool {
    #[cfg(target_os = "windows")]
    {
        tokio::process::Command::new("where")
            .arg("tor")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(target_os = "linux")]
    {
        tokio::process::Command::new("which")
            .arg("tor")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(target_os = "macos")]
    {
        tokio::process::Command::new("which")
            .arg("tor")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

pub async fn check_tor_running() -> bool {
    tokio::net::TcpStream::connect(format!("127.0.0.1:{}", DEFAULT_SOCKS_PORT))
        .await
        .is_ok()
}

pub async fn get_tor_exit_ip() -> Option<String> {
    let proxy = reqwest::Proxy::all(&format!("socks5h://127.0.0.1:{}", DEFAULT_SOCKS_PORT)).ok()?;
    let client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;

    let resp = client
        .get("https://check.torproject.org/api/ip")
        .send()
        .await
        .ok()?;

    let body: serde_json::Value = resp.json().await.ok()?;
    body.get("IP").and_then(|v| v.as_str()).map(|s| s.to_string())
}

pub async fn get_tor_status() -> TorStatus {
    let installed = check_tor_installed().await;
    let running = check_tor_running().await;
    let exit_ip = if running {
        get_tor_exit_ip().await
    } else {
        None
    };

    TorStatus {
        installed,
        running,
        exit_ip,
        socks_port: DEFAULT_SOCKS_PORT,
    }
}

pub async fn rotate_tor_circuit() -> Result<String, String> {
    info!("Attempting Tor circuit rotation via NEWNYM");

    let control_addr = format!("127.0.0.1:{}", DEFAULT_CONTROL_PORT);

    let mut stream = tokio::net::TcpStream::connect(&control_addr)
        .await
        .map_err(|e| format!("Cannot connect to Tor control port {}: {}", control_addr, e))?;

    use tokio::io::AsyncWriteExt;

    let greeting = read_line(&mut stream).await?;
    if !greeting.starts_with("250") {
        return Err(format!("Unexpected Tor greeting: {}", greeting));
    }

    stream
        .write_all(b"AUTHENTICATE\r\n")
        .await
        .map_err(|e| format!("Failed to send AUTHENTICATE: {}", e))?;

    let auth_response = read_line(&mut stream).await?;
    if !auth_response.starts_with("250") {
        return Err(format!("Tor authentication failed: {}", auth_response));
    }

    info!("Tor authenticated, sending NEWNYM signal");

    stream
        .write_all(b"SIGNAL NEWNYM\r\n")
        .await
        .map_err(|e| format!("Failed to send NEWNYM: {}", e))?;

    let nym_response = read_line(&mut stream).await?;
    if nym_response.starts_with("250") {
        info!("NEWNYM signal sent successfully");
        Ok("New circuit requested. Wait 10s for full rotation.".to_string())
    } else {
        Err(format!("NEWNYM failed: {}", nym_response))
    }
}

async fn read_line(stream: &mut tokio::net::TcpStream) -> Result<String, String> {
    use tokio::io::AsyncReadExt;
    let mut buf = [0u8; 1024];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| format!("Read error: {}", e))?;
    Ok(String::from_utf8_lossy(&buf[..n]).to_string())
}

pub async fn start_tor() -> Result<String, String> {
    if check_tor_running().await {
        return Ok("Tor is already running".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        tokio::process::Command::new("tor")
            .spawn()
            .map_err(|e| format!("Failed to start Tor: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        tokio::process::Command::new("sudo")
            .args(["systemctl", "start", "tor"])
            .output()
            .await
            .map_err(|e| format!("Failed to start Tor: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        tokio::process::Command::new("brew")
            .args(["services", "start", "tor"])
            .output()
            .await
            .map_err(|e| format!("Failed to start Tor: {}", e))?;
    }

    tokio::time::sleep(Duration::from_secs(3)).await;

    if check_tor_running().await {
        Ok("Tor started successfully".to_string())
    } else {
        Err("Tor may have failed to start. Check tor is installed.".to_string())
    }
}

pub async fn stop_tor() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        tokio::process::Command::new("taskkill")
            .args(["/IM", "tor.exe", "/F"])
            .output()
            .await
            .map_err(|e| format!("Failed to stop Tor: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        tokio::process::Command::new("sudo")
            .args(["systemctl", "stop", "tor"])
            .output()
            .await
            .map_err(|e| format!("Failed to stop Tor: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        tokio::process::Command::new("brew")
            .args(["services", "stop", "tor"])
            .output()
            .await
            .map_err(|e| format!("Failed to stop Tor: {}", e))?;
    }

    tokio::time::sleep(Duration::from_secs(1)).await;

    if !check_tor_running().await {
        Ok("Tor stopped".to_string())
    } else {
        Err("Tor may still be running".to_string())
    }
}

pub fn create_tor_proxy(port: u16) -> Result<reqwest::Proxy, String> {
    reqwest::Proxy::all(&format!("socks5h://127.0.0.1:{}", port))
        .map_err(|e| format!("Failed to create Tor proxy: {}", e))
}
