#!/usr/bin/env bash
set -euo pipefail

HEADER="/*\n * kystash - A simple image/file sharing server/client\n * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>\n */"

for file in "$@"; do
  existing_header=$(head -n 5 "$file")
  if [ "$(echo -e "$existing_header" | head -n 3)" != "$(echo -e "$HEADER" | head -n 3)" ]; then
    tmp=$(mktemp)
    echo -e "$HEADER" >"$tmp"

    first_line=$(head -n 1 "$file")
    if [ -n "$first_line" ]; then
      echo "" >>"$tmp"
    fi

    cat "$file" >>"$tmp"
    mv "$tmp" "$file"
  fi
done
