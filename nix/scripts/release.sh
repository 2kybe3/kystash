#! /usr/bin/env bash

VERSION="$1"

CARGO_FILE="./Cargo.toml"

if [ -z "$VERSION" ]; then
  echo "USAGE: release <version>"
  exit 1
fi

if [ ! -f "$CARGO_FILE" ]; then
  echo "Can't find $CARGO_FILE"
  exit 1
fi

RESULT=$(sed -E "s|^version = \".*\"$|version = \"$VERSION\"|" $CARGO_FILE)

diff <(echo "$RESULT") $CARGO_FILE

read -r -n1 -p "Apply? (y/n): " answer
echo
if [[ $answer != [Yy] ]]; then
  exit 0
fi

printf "%s\n" "$RESULT" >$CARGO_FILE
