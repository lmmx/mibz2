# Share-nothing pricing (docs/journal/2026-11-10-share-nothing-pricing.md, the registration;
# the idea is docs/journal/2026-11-10-share-nothing.md §4, falsifier 2). DIAGNOSTIC ONLY (RULES
# C1): no merge gate reads it. NOT FROZEN until bench/share-nothing.frozen names its commit (the
# recipe refuses every run but a DRY one until then).
#
# Plain independent workers (ab's `mibz2-workers`, today's decoder, built here) on the bench's
# Wikidata recompressed with lbzip2 -2 (0.8 MB of table a block) and -9 (3.6 MB, the in-run
# comparator), at 2, 4, 6, 8 and 10 workers, read as CPU-seconds and wall. In order:
# - the build of ab at HEAD, and the bench's corpus (built from its windows, checked by digest);
# - the corpora: the corpus decoded by bzip2 (libbzip2) to the registered content digest,
#   recompressed with lbzip2 -2 and -9 at -n 4 (and at -n 1, recorded: does the thread count
#   change the output), each decoded by bzip2 back to the original's digest, digests recorded;
# - the preflight (read-only, each collector bounded by timeout), which refuses the attempt,
#   archived, unless the i9-10900K and its declared settings (intel_pstate, powersave,
#   balance_performance, turbo) are established, gated on every collector's exit (exits.txt);
# - the drive: 1 warm-up round and 60 measured rounds, each the sentinel (the default plan at
#   10 threads on the bench's corpus, the condition the two modes were recorded in), the ten
#   cells in an order drawn for the round from the registered seed, and the sentinel again: one
#   `ab run` a slot (--settle 0.25, --max-busy 1.5, --sysfs-state), each under its own
#   deadline; the first failure stops it, recorded, never retried;
# - the judge (bench/share_nothing.py judge, written with tests and fixed before data).
# Every step that can wait runs under bench/bounded.py (its process group TERMed, then KILLed,
# and checked empty); a step past its deadline, or whose group can't be emptied, makes the
# attempt INVALID. The finalisation (bench/par_scan_finish.sh) writes the outcome, then bundles
# under a deadline; a bundle that fails or is late leaves the outcome INVALID and no .tar.gz to
# submit (exit 2). One attempt; no retry to pass. About 12 minutes once built. Exit: 0 VALID
# (whatever the readings say) or DRY, 1 INVALID or REFUSED, 2 a failed finalisation, 3 stopped
# before its verdict, 4 REFUSED by the preflight.
#   just submit bench-share-nothing
# A DRY value is the recipe's own check, untimed and never evidence: one warm-up and two
# measured rounds, a failed preflight recorded and not refused; a MAXBUSY (DRY only) relaxes
# ab's idle gate on a busy shared machine:
#   just bench-share-nothing 1 [MAXBUSY]

# Plain workers on lbzip2 -2 and -9 Wikidata at 2-10 workers: the CPU a decode costs, 4 to 10
bench-share-nothing DRY="" MAXBUSY="":
    #!/usr/bin/env bash
    set -euo pipefail
    # each attempt in a directory of its own, never overwritten
    B=$PWD/target/share-nothing
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-share-nothing: $W already exists; not reusing an attempt's directory"; exit 1; }
    exec > >(tee -a "$W/console.log") 2>&1
    outcome="INCOMPLETE: stopped before the run finished (see console.log)"
    C=""
    # finalisation: the outcome written, then the bundle under its deadline; a failed or late
    # bundle rewrites the outcome INVALID and leaves no .tar.gz for `just submit` (exit 2)
    finish() {
      echo "bench-share-nothing: OUTCOME: $outcome"
      rm -rf "$W/work" "${C:-}"
      local fst=0
      bash bench/par_scan_finish.sh "$W" "$outcome" 600 bash bench/bundle.sh "$W" || fst=$?
      outcome=$(cat "$W/outcome.txt")
      [ "$fst" -eq 0 ] || { echo "bench-share-nothing: OUTCOME: $outcome"; exit 2; }
      case "$outcome" in
        VALID*|DRY*) exit 0 ;;
        "REFUSED: the preflight"*) exit 4 ;;
        INVALID*|REFUSED*) exit 1 ;;
        *) exit 3 ;;
      esac
    }
    trap finish EXIT
    fail() { outcome="REFUSED: $*"; echo "bench-share-nothing: $*"; exit 1; }
    sn() { python3 bench/share_nothing.py "$@"; }
    # bd STAGE COMMAND...: the command under its stage's registered deadline and grace, in a
    # process group of its own (bench/bounded.py)
    bd() {
      local dl; dl=$(sn get DEADLINES | python3 -c 'import json, sys; print(*json.load(sys.stdin)[sys.argv[1]])' "$1"); shift
      python3 bench/bounded.py $dl "$@"
    }
    deadline() { [ "$1" -eq 124 ] || [ "$1" -eq 125 ]; }
    late() { outcome="INVALID: $1 passed its deadline or could not be stopped (exit $2)"; echo "bench-share-nothing: $outcome"; exit 1; }
    dry="{{DRY}}"; maxbusy="{{MAXBUSY}}"; dryarg=(); busyarg=()
    if [ -n "$dry" ]; then
      dryarg=(--dry)
      echo "bench-share-nothing: DRY RUN: one warm-up and two measured rounds, untimed; not evidence" | tee "$W/DRY-RUN.txt"
      [ -z "$maxbusy" ] || { busyarg=(--max-busy "$maxbusy"); echo "bench-share-nothing: DRY RUN: ab's idle gate relaxed to --max-busy $maxbusy" | tee -a "$W/DRY-RUN.txt"; }
    elif [ -n "$maxbusy" ]; then
      fail "a MAXBUSY is for a DRY check only; the registered run keeps ab's gate at 1.5"
    fi
    for tool in python3 git cargo lbzip2 bzip2 lscpu; do
      command -v "$tool" > /dev/null || fail "needs $tool on PATH"
    done
    # no environment that changes how ab observes or runs
    overrides=$(env | cut -d= -f1 | grep -E '^(AB_|SHARE_NOTHING_|MIBZ2_)' || true)
    [ -z "$overrides" ] || fail "unset these, so the harness runs the registered way: $(echo $overrides)"
    frozen=$(awk '$1 == "frozen" {print $2}' bench/share-nothing.frozen)
    if ! [[ $frozen =~ ^[0-9a-f]{40}$ ]]; then
      [ -n "$dry" ] || fail "bench/share-nothing.frozen names no frozen commit yet; nothing to run"
      echo "bench-share-nothing: NOT FROZEN (a DRY run only)"
      frozen=""
    fi
    # what is built, driven and judged (a path that doesn't exist counts too)
    measured=(src Cargo.toml Cargo.lock .cargo build.rs rust-toolchain rust-toolchain.toml
      examples/ab.rs examples/windows.rs tests/common bench/bundle.sh bench/wikidata.json
      bench/wikidata.py bench/share_nothing.py bench/bounded.py bench/par_scan_finish.sh
      Justfile .just/share-nothing.just docs/journal/2026-11-10-share-nothing-pricing.md)
    [ -z "$(git status --porcelain --untracked-files=no)" ] || fail "the checkout has uncommitted changes"
    [ -z "$(git status --porcelain --untracked-files=all -- "${measured[@]}")" ] || fail "untracked files among the measured inputs"
    match=false
    if [ -n "$frozen" ]; then
      git cat-file -e "$frozen^{commit}" || fail "fetch the branch: the frozen commit $frozen is missing"
      git diff --quiet "$frozen" HEAD -- "${measured[@]}" || fail "HEAD's measured inputs differ from the frozen $frozen"
      match=true
    fi
    head=$(git rev-parse HEAD)
    printf 'head %s\nfrozen %s\n' "$head" "${frozen:-none}" | tee "$W/commits.txt"
    sn schedule "${dryarg[@]}" --out "$W/schedule.json"
    # the build: ab and windows at HEAD
    st=0; bd build cargo build --release --example ab --example windows || st=$?
    deadline "$st" && late "the build" "$st"
    [ "$st" -eq 0 ] || { outcome="INCOMPLETE: the build failed (exit $st)"; exit 1; }
    mkdir "$W/bin"
    cp target/release/examples/ab "$W/bin/ab"
    AB=$W/bin/ab
    sha() { sha256sum "$1" | cut -d' ' -f1; }
    ab0=$(sha "$AB")
    # the bench's corpus, from its windows (windows/, else the bench-data branch), by digest
    want=$(sn get CORPUS_SHA256)
    corpus=corpora/wikidata.bz2
    if [ ! -f "$corpus" ] || [ "$(sha "$corpus")" != "$want" ]; then
      dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
      if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
        st=0; bd fetch git fetch -q origin bench-data:refs/remotes/origin/bench-data || st=$?
        deadline "$st" && late "the bench-data fetch" "$st"
        [ "$st" -ne 0 ] || git archive origin/bench-data "$dir" | tar -x || true
      fi
      st=0; bd corpus python3 bench/wikidata.py fetch bench/wikidata.json "$dir" > /dev/null || st=$?
      deadline "$st" && late "the windows' fetch" "$st"
      [ "$st" -eq 0 ] || fail "the corpus's windows could not be fetched"
      mkdir -p corpora
      st=0; bd corpus python3 bench/wikidata.py build bench/wikidata.json "$dir" target/release/examples/windows "$corpus" || st=$?
      deadline "$st" && late "the corpus's build" "$st"
      [ "$st" -eq 0 ] || fail "the corpus could not be built"
    fi
    [ "$(sha "$corpus")" = "$want" ] || fail "corpora/wikidata.bz2 is not bench/wikidata.json's $want"
    # the corpora: the content decoded by libbzip2, recompressed by lbzip2 at -2 and -9, each
    # decoded back and checked (bench/share_nothing.py corpora); kept outside the bundle
    mkdir "$W/corpora" "$W/work"
    st=0; bd corpora python3 bench/share_nothing.py corpora "$corpus" "$W/work" --record "$W/corpora.json" || st=$?
    deadline "$st" && late "the corpora's construction" "$st"
    [ "$st" -eq 0 ] || fail "the corpora are not as registered (corpora.json)"
    C=$B/corpora-$(date -u +%Y%m%dT%H%M%SZ)
    mkdir "$C"
    mv "$W/work/wikidata-l2.bz2" "$W/work/wikidata-l9.bz2" "$C/"
    cp "$corpus" "$C/wikidata.bz2"
    for n in wikidata-l2 wikidata-l9 wikidata; do ln -s "$C/$n.bz2" "$W/corpora/$n.bz2"; done
    {
      sha256sum "$AB" "$C"/*.bz2
      lbzip2 --version 2>&1 | head -1
      cargo --version; rustc --version
      uname -a
    } > "$W/inputs.txt"
    cat "$W/inputs.txt"
    # the preflight: read-only and passive, each collector bounded, after the build and before
    # anything is timed. PASS, or the attempt is REFUSED here and archived (a DRY check records
    # it and goes on)
    mkdir "$W/preflight"
    P=$W/preflight
    { timeout --kill-after=5 30 lscpu > "$P/lscpu.txt" 2> "$P/lscpu.err"; echo "lscpu: exit $?"
      timeout --kill-after=5 30 "$AB" sample --count 3 --interval 1 > "$P/sample.jsonl" 2> "$P/sample.err"; echo "ab sample: exit $?"
      timeout --kill-after=5 30 python3 bench/share_nothing.py static --out "$P/static.json" 2> "$P/static.err"; echo "static facts: exit $?"
    } > "$P/exits.txt" || true
    set +e
    python3 bench/share_nothing.py preflight "$W" > "$W/preflight.txt"
    pf=$?
    set -e
    cat "$P/exits.txt" "$W/preflight.txt"
    if [ "$pf" -ne 0 ]; then
      if [ -n "$dry" ]; then
        echo "bench-share-nothing: DRY RUN: the preflight did not pass (exit $pf), recorded; continuing"
      else
        outcome="REFUSED: the preflight, before any run: $(head -1 "$W/preflight.txt" 2>/dev/null || echo "its judgement wrote nothing (exit $pf)")"
        exit 4
      fi
    fi
    cat /proc/loadavg > "$W/loadavg-before.txt"
    # the drive: every slot of the schedule, once, in order
    set +e
    bd drive python3 bench/share_nothing.py drive "$W" "$AB" "${dryarg[@]}" "${busyarg[@]}"
    dst=$?
    set -e
    echo "drive: exit $dst" | tee -a "$W/exits.txt"
    cat /proc/loadavg > "$W/loadavg-after.txt"
    [ "$dst" -ne 125 ] || late "the drive" "$dst"
    clean_after=false; [ -n "$(git status --porcelain --untracked-files=no)" ] || clean_after=true
    sn identity "$W" dry:="$( [ -n "$dry" ] && echo true || echo false)" head="$head" \
      frozen:="$( [ -n "$frozen" ] && echo "\"$frozen\"" || echo null)" inputs_match_frozen:="$match" \
      tree_clean_after:="$clean_after" preflight_exit:="$pf" drive_exit:="$dst" \
      ab_sha256="$ab0" ab_sha256_after="$(sha "$AB")" settle_s:="$(sn get SETTLE_S)" \
      max_busy:="${maxbusy:-$(sn get MAX_BUSY)}" arm="$(sn get ARM)"
    # the judge, fixed before data; its first line is the outcome
    set +e
    bd judge python3 bench/share_nothing.py judge "$W" "${dryarg[@]}" > "$W/judgement.txt"
    rc=$?
    set -e
    cat "$W/judgement.txt"
    deadline "$rc" && late "the judge" "$rc"
    if [ "$rc" -le 2 ] && [ -s "$W/judgement.json" ]; then
      outcome="$(head -1 "$W/judgement.txt")"
    else
      outcome="INVALID: the judge exited $rc without writing its judgement"
    fi
