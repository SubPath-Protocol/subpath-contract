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

## Drips Wave Contribution Guidelines

This repository participates in the **Stellar Wave Program** on the [Drips Network](https://drips.network/wave).

1. **Find an Issue**: Look for issues tagged with the `Stellar Wave` label.
2. **Apply to Contribute**: Apply directly on the Drips Wave portal before starting work so maintainers can assign the task and lock in your due date.
3. **Mandatory Issue Linking**: Every Pull Request description **must explicitly link** to the issue it resolves using GitHub keywords:
   - Example: `Fixes #12` or `Closes #12`.
   - Pull Requests lacking an issue link will fail the automated `PR Lint` CI check.
4. **Branch Workflow**: Fork the repository, create a descriptive feature branch (e.g. `feat/issue-12-metadata`), push your commits, and open a PR against `main`.
5. **Squash and Merge**: Maintainers review code and squash-merge approved contributions, automatically issuing Drips Wave points to contributors upon completion.

## Pull Request Guidelines

1. **One Logical Change Per Commit**: Keep commits modular and descriptive.
2. **Commit Convention**: Follow conventional commit formats (e.g., `feat:`, `fix:`, `docs:`, `test:`, `refactor:`).
3. **No Co-Author Attribution**: Do not include automated AI co-author lines.
4. **No Secrets**: Never commit private keys, secret seeds, or sensitive test data.
5. **Passing CI**: All Pull Requests must pass automated CI checks (`PR Lint`, `fmt`, `clippy`, `test`, `build`) before review.

## Reporting Issues

* Open an issue on GitHub describing bugs or proposed interface changes.
* For security vulnerabilities, follow the reporting process in [SECURITY.md](SECURITY.md).
