#!/bin/sh
set -eu
# Usage from repo root: sh docs/journal/2026-11-18-astra-b-collective/extract.sh CORPUS WORKDIR
pin=1edb29fdc3d42dfddfc3d60e46456b611323760c
study=docs/journal/2026-11-18-astra-b-collective
corpus=$1
dest=$2
mkdir -p "$dest"
echo "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915  $corpus" | sha256sum -c - >&2
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
git archive "$pin" src | tar -x -C "$tmp"
cat "$tmp/src/lib.rs" "$study/extract.rs" > "$tmp/src/extract_main.rs"
rustc --edition=2021 -O "$tmp/src/extract_main.rs" -o "$tmp/extract"
"$tmp/extract" "$corpus" "$dest" > "$dest/inputs.csv"
