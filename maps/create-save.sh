factorio_root="$HOME/factorio"
factorio_bin="$factorio_root/bin/x64/factorio"
factorio_saves="$factorio_root/saves"

$factorio_bin \
  --create "$factorio_saves/loop-miner-chunk1000v2" \
  --map-gen-settings ./chunk1000v2.mapGenSettings.json \
  --map-settings ./chunk1000v2.mapSettings.json