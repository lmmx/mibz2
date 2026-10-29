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

# bench/variants.py: released versions (v0.1.1-v0.1.5) and walk settings on this build
# (16 or 8 lanes; prefetch off, into L2, non-temporal), each built in its own worktree,
# checked to give this build's bytes, then run beside this build at 2, 4, 6, 8 and 10
# threads, interleaved by round, back to back. Each one's scaling against its own 4-thread
# runs and its throughput against this build's. It writes variants-<UTC time>.json.
#   just bench-variants
#   just bench-variants --variants v0.1.2,v0.1.3,lanes8,pf-off --rounds 8

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

# bench/coop.sh: the knee pilot's cooperative inversion (examples/coop.rs, research only):
# one block's T-vector built and walked by k cores, then N blocks in flight on k cores each
# against N production workers, on one CPU of each physical core, with libsais 2.12.1's
# multi-threaded inverse BWT beside it when gcc has OpenMP. A few minutes. It writes
# coop-<UTC time>.md to share.
#   just bench-coop
#   just bench-coop --rounds 8 --arms p4,p8,4x2,5x2

# Whether k cores sharing one block's T-vector move the knee (#39, #65)
[positional-arguments]
bench-coop *args:
    bash bench/corpus.sh
    bash bench/coop.sh "$@"

# bench/coop_ab.py after bench/coop.sh: the #77 experiment (the coop journal's §12). The
# frozen cooperative kernel (3x3, 4x1) against the same with its walk in room passes (r3x3,
# r4x1), with production's p4 beside them, 20 rounds paired, each thread pinned. About two
# minutes. Close the browser and other busy programs first: the gains sought are a few
# percent. It writes coop-<UTC time>.md, its .jsonl and coop-ab-<UTC time>.md: upload the
# coop-ab and coop .md files and the .jsonl.
#   just bench-coop-ab
#   just bench-coop-ab --rounds 40

# The #77 A/B: the cooperative walk in room passes against the frozen kernel
[positional-arguments]
bench-coop-ab *args:
    bash bench/corpus.sh
    bash bench/coop.sh --arms 3x3,r3x3,4x1,r4x1,p4 --rounds 20 "$@"
    python3 bench/coop_ab.py

# bench/coop_ab.py after bench/coop.sh: #78's first experiment (the coop journal's §13),
# an optimistic bound. The room-walk kernel (r3x3, r4x1, #77's) against the same with its
# build given each range's counts from checkpoints made outside the timing (q3x3, q4x1),
# with production's p4 beside them, 20 rounds paired, each thread pinned. About two
# minutes; close the browser and other busy programs first. Upload the coop-ab and coop .md
# files and the .jsonl.
#   just bench-coop-given

# #78's bound: the cooperative build without its recount, the counts given for free
[positional-arguments]
bench-coop-given *args:
    bash bench/corpus.sh
    bash bench/coop.sh --arms r3x3,q3x3,r4x1,q4x1,p4 --rounds 20 "$@"
    python3 bench/coop_ab.py

# bench/coop_counters.py: the counters behind the cooperative grid (#76; the coop journal's
# §8.2 and §10): each arm alone under `perf stat` for 1 and 21 rounds, one counter pair a
# pass, 3 repeats, the memory controller machine-wide, then one `perf record` each of p4, 4x1
# and libsais. About 15 minutes (runs take 1-1.5 s each on the i9); needs sudo for perf.
# Check the events first (under a minute), then the full run: of the two offcore-response
# passes (l3_all, l3_all_ocr) one is expected to be left out, by perf's naming. Upload
# coop-counters-*.md and coop-counters-*.jsonl.
#   just bench-coop-counters --arms p4 --reps 1 --rounds 3 --no-record
#   just bench-coop-counters

# Why the cooperative arms win: the grid's arms under perf, residency against overlap (#76)
[positional-arguments]
bench-coop-counters *args:
    bash bench/corpus.sh
    python3 bench/coop_counters.py "$@"

# bench/coop_profile.py: where the cooperative kernel's instructions go (#77; the coop
# journal's §11): p4, 4x1, 3x3 and libsais's s4x1, each alone under `perf record` for
# retired instructions, cycles and memory-stall cycles; per symbol and phase per output byte,
# and the hottest instructions of each arm's hottest symbols. About 3 minutes; needs sudo for
# perf. Upload coop-profile-*.md and coop-profile-*.tar.gz.
#   just bench-coop-profile --arms p4,4x1 --rounds 3
#   just bench-coop-profile

# Where the cooperative kernel's extra instructions are, by symbol and phase (#77)
[positional-arguments]
bench-coop-profile *args:
    bash bench/corpus.sh
    python3 bench/coop_profile.py "$@"
