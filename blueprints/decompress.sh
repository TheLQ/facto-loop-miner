#!/bin/bash
set -euo pipefail
set -x

tail -c +2 "$1" | base64 -d  | pigz -dz
