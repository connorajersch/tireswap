#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <release_id>"
  echo "Environment: SKIP_BUILD=1 to skip running build scripts"
}

if [[ $# -ne 1 ]]; then
  usage >&2
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_ID="$1"
DIST_DIR="$ROOT_DIR/dist"
STAGE_DIR="$DIST_DIR/stage/$RELEASE_ID"
ARTIFACT_PATH="$DIST_DIR/tireswap-$RELEASE_ID.tar.gz"
BACKEND_BIN="$ROOT_DIR/backend/target/release/backend"
FRONTEND_DIST_DIR="$ROOT_DIR/frontend/dist"

if [[ "${SKIP_BUILD:-0}" != "1" ]]; then
  "$ROOT_DIR/scripts/build_backend.sh"
  "$ROOT_DIR/scripts/build_frontend.sh"
fi

if [[ ! -f "$BACKEND_BIN" ]]; then
  echo "error: expected backend binary at $BACKEND_BIN" >&2
  exit 1
fi

if [[ ! -d "$FRONTEND_DIST_DIR" ]]; then
  echo "error: expected frontend build output at $FRONTEND_DIST_DIR" >&2
  exit 1
fi

rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR/backend/bin" "$STAGE_DIR/frontend" "$STAGE_DIR/deploy/systemd" "$STAGE_DIR/deploy/nginx" "$STAGE_DIR/scripts" "$DIST_DIR"

cp "$BACKEND_BIN" "$STAGE_DIR/backend/bin/tireswap-backend"
chmod +x "$STAGE_DIR/backend/bin/tireswap-backend"
cp -R "$FRONTEND_DIST_DIR"/. "$STAGE_DIR/frontend/"
cp "$ROOT_DIR/deploy/systemd/tireswap-backend.service" "$STAGE_DIR/deploy/systemd/tireswap-backend.service"
cp "$ROOT_DIR/deploy/nginx/tireswap.conf" "$STAGE_DIR/deploy/nginx/tireswap.conf"
cp "$ROOT_DIR/scripts/deploy_vps.sh" "$ROOT_DIR/scripts/rollback_vps.sh" "$STAGE_DIR/scripts/"
chmod +x "$STAGE_DIR/scripts/deploy_vps.sh" "$STAGE_DIR/scripts/rollback_vps.sh"
printf "%s\n" "$RELEASE_ID" > "$STAGE_DIR/RELEASE_ID"

tar -C "$STAGE_DIR" -czf "$ARTIFACT_PATH" .

if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$ARTIFACT_PATH" > "$ARTIFACT_PATH.sha256"
elif command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "$ARTIFACT_PATH" > "$ARTIFACT_PATH.sha256"
fi

echo "Release artifact created: $ARTIFACT_PATH"
