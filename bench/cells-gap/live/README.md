# Frozen live-reader diagnostic preparation

Dispatch [363/6819644553](https://github.com/lmmx/mibz2/issues/363#issuecomment-6819644553).
[Prospective design](../../../docs/journal/2026-11-12-cells-live/PLAN.md),
[validation, safety scope and dependencies](../../../docs/journal/2026-11-12-cells-live/HANDOFF.md).
This is a diagnostic preparation, not a ready workstation job or production qualification.

## Noninteractive workstation binding

`bench-cells-live-path-i9` is the argument-free queue recipe. It selects Rust1.99.1
and stages all three hash-checked inputs under the new attempt's `inputs/` directory.
The existing generator creates pattern and onebyte. A verified canonical
`corpora/wikidata.bz2` is copied; otherwise the existing `../corpus.py` reconstructs
it from windows pinned at `a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a`, validating
their hashes and streams. This input construction is outside the diagnostic's
unchanged nine oracle/check jobs and 120 measured children; it adds no warmup.

The fixed CPU rule is the lowest logical CPU in the process's current allowed
affinity set. Selection is noninteractive and read-only: `preflight.json` records
the allowed set, selected CPU, core/socket/sibling topology, lscpu output, input
identities and preparation-source hashes. The driver rechecks affinity and pins
every child to that CPU. No archived CPU ID or corpus path is used. Missing pinned
Git objects are fetched by exact SHA before building; compiler/source checks stay
unchanged. Preparation failures retain the usual single submission bundle.

This binding requires exact-head green CI, Coordinator recipe review and the normal
PROTOCOL9 ready tag/queue admission before the owner runs `just workstation-queue`.
The parameterised recipe remains available with its existing two arguments.

This subcrate extends #367 without replacing its source or evidence. `prepare.py`
materializes corrected #365 **d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2** from Git,
then inserts feature-gated safe hooks into that private source tree. The generator
asserts that its patch deletes no original line. Every original file and every
edited file has a SHA256 in `.generated/source.json`; the full additive patch is
also emitted. The original unsafe operations, gates, fallback bodies, long-factor
writer tests, BzReader, CellsReader and streaming hash remain present byte-for-byte.
No production source/default/team or #365 branch is edited.

The fixed plan is **ten balanced pairs × Wikidata/pattern/onebyte × trace off/on**,
120 eventual measured fresh children plus nine untimed/oracle passes. There is no
warm mode or superseded three-round mode in this driver. A separate warm proposal
has an explicit finite budget in PLAN and requires separate Coordinator admission.

## Untimed preparation

Use Rust1.99.1 (`58cf8f9c35b355199727627d07ae34a4e2846d67`, LLVM23.1.1,
x86_64-unknown-linux-gnu), release fat LTO and one codegen unit; no custom Rust flags.
Python uses its standard library, including bz2. Real recipe tests use Just1.38.0.
The corrected source object must already exist locally (CI fetches that exact SHA).

```sh
python3 bench/cells-gap/live/prepare.py
CELLS_LIVE_REQUIRE_JUST=1 python3 bench/cells-gap/live/test_driver.py
python3 bench/cells-gap/live/driver.py plan
cargo test --locked --manifest-path bench/cells-gap/live/Cargo.toml --features live-trace,cells-reader
cargo test --locked --manifest-path bench/cells-gap/live/Cargo.toml -p mibz2 --lib --features live-diagnostic cells::tests::
python3 bench/cells-gap/live/build.py target/live-binaries
python3 bench/cells-gap/live/check.py target/live-binaries target/live-fixtures
```

Each destination is new; existing evidence is never overwritten. Source generation
also refuses an existing destination. Retain/move an earlier generated tree when
intentionally preparing a new revision. `build.py` rejects stale prepared source,
original-hash changes and non-frozen compiler/flags, and records exact binary and
harness digests. Local dirty builds are allowed for untimed tests; the measured
driver requires a recorded clean commit. Run no `measure` or recipe execution now.

`check.py` executes only tiny fixtures through all four binaries: full byte/SHA
parity against libbzip2, CRC corruption/truncation/error parity and I/O failure.
The Python suite exercises the entire 129-job schedule with fake child results,
not corpora or clocks; it verifies failure preservation and real Just argument/
exit wiring with a stub beneath Just. The actual `plan` subcommand has no decode,
clock, settle, build or network side effects.

`inputs.py` can later reproduce only the two frozen synthetic corpora; it verifies
the historical compressed SHA256. For Wikidata, reuse #367's `../corpus.py` and
pinned byte windows if needed. None of those full corpora is replayed in preparation.

## Boundaries and data

On builds use hierarchical RAII spans on the **actual** scanner, front, inversion,
Cells order/induction/writer/attempt, unchanged fallback, finish/CRC, and reader/
drain. Each span records inclusive and exclusive nanoseconds and call count. The
exclusive sum must equal inclusive whole-reader time exactly. Inclusive inversion
is a second view, never another summand. All untimed spans have zero clocks and
zero durations. The off builds contain no hook invocations.

Per-block records carry actual scanner bit offset, primary, n, record count, outer
run-route reach, budget, letters when observed, result/refusal, attempted cells,
period, requested cells scratch/growth, capacities, position initialisation, and
copy bytes. Attempted induction counts are retained on budget refusal, not only
success. Induction copy bytes are exactly two16-byte helper calls per completed
cell, including discarded representations; these are addressed bytes, not DRAM
traffic. `pos_initialized_bytes` counts the logical initialized extent (4 × run
count), not repeated resize/fill stores or physical traffic. Writer counts are actual small/long/repetition copy-call lengths.
Absent event counters mean no such event only for these documented additive
counters; missing observational fields (e.g. letters before density refusal) stay
absent. Invalid invariant exits are not credited with an invented attempt count.

Unavailable: complete scanner/front/fallback allocation inventory, allocator/page
traffic, diagnostic map-node allocation bytes, and isolated clock/log latency.
Cells `buf`, `pos`, inversion output and decoded output capacities are observed;
that is not a total worker-memory bound. Clock-call count, event-vector growth/
capacity and off/on inflation are recorded. Counters perturb the on build; they
are not claimed free. Output/hashing remains in the reader span; trace JSON is
serialized after it. Parent `wait4` records CPU/RSS per fresh child; parent monotonic
wall spans spawn/wait only, excluding parent JSON parsing. Process-minus-reader
charges startup, taskset, serialization, logging and exit together, not just logging.

`driver.py analyse RECORDS` replays complete-decode paired ratios and their frozen
95% log-t intervals, off/on inflation and nonoverlapping cost reconciliation.
Incomplete/duplicate data, wrong binaries/digests, output or CRC errors fail.
A failed phase reconciliation stops the run before appending a successful row.
Raw stdout/stderr and FAILED.json survive. Each measured child has a `.resources`
sidecar with command/job identity, PID, raw wait status, exit code and the collected
wall/CPU/RSS tuple, flushed immediately after wait4 and before output rejection.
Spawn failures retain null PID/status/resources explicitly. Sidecar `validation`
describes only exit/JSON/decoder-error checks; later identity/CRC/phase failures
remain in FAILED.json and are never appended to successful paired rows.
If instrumentation changes the off-build category, quantitative cost attribution
is discarded and raw counts retained. No automatic rerun, gate waiver or backend
selection follows. Attribution that passes remains specific to the instrumented
path; it does not prove a causal decomposition of the off build. Matching categories
are not effect-size equivalence: off ratio0.9 and on ratio0.45 both count as faster
despite unequal instrumentation inflation. No phase savings may be transported
causally to the off build from that agreement.

The prepared Just recipe wraps `run.sh`; it creates one attempt directory, records
console/build/identity/results and bundles both success and failure using the
repository's existing bundler. Files at least50MiB are excluded: full Wikidata
precheck byte files remain local; their length/SHA records and logs travel.
It has not been executed or queued. Coordinator
reconciliation, independent harness review, integration and PROTOCOL9 decisions
must precede any workstation execution. No execution command is issued here.
