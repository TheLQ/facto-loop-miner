factorio_root="$HOME/factorio"
factorio_bin="$factorio_root/bin/x64/factorio"
factorio_saves="$factorio_root/saves"

$factorio_bin \
  --start-server "loop-miner-chunk1000v2" \
  --rcon-port 28016 \
  --rcon-password xana \
  --server-settings "server-settings.json" \
  --verbose