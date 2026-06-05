# Oplire - Gap Analysis & Technical Debt

This document outlines the known gaps, incomplete features, and technical debt in the current implementation of Oplire.

## 1. Installation & Environment Gaps
- **NPM Cross-Platform Issues**: The Quick Install buttons for OpenCode and Claude Code rely on `npm install -g`. On Windows, the process name is `npm.cmd`, causing failures (`program not found`) if `npm` is hardcoded.
- **Tor Missing Install Flow**: While the application can start and stop Tor, it relies on the user to manually install the Tor daemon and add it to their system PATH. There is no automated downloading/bootstrapping of the Tor binary.
- **System Service Installation**: The application cannot currently install itself as a native Windows Service or `systemd` daemon. It relies on the GUI/CLI remaining open in the background.

## 2. Frontend & UI Gaps
- **System Logs Missing**: While the proxy captures HTTP request logs, the internal application logs (such as connection failures, WARP rotation events, and system errors) are only visible in the terminal stdout, not in the web UI.
- **Duplicate Elements**: The "Automation" card was duplicated during a recent HTML injection, causing visual clutter.
- **Progress States**: The UI lacks detailed progress bars for long-running operations (like `npm install` or full WARP resets). It relies on simple loading spinners and alerts.

## 3. Terminal Integration Gaps
- **Resizing**: The embedded `xterm.js` terminal does not dynamically notify the Rust PTY of window resizing, which can cause text wrapping issues if the browser window is shrunk.
- **Persistence**: If the user refreshes the page, the embedded terminal session is lost and a new PowerShell instance is spawned.
