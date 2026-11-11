# D100 occupancy ledger correction

Assignment: [#363/6789867142](https://github.com/lmmx/mibz2/issues/363#issuecomment-6789867142),
queued as [#145/6782527893](https://github.com/lmmx/mibz2/pull/145#issuecomment-6782527893).
Base: master `2635f04e9155dd31388b24fd3f7248f87b3ddcda`.
Branch: `fix/occupancy-route-ledger`. This is a separate instrument repair.

## Defect and correction

[D100](https://github.com/lmmx/mibz2/blob/1bfe226e98e9589b60a626e30a67ea1b31bf8c38/state/decisions.md#d100--inherited-occupancy-ledger-mismatch-scoped-b6-hold)
records the original failure: inherited Occupancy run 42923929886, job 130205457614,
ordinary-worker `ll` around 1.799 MB against a fixed 900 kB assumption and forced-team
`rle` near zero against an assumed block-sized floor. The ordinary arm printed a
violation without failing. That failure remains a Class A failure of the inherited
instrument; this patch is not an unchanged-code retry or a decoder allocation fix.

Successful `invert_runs` writes `Scratch.ll` even for Cooperative. Columns reserve
their current stream bound in `ll`; only separate-output column inversion uses `rle`.
Worker reuse retains allocations across both routes and different stream levels.
New trace-only snapshots record actual successful inversion route, rows, stream bound
and integer buffer sizes. Each worker's requests and finite growth bounds are checked
before means or medians. Final buffers must agree with the recorded history. Counts
and the one-core metadata checks remain; deduplicated segment counts can be below 512,
so two metadata vectors use actual logical-length history with bounded growth.
See [the ledger](../../ledger.md#route-aware-worker-validation) for the bound derivation.

The occupancy instrument retains raw histories and errors (`worker_ledger` schema
`route-history-v1`). Any traced round with missing/incomplete history or a worker
violation fails validation. A bad worker cannot hide in a mean, nor a bad round in a
median. Output is written before exit; workflow `pipefail` and `always()` artifact
upload preserve the failure and its evidence. Non-worker accounting assertions and
the experimental workload, ordering, rounds, overhead rule and thresholds are unchanged.

## Native evidence

Commands use Rust 1.99.1 unless specified. Logs and exact-byte compressed originals
are in [evidence](evidence); SHA256SUMS covers both evidence and changed sources.
No performance inference is made from test durations.

- `cargo test --features trace --lib trace::tests -- --nocapture`: 4/4, including
  eight asserted-route sequences (Segmented and Cooperative<2>, runs, columns,
  run→column, column→run). Every block has byte parity, successful CRC, independent
  RLE1 row count, actual route and stream-bound assertions, bounded capacities, and
  negative tests injecting impossible final `ll`, `rle` and `tt` capacities.
- D100 witness: 600,000 then 800,000 run rows gives `ll=1,200,000`, `rle=0` for both
  kernels. A subsequent 90,000-row level-1 stream retains that `ll`. Separate column
  output remains retained during later run blocks, whose output still uses `ll`.
- `cargo test --features trace --example occupancy -- --nocapture`: 5/5. A real
  concatenated stream traverses Workers(1), Workers(2), and Teams{teams:1,k:2}; each
  exports three actual route snapshots (two runs and one column), preserves bytes/CRC
  and passes completeness and per-worker validation. Synthetic ordinary workers
  whose mean `ll` capacity is exactly 900 kB still fail when one individual capacity
  is impossible. One bad round among two good rounds and missing results fail too.
- `cargo +1.90.0 clippy --features trace --lib --tests --example occupancy -- -D warnings`:
  pass after replacing the matrix's complex type annotation with a type alias.
- Full validation with affinity restricted to four allowed CPUs: `taskset -c <four allowed CPUs>`
  (exact CPU list recorded in `native-affinity.log`) followed by
  `cargo +1.99.1 test --features trace --lib --example occupancy`: 99 library tests
  passed, one existing census ignored; 5 occupancy tests passed. The actual recorded
  command is the reproduction command.
- `cargo +1.90.0 fmt --check`, `git diff --check` and the repository's whitespace-format
  check on changed files and evidence: pass.

Preserved failures: the first intended one-block level-1 fixture used 100,000 bytes,
which libbzip2 split after 99,981 bytes; it now uses 90,000. The first column-at-level-9
fixture was sparse enough to take the run route; it now uses 700,000 dense bytes, with
route assertions retained. The 70,000-row column exposed the old metadata floor
(508 deduplicated segments vs 512), corrected using logical counts rather than measured
capacities. Initial Clippy type-complexity failure is retained. The unconstrained full
library run passed 98, failed the existing `parallel::plan_tests::the_policy_here`
assertion (`5 cores of 4 CPUs`), and ignored one census. No policy/test assertion was
weakened; the follow-up constrains affinity to four allowed CPUs to match the container
quota. All failures are archived, not replaced by the passing logs.

## Scope and review boundary

No new unsafe operations or kernel changes. Recording, storage and transfer are behind
`trace`/test cfg; default builds contain none of the new state or recording operations.
Trace level zero records nothing; traced modes now allocate snapshots per block and
must still pass the existing overhead interpretation rule. This is not a claim of
zero tracing overhead or an independent safety certification.

Certified bounds cover `ll`, `rle`, `tt`, counts, and one-core segment metadata. Run
scratch, checkpoints, cooperative cuts/segments, segment byte buffers, tracing overhead
and RSS are measured or explicitly outside these bounds. Pipelined/service decoders do
not export these histories and cannot receive a passing verdict from this validator.
No local timed occupancy sweep, i9 run, workstation submission, cap/threshold/default
change, benchmark retry, merge or release was performed. #366's candidate, baseline,
decision rule and original adverse evidence are untouched. The bounded repair does
not itself lift D100: Guardian must independently perform the targeted resumption
check through the Coordinator, including the repaired CI evidence when available.

PR #365 remains frozen at `d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2`.
Researcher [6790022921](https://github.com/lmmx/mibz2/pull/145#issuecomment-6790022921)
accepted F01; Coordinator [6790163736](https://github.com/lmmx/mibz2/pull/145#issuecomment-6790163736)
keeps integration deferred for corrected-head default benchmark losses. Its source,
archived Miri39/39 and prior benchmark evidence were not modified here.

## Resumption

Coordinator: route the exact repair head, digests, native evidence and CI results to
Guardian for D100's targeted check. Neither a green check nor this implementation
handoff authorises workstation execution. Codex is still manually launched: the
current live session consumed this assignment, but posting a GitHub event does not
restart a stopped Codex session. A human/platform launch is required for future work
unless an independently verified execution trigger is established.
