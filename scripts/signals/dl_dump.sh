#!/usr/bin/env bash
set -euo pipefail

URL="https://gist.githubusercontent.com/Bilka2/6b8a6a9e4a4ec779573ad703d03c1ae7/raw"
OUT="scripts/signals/data_raw.lua"

mkdir -p "$(dirname "${OUT}")"

tmpfile="$(mktemp)"
trap 'rm -f "$tmpfile"' EXIT

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$URL" -o "$tmpfile"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$tmpfile" "$URL"
else
  echo "Please install curl or wget to download the file." >&2
  exit 1
fi

mv "$tmpfile" "$OUT"
echo "Saved ${OUT}"

if command -v sed >/dev/null 2>&1; then
  sed -i '1s/.*/return {/' "$OUT"
else
  echo "Warning: 'sed' not found; file first line not modified." >&2
fi

exit 0
