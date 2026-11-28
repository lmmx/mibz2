# Frozen C6 physical i9-10900K full-guard adoption recipe for the disjoint-slab
# four-lane forward walk (`forward-emission-adopt`, #1110 + #1217).
# Writes one self-contained run directory under target/forward-adopt/run-<UTC>
# and packs it into one tarball via bench/bundle.sh (`just submit bench-forward-adopt`).
bench-forward-adopt *args:
    #!/usr/bin/env bash
    set -euo pipefail

    export RUSTUP_TOOLCHAIN=1.99.1

    parent="$PWD/target/forward-adopt"
    mkdir -p "$parent"
    out="$parent/run-$(date -u +%Y%m%dT%H%M%SZ)"
    mkdir "$out"

    exec 3>&1 4>&2
    exec > >(tee -a "$out/console.log") 2>&1
    logger=$!

    finish() {
      rc=$?
      if [[ ! -s "$out/outcome.txt" ]]; then
        echo "INCOMPLETE (exit $rc)" > "$out/outcome.txt"
      fi
      exec 1>&3 2>&4
      wait "$logger" || rc=1
      bash bench/bundle.sh "$out" || rc=1
      exit "$rc"
    }
    trap finish EXIT

    python3 bench/forward_adopt.py run "$out" {{args}}
