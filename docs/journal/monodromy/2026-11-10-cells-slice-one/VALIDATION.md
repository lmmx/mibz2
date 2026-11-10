# Slice-one validation, 10 November 2026

This is the original local validation record. The later CI verdict, single permitted
rerun and benchmark isolation correction are in the
[qualification handoff](../2026-11-10-cells-qualification/HANDOFF.md). Original raw
evidence below is retained unchanged; its pending-CI statements describe that earlier point.

Scope and proof: [SAFETY.md](SAFETY.md). Source/evidence digests are in
[evidence/SHA256SUMS](evidence/SHA256SUMS). These are correctness checks on a shared
cloud machine, **not performance evidence**. Native toolchain: Rust 1.90.0; Miri:
nightly 1.102.0 (`ac8310ba6`, 2026-11-09). The source digest records the tested cells
module independently of this documentation commit.

## Passed

- `cargo test --lib cells::tests`: five native tests. Exhaustive binary columns of
  lengths 1..=10 at every primary row; exact cell/floor budgets; malformed records;
  marked-row/backlink invariants; reuse and allocation bounds; each refusal followed
  by byte-identical shipped fallback.
- `cargo test --test cells`: four tests. Bundled libbzip2 1.0.8 and shipped `Segmented`
  against normal and forced cells, scalar and detected kernels, structured words,
  arbitrary two-run columns, CRC/truncation errors, stream concatenation and partial
  output. Synthetic `text`, `random`, and `records` assert zero production admissions.
- `cargo +nightly miri test --lib -- miri_cells`: all three tests completed. This is
  full coverage of the new named Miri tests, not a claim that the whole library was
  interpreted. `XDG_CACHE_HOME` was set inside the workspace for sysroot setup.
- `cargo build --profile fuzz --example fuzz`, then
  `target/fuzz/examples/fuzz 30 1 work/fuzz-failures cells`: final-source run, with
  reach asserted: **26,027 cases**, 13,665 forced admissions, 9,648 forced short cycles,
  zero forced refusals. Exact counts are in `evidence/fuzz-final.log`. Earlier seeds 1 and 2
  also passed (26,840 and 9,682 cases respectively); their counts depend on elapsed
  time and are not a deterministic case-count guarantee.
- cargo-mutants 27.1.0: **11/11 caught**, unmutated baseline passes. Command:
  `cargo mutants -f src/cells.rs --re 'src/cells.rs:(311|442|538):' --timeout 60 -o work/admission-mutants-final -- --lib cells::tests`.
  These line anchors identify the density, floor and budget predicates at the recorded
  source digest. The archived outcomes include the exact mutation descriptions.
- `python3 docs/journal/monodromy/2026-11-10-cells-slice-one/mutations.py work/unsafe-mutants-final`:
  **5/5 hand-selected unsafe-kernel mutations killed**, baseline passes. Each mutant
  compiled first; no timeout or build failure is counted as a kill. Targets: uncounted
  cell, lost mark, wrong predecessor, wrong rotation, undersized slot regions.
- Benchmark harness: **28/28 tests passed**, including the new streaming adapter's
  byte/error comparison. Clean logs split the 27 ordinary tests and
  `tests::the_run_records_each_childs_termination` (the latter also passed separately
  with `--nocapture`, exit 0). No benchmark acceptance rule was changed.
- `cargo clippy --lib --tests --example ab --example fuzz --all-features -- -D warnings`;
  `cargo fmt --all --check`; source/documentation `git diff --check` (raw evidence
  logs retain their original trailing whitespace); `cargo test --doc` (one passed).
- `cargo check --lib --target aarch64-unknown-linux-gnu` and
  `cargo check --lib --target x86_64-pc-windows-gnu`. Compilation only, not execution
  on either platform. Aarch64 reports the existing unused run-front bitmap fields.
- Broader native checks: the initial `cargo test` completed all library tests, including
  the long existing `room_walk_crafted_permutations`, with only the topology failure
  below (101 passed, one ignored, one failed; before the extra marker test was added).
  The final-source continuation
  `cargo test --tests -- --skip parallel::plan_tests::the_policy_here --skip bwt::tests::room_walk_crafted_permutations`
  completed **169 passing tests across 13 test binaries**, one existing ignored test.
  The long walk test was excluded only from this continuation because it had already
  completed unchanged in the initial run.

## Findings resolved during validation

The first admission mutation campaign caught 9/11. The floor mutations `>` to `>=`
and subtraction to division survived because the exact-fit fixtures did not hit an
exact *floor* equality. Added the minimal two-row `[1, 0]` witness to the Miri/exact-fit
test; the repeated campaign caught all 11. The first result is retained in
`evidence/admission-initial.log`; it is not relabelled a pass.

## Existing local failures

- `parallel::plan_tests::the_policy_here` asserts physical topology <= Rust's available
  parallelism; this cloud container reports **5 cores of 4 CPUs**. Reproduced on an
  isolated copy of unchanged master `edaa984c4f57614f25bb897e808ab12c59c48d8f`.
  `src/parallel.rs` is unchanged.
- Full `cargo clippy --all-targets --all-features -- -D warnings` fails on the existing
  `examples/wait_audit.rs:70` constant assertion (`clippy::assertions_on_constants`).
  Reproduced on unchanged master with `--example wait_audit --all-features`.
  The changed-target Clippy command above passes. No team or unrelated source was
  modified to suppress these failures.

## Missing evidence / gates still open

- **Independent adversarial B5 review.** The implementer wrote the code, proof and tests;
  this is not an independent safety review. The PR is for review, not merge-ready.
- **Exact-head CI**, including its existing default head/base performance gate, platform
  execution and full Miri job. The local full check failures above are disclosed, not
  treated as green.
- **Pinned real Wikidata benchmark corpus and end-to-end performance/RSS evidence.**
  That corpus is not present locally. CI's normal corpus acquisition and the added
  `mibz2-cells` arm provide the independent whole-decoder measurements. No local result
  substitutes for them. Slice one's experimental numbers are reported, not judged.
- The **later default flip** and its i9 one-build A/B are outside this PR. The already
  passed i9 inversion-stage experiment was not repeated.
