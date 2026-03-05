#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: $0 [options]

Options:
  --backend-port <port>    Backend port (default: 3000)
  --frontend-port <port>   Frontend dev port (default: 5173)
  --db-path <path>         SQLite DB path for backend (default: backend/tireswap.db)
  --update-db-first        Run backend --update-db before starting both services
  -h, --help               Show this help message

Environment:
  VITE_API_TOKEN           Optional token passed through to frontend dev server
USAGE
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_PORT="${BACKEND_PORT:-3000}"
FRONTEND_PORT="${FRONTEND_PORT:-5173}"
DB_PATH="${DB_PATH:-$ROOT_DIR/backend/tireswap.db}"
UPDATE_DB_FIRST=0
FRONTEND_HOST="${FRONTEND_HOST:-127.0.0.1}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --backend-port)
      BACKEND_PORT="$2"
      shift 2
      ;;
    --frontend-port)
      FRONTEND_PORT="$2"
      shift 2
      ;;
    --db-path)
      DB_PATH="$2"
      shift 2
      ;;
    --update-db-first)
      UPDATE_DB_FIRST=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

for cmd in cargo npm; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: required command not found: $cmd" >&2
    exit 1
  fi
done

API_BASE_URL="http://127.0.0.1:${BACKEND_PORT}"

list_port_listeners() {
  local port="$1"

  if ! command -v lsof >/dev/null 2>&1; then
    return 0
  fi

  lsof -nP -iTCP:"$port" -sTCP:LISTEN 2>/dev/null \
    | awk 'NR>1 {printf "%s (pid %s)\n", $1, $2}' \
    || true
}

ensure_port_available() {
  local port="$1"
  local label="$2"
  local listeners

  listeners="$(list_port_listeners "$port")"
  if [[ -n "$listeners" ]]; then
    echo "error: ${label} port ${port} is already in use:" >&2
    while IFS= read -r line; do
      [[ -z "$line" ]] && continue
      echo "  - ${line}" >&2
    done <<<"$listeners"
    echo "hint: stop the process above or choose a different port with --${label}-port <port>" >&2
    exit 1
  fi
}

ensure_port_available "$BACKEND_PORT" "backend"
ensure_port_available "$FRONTEND_PORT" "frontend"

prefix_stream() {
  local prefix="$1"
  while IFS= read -r line || [[ -n "$line" ]]; do
    printf '[%s] %s\n' "$prefix" "$line"
  done
}

BACKEND_PID=""
FRONTEND_PID=""
BACKEND_LOG_PID=""
FRONTEND_LOG_PID=""
LOG_DIR=""
CLEANED_UP=0

cleanup() {
  if [[ "$CLEANED_UP" -eq 1 ]]; then
    return
  fi
  CLEANED_UP=1

  for pid in "$FRONTEND_PID" "$BACKEND_PID"; do
    if [[ -n "$pid" ]] && kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
    fi
  done

  for pid in "$FRONTEND_LOG_PID" "$BACKEND_LOG_PID"; do
    if [[ -n "$pid" ]] && kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
    fi
  done

  sleep 1

  for pid in "$FRONTEND_PID" "$BACKEND_PID"; do
    if [[ -n "$pid" ]] && kill -0 "$pid" >/dev/null 2>&1; then
      kill -9 "$pid" >/dev/null 2>&1 || true
    fi
  done

  if [[ -n "$LOG_DIR" && -d "$LOG_DIR" ]]; then
    rm -rf "$LOG_DIR"
  fi
}

trap cleanup INT TERM EXIT

if [[ "$UPDATE_DB_FIRST" -eq 1 ]]; then
  echo "[debug] Updating local database first..."
  (
    cd "$ROOT_DIR/backend"
    cargo run -- --update-db --db-path "$DB_PATH"
  )
fi

LOG_DIR="$(mktemp -d)"
BACKEND_FIFO="$LOG_DIR/backend.log"
FRONTEND_FIFO="$LOG_DIR/frontend.log"
mkfifo "$BACKEND_FIFO" "$FRONTEND_FIFO"

prefix_stream "backend" <"$BACKEND_FIFO" &
BACKEND_LOG_PID=$!

prefix_stream "frontend" <"$FRONTEND_FIFO" &
FRONTEND_LOG_PID=$!

(
  cd "$ROOT_DIR/backend"
  cargo run -- --serve --port "$BACKEND_PORT" --db-path "$DB_PATH"
) >"$BACKEND_FIFO" 2>&1 &
BACKEND_PID=$!

(
  cd "$ROOT_DIR/frontend"
  VITE_API_BASE_URL="$API_BASE_URL" VITE_API_TOKEN="${VITE_API_TOKEN:-}" npm run dev -- --host "$FRONTEND_HOST" --port "$FRONTEND_PORT"
) >"$FRONTEND_FIFO" 2>&1 &
FRONTEND_PID=$!

echo "[debug] Backend:  $API_BASE_URL"
echo "[debug] Frontend: http://${FRONTEND_HOST}:${FRONTEND_PORT}"
echo "[debug] Press Ctrl+C to stop both services"

EXIT_CODE=0
while true; do
  if ! kill -0 "$BACKEND_PID" >/dev/null 2>&1; then
    wait "$BACKEND_PID" || EXIT_CODE=$?
    echo "[debug] Backend process exited"
    break
  fi

  if ! kill -0 "$FRONTEND_PID" >/dev/null 2>&1; then
    wait "$FRONTEND_PID" || EXIT_CODE=$?
    echo "[debug] Frontend process exited"
    break
  fi

  sleep 1
done

exit "$EXIT_CODE"
