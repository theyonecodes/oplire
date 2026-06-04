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

---

## Install (Windows)

### Option 1: Download the .exe (Recommended)

**No build tools. No Rust. No pain.**

1. Download `oplire.exe` from [Releases](https://github.com/theyonecodes/oplire/releases/download/v2.4.0/oplire.exe)
3. Put it somewhere in your PATH (e.g. `C:\Users\You\.cargo\bin\`)
4. Open PowerShell and run:

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

## Usage

### WARP Reset
```powershell
oplire reset          # Full WARP tunnel reset
oplire quick-reset    # Fast IP rotation
oplire status         # Check connection status
```

### Proxy
```powershell
oplire proxy                          # Start on :8080
oplire connect claude-code            # One-click Claude Code setup
```

### Diagnostics
```powershell
oplire doctor         # Check everything is working
```

---

## Troubleshooting

**"oplire" not recognized** — Make sure the .exe location is in your PATH, or run it from its folder.

**WARP not connecting** — Install Cloudflare WARP from https://1.1.1.1/ first.

**Rate limit not resetting** — Run `oplire status`, then `oplire reset`.

---

## Credits

Created by [Berke Oruc](https://github.com/BerkeOruc). This fork adds pre-built Windows binaries and simplified installation.

Original repository: https://github.com/BerkeOruc/oplire

## License

MIT — see [LICENSE](LICENSE).
