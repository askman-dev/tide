#!/bin/bash

set -euo pipefail

APP_NAME="Tide"
BIN_NAME="tide"
APP_BUNDLE=".build/${APP_NAME}.app"

echo "Building ${APP_NAME}..."

cargo build --release

BIN_PATH="target/release/${BIN_NAME}"
if [ ! -f "${BIN_PATH}" ]; then
  echo "Build output not found: ${BIN_PATH}"
  exit 1
fi

if [ -d "${APP_BUNDLE}" ]; then
  rm -rf "${APP_BUNDLE}"
fi

mkdir -p "${APP_BUNDLE}/Contents/MacOS"
mkdir -p "${APP_BUNDLE}/Contents/Resources"

cp "${BIN_PATH}" "${APP_BUNDLE}/Contents/MacOS/"
cp "resources/Info.plist" "${APP_BUNDLE}/Contents/"

chmod +x "${APP_BUNDLE}/Contents/MacOS/${BIN_NAME}"

SIGN_CERT="Mac Developer Certificate"

echo "Signing with '${SIGN_CERT}'..."
codesign --force --deep --sign "${SIGN_CERT}" "${APP_BUNDLE}" 2>/dev/null || true

echo "Build complete: ${APP_BUNDLE}"
echo "Launching..."
open "${APP_BUNDLE}"
