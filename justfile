# List the available commands.
default:
    just --list

# Build onchain program.
build:
    cargo build --manifest-path program/Cargo.toml

# Build the BPF/SPF target for deployment.
build-sbf:
    cargo build-sbf --manifest-path program/Cargo.toml

# Run the litesvm integrated tests.
test *args:
    cargo test --manifest-path tests/Cargo.toml -- --nocapture {{args}}

# Lint program crate that ships on chain only.
lint:
    cargo clippy --manifest-path program/Cargo.toml -- -D warnings
