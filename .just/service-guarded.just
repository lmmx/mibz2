# The guarded service default's workstation A/B (docs/journal/2026-11-09-service-guarded-registration.md:
# a DRAFT, not frozen, not queued; bench/service-guarded.frozen names no commits yet, so the
# recipe refuses). The candidate build's default decoder (svc2 at exactly 10 threads on its
# target) against the current default's build, both through `BzReader::parallel` (ab's
# `mibz2`), at the pipelined default's grid (8, 9, 10, 11, 12, 16, 20 threads), 20 paired rounds on the bench's Wikidata sample, with the
# base binary's byte-identical twin as the noise control, judged by bench/service_guarded.py
# from the attempt's directory. An actual-target preflight (examples/svc_target.rs, checked
# whole by bench/svc_target_check.py) runs first and refuses before anything is measured
# unless it is TARGET; the refusal is archived in the bundle. No PMU telemetry
# (--machine-state is not passed); the diagnostics (`ab env --facts`, lscpu, `ab sample` and
# the raw cgroup discovery reads) are passive, each bounded (TERM at 30 s, KILL 5 s after),
# each one's exit archived, never judged (D064), and a failing one never refuses the attempt
# or touches a timed row. One attempt; no replacement is registered, and no retry-to-pass.
#   just submit bench-service-guarded
# A DRY value is the recipe's own check (one round, no settling, a failed preflight recorded
# and not refused), with optionally a CORPUS, both positional:
#   just bench-service-guarded 1 [CORPUS]
# It still reads clocks (ab run, one round), but its figures are not evidence, and the
# judge refuses it as a DRY run.
# Exit: 0 VALID, 1 INVALID (identity, workload, preflight, structural or N1, said which),
# 4 REFUSED by the preflight, 2 any other refusal or a report error, 3 INCOMPLETE.

# The service default (svc2 at 10 threads) against the current default, at the pipelined default's grid
bench-service-guarded DRY="" CORPUS="":
    #!/usr/bin/env bash
    set -euo pipefail
    B=$PWD/target/service-guarded
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-service-guarded: $W already exists; not reusing an attempt's directory"; exit 2; }
    exec > >(tee -a "$W/console.log") 2>&1
    outcome="INCOMPLETE: stopped before the run finished (see console.log)"
    finish() {
      echo "$outcome" > "$W/outcome.txt"
      echo "bench-service-guarded: OUTCOME: $outcome"
      bash bench/bundle.sh "$W" || { echo "bench-service-guarded: bundling failed"; exit 2; }
      case "$outcome" in
        "VALID: "*) exit 0 ;;
        INVALID*) exit 1 ;;
        "REFUSED: the preflight"*) exit 4 ;;
        REFUSED*|"REPORT ERROR"*) exit 2 ;;
        *) exit 3 ;;
      esac
    }
    trap finish EXIT
    fail() { outcome="REFUSED: $*"; echo "bench-service-guarded: $*"; exit 2; }
    rounds=20; busy=(); dry="{{DRY}}"; corpus="{{CORPUS}}"
    if [ -n "$dry" ]; then
      rounds=1; busy=(--max-busy 64 --settle 0.2)
      echo "bench-service-guarded: DRY RUN: one round; its figures are not evidence" | tee "$W/DRY-RUN.txt"
    elif [ -n "$corpus" ]; then
      fail "a CORPUS is for a DRY check only; the registered run builds the bench's sample"
    fi
    git rev-parse HEAD > "$W/commit.txt"
    git status --porcelain --untracked-files=no > "$W/git-status.txt"
    # the commits the freeze names (bench/service-guarded.frozen): the candidate, whose build
    # and workload inputs HEAD must match, and the base (master as fetched at the freeze, by
    # SHA, merged into the candidate before it), an ancestor of it, with the time that master
    # was fetched (UTC): ancestry alone does not say the base was current
    frozen=bench/service-guarded.frozen
    cand=$(awk '$1 == "candidate" {print $2}' "$frozen")
    base=$(awk '$1 == "base" {print $2}' "$frozen")
    fetched=$(awk '$1 == "base_fetched" {print $2}' "$frozen")
    [[ $cand =~ ^[0-9a-f]{40}$ && $base =~ ^[0-9a-f]{40}$ && $fetched =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$ ]] \
      || fail "$frozen names no frozen candidate, base and base fetch time (the registration is a DRAFT); nothing to run"
    measured=(src Cargo.toml Cargo.lock .cargo build.rs rust-toolchain rust-toolchain.toml examples tests/common
      bench/service_guarded.py bench/svc_target_check.py bench/wikidata.json bench/wikidata.py bench/bundle.sh
      Justfile .just/service-guarded.just docs/journal/2026-11-09-service-guarded-registration.md)
    overrides=$(env | cut -d= -f1 | grep -E '^(CARGO_|RUST|AB_CENSUS_MS$|AB_SAMPLE_MS$)' | grep -vxE 'CARGO_HOME|RUSTUP_HOME|RUST_BACKTRACE|RUST_LOG|CARGO_(HTTP|NET|TERM)_[A-Z_]+' || true)
    [ -z "$overrides" ] || fail "unset these, so both builds and every arm run the same, registered way: $(echo $overrides)"
    [ ! -s "$W/git-status.txt" ] || fail "the checkout has uncommitted changes"
    [ -z "$(git status --porcelain --untracked-files=all -- "${measured[@]}")" ] || fail "untracked files among the build and workload inputs"
    git cat-file -e "$cand^{commit}" && git cat-file -e "$base^{commit}" || fail "fetch the branch: the frozen commits are missing"
    git merge-base --is-ancestor "$base" "$cand" || fail "the base $base is not an ancestor of the candidate $cand"
    if ! git diff --quiet "$cand" HEAD -- "${measured[@]}"; then
      echo "the measured inputs differ from the frozen candidate $cand" > "$W/NOT-FROZEN.txt"
      fail "HEAD's build or workload inputs differ from the frozen candidate $cand"
    fi
    printf 'head %s\ncandidate %s\nbase %s\nbase_fetched %s\n' "$(cat "$W/commit.txt")" "$cand" "$base" "$fetched" | tee "$W/commits.txt"
    [ -d "$B/build-$base" ] || git worktree add -q --detach "$B/build-$base" "$base"
    [ "$(git -C "$B/build-$base" rev-parse HEAD)" = "$base" ] || fail "$B/build-$base is not at the base $base"
    [ -z "$(git -C "$B/build-$base" status --porcelain --untracked-files=all)" ] || fail "$B/build-$base has changes or untracked files"
    built() {  # built DIR EXAMPLE...: build in DIR, print each example's executable, in order
      local dir=$1; shift
      local args=(); for e in "$@"; do args+=(--example "$e"); done
      (cd "$dir" && cargo build -q --release --message-format=json-render-diagnostics "${args[@]}") \
        | python3 -c 'import json,sys
    want=sys.argv[1:]; got={}
    for line in sys.stdin:
        m=json.loads(line)
        if m.get("reason")=="compiler-artifact" and m.get("executable") and m["target"]["name"] in want and "example" in m["target"]["kind"]:
            got[m["target"]["name"]]=m["executable"]
    missing=[w for w in want if w not in got]
    if missing: sys.exit("no executable reported for: "+" ".join(missing))
    print("\n".join(got[w] for w in want))' "$@"
    }
    configs() { local d=$1 stop=$2; while :; do for f in "$d/.cargo/config" "$d/.cargo/config.toml"; do [ -f "$f" ] && echo "$f"; done; [ "$d" = "$stop" ] && break; d=$(dirname "$d"); done; return 0; }
    only_base=$(configs "$(dirname "$B/build-$base")" "$PWD")
    [ -z "$only_base" ] || fail "Cargo configuration only the base build would read: $(echo $only_base)"
    tool() { (cd "$1" && rustc -vV && cargo -V); }
    # (a gated reading: each side's toolchain query must itself succeed before they're compared)
    tool_head=$(tool "$PWD") || fail "the head checkout's toolchain query failed"
    tool_base=$(tool "$B/build-$base") || fail "the base checkout's toolchain query failed"
    [ "$tool_head" = "$tool_base" ] || fail "the two checkouts select different toolchains"
    mkdir "$W/bin"
    base_built=$(built "$B/build-$base" ab) || fail "the base build failed"
    cp "$base_built" "$W/bin/ab-base"
    # (the noise control's twin: the base's executable, byte for byte, under another path)
    cp "$W/bin/ab-base" "$W/bin/ab-base-twin"
    sha256sum "$W/bin/ab-base" "$W/bin/ab-base-twin" > "$W/bin/ab-base.sha256"
    [ "$(awk '{print $1}' "$W/bin/ab-base.sha256" | sort -u | wc -l)" = 1 ] || fail "the twin is not the base's bytes"
    head_built=$(built "$PWD" ab windows svc_target) || fail "the head build failed"
    [ "$(printf '%s\n' "$head_built" | wc -l)" = 3 ] || fail "Cargo reported no executables for the head build"
    cp "$(printf '%s\n' "$head_built" | sed -n 1p)" "$W/bin/ab-head"
    cp "$(printf '%s\n' "$head_built" | sed -n 2p)" "$W/bin/windows"
    cp "$(printf '%s\n' "$head_built" | sed -n 3p)" "$W/bin/svc_target"
    sha256sum -c --quiet "$W/bin/ab-base.sha256" || fail "the base executable changed after it was copied"
    AB=$W/bin/ab-head; BASEAB=$W/bin/ab-base; TWINAB=$W/bin/ab-base-twin; WINDOWS=$W/bin/windows
    # diagnostics only, never judged (D064), and passive: `ab env --facts` starts no thread,
    # spins nothing and opens no perf counter; lscpu and `ab sample` only read; the raw
    # discovery reads are the cgroup membership, the mounts and the ancestor quota files, with
    # each read's absence or error, and no quota verdict (svc_target's record below is the
    # target evidence). Each is bounded, TERM at 30 s and KILL 5 s after; its exit and stderr
    # are archived (diagnostics/exits.txt, NAME.stderr). A failing diagnostic is recorded as
    # failed and nothing else: it never refuses the attempt, and never deletes, replaces or
    # selects a timed row
    mkdir "$W/diagnostics"
    diag() {  # diag FILE COMMAND...: one bounded diagnostic reading into diagnostics/FILE
      local file=$1 rc=0; shift
      timeout --kill-after=5 30 "$@" > "$W/diagnostics/$file" 2> "$W/diagnostics/$file.stderr" || rc=$?
      echo "$file: exit $rc" >> "$W/diagnostics/exits.txt"
      [ "$rc" = 0 ] || echo "bench-service-guarded: the diagnostic $file failed (exit $rc), archived; not judged"
    }
    diag env.json "$AB" env --facts
    diag lscpu.txt lscpu
    diag discovery.json python3 bench/service_guarded.py discovery
    diag sample-before.jsonl "$AB" sample --count 5
    # the actual-target preflight, before anything is measured, checked whole: a required
    # collector, gated on its successful completion. It passes only when svc_target exited 0
    # (a timeout's 124 or 137, or any other status, refuses even after a complete TARGET
    # record), the checker exited 0, and the checker's verdict is TARGET; anything else, an
    # unreadable check included, refuses (except in a DRY check). Its raw stdout
    # (preflight.json), stderr (preflight.stderr) and exit (preflight.exit) are archived either way
    pf=0
    timeout --kill-after=10 120 "$W/bin/svc_target" > "$W/preflight.json" 2> "$W/preflight.stderr" || pf=$?
    echo "$pf" > "$W/preflight.exit"
    cat "$W/preflight.json"
    cat "$W/preflight.stderr" >&2
    pc=0
    python3 bench/svc_target_check.py "$W/preflight.json" --exit "$pf" --out "$W/preflight-check.json" || pc=$?
    echo "$pc" > "$W/preflight-check.exit"
    verdict=$(python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))["verdict"])' "$W/preflight-check.json" 2>/dev/null || echo "UNCHECKED")
    if [ "$pf" != 0 ] || [ "$pc" != 0 ] || [ "$verdict" != TARGET ]; then
      [ "$verdict" != TARGET ] || verdict="TARGET-SHAPED BUT FAILED"
      if [ -n "$dry" ]; then
        echo "bench-service-guarded: DRY RUN: the preflight is $verdict (exit $pf), recorded; continuing"
      else
        outcome="REFUSED: the preflight is $verdict (exit $pf): see preflight.json and preflight-check.json"
        exit 4
      fi
    fi
    if [ -z "$corpus" ]; then
      dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
      if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
        git fetch -q origin bench-data:refs/remotes/origin/bench-data 2>/dev/null \
          && git archive origin/bench-data "$dir" | tar -x || true
      fi
      python3 bench/wikidata.py fetch bench/wikidata.json "$dir" >/dev/null
      python3 bench/wikidata.py build bench/wikidata.json "$dir" "$WINDOWS" "$B/wikidata.bz2"
      corpus=$B/wikidata.bz2
    fi
    ln -sf "$(realpath "$corpus")" "$W/wikidata.bz2"
    sha256sum "$(realpath "$corpus")" > "$W/corpus.sha256"
    printf '%s\n' mibz2 "mibz2@$BASEAB" "mibz2@$TWINAB" > "$W/arms.txt"
    {
      sha256sum "$AB" "$BASEAB" "$TWINAB" "$WINDOWS" "$W/bin/svc_target" "$(realpath "$corpus")"
      tool "$PWD"
      { printf '%s\n' "${CARGO_HOME:-$HOME/.cargo}/config" "${CARGO_HOME:-$HOME/.cargo}/config.toml"; configs "$(dirname "$PWD")" /; } \
        | while IFS= read -r f; do if [ -f "$f" ]; then sha256sum "$f"; fi; done
      echo "census every 5 ms (AB_CENSUS_MS unset); no --machine-state (no PMU telemetry)"
    } > "$W/inputs.txt"
    cat "$W/inputs.txt"
    "$AB" run "$W/runs.jsonl" --arms "mibz2,mibz2@$BASEAB,mibz2@$TWINAB" \
      --threads "8,9,10,11,12,16,20" --rounds "$rounds" "${busy[@]}" "$W/wikidata.bz2"
    diag sample-after.jsonl "$AB" sample --count 5
    set +e
    python3 bench/service_guarded.py report "$W" --out "$W/service-guarded"
    rc=$?
    set -e
    if [ "$rc" -le 1 ] && [ -s "$W/service-guarded.json" ] && [ -s "$W/service-guarded.md" ]; then
      outcome=$(python3 -c 'import json, sys
    v = json.load(open(sys.argv[1]))
    if v["valid"]:
        p = v["p1"]
        print("VALID: P1 " + p["reading"] + " (speed " + p["speed"] + ", CPU " + p["cpu"] + ")")
    else:
        print("INVALID (" + v["failure"] + "): " + "; ".join(v["problems"]))' "$W/service-guarded.json")
    else
      outcome="REPORT ERROR: service_guarded.py exited $rc without writing its report"
    fi
