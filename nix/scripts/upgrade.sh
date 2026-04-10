#! /usr/bin/env bash

VERSION="${1:-}"

if [[ -z $VERSION ]]; then
  echo "Error: VERSION is not set"
  echo "Usage: $0 <version>"
  exit 1
fi

cargo set-version "$VERSION"
sed -i "0,/version = \".*\"/s|version = \".*\"|version = \"$VERSION\"|" ./docs/default.nix
