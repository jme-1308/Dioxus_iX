# Shared commands for you, coding agents and (later) CI. `just --list` shows them all.

set shell := ["bash", "-uc"]

# Type/borrow check (extra args go to cargo)
check *args:
    cargo check --features web --all-targets {{args}}

# Fast library-only check for the component edit loop
check-lib *args:
    cargo check --lib --features web {{args}}

# Check the desktop build (needs the WebKitGTK system packages)
check-desktop *args:
    cargo check --features desktop --all-targets {{args}}

# Clippy, with warnings as errors
lint:
    cargo clippy --features web --all-targets -- -D warnings

# Run the tests
test:
    cargo test --features ssr

# Format the code
fmt:
    cargo fmt

# NOTE: adjust the dx flags below if you normally run the gallery differently.
# Run the component gallery in the browser with hot reload
gallery:
    dx serve --example gallery --platform web --features web

# Check with the installed current stable compiler, bypassing local nightly flags.
# This does not declare or test a minimum supported Rust version.
check-stable:
    RUSTFLAGS="" cargo +stable check --features web --all-targets

# Install pinned Python parsers outside the checkout (requires network).
source-audit-setup:
    python3 -B tools/source_audit/bootstrap.py

# Read-only Rust component inventory; pass --json for attributes and byte ranges.
inventory-rust *args:
    "${SOURCE_AUDIT_PYTHON:-$HOME/.local/share/codex-py/bin/python}" -B tools/source_audit/cli.py {{args}}

# Fixture and real-source tests for the read-only inventory.
test-source-audit:
    "${SOURCE_AUDIT_PYTHON:-$HOME/.local/share/codex-py/bin/python}" -B -m unittest discover -s tools/source_audit/tests -v
