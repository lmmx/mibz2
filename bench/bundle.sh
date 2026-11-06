#!/usr/bin/env bash
# One file to upload from a workstation run: DIR packed as DIR.tar.gz beside it, symbolic
# links (a shared corpus) and files over 50 MB left out, then its path and size printed.
# Every recipe that asks the user to upload results ends with it (CLAUDE.md, "Workstation
# recipes").
#   bash bench/bundle.sh target/team-size/20261106T100854Z
set -euo pipefail
dir=$(realpath "${1:?usage: bench/bundle.sh DIR}")
out="$dir.tar.gz"
( cd "$(dirname "$dir")" && find "$(basename "$dir")" -type f -size -50M -print0 \
  | tar --null -czf "$out.tmp" -T - )
mv "$out.tmp" "$out"
echo
echo "Upload this one file: $(realpath "$out") ($(du -h "$out" | cut -f1))"
