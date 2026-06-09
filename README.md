# oplire

OpenCode Limit Reset + Anthropic Proxy Bridge

## What it does

1. **Cross-Platform WARP Reset** — Rotates your IP via Cloudflare WARP to reset rate limits (Supports Windows, Linux, and macOS).
2. **Proxy Bridge** — Connects Claude Code to OpenCode Zen's free models.
3. **Web GUI** — Control panel with real-time System and Request logs, advanced settings, and easy access to models.

## Install

### Windows
1. Run `build.cmd` to build from source
2. Or download `oplire Setup <version>.exe` from Releases

### Linux / macOS
```bash
cargo build --release
sudo cp target/release/oplire /usr/local/bin/
```

## Usage

```bash
# Check status
oplire status

# Reset WARP tunnel (Cross-platform support)
oplire reset

# Start proxy
oplire proxy

# Install dependencies
oplire install opencode
oplire install claude

# Launch web GUI
oplire gui
```

## Build

```bash
# Windows - double-click build.cmd
# Or from terminal:
.\build.ps1

# Linux/macOS
cargo build --release
```

## License

MIT
