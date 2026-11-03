# Compact30: the v2 construction-cost harness's i9 dry run (reports 213–215)

**APPROVED: one untimed i9 dry run of the v2 harness. No performance capture.**
- **Report 213:** the guardian reviewed the v2 protocol, the analyser's G1/G2, the
  regression tests, the recipe, the machine-code identity and the VM evidence at
  `1dc2edf`, and approved one untimed dry run.
- **Report 214:** the researcher independently supported exactly that, and no more.
- **Report 215:** the guardian confirmed that no further review is needed.

**The approved command,** on the i9 in `/home/louis/dev/mibz2`, on `research/compact30-constructor`:

    just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 dry-run

The recipe builds the frozen harness at `fa21f33`, in its own pinned worktree
(`target/c30-construction-v2-pin`), with rustc 1.97.0. It checks the corpus hash first.

**Conditions:**
- The run stops on any failed precondition. **No automatic rerun and no code change.**
- The archive and analysis are kept whether the run succeeds or fails.
- The dry-run schedule, G3, G4 and G6–G10 must pass. Its timing records are **not
  performance evidence**.
- A successful dry run does **not** authorise the measured 20-round capture, which needs a
  separate decision.
- Also not authorised: constructor tuning, changes to the validity gates, walker changes,
  production integration, and any whole-decoder performance claim.

**Fixed as of this approval:**
- The revised G1/G2 rules are accepted as prospectively registered for v2. That includes
  the one-outlier-per-arm allowance. They **must stay unchanged once workstation measurement
  begins**.
- The economic thresholds stay as they were.
- v1's INVALID capture and STOP, and the traversal's VALID ADVANCE, are unchanged.

**The targeted Miri evidence** (69/69 cases exact) stays on its own branch,
`research/compact30-ctor2-miri` at `e57a225`, in
`docs/journal/2026-11-03-compact30-constructor-v2-miri.md` there. Neither the guardian nor
the researcher requires merging it into the frozen harness, and no further Miri work is
needed.

**Result:** pending. The archive, its checksum and the analysis will be recorded here when
they arrive.

## Report 213: the guardian's approval, as relayed

Kept as relayed. Link tracking parameters are removed; the words are unchanged.

Guardian decision: APPROVED — one untimed i9 dry run of the v2 construction-cost harness. No performance capture yet.

I've reviewed the pinned v2 protocol, the analyzer's G1/G2 implementation, the regression tests, the workstation recipe and the archived VM dry-run evidence. I found no blocking issue for this next step.

The targeted Miri work is also complete. All 69 additional cases passed, including the exact wide-offset boundaries, forced RAW, malformed inputs and allocation reuse. There is no reason to rerun Miri on your i9.

### Review outcome

| Area                       | Disposition               |
| -------------------------- | ------------------------- |
| V2 constructor correctness | Independent audit PASS    |
| Targeted Miri              | 69/69 passed              |
| V2 economic thresholds     | Unchanged                 |
| G1/G2 revisions            | Preregistered and tested  |
| Production machine code    | Matches reviewed listing  |
| V2 machine code            | Matches audited listing   |
| VM dry run                 | Passed                    |
| V1 historical result       | Unchanged: INVALID / STOP |
| i9 untimed dry run         | Authorized                |

The protocol's main remaining limitation is that the full 20-round statistical decision has only been exercised against synthetic records. That's expected: the dry run deliberately suppresses performance conclusions.

I accept the revised G2 rules as prospectively registered for v2, including the one-outlier-per-arm allowance. They must remain unchanged once workstation measurements begin.

### What to run

On your i9, in `/home/louis/dev/mibz2`, with the usual corpus available:

```
just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 dry-run
```

This command uses the frozen harness at `fa21f33` and its own pinned worktree. It performs the correctness, scheduling, identity, memory and evidence checks without producing an experimental performance verdict.

The run must stop on any failed precondition. Preserve the resulting archive and analysis, whether successful or unsuccessful. No automatic retry.

### Message for Claude

Guardian review of compact30 constructor v2, protocol and harness at `1dc2edf`:

**APPROVED: one untimed i9 dry run only.**

I reviewed the protocol, the analyzer's G1/G2 implementation, its regression tests, the pinned workstation recipe, machine-code identity and VM validation evidence. No blocking issue was identified for an i9 dry run.

The 69 additional targeted Miri cases passed on the VM. Archive this evidence on its separate research branch; no further i9 Miri work or merge into the frozen harness is required.

The approved command is:

```
just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 dry-run
```

Use the pinned recipe and harness at `fa21f33`, preserving constructor v2 at `8beffe2` and all existing validity and economic thresholds.

Archive the console, analysis, manifest, checksums, machine identity, listing checks, memory evidence and any failure record. Verify that the dry-run schedule, G3, G4 and G6–G10 pass. Do not interpret its timing records as performance evidence.

If any condition fails, stop without an automatic rerun or code change.

**Not authorized:** the 20-round measured capture, constructor tuning, changing validity gates, modifying the walker, production integration or any whole-decoder performance claim.

Return the i9 dry-run archive, checksum and analysis for Guardian review. The measured capture requires a separate decision.

Constructor v1's INVALID/STOP and compact30 traversal's VALID ADVANCE remain unchanged.

We're now one successful workstation preflight away from deciding whether to measure v2's construction cost. The central economic question—whether the much shorter append path outweighs deferred normalization—is still unanswered, but the correctness work is in good shape.

## Report 214: the researcher's assessment, as relayed

Kept as relayed. The paragraph breaks and headings the relay lost are restored, and link
tracking parameters are removed. The words are unchanged.

This is a meaningful step forward for v2. I checked the new GitHub records, including the targeted Miri logs, the v2 protocol and the archived VM analysis.

My assessment: the evidence supports sending the v2 harness to the guardian for approval of an i9 dry run. It does not yet justify authorizing a performance capture.

### What has improved

The targeted Miri tests address the most important coverage gaps from my earlier audit. All 69 additional cases completed without reported undefined behaviour, including:

* The exact 65,533/65,534 ordinary-to-RAW boundary.
* Forced RAW construction and malformed-count rejection.
* Pending portions crossing epoch boundaries.
* Reusing allocations after shrinking and growing blocks.
* A wide group spanning multiple epochs.

Together with the earlier 473 completed cases, this materially strengthens the memory-safety evidence. It remains targeted testing, not exhaustive verification.

The archived [v2 VM dry-run analysis](https://github.com/lmmx/mibz2/blob/1dc2edf/docs/journal/2026-11-03-compact30-construction-v2-harness/vm-dry-run/c30-analysis.json) also reports every applicable schedule and non-timing gate passing. The full 434-block corpus was checked, with complete coverage in the separate poison-validation pass.

### Two things matter for the guardian

First, G1 and G2 have changed deliberately. V2 gates production's warm four-worker stability, and G2 permits one isolated synchronization outlier per arm, bounded by a 10% maximum and a 2% aggregate threshold.

These are prospective rules for v2 only. I verified that the protocol explicitly preserves v1's INVALID outcome and STOP decision. That's essential experimental discipline.

Second, the performance analysis hasn't been exercised on real v2 capture data. Its decision logic has synthetic regression coverage, but the dry run suppresses the actual timing figures. This is an acknowledged limitation, not evidence that the analysis is defective.

### Recommended next decision

I'd support the guardian reviewing the frozen harness at `fa21f33` and, if satisfied, authorizing one untimed i9 dry run, using the proposed command:

```
just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 dry-run
```

That authorization should be limited to the dry run and archiving its evidence. No automatic capture afterward.

The Miri branch can remain separate; it does not need merging merely to preserve its evidence.

Most importantly, the shorter v2 scalar loop is now a credible, correctly implemented architectural improvement—but not yet a demonstrated speed improvement. That is exactly the distinction the next construction-cost experiment needs to resolve.

## Report 215: the guardian's confirmation, as relayed

The researcher and Guardian are aligned. No further review is needed before the i9 dry run.

The researcher's recommendation was to authorize one untimed workstation preflight. I already granted exactly that authorization in my previous response, after reviewing the frozen harness and protocol.

I also rechecked the VM analysis on GitHub at `1dc2edf`. Every applicable schedule and non-timing gate passed. The performance decision was correctly suppressed.

### Current authorization

| Activity                          | Status         |
| --------------------------------- | -------------- |
| Constructor v2 independent audit  | PASS           |
| Additional targeted Miri tests    | 69/69 passed   |
| VM harness validation             | PASS           |
| One untimed i9 dry run            | APPROVED       |
| Measured 20-round i9 capture      | Not authorized |
| Constructor tuning or integration | Not authorized |

### Next action

The approved command remains:

```
just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 dry-run
```

Claude should archive its results and return the archive checksum and analysis for Guardian review. A successful dry run does not automatically authorize the measured capture.

The researcher's latest message adds useful independent confirmation, but it doesn't change the plan. There's no need for another Astra consultation, further Miri testing or a new round of protocol revisions.

We can proceed with the already-approved dry run.
