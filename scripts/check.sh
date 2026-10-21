#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if [ -d /Library/Developer/CommandLineTools ]; then
  export DEVELOPER_DIR="${DEVELOPER_DIR:-/Library/Developer/CommandLineTools}"
fi
npm run build
npm test
cargo fmt --manifest-path engine/Cargo.toml -- --check
cargo test --manifest-path engine/Cargo.toml --locked
cargo check --manifest-path src-tauri/Cargo.toml --locked
