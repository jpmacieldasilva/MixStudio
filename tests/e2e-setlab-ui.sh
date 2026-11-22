#!/usr/bin/env bash
# Verifica o tema SetLab e ausência de UI morta (aba Video, cue v2).
set -euo pipefail
cd "$(dirname "$0")/.."
export LIBRARY_PATH="${LIBRARY_PATH:-/usr/lib/gcc/x86_64-linux-gnu/12}"
export CXX="${CXX:-g++}"
export CC="${CC:-gcc}"

cargo build --locked -q
OUT="/opt/cursor/artifacts/mixstudio-setlab-ui-$(date +%Y%m%d).png"
DEMO=/tmp/ms-e2e-setlab-ui
rm -rf "$DEMO"

! rg -n 'Tab::Video|Headphone cue' src/app.rs

MIXSTUDIO_DEMO_TAB=library MIXSTUDIO_SCREENSHOT="$OUT" timeout 60 ./target/debug/mixstudio --demo "$DEMO" >/dev/null

test -s "$OUT"
echo "ok screenshot=$OUT"