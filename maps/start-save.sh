factorio_root="$HOME/factorio"
factorio_bin="$factorio_root/bin/x64/factorio"
factorio_saves="$factorio_root/saves"
set -x

#  --start-server "lm-artful-work" \

$factorio_bin \
  --start-server "hyperdar-1_000" \
  --rcon-port 28016 \
  --rcon-password xana \
  --server-settings "server-settings.json" \
  --verbose
