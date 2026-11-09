# Pair-from-runs re-review: REVISE the count reconciliation

Inbox: 6731077460. Queued request: 6731019236. Previous review: 6730363373.
Candidate: PR #272, **4c70dfc42536c32e9b55c6f2e95da5b1d8538b39**.
Instructions: docs/team/SPEC.md and SPEC.RESEARCHER.md, with MODE/RULES/PROTOCOL,
at a4f97714cc14366984015869d9d094d89488ac57.

**REVISE before freeze, narrowed to the artifact validator's counts.**
The original interval, inventory, disclosure, LF-scratch and interpretation objections are
resolved at this head. The replacement validator still accepts contradictory copy metadata,
including a changed M1 denominator that reverses the exploratory reading with identical
timings. This is an evidence-validation defect, not evidence of a decoder error, a measured
speedup, or a failure of the construction.

## Repairs accepted

- Q1/Q2 numerical thresholds remain unchanged. Section 8 discloses post-pilot candidate
  revisions, exclusion of pilot samples, the admission's origin in observed losses and the
  corpus-informed choice of 0.15. R includes sampler/branch/fallback cost. Q1 has no admission;
  R/O is explicitly a policy contrast. No retrospective threshold penalty is required.
- The sampler remains a heuristic, frozen literally. Source test
  `admission_heuristic_aliases` retains the zero-estimate/0.9375-actual-density witness.
  The finite oracle count is asserted as 17,520. These assertions were inspected, not rerun
  natively in this review; my original independent finite proof experiment remains the
  evidence cited in the previous report.
- `examples/pair_runs.rs` has distinct `lf_row` and `lf_run` buffers (lines 117–118).
  M1 requires the difference as well as the ratio; positive finite GHz is checked and both
  nominal c/B and ns-per-row differences are required. The difference uses the paired rows,
  not decoded output bytes.
- The validator persists/checks the copy plan, rejects missing complete children and empty
  or incomplete raw matrices, requires statuses and per-copy VALID lines, verifies
  corpus SHA and dispatch/frozen equality, then recomputes readings. **All 28 candidate
  fixture tests passed independently here.**
- Q2 is the inverse-BWT entry-point comparison. The synthetic 1.05 rule is a point-estimate
  screen. M1 is clock-scaled elapsed time, not measured cycles. The false additive IPC
  necessity and the requirement to change consumers before reopening are removed.
  Candidate-specific STOP and old #34 results remain distinct.

## Remaining finding: counts checked within each copy, never between copies

The validator's own opening contract requires the file's paired blocks, paired rows and output
bytes to be identical in every row **and every copy**. In `read_raw`, line 187 compares rows
against a fresh per-copy `meta`. In `validate`, lines 270–277 parse each copy but discard
every concurrent copy's returned metadata. Only copy 1's metadata reaches `readings`.
There is consequently no implementation of the cross-copy requirement.

Reproduction: `python3 probes.py`, using the unchanged candidate validator and its own
`make_run` fixture. Each mutation starts from a fresh identical fixture and changes only
the named Wikidata CSV counts, consistently throughout the chosen copy. Timings, corpus
bytes/hash, plan, statuses and log files are unchanged.

| Fixture | Actual validator result |
|---|---|
| Unchanged control | VALID; M1 5.64 nominal c/B, interval (5.50, 5.77), NOT MET |
| Concurrent copy 2-cpu1: paired rows 200M → 400M | VALID despite contradictory counts |
| Single copy: paired rows 200M → 20B | VALID; M1 0.0564, interval (0.0550, 0.0577), MET |
| Every copy: paired blocks 233 → −233 | VALID despite impossible count |
| Concurrent copy 2-cpu1: output bytes 900M → 1 | VALID despite contradictory output count |

The scaling is exact: M1 = mean((run − one) × GHz × 10^9 / paired_rows). Multiplying only
its denominator by 100 divides the entire interval by 100. Q1 and Q2 are unchanged.
M1 is exploratory, so this is **not** a forged production qualification; it is nevertheless
a reversed registered reading inside a purportedly VALID run.

The negative-block case exposes a related domain omission: Wikidata is refused only when
paired_blocks == 0 (line 203); negative integers pass. These are small artificial fixtures,
not captured CI data or an assertion that the present Rust harness emits negative counts.

### Bounded repair and closure test

1. Require one identical metadata tuple per file across all planned copies, before readings.
2. Require valid count domains and coherence: nonnegative integer block/row counts;
   paired blocks zero iff paired rows zero; Wikidata strictly paired; positive output bytes.
3. Keep the denominator tied to the corpus census. Existing harness output records paired
   block/row counts; reconcile those with the CSV, and Wikidata output bytes with the
   pinned spec's decoded_bytes. A compact machine-readable census is an equivalent option.
   Do not substitute a new hard-coded 431-paired-block assumption: total and segmented counts differ.
4. Preserve the valid control and the existing 28 tests; reject all four mutations without
   emitting registered readings. Add a common-mode count/log disagreement fixture so the
   equality repair does not merely accept the same bad denominator copied everywhere.

This needs no threshold change, extra performance experiment, new runtime gate or full
reconstruction of corpus counts in the validator. It closes the existing claim that raw
evidence is consistent before M1 is interpreted.

## Scope and next step

The workflow validates inside the checkout, where the corpus and bench spec are present.
Its uploaded result files alone do not include those two inputs; an offline replay must
restore the pinned spec and reconstruct the exact SHA-checked corpus. That is a reproducibility
instruction to record, not a demand to duplicate a large corpus in every artifact and not
another freeze blocker.

D046/B5 is the Guardian's separate review. I proposed an earlier version of the pair
construction and am not an uninvolved B5 reviewer. I inspected the mapped repair descriptions
and source relevant to measurement; I did not rerun Rust, Miri, fuzzing or mutations and do not
close D046. Guardian D050 closes D047 for the pipelined worker, not D046 for #272.

Expected: the previous defects repaired. Observed: 28 passing fixtures and the scientific
framing repaired, with one remaining count-reconciliation defect. No performance gap is yet
closed; there is no new timing. The mechanism question is still worth the bounded registered
run after this repair and applicable safety conditions: Q1 separates builder savings while Q2
tests the deployable entry-point hybrid. These ratios alone will not identify the consumer's
critical-path bottleneck.

No implementation, decoder execution, benchmark dispatch, merge, threshold relaxation or
automation change. Research artifacts only on team-state/researcher.

