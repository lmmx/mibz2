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

# The compact30 traversal comparison (docs/journal/2026-11-02-compact30-traversal-protocol.md,
# frozen at the pin below): the shipped walk (P) against the compact30 walk (C), on one core
# and on four in lockstep, 20 paired rounds and an untimed round -1, on the primary corpus
# (checked by its hash). It runs the frozen commit itself, in its own worktree
# (target/c30-traversal-pin), built with the reviewed toolchain, rustc 1.97.0 (LLVM 22.1.6),
# selected through rustup without touching the default (install it once with
# `rustup toolchain install 1.97.0 --profile minimal`), and stops before any timing if that
# toolchain is missing or not the reviewed build, if the analysis's tests fail, or if
# room::<false> is not the reviewed listing. The workers go on the second to fifth physical
# cores (the first takes more interrupts), or the first four where there are only four.
# No sudo, no counters, no setting changed. Every file goes to
# target/c30-traversal-<UTC time>[-mode]/, the console included, archived as
# target/c30-traversal-<UTC time>[-mode].tar.gz with a manifest, whether the run passed or
# failed: upload that one file.
# The third argument is the mode:
# - capture: the registered run;
# - dry-run: the whole path untimed (rounds -1, 0 and 1, the records marked a dry run, no
#   timing figure read);
# - fail-output, fail-pin: a dry run with a fault injected in round -1 (W4 worker 2's first
#   C output corrupted after the completion barrier, or W4 worker 1 pinned to a CPU that
#   does not exist), to show the run ends at once, with its failure recorded and archived.
#   just bench-c30-traversal
bench-c30-traversal corpora="target/walk-decoder/work/corpora" rounds="20" run="capture":
    #!/usr/bin/env bash
    set -uo pipefail
    pin=f1dafa31d88d7351a98afb005a84f23d5a2312bd
    # the reviewed toolchain (the walker journal's section 8): selected with rustup run, so the
    # machine's default and any rust-toolchain file are left alone; rustflags forced empty
    tc=1.97.0
    tc_commit=2d8144b7880597b6e6d3dfd63a9a9efae3f533d3
    tc_llvm=22.1.6
    case "{{run}}" in
      capture) flags=() ;;
      dry-run) flags=(--dry-run) ;;
      fail-output) flags=(--dry-run --inject output:w4:C:2) ;;
      fail-pin) flags=(--dry-run --inject pin:w4:1) ;;
      *) echo "c30: the mode is capture, dry-run, fail-output or fail-pin"; exit 2 ;;
    esac
    f=$(realpath "{{corpora}}")/wikidata-20261026.bz2
    wt=$(realpath -m target/c30-traversal-pin)
    suffix=$([ "{{run}}" = capture ] || echo "-{{run}}")
    out=$(realpath -m target)/c30-traversal-$(date -u +%Y%m%dT%H%M%SZ)$suffix
    mkdir -p "$out"
    (
      set -e
      echo "cf13f4394e4ea0a38a0e9e3bc5271b2dce717c8f98fe3a0b8a5186c84216b8fe  $f" | sha256sum -c -
      git cat-file -e "$pin^{commit}" 2> /dev/null || git fetch -q origin research/compact30-walker
      [ -d "$wt" ] || git worktree add -q --detach "$wt" "$pin"
      git -C "$wt" diff --quiet HEAD || { echo "c30: $wt has local changes"; exit 1; }
      if [ "$(git -C "$wt" rev-parse HEAD)" != "$pin" ]; then
        # an earlier pin's worktree, unchanged: moved to this pin (its build directory is kept)
        echo "c30: $wt moved from $(git -C "$wt" rev-parse --short HEAD) to the pin"
        git -C "$wt" checkout -q --detach "$pin"
      fi
      [ "$(git -C "$wt" rev-parse HEAD)" = "$pin" ] || { echo "c30: $wt is not at the pin"; exit 1; }
      echo "c30: the frozen commit $pin, corpus checked, mode {{run}}"
      if ! command -v rustup > /dev/null; then
        echo "c30: rustup is needed to select the reviewed toolchain, rustc $tc; install rustup, then: rustup toolchain install $tc --profile minimal"
        exit 1
      fi
      if ! rustup toolchain list | grep -q "^$tc-"; then
        echo "c30: the reviewed toolchain, rustc $tc (LLVM $tc_llvm), is not installed. Install it beside your default, which stays as it is:"
        echo "      rustup toolchain install $tc --profile minimal"
        exit 1
      fi
      RUSTUP_AUTO_INSTALL=0 rustup run "$tc" rustc -vV | tee "$out/rustc-vV.txt"
      grep -qx "release: $tc" "$out/rustc-vV.txt" && grep -qx "commit-hash: $tc_commit" "$out/rustc-vV.txt" && grep -qx "LLVM version: $tc_llvm" "$out/rustc-vV.txt" \
        || { echo "c30: rustup's $tc is not the reviewed build (want commit $tc_commit, LLVM $tc_llvm); no run"; exit 1; }
      RUSTUP_AUTO_INSTALL=0 rustup run "$tc" cargo -V | tee -a "$out/rustc-vV.txt"
      echo "c30: building with the reviewed toolchain rustc $tc, LLVM $tc_llvm, no rustflags"
      (cd "$wt" && CARGO_TARGET_DIR="$wt/target/rust-$tc" CARGO_ENCODED_RUSTFLAGS= RUSTUP_AUTO_INSTALL=0 rustup run "$tc" cargo build --locked --release --example c30walk)
      bin="$wt/target/rust-$tc/release/examples/c30walk"
      python3 "$wt/bench/test_c30_traversal.py" | tail -1
      python3 "$wt/bench/test_asm_listing.py" | tail -1
      listing=$(python3 "$wt/bench/asm_listing.py" "$bin" "$wt/docs/journal/2026-11-02-compact30-walker/room-false.asm" "$out")
      echo "c30: room::<false> is the reviewed listing: $listing"
      [ "$listing" = yes ] || { echo "c30: G3 fails (see room-false.diff); no run"; exit 1; }
      phys=$(for c in $(taskset -pc $$ | sed 's/.*: //' | tr ',' '\n' | while IFS=- read a b; do seq "$a" "${b:-$a}"; done); do
        cut -d, -f1 /sys/devices/system/cpu/cpu$c/topology/thread_siblings_list | cut -d- -f1
      done | sort -nu)
      n=$(echo "$phys" | wc -l)
      [ "$n" -ge 4 ] || { echo "c30: $n physical cores; four are needed"; exit 1; }
      if [ "$n" -ge 5 ]; then cpus=$(echo "$phys" | sed -n 2,5p | paste -sd,); else cpus=$(echo "$phys" | head -4 | paste -sd,); fi
      echo "c30: workers on CPUs $cpus (W1 on the first)"
      # revision 5: the spinning start barrier's own checks, untimed, on the capture's CPUs
      "$bin" barrier-test --cpus "$cpus" | tee "$out/barrier-test.txt"
      g=0
      timeout --kill-after=30 3600 "$bin" grid "$out/c30-traversal.jsonl" "$f" --rounds {{rounds}} --cpus "$cpus" "${flags[@]}" || g=$?
      echo "c30: the grid exited with status $g"
      a=0
      python3 "$wt/bench/c30_traversal.py" "$out/c30-traversal.jsonl" --listing-ok "$listing" --json "$out/c30-analysis.json" || a=$?
      [ "$g" = 0 ] || exit "$g"
      exit "$a"
    ) 2>&1 | tee "$out/console.log"
    status=${PIPESTATUS[0]}
    cmd="just bench-c30-traversal {{corpora}} {{rounds}} {{run}}"
    if [ -d "$wt" ]; then
      (cd "$wt" && python3 bench/archive.py "$out" --corpus "$f" --binary "$wt/target/rust-$tc/release/examples/c30walk" --command "$cmd" --status "$status")
    else
      python3 bench/archive.py "$out" --corpus "$f" --command "$cmd" --status "$status"
    fi
    exit "$status"

# The compact30 construction-cost comparison (docs/journal/2026-11-03-compact30-construction-
# protocol.md, draft 3; the harness's journal 2026-11-03-compact30-construction-harness.md):
# production's public bwt::build (P) against compact30's direct constructor,
# Ctor::build::<false> (C), warm and cold, on one core and on four in lockstep, 20 paired
# rounds and an untimed round -1, on the primary corpus (checked by its hash). It runs the
# frozen harness commit in its own worktree (target/c30-construction-pin), built with the
# reviewed toolchain, rustc 1.97.0 (LLVM 22.1.6), selected through rustup without touching
# the default, and stops before any timing if that toolchain is missing or not the reviewed
# build, if the analysis's or the listing check's tests fail, or if the timed path's machine
# code is not the reviewed listings (G3). Workers go on the second to fifth physical cores,
# or the first four where there are only four. No sudo, no counters, no setting changed.
# Every file goes to target/c30-construction-<UTC time>[-mode]/, the console included,
# archived as target/c30-construction-<UTC time>[-mode].tar.gz with a manifest, whether the
# run passed or failed.
# The third argument is the mode:
# - dry-run: the whole path untimed (rounds -1, 0 and 1, the records marked a dry run, no
#   timing figure read), then the six peak-RSS processes; its analysis records the capture
#   process's peak RSS (postflight VmHWM) beside the computed estimate;
# - capture: the registered run. It needs the fourth argument, the dry run's peak RSS in
#   bytes (the analysis's memory.peak_rss_vm_hwm), so the preflight requires
#   max(estimate, peak) plus max(2 GiB, 25%);
# - fail-output, fail-output-p, fail-pin, fail-reject, fail-preflight: a dry run with a fault
#   injected (C's first table of W4 worker 2 corrupted after the completion barrier; P's first
#   W1 table likewise; W4 worker 1 pinned to a CPU that does not exist; C given counts that
#   are not L's own; the memory preflight failed), to show the run ends at once, with its
#   failure recorded and archived.
#   just bench-c30-construction target/walk-decoder/work/corpora 20 dry-run
#   just bench-c30-construction target/walk-decoder/work/corpora 20 capture <peak-bytes>
bench-c30-construction corpora="target/walk-decoder/work/corpora" rounds="20" run="dry-run" peak_rss="0":
    #!/usr/bin/env bash
    set -uo pipefail
    pin=e264eed7e1a1c2f1d456d40207ff302811df2eae
    tc=1.97.0
    tc_commit=2d8144b7880597b6e6d3dfd63a9a9efae3f533d3
    tc_llvm=22.1.6
    rss=yes
    case "{{run}}" in
      capture) flags=(--peak-rss "{{peak_rss}}")
        [ "{{peak_rss}}" -gt 0 ] 2> /dev/null || { echo "c30: a capture needs the dry run's peak RSS (bytes) as its fourth argument"; exit 2; } ;;
      dry-run) flags=(--dry-run) ;;
      fail-output) flags=(--dry-run --inject output:w4:C:2); rss=no ;;
      fail-output-p) flags=(--dry-run --inject output:w1:P:0); rss=no ;;
      fail-pin) flags=(--dry-run --inject pin:w4:1); rss=no ;;
      fail-reject) flags=(--dry-run --inject reject:w4:3); rss=no ;;
      fail-preflight) flags=(--dry-run --inject preflight); rss=no ;;
      *) echo "c30: the mode is dry-run, capture, fail-output, fail-output-p, fail-pin, fail-reject or fail-preflight"; exit 2 ;;
    esac
    f=$(realpath "{{corpora}}")/wikidata-20261026.bz2
    wt=$(realpath -m target/c30-construction-pin)
    suffix=$([ "{{run}}" = capture ] || echo "-{{run}}")
    out=$(realpath -m target)/c30-construction-$(date -u +%Y%m%dT%H%M%SZ)$suffix
    mkdir -p "$out"
    (
      set -e
      echo "cf13f4394e4ea0a38a0e9e3bc5271b2dce717c8f98fe3a0b8a5186c84216b8fe  $f" | sha256sum -c -
      git cat-file -e "$pin^{commit}" 2> /dev/null || git fetch -q origin research/compact30-constructor
      [ -d "$wt" ] || git worktree add -q --detach "$wt" "$pin"
      git -C "$wt" diff --quiet HEAD || { echo "c30: $wt has local changes"; exit 1; }
      if [ "$(git -C "$wt" rev-parse HEAD)" != "$pin" ]; then
        echo "c30: $wt moved from $(git -C "$wt" rev-parse --short HEAD) to the pin"
        git -C "$wt" checkout -q --detach "$pin"
      fi
      [ "$(git -C "$wt" rev-parse HEAD)" = "$pin" ] || { echo "c30: $wt is not at the pin"; exit 1; }
      echo "c30: the frozen commit $pin, corpus checked, mode {{run}}"
      if ! command -v rustup > /dev/null || ! rustup toolchain list | grep -q "^$tc-"; then
        echo "c30: the reviewed toolchain, rustc $tc (LLVM $tc_llvm), is needed: rustup toolchain install $tc --profile minimal"
        exit 1
      fi
      RUSTUP_AUTO_INSTALL=0 rustup run "$tc" rustc -vV | tee "$out/rustc-vV.txt"
      grep -qx "release: $tc" "$out/rustc-vV.txt" && grep -qx "commit-hash: $tc_commit" "$out/rustc-vV.txt" && grep -qx "LLVM version: $tc_llvm" "$out/rustc-vV.txt" \
        || { echo "c30: rustup's $tc is not the reviewed build (want commit $tc_commit, LLVM $tc_llvm); no run"; exit 1; }
      RUSTUP_AUTO_INSTALL=0 rustup run "$tc" cargo -V | tee -a "$out/rustc-vV.txt"
      echo "c30: building with the reviewed toolchain rustc $tc, LLVM $tc_llvm, no rustflags"
      (cd "$wt" && CARGO_TARGET_DIR="$wt/target/rust-$tc" CARGO_ENCODED_RUSTFLAGS= RUSTUP_AUTO_INSTALL=0 rustup run "$tc" cargo build --locked --release --example c30walk)
      bin="$wt/target/rust-$tc/release/examples/c30walk"
      python3 -I "$wt/bench/test_c30_construction.py" | tail -1
      python3 -I "$wt/bench/test_c30_build_listing.py" | tail -1
      mkdir -p "$out/listing"
      ident=$(python3 -I "$wt/bench/c30_build_listing.py" "$bin" "$out/listing" "$wt/docs/journal/2026-11-03-compact30-construction-harness/listings")
      echo "c30: the timed path's machine code is the reviewed listings: $ident"
      [ "$ident" = yes ] || { echo "c30: G3 fails (see listing/*.diff); no run"; exit 1; }
      phys=$(for c in $(taskset -pc $$ | sed 's/.*: //' | tr ',' '\n' | while IFS=- read a b; do seq "$a" "${b:-$a}"; done); do
        cut -d, -f1 /sys/devices/system/cpu/cpu$c/topology/thread_siblings_list | cut -d- -f1
      done | sort -nu)
      n=$(echo "$phys" | wc -l)
      [ "$n" -ge 4 ] || { echo "c30: $n physical cores; four are needed"; exit 1; }
      if [ "$n" -ge 5 ]; then cpus=$(echo "$phys" | sed -n 2,5p | paste -sd,); else cpus=$(echo "$phys" | head -4 | paste -sd,); fi
      echo "c30: workers on CPUs $cpus (W1 on the first)"
      "$bin" barrier-test --cpus "$cpus" | tee "$out/barrier-test.txt"
      g=0
      timeout --kill-after=30 7200 "$bin" cbuild "$out/c30-construction.jsonl" "$f" --rounds {{rounds}} --cpus "$cpus" "${flags[@]}" || g=$?
      echo "c30: the collector exited with status $g"
      rfiles=()
      if [ "$g" = 0 ] && [ "$rss" = yes ]; then
        for k in 1 2 3; do
          for arm in P C; do
            "$bin" cbuild-rss "$out/rss-$k-$arm.json" "$f" --arm "$arm" --cpus "$cpus"
            rfiles+=("$out/rss-$k-$arm.json")
          done
        done
      fi
      a=0
      python3 -I "$wt/bench/c30_construction.py" "$out/c30-construction.jsonl" --identity-ok "$ident" --json "$out/c30-analysis.json" --rss "${rfiles[@]}" || a=$?
      [ "$g" = 0 ] || exit "$g"
      exit "$a"
    ) 2>&1 | tee "$out/console.log"
    status=${PIPESTATUS[0]}
    cmd="just bench-c30-construction {{corpora}} {{rounds}} {{run}} {{peak_rss}}"
    if [ -d "$wt" ]; then
      (cd "$wt" && python3 bench/archive.py "$out" --corpus "$f" --binary "$wt/target/rust-$tc/release/examples/c30walk" --command "$cmd" --status "$status")
    else
      python3 bench/archive.py "$out" --corpus "$f" --command "$cmd" --status "$status"
    fi
    exit "$status"
