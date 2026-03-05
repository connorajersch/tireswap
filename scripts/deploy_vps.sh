#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <release_id>"
  echo "Environment overrides:"
  echo "  APP_DIR=/opt/tireswap"
  echo "  ARTIFACT_PATH=<path to tar.gz>"
  echo "  SERVICE_NAME=tireswap-backend"
  echo "  HEALTHCHECK_URL=http://127.0.0.1:3000/health"
}

if [[ $# -ne 1 ]]; then
  usage >&2
  exit 1
fi

RELEASE_ID="$1"
APP_DIR="${APP_DIR:-/opt/tireswap}"
ARTIFACT_DIR="${ARTIFACT_DIR:-$APP_DIR/artifacts}"
ARTIFACT_PATH="${ARTIFACT_PATH:-$ARTIFACT_DIR/tireswap-$RELEASE_ID.tar.gz}"
RELEASES_DIR="${RELEASES_DIR:-$APP_DIR/releases}"
RELEASE_DIR="$RELEASES_DIR/$RELEASE_ID"
CURRENT_LINK="${CURRENT_LINK:-$APP_DIR/current}"
SERVICE_NAME="${SERVICE_NAME:-tireswap-backend}"
SYSTEMD_UNIT_PATH="${SYSTEMD_UNIT_PATH:-/etc/systemd/system/$SERVICE_NAME.service}"
NGINX_SITE_NAME="${NGINX_SITE_NAME:-tireswap}"
NGINX_SITE_AVAILABLE="${NGINX_SITE_AVAILABLE:-/etc/nginx/sites-available/$NGINX_SITE_NAME.conf}"
NGINX_SITE_ENABLED="${NGINX_SITE_ENABLED:-/etc/nginx/sites-enabled/$NGINX_SITE_NAME.conf}"
HEALTHCHECK_URL="${HEALTHCHECK_URL:-http://127.0.0.1:3000/health}"
RUN_USER="${RUN_USER:-tireswap}"
RUN_GROUP="${RUN_GROUP:-$RUN_USER}"
FORCE_DEPLOY="${FORCE_DEPLOY:-0}"

if [[ ! -f "$ARTIFACT_PATH" ]]; then
  echo "error: release artifact not found: $ARTIFACT_PATH" >&2
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

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: required command not found: $cmd" >&2
    exit 1
  fi
}

require_cmd tar
require_cmd systemctl
require_cmd nginx
require_cmd curl

run_root mkdir -p "$ARTIFACT_DIR" "$RELEASES_DIR" "$APP_DIR"

if [[ -d "$RELEASE_DIR" ]]; then
  if [[ "$FORCE_DEPLOY" == "1" ]]; then
    run_root rm -rf "$RELEASE_DIR"
  else
    echo "error: release already exists at $RELEASE_DIR (set FORCE_DEPLOY=1 to replace)" >&2
    exit 1
  fi
fi

run_root mkdir -p "$RELEASE_DIR"
run_root tar -xzf "$ARTIFACT_PATH" -C "$RELEASE_DIR"
run_root chmod +x "$RELEASE_DIR/backend/bin/tireswap-backend"

if ! id -u "$RUN_USER" >/dev/null 2>&1; then
  run_root useradd --system --home "$APP_DIR" --shell /usr/sbin/nologin "$RUN_USER"
fi

run_root mkdir -p /var/lib/tireswap /etc/tireswap
run_root chown -R "$RUN_USER:$RUN_GROUP" /var/lib/tireswap "$APP_DIR"
run_root install -m 0644 "$RELEASE_DIR/deploy/systemd/tireswap-backend.service" "$SYSTEMD_UNIT_PATH"
run_root install -m 0644 "$RELEASE_DIR/deploy/nginx/tireswap.conf" "$NGINX_SITE_AVAILABLE"
run_root ln -sfn "$NGINX_SITE_AVAILABLE" "$NGINX_SITE_ENABLED"

if [[ "${DISABLE_DEFAULT_NGINX_SITE:-1}" == "1" && -e /etc/nginx/sites-enabled/default ]]; then
  run_root rm -f /etc/nginx/sites-enabled/default
fi

run_root ln -sfn "$RELEASE_DIR" "$CURRENT_LINK"
run_root nginx -t
run_root systemctl daemon-reload
run_root systemctl enable "$SERVICE_NAME"
run_root systemctl restart "$SERVICE_NAME"
run_root systemctl reload nginx

sleep "${HEALTHCHECK_DELAY:-2}"
curl --fail --silent --show-error "$HEALTHCHECK_URL" >/dev/null

echo "Deployment successful"
echo "  release:   $RELEASE_ID"
echo "  current:   $CURRENT_LINK -> $RELEASE_DIR"
echo "  healthcheck: $HEALTHCHECK_URL"
