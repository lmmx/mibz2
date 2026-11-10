# The two modes' discriminator (docs/journal/2026-11-10-two-modes-discriminator.md, the
# registration; the design is §3 of docs/journal/2026-11-10-two-modes.md, revised after the
# Researcher's REVISE, #145 6757560564). DIAGNOSTIC ONLY: no merge gate reads it, and its
# outcomes are compatibility with registered signatures, never an identified mechanism. NOT
# FROZEN until bench/two-modes.frozen names its commit (the recipe refuses every run but a DRY
# one until then).
#
# The harness is the one that produced the two modes, unchanged: the run-fed leader's second
# attempt's own `ab` and `windows`, taken from its archived bundle on the `evidence` branch and
# checked by digest (nothing of the decoder is built here). In order:
# - the preflight (read-only, each collector bounded), which refuses the attempt, archived,
#   unless the registered host and its declared settings are established;
# - the gap block, the primary cheap discriminator: no probe, all CPUs, ab's own child, the
#   default plan at 10 threads, 30 runs after a 0.25 s idle gap and 30 spawned at once, in a
#   seeded order; deadline 600 s + 10 s grace;
# - the sustained block: one process (ab's own child, unpinned, no probe) decodes the corpus
#   repeated 50 times (built and checked by content, outside the bundle), its input consumption
#   read every 10 ms; deadline 300 s + 10 s grace; then a fixed 60 s rest;
# - pass 1: 60 runs of `mibz2-default33` at 10 threads with --settle 0.25 and --sysfs-state,
#   through `ARM@EXE` into bench/two_modes_probe.c's `child`, which reads the counters, starts
#   the spin probes, sets the decoder's mask (the probe core, CPUs 9 and 19, taken out), execs
#   ab's own child, samples its threads' placement every 2 ms and every CPU's clock every
#   10 ms, and reaps it with wait4; P-idle on CPU 9 in 40 runs, none in 20, in the order the
#   registered seed drew; deadline 900 s + 10 s grace, the archived ab's busy-gate waits
#   included;
# - pass 2: the same, P-idle beside P-sib on CPU 19 in every run; the same deadline.
# ab itself runs on the decoder's mask (taskset), so nothing of ours shares the probe core.
# Every stage that can wait runs under bench/bounded.py (its process group TERMed, then
# KILLed, its descendants reaped), setup and bundling included; nothing is ever retried. A
# timeout in setup (fetch, corpus, extraction, compilation, concatenation) REFUSES the attempt
# before anything is measured; one in a measured stage (the gap block, the sustained block, a
# pass) is recorded and makes the attempt INVALID; one in the judge is a REPORT ERROR; a group
# that cannot be emptied (125) stops the attempt there. Every outcome keeps its partial records
# and is bundled. One attempt; no retry to pass. About 8 minutes; its operational ceiling is
# OPERATIONAL_CEILING_S in bench/two_modes.py, 8460 s (every supervised call at deadline,
# grace and both cleanup phases, the bounded collectors and the rest), plus seconds of
# unsupervised bookkeeping. Exit: 0
# VALID (whatever the table says: the outcome says), 1 INVALID, 4 REFUSED by the preflight, 2
# any other refusal or a report error, 3 stopped before its verdict.
#   just submit bench-two-modes
# A DRY value is the recipe's own check (three runs a pass, three copies in the sustained
# block, a 1 s rest, the machine's own last core as the probe core, a failed preflight
# recorded and not refused), with optionally a CORPUS:
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
      python3 bench/bounded.py $(python3 bench/two_modes.py get DEADLINES | python3 -c 'import json, sys; print(*json.load(sys.stdin)["bundle"])') \
        bash bench/bundle.sh "$W" || { echo "bench-two-modes: bundling failed or timed out; $W is kept"; exit 2; }
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
    measured=(bench/two_modes_probe.c bench/two_modes.py bench/bounded.py bench/wikidata.json
      bench/wikidata.py bench/bundle.sh Justfile .just/two-modes.just
      docs/journal/2026-11-10-two-modes-discriminator.md)
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
    tm() { python3 bench/two_modes.py get "$1"; }
    # bd STAGE COMMAND...: the command under its stage's registered deadline and grace, in a
    # process group of its own (bench/bounded.py); 125, a group that could not be emptied,
    # stops the attempt here
    bd() {
      local dl; dl=$(tm DEADLINES | python3 -c 'import json, sys; print(*json.load(sys.stdin)[sys.argv[1]])' "$1"); shift
      local rc=0
      python3 bench/bounded.py $dl "$@" || rc=$?
      [ "$rc" != 125 ] || fail "a bounded stage's processes could not be stopped (bench/bounded.py exit 125)"
      return "$rc"
    }
    # the harness that produced the two modes: its archived bundle, by commit, path and digest
    ev=$(reg evidence_commit); src=$(reg path)
    git cat-file -e "$ev^{commit}" 2>/dev/null || bd fetch git fetch -q origin evidence || fail "the evidence branch could not be fetched"
    git cat-file -e "$ev:$src" || fail "the archived harness $src is not at $ev"
    S=$B/source-$(reg sha256 | cut -c1-12)
    if [ ! -f "$S/bin/ab" ]; then
      rm -rf "$S.tmp"; mkdir -p "$S.tmp"
      bd setup bash -c 'git show "$1" > "$2"' _ "$ev:$src" "$S.tmp/bundle.tar.gz" || fail "the archived bundle could not be read"
      [ "$(sha256sum "$S.tmp/bundle.tar.gz" | cut -d' ' -f1)" = "$(reg sha256)" ] || fail "the archived bundle's digest is not the registered one"
      top=$(basename "$src" .tar.gz)
      bd setup tar -xzf "$S.tmp/bundle.tar.gz" -C "$S.tmp" "$top/bin/ab" "$top/bin/windows" || fail "the archived executables could not be extracted"
      mkdir -p "$S.tmp/bin"; mv "$S.tmp/$top/bin/ab" "$S.tmp/$top/bin/windows" "$S.tmp/bin/"
      rm -rf "$S.tmp/$top" "$S.tmp/bundle.tar.gz"; mv "$S.tmp" "$S"
    fi
    mkdir "$W/bin"
    cp "$S/bin/ab" "$S/bin/windows" "$W/bin/"
    sha() { sha256sum "$1" | cut -d' ' -f1; }
    [ "$(sha "$W/bin/ab")" = "$(reg ab_sha256)" ] || fail "the archived ab is not the registered executable"
    [ "$(sha "$W/bin/windows")" = "$(reg windows_sha256)" ] || fail "the archived windows is not the registered executable"
    # the probe: a standalone program, compiled here (it never links the decoder)
    bd setup cc -O2 -Wall -Wextra -pthread -o "$W/bin/two_modes_probe" bench/two_modes_probe.c || fail "the probe did not compile"
    AB=$W/bin/ab; PROBE=$W/bin/two_modes_probe
    if [ -z "$corpus" ]; then
      dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
      if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
        bd fetch git fetch -q origin bench-data:refs/remotes/origin/bench-data 2>/dev/null \
          && git archive origin/bench-data "$dir" | tar -x || true
      fi
      bd corpus python3 bench/wikidata.py fetch bench/wikidata.json "$dir" >/dev/null || fail "the corpus's windows could not be fetched"
      bd corpus python3 bench/wikidata.py build bench/wikidata.json "$dir" "$W/bin/windows" "$B/wikidata.bz2" || fail "the corpus could not be built"
      corpus=$B/wikidata.bz2
      want=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["corpus_sha256"])')
      [ "$(sha "$corpus")" = "$want" ] || fail "the corpus built is not bench/wikidata.json's $want"
    fi
    ln -sf "$(realpath "$corpus")" "$W/wikidata.bz2"
    ab0=$(sha "$AB"); win0=$(sha "$W/bin/windows"); pr0=$(sha "$PROBE"); cor0=$(sha "$W/wikidata.bz2")
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
    # the gap block, FIRST and the primary cheap discriminator: no probe, all CPUs, ab's own
    # child unchanged, one fresh process a run; GAP_EACH runs after a GAP_S idle gap and
    # GAP_EACH spawned as soon as the run before exits, in the registered seeded order
    mkdir "$W/gaps"
    # (after one conditioning child, retained and never counted, so that the first G slot has
    # a run before it to wait after)
    gruns=$(( 2 * $(tm GAP_EACH) )); [ -n "$dry" ] && gruns=$(tm DRY_GAP_RUNS)
    echo "bench-two-modes: the gap block: $gruns runs, order $(tm GAP_ORDER | cut -c1-"$gruns")"
    set +e
    bd gaps python3 bench/two_modes.py gaps "$AB" "$W/wikidata.bz2" "$W/gaps/runs.jsonl" --runs "$gruns"
    gaps_rc=$?
    set -e
    echo "gaps: exit $gaps_rc" | tee -a "$W/exits.txt"
    # the sustained block, then, with no probe: one process (ab's own child, unpinned)
    # decodes the corpus repeated; its input is built or validated by content (the digest of
    # the corpus streamed COPIES times against the file's own), recorded, and digested again
    # after the block. Then a fixed rest before pass 1
    copies=$(tm SUSTAINED_COPIES); [ -n "$dry" ] && copies=$(tm DRY_SUSTAINED_COPIES)
    X=$B/wikidata-x$copies.bz2
    mkdir "$W/sustained"
    bd concat python3 bench/two_modes.py concat "$W/wikidata.bz2" "$copies" "$X" --record "$W/sustained/concat.json" >/dev/null \
      || fail "the sustained block's input is not the corpus repeated $copies times"
    echo "bench-two-modes: the sustained block: $copies corpora in one process, no probe"
    set +e
    TWO_MODES_AB=$AB TWO_MODES_OUT=$W/sustained/sustained.json \
      bd sustained "$PROBE" sustained "$X" > "$W/sustained/child.json"
    sus_rc=$?
    set -e
    echo "sustained: exit $sus_rc" | tee -a "$W/exits.txt"
    sus_after=$(bd concat python3 bench/two_modes.py digest "$X") || sus_after=unread
    rest=$(tm SUSTAINED_REST_S); [ -n "$dry" ] && rest=1
    echo "bench-two-modes: a fixed rest of ${rest} s before pass 1"; sleep "$rest"
    # the two passes: ab exactly as the run-fed leader's recipe ran it, one arm, through the
    # probe, each under its deadline; after a pass that does not complete, none follows
    passes=(1)
    [ -n "$sib" ] && passes+=(2) || echo "bench-two-modes: no sibling for P-sib on this machine; pass 2 not run (DRY only: the preflight refuses this host otherwise)"
    declare -A pass_rc
    ran=()
    for pass in "${passes[@]}"; do
      mkdir "$W/pass$pass"
      sched=$(tm SCHEDULE | python3 -c 'import json, sys; print(json.load(sys.stdin)[sys.argv[1]])' "$pass")
      echo "bench-two-modes: pass $pass: $rounds runs, P-idle on $idle$( [ "$pass" = 2 ] && echo ", P-sib on $sib"), schedule $sched"
      set +e
      TWO_MODES_AB=$AB TWO_MODES_OUT=$W/pass$pass/probe.jsonl TWO_MODES_PASS=$pass TWO_MODES_SCHEDULE=$sched \
        TWO_MODES_IDLE_CPU=$idle TWO_MODES_SIB_CPU=$( [ "$pass" = 2 ] && echo "$sib") TWO_MODES_DECODER_CPUS=$dec \
        bd pass taskset -c "$dec" "$AB" run "$W/pass$pass/runs.jsonl" --arms "mibz2-default33@$PROBE" --threads 10 \
        --rounds "$rounds" --settle 0.25 --sysfs-state "$W/wikidata.bz2"
      pass_rc[$pass]=$?
      set -e
      ran+=("$pass")
      echo "pass $pass ab run: exit ${pass_rc[$pass]}" | tee -a "$W/exits.txt"
      [ "${pass_rc[$pass]}" = 0 ] || { echo "bench-two-modes: pass $pass did not complete (exit ${pass_rc[$pass]}): recorded, not retried; no further pass"; break; }
    done
    pass_exits=$(for p in "${ran[@]}"; do printf '"%s": %s, ' "$p" "${pass_rc[$p]}"; done)
    # (diagnostics only, never judged: three sysfs samples after the passes)
    set +e
    timeout --kill-after=5 30 "$AB" sample --count 3 --interval 1 > "$W/sample-after.jsonl"
    echo "ab sample after: exit $?" | tee -a "$W/exits.txt"
    set -e
    clean_after=true; [ -z "$(git status --porcelain --untracked-files=no)" ] || clean_after=false
    at_frozen() { [ -n "$frozen" ] && git cat-file -e "$frozen:$1" 2>/dev/null && git show "$frozen:$1" | sha256sum | cut -d' ' -f1 || true; }
    srcs=$(for p in $(tm SOURCES | tr -d "[]',"); do printf '"%s": {"head": "%s", "frozen": %s}, ' "$p" "$(sha "$p")" "$(f=$(at_frozen "$p"); [ -n "$f" ] && echo "\"$f\"" || echo None)"; done)
    python3 - "$W/identity.json" <<PY
    import json, sys
    sys.path.insert(0, "bench")
    import two_modes as t
    json.dump({
        "schema": t.IDENTITY_SCHEMA,
        "dry": bool("$dry"),
        "head": "$head",
        "frozen": "$frozen" or None,
        "inputs_match_frozen": "$match" == "true",
        "tree_clean_before": True,
        "tree_clean_after": "$clean_after" == "true",
        "sources": {$srcs},
        "harness": {"evidence_commit": "$ev", "path": "$src", "bundle_sha256": "$(reg sha256)"},
        "ab_sha256": "$ab0", "ab_sha256_after": "$(sha "$AB")",
        "windows_sha256": "$win0", "windows_sha256_after": "$(sha "$W/bin/windows")",
        "probe_sha256": "$pr0", "probe_sha256_after": "$(sha "$PROBE")",
        "corpus_sha256": "$cor0", "corpus_sha256_after": "$(sha "$W/wikidata.bz2")",
        "arm": "mibz2-default33", "threads": 10, "rounds": $rounds, "settle": 0.25, "sysfs_state": True,
        "layout": {"idle": $idle, "sib": ${sib:-None}, "decoder": "$dec"},
        "schedule": {str(k): v for k, v in t.SCHEDULE.items()}, "schedule_seed": t.SCHEDULE_SEED,
        "deadlines": t.DEADLINES,
        "passes": [$(IFS=,; echo "${passes[*]}")],
        "pass_exits": {$pass_exits},
        "preflight_exit": $pf,
        "gaps": {"exit": $gaps_rc, "runs": $gruns, "order": t.GAP_ORDER, "seed": t.GAP_SEED, "gap_s": t.GAP_S},
        "sustained": {"copies": $copies, "exit": $sus_rc, "rest_s": $rest,
                      "concat": json.load(open("$W/sustained/concat.json")), "sha256_after": "$sus_after"},
    }, open(sys.argv[1], "w"), indent=1)
    PY
    # the table (§3), fixed before data; its first line is the outcome
    set +e
    bd judge python3 bench/two_modes.py judge "$W" "${dryarg[@]}" > "$W/judgement.txt"
    rc=$?
    set -e
    cat "$W/judgement.txt"
    if [ "$rc" -le 2 ] && [ -s "$W/judgement.json" ]; then
      outcome="$(head -1 "$W/judgement.txt")"
    else
      outcome="REPORT ERROR: two_modes.py exited $rc without writing its judgement"
    fi
