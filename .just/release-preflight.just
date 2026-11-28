# Pre-release physical i9 regression gate (docs/RELEASING.md §3.5).
#
# Run on the workstation via the standard queue or directly with `just submit`:
#   just submit bench-release-preflight
#   just submit bench-release-preflight <candidate_sha> <base_sha_or_tag> [recovery_base_sha_or_tag]
#
# Verify the gate locally before releasing (fail closed):
#   just release-preflight-verify
#   just release-preflight-status

# Paired pre-tag release-candidate comparison on the i9 (wikidata + text + secondary guards)
bench-release-preflight CANDIDATE="" BASE="" RECOVERY_BASE="" ROUNDS="10" THREADS="":
    #!/usr/bin/env bash
    set -euo pipefail
    B=$PWD/target/release-preflight
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-release-preflight: $W already exists; not reusing an attempt directory"; exit 1; }
    exec > >(tee -a "$W/console.log") 2>&1
    outcome="INCOMPLETE: stopped before the preflight finished (see console.log)"
    finish() {
      status=$?
      if [ -s "$W/outcome.txt" ]; then
        outcome=$(cat "$W/outcome.txt")
      else
        echo "$outcome" > "$W/outcome.txt"
      fi
      echo "bench-release-preflight: OUTCOME: $outcome"
      bash bench/bundle.sh "$W" || { echo "bench-release-preflight: bundling failed"; exit 2; }
      case "$outcome" in
        "VALID PASS"*) exit 0 ;;
        "VALID FAIL"*|INVALID*|REFUSED*) exit 1 ;;
        *) [ "$status" -ne 0 ] && exit "$status"; exit 2 ;;
      esac
    }
    trap finish EXIT
    args=(--out "$W" --rounds "{{ROUNDS}}")
    [ -n "{{CANDIDATE}}" ] && args+=(--candidate "{{CANDIDATE}}")
    [ -n "{{BASE}}" ] && args+=(--base "{{BASE}}")
    [ -n "{{RECOVERY_BASE}}" ] && args+=(--recovery-base "{{RECOVERY_BASE}}")
    [ -n "{{THREADS}}" ] && args+=(--threads "{{THREADS}}")
    python3 bench/release_preflight.py run "${args[@]}"

# Verify the fail-closed pre-release i9 regression gate before any bump, tag, push, or publish
[positional-arguments]
release-preflight-verify *args:
    python3 bench/release_preflight.py verify "$@"

# Show the current pre-release i9 gate status (read-only)
[positional-arguments]
release-preflight-status *args:
    python3 bench/release_preflight.py status "$@"
