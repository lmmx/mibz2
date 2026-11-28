# Pre-release physical i9 regression gate (docs/RELEASING.md §3.5).
#
# Run on the workstation via the standard queue or directly with `just submit`:
#   just submit bench-release-preflight
#   just submit bench-release-preflight <candidate_sha> <base_sha_or_tag> [recovery_base_sha_or_tag]
#
# Verify the gate locally before releasing (fail closed):
#   just release-preflight-verify
#   just release-preflight-status
#
# Once the bundle is archived on origin/evidence, whoever reads it records the receipt that
# `verify` looks for: a committed delta in frontier/deltas/ carrying these lines (backticks
# optional). `verify` re-judges the bundle from origin/evidence itself; the receipt only says
# which bundle, so a verdict written in it is never trusted.
#   - `preflight_candidate`: <40-hex candidate SHA>
#   - `preflight_base`: <40-hex SHA of the previous vX.Y.Z tag>
#   - `preflight_evidence_commit`: <40-hex origin/evidence commit holding the bundle>
#   - `preflight_bundle`: <path of the .tar.gz on origin/evidence; its .sha256 sits beside it>
#   - `preflight_sha256`: <the bundle's SHA-256>
#   - `preflight_recalibration`: completed | not required   (§5c: "completed", with the two
#     lines below, whenever worker-cost paths changed since the base tag)
#   - `preflight_recalibration_commit`: <40-hex commit the §5c sweep measured>
#   - `preflight_recalibration_evidence`: <its evidence commit and bundle>
#   - `preflight_recovery_base`, `preflight_recovery_cells`: only to add to what `verify`
#     auto-detects from bench/release-scaling/ (corpus:threads,...)
# Paths outside the published crate's and wheels' build inputs (docs, frontier, bench, tests,
# examples; not .just/ or the Justfile, which drive the release build) may change after the
# candidate without staling the receipt; anything else does (NON_PERFORMANCE_* in
# bench/release_preflight.py).

# Paired pre-tag release-candidate comparison on the i9 (wikidata + text + secondary guards).
# 12 rounds: ab rotates the arms' order and balances positions over 2 x arms rounds, so 12
# balances both the three-arm primary run (with a recovery base) and the two-arm guard run.
bench-release-preflight CANDIDATE="" BASE="" RECOVERY_BASE="" ROUNDS="12" THREADS="":
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
