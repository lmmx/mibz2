#!/usr/bin/env bash
# One file to upload from a workstation run: DIR packed as DIR.tar.gz beside it, symbolic
# links (a shared corpus) and files over 50 MB left out, then its path and size printed.
# Every recipe that asks the user to upload results ends with it (CLAUDE.md, "Workstation
# recipes").
# It first records the toolchain the checkout selects (the one the recipe's builds used, from
# rust-toolchain.toml unless the recipe named another) in DIR/toolchain.txt, unless the recipe
# wrote one: `rustc -vV`, `cargo -V` and how rustup chose it. Best effort: a missing tool is
# recorded as missing, never a failed bundle.
#   bash bench/bundle.sh target/team-size/20261106T100854Z
set -euo pipefail
dir=$(realpath "${1:?usage: bench/bundle.sh DIR}")
out="$dir.tar.gz"
if [ ! -e "$dir/toolchain.txt" ]; then
  # (RUSTUP_AUTO_INSTALL=0: recording a toolchain never downloads one)
  { RUSTUP_AUTO_INSTALL=0 rustc -vV; RUSTUP_AUTO_INSTALL=0 cargo -V
    echo "rustup: $(RUSTUP_AUTO_INSTALL=0 rustup show active-toolchain 2>&1)"; } > "$dir/toolchain.txt" 2>&1 || true
fi
( cd "$(dirname "$dir")" && find "$(basename "$dir")" -type f -size -50M -print0 \
  | tar --null -czf "$out.tmp" -T - )
mv "$out.tmp" "$out"
echo
echo "Upload this one file: $(realpath "$out") ($(du -h "$out" | cut -f1))"
