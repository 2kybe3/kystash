#!/usr/bin/env bash
set -euo pipefail

HEADER="/*\n * kystash - A simple image/file sharing server\n * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>\n */\n\n"

for file in "$@"; do
    existing_header=$(head -n 5 "$file")
    if [ "$existing_header" != "$(echo -e "$HEADER" | head -n 5)" ]; then
        tmp=$(mktemp)
        echo -e "$HEADER" > "$tmp"
        cat "$file" >> "$tmp"
        mv "$tmp" "$file"
    fi
done

