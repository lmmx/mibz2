# Run-fed stage re-review: PROCEED to freeze

Inbox: https://github.com/lmmx/mibz2/pull/162#issuecomment-6729877063
Source: PR #269, fd60fffada25d475bbcc2b54165b96cf5fd7a92f.
Supersedes the freeze objections in Researcher REVIEW 6729473179, at 187c0ba517c3324aa28c5e7ccd2667f6c0572145.
Policy: docs/team at a4f97714cc14366984015869d9d094d89488ac57.

## Decision and scope

PROCEED to freeze this bounded diagnostic. The Coordinator records the frozen source and recipe before any judged run. This is a Researcher assessment, not execution, merge, production qualification, or closure of B5.

The specific B6 witness in my prior review is resolved by the checked state interface. B5 remains: its written argument, differential/fuzz/mutation evidence and independent adversarial assessment are not replaced by the word diagnostic or by a green CI badge. Guardian may assess sequencing and impose a scoped HOLD; it cannot waive the standing requirement. This review is not the full emitter/unsafe-code audit.

## Re-derived corrections

1. **Checked state.** Scratch.fo and Sorted fields are private; sort validates the L tiling, total counts and per-symbol cumulative lengths before returning a token borrowing the records and scratch. fill/fill_rows consume that token, and the AVX2 implementation is private unsafe. The old empty-scratch safe-call witness cannot be expressed through that interface. Immutable borrowing prevents changing the validated inputs while the token lives. The row bound is checked. In the AVX2 sweep, each positive cell is filled with rounded-up eight-slot stores into n+8 capacity; intervening excess is overwritten by the next cell and the logical length is n. This is a source argument about the repaired boundary, not evidence that every unsafe path in the emitter has been checked.

2. **Cost and lifetime.** entry(1) invokes run_stage::build itself inside the Instant bracket. Selector, dispatch and checked construction are charged. The records/fallback classification outside the bracket is used for attribution, and the selected path is checked afterwards. The incumbent and other arms now walk into their own ll; the next pass reuses that overwritten buffer as production does. The separate phase sweep is attribution, not the judged whole-stage measure. R_fb uses only fallback-path entry times; the executable test distinguishes it from an accepted-block slowdown.

3. **Admission.** The detached verifier requires seven named corpora, the registered Wikidata digest and full source head, fifteen rounds, AVX2, pinned primary affinity, rotation, raw phase/entry timing arrays, present-path positive times and absent-path zero times. It refuses missing corpus/time evidence rather than reducing over only surviving intervals. copies.sh waits on every child and marks failures INCOMPLETE. sweep.yml preserves an incomplete contended condition as a failing step, even when the primary record has its own admissible reading. A primary VALID is not a statement that all contended copies passed. The registration removes correctness-based void retries.

4. **Inference.** Keeping the disclosed discovery history and the 0.90/0.95 thresholds is appropriate. The <=1.02 point-estimate screens are not simultaneous noninferiority guarantees. Q2 is a serial-front exploratory comparison, not a team prediction. M1 is a pre-touch contrast, M2 a store-only reference with different slot traffic, and M3 nominal cycles unless actual PMU cycles are used. Neither M nor the broader run-based construction line is refuted by a candidate's STOP.

## Residual limitations, nonblocking for this freeze

- The verifier does not authenticate raw records or synthetic contents merely by checking names and SHA-256 syntax. Source/head provenance and archived producer output remain necessary. The fixture deliberately uses placeholder synthetic hashes.
- A concrete metadata mutation passes check(): set ghz=-1 and, in the all-fallback synthetic-random file, rows_runs=123. It leaves all primary stage ratios unchanged. Tightening positive finite nominal GHz and zero accepted-path metadata for absent paths would improve diagnostic admission; do not interpret VALID as proving those exploratory fields consistent today. This is not a reason to block the primary timing freeze.
- black_box is not an inspected-machine-code proof of the store-only reference. Any strong claim about the exact stores retained or cache residency still needs that evidence.
- Fixed cyclic ordering over fifteen rounds is not perfect position balance; the registered result is for this protocol, workload and runner. It does not identify cold-cache causation, an i9 outcome, or a pipelined team's critical path.
- Any journal language about pausing a line is an operational opportunity-cost decision under C3, not a permanent rule that only one particular profile may reopen all run-based research.

## Evidence and validation

Read the exact-head src/run_stage.rs, examples/run_fed_stage.rs, bench/run_fed_stage.py, its tests/fixture, bench/copies.sh, sweep workflow and journal revision; compared against my previous report. PR API confirms that exact open head. Its reported checks include successful rust, miri and bench jobs; no check reports failure. I did not independently rerun Rust, Miri, fuzzing or native decoder code.

Locally ran the unchanged bench/test_run_fed_stage.py against its exact-head fixture: **37 tests passed**. These are fixture calculations and shell child-status tests, not decoder timing. tests.txt retains the output. audit.txt records the pinned routing audit: only its five already-resolved historical entries, with REVIEW routed to #145.

Expected: close the five earlier freeze objections. Observed: repaired checked boundary, whole-entry measurement and buffer lifetime, executable admission and narrowed claims. The freeze-readiness gap is closed; no performance gap has been measured or closed. The next useful evidence is the bounded frozen diagnostic, after applicable safety obligations are satisfied, because it prices whole entry versus phase attribution on one runner. Another discovery timing or broad implementation is not justified by this review.
