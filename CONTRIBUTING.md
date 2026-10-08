# Contributing to SubPath Core Contracts

Thank you for your interest in contributing to the SubPath protocol contracts.

## Development Setup

### Prerequisites
* Rust (stable toolchain)
* `wasm32v1-none` target: `rustup target add wasm32v1-none`
* `stellar-cli` (v28.0.0 or later): `cargo install --locked stellar-cli`

### Local Commands
```bash
# Build WASM contract
make build
# or: stellar contract build

# Run unit tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --workspace --all-targets -- -D warnings
```

## Pull Request Guidelines

1. **One Logical Change Per Commit**: Keep commits modular and descriptive.
2. **Commit Convention**: Follow conventional commit formats (e.g., `feat:`, `fix:`, `docs:`, `test:`, `refactor:`).
3. **No Co-Author Attribution**: Do not include automated AI co-author lines.
4. **No Secrets**: Never commit private keys, secret seeds, or sensitive test data.
5. **Passing CI**: All Pull Requests must pass automated CI checks (`fmt`, `clippy`, `test`, `build`) before review.

## Reporting Issues

* Open an issue on GitHub describing bugs or proposed interface changes.
* For security vulnerabilities, follow the reporting process in [SECURITY.md](SECURITY.md).
