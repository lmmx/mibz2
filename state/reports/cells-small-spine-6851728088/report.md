# Guarded Cells writer: implementation and negative performance verdict

PR379; candidate writer835984e849bd4200c0e74b76fb67a336cd82ae2c. Current head93781bd9578eae95c599583e387180d673659180 adds only the demonstrated service-fixture and cached-source preparation repairs. Writer SHA256 remains7543fdd68d3c99ed88799da7726340c0cd8a34d8f6ffe465ae0c1231ead2e23d. Assignment6851728088, after completed seven-family screen6850979995.

**Decision: the guard alone does not qualify for default adoption or a speedup release.** The first standing paired CI reports a Wikidata loss, a large pattern loss and no primary gain. This is retained regardless of later CI triggered by genuine validation repairs. No secondary rerun-to-pass. This is not evidence that the guard caused a regression relative to the old Cells writer: no matched old-writer arm was run.

## Actual whole-reader measurements

AMD EPYC9V45, 4 logical CPUs/2 cores visible to the benchmark, Ubuntu26 kernel7.0.0-1019-azure, Rust1.100.0, fat LTO/one codegen unit. Run43420282511, head835984e, default base3e7e294. Experimental input generator/text57f51; all seven compressed hashes checked. Candidate-build Segmented BzReader versus candidate CellsReader, one thread, ten paired rounds, hooks off. Wall and CPU are complete-reader/drain times; all140 rows succeed with matching output fingerprints. A separate21-decode byte-for-byte libbzip2 check passed before timing. Default-arm1/2/4-thread schedule and gates were unchanged. Paired ratios below are recomputed from raw rows with existing bench/report.py::ratio_ci.

| Corpus | Median wall ms S / C | Wall C/S [95% CI] | CPU C/S [95% CI] | Native peak KiB S / C |
|---|---:|---:|---:|---:|
| wikidata | 333.55 / 360.85 | 1.0987 [1.0411, 1.1595] | 1.0983 [1.0408, 1.1591] | 26702 / 35076 |
| onebyte | 15.55 / 16.95 | 1.0444 [0.9670, 1.1279] | 1.0322 [0.9535, 1.1174] | 34234 / 36802 |
| pattern | 20.40 / 34.55 | 1.7819 [1.5986, 1.9862] | 1.7208 [1.5526, 1.9074] | 16572 / 18522 |
| random | 89.70 / 89.25 | 1.0007 [0.9836, 1.0180] | 1.0010 [0.9843, 1.0180] | 23362 / 23500 |
| records | 93.65 / 93.95 | 0.9926 [0.9786, 1.0068] | 0.9930 [0.9790, 1.0072] | 23538 / 23736 |
| runs | 10.25 / 10.20 | 0.9357 [0.7996, 1.0950] | 0.9402 [0.8179, 1.0808] | 23628 / 23778 |
| text | 50.80 / 49.25 | 0.9784 [0.9348, 1.0239] | 0.9783 [0.9367, 1.0216] | 19678 / 19824 |

Native RSS here is ab child getrusage(RUSAGE_SELF), not the invalid uniform41168KiB Python-parent measurement. It varies by corpus and matches the expected direction of the earlier native screen. The separate screen validated exact-child wait4/native VmHWM with1MiB/64MiB controls; those instrumented RSS numbers are not substituted for this production footprint. This benchmark did not add per-row VmHWM/wait4 crosschecks, so the screen apparatus control is not falsely presented as an independent per-row audit of ab. Observed RSS increases31.0% on Wikidata and11.7% on pattern; no acceptance exception is claimed.

Random, records, runs and text are now measured in this candidate build; their wall/CPU intervals include parity. This does not fill an unmodified old-Cells i9 table. The prior i9 result0.881643 Wikidata/1.530338 pattern/0.535508 onebyte remains distinct; hardware, compiler and source context differ. Instrumented Wikidata phase attribution remains STOP. Logical metadata work saved is not a demonstrated time saving against the old writer.

## Implementation and correctness

One scalar all-small predicate is folded into the existing marked-origin walk. Only eligible blocks skip position initialization, the two grammar passes and long-only final spine walk. Long-factor operations, phase, short copies, slack, repetition, admission and fallback are unchanged. Separate reviewable proof is bench/cells-small-spine/SAFETY.md. New code does not alter default/API/team paths. The inherited #365 source/history is preserved in mergebe25366, and candidate-ignore-indentation.patch isolates the delta.

- Eight focused native Cells tests and five differential tests pass.
- Six Cells Miri witnesses pass, including16/17 boundary, poisoned output and long→small→long position-buffer reuse.
- Seed42 forced-route fuzz:27380 cases/30s, with production admission/refusal and forced reach asserted.
- Both new copy omissions compile then fail byte equality; all six original invariant mutants are killed after compiled baseline passes.
- Pinned full-reader/libbzip2 parity: all seven inputs/21 untimed decodes pass.
- Full Clippy all targets/features passes. Native suite with quota-consistent four-CPU affinity:174 tests pass in13 suites, one pre-existing ignored test; two expensive room-walk tests skipped in this second invocation had already passed in the first run.27 harness tests pass.
- Optional all-features test link cannot find libpython3.12 locally; this remains an explicit limitation, separate from successful Clippy and hosted packaging.

## Preserved failures and corrections

1. Initial boundary test assumed a two-run column had maximum factor16; actual factor27 disproved the fixture assumption. Its failed assertion is retained. A bounded native search found exact n17/n18 witnesses; no byte mismatch occurred.
2. Local default affinity exposes5 cores but available_parallelism returns4 quota CPUs; unchanged the_policy_here assertion fails. No production scheduling change was made. The test passes with four allowed CPUs.
3. Service fixture >=4-block assumptions fail after combining current master with #365: source-copy size1,500,435 already exceeds the1.5MiB-scale repeat boundary, so corpora(1) yields only two source copies and RLE1 leaves three blocks. Two fixtures now request scale2 and retain their reach assertions; all10 service tests pass. Failed logs are retained.
4. Changed-head CI43421337109 fails with FileExistsError at target/cells-pinned-generator restored by rust-cache. Default rows completed; experimental rows were not run. Head93781bd extracts a fresh exact pinned tree outside cached target/. Source/input/measurement identities and gates are unchanged. Partial artifact and log retained.

## Review, integration and next action

Researcher6852258044 accepts the conditional omitted-operation source proof, with B5 evidence closure pending. Coordinator routed targeted recheck6852337017 to that same reviewer. Raw native/Miri/fuzz/mutation evidence was supplied at9039563; current writer bytes are unchanged. This is not implementer self-certification or a claimed MONODROMY disposition.

Exact-head final CI is pending at this report. The original #365 DEFER and frozen slice-one gate remain separate dependencies. First-run default assessment reports no resolved change; that does not merge the opt-in API or qualify this experimental route. No merge, default adoption, release, i9 execution or new queue admission has occurred.

Coordinator6852388781 rejected guard-alone default adoption and dispatched the single saved-evidence Optimizer next-choice delta6852341904. The next technical decision is a target-qualified early route or another existing construction that addresses BOTH pattern cost and the observed host/build difference. An unconditional n/r cutoff does not solve the AMD Wikidata loss. Do not invent an Intel/AMD classifier or silently launch another candidate. Existing MONODROMY/Pathfinder ownership and independent review remain. The original useful i9 gain is a reason to pursue that bounded next choice, not to waive the failed qualification.

Full first-run artifact: ci-835984e.zip, SHA25694cde4d012bf878857c8edd696401e562bb025213804ba7cb51868b2664fae65; extracted raw rows/manifests and recomputed ratios are retained. Failed successor partial artifact: ci-4f6e6ff-partial.zip. GitHub comments do not automatically launch Codex; this session consumes updates while active.
