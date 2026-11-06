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
# bench/release-scaling/<date>-<machine>/ for a pull request. Needs lbzip2.
#   just bench-release-scaling
#   just bench-release-scaling v0.2.4 v0.2.1 10 "1,4,10,20" /path/to/big.bz2

# Two releases and lbzip2 on this machine, up to its full thread count
bench-release-scaling NEW="v0.2.4" OLD="v0.2.1" ROUNDS="10" THREADS="" *FILES="":
    #!/usr/bin/env bash
    set -euo pipefail
    W=$PWD/target/release-scaling
    mkdir -p "$W"
    O=$W/run-$(date -u +%Y%m%dT%H%M%SZ)
    mkdir "$O"
    exec > >(tee -a "$O/console.log") 2>&1
    git fetch -q --tags origin
    for v in {{NEW}} {{OLD}}; do
      [ -d "$W/build-$v" ] || git worktree add -q --detach "$W/build-$v" "$v"
      (cd "$W/build-$v" && cargo build -q --release --example ab --example windows)
    done
    NEWAB=$W/build-{{NEW}}/target/release/examples/ab
    OLDAB=$W/build-{{OLD}}/target/release/examples/ab
    dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
    if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
      git fetch -q origin bench-data:refs/remotes/origin/bench-data 2>/dev/null \
        && git archive origin/bench-data "$dir" | tar -x || true
    fi
    python3 bench/wikidata.py fetch bench/wikidata.json "$dir" >/dev/null
    python3 bench/wikidata.py build bench/wikidata.json "$dir" "$W/build-{{NEW}}/target/release/examples/windows" "$W/wikidata.bz2"
    threads="{{THREADS}}"
    [ -n "$threads" ] || threads=$(python3 -c "import os; n = os.cpu_count(); print(','.join(map(str, sorted(t for t in {1, 2, 4, 8, 12, 16, n // 2, n} if t <= n))))")
    "$NEWAB" env > "$W/env.json"
    lscpu > "$W/lscpu.txt"
    rm -f "$W/runs.jsonl"
    "$NEWAB" run "$W/runs.jsonl" --arms "mibz2,mibz2@$OLDAB,lbzip2" \
      --threads "$threads" --rounds {{ROUNDS}} --spread "$W/wikidata.bz2" {{FILES}}
    "$NEWAB" summary "$W/runs.jsonl" --ref mibz2 | tee "$W/summary.txt"
    lb=$(lbzip2 --version 2>&1 | head -1 | grep -oE '[0-9]+(\.[0-9]+)+' || true)
    files=(--file "wikidata=$W/wikidata.bz2")
    for f in {{FILES}}; do files+=(--file "$(basename "$f" .bz2)=$f"); done
    python3 bench/release_scaling.py record bench/release-scaling --runs "$W/runs.jsonl" \
      --env "$W/env.json" --lscpu "$W/lscpu.txt" --ref mibz2 \
      --arm "mibz2={{trim_start_match(NEW, 'v')}}" --arm "mibz2@$OLDAB={{trim_start_match(OLD, 'v')}}" \
      --arm "lbzip2=lbzip2 $lb" "${files[@]}"
    cp "$W/runs.jsonl" "$W/env.json" "$W/lscpu.txt" "$W/summary.txt" "$O/"
    cp -r "bench/release-scaling/$(ls -t bench/release-scaling | head -1)" "$O/recorded"
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
    git fetch -q origin master
    head=$(git rev-parse HEAD)
    base=$(git merge-base HEAD origin/master)
    if [ "$head" = "$base" ]; then echo "bench-three-team-policy: HEAD is its own merge base; check out the policy branch"; exit 1; fi
    echo "head $head"; echo "base $base"
    printf 'head %s\nbase %s\n' "$head" "$base" > "$W/commits.txt"
    [ -d "$B/build-$base" ] || git worktree add -q --detach "$B/build-$base" "$base"
    (cd "$B/build-$base" && cargo build -q --release --example ab)
    cargo build -q --release --example ab --example windows
    AB=$PWD/target/release/examples/ab
    BASEAB=$B/build-$base/target/release/examples/ab
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
    "$AB" run "$W/runs.jsonl" --arms "mibz2,mibz2@$BASEAB" \
      --threads "8,9,10,11,12,16,20" --rounds 20 "$W/wikidata.bz2"
    python3 bench/policy_ab.py report "$W/runs.jsonl" --out "$W/policy-ab" \
      || echo "bench-three-team-policy: the run is not valid; see policy-ab.md"
    bash bench/bundle.sh "$W"
