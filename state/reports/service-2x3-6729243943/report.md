# #258 workstation admission: HOLD

Researcher review of inbox **6729243943**, dispatching **6729027210**.
Reviewed head: **3e4477d035e96310eb4ac3d4014e01e13ffaa33c**.
Frozen inputs: **21cf20a49e3a90817e718a14abcc254679de6b2a**.
Instructions: docs/team/{SPEC,SPEC.RESEARCHER,MODE,RULES,PROTOCOL}.md at
**a4f97714cc14366984015869d9d094d89488ac57**; Guardian conditions applied within that scope.

**HOLD for PROTOCOL §9 queue admission.** The experiment is worth preparing: the missing
matched two-team controls directly address the previous comparison's ambiguity. The freeze
and narrow harness change check out. The executable report nevertheless accepts malformed
evidence, and its final causal readings claim more than these comparisons identify. Resolve
H1–H3 below before spending the workstation run; no decoder modification or new timing is
needed to resolve them. This is neither a line STOP nor a production qualification.

## H1 — VALID does not imply the registered evidence is present

I exercised the exact report with synthetic records, not decoder timings. `probe.py` imports
the reviewed `service_2x3.py`, `service_core.py`, and `team_size.py`. Only the report's
`git rev-parse HEAD` subprocess is replaced with the reviewed SHA. The control contains all
120 rows, registered round IDs and thread counts, N1 ratio 1, ten-core lscpu metadata, and
an occupancy summary. Full results are in `probe.json`.

| Mutation of that control | Observed result | Interpretation |
|---|---|---|
| Delete `hash`, `out`, or `wall` from every row | KeyError, no VALID | The earlier deleted-field acceptance holes are **not** reproduced here; direct indexing fails closed, albeit without an INVALID report |
| Every hash and output size null | VALID | Equal missing values stand in for byte equality |
| Every hash empty and size zero | VALID | Empty evidence passes for the registered nonempty workload |
| Every wall = -1 and CPU = -6 | VALID | Positive ratios hide impossible measurements |
| Every corpus name changed; or corpus digest file absent | VALID | The report does not bind records to the registered workload |
| Trace contains only `complete_rounds: 1` | VALID, saturation UNRESOLVED | Contradicts the registration's explicit UNRESOLVED → INVALID requirement |
| Trace result says one thread and one service thread | VALID | The expected filename is accepted without checking result configuration |
| One round, duplicate round, incorrect observed plan counts, DRY marker, or NOT-FROZEN marker | INVALID | These rejection paths work |

The previous external-base-path bug is not the same experiment here: this recipe selects
six named arms from one binary, with no separate base executable. Substituting an unknown
arm makes this report INVALID. Do not carry the old finding over by name.

**Before admission:** require usable output evidence for this nonempty corpus; finite,
positive wall/CPU values; consistent registered corpus identity; and the actual service
trace's configuration and required finite metrics. Emit INVALID with a reason for absent
or malformed fields. Cover these cases with cheap report fixtures. The recipe currently
permits CORPUS overrides outside DRY too; a production-sized alternative must not silently
become the registered corpus. Preserve that workload, rather than authorising a new one.

These are inexpensive harness/evidence checks under C6, not new statistical thresholds.
Checking a bundle afterwards could quarantine these defects, but cannot make the current
report's VALID assertion true. No reason to leave deterministic acceptance holes until
after scarce workstation time is spent. Amend and freeze the report before running it.

There is a narrower provenance limit too: the report checks absence of NOT-FROZEN and
agreement with the ambient checkout, not the actual frozen-tree comparison. The recipe
does perform that comparison. Thus the report alone is not a detached attestation; bundle
admission must retain the exact recipe/head/frozen identity and corpus digest. The fixture's
VALID is evidence about report predicates, not a forged authorised workstation submission.

## H2 — useful contrasts; an overstrong final cause

C1 holds two teams of three and the eight-block window fixed. It estimates the service
**bundle** relative to plain two-team decoding: two added service threads, displaced serial
work, feeding, hub synchronisation and hand-offs. The registration correctly discloses
this. P2 compares pipelined two-team and three-team policies; thread count, tables and
window change together. Neither comparison isolates cache residency, window size, or
service execution location individually.

The report's `readings()` computes its overall answer from P1 and C1; **P2 never affects
that answer**. Counterexample using illustrative wall times, not measured data:

| default | plain 2×3 | pipe 2×3 | svc2 |
|---:|---:|---:|---:|
| 1.00 | 1.10 | 0.90 | 0.95 |

P1 = 1.0526, C1 = 1.1579, P2 = 1.1111, while S1 = 0.9474. All three judged speed
contrasts can be GAIN; the simpler pipelined two-team alternative still beats svc2. With
CPU NULL the current map prints FOR SVC2 and “the service threads cause it.” C1 establishes
a bundle advantage against **plain2**, not that service is the explanation uniquely needed
for P1, nor that svc2 is the better candidate.

The reverse label “TWO TEAMS, NOT THE SERVICE THREADS” also overstates a C1 NULL: an interval
inside the registered ±3% region permits small nonzero effects. Report bounded uncertainty,
not absence of an effect. P1/C1/P2 can support multiple contributors or leave their roles
unresolved; the binary explanation is not exhaustive.

The existing secondary contrasts already permit a sharper descriptive decomposition.
For geometric means over these same paired rounds:

`P1 = S1 × P2 = C1 × S2`.

The identities follow per round, then on the log scale. They are comparison decompositions,
not identified biological-style mediation effects. Marginal CI endpoints cannot simply be
multiplied: the shared observations induce covariance. Retain S1 as secondary unless Louis
changes the judged family; the immediate correction is honest interpretation, not a quiet
multiplicity or acceptance-policy change.

The CPU rule is reproduced faithfully: INCONCLUSIVE is “not CPU COST” and can print FOR
SVC2. That means **no detected CPU cost**, not a demonstrated absence of cost. Keep both
co-primary outcomes visible. I do not replace this frozen rule or the per-comparison 95%
intervals with new acceptance criteria. C6 changes remain Louis's.

## H3 — the trace does not prove that service capacity causes the null

The inherited saturation predicate uses the **maximum** service utilisation ≥0.90 and
prep starvation or finish backlog. Those observations can coexist with load imbalance,
queue policy or scheduling constraints. One busy service thread is not proof that two
threads' aggregate capacity is exhausted. A C1 NULL/regression does not identify its cause.

Therefore “SATURATION BINDS; the null is the service's capacity, not the idea” is unsupported
by this design. Keep the registered numeric flag and C1 outcome, but describe capacity as
a hypothesis consistent with them. A matched capacity intervention or a defensible
critical-path argument would be needed to establish the stronger causal claim. This review
does not authorise such a run or require expanding this one. Likewise, prep starvation
below that utilisation threshold does not uniquely identify queue lookahead.

## Checks that passed, and their boundaries

- **Freeze:** the compare from 21cf20a4 to the reviewed head contains exactly one commit,
  changing only `bench/service-2x3.frozen`. Its target is correct. The recipe checks tracked
  cleanliness, untracked measured inputs, and frozen-vs-head changes for source, Cargo,
  toolchain/configuration, examples, tests/common, the report/helpers, corpus tooling,
  bundle helper, recipe and registration. This closes the root-input omission from earlier
  work; no reason to repeat that old objection here.
- **Harness:** the new arms call existing `parallel_planned(Teams{2,3})` and
  `parallel_pipelined_levers(2,3,...,PIPELINED_LEVERS)`, both with `SLOTS_PER_THREAD*2`.
  #258 changes five files, no `src/`. A separate compare against full prototype commit
  `9ef3c17d12b0fcfc7609322b1d78e63ede8acdd3` also shows no `src/` changes.
- **Statistics:** the report imports the parent's outcome, thread label, timeline and
  failure-mode predicates unchanged; its paired log-ratio geometric means and t intervals
  use twenty rounds. GAIN lower bound >1.02, NULL inside [0.97,1.03], and N1 both bounds
  inside that interval agree with the written rules. The predicates' reuse does not
  validate the causal wording built on them. No familywise 95% guarantee is inferred.
- **C1 justification:** new controls address a real missing comparison; the old 1.045 result
  is motivation, not fresh confirmation. From 209.7 MB/(2×558–583 MB/s), plain2 predicts
  0.180–0.188 s; dividing by 0.136 gives C1 ≈1.32–1.38. A transferred 1.07 pipeline factor
  gives 0.168–0.176 s and P2 ≈0.82–0.85. These are explicitly conditional forecasts. No
  gap has been closed empirically by this review. The six-minute estimate fits the stated
  bound; the run's value is separating product comparisons, not qualifying production.
- **CI:** 21 latest checks on the head: 19 success, Release and Publish crate skipped.
  Wheel run 42475361875, first-attempt aarch64 job 128715377975, fails during Install maturin
  with “gzip: stdin: not in gzip format” and tar/docker failure, before crate compilation.
  The successful latest check supports the recorded class-B recovery; this was not a
  performance-result retry. The check IDs are retained in `evidence.json`.
- **Dry run:** the PR body reports the exact head, expected INVALID on the four-core DRY
  run, matching output across all six arms and a complete trace. I read that record; I did
  not retrieve/replay its deleted scratch bundle. “Untimed” should mean excluded from
  timing evidence: the recipe still invokes the wall-recording AB harness. No local
  measurements are admitted here.
- **Safety:** unchanged source and correct constructor selection are not new B5 proof.
  The parent registration discusses cancellation, stage panic → Internal and buffer return.
  I did not rerun Rust, Miri, fuzz or mutations or independently recertify that implementation.
  Guardian D042 (6729223027) holds the composed pipeline's production-safety qualification
  pending reached-path fuzz/mutation evidence; it explicitly reports no concrete UB and no
  line STOP. Preserve that scoped HOLD. It is not evidence that all bounded research is
  prohibited, and this experiment cannot discharge it. State inherited safety/stop
  conditions explicitly in the amended registration; matching hashes and green CI alone
  are not a substitute.

## Completion and reproducibility

Read at the reviewed head: the 2×3 and parent service-core registrations; `.frozen`;
`bench/service_2x3.py`, `service_core.py`, `team_size.py`, `bundle.sh`; `.just/bench.just`;
`examples/ab.rs` and `examples/occupancy.rs`; PR metadata, diffs, CI checks and failed-job
log. Read the pinned role/common instructions and Guardian conditions, inbox/channel
records and migration manifest. No new timing, decoder implementation, merge, queue edit,
tag change, acceptance-policy change or automation change.

To reproduce fixtures, place the three Python report/helper files from the exact reviewed
head together, then run `PYTHONPATH=/path/to/those/files python probe.py`. The script
creates and removes temporary evidence directories, invokes no decoder and runs no git
subprocess because that single identity lookup is mocked. The observations, including
working rejection controls, are archived in `probe.json`.

`coord.py audit` at the instruction pin found only five already-resolved historical
tripwires; `route --to coordinator --type REVIEW` returned #145. Routing output is retained.
Admission remains HOLD until a newly identified frozen revision closes H1 and narrows
H2–H3 without rewriting prior verdicts. The review does not require a new performance gate.
