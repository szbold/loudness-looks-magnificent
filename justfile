# Single command surface for agents and humans (AGENTS.md).
# Run `just --list` to see every recipe.

default:
    @just --list

# Install prerequisites: pinned toolchain (rust-toolchain.toml) + git hooks.
setup:
    rustup toolchain install
    git config core.hooksPath .githooks
    @test -x .githooks/prepare-commit-msg || { echo '.githooks scripts must be executable (chmod +x)'; exit 1; }
    @echo "hooks installed: core.hooksPath = $(git config core.hooksPath)"

# Full local mirror of CI: format check, clippy, tests, e2e.
verify: fmt-check lint test e2e
    @echo "verify: all checks green"

# Check formatting (rustfmt defaults, no overrides).
fmt-check:
    cargo fmt --all -- --check

# Auto-format all targets.
fmt:
    cargo fmt --all

# Lint: clippy, warnings denied (CONVENTIONS.md lints policy).
lint:
    cargo clippy --all-targets -- -D warnings

# Unit + trait-level integration tests (e2e has its own recipe below).
test:
    cargo test --lib --test pipeline

# End-to-end: drive the real binary headless (FR-M1-6, see e2e/).
e2e:
    cargo test --test e2e

# Release build.
build:
    cargo build --release
