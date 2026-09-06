#!/usr/bin/env bash
# Deploy CivitForge to the cachyos-server (tailnet 100.64.0.2).
# Usage: ./deploy/civitforge.sh [--skip-build] [--skip-dist]
set -euo pipefail

SERVER=${SERVER:-cachyos-server}
REMOTE_DIR=/home/wyatt/civitforge
LOCAL_REPO=/home/wyatt/dev/src/github.com/WyattAu/CivitForge

log() { printf '\033[1;34m[deploy]\033[0m %s\n' "$*"; }

SKIP_BUILD=false; SKIP_DIST=false
for arg in "$@"; do
  case "$arg" in
    --skip-build) SKIP_BUILD=true ;;
    --skip-dist)  SKIP_DIST=true ;;
  esac
done

if [ "$SKIP_BUILD" = false ]; then
  log "building release binary locally..."
  (cd "$LOCAL_REPO" && cargo build --release --bin civit-core)
fi

log "rsyncing binary to $SERVER ..."
rsync -z --checksum "$LOCAL_REPO/target/release/civit-core" \
  "$SERVER:$REMOTE_DIR/target/release/civit-core.new"

if [ "$SKIP_DIST" = false ]; then
  log "rsyncing UI dist ..."
  rsync -az --delete "$LOCAL_REPO/crates/civit-ui/dist/" "$SERVER:$REMOTE_DIR/crates/civit-ui/dist/"
fi

log "installing binary + systemd unit (atomic swap) ..."
ssh "$SERVER" "
  set -e
  mv $REMOTE_DIR/target/release/civit-core.new $REMOTE_DIR/target/release/civit-core
  chmod +x $REMOTE_DIR/target/release/civit-core
  sudo cp $REMOTE_DIR/deploy/civitforge.service /etc/systemd/system/civitforge.service
  sudo systemctl daemon-reload
  sudo systemctl enable --now civitforge
  sudo systemctl restart civitforge
"

log "waiting for healthz ..."
for i in $(seq 1 30); do
  if ssh "$SERVER" "curl -fsS --max-time 2 http://127.0.0.1:8080/healthz" 2>/dev/null | grep -q OK; then
    break
  fi
  sleep 1
done

log "deployment complete. version:"
ssh "$SERVER" "curl -fsS --max-time 3 http://127.0.0.1:8080/api/v1/version" || log "(version endpoint unreachable — check journalctl -u civitforge)"
