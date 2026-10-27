#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if [ -d /Library/Developer/CommandLineTools ]; then
  export DEVELOPER_DIR="${DEVELOPER_DIR:-/Library/Developer/CommandLineTools}"
fi
export MACOSX_DEPLOYMENT_TARGET=14.0
version=$(python3 -c 'import json; print(json.load(open("package.json"))["version"])')
output="release/$version"
npm run release
mkdir -p "$output"
app="src-tauri/target/aarch64-apple-darwin/release/bundle/macos/CoquiPCCleaner.app"
/usr/bin/codesign --force --options runtime --sign - "$app"
/usr/bin/codesign --verify --deep --strict "$app"
/usr/bin/ditto -c -k --sequesterRsrc --keepParent "$app" "$output/CoquiPCCleaner-$version-macOS-arm64.zip"
(
  cd "$output"
  /usr/bin/shasum -a 256 "CoquiPCCleaner-$version-macOS-arm64.zip" > SHA256SUMS
)
