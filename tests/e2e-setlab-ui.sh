#!/usr/bin/env bash
# E2E: shell SetLab (sidebar + toolbar + transport) e telas principais.
set -euo pipefail
cd "$(dirname "$0")/.."
export LIBRARY_PATH="${LIBRARY_PATH:-/usr/lib/gcc/x86_64-linux-gnu/12}"
export CXX="${CXX:-g++}"
export CC="${CC:-gcc}"
ART="/opt/cursor/artifacts"
mkdir -p "$ART"

cargo build --locked -q

! rg -n 'Tab::Video|Headphone cue' src/app.rs
test -f src/app/shell.rs
test -f src/app/studio_nav.rs
rg -n 'shell_frame|studio_sidebar_body|shell_transport' src/app/shell.rs >/dev/null
rg -n 'studio_track_picker|LibDrag' src/app/studio_nav.rs >/dev/null

STORE_MEDIA="${MIXSTUDIO_STORE_MEDIA:-}"
mkdir -p "$STORE_MEDIA" 2>/dev/null || true

for tab in playlist library transition master; do
  name="$tab"
  [[ "$tab" == "playlist" ]] && name="arrangement"
  out="$ART/mixstudio-daw-${name}.png"
  MIXSTUDIO_DEMO_TAB="$tab" MIXSTUDIO_SCREENSHOT="$out" timeout 60 ./target/debug/mixstudio --demo "/tmp/ms-e2e-$tab" >/dev/null
  test -s "$out"
  if [[ -n "$STORE_MEDIA" ]]; then
    cp -f "$out" "$STORE_MEDIA/mixstudio-daw-${name}.png"
  fi
done

echo "ok artifact_dir=$ART store_media=${STORE_MEDIA:-none}"