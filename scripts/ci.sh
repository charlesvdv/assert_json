#!/bin/sh

set -e

start=$(date -Iseconds -u)
host_name=$(hostname)
echo "Starting build at: ${start} on ${host_name}"

export RUST_BACKTRACE="full"

cargo deny check
cargo build --verbose
cargo test --verbose --all-features
cargo clippy --workspace --all-targets --all-features -- --deny warnings
