# Compact30: the reviews of the construction-cost harness (reports 195–197)

The harness at `e264eed`, with its recipe and records at `ce11857`
([2026-11-03-compact30-construction-harness.md](2026-11-03-compact30-construction-harness.md)),
was reviewed three times:

- **Report 195**, the guardian: conditional approval. Amendment A1, reading production's
  table by reinterpreting `&TVector` as `&Vec<u32>`, must be repaired with a read-only
  accessor in `src/bwt.rs`. A2–A8 are disposed of.
- **Report 196**, the researcher's independent review: CONDITIONAL PASS for the harness
  design, HOLD on the i9 capture. Besides A1 it finds three validation weaknesses (G2, G9,
  G7) and smaller memory-diagnostic issues. The full review is kept as received in
  [2026-11-03-compact30-harness-review/INDEPENDENT_HARNESS_REVIEW.md](2026-11-03-compact30-harness-review/INDEPENDENT_HARNESS_REVIEW.md)
  (sha256 `4b0a1e00…`).
- **Report 197**, the guardian on report 196: agrees, decides each finding, and asks for one
  bounded validation repair covering all of them before any workstation run.

The constructor at `e003a32`, the timed build boundaries, the paired design, the 20 rounds,
the statistical decision rules and the thresholds (warm four-core b4 upper ≤ 1.06714 and e4
upper ≤ 1.02 for PASS) remain accepted and unchanged. No construction timing and no i9 run
is authorised.

## Report 195: the guardian on A1 and the amendments (as relayed)

The construction harness is close, but I would hold the i9 capture for one narrow repair. The 61 analysis tests, successful VM dry run and machine-code comparisons are encouraging. I found one issue in A1 that I would not accept as an experimental limitation.

**The blocker: A1.** Claude's harness contains:

```
let v = unsafe { &*(t as *const TVector as *const Vec<u32>) };
```

Production defines `TVector` as a private-field wrapper around `Vec<u32>`, without `#[repr(transparent)]`.

Checking that the two types have equal size and alignment does not provide a general language-level guarantee that this reinterpretation is sound. The Rust Reference distinguishes the default representation from the explicit transparent-layout guarantee.

I would not introduce an avoidable layout assumption into a benchmark whose purpose is to validate an unsafe, performance-sensitive implementation.

The repair should be a narrowly permitted, read-only accessor in `src/bwt.rs`. For example:

```
pub fn as_slice(&self) -> &[u32] {
    &self.0
}

pub fn capacity(&self) -> usize {
    self.0.capacity()
}
```

These expose no mutable access. Claude can then compare production table contents, inspect capacity and derive the allocation pointer for its glibc-only memory accounting without type punning.

The existing `tv_mut` is used solely to inject a test failure. Replace that mechanism with corruption of a temporary expected value or reference copy, after the completion barrier. Do not add a public mutable accessor: production's unchecked walker relies on its table invariants.

This is a justified exception to our no-`src/`-changes rule because it changes observation, not production's construction algorithm. The production builder's compiled instructions must still be checked against the reviewed reference.

| Amendment | Decision |
|---|---|
| A1: `TVector` reinterpretation | Repair required |
| A2: thin timing wrappers | Accept, with their differing overhead documented |
| A3: frozen machine-code references | Accept; recheck after the accessor change |
| A4: G9/G10 also checked in dry runs | Accept |
| A5: independent P-table counting-scatter check | Accept |
| A6: dry-run peak memory required for capture | Accept |
| A7: mixed-size completion step excluded from subsets | Accept, but report that step separately as the protocol specifies |
| A8: failure-message wording | Accept |

I also noticed a minor memory-accounting discrepancy: the candidate's reported arena capacity includes `rec` and `raw` but not the constructor bitmap. This does not affect the timing decision. Claude should either include that capacity safely or label the reported figure as `rec + raw`, distinguishing it from complete representation memory.

The VM memory evidence is reassuring: the recorded peak was approximately 1.77 GB, below the estimated 1.86 GB. The workstation must still pass its own memory preflight.

## Report 196: the researcher's summary (as relayed)

Reviewed: `e264eed`, with the recipe and research records at `ce11857`.

Disposition: CONDITIONAL PASS for the harness design; HOLD on authorizing the i9 capture until the outstanding validation issues are resolved.

This is not a rejection of compact30 or its audited constructor. The new harness is carefully designed in several important respects, and I found no evidence that the two timed builder calls exclude essential construction work. However, I found weaknesses in how the harness establishes that a capture is valid.

| Severity | Finding | Recommendation |
|---|---|---|
| Medium — approval required | A1: `TVector` is reinterpreted as `Vec<u32>` without an explicit representation guarantee. | Obtain the guardian's acceptance or make the layout contract explicit. |
| Medium — gate weakness | G2 can hide a badly staggered four-worker step. It checks a weighted average of start skew, not the maximum individual step. | Clarify the registered criterion and add an outlier check if every step must be genuinely concurrent. |
| Medium — gate weakness | The dry-run memory requirement is not independently enforced. The collector can run a capture with peak RSS set to zero, and the analyzer can accept its memory preflight. | Require verified dry-run evidence, not merely a user-supplied positive number. |
| Low–medium — evidence overstatement | G7 reports the record arena as poisoned even when no memory was actually poisoned. This occurs on initial allocation and can recur on capacity growth. | Correct the coverage claim and record whether poisoning actually took effect. |

The second finding has a concrete counterexample. The analysis calculates the row-weighted mean of per-step start-skew ratios. With 57 equally weighted full-block steps, one step with 33.3% skew and 56 perfectly synchronized steps produces a mean of just 0.585%. That passes G2's 2% threshold.

The memory finding is similarly concrete. The official `just` recipe rejects a missing peak-RSS argument, but the collector defaults it to zero, and the analysis checks only that `required` is at least the computed estimate. Therefore a directly invoked capture can satisfy G9 without carrying evidence from a successful dry run. Even the recipe accepts any positive peak value without verifying its provenance.

These are validation defects, not demonstrated defects in the measured builder.

What I verified positively: construction work is inside both timed windows; production's one-pass builder is properly established; the scheduling and decision calculation are substantially faithful (pinned workers, spinning start barrier, completion barrier, checks performed afterward; exactly 20 paired rounds, confidence intervals from logarithms of round-level ratios; only the warm four-worker cost and completion bounds decide); the normalized machine-code evidence is persuasive within its scope (normalized instruction comparisons, not identical binary bytes); the committed VM analysis reports 10,416 exact build records, successful scheduling and correctness checks, zero major faults, and a peak RSS below the preflight estimate.

Smaller concerns: the RSS analysis does not independently validate that its six input records contain three valid runs per arm, with matching corpus and CPUs and successful high-watermark resets; the cold regime measures fresh Rust arenas, not guaranteed freshly mapped physical memory; the full/short-block completion breakdown omits the mixed step rather than reporting it separately. On A1, the researcher would not call the current cast demonstrated undefined behavior, but `#[repr(transparent)]` is the Rust-supported way to state the intended layout contract.

The companion probes reproduce the G2, G9 and G7 weaknesses without performing timing. The researcher could not replay the original dry-run record file, which is named in the manifest but not committed. They did not execute the Rust benchmark or run anything on the i9.

My overall assessment: the experiment is close to being ready. The timing architecture and primary statistical analysis are sound enough to preserve. The outstanding work is to make its validity claims as strong as the experiment intends them to be.

## Report 197: the guardian on report 196 (as relayed)

I agree with the researcher's CONDITIONAL PASS. The constructor and the core measurement design remain accepted, but the i9 capture should stay on hold.

The review found three genuine validation weaknesses beyond the A1 accessor issue we had already identified. They are worth fixing together rather than discovering another problem after you've run the workstation benchmark.

| Finding | Decision |
|---|---|
| A1 — unsafe `TVector` reinterpretation | Replace with safe read-only accessors. Already authorized. |
| G2 — one badly staggered step can pass | Require every full four-worker step to meet the existing 2% start-skew threshold. |
| G9 — dry-run peak memory is unverified | Capture must consume verified evidence from a successful dry run on the capture machine. |
| G7 — poisoning coverage overstated | Record actual coverage, including when allocation growth discards poisoned memory. |
| RSS diagnostic records | Validate completeness and provenance before presenting memory figures. |

**G2:** I choose the stricter per-step interpretation, rather than retaining the weighted average alone. The protocol describes synchronized four-worker construction. One step with 33% skew should not be accepted merely because 56 other steps started cleanly. The 2% numerical threshold remains unchanged. This is an explicit amendment before any construction performance capture, not a retrospective change based on unfavourable results. The analysis should report both the maximum per-step skew and the aggregate weighted statistic. This is stricter than the traversal experiment's aggregate gate. It may reject a run containing an isolated scheduling interruption; that is an acknowledged consequence of requiring every full step to meet the condition.

**G9:** A positive number supplied on the command line isn't evidence that a successful dry run measured it. I want the capture recipe to accept a specific completed dry-run artifact, verify its successful checks, source identity, corpus hash, machine/CPU identity and recorded peak RSS, then preserve that evidence with the capture. The collector and analyser must reject missing or inconsistent evidence even when invoked directly. There is no need for an elaborate new infrastructure—just a verifiable link between the dry run and capture.

### The instruction

The independent researcher reviewed the construction-cost harness at `e264eed`, recipe at `ce11857`, and recommends CONDITIONAL PASS with the i9 capture held.

I agree. The constructor at `e003a32` remains accepted and must stay unchanged. The timed build boundaries, paired design, 20 rounds and statistical decision rules are also accepted.

**Please complete one bounded validation repair covering these findings.**

1. **A1 — production table access.** As previously authorized, replace the unsafe `TVector`→`Vec<u32>` casts with minimal safe, read-only accessors in `src/bwt.rs`. Do not expose mutable table access. Replace P-arm failure injection with corruption of an expected/reference value outside timing. Verify that production's timed machine code is unchanged or report any difference.
2. **G2 — synchronized starts.** Before any construction timing, amend the protocol to require start skew ≤ 2% for **each individual all-full four-worker step**, in every arm, allocation regime and measured round. The current row-weighted average alone is insufficient. Report both maximum per-step skew and the original aggregate statistic. Add the researcher's single-outlier counterexample: one step at 33.3% skew among 56 clean steps must be INVALID. Do not change the 2% threshold or alter the barriers.
3. **G9 — verified dry-run evidence.** Replace the free-form peak-RSS argument with a required, verified dry-run artifact or receipt. Before capture, validate that its analysis passed, its source/toolchain/corpus identities match, its CPU selection and machine identity correspond to the capture, and its peak RSS is present and positive. Preserve the evidence and its checksum in the capture archive. The collector and analyser must reject missing, stale, mismatched or fabricated tiny peak values, including direct-binary invocations. The capture's required memory remains max(estimated requirement, verified dry-run peak), plus the already registered safety margin.
4. **G7 — initialization evidence.** Do not record `poisoned=true` when the record arena had no spare capacity or when a growing constructor replaced the poisoned allocation. Record actual poisoned capacity and whether poisoning survived allocation. If universal poisoned-coverage evidence is desired, use a separate untimed validation pass with sufficient preallocated capacity; do not preallocate or alter the measured construction arms. Preserve exact oracle and walker correctness checks.
5. **Memory diagnostics.** Validate the six RSS records before presenting their summary: three distinct runs per arm, matching corpus and CPUs, successful high-watermark reset, and sensible nonnegative measurements. Clearly identify any unavailable diagnostic rather than presenting incomplete records as valid. Memory-pressure and swap sources must not silently become zero when an expected readable source fails.

Preserve the accepted A2–A6 and A8 decisions. For A7, report the mixed-size construction step separately rather than silently omitting it from the subgroup breakdown.

**After the repairs**, rerun the relevant synthetic rejection tests, compilation and machine-code comparisons, untimed VM dry run and failure injections. Verify both builders' timed paths against their reviewed listings. Return a revised source pin, protocol, tests, dry-run evidence and proposed workstation commands.

No new constructor strategy, walker tuning, different workload, cache flushing, threshold optimization or production integration is authorized.

**No construction performance timing and no i9 run yet.** We will review this completed repair before authorizing the workstation dry run and subsequent registered capture.

### Where we stand

The researcher also confirms that the important parts of the harness are sound: both constructors perform their actual work inside the timing windows, production's one-pass path is established, and the paired analysis uses the correct primary denominators and confidence-interval calculation. So this is not a reason to revisit Astra's algorithm or reopen compact30's correctness review.

The next handoff goes only to Claude. Astra and the researcher have completed their current assignments.

Once Claude returns the repaired harness, the next useful action should be a properly evidenced workstation dry run—not another speculative redesign.
