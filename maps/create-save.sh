factorio_root="$HOME/factorio"
factorio_bin="$factorio_root/bin/x64/factorio"
factorio_saves="$factorio_root/saves"

$factorio_bin \
  --create "$factorio_saves/lm-artful-tmp" \
  --map-gen-settings ./lm-artful.mapGenSettings.json \
  --map-settings ./lm-artful.mapSettings.json