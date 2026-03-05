#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <release_id>"
  echo "Environment overrides:"
  echo "  APP_DIR=/opt/tireswap"
  echo "  SERVICE_NAME=tireswap-backend"
  echo "  HEALTHCHECK_URL=http://127.0.0.1:3000/health"
}

if [[ $# -ne 1 ]]; then
  usage >&2
  exit 1
fi

RELEASE_ID="$1"
APP_DIR="${APP_DIR:-/opt/tireswap}"
RELEASES_DIR="${RELEASES_DIR:-$APP_DIR/releases}"
TARGET_RELEASE_DIR="$RELEASES_DIR/$RELEASE_ID"
CURRENT_LINK="${CURRENT_LINK:-$APP_DIR/current}"
SERVICE_NAME="${SERVICE_NAME:-tireswap-backend}"
HEALTHCHECK_URL="${HEALTHCHECK_URL:-http://127.0.0.1:3000/health}"

if [[ ! -d "$TARGET_RELEASE_DIR" ]]; then
  echo "error: release does not exist: $TARGET_RELEASE_DIR" >&2
  exit 1
fi

if [[ $(id -u) -eq 0 ]]; then
  SUDO_BIN=""
else
  if ! command -v sudo >/dev/null 2>&1; then
    echo "error: sudo is required when not running as root" >&2
    exit 1
  fi
  SUDO_BIN="sudo"
fi

run_root() {
  if [[ -n "$SUDO_BIN" ]]; then
    "$SUDO_BIN" "$@"
  else
    "$@"
  fi
}

run_root ln -sfn "$TARGET_RELEASE_DIR" "$CURRENT_LINK"
run_root systemctl daemon-reload
run_root systemctl restart "$SERVICE_NAME"
run_root systemctl reload nginx

sleep "${HEALTHCHECK_DELAY:-2}"
curl --fail --silent --show-error "$HEALTHCHECK_URL" >/dev/null

echo "Rollback successful"
echo "  current: $CURRENT_LINK -> $TARGET_RELEASE_DIR"
