# P74-C controlled stage validation: prepared, timing not authorized

Implementation reference: `1bf554db61a82ff86f832b8697b90675e6a6ba46`.
The implementation and production dispatch are unchanged. Preparation adds only
`examples/p74_experiment.rs`, `bench/p74_experiment.py`, a test-only audit, and these
records. The machine-readable preparation manifest records the exact preparation
HEAD, dirty-tree state, source hashes, release-binary hash, corpus hashes and host.
A final clean-checkout manifest on the approved host is required before timing.
No elapsed-performance results have been collected.

## Assembly-gate amendment, declared before any timings

The original zero-extra-instruction gate **failed**. This experiment explicitly
replaces that gate with acceptance of the documented +1 plain / +1 checkpoint
instruction discrepancy in `1bf554d`: six ordinary low-rank conditional branches,
no added YMM-head spills and no added head-to-head dependency. The old gate did not
pass. This amendment permits measurement only; it is not evidence of speed, safety
on controls, or authorization for production integration. Policy constants remain
S<16384 or 2*S>=N_max for standalone dispatch and
F+1>floor(S_consumed/32)+123 for permanent switching, including the trigger in S.

## Host and authorization boundary

Available host: Codex Cloud KVM, AMD EPYC 9V74 (Zen 4), affinity CPUs 0..2,
three exposed physical cores, one exposed thread/core and no exposed SMT siblings.
This is a virtual topology, not proof of dedicated physical host cores. cpu.max is
`200000 100000` (two CPUs). It is **excluded from timings** because of quota/shared
host interference. Affinity 0,1 is used for correctness because the existing topology
test assumes affinity and quota agree. No i9 use.

The repository's existing `.github/workflows/bench.yml` job `bench` uses
`ubuntu-latest`, not a fixed named runner. Its dispatch also launches unrelated
whole-decoder/alternative workloads and does not call this harness. It is **not an
approved execution mechanism for this experiment**. Historical Gate 1 EPYC runners
are observations of past jobs, not a currently reserved host. No suitable specific
approved performance runner is attached or identified. No CI dispatch, PR creation,
performance job or workflow change is made here.

Before execution obtain approval naming the host/runner, mechanism, affinity CPU,
physical core/SMT topology, quota (no CPU cap), isolation/background-load conditions,
and exact commands below. Freeze a new host manifest plus approval record; pin one
physical core with its SMT sibling idle, use that CPU for all arms. If any condition
cannot be established, stop without timing. The Rust measurement mode requires an
explicit `P74_APPROVED_RUNNER` value and refuses a capped cgroup. These are accidental
execution guards; setting the variable is not itself approval or host verification.

## Arm routing and measurement validity

T0 calls untouched standalone shipped MTF; T1 calls the combined kernel with its
runtime start_shipped flag before token zero, without bypass; T2 uses frozen bypass
and guard. Both plain and checkpoint variants are linked in the same release binary.
An AVX2 host is required. The driver supplies runtime black_box Symbols, Arm and
Kernels arguments; output length, bytes, all 256 counts and checkpoint arrays are
consumed and compared on **every block of every pass**. Release assertions stay on.
The compiler cannot discard decoding as unused or merge the three logical arms.
Kernels detection, allocation, scanning, checks and printing are outside timing.

Disassembly of the prepared binary's `p74_experiment::apply` was inspected: distinct
calls to both standalone AVX2 kernels and both combined instantiations; T2 checks
S<0x4000 and 2*S>=max before standalone entry; T1/T2 set start_shipped from arm==1.
Those calls survive optimized fat LTO. The artifact `apply.asm` records final addresses.
This establishes routing, not a performance result. Archive the measured binary and
its assembly; do not rebuild between arm measurements.

## Corpora and coverage

The companion manifest contains compressed/decompressed sizes and SHA-256 for every
file, block counts, encoded-symbol totals, L bytes, checkpoint-record totals and
observed per-block switch locations. Keep both decoded bytes and L bytes distinct:
MTF throughput uses **L bytes**, not final RLE1-expanded decoder output.

- `wikidata-fitted`: the pinned 2026-10-19 corpus from `bench/wikidata.json`.
  Compressed 11854933 bytes, SHA-256
  `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`;
  decoded 209700196 bytes, L=209706557, 431 blocks. Twelve 1-MiB windows at
  (2i+1)/24 of the dated dump, exact offsets/checksums in the existing spec.
  **Fitted**, not independent Wikidata. Reassembled bit-for-bit from `bench-data`
  cached windows, checked against bundled libbzip2 and shipped mibz2. The small
  `--windows` helper uses the existing window reconstruction method without requiring
  an unavailable external lbzip2 executable.
- `random` and `records`: existing `tests/common::corpora(4)` generators, full data,
  8000000 and 9600000 decoded bytes respectively. The exact compressed and decoded
  hashes are frozen in the manifest. These are existing negative controls, not
  independent Wikidata validation.
- `mixed-independent`: 2000000 bytes; xorshift seed `0x74c0202610310001`, alternating
  10000-byte chunks from `mixed-control` and full-range random bytes. Seed/distribution
  fixed before timing, not fitted to outcomes. It tests an actual online handoff.
- `text`: existing source-corpus generator, scale 4, descriptive secondary result.
  Its bytes depend on source contents; freeze the compressed file rather than rebuilding
  it after preparation. Reproduction uses the exact preparation checkout.
- Independent Wikidata is **unavailable in the attached data**: the cached branch contains
  only fitted windows. No independent source is substituted or called independent.
  Generalization/selection-robustness qualification stays incomplete. Add any disjoint
  dump slice only through a named/checksummed protocol addendum before timings.

## Timing and analysis, frozen before measurement

Use rustc 1.90.0 (`1159e78c4`), LLVM 20.1.8, existing release flags (fat LTO,
codegen-units=1), empty RUSTFLAGS and no target-cpu=native. Record toolchain, lockfile,
binary/source hashes. No kernel edits, policy tuning or result-driven compiler changes.

Corpus order: fitted Wikidata, random, records, mixed-independent, text. Variant order:
plain then checkpoints. Each variant warms each arm twice in fixed T0/T1/T2 order,
untimed. Then **12 paired rounds**, all six arm permutations repeated twice, shuffled
with xorshift Fisher-Yates seed `0x74c0202610310002`. Each arm occupies each position
four times. The same order schedule is used for each variant/corpus. Each observation
is the sum of two complete corpus passes, preserving block decode order. No fastest
pass, outlier removal, adaptive repeat count, rerun after unfavorable results, optional
stopping or threshold adjustment.

Instant monotonic wall time surrounds each MTF call; sum per-block durations, divide
by two for corpus latency. The shared wrapper/black_box/Instant cost is not subtracted.
Assertions are outside each timed interval. Capacity is reserved untimed per corpus
pass; buffers are reused over its blocks. Checks between blocks affect cache state
identically for all arms. This is a prepared-symbol stage experiment, not end-to-end
pipeline timing. Cache/checking overhead, small-block timer cost, noise and temporal
correlation limit interpretation; twelve rounds do not prove independent samples.

For each variant/corpus: save every raw round/position/arm elapsed ns and L bytes.
Report latency and decimal MB/s, paired T2/T0 (primary), T1/T0 (refactor tax), T2/T1
(incremental policy benefit), and paired ns differences. Mean within-round ratios;
two-sided nominal 95% Student t intervals with 11 df, critical 2.200985. No claim of
simultaneous family coverage. Summaries validate completeness/no duplicate observations.
Plain/checkpoint cost comparisons are descriptive because variants run in separate
sessions. Switch audits record S, explicit ranks, high-rank events, RUNA/B digits,
initial bypass, observed consumed-symbol switch location, n at handoff and F before
handoff. Checkpoint record count is total count-array writes (1024 bytes each), not
an isolated timing attribution. No PMU attribution without separate authorization.

Registered criteria:

1. Any output/count/checkpoint/error mismatch fails correctness and invalidates timings.
2. Fitted Wikidata checkpoint: T2/T0 upper 95% endpoint <=0.95 (at least 5% saving),
   and its lower confidence bound on saving exceeds twice the largest absolute
   departure from 1 in T1/T0's confidence interval. This conservatively charges the
   refactor/layout control. Plain Wikidata is reported separately, not substituted.
3. Random and records, both variants: T2/T0 upper 95% endpoint <=1.03. Apply the
   same safety test to the independently seeded mixed control. Failure to establish
   the bound is failure of acceptance, not proof that the true regression exceeds 3%.
4. T1/T0 is reported for every row, with intervals; material refactor tax is interpreted
   before attributing T2's results. No estimated cycle cost substitutes for measurement.
5. Observed switches must match the frozen policy at every block in both variants.
6. Independent Wikidata robustness and the complete original adversarial performance
   suite are not covered by fitted-corpus success. No claim of complete original
   portfolio acceptance or shipping qualification from this narrower experiment.

Until timings are authorized all economic criteria are **not evaluated**, not pass.
No whole-decoder gain is measured. If later useful, a model may use
`speedup = 1/(1-f+f*r)`, independently justified baseline stage fraction f and measured
stage time ratio r. Carry uncertainty/host differences; do not reuse an old runner's
fraction as if measured on this host. No numeric modeled gain exists yet.

## Preparation and proposed execution commands

Run preparation at the exact checkout, preserving its generated files:

```sh
unset RUSTFLAGS
cargo +1.90.0 build --locked --release --example p74_experiment
work=/absolute/path/to/p74-validation
mkdir -p "$work/corpora"
git fetch origin bench-data:refs/remotes/origin/bench-data
git archive origin/bench-data windows/wikidata-20261019 | tar -x -C "$work"
target/release/examples/p74_experiment --prepare "$work/corpora"
target/release/examples/p74_experiment --windows "$work/corpora/wikidata-fitted.bz2" "$work"/windows/wikidata-20261019/w*.bin
# files.txt: absolute corpus paths, one per line, in the frozen order above.
target/release/examples/p74_experiment --check $(cat "$work/files.txt") > "$work/check.tsv"
P74_AUDIT_FILES="$work/files.txt" cargo +1.90.0 test --locked --lib audit_manifest_corpora -- --ignored --nocapture > "$work/audit.log"
python3 bench/p74_experiment.py manifest --files "$work/files.txt" --check "$work/check.tsv" --audit "$work/audit.log" --binary target/release/examples/p74_experiment --out "$work/manifest.json"
```

Paths here must contain no whitespace (the shell expands files.txt). Validate all
cached window SHA-256 against `bench/wikidata.json`; final fitted corpus hash is also
asserted by manifest generation. Archive manifest/approval/binary before timing.
The generated manifest marks its host unapproved by default; preserve it and attach
an explicit approval/runner record rather than silently changing that status.

Only after approval naming a suitable runner and CPU, proposed commands are:

```sh
# APPROVED_HOST_ID and APPROVED_CPU must come from that explicit approval.
P74_APPROVED_RUNNER="$APPROVED_HOST_ID" taskset -c "$APPROVED_CPU" target/release/examples/p74_experiment --measure-approved $(cat "$work/files.txt") > "$work/raw.tsv"
python3 bench/p74_experiment.py summarize --manifest "$work/manifest.json" --raw "$work/raw.tsv" --out "$work/summary.json"
```

The raw TSV is flushed/printed after each completed arm; a crashed or incomplete run
cannot pass summary completeness checks. Record the interruption, never silently
replace an individual row. Counters/assembly-driven investigations require separate
authorization. If the stage criteria are met, propose a separate experiment at 1,
2 and 4 verified physical cores, reporting wall throughput, CPU-seconds/GB and peak
RSS. No such decoder run is authorized or executed here.

A [clean manifest](2026-10-31-p74-c-validation/clean-preparation-manifest.json)
records preparation commit `eccf1cec2ae401e80090f929e2fbb2c26a10cc8a` with an empty
working tree; subsequent artifact-only documentation does not change harness or kernel.
Its SHA-256 is `99c90edf8dd9c8774d43928ed05632f68d23507f6afd23da1d0617d1f63f6d83`.

The companion [preparation manifest](2026-10-31-p74-c-validation/preparation-manifest.json)
records the honest dirty preparation snapshot and all five source hashes before commit;
[actual switch results](2026-10-31-p74-c-validation/switches.md) and
[optimized arm routing](2026-10-31-p74-c-validation/apply.asm) are preserved alongside it.
Do not treat its unapproved host fields as execution approval. Generate a fresh clean
manifest on the named runner before any timing, preserving these dataset hashes.
The source corpus text is an exploratory input whose bytes are pinned here; changing
checkout/source content can regenerate different text. Random/records/mixed generation
is independent of the source-text contents.

Only T0/T1/T2 are timed. Historical P74 Free speedups are not same-run baselines, so
no exact fraction of the historical Free advantage can be claimed from this experiment.
T2/T0 gives the current stage benefit; T2/T1 gives the net adaptive benefit.

## Preparation checks

At the unchanged reference kernel: `taskset -c 0,1 cargo test --offline` passed
74 tests and one doctest before preparation; exact-output/checkpoint/error tests
included. After adding the audit, the seven focused library tests and five interface
tests pass; the audit is an explicitly selected ignored test. An existing helper
assumed the synthetic reverse alphabet for prefix reconstruction; it now copies the
real block's seq_to_unseq. The initial audit failure was in that oracle helper, not
an arm output mismatch. Repeating the corpus audit passes for all five corpora.

Final release `--check` compares all six arms over all blocks with independent
libbzip2 and inverse-BWT/RLE1 checks, without calling Instant. Format and all-target,
all-feature Clippy pass. Analysis code is exercised with clearly labeled synthetic
TSV fixtures only; these are not timings and do not supply empirical confidence
intervals. The final machine manifest and switch table are companion artifacts.
