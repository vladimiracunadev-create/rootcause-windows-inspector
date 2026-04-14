#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
PS_SCRIPT="$ROOT_DIR/scripts/release-product.ps1"

if command -v pwsh >/dev/null 2>&1; then
  exec pwsh -NoProfile -ExecutionPolicy Bypass -File "$PS_SCRIPT" "$@"
fi

if command -v powershell >/dev/null 2>&1; then
  exec powershell -NoProfile -ExecutionPolicy Bypass -File "$PS_SCRIPT" "$@"
fi

echo "No se encontro pwsh ni powershell en PATH." >&2
exit 1
