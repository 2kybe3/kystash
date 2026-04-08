#! /usr/bin/env bash

rm -rf results
mkdir -pv results/bin

targets=(
  "aarch64-unknown-linux-gnu"
  "aarch64-unknown-linux-musl"
  "x86_64-unknown-linux-gnu"
  "x86_64-unknown-linux-musl"
)

echo ">> starting building"
for target in "${targets[@]}"; do
  echo ">> building $target"
  cross build --locked --release --target "$target"
  cp "target/$target/release/kystash" "results/bin/$target"
done
echo ">> finished building"

echo ">> starting hashing"
: >results/hashes
for target in "${targets[@]}"; do
  HASH="$(sha256sum "results/bin/$target" | awk '{print $1}') $target"
  echo "$HASH" | tee -a results/hashes
done
echo ">> finished hashing"

DEPLOY="${DEPLOY:-}"
if [[ $DEPLOY != [yY] ]]; then
  echo ">> Not creating release"
  exit 0
fi

echo ">> Starting deployment"

# Creates a forgejo release (requires API_URL, OWNER, REPO, TOKEN)

URL="$FORGEJO_API_URL/repos/$FORGEJO_REPOSITORY/releases"
TOKEN="$ACTIONS_TOKEN"

VERSION="$(grep '^version' Cargo.toml | cut -d'"' -f2)"
TAG="v$VERSION"
echo ">> tag: $TAG"

BODY=$(printf "\`\`\`\n%s\n\`\`\`" "$(cat results/hashes)")

: >results/release.json
jq -n \
  --arg tag "$TAG" \
  --arg target "main" \
  --arg name "Release $TAG" \
  --arg body "$BODY" \
  '{tag_name: $tag, target_commitish: $target, name: $name, body: $body, draft: false, prerelease: false}' \
  >results/release.json

echo ">> release.json $(cat results/release.json)"

RESP="$(curl -X POST -H "Content-Type: application/json" -H "Authorization: token $TOKEN" -d "$(cat results/release.json)" "$URL")"
echo ">> Response: $RESP"

ID="$(echo "$RESP" | jq '.id')"
echo ">> Id: $ID"

echo ">> starting uploading assets"
for target in "${targets[@]}"; do
  echo ">> uploading $target"
  curl -X POST -H "Authorization: token $TOKEN" -F "attachment=@results/bin/$target" "$URL/$ID/assets?name=$target"
done
echo ">> finished uploading assets"
