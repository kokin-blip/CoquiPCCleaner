#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if [ -d /Library/Developer/CommandLineTools ]; then
  export DEVELOPER_DIR="${DEVELOPER_DIR:-/Library/Developer/CommandLineTools}"
fi
export MACOSX_DEPLOYMENT_TARGET=14.0
npm run desktop
