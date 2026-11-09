# #258 workstation admission re-review: PROCEED

Inbox **6729699102**, dispatching **6729516361**.
Reviewed head: **3e201246cf1266d4b2a984cb19eb9fdba5b9835d**.
Frozen inputs: **7aeb5bf9b63355b4bc56691fec5da30b7fb3ff10**.
Operating instructions: docs/team/{SPEC,SPEC.RESEARCHER,MODE,RULES,PROTOCOL}.md at **a4f97714cc14366984015869d9d094d89488ac57**.

**PROCEED for PROTOCOL §9 workstation-queue admission at this exact head and freeze.** My H1–H3 admission HOLD in [6729358962](https://github.com/lmmx/mibz2/pull/145#issuecomment-6729358962) is resolved for the specified findings. The controls now reject the missing or impossible evidence, and the final readings no longer mistake a bundle comparison or saturation flag for an identified cause.

This is admission of the registered experiment, not a production qualification or instruction to run it. Coordinator owns queue/tag actions; Louis runs the frozen recipe. Guardian D042's composed-pipeline B5 production-safety HOLD remains unchanged.

## H1: usable evidence and registered workload

I read the revised [report](https://github.com/lmmx/mibz2/blob/3e201246cf1266d4b2a984cb19eb9fdba5b9835d/bench/service_2x3.py) and ran its [30 repository fixtures](https://github.com/lmmx/mibz2/blob/3e201246cf1266d4b2a984cb19eb9fdba5b9835d/bench/test_service_2x3.py): all passed. The run emitted unclosed-file ResourceWarnings; these did not change test outcomes.

I also replayed the original review's fixture/mutation code against the new report. The original stand-in control itself is now INVALID: its corpus digest is fictitious and its trace lacks the registered configuration. Thus “all old cases now INVALID” alone would not demonstrate a working discriminator.

For the independent positive control, I changed only its corpus digest and full trace to the registered schema, retaining the original mutations. The corrected control is VALID; all **16 original mutations** are INVALID, including all original acceptance holes and rejection controls. I executed the original fixture/case definitions without their obsolete assertions expecting the bugs to remain. This is not a claim that the unchanged old script exits successfully.

| Obligation | Current check / observed result |
|---|---|
| Nonempty output evidence | Hex digest and exactly 209,700,196 decoded bytes; all usable rows must match the reference digest. Null/empty evidence rejected. |
| Numeric evidence | Wall, CPU and RSS must be finite and positive, excluding booleans. Negative/absent wall, NaN CPU, zero/infinite RSS and boolean wall rejected. |
| Workload binding | Every row names wikidata; corpus.sha256 matches bench/wikidata.json. Wrong name, missing digest and different digest rejected. |
| Trace identity | One result at 10 threads, registered svc2 plan, two service threads, matching commit, parity true and registered output size. Wrong configuration/plan/commit/parity rejected. |
| Trace content | All metrics read must be finite and in range; at least one complete round; saturation resolved. Sparse or NaN-metric trace rejected. |
| Existing controls | One round, duplicate round, wrong plan, unregistered arm, DRY and NOT-FROZEN still rejected. |

The recipe requests six trace rounds. Its inherited validity rule explicitly allows at least one complete trace round; the new report preserves that rule rather than quietly imposing six complete rounds.

**Non-blocking robustness defect, not a false VALID:** replacing round with [] produces TypeError at the set used to count rounds. The earlier “any malformed record gives INVALID, never an exception” wording is too broad. The recipe catches a failed report and records REPORT FAILED; no result is judged. Harden this separately by excluding malformed round values before building that set. It does not reopen H1's acceptance holes or justify withholding this bounded run with the harness's fixed integer-round schema. Do not advertise universal parser robustness from these fixtures.

The evidence-provenance boundary is unchanged. VALID checks a directory's predicates, not detached authenticity: commit.txt versus checkout and absence of NOT-FROZEN do not independently prove a frozen tree. Admission still uses the exact recipe/head/freeze and corpus digest; a future submitted bundle needs that provenance checked.

## H2: comparisons and preference are now distinguished

The illustrative wall values remain synthetic:

| default | plain2 | pipe2 | svc2 |
|---:|---:|---:|---:|
| 1.00 | 1.10 | 0.90 | 0.95 |

On the revised report, P1/C1/P2 are GAIN, but S1 is about 0.9474, and the overall text says **REGISTERED STEP 6 MET, BUT SVC2@10 NOT SHOWN BETTER THAN THE SIMPLER PIPE2X3@10**. It no longer says the service threads caused the product gain. C1 is explicitly the service bundle against plain2 at the same teams/window; P2 and S1 are visible separately.

The common-round geometric-mean identities replay correctly:

- P1 = 1.0526315789;
- S1 × P2 = 0.9473684211 × 1.1111111111 = P1;
- C1 × S2 = 1.1578947368 × 0.9090909091 = P1.

They hold for CPU too in the fixture. These decompose comparisons, not causes; confidence-interval endpoints are not multiplied.

The report preserves the registered step-6 outcome while narrowing the narrative recommendation using the separately reported S1. S1 remains secondary; this is not a changed judged family, multiplicity policy or production acceptance rule. NULL is bounded uncertainty, and CPU INCONCLUSIVE means no detected cost, not proof of zero cost.

## H3: saturation remains a flag, capacity a hypothesis

The revised counterexample text retains the registered numeric saturation predicate but explicitly admits service-thread imbalance and queue/scheduling explanations. Prep starvation without saturation is consistent with lookahead binding, not proof of it. The corresponding fixtures pass. This closes the causal overstatement without changing thresholds or requiring another experiment.

## Freeze, source scope, CI and dry-run provenance

- GitHub PR #258 still names the full reviewed head. The compare from frozen 7aeb5bf9… to head contains only bench/service-2x3.frozen, pointing to the correct freeze.
- Compared with the previously reviewed 3e4477d0…, only five paths changed: the report, its new tests, the registration's explicit correction section, the freeze file and the test workflow. No decoder source, recipe, service_core.py or team_size.py changed.
- The existing recipe still compares the measured tree against the freeze, checks tracked cleanliness and untracked measured inputs, and includes src, Cargo/build/toolchain/configuration, examples, tests/common, report/helpers, corpus tools/manifest, bundle helper, recipe and registration.
- **21 latest checks on the reviewed head:** 19 success, Release and Publish crate skipped. The submit job, which now runs the report fixtures, is [128917071996](https://github.com/lmmx/mibz2/actions/runs/42541452645/job/128917071996). Check identities are archived in evidence.json. This is a correctness/admission readback, not a new performance verdict.
- The PR body records a dry run at this exact head: expected INVALID for DRY/four-core/one-round conditions; all six outputs equal at digest a8830caee5c511a6 and the registered decoded size; new record/trace checks passed. It also reports the freeze-refusal control. Its scratch bundle was deleted. I read that record, not an independently replayable raw bundle, and did not run the decoder locally. No timing credit attaches to it.
- [D042, 6729223027](https://github.com/lmmx/mibz2/pull/145#issuecomment-6729223027), holds production B5 evidence, explicitly finding no concrete UB or line stop. This narrow admission verdict cannot discharge it.

## C3 accounting and reproducibility

Expected the revision to reject the known bad evidence and narrow the claims. It does so with a valid positive control. **No measured decode gap closed**: this review removes an admission obstacle. The matched two-team comparisons remain valuable because the earlier product result did not isolate the service bundle from plan changes. The registered bounded run now has higher information value than another wording cycle.

The accompanying replay.py reuses original_probe.py, plus the exact reviewed report/helper/test files. It invokes no decoder and mocks only git HEAD for fixture identity. replay.json records original and repaired controls, the additional numeric/parser cases and illustrative readings. Download the five reviewed bench Python/manifest inputs named in evidence.json beside the replay files to reproduce it.

No implementation, benchmark, CI dispatch, queue/tag edit, merge or automation change. Migration manifest/head verified before receipt; routing audit found only the five already-resolved historical cases. This PROCEED is limited to the named registration/head, with its existing stop conditions and C5/C6 boundaries intact.

