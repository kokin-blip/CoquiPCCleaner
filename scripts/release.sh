#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if [ -d /Library/Developer/CommandLineTools ]; then
  export DEVELOPER_DIR="${DEVELOPER_DIR:-/Library/Developer/CommandLineTools}"
fi
export MACOSX_DEPLOYMENT_TARGET=14.0
npm run release
mkdir -p release
app="src-tauri/target/aarch64-apple-darwin/release/bundle/macos/CoquiPCCleaner.app"
/usr/bin/codesign --verify --deep --strict "$app"
/usr/bin/ditto -c -k --sequesterRsrc --keepParent "$app" release/CoquiPCCleaner-macOS-arm64.zip
/usr/bin/shasum -a 256 release/CoquiPCCleaner-macOS-arm64.zip > release/SHA256SUMS
