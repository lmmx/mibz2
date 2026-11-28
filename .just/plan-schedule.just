# Frozen C6 physical i9-10900K verification recipes for:
# - `plan-schedule-and-lazy-threshold` (`bench-plan-schedule`)
# - `trim-at-measured-kstar` (`bench-trim-kstar`)
#
# Both produce one self-contained run directory under target/plan-schedule/run-<UTC>
# (or target/trim-kstar/run-<UTC>) and pack it into one tarball via bench/bundle.sh
# (directly compatible with `just submit`).

bench-plan-schedule *args:
    #!/usr/bin/env bash
    set -euo pipefail
    export RUSTUP_TOOLCHAIN=1.99.1
    parent="$PWD/target/plan-schedule"
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
    python3 bench/plan_schedule.py run "$out" --mode plan-schedule {{ args }}

bench-trim-kstar *args:
    #!/usr/bin/env bash
    set -euo pipefail
    export RUSTUP_TOOLCHAIN=1.99.1
    parent="$PWD/target/trim-kstar"
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
    python3 bench/plan_schedule.py run "$out" --mode trim-kstar {{ args }}
