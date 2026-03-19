#!/usr/bin/env bash
set -euo pipefail

BASE="${1:-http://localhost:3000}"
KEY="t0-free-rpdb"
OUT="$(dirname "$0")/../web/public/examples"

declare -A POSTERS=(
  [namakura-gatana]=tt0438075
  [nosferatu]=tt0013442
  [metropolis]=tt0017136
  [caligari]=tt0010323
  [phantom]=tt0016220
  [trip-to-moon]=tt0000417
  [safety-last]=tt0014429
  [the-general]=tt0017925
)

# --- Standard examples (no settings changes needed) ---

echo "=== Posters ==="
for name in "${!POSTERS[@]}"; do
  id="${POSTERS[$name]}"
  echo -n "poster: $name ($id)... "
  curl -sf "$BASE/$KEY/imdb/poster-default/$id.jpg" -o "$OUT/$name.jpg"
  echo "OK"
done

echo "=== Logos ==="
for name in "${!POSTERS[@]}"; do
  id="${POSTERS[$name]}"
  echo -n "logo: $name ($id)... "
  if curl -sf "$BASE/$KEY/imdb/logo-default/$id.png" -o "$OUT/logo-$name.png"; then
    echo "OK"
  else
    rm -f "$OUT/logo-$name.png"
    echo "SKIP (not available)"
  fi
done

echo "=== Backdrops ==="
for name in "${!POSTERS[@]}"; do
  id="${POSTERS[$name]}"
  echo -n "backdrop: $name ($id)... "
  if curl -sf "$BASE/$KEY/imdb/backdrop-default/$id.jpg" -o "$OUT/backdrop-$name.jpg"; then
    echo "OK"
  else
    rm -f "$OUT/backdrop-$name.jpg"
    echo "SKIP (not available)"
  fi
done

# --- Preview images (use the preview endpoint with query params) ---

echo ""
echo "=== Preview Images ==="
echo "These use the preview endpoint (no real movies needed)."
echo -n "Enter full 64-character API key (or press Enter to skip): "
read -r API_KEY

if [[ -z "$API_KEY" ]]; then
  echo "Skipping preview images."
  exit 0
fi

# Log in with the API key to get a JWT token
LOGIN_RESPONSE=$(curl -sf -X POST "$BASE/api/auth/key-login" \
  -H "Content-Type: application/json" \
  -d "$(jq -n --arg k "$API_KEY" '{"api_key": $k}')" 2>&1) || {
  echo "ERROR: API key rejected. Check the key and that the server is running."
  exit 1
}

JWT_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')

if [[ -z "$JWT_TOKEN" || "$JWT_TOKEN" == "null" ]]; then
  echo "ERROR: Failed to obtain auth token."
  exit 1
fi

AUTH=(-H "Authorization: Bearer $JWT_TOKEN")
PREVIEW="$BASE/api/key/me/preview"
echo "Authenticated."
echo ""

# Position variants
echo "--- Positions ---"
for pos in tl tc tr r bl bc br l; do
  echo -n "pos-$pos... "
  curl -sf "${AUTH[@]}" \
    "$PREVIEW/poster?poster_position=$pos" \
    -o "$OUT/pos-$pos.jpg"
  echo "OK"
done

# Badge style variants
echo "--- Badge Styles ---"
for style in h v; do
  echo -n "style-$style... "
  curl -sf "${AUTH[@]}" \
    "$PREVIEW/poster?badge_style=$style" \
    -o "$OUT/style-$style.jpg"
  echo "OK"
done

# Label style variants
echo "--- Label Styles ---"
for label_val in i t o; do
  case "$label_val" in
    i) label_name="icon" ;;
    t) label_name="text" ;;
    o) label_name="official" ;;
  esac
  echo -n "label-$label_name... "
  curl -sf "${AUTH[@]}" \
    "$PREVIEW/poster?label_style=$label_val&ratings_limit=0" \
    -o "$OUT/label-$label_name.jpg"
  echo "OK"
done

# Logo badge style variants
echo "--- Logo Styles ---"
for style in h v; do
  echo -n "logo-$style... "
  curl -sf "${AUTH[@]}" \
    "$PREVIEW/logo?badge_style=$style" \
    -o "$OUT/logo-$style.png"
  echo "OK"
done

# Backdrop badge style variants
echo "--- Backdrop Styles ---"
for style in h v; do
  echo -n "backdrop-$style... "
  curl -sf "${AUTH[@]}" \
    "$PREVIEW/backdrop?badge_style=$style" \
    -o "$OUT/backdrop-$style.jpg"
  echo "OK"
done

# Badge size variants for posters
echo "--- Poster Badge Sizes ---"
for size in xs s m l xl; do
  echo -n "size-poster-$size... "
  extra=""
  [[ "$size" == "l" || "$size" == "xl" ]] && extra="&ratings_limit=2"
  curl -sf "${AUTH[@]}" \
    "$PREVIEW/poster?badge_size=$size$extra" \
    -o "$OUT/size-poster-$size.jpg"
  echo "OK"
done

# Badge size variants for logos
echo "--- Logo Badge Sizes ---"
for size in xs s m l xl; do
  echo -n "size-logo-$size... "
  extra=""
  [[ "$size" == "l" || "$size" == "xl" ]] && extra="&ratings_limit=2"
  curl -sf "${AUTH[@]}" \
    "$PREVIEW/logo?badge_size=$size$extra" \
    -o "$OUT/size-logo-$size.png"
  echo "OK"
done

# Badge size variants for backdrops
echo "--- Backdrop Badge Sizes ---"
for size in xs s m l xl; do
  echo -n "size-backdrop-$size... "
  extra=""
  [[ "$size" == "l" || "$size" == "xl" ]] && extra="&ratings_limit=2"
  curl -sf "${AUTH[@]}" \
    "$PREVIEW/backdrop?badge_size=$size$extra" \
    -o "$OUT/size-backdrop-$size.jpg"
  echo "OK"
done

echo ""
echo "Done!"
