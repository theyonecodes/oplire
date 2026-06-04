# Contributing to oplire

Thanks for your interest in contributing!

## Development Setup

1. Install Rust: https://rustup.rs
2. Clone the repo
3. `cargo build`
4. `cargo test`

## Versioning

We follow [Semantic Versioning](https://semver.org/):

```
MAJOR.MINOR.PATCH
```

| Increment | When | Example |
|-----------|------|---------|
| **MAJOR** | Breaking changes (CLI args, config format, API changes) | 1.0.0 → 2.0.0 |
| **MINOR** | New features, backwards compatible | 2.5.0 → 2.6.0 |
| **PATCH** | Bug fixes, small improvements | 2.6.0 → 2.6.1 |

### Pre-release

For testing before a stable release:

```
2.7.0-beta.1
2.7.0-rc.1
```

## Release Process

### Automated (Recommended)

1. Update version in `Cargo.toml`
2. Update version in `README.md` download link
3. Commit: `git commit -m "bump version to X.Y.Z"`
4. Tag: `git tag vX.Y.Z`
5. Push: `git push origin master --tags`

GitHub Actions will automatically:
- Build binaries for Windows, Linux, macOS
- Create a GitHub Release with auto-generated notes
- Upload all platform binaries as release assets

### Manual

If you need to create a release manually:

```bash
cargo build --release
gh release create vX.Y.Z \
  target/release/oplire.exe \
  --title "Release vX.Y.Z" \
  --notes "Release notes here"
```

## Pull Requests

1. Create a feature branch
2. Make your changes
3. Ensure `cargo test` and `cargo clippy` pass
4. Open a PR with a clear description
5. Link any related issues

## Code Style

- Follow Rust idioms (`cargo fmt`)
- No warnings (`cargo clippy -- -D warnings`)
- Add tests for new functionality
- Update documentation if needed

## Questions?

Open an issue or start a discussion.
