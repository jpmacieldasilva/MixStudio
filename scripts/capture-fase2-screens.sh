#!/usr/bin/env bash
# Capture demo screenshots for MixStudio phase 2 verification.
set -euo pipefail
cd "$(dirname "$0")/.."
export LIBRARY_PATH="${LIBRARY_PATH:-/usr/lib/gcc/x86_64-linux-gnu/12}"
export CXX="${CXX:-g++}"
export CC="${CC:-gcc}"
STORE="${MIXSTUDIO_STORE_MEDIA:-/cursor/stores/bc-f17bb68c-e023-4982-a956-41c30232fe31/media}"
PROPOSTAS="/cursor/stores/bc-f17bb68c-e023-4982-a956-41c30232fe31/internal/propostas"
PRANCHAS="${MIXSTUDIO_PRANCHAS:-$PROPOSTAS}"
mkdir -p "$STORE" "$STORE/comparison"
if [[ ! -f "$PRANCHAS/01-arrangement-proposta.png" ]]; then
  tar -xzf /cursor/stores/bc-f17bb68c-e023-4982-a956-41c30232fe31/internal/mixstudio-revisao-spec.tar.gz -C /tmp revisao/pranchas 2>/dev/null || true
  PRANCHAS="${MIXSTUDIO_PRANCHAS:-/tmp/mixstudio-revisao/revisao/pranchas}"
fi
cargo build --release --locked -q
BIN="./target/release/mixstudio"
DEMO_ROOT="/tmp/mixstudio-fase2-demo"

shot() {
  local tab=$1 w=$2 h=$3 out=$4
  rm -rf "$DEMO_ROOT"
  mkdir -p "$DEMO_ROOT"
  MIXSTUDIO_VIEWPORT_W="$w" MIXSTUDIO_VIEWPORT_H="$h" \
    MIXSTUDIO_DEMO_TAB="$tab" MIXSTUDIO_SCREENSHOT="$STORE/$out" \
    timeout 90 xvfb-run -s "-screen 0 ${w}x${h}x24" -a "$BIN" --demo "$DEMO_ROOT" >/dev/null
  test -s "$STORE/$out"
  python3 - <<PY
from PIL import Image
p="$STORE/$out"
im=Image.open(p)
if im.size!=(int("$w"),int("$h")):
    raise SystemExit(f"wrong capture size {im.size} for {p}")
print(f"captured {p} {im.size}")
PY
}

shot track 1440 900 mixstudio-fase2-arrangement-1440.png
shot transition 1440 900 mixstudio-fase2-transition-editor-1440.png
shot library 1440 900 mixstudio-fase2-library-1440.png
shot autoorder 1440 900 mixstudio-fase2-playlist-autoorder-1440.png
shot transition 1280 800 mixstudio-fase2-transition-1280.png

shot empty 1440 900 mixstudio-fase2-state-empty-1440.png
shot loading 1440 900 mixstudio-fase2-state-loading-1440.png
shot error 1440 900 mixstudio-fase2-state-error-1440.png
shot pending 1440 900 mixstudio-fase2-state-pending-1440.png

python3 - <<'PY'
from PIL import Image
store="/cursor/stores/bc-f17bb68c-e023-4982-a956-41c30232fe31/media"
labels=["Empty","Loading","Error","Pending"]
tiles=[f"{store}/mixstudio-fase2-state-{k}-1440.png" for k in ["empty","loading","error","pending"]]
ims=[Image.open(p) for p in tiles]
w,h=ims[0].size
grid=Image.new("RGBA",(w,h))
for i,im in enumerate(ims):
    x=(i%2)*(w//2)
    y=(i//2)*(h//2)
    grid.paste(im.resize((w//2,h//2),Image.Resampling.LANCZOS),(x,y))
out=f"{store}/mixstudio-fase2-states-1440.png"
grid.convert("RGB").save(out)
print("states grid",out,grid.size)
PY

export STORE PRANCHAS
python3 - <<PY
import os
from PIL import Image
store=os.environ["STORE"]
pr=os.environ["PRANCHAS"]
comp=f"{store}/comparison"
states_prop=f"{pr}/05-estados-proposta.png"
if not os.path.isfile(states_prop):
    alt="/tmp/mixstudio-revisao/revisao/pranchas/05-estados-proposta.png"
    if os.path.isfile(alt):
        states_prop=alt
pairs=[
("01-arrangement-proposta.png","mixstudio-fase2-arrangement-1440.png","mixstudio-compare-arrangement-1440.png"),
("02-editor-transicao-proposta.png","mixstudio-fase2-transition-editor-1440.png","mixstudio-compare-transition-editor-1440.png"),
("03-library-proposta.png","mixstudio-fase2-library-1440.png","mixstudio-compare-library-1440.png"),
("04-playlist-auto-order-proposta.png","mixstudio-fase2-playlist-autoorder-1440.png","mixstudio-compare-playlist-autoorder-1440.png"),
("02-editor-transicao-proposta.png","mixstudio-fase2-transition-1280.png","mixstudio-compare-transition-1280.png"),
]
if os.path.isfile(states_prop):
    pairs.append((states_prop,"mixstudio-fase2-states-1440.png","mixstudio-compare-states-1440.png"))
resolved=[]
for prop,app,out in pairs:
    path=prop if os.path.isabs(prop) or prop.startswith("/") else f"{pr}/{prop}"
    resolved.append((path,app,out))
for prop_path,app,out in resolved:
    a=Image.open(prop_path); b=Image.open(f"{store}/{app}")
    h=max(a.height,b.height)
    if a.height!=h: a=a.resize((int(a.width*h/a.height),h),Image.Resampling.LANCZOS)
    if b.height!=h: b=b.resize((int(b.width*h/b.height),h),Image.Resampling.LANCZOS)
    combo=Image.new("RGB",(a.width+b.width,h)); combo.paste(a,(0,0)); combo.paste(b,(a.width,0))
    combo.save(f"{comp}/{out}"); print("comparison",out,combo.size)
PY

echo "artifacts in $STORE"