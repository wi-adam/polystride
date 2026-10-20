default:
    @just --list

# Check formatting, lint all targets including pedantic, and run the test harness.
check:
    cargo fmt --all -- --check
    cargo clippy --locked --offline --workspace --all-targets -- -D warnings
    cargo test --locked --offline --workspace

# Build the host-native development CLI.
build:
    cargo build --locked --offline --workspace

# Show the scaffold CLI.
run *args:
    cargo run --locked --offline -p polystride -- {{args}}

# Exercise a real local TLS coordinator/agent and retain raw evidence.
lab-smoke:
    python3 scripts/lab-smoke.py
