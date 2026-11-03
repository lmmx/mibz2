# Compact30: the independent constructor audit (report 193) and the guardian's decision (report 194)

**The constructor correctness checkpoint is closed: PASS.** The researcher audited the frozen
direct constructor, `examples/c30walk/ctor.rs` at `e003a32` (blob `73b3179`), against the
[review handoff](2026-11-03-compact30-constructor-review-handoff.md), and found no material
correctness or memory-safety defect for the baseline 64-bit x86-64 research build. The
guardian accepts the constructor unchanged for cost measurement, accepts the
[construction-cost protocol](2026-11-03-compact30-construction-protocol.md) at draft 3
(`371c343`), and authorises the measurement harness and untimed VM validation. No measured
benchmark or i9 run is authorised.

**The audit's evidence**, kept in
[2026-11-03-compact30-constructor-audit/](2026-11-03-compact30-constructor-audit/) as
received:

| file | what it is | sha256 |
|---|---|---|
| `INDEPENDENT_SOURCE_AUDIT.md` | the full audit: six areas, proofs, source locations, findings | `8b9b856b…` |
| `independent_invariants.py` | the researcher's independent Python state-machine model and counting-scatter oracle (not Rust, not Miri, no timing) | `a38b5565…` |
| `independent_checks.json` | its recorded output: all assertions passed | `48f9c405…` |

Rerunning `independent_invariants.py` here (`python3 -I`) reproduces `independent_checks.json`
exactly.

**Scope and limits, as the audit states them:**
- A source-level PASS for the pinned source on baseline 64-bit x86-64. It is not a claim of
  exhaustive memory-safety verification, and must not be extended to 32-bit targets: the
  malformed-count arithmetic proof assumes a 64-bit `usize`.
- The researcher did not compile or run the Rust source, or Miri. The archived Rust test
  logs were inspected, not rerun. The Python probes support the algorithmic proof, but can't
  establish Rust pointer provenance or compiler behaviour.
- Miri's archived run covered 942 small cases with zero wide promotions. That is an evidence
  limit, not a demonstrated defect.

**Nonblocking findings** (none changes the frozen constructor before the cost experiment):
1. **Low, invariant fragility:** in `canonicalise` (lines 667–675), `rank == raw.len()` alone
   doesn't establish that provisional RAW indices are unique, and the debug-only check misses
   one duplicate case. Uniqueness is established structurally: each RAW-creation path
   allocates a fresh index and marks one previously unmarked group. A stronger runtime check
   would belong in a future validation-only change, not in the timed implementation.
2. **Low, harness gap:** the empty block (n = 0) is tested through the counted build only.
3. **Low, harness gap:** the single poison value 0xA5A5 could coincide with a valid offset,
   and a capacity growth discards the poisoned allocation.
4. **Platform qualification:** 64-bit x86-64 only, as above.

## Report 193: the researcher's summary, as relayed

PASS — source-level audit

I completed an independent review of the frozen compact30 direct constructor at commit [`e003a32`](https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs).

No material correctness or memory-safety defect was found. My recommendation is that this exact implementation is suitable to proceed to a separately approved construction-cost experiment, without changing the constructor or walker.

This is a source-level PASS for the intended 64-bit x86-64 research build, not a claim of exhaustive memory-safety verification.

### Source-level findings

I read the complete handoff and all 1,963 lines of `ctor.rs`, including its correctness harness, together with the parent implementation defining `Rec`, `RawG`, the frozen oracle and navigation conventions.

| Audit area | Finding |
|---|---|
| Unsafe initialization | No issue found. The destination cursors cover each required slot exactly once on success. Headers are written before subsequent reads. The only `rec.set_len` occurs after completing the records, padding and PARK. |
| Release-mode bounds | No issue found on x86-64. The unchecked release-path assumptions follow from the bounded cursor and source-position invariants, rather than depending on `debug_assert!` executing. |
| RAW promotion | No issue found. The 65,533/65,534 distinction is correct. Earlier fields are reconstructed before replacement; an already-RAW group cannot be promoted again through the normal paths. |
| RAW canonicalization | No reachable defect found. Provisional indices are unique by construction, and the in-place cycles preserve records and restore padding. One defensive assertion is weaker than this invariant, discussed below. |
| Reuse and failure | No issue found. Rejected builds expose empty arrays. Reused allocations and bitmap state are reset before a subsequent accepted build. |
| Harness independence | Adequate for this gate, with limitations. The oracle uses a full successor array rather than the new constructor's algorithm, and the harness checks complete record contents, walker output and marker restoration. Some constants and helpers remain shared. |

#### One particularly important proof

In [`run`, lines 535–546](https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs#L535-L546), the encoded destination cursor advances by the number of rows plus the two reserved labels crossed at each 30-row boundary.

The run verifies its final cursor position against the byte bucket's limit before writing anything. The scalar path has an equivalent check.

This establishes that writes stay within the allocated record groups even when supplied character counts are incorrect. A build that cannot account for every destination fails before publishing its partially initialized arena.

That was the central unsafe-initialization obligation.

### Issues and evidence limits

I found one low-severity defensive weakness, not a triggering defect in the frozen implementation.

At [`canonicalise`, lines 667–675](https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs#L667-L675), checking that the number of RAW bitmap entries equals `raw.len()` does not independently establish that all provisional indices are unique. The debug-only check is also insufficient in one duplicate-index case.

If an internal invariant were violated, duplicate indices could cause incorrect ordering or nontermination. However, I traced every RAW-creation and promotion path: each allocates a fresh index and marks one previously unmarked group. I found no source path that creates the duplicate. This is not a benchmark blocker.

There are three further limitations worth recording:

- The zero-length synthetic case exercises the counted constructor but not the uncounted variant.
- The single poison pattern could theoretically coincide with a valid field value.
- The malformed-count arithmetic proof assumes 64-bit `usize`; it should not be extended to 32-bit targets without additional checks.

None requires a change to the frozen x86-64 implementation before this particular experiment.

### Independent checks

9,405 cursor identities; 43 offset-boundary cases; 50 RAW permutation cases.

The independent probes also covered 1,092 small oracle comparisons, 110 malformed-count rejections and 10 large/reuse-shaped cases, including the maximum accepted size of 983,010 rows.

They explicitly track whether a header or field has been initialized before an abstract read. All assertions passed.

These tests execute an independently written Python model, not the Rust source. They support the algorithmic proof but cannot establish Rust pointer provenance or compiler behavior.

I separately inspected the archived Rust test logs. They support the recorded 174,591-case full synthetic run, 11-corpus checks and isolated RAW-canonicalization tests.

The Miri limitation remains important: 942 small cases passed, with zero wide promotions exercised. I would not describe the constructor as comprehensively Miri-verified.

### Final recommendation

PASS — no material source-level issue found in `e003a32`.

The constructor meets the requested correctness and memory-safety review gate for the baseline x86-64 research build. Its next step can be a separately registered construction-cost experiment, subject to your review and authorization.

I would preserve this source pin, avoid redesigning the constructor, and keep any optional validation hardening separate from the benchmarked implementation.

No repository changes, performance measurements or i9 runs were made during this audit.

## Report 194: the guardian's decision, as relayed

The independent audit passes. I accept the researcher's recommendation and close the constructor correctness checkpoint as PASS.

This clears the main blocker to the next phase: measuring whether Astra's direct constructor is economical enough to retain compact30's four-core advantage.

The researcher found no reachable correctness or memory-safety defect in the frozen `e003a32` implementation. Importantly, the review examined the actual Rust source and its unsafe invariants—not merely Claude's successful test logs.

The remaining qualifications are manageable:

- The RAW permutation's index uniqueness is structurally established, although one defensive assertion would not detect every hypothetical internal corruption.
- Miri's coverage is partial, particularly for wide promotion.
- The zero-length case and single poison value leave minor testing limitations.
- The safety conclusions are specific to the reviewed 64-bit x86-64 research build.

None warrants changing the accepted constructor before its first cost comparison.

I also reviewed construction-protocol draft 3 at `371c343` through GitHub. Its matched build boundaries, warm/cold distinction, registered decision rules and resource safeguards are sufficiently specified to proceed to harness implementation.

### Decision: authorize the measurement harness, not the benchmark run

| | |
|---|---|
| Independent constructor audit | PASS |
| Construction protocol draft 3 | Accepted |
| Harness implementation and untimed VM validation | Authorized |
| i9 construction-cost capture | Not yet authorized |

The registered construction-cost decision remains:

| Outcome | Warm four-core criteria |
|---|---|
| PASS | Upper 95% bounds: `b4 ≤ 1.06714` and `e4 ≤ 1.02` |
| FAIL | Lower 95% bound exceeds either threshold |
| INCONCLUSIVE | Neither condition holds |
| INVALID | Any registered validity check fails |

Cold-allocation costs, memory footprint and one-core costs remain reported evidence rather than additional investment gates.

### The instruction

The independent researcher has completed the source-level audit of compact30's direct constructor at `e003a32`.

**Disposition: PASS.** No material correctness or memory-safety defect was found for the reviewed x86-64 research implementation.

The review independently examined all 1,963 lines of `ctor.rs`, including unsafe record initialization, release-mode bounds invariants, wide promotion, RAW canonicalization, allocation reuse and harness blind spots. It also performed independent invariant probes.

The review notes one low-severity defensive limitation in RAW provisional-index uniqueness checking, but establishes uniqueness structurally through the reachable creation paths. Miri's lack of wide-promotion coverage remains an evidence limitation, not a demonstrated defect.

**Guardian decision: accept this constructor unchanged for cost measurement.**

I have also reviewed `docs/journal/2026-11-03-compact30-construction-protocol.md`, draft 3 at `371c343`, and accept its experimental design and registered decision rules.

You are now authorized to:

1. Archive the researcher's PASS report and supporting evidence, recording its scope and limitations. Close the constructor correctness checkpoint without altering the accepted implementation.
2. Implement the minimum construction-cost collector and analysis specified in protocol draft 3. Compare `Ctor::build::<false>` with production's public `bwt::build`, on identical inputs. Preserve the complete timed construction boundary.
3. Implement the warm and cold regimes, actual thread CPU-time measurement, construction-step completion time, exact table checks, one-pass production precondition, memory preflight, calibration, major-fault recording and full schedule validation.
4. Add synthetic regression tests demonstrating that malformed schedules, wrong denominators, missing calibration, correctness failures, memory-gate violations, major faults and insufficient measured rounds cannot produce a PASS or FAIL decision.
5. Verify the source pin, toolchain and generated assembly of **both timed functions** after the harness has been added. The original disassembly addresses are not an identity guarantee. Preserve the accepted constructor and walker algorithms.
6. Run untimed end-to-end VM dry runs and injected failure tests. Confirm that the selected CPU placement, barriers, allocation regimes, error handling, analysis and archiving behave as registered.
7. Return the new pinned collector, analysis tests, machine-code evidence, dry-run records and one proposed i9 command for guardian review.

**The thresholds remain frozen:** warm four-core `b4` upper ≤ 1.06714 and warm four-core `e4` upper ≤ 1.02 for PASS. Cold costs and one-core results are reported, not deciding.

Do not change `Ctor::build`, the walker, the record format or production code to improve performance. Do not introduce another constructor variant or conduct tuning.

**No measured benchmark, i9 run, production integration or full-decoder speedup claim is authorized yet.** We will approve the actual capture only after reviewing the completed harness and its preflight evidence.

### What happens next

The researcher has finished the required audit, and Astra has finished the design assignment. Neither needs another message now.

The interesting question is finally economic rather than architectural: can the constructor's 74.34% bulk-path coverage offset its fragment processing, scalar fallback and RAW canonicalization sufficiently to stay within the 6.714% permitted construction overhead?

We have enough correctness evidence to measure that question. We don't need another design round before doing so.
