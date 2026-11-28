# Frozen i9-10900K adoption run for the zero-table AVX2 ring CRC-32 (#1179's kernel) against
# its master merge base (claim crc32-avx2-ring-rescue; registration
# docs/journal/2026-11-28-crc32-ring-rescue/registration.md). One new directory per attempt
# under target/crc32-ring-rescue/, packed by bench/bundle.sh (`just submit bench-crc32-ring-rescue`).
bench-crc32-ring-rescue:
    #!/usr/bin/env bash
    set -euo pipefail
    export RUSTUP_TOOLCHAIN=1.99.1
    parent="$PWD/target/crc32-ring-rescue"
    mkdir -p "$parent"
    out="$parent/run-$(date -u +%Y%m%dT%H%M%SZ)"
    mkdir "$out"
    exec 3>&1 4>&2
    exec > >(tee -a "$out/console.log") 2>&1
    logger=$!
    finish() {
      rc=$?
      if [[ ! -s "$out/outcome.txt" ]]; then echo "INCOMPLETE (exit $rc)" > "$out/outcome.txt"; fi
      exec 1>&3 2>&4
      wait "$logger" || rc=1
      bash bench/bundle.sh "$out" || rc=1
      exit "$rc"
    }
    trap finish EXIT
    python3 bench/crc32_ring_rescue.py "$out"
