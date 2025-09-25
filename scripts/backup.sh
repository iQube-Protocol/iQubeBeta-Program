#!/usr/bin/env bash
set -euo pipefail

# Backup script for iQubeBeta-Program
# - Creates a timestamped repository snapshot (excluding heavy/transient dirs)
# - Optionally builds the Next.js app and packages a production artifact
# - Writes/updates SHA256 checksums in backups/SHASUMS.txt
#
# Usage:
#   scripts/backup.sh              # snapshot only
#   scripts/backup.sh --build      # snapshot + build app artifact
#
# Requirements: tar, shasum, npm

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BACKUPS_DIR="$ROOT_DIR/backups"
APP_DIR="$ROOT_DIR/apps/aigent-z"
TS="$(date +%Y%m%d_%H%M%S)"

mkdir -p "$BACKUPS_DIR"

# 1) Repo snapshot (exclude heavy caches)
SNAP_NAME="az_repo_${TS}.tar.gz"
(
  cd "$ROOT_DIR"
  tar \
    --exclude='**/node_modules' \
    --exclude='**/.next' \
    --exclude='.dfx' \
    --exclude='backups' \
    -czf "$BACKUPS_DIR/$SNAP_NAME" .
)

echo "[backup] Repo snapshot: $BACKUPS_DIR/$SNAP_NAME"

# 2) Optional app build artifact
if [[ "${1:-}" == "--build" ]]; then
  echo "[backup] Building Next.js app (production) ..."
  (
    cd "$APP_DIR"
    npm ci
    npm run build
    ART_NAME="aigentz_build_${TS}.tar.gz"
    tar -czf "$BACKUPS_DIR/$ART_NAME" \
      .next public package.json package-lock.json .env.local \
      next.config.js tailwind.config.js postcss.config.js || {
        echo "[backup] WARN: Could not package some files (one or more may be missing)."
      }
    echo "[backup] App build artifact: $BACKUPS_DIR/$ART_NAME"
  )
fi

# 3) Checksums
(
  cd "$ROOT_DIR"
  # shellcheck disable=SC2046
  shasum -a 256 $(ls -1 "$BACKUPS_DIR"/*.tar.gz) >> "$BACKUPS_DIR/SHASUMS.txt"
)

echo "[backup] Checksums updated: $BACKUPS_DIR/SHASUMS.txt"

echo "[backup] Done."
