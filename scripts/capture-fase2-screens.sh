#!/usr/bin/env bash
# Capture demo screenshots for MixStudio phase 2 verification.
set -euo pipefail
cd "$(dirname "$0")/.."
export LIBRARY_PATH="${LIBRARY_PATH:-/usr/lib/gcc/x86_64-linux-gnu/12}"
export CXX="${CXX:-g++}"
export CC="${CC:-gcc}"
STORE="${MIXSTUDIO_STORE_MEDIA:-/cursor/stores/bc-f17bb68c-e023-4982-a956-41c30232fe31/media}"
PRANCHAS="${MIXSTUDIO_PRANCHAS:-/tmp/mixstudio-revisao/revisao/pranchas}"
mkdir -p "$STORE" "$STORE/comparison"
cargo build --release --locked -q
BIN="./target/release/mixstudio"
DEMO_ROOT="/tmp/mixstudio-fase2-demo"
rm -rf "$DEMO_ROOT"
mkdir -p "$DEMO_ROOT"

shot() {
  local tab=$1 w=$2 h=$3 out=$4
  rm -rf "$DEMO_ROOT"
  mkdir -p "$DEMO_ROOT"
  MIXSTUDIO_VIEWPORT_W="$w" MIXSTUDIO_VIEWPORT_H="$h" \
    MIXSTUDIO_DEMO_TAB="$tab" MIXSTUDIO_SCREENSHOT="$STORE/$out" \
    timeout 90 xvfb-run -a "$BIN" --demo "$DEMO_ROOT" >/dev/null
  test -s "$STORE/$out"
  echo "captured $out (${w}x${h})"
}

# 1440×900 matrix
shot track 1440 900 mixstudio-fase2-arrangement-1440.png
shot transition 1440 900 mixstudio-fase2-transition-editor-1440.png
shot library 1440 900 mixstudio-fase2-library-1440.png
shot autoorder 1440 900 mixstudio-fase2-playlist-autoorder-1440.png
shot empty 1440 900 mixstudio-fase2-empty-1440.png
shot loading 1440 900 mixstudio-fase2-loading-1440.png
shot transition 1280 800 mixstudio-fase2-transition-1280.png

pair() {
  local prop=$1 app=$2 out=$3
  if [[ -f "$PRANCHAS/$prop" && -f "$STORE/$app" ]]; then
    convert "$PRANCHAS/$prop" "$STORE/$app" +append "$STORE/comparison/$out"
    echo "comparison $out"
  else
    echo "skip comparison $out (missing $prop or $app)" >&2
  fi
}

if command -v convert >/dev/null 2>&1; then
  pair 01-arrangement-proposta.png mixstudio-fase2-arrangement-1440.png mixstudio-compare-arrangement-1440.png
  pair 02-editor-transicao-proposta.png mixstudio-fase2-transition-editor-1440.png mixstudio-compare-transition-editor-1440.png
  pair 03-library-proposta.png mixstudio-fase2-library-1440.png mixstudio-compare-library-1440.png
  pair 04-playlist-auto-order-proposta.png mixstudio-fase2-playlist-autoorder-1440.png mixstudio-compare-playlist-autoorder-1440.png
  pair 06-transicao-1280-proposta.png mixstudio-fase2-transition-1280.png mixstudio-compare-transition-1280.png
  pair 05-estados-proposta.png mixstudio-fase2-empty-1440.png mixstudio-compare-states-empty-1440.png
fi

echo "artifacts in $STORE"