# General plan policy: independent review

Assignments: [6715369675](https://github.com/lmmx/mibz2/pull/162#issuecomment-6715369675), resumed by [6729118037](https://github.com/lmmx/mibz2/pull/162#issuecomment-6729118037).
Instruction and master pin: `a4f97714cc14366984015869d9d094d89488ac57`.
Researcher verdict: **REVISE**. Retain the evidence envelope and opt-in research separation. Do not promote rule A as a general performance rule. Its arithmetic is reproducible; its fallback is now empirically falsified, its causal interpretations are too strong, and its report validator accepts incomplete evidence.

## What has moved

The assignment described diagnostics waiting in the workstation queue. All three ran on 8 November, frozen at `9f13b4e8bc7fc41e7a105d8ec25ec79c48cb653b`, execution head `a5dcaebda28fecaa8165e81604bb2824d3842c71`. #254 remains an unmerged research branch, now `fc9a62051418496beedde97bf805a401791333e5`. Its three result journals record boundaries, bytes, and levels. The current queue is empty. The general design on master still contains prospective passages; those are historical, not a reason to queue these runs again. A later small-block change on #258 is queued for separate review, not assessed here.

This review ran pure arithmetic, the frozen report's 21 synthetic tests, and three malformed-evidence probes. It did not build or time a decoder. I read the submitted notices and result journals; **I did not download/hash the three tar archives or recompute their raw paired observations**. Numerical statements about the actual runs below are attributed to those pinned journals, not an independent certification of their VALID status. The validator at #254's current head is byte-identical to the frozen source.

## Ranked objections

### 1. Report validity is not closed under missing evidence

In frozen `bench/plan_diagnostics.py`, `validity` collects `(r.get("hash"), r.get("out"))` and accepts one distinct pair. Deleting both fields from every synthetic run leaves the singleton `(None, None)` and **VALID**, including substantive outcome readings. `err` need not be present either. This repeats the missing-evidence failure previously raised in the policy A/B review; agreement among absent fields proves nothing about output.

Deleting every `wall` also leaves **VALID**. `compare` returns no comparison, but the boundary reading still says NOT FALSIFIED AT 8 / NEITHER RESOLVED. Incomplete observation is not an unresolved valid experiment. Require finite positive wall values and complete registered pairs as part of validity, not just as a filter downstream. Require nonempty well-formed output hashes and the registered output size, and an explicit success record.

For levels, changing every L20 base arm from `mibz2@/w/bin/ab-base` to `mibz2@/different/base`, while leaving L10 unchanged, is also **VALID**. The one-baseline check is per cell, not per attempt. Different paths can hold identical binaries, so this does not itself prove different code, but the report cannot establish the promised common baseline without an attempt-wide identity/hash binding.

`probe.py` and `probe.json` reproduce all three failures. Existing tests: 21/21 pass. The tests' ambient git identity was replaced with a deterministic synthetic SHA for these fixtures only; the validator source was not edited. These are validator counterexamples, **not evidence that the real archives lack the fields**. Before relying on the old VALID labels, audit the existing bundles for these predicates and baseline identity. No timing rerun is implied unless that audit fails. Fixing validators is for the implementation owner; none is changed here.

### 2. The measured fallback failure supersedes acceptance of rule A

The boundaries journal reports U8 `2×4p / 4×2 = 1.079 [1.055,1.102]`, and U9 `3×3p / 4×2 = 1.141 [1.119,1.163]`. Thus the registered rule-A fallback claim is falsified at eight threads while the shared nine-thread choice is supported. This is already acknowledged by the Coordinator. It is a reason to revise the candidate rule, not to loosen the existing acceptance rule after seeing the result.

Rule B remains unresolved under its registered three-cell eight-thread map: its upper bounds are 1.009, 1.002 and 1.036, so not all exclude 1.03, and none establishes the required gain in every cell. That does not validate its fitted curve. The curve transfers one machine's rate/footprint relationship and assigns a constant 1.072 pipeline factor; it is useful for exposing internal choices, not a physical performance law.

The journal's statement that width 2 is dominated “wherever a pipelined plan fits” must be restricted to the measured cells. In particular, two pipelined teams of four were measured unmasked at eight, not across all masks, budgets, levels and machines. CPU and RSS summaries here are descriptive, not simultaneous confidence bounds for Pareto dominance.

### 3. The diagnostics distinguish outcomes better than causes

**Boundaries:** the registered matched-arm wall comparisons can decide their stated local speed hypotheses, conditional on valid evidence. They do not change L3. M9's `1.146 [1.127,1.166]` shows a win without an extra available core; it refutes that core's necessity for this win. It does not show that reserving a core has zero incremental value. Comparing U9 and M9 point estimates is not a randomized estimate of the spare core's effect.

**Bytes:** the published plain-arm ratios at levels 6/7/9 are `0.916 [0.890,0.943]`, `0.957 [0.942,0.972]`, and `1.087 [1.067,1.108]`. These satisfy the frozen descriptive PREMISE SUPPORTED map. But recompression changes block count, per-block fixed work, compressed input, and stage proportions along with table size. Therefore “width 3's advantage depends on bytes against L3” is a compatible explanation, not identified mediation. The original compact-Ψ same-byte control is stronger footprint evidence, at a different kernel boundary. It cannot supply the missing identification here.

Moreover the pipelined arm wins at all three levels (1.163, 1.204, 1.264): a cap inferred for plain width 3 does not thereby govern the pipelined candidate. Accept the result journal's own proposal to test the revised candidate as pipelined. No i9 cell establishes cross-machine L3 scaling or the 6 MB constant.

**Levels:** the same-head forced-4×2 arm is a strong practical intervention on the plan. It is not a decoder-count-only intervention: it also changes pipeline mode and the team-scaled window (12 to 16 slots with four requested slots per thread). Observing three versus four busy leaders and no helpers at levels 1–2 supports the count explanation, but does not isolate it. A matched mode/window control would be needed for that causal claim; it is not needed to state that forcing the tested plan recovers performance.

The outcome map also confuses failure to detect harm with absence of harm: K FASTER plus R not SLOWER becomes THE DECODER COUNT. The actual R intervals include 1 and allow small losses (as low as 0.986). Write instead: **“The forced-plan intervention recovers the observed loss; remaining same-plan release differences are unresolved within the reported intervals.”** A claim of equivalence needs a prospective margin and an interval inside it. Preserve the original registered map as history; correct the interpretation separately.

### 4. Topology guards do not prove the advertised homogeneous domain

Keeping the old exception before the new guards preserves compatibility, including the known hybrid/multiple-socket false matches. That is defensible as preserving shipped behavior; calling the whole matched domain “measured” is not. Matching ten cores and cpu0's 20 MiB cache does not identify the i9 workstation. Keep legacy compatibility and new admission evidence conceptually separate. The guarded nine-thread extension is a necessary narrowing and still requires its own qualification.

At current #254, `Topology::fast_eligible` tests `hybrid() != Some(true)`: **unknown core kind is eligible**. Its partial `cpu_capacity`/frequency reads can likewise classify only the readable subset as uniform. A synthetic twelve-core, 20 MiB, shared-cache topology with all kind signals absent admits `3×3p`. Thus the wording “cores are of one kind” is stronger than the implemented evidence. Either require adequately complete evidence for new admissions or disclose this as an unverified heuristic. A single shared-cache check limits obvious multi-socket cases, but cannot establish honest VM topology. No new fast-path authority follows from the lack of a heterogeneity signal.

### 5. Fill-the-budget is a research preference, not an efficiency theorem

I independently enumerated admissible rule-A plans and matched **528,320 cells**, P=1–64, L3=2–128 MiB in 1 MiB steps, N=1–2P. Re-running the supplied script reproduces every displayed table, including **57 L3 cliffs**, ratios **0.849133–0.955006** under rule B's score, and the partial variant's **2,070** width-3-involved modelled falls as N rises. The independent enumeration uses the specified rule; the cliff ratios deliberately use the published scoring function and are not measurements.

For example, at ten cores, 22→23 MiB changes `3×3p` to `5×2`, score ratio 0.88834. Since `Tcap=floor(L/6 MB)`, the core condition gives the exact necessary bound `L < 6 MB × (floor((P−1)/3)+1)`; `Tcap<Tb` supplies another condition. The cited approximate reach and constant interval `(20 MiB/4,20 MiB/3] = (5.24288,6.99051] MB` follow. The i9 selects an interval of constants, not 6 MB uniquely.

Replacing the full-budget condition with the cores-bound partial plan removes these cache-axis cliffs but introduces the reported thread-axis score falls. Neither variant dominates on the supplied model. Prioritizing predictable behavior as a user changes N is a reasonable product preference; the numerical scan does not prove it maximizes throughput. Actual L3 allocation can also change under masks/VM configuration, so cache-axis discontinuities are not intrinsically harmless merely because most users own one machine.

The current condition is **P−1**, not `min(N,P)−1`: N=9,P=10 permits nine decoder workers. It leaves arithmetic capacity for other work; it does not pin the scanner to a dedicated physical core or prevent SMT sharing. Preserve that distinction in the mechanism. Both full and partial rules inherit a small candidate set that excludes the now-winning U8 width-four pipeline, so choosing between them cannot repair that missing candidate by itself.

## Is the low-level regression sound?

**Yes, as a local measured regression**, conditional on the bundle audit above. The result journal's four level-1/2 S ratios are .763, .765, .776, .772 with all upper bounds below one. They mean **22.4–23.7% lower throughput**, or **28.9–31.1% longer elapsed time**. “23% slower” should name the throughput convention; it is not 23% more time. The earlier .76–.83 forecast was explicitly an older-build inference and is now superseded for these tested cells.

Do not extend two thread counts on one recompressed corpus to every N≥10, all inputs, all ten-core/cache lookalikes, or all levels 3–9. Level 3's observed gain supports that cell; “elsewhere exactly as 0.2.4” does not follow from equal plan selection, since kernels and other release behavior also changed. The plan intervention is enough to motivate the separately registered scheduling remedy, not to claim its proposed implementation is already proven or that all loss is exclusively decoder count.

## Seven earlier knee objections: read-back

1. **Confirm:** E4 now uses the per-run inversion-call denominator and 28.75/32.14/32.64% outside shares; total serial work remains unidentified.
2. **Confirm:** E3 withdraws memory-saturation identification and lists competing limits; E1 says planned teams.
3. **Confirm:** H1 uses conditional alpha/overhead sensitivity and permits null/harm; no 1.10 floor survives.
4. **Confirm:** the outcome map separates overlap validity from mechanisms and requires phase evidence.
5. **Confirm:** E5 labels compressed input and inverse-BWT output correctly and separates capacity from residency.
6. **Confirm:** requested budgets are separated from the actual nine-worker plan; masks are not per-thread placements.
7. **Confirm:** the ranking is conditional and includes adverse compact-table comparisons, rather than treating smaller tables as an established win.

These are textual closure of the seven earlier objections, not a fresh replay of every archived trace. H3 has since been superseded by the left-field archive check: no renewed libsais implementation or timing is recommended. Its surviving RUN-path instruction-count screen is a bounded research discriminator, not proof that instruction thresholds alone predict wall performance. Prior art and negative results remain material; an inversion-only historical win does not establish a current whole-decoder opportunity.

## Disposition and reproduction

Priorities: (1) audit existing evidence for the validator gaps and repair future validation; (2) retain the already-recorded rule-A fallback rejection; (3) narrow causal, equivalence and user-facing scope claims; (4) clarify unknown topology eligibility before any new admission. Keep #254 research-only. No implementation, merge, new run, envelope widening or authority change is performed by this review.

Files beside this report: frozen `diagnostics.py`, `tests.py`; master `plans.py`; research-only `probe.py`, its `probe.json`, and the full arithmetic `tables.md`. Run `python probe.py` and `python plans.py`. The original sources are pinned below; the copied code has no functional edits. Routing audit and output route are in `routing.json`. It finds only five resolved historical misroutes. The pinned verification helper still hard-codes FROM: coordinator, so it cannot pass for a correctly headed Researcher REVIEW; record its result and independently verify exact body, header and #145 destination without falsifying authorship.

Primary sources:
- [General design and linked tables](https://github.com/lmmx/mibz2/blob/a4f97714cc14366984015869d9d094d89488ac57/docs/journal/2026-11-07-general-plan-policy.md).
- [Revised knee](https://github.com/lmmx/mibz2/blob/a4f97714cc14366984015869d9d094d89488ac57/docs/journal/2026-11-07-knee-mechanism.md), [archive check](https://github.com/lmmx/mibz2/blob/a4f97714cc14366984015869d9d094d89488ac57/docs/journal/2026-11-07-left-field-archive-check.md).
- [Frozen validator](https://github.com/lmmx/mibz2/blob/9f13b4e8bc7fc41e7a105d8ec25ec79c48cb653b/bench/plan_diagnostics.py) and adjacent tests; the three `2026-11-07-plan-*-registration.md` files at that commit.
- [Boundaries result](https://github.com/lmmx/mibz2/blob/fc9a62051418496beedde97bf805a401791333e5/docs/journal/2026-11-08-plan-boundaries-result.md), [bytes result](https://github.com/lmmx/mibz2/blob/fc9a62051418496beedde97bf805a401791333e5/docs/journal/2026-11-08-plan-bytes-result.md), [levels result](https://github.com/lmmx/mibz2/blob/fc9a62051418496beedde97bf805a401791333e5/docs/journal/2026-11-08-plan-levels-result.md).
- [Current topology and candidate implementation](https://github.com/lmmx/mibz2/blob/fc9a62051418496beedde97bf805a401791333e5/src/parallel/general.rs).

Submission notices: boundaries 6719871923 (evidence `ad4f73214d3e87fecb3918e232ec7198dbbb27e9`); bytes 6719942262 (`a078ca88c05b08ac1d0b79751638d3db19e83a5f`); levels 6720074438 (`71e8dc17a733609f79222a1c0ea8103882992e3d`). Their bundle digests are recorded in the linked result journals; those digests were not independently verified here.
