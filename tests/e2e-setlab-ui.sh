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
rg -n 'shell_frame|shell_sidebar|shell_transport' src/app/shell.rs >/dev/null

for tab in playlist library transition master; do
  out="$ART/mixstudio-after-${tab}.png"
  MIXSTUDIO_DEMO_TAB="$tab" MIXSTUDIO_SCREENSHOT="$out" timeout 60 ./target/debug/mixstudio --demo "/tmp/ms-e2e-$tab" >/dev/null
  test -s "$out"
done

echo "ok artifact_dir=$ART"