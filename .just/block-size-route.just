# Frozen C1a diagnostic calibration recipe for `block-size-route-i9-probe`
# (Step 1 of `open-admission-block-size`): per-block Open vs Segmented across
# bzip2 -1, -2, -3, -5, -9 (wikidata) and -1, -9 (text) on physical i9-10900K.
# Writes one self-contained run directory under target/block-size-route/run-<UTC>
# and packs it into one tarball via bench/bundle.sh (`just submit bench-block-size-route`).
bench-block-size-route *args:
    #!/usr/bin/env bash
    set -euo pipefail

    export RUSTUP_TOOLCHAIN=1.99.1

    parent="$PWD/target/block-size-route"
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

    python3 bench/block_size_route.py run "$out" {{args}}
