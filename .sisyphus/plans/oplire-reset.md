# oplire-reset - OpenCode Limit Reset

## TL;DR

> **Quick Summary**: Cloudflare WARP tünelini sıfırlayarak OpenCode rate limitinden kurtulmak için Rust CLI aracı. Windows (.exe) ve Linux (binary) için otomatik build + auto-install.

> **Deliverables**:
> - `oplire-reset.exe` (Windows x64)
> - `oplire-reset` (Linux x64 binary)
> - Shell script wrapper (`install.sh`, `run.sh`)
> - GitHub Actions ile auto-build ve release

> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 2 waves
> **Critical Path**: Project setup → Rust CLI → WARP logic → GitHub Actions → Release

---

## Context

### Original Request
Kullanıcı OpenCode rate limitinden kurtulmak için Cloudflare WARP tüneli sıfırlayan bir tool istiyor. Proje adı: `oplire-reset`. Yapımcı: Berke Oruc.

### Interview Summary
**Key Discussions**:
- Proje adı: `oplire-reset` (OpenCode Limit Reset)
- Amaç: WARP tüneli resetleyerek yeni IP al
- Platform: Windows (.exe) + Linux (binary)
- Auto-install: Hem Windows hem Linux için
- Build: GitHub Actions ile her push/tag'de auto-build
- Kullanıcı adı: "berke" (hardcoded veya argüman)

**Research Findings**:
- WARP CLI komutları: `warp-cli disconnect`, `warp-cli registration delete`, `warp-cli registration new`, `warp-cli connect`
- Service: `sudo systemctl restart warp-svc`
- Data reset: `sudo rm -rf /var/lib/cloudflare-warp/*`
- GitHub Actions: Matrix-based native runners, targets: `x86_64-pc-windows-gnu`, `x86_64-unknown-linux-gnu`

### Metis Review
**Identified Gaps** (addressed):
- `warp-cli delete` yerine `warp-cli registration delete` kullanılmalı
- `--dry-run` flag eklendi (test için)
- Permission handling (sudo/elevation) netleştirildi
- Error handling: WARP not installed, no internet, service failure

**Guardrails Applied**:
- Scheduling/daemon functionality YOK (scope creep önleme)
- Config file YOK (CLI args only)
- Auto-reset loop YOK (manual invocation only)

---

## Work Objectives

### Core Objective
OpenCode rate limitinden kurtulmak için Cloudflare WARP tüneli sıfırlayan, cross-platform Rust CLI aracı.

### Concrete Deliverables
- `src/main.rs` - Ana Rust CLI
- `.github/workflows/build.yml` - Auto-build workflow
- `README.md` - Modern dokümantasyon
- `scripts/install-linux.sh` - Linux auto-install script
- `scripts/run-linux.sh` - Linux wrapper script

### Definition of Done
- [ ] `oplire-reset --help` çalışıyor
- [ ] `oplire-reset status` WARP durumunu gösteriyor
- [ ] `oplire-reset reset` yeni IP alıyor
- [ ] Windows .exe build oluyor
- [ ] Linux binary build oluyor
- [ ] GitHub Release'e artifact'lar yükleniyor

### Must Have
- Cross-platform CLI (Windows + Linux)
- Auto-install (WARP kurulumu)
- GitHub Actions auto-build
- Modern README

### Must NOT Have
- GUI arayüz
- Config file
- Scheduling/daemon mode
- Proxy configuration
- Multi-account support

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: NO (new project)
- **Automated tests**: NO (simple CLI tool)
- **Agent-Executed QA**: YES (manual verification scenarios)

### QA Policy
Her task için agent-executed QA scenarios. Evidence `.sisyphus/evidence/` altında.

---

## Execution Strategy

### Parallel Waves

```
Wave 1 (Foundation):
├── Task 1: Project scaffolding (Cargo.toml, src/)
├── Task 2: CLI structure (clap, commands)
├── Task 3: README.md (modern documentation)
└── Task 4: GitHub Actions workflow

Wave 2 (Core + Release):
├── Task 5: WARP reset logic (main.rs)
├── Task 6: Auto-install scripts (Linux)
├── Task 7: Auto-install (Windows PowerShell)
├── Task 8: Final build test + Release config
└── Task 9: Commit + push to GitHub
```

---

## TODOs

- [x] 1. Project Scaffolding

  **What to do**:
  - Initialize Rust project: `cargo new oplire-reset`
  - Add dependencies: `clap`, `colored`, `serde`, `serde_json`
  - Create `Cargo.toml` with proper metadata
  - Add `.gitignore`, `LICENSE`, `CHANGELOG.md`

  **Must NOT do**:
  - GUI dependencies (termui, cursive)
  - Database or config file crates

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple project setup, standard Rust patterns
  - **Skills**: []
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3, 4)
  - **Blocks**: Tasks 5-9
  - **Blocked By**: None

  **References**:
  - `https://doc.rust-lang.org/cargo/reference/manifest.html` - Cargo.toml format
  - `https://docs.rs/clap/latest/clap/` - CLI argument parsing

  **Acceptance Criteria**:
  - [ ] `cargo build` başarılı
  - [ ] Binary oluştu: `target/debug/oplire-reset`

  **QA Scenarios**:
  ```
  Scenario: Build project
    Tool: Bash
    Preconditions: Rust installed
    Steps:
      1. cargo build --release
      2. ls -la target/release/oplire-reset
    Expected Result: Binary exists, executable
    Evidence: .sisyphus/evidence/task-1-build.png
  ```

  **Commit**: YES (Task 1-4 together)
  - Message: `feat: project setup with basic structure`
  - Files: `Cargo.toml`, `src/main.rs`, `.gitignore`

---

- [x] 2. CLI Structure with Clap

  **What to do**:
  - Add `clap` dependency for CLI parsing
  - Define commands: `status`, `reset`, `install`, `help`
  - Add `--dry-run` flag for testing
  - Add `--verbose` flag for debug output
  - Create help messages with examples

  **Must NOT do**:
  - Complex subcommands (keep simple)
  - Config file loading

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Standard CLI pattern, clap is straightforward

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1

  **References**:
  - `https://docs.rs/clap/latest/clap/_derive/index.html` - Derive macros
  - Example: `https://github.com/clap-rs/clap/tree/master/examples` - CLI examples

  **Acceptance Criteria**:
  - [ ] `oplire-reset --help` çalışıyor
  - [ ] `oplire-reset status` çalışıyor
  - [ ] `oplire-reset reset --help` çalışıyor

  **QA Scenarios**:
  ```
  Scenario: CLI help displays
    Tool: Bash
    Preconditions: Binary built
    Steps:
      1. ./target/release/oplire-reset --help
    Expected Result: Help text with all commands listed
    Evidence: .sisyphus/evidence/task-2-help.png

  Scenario: Dry-run mode works
    Tool: Bash
    Steps:
      1. ./target/release/oplire-reset reset --dry-run
    Expected Result: Shows what would do, doesn't execute
    Evidence: .sisyphus/evidence/task-2-dryrun.png
  ```

  **Commit**: YES (with Task 1, 3, 4)

---

- [x] 3. Modern README Documentation

  **What to do**:
  - Create modern, visually appealing README.md
  - Include: Badges (build, release, license), Quick Start, Features
  - Add platform-specific instructions (Linux + Windows)
  - Include usage examples with expected output
  - Add "Made by Berke Oruc" section
  - Add ASCII art or nice header

  **Must NOT do**:
  - Too much technical detail (keep accessible)
  - Outdated information

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Documentation writing, markdown formatting

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1

  **References**:
  - `https://github.com/rust-lang/crates.io` - Example of modern crate README
  - `https://www.makeareadme.com/` - README best practices

  **Acceptance Criteria**:
  - [ ] README.md exists with proper sections
  - [ ] Build badge points to correct repo
  - [ ] Quick Start section for both platforms

  **QA Scenarios**:
  ```
  Scenario: README structure
    Tool: Bash
    Steps:
      1. cat README.md | head -50
    Expected Result: Contains badges, install, usage sections
    Evidence: .sisyphus/evidence/task-3-readme.png
  ```

  **Commit**: YES (with Task 1, 2, 4)

---

- [x] 4. GitHub Actions Workflow

  **What to do**:
  - Create `.github/workflows/build.yml`
  - Matrix build: ubuntu (linux), windows-latest, macos-latest
  - Build targets: 
    - Linux: `x86_64-unknown-linux-gnu`
    - Windows: `x86_64-pc-windows-gnu`
    - macOS: `x86_64-apple-darwin`, `aarch64-apple-darwin`
  - Auto-release on tag (v* pattern)
  - Upload artifacts to GitHub Releases

  **Must NOT do**:
  - Complex test matrix (no tests yet)
  - Unnecessary caching (keep simple)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
    - Reason: Standard GitHub Actions, well-documented patterns

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1

  **References**:
  - `https://github.com/softprops/action-gh-release` - Release action
  - `https://github.com/actions/upload-artifact` - Artifact upload

  **Acceptance Criteria**:
  - [ ] workflow file exists at `.github/workflows/build.yml`
  - [ ] Triggers on push to main and tags
  - [ ] Builds all 3 platforms

  **QA Scenarios**:
  ```
  Scenario: Workflow syntax valid
    Tool: Bash
    Steps:
      1. cat .github/workflows/build.yml | python3 -c "import yaml, sys; yaml.safe_load(sys.stdin)"
    Expected Result: Valid YAML, no syntax errors
    Evidence: .sisyphus/evidence/task-4-workflow.png
  ```

  **Commit**: YES (with Task 1, 2, 3)

---

- [x] 5. WARP Reset Logic (Core)

  **What to do**:
  - Implement `reset` command
  - Linux WARP commands:
    ```bash
    warp-cli disconnect
    sudo systemctl stop warp-svc
    sudo rm -rf /var/lib/cloudflare-warp/*
    sudo systemctl start warp-svc
    warp-cli registration new
    warp-cli connect
    ```
  - Windows WARP commands (via PowerShell):
    ```powershell
    warp-cli disconnect
    # Windows data reset path
    # registration new
    warp-cli connect
    ```
  - Add colored output (success/error/info)
  - Add verbose logging

  **Must NOT do**:
  - Hardcoded credentials
  - Log sensitive data

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Platform-specific commands, error handling needed

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2

  **References**:
  - Metis findings: correct commands are `warp-cli registration delete` not `delete`
  - Linux WARP CLI docs

  **Acceptance Criteria**:
  - [ ] `oplire-reset reset` executes full sequence
  - [ ] Error handling for WARP not installed
  - [ ] Colored output for each step

  **QA Scenarios**:
  ```
  Scenario: Reset shows progress
    Tool: Bash (simulated)
    Steps:
      1. cargo run -- reset --dry-run
    Expected Result: Shows each step that would run
    Evidence: .sisyphus/evidence/task-5-reset-dryrun.png

  Scenario: Error when WARP not installed
    Tool: Bash
    Steps:
      1. cargo run -- reset
    Expected Result: Clear error message if warp-cli not found
    Evidence: .sisyphus/evidence/task-5-error.png
  ```

  **Commit**: YES (Wave 2 commit)
  - Message: `feat: implement WARP reset logic`

---

- [x] 6. Linux Auto-Install Script

  **What to do**:
  - Create `scripts/install-linux.sh`
  - Detect OS (Ubuntu, Debian, Fedora, Arch)
  - Add Cloudflare WARP repository
  - Install `cloudflare-warp` package
  - Verify installation with `warp-cli --version`
  - Add sudo elevation check
  - Support `--force` flag

  **Must NOT do**:
  - Modify unrelated system files
  - Install from untrusted sources

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: System-level installation, multiple distros

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2

  **References**:
  - `https://pkg.cloudflareclient.com/` - Official repo setup
  - WARP client installation guides

  **Acceptance Criteria**:
  - [ ] Script works on Ubuntu 22.04
  - [ ] Detects if WARP already installed
  - [ ] Shows clear success/failure

  **QA Scenarios**:
  ```
  Scenario: Install script exists
    Tool: Bash
    Steps:
      1. ls -la scripts/install-linux.sh
    Expected Result: File exists, executable bit set
    Evidence: .sisyphus/evidence/task-6-script.png

  Scenario: Script shows help
    Tool: Bash
    Steps:
      1. bash scripts/install-linux.sh --help
    Expected Result: Help text with install options
    Evidence: .sisyphus/evidence/task-6-help.png
  ```

  **Commit**: YES (Wave 2 commit)

---

- [x] 7. Windows Auto-Install (PowerShell)

  **What to do**:
  - Create `scripts/install-windows.ps1`
  - Download and install Cloudflare WARP client
  - Support silent install with `/quiet` flag
  - Verify installation
  - Add admin elevation check
  - Support `--force` flag

  **Must NOT do**:
  - Download from unofficial sources
  - Modify Windows registry unnecessarily

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Windows-specific, PowerShell scripting

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2

  **References**:
  - Cloudflare WARP Windows installer location
  - PowerShell silent install patterns

  **Acceptance Criteria**:
  - [ ] Script exists and is valid PowerShell
  - [ ] Shows clear install steps

  **QA Scenarios**:
  ```
  Scenario: PowerShell script syntax valid
    Tool: Bash
    Steps:
      1. cat scripts/install-windows.ps1 | head -20
    Expected Result: Valid PowerShell syntax
    Evidence: .sisyphus/evidence/task-7-ps1.png
  ```

  **Commit**: YES (Wave 2 commit)

---

- [x] 8. Build Test + Release Config

  **What to do**:
  - Final cross-platform build test
  - Verify all binaries run without crash
  - Configure release artifacts in workflow
  - Add version detection (`CARGO_PKG_VERSION`)
  - Test `--version` flag

  **Must NOT do**:
  - Skip platform-specific testing

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Verification across platforms

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2

  **Acceptance Criteria**:
  - [ ] Linux binary runs and shows version
  - [ ] Windows .exe (cross-compile) artifact exists

  **QA Scenarios**:
  ```
  Scenario: Version flag works
    Tool: Bash
    Steps:
      1. ./target/release/oplire-reset --version
    Expected Result: Shows version like "oplire-reset 0.1.0"
    Evidence: .sisyphus/evidence/task-8-version.png
  ```

  **Commit**: YES (with Task 9)

---

- [ ] 9. Commit + Push to GitHub

  **What to do**:
  - Initialize git repo (if not already)
  - Create initial commit
  - Create GitHub repo (via gh CLI or manual)
  - Push to remote
  - Verify GitHub Actions triggers

  **Must NOT do**:
  - Push secrets or credentials

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Git operations, straightforward

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2

  **Acceptance Criteria**:
  - [ ] Repo exists on GitHub
  - [ ] First commit visible
  - [ ] Actions workflow visible

  **QA Scenarios**:
  ```
  Scenario: GitHub repo accessible
    Tool: Bash
    Steps:
      1. gh repo view --public 2>/dev/null || echo "Need to create"
    Expected Result: Repo details or create prompt
    Evidence: .sisyphus/evidence/task-9-repo.png
  ```

  **Commit**: YES (final commit)
  - Message: `feat: initial release - full feature set`

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. Verify all "Must Have" items implemented.
  Output: `Must Have [9/9] | Must NOT Have [0 found] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy`, `cargo fmt --check`, check for warnings.
  Output: `Clippy [N warnings] | Format [OK/FIX] | VERDICT`

- [ ] F3. **GitHub Actions Test** — `unspecified-high`
  Trigger workflow manually, verify all matrix builds start.
  Output: `Workflow [triggered/pending] | Matrix [all 3 platforms] | VERDICT`

---

## Commit Strategy

- **1**: `feat: project setup` - Cargo.toml, src/, .gitignore, LICENSE
- **2**: `feat: cli commands` - clap implementation, help text
- **3**: `feat: documentation` - README.md
- **4**: `feat: ci workflow` - GitHub Actions build.yml
- **5**: `feat: warp reset` - Core reset logic
- **6**: `feat: install scripts` - Linux + Windows install
- **7**: `feat: release ready` - Final build config

---

## Success Criteria

### Verification Commands
```bash
# Build
cargo build --release

# Test CLI
./target/release/oplire-reset --help
./target/release/oplire-reset --version
./target/release/oplire-reset status

# Dry run
./target/release/oplire-reset reset --dry-run

# Install test (Linux)
bash scripts/install-linux.sh --help
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] README modern and complete
- [ ] GitHub Actions triggers on push
- [ ] Artifacts ready for release