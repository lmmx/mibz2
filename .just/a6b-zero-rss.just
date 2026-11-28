# Frozen C6 physical i9-10900K verification recipe for `a6b-zero-rss-endpoint-build`:
# paired alternating `ab run` of zero-extra-RSS `overlay_aos` (`src/open/endpoint.rs`,
# `src/open.rs`) against its frozen master BASE across `mibz2-workers-lazy` (wikidata 1/6/8W),
# `mibz2` (wikidata 1/4/6/8/10W), and all 6 guard corpora (1/8W).
# Writes one self-contained run directory under target/a6b-zero-rss/run-<UTC>
# and packs it into one tarball via bench/bundle.sh (`just submit bench-a6b-zero-rss`).
bench-a6b-zero-rss *args:
    #!/usr/bin/env bash
    set -euo pipefail

    export RUSTUP_TOOLCHAIN=1.99.1

    parent="$PWD/target/a6b-zero-rss"
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

    python3 bench/a6b_zero_rss_i9.py run "$out" {{args}}
