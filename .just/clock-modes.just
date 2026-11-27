# Frozen i9-10900K probe: separate clock modes (pinning, settle gap, pair position)
# and validate the mode-robust ab estimator (i9-clock-modes).
# Produces one self-contained run directory under target/clock-modes/run-<UTC> and packs it
# into one tarball via bench/bundle.sh (compatible with `just submit bench-clock-modes`).
bench-clock-modes *args:
    #!/usr/bin/env bash
    set -euo pipefail

    export RUSTUP_TOOLCHAIN=1.99.1

    parent="$PWD/target/clock-modes"
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

    python3 bench/clock_modes.py run "$out" {{args}}
