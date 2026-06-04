```
   ___      ___    _       ___     ___     ___
  / _ \    | _ \  | |     |_ _|   | _ \   | __|
 | (_) |   |  _/  | |__    | |    |   /   | _|
  \___/   _|_|_   |____|  |___|   |_|_\   |___|
_|"""""|_| """ |_|"""""|_|"""""|_|"""""|_|"""""|
"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'
     OpenCode Limit Reset + Proxy
```

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)

## What is oplire?

**oplire** is a dual-purpose tool:

1. **WARP Rate Limit Reset** — Rotates your IP via Cloudflare WARP to reset OpenCode rate limits
2. **Anthropic Proxy Bridge** — Reverse proxy that connects Claude Code to OpenCode Zen's free models with automatic rate limit recovery

### Features

- **One-liner CLI** — `oplire start` launches proxy + Claude Code in one command
- **Web Control Panel** — Glass morphism UI with model cards, effort picker, and WARP/Tor controls
- **Tor Integration** — Route traffic through Tor with automatic circuit rotation on rate limits
- **Tone System** — Professional, Witty, or Minimal personality for the control panel
- **Pre-built Binary** — No Rust required. Download the `.exe` and go.

---

## Install (Windows)

### Option 1: Download the .exe (Recommended)

**No build tools. No Rust. No pain.**

1. Download `oplire-v2.6.0.exe` from [Releases](https://github.com/theyonecodes/oplire/releases/download/v2.6.0/oplire-v2.6.0.exe)
2. Put it somewhere in your PATH (e.g. `C:\Users\You\.cargo\bin\`)
3. Open PowerShell and run:

```powershell
# Make sure Cloudflare WARP is installed (download from https://1.1.1.1/ if not)
oplire install warp

# Reset your IP
oplire reset
```

Done.

### Option 2: Install via Cargo (if you already have Rust)

```powershell
cargo install --git https://github.com/theyonecodes/oplire.git
```

---

## Linux / macOS

### AUR (Arch Linux)
```bash
yay -S oplire
```

### Homebrew (macOS)
```bash
brew install berkeoruc/oplire/oplire
```

### From Source
```bash
git clone https://github.com/theyonecodes/oplire.git
cd oplire
cargo build --release
sudo cp target/release/oplire /usr/bin/oplire
```

---

## Quick Start

### Free Models via Claude Code

One command to start the proxy and launch Claude Code with free models:

```powershell
# Start proxy + Claude Code with default model (GLM 4.7 Free)
oplire start

# Pick a specific model
oplire start --model minimax-m2.1-free

# Set effort level (low/medium/high/xhigh/max)
oplire start --effort low

# Both
oplire start --model kimi-k2.5-free --effort medium

# Use Tor for routing
oplire start --tor
```

### Web Control Panel

Start the proxy and open the browser-based control panel:

```powershell
# Start proxy + auto-open browser
oplire gui

# Or start proxy manually
oplire proxy
# Then open http://localhost:8080
```

The control panel includes:

- **Model Picker** — Visual cards for each free model with descriptions, specialties, and speed ratings
- **Effort Level** — One-click buttons: Low, Medium, High, XHigh, Max
- **Tone Selector** — Professional, Witty, or Minimal personality
- **WARP Controls** — Status, Reset IP, Full Reset, Stop
- **Tor Controls** — Start, Rotate Circuit, Stop, Exit IP display
- **Diagnostics** — System health checks for WARP, Claude Code, Node.js, and port availability
- **Quick Install** — Install OpenCode or Claude Code directly from the browser
- **Request Log** — Real-time request monitoring with status codes and latency

---

## Available Free Models

| Model | Provider | Specialty | Speed |
|-------|----------|-----------|-------|
| GLM 4.7 Free | Zhipu AI | Reasoning & Code | Fast |
| MiniMax M2.1 Free | MiniMax | Creative & Chat | Fast |
| Kimi K2.5 Free | Moonshot AI | Long Context & Analysis | Medium |
| Qwen 2.5 72B Free | Alibaba Cloud | General Purpose | Medium |
| Llama 3.3 70B Free | Meta | Balanced Performance | Fast |

All models are **free** with **128K context windows**.

---

## CLI Commands

### Proxy

```powershell
oplire proxy                    # Start proxy on :8080
oplire proxy --port 9090        # Custom port
oplire proxy --tor              # Route through Tor
```

### Start (One-liner)

```powershell
oplire start                                    # Default model + effort
oplire start --model llama-3.3-70b-free         # Specific model
oplire start --effort max                       # Max effort
oplire start --tor                              # Use Tor routing
```

### Web GUI

```powershell
oplire gui                      # Start proxy + open browser
oplire gui --port 9090          # Custom port
oplire gui --tor                # Route through Tor
```

### WARP Reset

```powershell
oplire reset                    # Full WARP tunnel reset
oplire quick-reset              # Fast IP rotation
oplire status                   # Check WARP status
```

### Tor

```powershell
oplire tor status               # Check Tor status
oplire tor start                # Start Tor daemon
oplire tor stop                 # Stop Tor daemon
oplire tor rotate               # Force circuit rotation
oplire tor ip                   # Show current exit IP
```

### Diagnostics

```powershell
oplire doctor                   # System health check
```

---

## Configuration

Configuration is stored at `~/.config/oplire/config.json`:

```json
{
  "listen": "127.0.0.1:8080",
  "upstream": "http://localhost:3000",
  "max_retries": 3,
  "warp_delay": 5000,
  "tone": "witty",
  "use_tor": false,
  "tor_port": 9050,
  "rotation_mode": "on_block",
  "rotation_interval_secs": 300
}
```

### Tone Settings

| Tone | Description |
|------|-------------|
| `professional` | Clean, corporate, no-nonsense |
| `witty` | Fun, sarcastic, personality-driven |
| `minimal` | Just the facts, bare minimum |

### Tor Rotation Modes

| Mode | Behavior |
|------|----------|
| `on_block` | Rotate IP only when rate-limited (default) |
| `timed` | Rotate every N seconds (configurable) |

---

## API Endpoints

The proxy exposes a REST API for the web control panel:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Web control panel |
| `/api/status` | GET | System status (WARP, Tor, OpenCode, proxy) |
| `/api/config` | GET/POST | Get/update configuration |
| `/api/config/reset` | POST | Reset config to defaults |
| `/api/logs` | GET | Request logs |
| `/api/launch` | POST | Launch Claude Code |
| `/api/doctor` | GET | System diagnostics |
| `/api/warp/reset` | POST | Reset WARP IP |
| `/api/warp/full-reset` | POST | Full WARP reset (disconnect + re-register) |
| `/api/warp/stop` | POST | Disconnect WARP |
| `/api/tor/status` | GET | Tor status |
| `/api/tor/start` | POST | Start Tor |
| `/api/tor/stop` | POST | Stop Tor |
| `/api/tor/rotate` | POST | Force circuit rotation |
| `/api/tor/config` | POST | Update Tor settings |
| `/api/settings` | GET/POST | Tone settings |
| `/api/install/opencode` | POST | Install OpenCode via npm |
| `/api/install/claude-code` | POST | Install Claude Code via npm |
| `/v1/models` | GET | Anthropic-compatible model list |
| `/v1/messages` | POST | Anthropic-compatible message endpoint |
| `/health` | GET | Health check |

---

## Troubleshooting

**"oplire" not recognized** — Make sure the .exe location is in your PATH, or run it from its folder.

**WARP not connecting** — Install Cloudflare WARP from https://1.1.1.1/ first.

**Rate limit not resetting** — Run `oplire status`, then `oplire reset`.

**Tor not working** — Make sure Tor is installed (`oplire tor status`). On Windows, install Tor Browser or use `tor` from the Tor expert bundle.

**Claude Code not found** — Run `oplire doctor` to check. Install via `npm install -g @anthropic-ai/claude-code` or use the Install button in the web GUI.

**Port 8080 in use** — Use `oplire proxy --port 9090` or stop the other process.

---

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐
│ Claude Code │────▶│  oplire      │────▶│  OpenCode Zen   │
│             │◀────│  Proxy       │◀────│  (free models)  │
└─────────────┘     └──────────────┘     └─────────────────┘
                          │
                    ┌─────┴─────┐
                    │  WARP /   │
                    │  Tor      │
                    └───────────┘
```

- **Claude Code** connects to oplire as if it were Anthropic's API
- **oplire** translates requests to OpenCode Zen's format
- **WARP/Tor** provides IP rotation when rate limits hit
- **Web GUI** controls everything from the browser

---

## Credits

Created by [Berke Oruc](https://github.com/BerkeOruc). This fork adds pre-built Windows binaries, web control panel, Tor integration, and simplified installation.

Original repository: https://github.com/BerkeOruc/oplire

## License

MIT — see [LICENSE](LICENSE).
