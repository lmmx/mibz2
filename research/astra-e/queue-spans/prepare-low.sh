#!/usr/bin/env bash
# Reproduce the low-u16 checks and stage from the original span harness.
set -euo pipefail
cd "$(dirname "$0")"
for name in check ledger stage; do
  cp "$name.rs" "$name-low.rs"
done
patch -p0 --quiet --input=low-harness.patch
