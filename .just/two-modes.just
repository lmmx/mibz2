# The two modes' discriminator (docs/journal/2026-11-10-two-modes-discriminator.md, the
# registration; the design is §3 of docs/journal/2026-11-10-two-modes.md). DIAGNOSTIC ONLY: no
# merge gate reads it. NOT FROZEN until bench/two-modes.frozen names its commit (the recipe
# refuses every run but a DRY one until then).
#
# The harness is the one that produced the two modes, unchanged: the run-fed leader's second
# attempt's own `ab` and `windows`, taken from its archived bundle on the `evidence` branch and
# checked by digest (nothing of the decoder is built here). One arm, the default plan
# (`mibz2-default33`), 60 runs at 10 threads with --settle 0.25 and --sysfs-state, as that
# recipe ran them, through `ARM@EXE`: ab spawns bench/two_modes_probe.c's `child`, which reads
# the counters, starts the spin probes, sets the decoder's mask (the probe core, CPUs 9 and 19,
# taken out), execs ab's own child, samples its threads' placement every 2 ms and every CPU's
# clock every 10 ms, and reaps it with wait4. Pass 1: P-idle on CPU 9 alone, in two runs of
# every three (schedule PPN: the third has no probe, the same mask). Pass 2: P-idle beside P-sib
# on CPU 19, its sibling, in every run. ab itself runs on the decoder's mask (taskset), so
# nothing of ours shares the probe core. The preflight (read-only, bounded) refuses the
# attempt, archived, unless the registered host and its declared settings are established.
# One attempt; no retry to pass. About 5 minutes. Exit: 0 VALID (whatever the table says: the
# outcome says), 1 INVALID, 4 REFUSED by the preflight, 2 any other refusal or a report error,
# 3 stopped before its verdict.
#   just submit bench-two-modes
# A DRY value is the recipe's own check (three runs a pass, the machine's own last core as
# the probe core, a failed preflight recorded and not refused), with optionally a CORPUS:
#   just bench-two-modes 1 [CORPUS]
# It still reads clocks, but its figures are not evidence: its label is DRY.

# The two modes' discriminator: spin probes and placement around 2 x 60 default-plan runs at 10 threads
bench-two-modes DRY="" CORPUS="":
    #!/usr/bin/env bash
    set -euo pipefail
    # each attempt in a directory of its own, never overwritten
    B=$PWD/target/two-modes
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-two-modes: $W already exists; not reusing an attempt's directory"; exit 2; }
    exec > >(tee -a "$W/console.log") 2>&1
    # every attempt ends with one stated outcome (outcome.txt) and is packed, however it ends
    outcome="INCOMPLETE: stopped before the run finished (see console.log)"
    finish() {
      echo "$outcome" > "$W/outcome.txt"
      echo "bench-two-modes: OUTCOME: $outcome"
      bash bench/bundle.sh "$W" || { echo "bench-two-modes: bundling failed"; exit 2; }
      case "$outcome" in
        "VALID: "*|"DRY (not evidence): "*) exit 0 ;;
        INVALID*) exit 1 ;;
        "REFUSED: the preflight"*) exit 4 ;;
        REFUSED*|"REPORT ERROR"*) exit 2 ;;
        *) exit 3 ;;
      esac
    }
    trap finish EXIT
    fail() { outcome="REFUSED: $*"; echo "bench-two-modes: $*"; exit 2; }
    rounds=60; dry="{{DRY}}"; corpus="{{CORPUS}}"; dryarg=()
    if [ -n "$dry" ]; then
      rounds=3; dryarg=(--dry)
      echo "bench-two-modes: DRY RUN: three runs a pass; not evidence" | tee "$W/DRY-RUN.txt"
    elif [ -n "$corpus" ]; then
      fail "a CORPUS is for a DRY check only; the registered run builds the bench's sample"
    fi
    frozen=$(awk '$1 == "frozen" {print $2}' bench/two-modes.frozen)
    if ! [[ $frozen =~ ^[0-9a-f]{40}$ ]]; then
      [ -n "$dry" ] || fail "bench/two-modes.frozen names no frozen commit yet; nothing to run"
      echo "bench-two-modes: NOT FROZEN (a DRY run only)"
      frozen=""
    fi
    # what the probe, the workload and the judgement are made from (a path that doesn't exist
    # counts too). The decoder's sources are not among them: its executable is the archived one
    measured=(bench/two_modes_probe.c bench/two_modes.py bench/wikidata.json bench/wikidata.py
      bench/bundle.sh Justfile .just/two-modes.just docs/journal/2026-11-10-two-modes-discriminator.md)
    # no environment that changes how ab observes, or that would reach the probe
    overrides=$(env | cut -d= -f1 | grep -E '^(AB_|TWO_MODES_)' || true)
    [ -z "$overrides" ] || fail "unset these, so the harness and the probe run the registered way: $(echo $overrides)"
    [ -z "$(git status --porcelain --untracked-files=no)" ] || fail "the checkout has uncommitted changes"
    [ -z "$(git status --porcelain --untracked-files=all -- "${measured[@]}")" ] || fail "untracked files among the inputs"
    match=false
    if [ -n "$frozen" ]; then
      git cat-file -e "$frozen^{commit}" || fail "fetch the branch: the frozen commit $frozen is missing"
      git diff --quiet "$frozen" HEAD -- "${measured[@]}" || fail "HEAD's inputs differ from the frozen commit $frozen"
      match=true
    fi
    head=$(git rev-parse HEAD)
    printf 'head %s\nfrozen %s\n' "$head" "${frozen:-none}" | tee "$W/commits.txt"
    reg() { python3 -c 'import sys; sys.path.insert(0, "bench"); import two_modes as t; print(t.SOURCE[sys.argv[1]])' "$1"; }
    # the harness that produced the two modes: its archived bundle, by commit, path and digest
    ev=$(reg evidence_commit); src=$(reg path)
    git cat-file -e "$ev^{commit}" 2>/dev/null || git fetch -q origin evidence || fail "the evidence branch could not be fetched"
    git cat-file -e "$ev:$src" || fail "the archived harness $src is not at $ev"
    S=$B/source-$(reg sha256 | cut -c1-12)
    if [ ! -f "$S/bin/ab" ]; then
      rm -rf "$S.tmp"; mkdir -p "$S.tmp"
      git show "$ev:$src" > "$S.tmp/bundle.tar.gz"
      [ "$(sha256sum "$S.tmp/bundle.tar.gz" | cut -d' ' -f1)" = "$(reg sha256)" ] || fail "the archived bundle's digest is not the registered one"
      top=$(basename "$src" .tar.gz)
      tar -xzf "$S.tmp/bundle.tar.gz" -C "$S.tmp" "$top/bin/ab" "$top/bin/windows"
      mkdir -p "$S.tmp/bin"; mv "$S.tmp/$top/bin/ab" "$S.tmp/$top/bin/windows" "$S.tmp/bin/"
      rm -rf "$S.tmp/$top" "$S.tmp/bundle.tar.gz"; mv "$S.tmp" "$S"
    fi
    mkdir "$W/bin"
    cp "$S/bin/ab" "$S/bin/windows" "$W/bin/"
    sha() { sha256sum "$1" | cut -d' ' -f1; }
    [ "$(sha "$W/bin/ab")" = "$(reg ab_sha256)" ] || fail "the archived ab is not the registered executable"
    [ "$(sha "$W/bin/windows")" = "$(reg windows_sha256)" ] || fail "the archived windows is not the registered executable"
    # the probe: a standalone program, compiled here (it never links the decoder)
    cc -O2 -Wall -Wextra -pthread -o "$W/bin/two_modes_probe" bench/two_modes_probe.c || fail "the probe did not compile"
    AB=$W/bin/ab; PROBE=$W/bin/two_modes_probe
    if [ -z "$corpus" ]; then
      dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
      if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
        git fetch -q origin bench-data:refs/remotes/origin/bench-data 2>/dev/null \
          && git archive origin/bench-data "$dir" | tar -x || true
      fi
      python3 bench/wikidata.py fetch bench/wikidata.json "$dir" >/dev/null
      python3 bench/wikidata.py build bench/wikidata.json "$dir" "$W/bin/windows" "$B/wikidata.bz2"
      corpus=$B/wikidata.bz2
      want=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["corpus_sha256"])')
      [ "$(sha "$corpus")" = "$want" ] || fail "the corpus built is not bench/wikidata.json's $want"
    fi
    ln -sf "$(realpath "$corpus")" "$W/wikidata.bz2"
    ab0=$(sha "$AB"); pr0=$(sha "$PROBE"); cor0=$(sha "$W/wikidata.bz2")
    {
      sha256sum "$AB" "$W/bin/windows" "$PROBE" "$(realpath "$corpus")"
      echo "harness: $src at $ev"
      cc --version | head -1
      uname -a
    } > "$W/inputs.txt"
    cat "$W/inputs.txt"
    # the preflight: read-only, passive and bounded (each collector by its own timeout with a
    # forced kill after it), after the corpus and before anything is timed; the probe's
    # selftest spins one CPU for 0.2 s and is plumbing, never a reading. PASS, or the attempt
    # is REFUSED here and archived. A DRY check records it and goes on
    mkdir "$W/preflight"
    P=$W/preflight
    { timeout --kill-after=5 30 "$AB" env --facts > "$P/env.json" 2> "$P/env.err"; echo "ab env --facts: exit $?"
      timeout --kill-after=5 30 lscpu > "$P/lscpu.txt" 2> "$P/lscpu.err"; echo "lscpu: exit $?"
      timeout --kill-after=5 30 "$AB" sample --count 3 --interval 1 > "$P/sample.jsonl" 2> "$P/sample.err"; echo "ab sample: exit $?"
      timeout --kill-after=5 30 python3 bench/two_modes.py static --out "$P/static.json" 2> "$P/static.err"; echo "static facts: exit $?"
    } > "$P/exits.txt" || true
    lay=$(python3 bench/two_modes.py layout "$P/env.json" "${dryarg[@]}") || fail "the layout could not be read from ab env --facts"
    read -r idle sib dec <<< "$lay"
    [ "$sib" = "-" ] && sib=""
    echo "bench-two-modes: P-idle on CPU $idle, P-sib on CPU ${sib:-none}, the decoder on $dec"
    { timeout --kill-after=5 30 taskset -c "$dec" "$PROBE" selftest "$idle" 200 > "$P/selftest.json" 2> "$P/selftest.err"; echo "probe selftest: exit $?"
    } >> "$P/exits.txt" || true
    set +e
    python3 bench/two_modes.py preflight "$W" > "$W/preflight.txt"
    pf=$?
    set -e
    cat "$P/exits.txt" "$W/preflight.txt"
    for e in "$P"/*.err; do [ -s "$e" ] && { echo "--- $e"; cat "$e"; } || true; done
    if [ "$pf" -ne 0 ]; then
      if [ -n "$dry" ]; then
        echo "bench-two-modes: DRY RUN: the preflight did not pass (exit $pf), recorded; continuing"
      else
        outcome="REFUSED: the preflight, before any run: $(head -1 "$W/preflight.txt" 2>/dev/null || echo "its judgement wrote nothing (exit $pf)")"
        exit 4
      fi
    fi
    # the two passes: ab exactly as the run-fed leader's recipe ran it, one arm, through the probe
    passes=(1)
    [ -n "$sib" ] && passes+=(2) || echo "bench-two-modes: no sibling for P-sib on this machine; pass 2 not run (DRY only: the preflight refuses this host otherwise)"
    set +e
    for pass in "${passes[@]}"; do
      mkdir "$W/pass$pass"
      sched=$(python3 -c 'import sys; sys.path.insert(0, "bench"); import two_modes as t; print(t.SCHEDULE[int(sys.argv[1])])' "$pass")
      echo "bench-two-modes: pass $pass: $rounds runs, P-idle on $idle$( [ "$pass" = 2 ] && echo ", P-sib on $sib"), schedule $sched"
      TWO_MODES_AB=$AB TWO_MODES_OUT=$W/pass$pass/probe.jsonl TWO_MODES_PASS=$pass TWO_MODES_SCHEDULE=$sched \
        TWO_MODES_IDLE_CPU=$idle TWO_MODES_SIB_CPU=$( [ "$pass" = 2 ] && echo "$sib") TWO_MODES_DECODER_CPUS=$dec \
        taskset -c "$dec" "$AB" run "$W/pass$pass/runs.jsonl" --arms "mibz2-default33@$PROBE" --threads 10 \
        --rounds "$rounds" --settle 0.25 --sysfs-state "$W/wikidata.bz2"
      echo "pass $pass ab run: exit $?" | tee -a "$W/exits.txt"
    done
    # (diagnostics only, never judged: three sysfs samples after the passes)
    timeout --kill-after=5 30 "$AB" sample --count 3 --interval 1 > "$W/sample-after.jsonl"
    echo "ab sample after: exit $?" | tee -a "$W/exits.txt"
    set -e
    at_frozen() { [ -n "$frozen" ] && git show "$frozen:$1" | sha256sum | cut -d' ' -f1 || true; }
    python3 - "$W/identity.json" <<PY
    import hashlib, json, sys
    src = {}
    for p in ["bench/two_modes_probe.c", "bench/two_modes.py", ".just/two-modes.just",
              "docs/journal/2026-11-10-two-modes-discriminator.md"]:
        src[p] = hashlib.sha256(open(p, "rb").read()).hexdigest()
    json.dump({
        "schema": "two_modes/identity/1",
        "dry": bool("$dry"),
        "head": "$head",
        "frozen": "$frozen" or None,
        "inputs_match_frozen": "$match" == "true",
        "sources": src,
        "recipe_at_frozen": "$(at_frozen .just/two-modes.just)" or None,
        "harness": {"evidence_commit": "$ev", "path": "$src"},
        "ab_sha256": "$ab0", "ab_sha256_after": "$(sha "$AB")",
        "windows_sha256": "$(sha "$W/bin/windows")",
        "probe_sha256": "$pr0", "probe_sha256_after": "$(sha "$PROBE")",
        "corpus_sha256": "$cor0", "corpus_sha256_after": "$(sha "$W/wikidata.bz2")",
        "arm": "mibz2-default33", "threads": 10, "rounds": $rounds, "settle": 0.25, "sysfs_state": True,
        "layout": {"idle": $idle, "sib": ${sib:-None}, "decoder": "$dec"},
        "passes": [$(IFS=,; echo "${passes[*]}")],
        "preflight_exit": $pf,
    }, open(sys.argv[1], "w"), indent=1)
    PY
    [ -z "$(git status --porcelain --untracked-files=no)" ] || fail "the checkout changed during the run"
    # the table (§3), fixed before data; its first line is the outcome
    set +e
    python3 bench/two_modes.py judge "$W" "${dryarg[@]}" > "$W/judgement.txt"
    rc=$?
    set -e
    cat "$W/judgement.txt"
    if [ "$rc" -le 2 ] && [ -s "$W/judgement.json" ]; then
      outcome="$(head -1 "$W/judgement.txt")"
    else
      outcome="REPORT ERROR: two_modes.py exited $rc without writing its judgement"
    fi
