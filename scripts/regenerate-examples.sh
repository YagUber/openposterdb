#!/usr/bin/env bash
set -euo pipefail

BASE="${1:-http://localhost:3000}"
KEY="t0-free-rpdb"
OUT="$(dirname "$0")/../web/public/examples"

declare -A POSTERS=(
  [nosferatu]=tt0013442
  [metropolis]=tt0017136
  [caligari]=tt0010323
  [phantom]=tt0016220
  [trip-to-moon]=tt0000417
  [safety-last]=tt0014429
  [the-general]=tt0017925
)

for name in "${!POSTERS[@]}"; do
  id="${POSTERS[$name]}"
  echo -n "$name ($id)... "
  curl -sf "$BASE/$KEY/imdb/poster-default/$id.jpg" -o "$OUT/$name.jpg"
  echo "OK"
done
