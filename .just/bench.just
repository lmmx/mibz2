# bench/occupancy.sh, exactly as CI runs it, with any arguments passed on unchanged:
#   just bench-occupancy
#   just bench-occupancy --threads 1,2,4,8,12,16,20
# It writes occupancy-<UTC time>.json, the file to share.

# Where the parallel decoder's time goes at each thread count (#39), on the pinned corpus
[positional-arguments]
bench-occupancy *args:
    bash bench/occupancy.sh "$@"

# bench/imc.py: the machine's inventory, 30 s of idle readings, a memcpy calibration, then
# the core and memory-controller counters at 1, 6, 8 and 10 threads, each reading beside an
# idle reading of its own. Needs perf and sudo. It writes imc-<UTC time>.json to share.
#   just bench-imc
#   just bench-imc --note "fresh boot, nothing open" --threads 1,4,6,8,10
#   just bench-imc idle calibrate

# The memory controller's and the cores' counters around the decode (#39)
[positional-arguments]
bench-imc *args:
    bash bench/corpus.sh
    python3 bench/imc.py "$@"

# bench-imc (its arguments passed on), then bench-occupancy at 6, 8 and 10 threads; it
# writes imc-<UTC time>.json and occupancy-<UTC time>.json.
#   just bench-quiet --note "fresh boot, nothing open"

# On a quiet machine: the counters, then the occupancy curve at 6, 8 and 10 threads
[positional-arguments]
bench-quiet *args:
    bash bench/corpus.sh
    python3 bench/imc.py "$@"
    bash bench/occupancy.sh --threads 6,8,10

# bench/thermal.py: turbostat's busy clock, package temperature and power sampled
# throughout; the occupancy runs at 6 and 8 threads each joined to the samples taken while
# they ran, then 90 s of back-to-back decodes at 4, 6, 8 and 20 threads, each after 30 s of
# idle, as a release decodes. Needs turbostat (linux-tools) and sudo. It writes
# thermal-<UTC time>.json (and the occupancy file) to share.
#   just bench-thermal
#   just bench-thermal --occupancy 6 --threads 4,5,6 --seconds 120
#   just bench-thermal sustained --note "fresh boot"

# The clock, temperature and power beside the decode, and sustained throughput (#39)
[positional-arguments]
bench-thermal *args:
    bash bench/corpus.sh
    python3 bench/thermal.py "$@"

# bench/regimes.py: the same decode run as the occupancy instrument runs it (plain), under
# perf stat as this user, and under sudo perf stat as bench-imc's core step does, each with
# the instrument's 0.15 s gap before a run and with none, interleaved by round. Each run's
# wall time, and where perf counts it its clock (cycles / ref-cycles) and IPC. It writes
# regimes-<UTC time>.json to share.
#   just bench-regimes
#   just bench-regimes --threads 8 --rounds 20

# Which way of running the decode gives the slow runs, and at what clock (#39)
[positional-arguments]
bench-regimes *args:
    bash bench/corpus.sh
    python3 bench/regimes.py "$@"

# bench/scaling.py: examples/ab.rs's mibz2, pbz2 and lbzip2 arms on the benchmark's
# Wikidata corpus at 2, 4, 6, 8, 10 and 20 threads, interleaved by round, back to back (no
# idle wait before a run); each arm's throughput and its scaling against its own 4-thread
# runs, paired by round. It writes scaling-<UTC time>.json (and ab's .jsonl) to share.
#   just bench-scaling
#   just bench-scaling --threads 2,4,5,6,8 --rounds 10

# Whether mibz2's scaling knee is the decoder's or the machine's: other decoders beside it (#39)
[positional-arguments]
bench-scaling *args:
    bash bench/corpus.sh
    cargo build --release --example ab
    python3 bench/scaling.py "$@"

# bench/coop_decoder.py: the cooperative decoder's full-decoder A/B against its release
# gates. examples/ab.rs runs the Wikidata corpus from disk to hashed output (every CRC
# checked): today's decoder, cooperative teams of 2 and 3, and the policy the release would
# make the default (mibz2-auto), at 4, 6, 8 and 10 threads and all CPUs, 20 rounds
# interleaved, unpinned; then a pass limited to 4 physical cores, where the policy must
# pick today's decoder and be no slower. About 10 minutes; close the browser and other
# busy programs first. Upload coop-decoder-*.md, coop-decoder-*.json and the .jsonl files.
#   just bench-coop-decoder

# The cooperative decoder against today's, end to end, with its release gates
[positional-arguments]
bench-coop-decoder *args:
    bash bench/corpus.sh
    cargo build --release --example ab
    python3 bench/coop_decoder.py "$@"

# bench/variants.py: released versions (v0.1.1-v0.1.5) and walk settings on this build
# (16 or 8 lanes; prefetch off, into L2, non-temporal), each built in its own worktree,
# checked to give this build's bytes, then run beside this build at 2, 4, 6, 8 and 10
# threads, interleaved by round, back to back. Each one's scaling against its own 4-thread
# runs and its throughput against this build's. It writes variants-<UTC time>.json.
#   just bench-variants
#   just bench-variants --variants v0.1.2,v0.1.3,lanes8,pf-off --rounds 8
#   just bench-variants --variants v0.1.4,pairwalk,v0.1.5 --threads 1,4,6,8,10 --rounds 10

# The one-pass build (docs/journal/2026-10-29-one-pass-build.md, pre-registered): this
# build against v0.2.0, whose source differs from it by that change alone, at 1-10 threads,
# paired by round. At 1-4 threads the default plan is a block a worker (`bwt::build`); from
# 6 it plans teams, whose fused build is untouched, so those rows are the control. About
# 5 minutes. Upload variants-*.json, its .jsonl and the console.
#   just bench-one-pass 2>&1 | tee one-pass-console.log
bench-one-pass rounds="10":
    git rev-parse -q --verify 'v0.2.0^{commit}' > /dev/null || git fetch origin tag v0.2.0 --no-tags
    just bench-variants --variants v0.2.0 --threads 1,2,4,6,8,10 --rounds {{rounds}} --note "one-pass build against v0.2.0"

# Which of mibz2's own changes moved its scaling knee: versions and walk settings (#39, #52)
[positional-arguments]
bench-variants *args:
    bash bench/corpus.sh
    cargo build --release --example ab
    python3 bench/variants.py "$@"

# bench/contend.py: N copies of examples/phases.rs at once (1, 2, 4, 6, 8, 10), each pinned
# to a physical core of its own, sharing the L3 and memory as N workers do: each stage's
# time a byte (scanner, MTF, build, walk, assembly, RLE1, CRC) against one copy alone. It
# writes contend-<UTC time>.json to share.
#   just bench-contend
#   just bench-contend --copies 1,4,8 --reps 3

# Which of the decode's stages slow down as more decodes share the machine (#39)
[positional-arguments]
bench-contend *args:
    bash bench/corpus.sh
    cargo build --release --example phases
    python3 bench/contend.py "$@"

# The workstation session of #39's last questions, in order: the unmultiplexed counter
# groups at 4-10 threads, the stages under contention, and the walk gates (at most 2, 3, 4
# or 6 workers in the T-vector phase at once) beside this build. About 10-15 minutes;
# needs sudo for perf. Upload imc-*.json, contend-*.json and variants-*.json(l).
#   just bench-session --note "quiet, recent reboot"
[positional-arguments]
bench-session *args:
    bash bench/corpus.sh
    cargo build --release --example ab --example phases
    python3 bench/imc.py core --events grouped --threads 4,6,8,10 --reps 3 --settle 0 "$@"
    python3 bench/contend.py "$@"
    python3 bench/variants.py --variants gate2,gate3,gate4,gate6 --threads 4,6,8,10 "$@"

# bench/blocks.py: the benchmark's content recompressed at -1, -3, -6 and -9 (lbzip2), each
# decoded by mibz2 at 2, 4, 6, 8 and 10 threads back to back: whether the best worker
# count moves as the blocks (and so every worker's T-vector) shrink. It writes
# blocks-<UTC time>.json (and ab's .jsonl per level) to share.
#   just bench-blocks --note "quiet, recent reboot"

# Whether the scaling knee follows the block size: the same content at -1 to -9 (#39, #65)
[positional-arguments]
bench-blocks *args:
    bash bench/corpus.sh
    cargo build --release --example ab
    python3 bench/blocks.py "$@"

# The footprint ledger (src/trace.rs, docs/ledger.md) for the pinned corpus: what the
# parallel decoder reserves, holds live and reads at random, by category, at each thread
# count, from the code alone (no run). bench-occupancy prints it with the measured figures.
#   just bench-ledger
#   just bench-ledger --threads 1,2,4,10

# The parallel decoder's footprint, by category, from its arithmetic (#39, #65)
[positional-arguments]
bench-ledger *args:
    bash bench/corpus.sh
    target/release/examples/occupancy corpora/wikidata.bz2 --ledger "$@"

# bench/fingerprint.py: the contention fingerprint. mibz2 at 1 thread, pinned to one
# physical core, alone and beside a competitor on another core: a register-only loop (the
# control), a random read-modify-write bubble of half the effective cache, a streaming copy
# over 2 x 64 MB, and a second copy of itself, in randomized interleaved rounds, after a
# calibration on this machine (a pointer chase gives the effective cache; each stressor
# is checked). Each condition's slowdown against solo and against the control, or
# INCONCLUSIVE with the reason. About 1-2 minutes. It writes fingerprint-<UTC time>.json
# and .md to share.
#   just bench-fingerprint
#   just bench-fingerprint --rounds 12 --sibling
#   just bench-fingerprint --victim head=target/release/examples/ab --victim v0.1.5=/path/to/ab

# Whether the decoder slows unusually beside a co-runner on another core
[positional-arguments]
bench-fingerprint *args:
    bash bench/corpus.sh
    cargo build --release --example ab --example stress
    python3 bench/fingerprint.py "$@"

# bench/compete.sh: mibz2 against bz2ngrs on one physical core (research only: the whole
# decoders with libbzip2, libbz2-rs and bz2-ng's C beside them, then the two inverse BWTs
# phase by phase on the same blocks, then perf's hottest symbols). About 10 minutes. It
# writes compete-<UTC time>.md (and its .jsonl and .kernels.jsonl) to share.
#   just bench-compete
#   just bench-compete --rounds 7 --passes 6 --no-perf

# A competitor study: mibz2 against bz2ngrs, one core (docs/journal/2026-10-29-bz2ngrs.md)
[positional-arguments]
bench-compete *args:
    bash bench/corpus.sh
    bash bench/compete.sh "$@"

# bench/pmu_probe.py, as the PMU probe workflow runs it on CI: the CPU, its caches and
# quota, perf's permissions and the PMUs registered, then each candidate counter tried on
# its own over a short busy loop. Needs perf and sudo. It writes pmu-probe.json to share.
#   just bench-pmu-probe
#   just bench-pmu-probe --out pmu-probe-i9.json

# What this machine's performance counters can measure (diagnostic only)
[positional-arguments]
bench-pmu-probe *args:
    python3 bench/pmu_probe.py "$@"

# Release scaling (the History page's Release scaling view): NEW against OLD and lbzip2, from
# 1 thread to every CPU, in paired rounds on the bench's Wikidata sample (and any FILES).
# Builds both releases in target/release-scaling/, then records the run in
# bench/release-scaling/<date>-<machine>-<release>/ for a pull request. Needs lbzip2.
# With no NEW, it is the current release (the newest vX.Y.Z tag); with no OLD, the release
# before NEW. The release runbook (docs/RELEASING.md) runs it bare after every release.
#   just bench-release-scaling
#   just bench-release-scaling v0.2.4 v0.2.1 10 "1,4,10,20" /path/to/big.bz2
# The held spinner (bench 3.8.0, `ab run --hold-cpu auto --hold-max-threads 10`): around
# every run of 10 threads or fewer, one CPU the run leaves free is held busy, so no decode
# starts on an idle machine (cold-start/20261110T102700Z). The runs are NEW's harness, so it
# is on only where NEW's `ab` has the option (a release or commit from bench 3.8.0 on), and
# the console says which; both arms run under the same spinner, the pairing untouched. Each
# held run's record carries `held_spinner`. HOLD_CPU=off turns it off; HOLD_CPU=N holds CPU N.

# Two releases and lbzip2 on this machine, up to its full thread count
bench-release-scaling NEW="" OLD="" ROUNDS="10" THREADS="" *FILES="":
    #!/usr/bin/env bash
    set -euo pipefail
    # Release tags only: a fetch of every tag fails ("would clobber existing tag") once a
    # ready/<recipe> tag has moved on origin, as ready tags do (bench/workstation-queue.txt).
    git fetch -q origin '+refs/tags/v*:refs/tags/v*'
    # (release tags only, newest first: vX.Y.Z, no pre-releases)
    releases=$(git tag -l 'v*' | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | sort -rV)
    NEW="{{NEW}}"; OLD="{{OLD}}"
    [ -n "$NEW" ] || NEW=$(echo "$releases" | head -1)
    [ -n "$OLD" ] || OLD=$(echo "$releases" | awk -v n="$NEW" 'found {print; exit} $0 == n {found = 1}')
    [ -n "$NEW" ] && [ -n "$OLD" ] || { echo "bench-release-scaling: no release before ${NEW:-the newest tag} to compare against"; exit 1; }
    W=$PWD/target/release-scaling
    mkdir -p "$W"
    O=$W/run-$(date -u +%Y%m%dT%H%M%SZ)
    mkdir "$O"
    exec > >(tee -a "$O/console.log") 2>&1
    echo "bench-release-scaling: $NEW against $OLD"
    for v in "$NEW" "$OLD"; do
      [ -d "$W/build-$v" ] || git worktree add -q --detach "$W/build-$v" "$v"
      (cd "$W/build-$v" && cargo build -q --release --example ab --example windows)
    done
    NEWAB=$W/build-$NEW/target/release/examples/ab
    OLDAB=$W/build-$OLD/target/release/examples/ab
    dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
    if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
      git fetch -q origin bench-data:refs/remotes/origin/bench-data 2>/dev/null \
        && git archive origin/bench-data "$dir" | tar -x || true
    fi
    python3 bench/wikidata.py fetch bench/wikidata.json "$dir" >/dev/null
    python3 bench/wikidata.py build bench/wikidata.json "$dir" "$W/build-$NEW/target/release/examples/windows" "$W/wikidata.bz2"
    threads="{{THREADS}}"
    [ -n "$threads" ] || threads=$(python3 -c "import os; n = os.cpu_count(); print(','.join(map(str, sorted(t for t in {1, 2, 4, 8, 12, 16, n // 2, n} if t <= n))))")
    "$NEWAB" env > "$W/env.json"
    lscpu > "$W/lscpu.txt"
    rm -f "$W/runs.jsonl"
    # (the held spinner where NEW's harness has it: runs of 10 threads or fewer)
    hold=(); hold_cpu=${HOLD_CPU:-auto}
    if [ "$hold_cpu" = off ]; then
      echo "bench-release-scaling: held spinner off (HOLD_CPU=off)"
    elif grep -qa -- '--hold-max-threads' "$NEWAB"; then
      hold=(--hold-cpu "$hold_cpu" --hold-max-threads 10)
      echo "bench-release-scaling: held spinner on ($hold_cpu) around runs of 10 threads or fewer"
    else
      echo "bench-release-scaling: $NEW's ab has no --hold-cpu (before bench 3.8.0): no held spinner"
    fi
    "$NEWAB" run "$W/runs.jsonl" --arms "mibz2,mibz2@$OLDAB,lbzip2" \
      --threads "$threads" --rounds {{ROUNDS}} --spread "${hold[@]}" "$W/wikidata.bz2" {{FILES}}
    "$NEWAB" summary "$W/runs.jsonl" --ref mibz2 | tee "$W/summary.txt"
    lb=$(lbzip2 --version 2>&1 | head -1 | grep -oE '[0-9]+(\.[0-9]+)+' || true)
    files=(--file "wikidata=$W/wikidata.bz2")
    for f in {{FILES}}; do files+=(--file "$(basename "$f" .bz2)=$f"); done
    # The record goes under target/, not into the checkout: a file left in the working tree would make
    # the workstation queue refuse every later entry (bench/await-submit.sh needs a clean tree).
    # The coordinator commits recorded/ to bench/release-scaling/ from the bundle (RELEASING §5).
    rec=$(mktemp -d "$W/record.XXXXXX")
    python3 bench/release_scaling.py record "$rec" --runs "$W/runs.jsonl" \
      --env "$W/env.json" --lscpu "$W/lscpu.txt" --ref mibz2 \
      --arm "mibz2=${NEW#v}" --arm "mibz2@$OLDAB=${OLD#v}" \
      --arm "lbzip2=lbzip2 $lb" "${files[@]}"
    cp "$W/runs.jsonl" "$W/env.json" "$W/lscpu.txt" "$W/summary.txt" "$O/"
    mv "$rec/$(ls "$rec")" "$O/recorded" && rmdir "$rec"
    bash bench/bundle.sh "$O"

# Team size against live tables (docs/journal/2026-11-06-team-size.md, registered before any
# run): what holds the decoder at its 8-thread ceiling. The cooperative decoder at teams × k
# with the default's window (mibz2-team2/3/4 at T threads run T // k teams of k; mibz2, the
# policy, runs 4 × 2 on the i9 from 8 threads up) at 8, 10, 12 and 16 threads, paired by
# round against mibz2 at 8; then perf's counters on the registered cells; then the
# occupancy trace of 4 × 2, 4 × 3 and 4 × 4, pinned spread against packed. Writes
# target/team-size/ and prints the report. About 20 minutes. Close the browser and anything
# busy first.
#   just bench-team-size
#   just bench-team-size 20 "8,10,12,16" /path/to/big.bz2
# It keeps its own console log and ends by packing the attempt into one .tar.gz to upload.

# Teams of 2, 3 and 4 at fixed live tables, against today's 4 × 2
bench-team-size ROUNDS="20" THREADS="8,10,12,16" *FILES="":
    #!/usr/bin/env bash
    set -euo pipefail
    # each attempt in a directory of its own, never overwritten: the registration's
    # invalidation and confirmation rules need every attempt's evidence
    B=$PWD/target/team-size
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-team-size: $W already exists; not reusing an attempt's directory"; exit 1; }
    exec > >(tee -a "$W/console.log") 2>&1
    cargo build -q --release --example ab --example windows
    cargo build -q --release --features trace --example occupancy
    AB=$PWD/target/release/examples/ab
    dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
    if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
      git fetch -q origin bench-data:refs/remotes/origin/bench-data 2>/dev/null \
        && git archive origin/bench-data "$dir" | tar -x || true
    fi
    python3 bench/wikidata.py fetch bench/wikidata.json "$dir" >/dev/null
    python3 bench/wikidata.py build bench/wikidata.json "$dir" target/release/examples/windows "$B/wikidata.bz2"
    ln -sf "$B/wikidata.bz2" "$W/wikidata.bz2"
    "$AB" env > "$W/env.json"
    lscpu > "$W/lscpu.txt"
    git rev-parse HEAD > "$W/commit.txt"
    "$AB" run "$W/runs.jsonl" --arms "mibz2,mibz2-team2,mibz2-team3,mibz2-team4" \
      --threads "{{THREADS}}" --rounds {{ROUNDS}} "$W/wikidata.bz2" {{FILES}}
    cells="mibz2@8,mibz2-team3@12,mibz2-team4@16,mibz2-team2@10,mibz2-team2@12,mibz2-team3@10"
    if perf stat -e cycles -- true >/dev/null 2>&1; then
      python3 bench/team_size.py counters "$AB" "$W/wikidata.bz2" "$W/counters.jsonl" --cells "$cells" \
        || echo "bench-team-size: the counters failed; the timed report stands without them"
    else
      echo "bench-team-size: perf can't count here (try: sudo sysctl kernel.perf_event_paranoid=1); no counters"
    fi
    ctr=(); [ -s "$W/counters.jsonl" ] && ctr=(--counters "$W/counters.jsonl")
    python3 bench/team_size.py report "$W/runs.jsonl" "${ctr[@]}" --ref mibz2@8 --rounds {{ROUNDS}} \
      --arms "mibz2,mibz2-team2,mibz2-team3,mibz2-team4" --threads "{{THREADS}}" --out "$W/team-size"
    for tk in 8:2 12:3 16:4; do
      t=${tk%:*}; k=${tk#*:}
      target/release/examples/occupancy "$W/wikidata.bz2" --threads "$t" --teams "$k" \
        --placements spread/packed --rounds 6 --out "$W/occupancy-t${t}-k${k}.json" \
        --meta "commit=$(git rev-parse HEAD)" \
        || echo "bench-team-size: the occupancy pass at $t threads, teams of $k, failed"
    done
    bash bench/bundle.sh "$W"

# The three-team policy's workstation A/B (docs/journal/2026-11-06-three-team-policy.md,
# registered before any run): this commit's default plan against its merge base's, at the
# same requested thread counts, 20 paired rounds on the bench's Wikidata sample, judged by
# bench/policy_ab.py under the rule Louis approved. The run and its confirmation are two runs
# of this same recipe at the same commit. About 10 minutes; it ends with one .tar.gz to upload.
#   just bench-three-team-policy

# Head default against base default, 8 to 20 threads
bench-three-team-policy:
    #!/usr/bin/env bash
    set -euo pipefail
    # each attempt in a directory of its own, never overwritten
    B=$PWD/target/three-team-policy
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-three-team-policy: $W already exists; not reusing an attempt's directory"; exit 1; }
    exec > >(tee -a "$W/console.log") 2>&1
    # every attempt ends with one stated outcome (outcome.txt) and is packed, however it ends:
    # a refusal, a failed build, a broken report or a full run
    outcome="INCOMPLETE: stopped before the run finished (see console.log)"
    finish() {
      status=$?
      echo "$outcome" > "$W/outcome.txt"
      echo "bench-three-team-policy: OUTCOME: $outcome"
      bash bench/bundle.sh "$W"
      # (VALID PASS and VALID FAIL exit 0, INVALID 1, anything that kept the run from a
      # verdict, a refusal or a report error among them, 2 or its own failing status)
      case "$outcome" in
        "VALID "*) exit 0 ;;
        INVALID*) exit 1 ;;
        *) [ "$status" -ne 0 ] && exit "$status"; exit 2 ;;
      esac
    }
    trap finish EXIT
    fail() { outcome="REFUSED: $*"; echo "bench-three-team-policy: $*"; exit 1; }
    # the commits the registration froze (bench/three-team-policy.frozen): the candidate, whose
    # build and workload inputs HEAD must match exactly, and the base it is measured against
    frozen=bench/three-team-policy.frozen
    cand=$(awk '$1 == "candidate" {print $2}' "$frozen")
    base=$(awk '$1 == "base" {print $2}' "$frozen")
    [[ $cand =~ ^[0-9a-f]{40}$ && $base =~ ^[0-9a-f]{40}$ ]] || fail "$frozen names no frozen candidate and base yet; nothing to run"
    # what the builds and the workload are made from: sources, Cargo's files and configuration,
    # the harness, the corpus's builder and manifest, the judge and this recipe
    # (a path that doesn't exist counts too: one appearing, such as a build.rs or a toolchain
    # file, is a change)
    measured=(src Cargo.toml Cargo.lock .cargo build.rs rust-toolchain rust-toolchain.toml examples bench/policy_ab.py bench/wikidata.json bench/wikidata.py .just/bench.just)
    # no environment that changes how Cargo, rustc or rustup build, or how ab observes: any
    # CARGO_* or RUST* variable but those that only say where they are installed, how Cargo
    # reaches its registry (the lockfile pins what it fetches) or how it prints, and the
    # census's interval, which stays at its registered 5 ms
    overrides=$(env | cut -d= -f1 | grep -E '^(CARGO_|RUST|AB_CENSUS_MS$|AB_SAMPLE_MS$)' | grep -vxE 'CARGO_HOME|RUSTUP_HOME|RUST_BACKTRACE|RUST_LOG|CARGO_(HTTP|NET|TERM)_[A-Z_]+' || true)
    [ -z "$overrides" ] || fail "unset these, so both builds and both arms run the same, registered way: $(echo $overrides)"
    [ -z "$(git status --porcelain --untracked-files=no)" ] || fail "the checkout has uncommitted changes"
    [ -z "$(git status --porcelain --untracked-files=all -- "${measured[@]}")" ] || fail "untracked files among the build and workload inputs"
    git cat-file -e "$cand^{commit}" && git cat-file -e "$base^{commit}" || fail "fetch the branch: the frozen commits are missing"
    git diff --quiet "$cand" HEAD -- "${measured[@]}" || fail "HEAD's build or workload inputs differ from the frozen candidate $cand"
    head=$(git rev-parse HEAD)
    echo "head $head (build and workload inputs as candidate $cand)"; echo "base $base"
    printf 'head %s\ncandidate %s\nbase %s\n' "$head" "$cand" "$base" > "$W/commits.txt"
    [ -d "$B/build-$base" ] || git worktree add -q --detach "$B/build-$base" "$base"
    [ "$(git -C "$B/build-$base" rev-parse HEAD)" = "$base" ] || fail "$B/build-$base is not at the base $base"
    [ -z "$(git -C "$B/build-$base" status --porcelain --untracked-files=all)" ] || fail "$B/build-$base has changes or untracked files"
    # (each executable as Cargo reports building it, from its own checkout, then copied into
    # this attempt's directory: the copies are what run and what the digests describe, so
    # neither build can overwrite the other's, wherever Cargo's target directory is)
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
    # Cargo reads `.cargo/config(.toml)` in every directory above where it builds. The base
    # checkout sits inside this one, so a configuration between the two (in target/, or this
    # checkout's own root) would reach the base build only: refused. Above this checkout both
    # builds read the same files, which inputs.txt records
    configs() { local d=$1 stop=$2; while :; do for f in "$d/.cargo/config" "$d/.cargo/config.toml"; do [ -f "$f" ] && echo "$f"; done; [ "$d" = "$stop" ] && break; d=$(dirname "$d"); done; return 0; }
    only_base=$(configs "$(dirname "$B/build-$base")" "$PWD")
    [ -z "$only_base" ] || fail "Cargo configuration only the base build would read: $(echo $only_base)"
    # the toolchain each build will use, asked in its own checkout (a toolchain file there
    # would change it): they must be the same
    tool() { (cd "$1" && rustc -vV && cargo -V); }
    [ "$(tool "$PWD")" = "$(tool "$B/build-$base")" ] || fail "the two checkouts select different toolchains"
    mkdir "$W/bin"
    # (one build at a time, its status checked here, and its executable copied before the
    # next build starts: a target directory both share can't swap one build's for the other's)
    base_built=$(built "$B/build-$base" ab) || fail "the base build failed"
    cp "$base_built" "$W/bin/ab-base"
    sha256sum "$W/bin/ab-base" > "$W/bin/ab-base.sha256"
    head_built=$(built "$PWD" ab windows) || fail "the head build failed"
    [ "$(printf '%s\n' "$head_built" | wc -l)" = 2 ] || fail "Cargo reported no executables for the head build"
    cp "$(printf '%s\n' "$head_built" | sed -n 1p)" "$W/bin/ab-head"
    cp "$(printf '%s\n' "$head_built" | sed -n 2p)" "$W/bin/windows"
    sha256sum -c --quiet "$W/bin/ab-base.sha256" || fail "the base executable changed after it was copied"
    AB=$W/bin/ab-head
    BASEAB=$W/bin/ab-base
    WINDOWS=$W/bin/windows
    dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
    if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
      git fetch -q origin bench-data:refs/remotes/origin/bench-data 2>/dev/null \
        && git archive origin/bench-data "$dir" | tar -x || true
    fi
    python3 bench/wikidata.py fetch bench/wikidata.json "$dir" >/dev/null
    python3 bench/wikidata.py build bench/wikidata.json "$dir" "$WINDOWS" "$B/wikidata.bz2"
    ln -sf "$B/wikidata.bz2" "$W/wikidata.bz2"
    # what ran: both executables, the corpus and the toolchain, by digest, and the Cargo
    # configuration outside the repository, which applies to both builds alike
    {
      sha256sum "$AB" "$BASEAB" "$WINDOWS" "$B/wikidata.bz2"
      tool "$PWD"
      # (one path a line, so a directory with a space in its name stays one path)
      { printf '%s\n' "${CARGO_HOME:-$HOME/.cargo}/config" "${CARGO_HOME:-$HOME/.cargo}/config.toml"; configs "$(dirname "$PWD")" /; } \
        | while IFS= read -r f; do [ -f "$f" ] && sha256sum "$f"; done; true
      echo "census every 5 ms (AB_CENSUS_MS unset)"
    } > "$W/inputs.txt"
    cat "$W/inputs.txt"
    "$AB" env > "$W/env.json"
    lscpu > "$W/lscpu.txt"
    "$AB" run "$W/runs.jsonl" --arms "mibz2,mibz2@$BASEAB" \
      --threads "8,9,10,11,12,16,20" --rounds 20 "$W/wikidata.bz2"
    # the verdict: VALID PASS, VALID FAIL or INVALID, from the report's own fields; a report
    # that couldn't be written is said to be so, and points to nothing
    set +e
    python3 bench/policy_ab.py report "$W/runs.jsonl" --out "$W/policy-ab"
    rc=$?
    set -e
    if [ "$rc" -le 1 ] && [ -s "$W/policy-ab.json" ] && [ -s "$W/policy-ab.md" ]; then
      outcome=$(python3 -c 'import json,sys; v=json.load(open(sys.argv[1])); print(("VALID PASS" if v["pass"] else "VALID FAIL") if v["valid"] else "INVALID: " + "; ".join(v["problems"]))' "$W/policy-ab.json")
    else
      outcome="REPORT ERROR: policy_ab.py exited $rc without writing its report"
    fi

# The pipelined team on the workstation (docs/journal/2026-11-07-team-pipeline-registration.md,
# registered before any run): 3 × 3 and 3 × 4, each pipelined (mibz2-pipe3/4, research,
# opt-in) and not (mibz2-team3/4), with the default (mibz2) as the product reference, at 10
# and 12 threads, interleaved by round; then occupancy traces of each registered plan with the
# pipeline's phase marks; then bench/team_pipeline.py's report and verdict. About 25 minutes.
# Close the browser and anything busy first.
#   just submit bench-team-pipeline
# DRY=1 is the untimed check of the recipe itself (one round, one trace round, no settling):
# never evidence, and the report never calls it VALID. CORPUS= a .bz2 to use instead of
# building the bench's sample.

# Pipelined teams against their non-pipelined twins, at 3 × 3 and 3 × 4
bench-team-pipeline ROUNDS="20" DRY="" CORPUS="":
    #!/usr/bin/env bash
    set -euo pipefail
    # each attempt in a directory of its own, never overwritten
    B=$PWD/target/team-pipeline
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-team-pipeline: $W already exists; not reusing an attempt's directory"; exit 1; }
    exec > >(tee -a "$W/console.log") 2>&1
    # every attempt ends with one stated outcome (outcome.txt) and is packed, however it ends:
    # a refusal, a failed build, a broken report or a full run
    outcome="INCOMPLETE: stopped before the run finished (see console.log)"
    finish() {
      status=$?
      echo "$outcome" > "$W/outcome.txt"
      echo "bench-team-pipeline: OUTCOME: $outcome"
      bash bench/bundle.sh "$W"
      case "$outcome" in
        VALID*) exit 0 ;;
        INVALID*) exit 1 ;;
        *) [ "$status" -ne 0 ] && exit "$status"; exit 2 ;;
      esac
    }
    trap finish EXIT
    fail() { outcome="REFUSED: $*"; echo "bench-team-pipeline: $*"; exit 1; }
    rounds={{ROUNDS}}; trace_rounds=6; busy=(); dry="{{DRY}}"
    if [ -n "$dry" ]; then
      rounds=1; trace_rounds=1; busy=(--max-busy 64 --settle 0.2)
      echo "bench-team-pipeline: DRY RUN: one round, untimed; not evidence" | tee "$W/DRY-RUN.txt"
    fi
    git rev-parse HEAD > "$W/commit.txt"
    git status --porcelain --untracked-files=no > "$W/git-status.txt"
    # the registration's frozen commit (bench/team-pipeline.frozen): everything the run is
    # built, driven and judged from must be as it was there
    frozen=$(awk '$1 == "frozen" {print $2}' bench/team-pipeline.frozen)
    [[ $frozen =~ ^[0-9a-f]{40}$ ]] || fail "bench/team-pipeline.frozen names no frozen commit"
    measured=(src Cargo.toml Cargo.lock .cargo build.rs rust-toolchain rust-toolchain.toml examples
      bench/team_pipeline.py bench/team_size.py bench/wikidata.py bench/wikidata.json bench/bundle.sh
      .just/bench.just docs/journal/2026-11-07-team-pipeline-registration.md)
    [ ! -s "$W/git-status.txt" ] || fail "the checkout has uncommitted changes"
    [ -z "$(git status --porcelain --untracked-files=all -- "${measured[@]}")" ] || fail "untracked files among the measured inputs"
    git cat-file -e "$frozen^{commit}" || fail "fetch the branch: the frozen commit $frozen is missing"
    if ! git diff --quiet "$frozen" HEAD -- "${measured[@]}"; then
      echo "the measured inputs differ from the frozen $frozen" > "$W/NOT-FROZEN.txt"
      fail "HEAD's measured inputs differ from the frozen $frozen"
    fi
    echo "frozen $frozen" > "$W/frozen.txt"
    cargo build -q --release --example ab --example windows
    cargo build -q --release --features trace --example occupancy
    AB=$PWD/target/release/examples/ab
    corpus="{{CORPUS}}"
    if [ -z "$corpus" ]; then
      dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
      if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
        git fetch -q origin bench-data:refs/remotes/origin/bench-data 2>/dev/null \
          && git archive origin/bench-data "$dir" | tar -x || true
      fi
      python3 bench/wikidata.py fetch bench/wikidata.json "$dir" >/dev/null
      python3 bench/wikidata.py build bench/wikidata.json "$dir" target/release/examples/windows "$B/wikidata.bz2"
      corpus=$B/wikidata.bz2
    fi
    ln -sf "$(realpath "$corpus")" "$W/wikidata.bz2"
    sha256sum "$corpus" > "$W/corpus.sha256"
    "$AB" env > "$W/env.json"
    lscpu > "$W/lscpu.txt"
    arms="mibz2,mibz2-team3,mibz2-pipe3,mibz2-team4,mibz2-pipe4"
    "$AB" run "$W/runs.jsonl" --arms "$arms" --threads 10,12 --rounds "$rounds" "${busy[@]}" "$W/wikidata.bz2"
    "$AB" summary "$W/runs.jsonl" --ref mibz2 > "$W/summary.txt" || true
    occ=()
    for tkm in 10:3:teams 10:3:pipe 12:4:teams 12:4:pipe; do
      IFS=: read -r t k m <<< "$tkm"
      o="$W/occupancy-t$t-$m$k.json"
      if target/release/examples/occupancy "$W/wikidata.bz2" --threads "$t" "--$m" "$k" \
          --rounds "$trace_rounds" --out "$o" --meta "commit=$(git rev-parse HEAD)" \
          > "$W/occupancy-t$t-$m$k.txt" 2>&1; then
        occ+=("$o")
      else
        echo "bench-team-pipeline: the occupancy pass at $t threads, $m of $k, failed"
      fi
    done
    status=0
    python3 bench/team_pipeline.py report "$W/runs.jsonl" --occupancy "${occ[@]}" \
      --out "$W/team-pipeline" || status=$?
    verdict=$(python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))["verdict"])' "$W/team-pipeline.json" 2>/dev/null || echo "")
    case "$verdict" in
      VALID) outcome="VALID: see team-pipeline.md for the registered comparisons' outcomes" ;;
      INVALID) outcome="INVALID: see team-pipeline.md's validity checks" ;;
      *) outcome="REPORT FAILED (status $status): see console.log" ;;
    esac

# The machine-state recorder's own check on the workstation (bench 3.4.0;
# docs/journal/2026-11-09-telemetry-check-registration.md, registered and frozen before use):
# `ab telemetry-check`'s cases, with `ab env`, three `ab sample` readings and lscpu, as one
# bundle. It times no decoder and is no performance evidence; it uses CPU-time budgets and
# counters only to exercise the recorder. Read-only: it changes no setting. Run once; no retry.
# Its exit: 0 CHECKED (no case FAILed or was INCOMPLETE), 1 FAILED or REFUSED, 3 INCOMPLETE,
# kept through the bundle and `just submit`. About a minute once built (at most 5 minutes).
#   just submit bench-telemetry-check

# The machine-state recorder's failure paths and coverage, checked once
bench-telemetry-check:
    #!/usr/bin/env bash
    set -euo pipefail
    # each attempt in a directory of its own, never overwritten
    B=$PWD/target/telemetry-check
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-telemetry-check: $W already exists; not reusing an attempt's directory"; exit 1; }
    exec > >(tee -a "$W/console.log") 2>&1
    # the outcome decides the exit, whatever the last command's status: a FAIL or an
    # INCOMPLETE check is bundled, and still exits nonzero
    outcome="INCOMPLETE: stopped before the check finished (see console.log)"
    finish() {
      echo "$outcome" > "$W/outcome.txt"
      echo "bench-telemetry-check: OUTCOME: $outcome"
      bash bench/bundle.sh "$W" || { echo "bench-telemetry-check: bundling failed"; exit 2; }
      case "$outcome" in
        CHECKED*) exit 0 ;;
        FAILED*|REFUSED*) exit 1 ;;
        *) exit 3 ;;
      esac
    }
    trap finish EXIT
    fail() { outcome="REFUSED: $*"; echo "bench-telemetry-check: $*"; exit 1; }
    git rev-parse HEAD > "$W/commit.txt"
    printf 'head %s\n' "$(cat "$W/commit.txt")" > "$W/commits.txt"
    git status --porcelain --untracked-files=no > "$W/git-status.txt"
    # the registration's frozen commit (bench/telemetry-check.frozen): what the check is built
    # and driven from must be as it was there, and committed
    frozen=$(awk '$1 == "frozen" {print $2}' bench/telemetry-check.frozen)
    [[ $frozen =~ ^[0-9a-f]{40}$ ]] || fail "bench/telemetry-check.frozen names no frozen commit"
    measured=(src Cargo.toml Cargo.lock .cargo build.rs rust-toolchain rust-toolchain.toml examples
      tests/common bench/bundle.sh .just/bench.just
      docs/journal/2026-11-09-telemetry-check-registration.md)
    [ ! -s "$W/git-status.txt" ] || fail "the checkout has uncommitted changes"
    [ -z "$(git status --porcelain --untracked-files=all -- "${measured[@]}")" ] || fail "untracked files among the measured inputs"
    git cat-file -e "$frozen^{commit}" || fail "fetch the branch: the frozen commit $frozen is missing"
    if ! git diff --quiet "$frozen" HEAD -- "${measured[@]}"; then
      echo "the measured inputs differ from the frozen $frozen" > "$W/NOT-FROZEN.txt"
      fail "HEAD's measured inputs differ from the frozen $frozen"
    fi
    echo "frozen $frozen" > "$W/frozen.txt"
    # the executable Cargo reports building, wherever its target directory is: never an
    # older one at a guessed path
    AB=$(cargo build -q --release --example ab --message-format=json-render-diagnostics \
      | python3 -c 'import json, sys
    got = [m["executable"] for m in map(json.loads, sys.stdin) if m.get("reason") == "compiler-artifact" and m.get("executable") and m["target"]["name"] == "ab" and "example" in m["target"]["kind"]]
    sys.exit("cargo reported no ab executable") if len(got) != 1 else print(got[0])') \
      || fail "the build reported no ab executable"
    [ -x "$AB" ] || fail "cargo's reported $AB is not an executable"
    { echo "executable $AB"; rustc -vV; cargo -V; } > "$W/build.txt"
    sha256sum "$AB" | sed "s|$AB|ab|" > "$W/inputs.txt"
    "$AB" env > "$W/env.json"
    lscpu > "$W/lscpu.txt"
    "$AB" sample --count 3 --interval 1 > "$W/sample.jsonl"
    # once, bounded: each case by its own deadline, the whole check by 300 s
    status=0
    timeout 300 "$AB" telemetry-check > "$W/telemetry-check.jsonl" || status=$?
    [ "$(sha256sum "$AB" | cut -d' ' -f1)" = "$(cut -d' ' -f1 "$W/inputs.txt")" ] \
      || { outcome="INCOMPLETE: the executable changed while the check ran"; exit 3; }
    python3 -c 'import json, sys
    for l in open(sys.argv[1]):
        r = json.loads(l)
        print(r["status"].ljust(12), r["case"].ljust(36), r["detail"])' "$W/telemetry-check.jsonl" | tee "$W/telemetry-check.txt" || true
    counts=$(python3 -c 'import collections, json, sys
    c = collections.Counter(json.loads(l)["status"] for l in open(sys.argv[1]))
    print(", ".join(f"{k} {v}" for k, v in sorted(c.items())))' "$W/telemetry-check.jsonl" 2>/dev/null || echo "unreadable")
    case "$status" in
      0) outcome="CHECKED: no case FAILed or INCOMPLETE ($counts); see telemetry-check.txt" ;;
      1) outcome="FAILED: a case FAILed ($counts); see telemetry-check.txt" ;;
      3) outcome="INCOMPLETE: a case could not finish validly ($counts); see telemetry-check.txt" ;;
      124) outcome="INCOMPLETE: the check passed its 300 s bound ($counts)" ;;
      *) outcome="INCOMPLETE: the check exited $status ($counts); see console.log" ;;
    esac

# The pipelined team's two levers on the workstation
# (docs/journal/2026-11-07-pipeline-levers-registration.md, registered before any run): at
# 3 × 3 on 10 threads and 3 × 4 on 12, the registered pipe (mibz2-pipe3/4) against each lever
# alone (-park, -small), the spin-budget control (-spin2k), and park and small together
# (-both), with the non-pipelined twins (mibz2-team3/4) and the default (mibz2, the noise
# control's reference at 10); each thread count its own ab run, interleaved by round; then
# occupancy traces of every pipelined arm with the phase marks; then bench/pipeline_levers.py's
# report and verdict. About 40 minutes. Close the browser and anything busy first.
#   just submit bench-pipeline-levers
# DRY=1 is the untimed check of the recipe itself (one round, one trace round, no settling):
# never evidence, and the report never calls it VALID. CORPUS= a .bz2 to use instead of
# building the bench's sample.

# The pipelined team's levers (park, spin2k, small blocks, both) against the registered pipe
bench-pipeline-levers ROUNDS="20" DRY="" CORPUS="":
    #!/usr/bin/env bash
    set -euo pipefail
    # each attempt in a directory of its own, never overwritten
    B=$PWD/target/pipeline-levers
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-pipeline-levers: $W already exists; not reusing an attempt's directory"; exit 1; }
    exec > >(tee -a "$W/console.log") 2>&1
    # every attempt ends with one stated outcome (outcome.txt) and is packed, however it ends:
    # a refusal, a failed build, a broken report or a full run
    outcome="INCOMPLETE: stopped before the run finished (see console.log)"
    finish() {
      status=$?
      echo "$outcome" > "$W/outcome.txt"
      echo "bench-pipeline-levers: OUTCOME: $outcome"
      bash bench/bundle.sh "$W"
      case "$outcome" in
        VALID*) exit 0 ;;
        INVALID*) exit 1 ;;
        *) [ "$status" -ne 0 ] && exit "$status"; exit 2 ;;
      esac
    }
    trap finish EXIT
    fail() { outcome="REFUSED: $*"; echo "bench-pipeline-levers: $*"; exit 1; }
    rounds={{ROUNDS}}; trace_rounds=6; busy=(); dry="{{DRY}}"
    if [ -n "$dry" ]; then
      rounds=1; trace_rounds=1; busy=(--max-busy 64 --settle 0.2)
      echo "bench-pipeline-levers: DRY RUN: one round, untimed; not evidence" | tee "$W/DRY-RUN.txt"
    fi
    git rev-parse HEAD > "$W/commit.txt"
    git status --porcelain --untracked-files=no > "$W/git-status.txt"
    # the registration's frozen commit (bench/pipeline-levers.frozen): everything the run is
    # built, driven and judged from must be as it was there
    frozen=$(awk '$1 == "frozen" {print $2}' bench/pipeline-levers.frozen)
    [[ $frozen =~ ^[0-9a-f]{40}$ ]] || fail "bench/pipeline-levers.frozen names no frozen commit"
    measured=(src Cargo.toml Cargo.lock .cargo build.rs rust-toolchain rust-toolchain.toml examples
      bench/pipeline_levers.py bench/team_size.py bench/wikidata.py bench/wikidata.json bench/bundle.sh
      .just/bench.just docs/journal/2026-11-07-pipeline-levers-registration.md)
    [ ! -s "$W/git-status.txt" ] || fail "the checkout has uncommitted changes"
    [ -z "$(git status --porcelain --untracked-files=all -- "${measured[@]}")" ] || fail "untracked files among the measured inputs"
    git cat-file -e "$frozen^{commit}" || fail "fetch the branch: the frozen commit $frozen is missing"
    if ! git diff --quiet "$frozen" HEAD -- "${measured[@]}"; then
      echo "the measured inputs differ from the frozen $frozen" > "$W/NOT-FROZEN.txt"
      fail "HEAD's measured inputs differ from the frozen $frozen"
    fi
    echo "frozen $frozen" > "$W/frozen.txt"
    cargo build -q --release --example ab --example windows
    cargo build -q --release --features trace --example occupancy
    AB=$PWD/target/release/examples/ab
    corpus="{{CORPUS}}"
    if [ -z "$corpus" ]; then
      dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
      if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
        git fetch -q origin bench-data:refs/remotes/origin/bench-data 2>/dev/null \
          && git archive origin/bench-data "$dir" | tar -x || true
      fi
      python3 bench/wikidata.py fetch bench/wikidata.json "$dir" >/dev/null
      python3 bench/wikidata.py build bench/wikidata.json "$dir" target/release/examples/windows "$B/wikidata.bz2"
      corpus=$B/wikidata.bz2
    fi
    ln -sf "$(realpath "$corpus")" "$W/wikidata.bz2"
    sha256sum "$corpus" > "$W/corpus.sha256"
    "$AB" env > "$W/env.json"
    lscpu > "$W/lscpu.txt"
    # one ab run a plan (each lever is compared within its own thread count), appended to one
    # file: 3 × 3 at 10 threads, then 3 × 4 at 12
    for tk in 10:3 12:4; do
      IFS=: read -r t k <<< "$tk"
      if [ "$t" = 10 ]; then arms="mibz2,mibz2-team3"; else arms="mibz2-team4"; fi
      arms="$arms,mibz2-pipe$k,mibz2-pipe$k-spin2k,mibz2-pipe$k-park,mibz2-pipe$k-small,mibz2-pipe$k-both"
      "$AB" run "$W/runs.jsonl" --arms "$arms" --threads "$t" --rounds "$rounds" "${busy[@]}" "$W/wikidata.bz2"
    done
    "$AB" summary "$W/runs.jsonl" --ref mibz2-pipe3 > "$W/summary.txt" || true
    occ=()
    for tkl in 10:3: 10:3:spin2k 10:3:park 10:3:small 10:3:both 12:4: 12:4:spin2k 12:4:park 12:4:small 12:4:both; do
      IFS=: read -r t k l <<< "$tkl"
      lever=(); name="occupancy-t$t-pipe$k"
      if [ -n "$l" ]; then lever=(--levers "$l"); name="$name-$l"; fi
      o="$W/$name.json"
      if target/release/examples/occupancy "$W/wikidata.bz2" --threads "$t" --pipe "$k" "${lever[@]}" \
          --rounds "$trace_rounds" --out "$o" --meta "commit=$(git rev-parse HEAD)" \
          > "$W/$name.txt" 2>&1; then
        occ+=("$o")
      else
        echo "bench-pipeline-levers: the occupancy pass at $t threads, pipe $k ${l:-(no lever)}, failed"
      fi
    done
    status=0
    python3 bench/pipeline_levers.py report "$W/runs.jsonl" --occupancy "${occ[@]}" \
      --out "$W/pipeline-levers" || status=$?
    verdict=$(python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))["verdict"])' "$W/pipeline-levers.json" 2>/dev/null || echo "")
    case "$verdict" in
      VALID) outcome="VALID: see pipeline-levers.md for the registered comparisons' outcomes" ;;
      INVALID) outcome="INVALID: see pipeline-levers.md's validity checks" ;;
      *) outcome="REPORT FAILED (status $status): see console.log" ;;
    esac

# The pipelined default's workstation A/B (docs/journal/2026-11-07-pipelined-default-registration.md,
# registered before any run): this commit's default decoder (pipelined teams with park and
# small at three teams of three) against v0.2.5's (plain teams), pinned by the release
# commit's SHA, at the same requested thread counts, 20 paired rounds on the bench's Wikidata
# sample, judged by bench/pipelined_default.py. One run, as decided. About 10 minutes; it ends
# with one .tar.gz.
#   just submit bench-pipelined-default
# A DRY value is the untimed check of the recipe itself (one round, no settling), with
# optionally a CORPUS, a .bz2 to use instead of building the bench's sample, both positional:
#   just bench-pipelined-default 1 [CORPUS]
# never evidence, and the report never calls it VALID.

# Head default (pipelined at 3 × 3) against v0.2.5's default, 8 to 20 threads
bench-pipelined-default DRY="" CORPUS="":
    #!/usr/bin/env bash
    set -euo pipefail
    # each attempt in a directory of its own, never overwritten
    B=$PWD/target/pipelined-default
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-pipelined-default: $W already exists; not reusing an attempt's directory"; exit 1; }
    exec > >(tee -a "$W/console.log") 2>&1
    # every attempt ends with one stated outcome (outcome.txt) and is packed, however it ends:
    # a refusal, a failed build, a broken report or a full run
    outcome="INCOMPLETE: stopped before the run finished (see console.log)"
    finish() {
      status=$?
      echo "$outcome" > "$W/outcome.txt"
      echo "bench-pipelined-default: OUTCOME: $outcome"
      bash bench/bundle.sh "$W"
      # (VALID PASS and VALID FAIL exit 0, INVALID 1, anything that kept the run from a
      # verdict, a refusal or a report error among them, 2 or its own failing status)
      case "$outcome" in
        "VALID "*) exit 0 ;;
        INVALID*) exit 1 ;;
        *) [ "$status" -ne 0 ] && exit "$status"; exit 2 ;;
      esac
    }
    trap finish EXIT
    fail() { outcome="REFUSED: $*"; echo "bench-pipelined-default: $*"; exit 1; }
    rounds=20; busy=(); dry="{{DRY}}"; corpus="{{CORPUS}}"
    if [ -n "$dry" ]; then
      rounds=1; busy=(--max-busy 64 --settle 0.2)
      echo "bench-pipelined-default: DRY RUN: one round, untimed; not evidence" | tee "$W/DRY-RUN.txt"
    elif [ -n "$corpus" ]; then
      fail "a CORPUS is for a DRY check only; the registered run builds the bench's sample"
    fi
    # the commits the registration froze (bench/pipelined-default.frozen): the candidate, whose
    # build and workload inputs HEAD must match exactly, and the base it is measured against
    frozen=bench/pipelined-default.frozen
    cand=$(awk '$1 == "candidate" {print $2}' "$frozen")
    base=$(awk '$1 == "base" {print $2}' "$frozen")
    [[ $cand =~ ^[0-9a-f]{40}$ && $base =~ ^[0-9a-f]{40}$ ]] || fail "$frozen names no frozen candidate and base yet; nothing to run"
    # (the base is v0.2.5's release commit, by its SHA, never by the tag's name)
    [ "$base" = e46f826379313b7a04d93005c349e2aada203582 ] || fail "$frozen's base $base is not v0.2.5's release commit e46f8263"
    # what the builds and the workload are made from: sources, Cargo's files and configuration,
    # the harness, the corpus's builder and manifest, the judge and this recipe
    # (a path that doesn't exist counts too: one appearing, such as a build.rs or a toolchain
    # file, is a change)
    measured=(src Cargo.toml Cargo.lock .cargo build.rs rust-toolchain rust-toolchain.toml examples bench/pipelined_default.py bench/wikidata.json bench/wikidata.py bench/bundle.sh .just/bench.just
      docs/journal/2026-11-07-pipelined-default-registration.md)
    # no environment that changes how Cargo, rustc or rustup build, or how ab observes: any
    # CARGO_* or RUST* variable but those that only say where they are installed, how Cargo
    # reaches its registry (the lockfile pins what it fetches) or how it prints, and the
    # census's interval, which stays at its registered 5 ms
    overrides=$(env | cut -d= -f1 | grep -E '^(CARGO_|RUST|AB_CENSUS_MS$|AB_SAMPLE_MS$)' | grep -vxE 'CARGO_HOME|RUSTUP_HOME|RUST_BACKTRACE|RUST_LOG|CARGO_(HTTP|NET|TERM)_[A-Z_]+' || true)
    [ -z "$overrides" ] || fail "unset these, so both builds and both arms run the same, registered way: $(echo $overrides)"
    [ -z "$(git status --porcelain --untracked-files=no)" ] || fail "the checkout has uncommitted changes"
    [ -z "$(git status --porcelain --untracked-files=all -- "${measured[@]}")" ] || fail "untracked files among the build and workload inputs"
    git cat-file -e "$cand^{commit}" && git cat-file -e "$base^{commit}" || fail "fetch the branch: the frozen commits are missing"
    git diff --quiet "$cand" HEAD -- "${measured[@]}" || fail "HEAD's build or workload inputs differ from the frozen candidate $cand"
    head=$(git rev-parse HEAD)
    echo "head $head (build and workload inputs as candidate $cand)"; echo "base $base"
    printf 'head %s\ncandidate %s\nbase %s\n' "$head" "$cand" "$base" > "$W/commits.txt"
    [ -d "$B/build-$base" ] || git worktree add -q --detach "$B/build-$base" "$base"
    [ "$(git -C "$B/build-$base" rev-parse HEAD)" = "$base" ] || fail "$B/build-$base is not at the base $base"
    [ -z "$(git -C "$B/build-$base" status --porcelain --untracked-files=all)" ] || fail "$B/build-$base has changes or untracked files"
    # (each executable as Cargo reports building it, from its own checkout, then copied into
    # this attempt's directory: the copies are what run and what the digests describe, so
    # neither build can overwrite the other's, wherever Cargo's target directory is)
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
    # Cargo reads `.cargo/config(.toml)` in every directory above where it builds. The base
    # checkout sits inside this one, so a configuration between the two (in target/, or this
    # checkout's own root) would reach the base build only: refused. Above this checkout both
    # builds read the same files, which inputs.txt records
    configs() { local d=$1 stop=$2; while :; do for f in "$d/.cargo/config" "$d/.cargo/config.toml"; do [ -f "$f" ] && echo "$f"; done; [ "$d" = "$stop" ] && break; d=$(dirname "$d"); done; return 0; }
    only_base=$(configs "$(dirname "$B/build-$base")" "$PWD")
    [ -z "$only_base" ] || fail "Cargo configuration only the base build would read: $(echo $only_base)"
    # the toolchain each build will use, asked in its own checkout (a toolchain file there
    # would change it): they must be the same
    tool() { (cd "$1" && rustc -vV && cargo -V); }
    [ "$(tool "$PWD")" = "$(tool "$B/build-$base")" ] || fail "the two checkouts select different toolchains"
    mkdir "$W/bin"
    # (one build at a time, its status checked here, and its executable copied before the
    # next build starts: a target directory both share can't swap one build's for the other's)
    base_built=$(built "$B/build-$base" ab) || fail "the base build failed"
    cp "$base_built" "$W/bin/ab-base"
    sha256sum "$W/bin/ab-base" > "$W/bin/ab-base.sha256"
    head_built=$(built "$PWD" ab windows) || fail "the head build failed"
    [ "$(printf '%s\n' "$head_built" | wc -l)" = 2 ] || fail "Cargo reported no executables for the head build"
    cp "$(printf '%s\n' "$head_built" | sed -n 1p)" "$W/bin/ab-head"
    cp "$(printf '%s\n' "$head_built" | sed -n 2p)" "$W/bin/windows"
    sha256sum -c --quiet "$W/bin/ab-base.sha256" || fail "the base executable changed after it was copied"
    AB=$W/bin/ab-head
    BASEAB=$W/bin/ab-base
    WINDOWS=$W/bin/windows
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
    # what ran: both executables, the corpus and the toolchain, by digest, and the Cargo
    # configuration outside the repository, which applies to both builds alike
    {
      sha256sum "$AB" "$BASEAB" "$WINDOWS" "$(realpath "$corpus")"
      tool "$PWD"
      # (one path a line, so a directory with a space in its name stays one path)
      { printf '%s\n' "${CARGO_HOME:-$HOME/.cargo}/config" "${CARGO_HOME:-$HOME/.cargo}/config.toml"; configs "$(dirname "$PWD")" /; } \
        | while IFS= read -r f; do if [ -f "$f" ]; then sha256sum "$f"; fi; done
      echo "census every 5 ms (AB_CENSUS_MS unset)"
    } > "$W/inputs.txt"
    cat "$W/inputs.txt"
    "$AB" env > "$W/env.json"
    lscpu > "$W/lscpu.txt"
    "$AB" run "$W/runs.jsonl" --arms "mibz2,mibz2@$BASEAB" \
      --threads "8,9,10,11,12,16,20" --rounds "$rounds" "${busy[@]}" "$W/wikidata.bz2"
    # the verdict: VALID PASS, VALID FAIL or INVALID, from the report's own fields; a report
    # that couldn't be written is said to be so, and points to nothing
    set +e
    python3 bench/pipelined_default.py report "$W/runs.jsonl" --out "$W/pipelined-default"
    rc=$?
    set -e
    if [ "$rc" -le 1 ] && [ -s "$W/pipelined-default.json" ] && [ -s "$W/pipelined-default.md" ]; then
      outcome=$(python3 -c 'import json,sys; v=json.load(open(sys.argv[1])); print(("VALID PASS" if v["pass"] else "VALID FAIL") if v["valid"] else "INVALID: " + "; ".join(v["problems"]))' "$W/pipelined-default.json")
    else
      outcome="REPORT ERROR: pipelined_default.py exited $rc without writing its report"
    fi

# The service core's target preflight, alone (docs/journal/2026-11-09-svc-target-registration.md;
# frozen by bench/svc-target.frozen, which names no commit until the Coordinator freezes it):
# examples/svc_target.rs built as Cargo reports it, run once, its record checked whole by
# bench/svc_target_check.py. It times no decoder (no ab grid) and is no performance evidence.
# Read-only. Run once; no retry. Whatever the outcome, the bundle keeps what was reached, and
# `stages.txt` says which stages were reached and which not (an early refusal has no build or
# record, and none is made up). The preflight is sent TERM at 120 s and KILL 10 s later.
# Its exit: 0 TARGET, 4 NOT TARGET, 5 FAILED (a dispatch or decode check failed), 1 REFUSED or
# INVALID RECORD, 3 INCOMPLETE; kept through the bundle and `just submit`. About a minute once
# built.
#   just submit bench-svc-target

# The service core's target predicate and dispatch, read back on this machine, once
bench-svc-target:
    #!/usr/bin/env bash
    set -euo pipefail
    B=$PWD/target/svc-target
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-svc-target: $W already exists; not reusing an attempt's directory"; exit 1; }
    exec > >(tee -a "$W/console.log") 2>&1
    outcome="INCOMPLETE: stopped before the preflight finished (see console.log)"
    # the stages, in order, each marked as it completes; stages.txt lists every one, reached
    # or not, with the files that stage leaves
    all_stages=("checks: frozen and clean (frozen.txt)" "build (build.txt, inputs.txt)"
      "raw reads (raw/)" "preflight run (preflight.json, preflight.exit)" "record check (preflight-check.json)")
    reached=0
    mark() { reached=$((reached + 1)); }
    finish() {
      echo "$outcome" > "$W/outcome.txt"
      for i in "${!all_stages[@]}"; do
        if [ "$i" -lt "$reached" ]; then echo "reached: ${all_stages[$i]}"; else echo "not reached: ${all_stages[$i]}"; fi
      done > "$W/stages.txt"
      echo "bench-svc-target: OUTCOME: $outcome"
      bash bench/bundle.sh "$W" || { echo "bench-svc-target: bundling failed"; exit 2; }
      case "$outcome" in
        "TARGET"*) exit 0 ;;
        "NOT TARGET"*) exit 4 ;;
        "FAILED"*) exit 5 ;;
        REFUSED*|"INVALID RECORD"*) exit 1 ;;
        *) exit 3 ;;
      esac
    }
    trap finish EXIT
    fail() { outcome="REFUSED: $*"; echo "bench-svc-target: $*"; exit 1; }
    git rev-parse HEAD > "$W/commit.txt"
    git status --porcelain --untracked-files=no > "$W/git-status.txt"
    frozen=$(awk '$1 == "frozen" {print $2}' bench/svc-target.frozen)
    [[ $frozen =~ ^[0-9a-f]{40}$ ]] || fail "bench/svc-target.frozen names no frozen commit"
    measured=(src Cargo.toml Cargo.lock .cargo build.rs rust-toolchain rust-toolchain.toml examples
      tests/common bench/svc_target_check.py bench/bundle.sh .just/bench.just
      docs/journal/2026-11-09-svc-target-registration.md)
    [ ! -s "$W/git-status.txt" ] || fail "the checkout has uncommitted changes"
    [ -z "$(git status --porcelain --untracked-files=all -- "${measured[@]}")" ] || fail "untracked files among the measured inputs"
    git cat-file -e "$frozen^{commit}" || fail "fetch the branch: the frozen commit $frozen is missing"
    if ! git diff --quiet "$frozen" HEAD -- "${measured[@]}"; then
      echo "the measured inputs differ from the frozen $frozen" > "$W/NOT-FROZEN.txt"
      fail "HEAD's measured inputs differ from the frozen $frozen"
    fi
    echo "frozen $frozen" > "$W/frozen.txt"
    mark
    # the executable Cargo reports building, wherever its target directory is; the build's
    # platform is rustc's host (build.txt), the one the predicate's compile-time x86-64 Linux
    # condition is decided for
    EXE=$(cargo build -q --release --example svc_target --message-format=json-render-diagnostics \
      | python3 -c 'import json, sys
    got = [m["executable"] for m in map(json.loads, sys.stdin) if m.get("reason") == "compiler-artifact" and m.get("executable") and m["target"]["name"] == "svc_target" and "example" in m["target"]["kind"]]
    sys.exit("cargo reported no svc_target executable") if len(got) != 1 else print(got[0])') \
      || fail "the build reported no svc_target executable"
    [ -x "$EXE" ] || fail "cargo's reported $EXE is not an executable"
    { echo "executable $EXE"; rustc -vV; cargo -V; uname -srm; } > "$W/build.txt"
    sha256sum "$EXE" | sed "s|$EXE|svc_target|" > "$W/inputs.txt"
    mark
    # the files discovery reads, read again by this shell just before the preflight (the
    # same paths, not the same reads: a diagnostic beside the record, which is what is
    # checked), each value or its read error: the process's status, cgroup membership and
    # mounts, cpuinfo, sysfs's online and present lists, every CPU's core, package and L3
    # files, and every cgroup v2 level's cpu.max and cgroup.type from the process's cgroup up
    # to the mount point
    mkdir "$W/raw"
    lscpu > "$W/raw/lscpu.txt" 2>&1 || true
    python3 - "$W/raw/reads.txt" <<'PY' || echo "the raw reads' script failed" >> "$W/raw/reads.txt"
    import glob, os, sys
    out = open(sys.argv[1], "w")
    def read(p):
        try:
            v = open(p).read()
            out.write(f"== {p}\n{v}" + ("" if v.endswith("\n") else "\n"))
        except OSError as e:
            out.write(f"== {p}\nREAD ERROR: {e.__class__.__name__}: {e}\n")
    for p in ("/proc/self/status", "/proc/self/cgroup", "/proc/self/mountinfo", "/proc/cpuinfo",
              "/sys/devices/system/cpu/online", "/sys/devices/system/cpu/present"):
        read(p)
    for d in sorted(glob.glob("/sys/devices/system/cpu/cpu[0-9]*"), key=lambda d: int(d.rsplit("cpu", 1)[1])):
        for f in ("topology/core_id", "topology/physical_package_id", "cache/index3/level", "cache/index3/size"):
            read(f"{d}/{f}")
    try:
        path = next(l[3:].strip() for l in open("/proc/self/cgroup") if l.startswith("0::"))
        mount = next(l.split(" - ")[0].split()[4] for l in open("/proc/self/mountinfo")
                     if " - " in l and l.split(" - ")[1].split()[0] == "cgroup2")
    except (OSError, StopIteration) as e:
        out.write(f"== the cgroup v2 levels\nNOT RESOLVED: {e.__class__.__name__}: {e}\n")
    else:
        d = os.path.normpath(mount + path)
        while True:
            for f in ("cpu.max", "cgroup.type"):
                read(f"{d}/{f}")
            if d == os.path.normpath(mount) or d == "/":
                break
            d = os.path.dirname(d)
    PY
    mark
    # once, bounded: TERM at 120 s, KILL 10 s after (so at most 130 s)
    status=0
    timeout --kill-after=10 120 "$EXE" > "$W/preflight.json" || status=$?
    echo "$status" > "$W/preflight.exit"
    [ "$(sha256sum "$EXE" | cut -d' ' -f1)" = "$(cut -d' ' -f1 "$W/inputs.txt")" ] \
      || { outcome="INCOMPLETE: the executable changed while it ran"; exit 3; }
    cat "$W/preflight.json"
    case "$status" in
      0|3) ;;
      124) outcome="INCOMPLETE: the preflight passed its 120 s bound and ended on TERM"; exit 3 ;;
      137) outcome="INCOMPLETE: the preflight ignored TERM at 120 s and was killed at 130 s"; exit 3 ;;
      *) outcome="INCOMPLETE: the preflight exited $status (see console.log)"; exit 3 ;;
    esac
    mark
    set +e
    python3 bench/svc_target_check.py "$W/preflight.json" --exit "$status" --out "$W/preflight-check.json"
    set -e
    verdict=$(python3 -c 'import json, sys; v = json.load(open(sys.argv[1])); print(v["verdict"] + ("" if not v["problems"] else ": " + "; ".join(v["problems"])))' "$W/preflight-check.json" 2>/dev/null || echo "")
    case "$verdict" in
      "") outcome="INCOMPLETE: the record's check wrote nothing" ;;
      *) mark; outcome="$verdict (see preflight.json)" ;;
    esac

# The scan without the scanner, on the workstation (docs/journal/2026-11-10-parallel-scan.md,
# section 5, registered and frozen before use): a research harness's two arms, one reading every
# block's symbols on a scanner thread and one finding blocks by magic and reading them on the
# workers, at 1 to 10 workers on three inputs. A diagnostic (RULES C1): it qualifies no
# production change. Run once; no retry. Its exit: 0 VALID (whatever the readings), 1 INVALID
# or REFUSED, 3 INCOMPLETE, kept through the bundle and `just submit`. About five minutes once
# built. The corpus's preparation, the build, the fault tests, the inputs' generation and
# anchoring, the checks, the cells, the reports and the bundle each run under a deadline in a
# process group of their own (bench/bounded.py): past it, the whole group gets TERM, then KILL
# 30 s later whatever its leader's state, and the group must be empty before the recipe goes
# on. A step past its deadline, or whose group can't be stopped, makes the run INVALID; what it
# wrote is bundled. The small queries (git, lscpu, taskset, reading /proc) are not bounded.
# The steps' deadlines sum to 18,600 s (5 h 10 min) before the graces, cleanup and the small
# queries: a ceiling, not an expected duration, and not a bound on the whole run. A bundle
# that fails or passes its deadline leaves the outcome INVALID and no .tar.gz to submit
# (bench/par_scan_finish.sh; exit 2).
#   just submit bench-par-scan
# A DRY value is the untimed check of the recipe itself: one round, and a check that the
# deadline stops a stalled child's group, a grandchild that ignores TERM under a parent that
# exits on it included. Never evidence; it never reads VALID:
#   just bench-par-scan 1

# Blocks found by magic and read on the workers, against one scanner thread, at 1 to 10 workers
bench-par-scan DRY="":
    #!/usr/bin/env bash
    set -euo pipefail
    # each attempt in a directory of its own, never overwritten
    B=$PWD/target/par-scan
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-par-scan: $W already exists; not reusing an attempt's directory"; exit 1; }
    exec > >(tee -a "$W/console.log") 2>&1
    # a step in a process group of its own, under a deadline of SECS: the group gets TERM at
    # SECS and KILL 30 s later, and must then be empty. Exit 124 past the deadline, 125 if the
    # group could not be emptied, else the step's own status. Its output stays where it was sent
    bounded() { local secs=$1; shift; python3 bench/bounded.py "$secs" 30 "$@"; }
    deadline() { [ "$1" -eq 124 ] || [ "$1" -eq 125 ]; }
    # the outcome decides the exit, whatever the last command's status
    outcome="INCOMPLETE: stopped before the run finished (see console.log)"
    # finalisation (bench/par_scan_finish.sh): the outcome written, then the bundle under its
    # deadline. If the bundle fails or is late, the outcome is rewritten INVALID, no .tar.gz is
    # left for `just submit`, and the directory is kept: exit 2
    finish() {
      echo "bench-par-scan: OUTCOME: $outcome"
      rm -rf "$W/inputs"
      local fst=0
      bash bench/par_scan_finish.sh "$W" "$outcome" 600 bash bench/bundle.sh "$W" || fst=$?
      outcome=$(cat "$W/outcome.txt")
      [ "$fst" -eq 0 ] || { echo "bench-par-scan: OUTCOME: $outcome"; exit 2; }
      case "$outcome" in
        VALID*|DRY*) exit 0 ;;
        INVALID*|REFUSED*) exit 1 ;;
        *) exit 3 ;;
      esac
    }
    trap finish EXIT
    fail() { outcome="REFUSED: $*"; echo "bench-par-scan: $*"; exit 1; }
    # a step that passed its deadline: the registered INVALID, whatever step it was
    # (124: past its deadline; 125: its process group could not be stopped and reaped)
    late() { outcome="INVALID: $1 passed its deadline or could not be stopped (exit $st; $2)"; echo "bench-par-scan: $outcome"; exit 1; }
    ROUNDS=5
    WORKERS=1,2,4,6,8,10
    dry="{{DRY}}"
    if [ -n "$dry" ]; then
      ROUNDS=1
      echo "bench-par-scan: DRY RUN: one round, untimed; not evidence" | tee "$W/DRY-RUN.txt"
    fi
    # (the harness's fault injection is the dry run's alone, and only where it sets it below)
    [ -z "${PAR_SCAN_INJECT:-}" ] || fail "PAR_SCAN_INJECT is set ($PAR_SCAN_INJECT): a measured run takes no injected fault"
    unset PAR_SCAN_INJECT
    # a cell run's failure, classified: nothing if it succeeded, else the registered INVALID
    knee_outcome() {
      local st=$1 name=$2 log=$3
      [ "$st" -eq 0 ] && return 0
      if deadline "$st"; then echo "INVALID: the harness on $name passed its deadline or could not be stopped (exit $st; $log)"
      else echo "INVALID: the harness failed on $name ($log)"; fi
    }
    for tool in python3 git cargo; do
      command -v "$tool" > /dev/null || fail "needs $tool on PATH"
    done
    git rev-parse HEAD > "$W/commit.txt"
    printf 'head %s\n' "$(cat "$W/commit.txt")" > "$W/commits.txt"
    git status --porcelain --untracked-files=no > "$W/git-status.txt"
    # the registration's frozen commit (bench/par-scan.frozen): what is built and driven must
    # be as it was there, and committed
    frozen=$(awk '$1 == "frozen" {print $2}' bench/par-scan.frozen)
    [[ $frozen =~ ^[0-9a-f]{40}$ ]] || fail "bench/par-scan.frozen names no frozen commit"
    measured=(src Cargo.toml Cargo.lock .cargo build.rs rust-toolchain rust-toolchain.toml
      examples/par_scan.rs examples/ab.rs tests/common bench/bundle.sh bench/corpus.sh
      bench/wikidata.json bench/wikidata.py bench/par_scan_knee.py bench/par_scan_inputs.py
      bench/bounded.py bench/par_scan_finish.sh .just/bench.just
      docs/journal/2026-11-10-parallel-scan.md)
    [ ! -s "$W/git-status.txt" ] || fail "the checkout has uncommitted changes"
    [ -z "$(git status --porcelain --untracked-files=all -- "${measured[@]}")" ] || fail "untracked files among the measured inputs"
    git cat-file -e "$frozen^{commit}" || fail "fetch the branch: the frozen commit $frozen is missing"
    if ! git diff --quiet "$frozen" HEAD -- "${measured[@]}"; then
      echo "the measured inputs differ from the frozen $frozen" > "$W/NOT-FROZEN.txt"
      fail "HEAD's measured inputs differ from the frozen $frozen"
    fi
    echo "frozen $frozen" > "$W/frozen.txt"
    if [ -n "$dry" ]; then
      # the deadline itself, at 2 s and a 2 s grace: a stalled parent that exits on TERM, with
      # a stalled grandchild that answers TERM, then one that ignores it. Both stopped, and
      # the supervisor returns only once the group is empty
      alive() { local s; s=$(awk '{print $3}' "/proc/$1/stat" 2> /dev/null) && [ "$s" != Z ] && [ "$s" != X ]; }
      for trap in "" 'trap "" TERM;'; do
        what=${trap:+a grandchild that ignores TERM}; what=${what:-a grandchild that answers TERM}
        rm -f "$W/stall-pids.txt"
        st=0
        python3 bench/bounded.py 2 2 bash -c "echo \$\$ > \"\$1\"; ($trap exec sleep 600) & echo \$! >> \"\$1\"; wait" _ "$W/stall-pids.txt" || st=$?
        [ "$st" -eq 124 ] || fail "DRY: the deadline did not stop a stalled child with $what (exit $st)"
        for p in $(cat "$W/stall-pids.txt"); do
          ! alive "$p" || { kill -9 "$p" 2> /dev/null; fail "DRY: process $p ($what) outlived the deadline"; }
        done
        echo "DRY: a stalled child and $what: both stopped at the deadline (exit $st)" | tee -a "$W/stall-check.txt"
      done
    fi
    st=0; bounded 3600 bash bench/corpus.sh || st=$?
    deadline "$st" && late "the corpus's preparation" "console.log"
    [ "$st" -eq 0 ] || { outcome="INCOMPLETE: bench/corpus.sh failed (exit $st)"; exit 1; }
    lscpu > "$W/lscpu.txt"
    # where the run may execute: the harness pins nothing, so every thread inherits this. At
    # t workers a cell is t + 2 threads (the workers, the finder or scanner, and the caller,
    # which is the joiner): 10 workers are 12 threads sharing these CPUs. cpu.max is the file
    # at the mounted path, not necessarily the effective quota of a nested cgroup
    {
      echo "nproc $(nproc)"
      echo "shell affinity: $(taskset -cp $$ 2> /dev/null || grep Cpus_allowed_list /proc/self/status)"
      grep -E '^Cpus_allowed_list' /proc/self/status
      echo "cgroup: $(cat /proc/self/cgroup 2> /dev/null | tr '\n' ' ')"
      echo "cgroup cpu.max (the mounted path's file): $(cat /sys/fs/cgroup/cpu.max 2> /dev/null || echo none)"
      lscpu -e=CPU,CORE,SOCKET,NODE,ONLINE 2> /dev/null || true
    } > "$W/placement.txt"
    cat /proc/loadavg > "$W/loadavg-before.txt"
    st=0; bounded 3600 cargo build --release --example par_scan --example ab || st=$?
    deadline "$st" && late "the build" "console.log"
    [ "$st" -eq 0 ] || { outcome="INCOMPLETE: the build failed (exit $st)"; exit 1; }
    P=target/release/examples/par_scan
    # failure propagation: a panic at each role of each arm ends the decode, never hangs it
    st=0; bounded 300 "$P" - faults > "$W/faults.txt" 2>&1 || st=$?
    cat "$W/faults.txt"
    deadline "$st" && late "the fault tests" "faults.txt"
    [ "$st" -eq 0 ] || { outcome="INVALID: the fault tests failed (faults.txt)"; exit 1; }
    # the inputs: the corpus; its bytes at level 1 (Python's bz2, libbzip2); the crate's
    # records. Each with a decoded size recorded apart from the run, and checked against
    # libbzip2's decode: the manifest's for the corpus and the level-1 file, whose decode must
    # also have the corpus's decoded SHA-256; the generator's for the records
    I=$W/inputs
    mkdir "$I"
    st=0; bounded 600 target/release/examples/ab gen "$I/gen" 8 > "$W/gen.txt" 2>&1 || st=$?
    deadline "$st" && late "the inputs' generation" "gen.txt"
    [ "$st" -eq 0 ] || fail "ab gen failed (gen.txt)"
    st=0
    bounded 900 python3 bench/par_scan_inputs.py bench/wikidata.json corpora/wikidata.bz2 \
      "$W/gen.txt" "$I/gen/records.bz2" "$I" > "$W/inputs.json" || st=$?
    deadline "$st" && late "the inputs' anchoring" "console.log"
    [ "$st" -eq 0 ] || fail "the inputs are not as recorded (console.log)"
    cat "$W/inputs.json"
    if [ -n "$dry" ]; then
      # a failed getrusage in a cell (injected: the harness's stand-in that returns -1 and
      # writes nothing): the cell panics, no row is written, and the cells' classification
      # below makes it INVALID; its log is kept in the attempt
      st=0
      PAR_SCAN_INJECT=getrusage-fails bounded 300 "$P" "$I/records.bz2" knee 1 1 \
        > "$W/injected-getrusage.jsonl" 2> "$W/injected-getrusage.log" || st=$?
      o=$(knee_outcome "$st" "records, with an injected getrusage failure" injected-getrusage.log)
      case "$o" in INVALID*) ;; *) fail "DRY: an injected getrusage failure was not INVALID (exit $st)" ;; esac
      grep -q 'getrusage failed' "$W/injected-getrusage.log" || fail "DRY: the injected failure's panic is not in its log"
      [ ! -s "$W/injected-getrusage.jsonl" ] || fail "DRY: a cell was recorded despite a failed getrusage"
      echo "DRY: an injected getrusage failure: exit $st, no cell recorded, classified: $o" | tee "$W/injected-getrusage.txt"
    fi
    expected() { python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["inputs"][sys.argv[2]]["expected_bytes"])' "$W/inputs.json" "$1"; }
    readings=""
    for name in wikidata wikidata-l1 records; do
      # every arm's bytes equal to mibz2::decode's, and those to libbzip2's, or nothing is
      # timed (and the warm-up)
      st=0; bounded 900 "$P" "$I/$name.bz2" check > "$W/check-$name.txt" 2>&1 || st=$?
      if [ "$st" -ne 0 ]; then
        tail -5 "$W/check-$name.txt"
        deadline "$st" && late "the check on $name" "check-$name.txt"
        outcome="INVALID: the harness's bytes differ from mibz2::decode's or libbzip2's on $name (check-$name.txt)"
        exit 1
      fi
      st=0; bounded 1800 "$P" "$I/$name.bz2" knee "$ROUNDS" "$WORKERS" > "$W/runs-$name.jsonl" 2> "$W/harness-$name.log" || st=$?
      if [ "$st" -ne 0 ]; then
        tail -5 "$W/harness-$name.log"
        outcome=$(knee_outcome "$st" "$name" "harness-$name.log")
        echo "bench-par-scan: $outcome"
        exit 1
      fi
      st=0
      bounded 300 python3 bench/par_scan_knee.py "$W/runs-$name.jsonl" "$ROUNDS" "$WORKERS" "$W/report-$name.json" "$(expected "$name")" > "$W/summary-$name.md" || st=$?
      if [ "$st" -ne 0 ]; then
        deadline "$st" && late "the report on $name" "console.log"
        outcome="INVALID: runs-$name.jsonl is not the registered grid, or not the expected size"
        exit 1
      fi
      echo "## $name"; cat "$W/summary-$name.md"
      readings="$readings $name: $(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["reading"].split(":")[0])' "$W/report-$name.json");"
    done
    cat /proc/loadavg > "$W/loadavg-after.txt"
    if [ -n "$dry" ]; then
      outcome="DRY: the recipe ran end to end, one round, untimed; not evidence;$readings"
    else
      outcome="VALID:$readings"
    fi

# The cooperative ports (#335, #337, #339 stacked) against their base, on the i9's team path:
# wikidata wall at 8-20 threads. Decision rule set before the run: #145 LOG 6764355047.
bench-team-ports:
    git fetch -q origin research/team-mtf-port
    just bench-release-scaling f76f0e3cdeec9260179eacd5e1a111a5a434bec5 8f540ef1190689f59a279f081d8405f46669fba0 10 "8,10,12,16,20"

# #324 (run-record front) against its base on the i9, wikidata wall at 1-20 threads, the team path included.
# Pathfinder W2 (#145 PROPOSAL 6764488488). Decision: PERFORMANCE-POLICY.md, primary corpus.
bench-324-i9:
    git fetch -q origin perf/run-front-records
    just bench-release-scaling e88c063155c3e734e748e99ccda025393302720f 5e7b995471591eacdd5ae32cebfb8bbb578cbff5 10

# #359 (the team-runs port) against its base on the i9: wall at 6-20 threads on wikidata, and on
# text, records and random from `ab gen` at scale 16 (the CI bench's corpora, four times its scale,
# compressed at level 9). Pathfinder PROPOSAL 6771355497; the contrast frozen there (its base's
# source is master 910472e8's). Rule, written before the data: the note's §8
# (docs/journal/2026-11-10-team-runs-port.md, on #313 at 37268459).
bench-team-runs:
    #!/usr/bin/env bash
    set -euo pipefail
    git fetch -q origin perf/team-runs
    G=$PWD/target/team-runs-corpora
    rm -rf "$G"
    cargo build -q --release --example ab
    target/release/examples/ab gen "$G" 16
    just bench-release-scaling 58e0a4be697f740b339eae1089c40b567a9f6e93 ba52e4d6f05729b6317ac8b7b37e576d79b193d3 10 "6,8,10,12,16,20" "$G/text.bz2" "$G/records.bz2" "$G/random.bz2"

# Does 0.2.9's faster team path move the workers-to-teams crossover below 6 threads? The plan
# (mibz2: workers below 6, teams from 6) against forced teams of 2, 3 and 4, at 4, 5, 6 and 8
# threads on the i9, paired by round. Rule, written before the data: LOG on #145 (team threshold).
bench-team-threshold:
    just bench-team-size 20 "4,5,6,8"

# MONODROMY's cells kernel against production's records path on the i9 (rule: LOG on #145):
# the rauzy census on the bench's Wikidata sample, one thread pinned to core 2, five rounds a
# block (fastest kept), three repeats. Both clocks read the same run records; `ns_rec` is
# production's inversion stage since 0.2.8. Indicative of the kernel stage, not the decoder.
#   just bench-cells-vs-prod

# Cells kernel vs production's records path, 1 thread, 3 repeats
bench-cells-vs-prod:
    #!/usr/bin/env bash
    set -euo pipefail
    B=$PWD/target/cells-vs-prod
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-cells-vs-prod: $W already exists; not reusing an attempt's directory"; exit 1; }
    exec > >(tee -a "$W/console.log") 2>&1
    cargo build -q --release --example rauzy --example windows
    dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
    if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
      git fetch -q origin bench-data:refs/remotes/origin/bench-data 2>/dev/null \
        && git archive origin/bench-data "$dir" | tar -x || true
    fi
    python3 bench/wikidata.py fetch bench/wikidata.json "$dir" >/dev/null
    python3 bench/wikidata.py build bench/wikidata.json "$dir" target/release/examples/windows "$B/wikidata.bz2"
    ln -sf "$B/wikidata.bz2" "$W/wikidata.bz2"
    lscpu > "$W/lscpu.txt"
    git rev-parse HEAD > "$W/commit.txt"
    for rep in 1 2 3; do
      taskset -c 2 target/release/examples/rauzy "$B/wikidata.bz2" 1 65536 5 > "$W/census-$rep.jsonl"
    done
    python3 bench/cells_vs_prod.py "$W/summary.json" "$W"/census-*.jsonl | tee "$W/summary.txt"
    bash bench/bundle.sh "$W"

# #361: i9 one-core inlining diagnostic, frozen before data.
# Exact contrast and decision rule: docs/journal/2026-11-10-onecore-inline-i9.md.
# Existing release-scaling harness, 10 paired rounds, Wikidata, one thread.
bench-onecore-inline-i9:
    git fetch -q origin perf/onecore-walk-inline
    just bench-release-scaling d33787e138426cb441db6975f54b2eccca02e89a b65b2aa54488f072cf005b8fe62461c32e6ef083 10 "1"


# Prepared cells live-path diagnostic; separate Coordinator admission still required.
# Ten balanced paired rounds, three corpora, original readers × trace off/on.
[positional-arguments]
bench-cells-live-path cpu corpora:
    bash bench/cells-gap/live/run.sh "$1" "$2"

# The 0.3.0 candidate (Cells on the calling thread and in workers, behind the 64r < n gate)
# against master on the i9 at every thread count, with production's plan (mibz2) and with
# forced workers (mibz2-workers) in both builds: is the candidate faster where it changes
# the decoder (below 6 threads), level where it does not (teams, from 6), and do workers
# running Cells beat teams at 10 and 20 threads? Rule, written before the data: LOG on #145
# (cells scaling). Wikidata, text and pattern, paired by round. About 15 minutes.
#   just submit bench-cells-scaling
bench-cells-scaling OLD="6dec95ba6aaf59d5134e5ab119fd52433e2032ee" ROUNDS="10" THREADS="1,2,4,6,8,10,20":
    #!/usr/bin/env bash
    set -euo pipefail
    B=$PWD/target/cells-scaling
    mkdir -p "$B"
    O=$B/run-$(date -u +%Y%m%dT%H%M%SZ)
    mkdir "$O"
    exec > >(tee -a "$O/console.log") 2>&1
    echo "bench-cells-scaling: $(git rev-parse HEAD) against {{OLD}}"
    git rev-parse HEAD > "$O/new.txt"
    echo "{{OLD}}" > "$O/old.txt"
    cargo build -q --release --example ab --example windows
    NEWAB=$PWD/target/release/examples/ab
    [ -d "$B/build-{{OLD}}" ] || git worktree add -q --detach "$B/build-{{OLD}}" "{{OLD}}"
    (cd "$B/build-{{OLD}}" && cargo build -q --release --example ab)
    OLDAB=$B/build-{{OLD}}/target/release/examples/ab
    dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
    if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
      git fetch -q origin bench-data:refs/remotes/origin/bench-data 2>/dev/null \
        && git archive origin/bench-data "$dir" | tar -x || true
    fi
    python3 bench/wikidata.py fetch bench/wikidata.json "$dir" >/dev/null
    [ -f "$B/wikidata.bz2" ] || python3 bench/wikidata.py build bench/wikidata.json "$dir" \
      target/release/examples/windows "$B/wikidata.bz2"
    [ -f "$B/gen/text.bz2" ] || "$NEWAB" gen "$B/gen" 16
    sha256sum "$B/wikidata.bz2" "$B/gen/text.bz2" "$B/gen/pattern.bz2" | tee "$O/inputs.sha256"
    "$NEWAB" env > "$O/env.json"
    lscpu > "$O/lscpu.txt"
    hold=(); hold_cpu=${HOLD_CPU:-auto}
    if [ "$hold_cpu" != off ]; then hold=(--hold-cpu "$hold_cpu" --hold-max-threads 10); fi
    "$NEWAB" run "$O/runs.jsonl" \
      --arms "mibz2,mibz2@$OLDAB,mibz2-workers,mibz2-workers@$OLDAB" \
      --threads "{{THREADS}}" --rounds {{ROUNDS}} --spread "${hold[@]}" \
      "$B/wikidata.bz2" "$B/gen/text.bz2" "$B/gen/pattern.bz2"
    "$NEWAB" summary "$O/runs.jsonl" --ref "mibz2@$OLDAB" | tee "$O/summary.txt"
    bash bench/bundle.sh "$O"
