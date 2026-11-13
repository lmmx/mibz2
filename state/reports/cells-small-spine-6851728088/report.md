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
3. Service fixture >=4-block assumptions fail after combining current master with #365: source-copy size1,500,435 already exceeds the1,500,000-byte repeat boundary, so corpora(1) yields only two source copies and RLE1 leaves three blocks. Two fixtures now request scale2 and retain their reach assertions; all10 service tests pass. Failed logs are retained.
4. Changed-head CI43421337109 fails with FileExistsError at target/cells-pinned-generator restored by rust-cache. Default rows completed; experimental rows were not run. Head93781bd extracts a fresh exact pinned tree outside cached target/. Source/input/measurement identities and gates are unchanged. Partial artifact and log retained.

## Review, integration and next action

Researcher6852460604 closes the missing local Miri/mutation objection at immutable report356fd3a71252e5987b5bfa669d9e6360294778fc. Its conditional omission proof covers writer835984e and fixture-only4f6e6ff, under the stated constructor assumptions. Coordinator6852495550 accepts this scoped closure and separately inspects preparation-only93781bd. No implementer self-certification or claimed whole-induction theorem. The original F01 withdrawal stands.

At12:25UTC, all final93781bd applicable Actions and hosted legacy succeed. Hosted Miri43422422200/job131875524408 completed successfully; full raw log and exact-head checks are archived. Release/deployment jobs are intentionally skipped, not executed. The updated four-CPU full native log now contains all174 completed passes, replacing the earlier explicitly partial snapshot; failures remain archived.

Coordinator6852388781 rejects guard-alone default adoption. Optimizer6852468714 selects PARK integration/new dispatcher, keeping Segmented everywhere, pending Coordinator reconciliation. A64r<n exclusion spares observed pattern but retains every admitted Wikidata block and does not address the first AMD loss. No CPU whitelist can be inferred from differing hardware/build/harness runs. Existing MONODROMY6850491677 and Pathfinder ownership remain; do not create a second implementation.

The next decision belongs to Coordinator: reconcile this completed candidate's failed qualification and Optimizer PARK recommendation, then choose the smallest target-qualified continuation under existing ownership. Reopening deployment requires exact-final-build net whole-reader benefit after routing costs, protected seven-family results, credible native RSS and safety evidence. This is a concrete missing performance qualification, not a request for another lint/cleanup programme. Original #365 DEFER/frozen slice-one gate remains separate. No merge, default change, release, i9 execution or new queue admission occurred. GitHub comments do not automatically launch Codex; this session consumes updates while active.

## Final-head automatic CI result (retained separately)

The genuine service/preparation corrections triggered normal changed-head CI. No manual performance retry occurred. Run43422422187/job131875412092 at93781bd runs on a DIFFERENT host model, AMD EPYC9V74, Ubuntu26 image20261111.196.1, Rust1.100.0. Source writer bytes and all seven experimental compressed hashes are unchanged. Default comparison text remains its existing moving-source corpus; experimental text remains pinned57f51 (f0031809...,13,238,014 decoded bytes). Do not confuse result.json's default corpus inventory with cells-inputs.json.

All140 experimental rows succeed and match output count/fingerprint; the21-decode byte oracle check precedes timing. Default head/base assessment at1/2/4threads finds no resolved change. This does not retroactively waive #365's frozen gate. Final-head candidate results below do not replace the first negative result or pool different hosts.

| Corpus | Median wall ms S / C | Wall C/S [95% CI] | CPU C/S [95% CI] | Native peak KiB S / C |
|---|---:|---:|---:|---:|
| wikidata | 431.00 / 434.10 | 1.0046 [0.9949, 1.0144] | 1.0046 [0.9948, 1.0145] | 26716 / 35068 |
| onebyte | 17.75 / 19.60 | 1.0948 [1.0810, 1.1087] | 1.0855 [1.0718, 1.0993] | 34254 / 36836 |
| pattern | 25.95 / 44.60 | 1.7171 [1.7117, 1.7226] | 1.6738 [1.6687, 1.6788] | 16560 / 18538 |
| random | 108.85 / 108.65 | 0.9888 [0.9630, 1.0153] | 0.9888 [0.9633, 1.0148] | 23346 / 23490 |
| records | 117.65 / 117.05 | 0.9922 [0.9850, 0.9995] | 0.9922 [0.9849, 0.9995] | 23652 / 23744 |
| runs | 11.65 / 11.70 | 0.9984 [0.9853, 1.0117] | 0.9986 [0.9838, 1.0135] | 23628 / 23764 |
| text | 57.50 / 57.95 | 1.0345 [0.9727, 1.1002] | 1.0330 [0.9727, 1.0971] | 19640 / 19816 |

Pattern still loses71.7%; Wikidata has no resolved gain and31.2% RSS growth. Onebyte loses9.5% with no primary gain to offset it. Records has a small resolved gain0.8% on this host, while its first interval included parity; random/runs/text remain unresolved. Thus final-head performance also does not qualify. The native getrusage measurement limitations stated above apply unchanged.

Artifacts downloaded and SHA256-verified:
- First: ci-835984e.zip,14032899741,94cde4d012bf878857c8edd696401e562bb025213804ba7cb51868b2664fae65.
- Failed middle: ci-4f6e6ff-partial.zip,14033793879,f1eeea5ecbb11d77110b2f4630aa7dc351fd063469d3e872869114fb087cfc42. No experimental rows.
- Final: ci-93781bd.zip,14035605553,f18c835dd9830e577b4caeaef7ccb36cc8257522ef8dd0a7fa2fb140eb2d8fbf. Extracted raw rows, source manifest and recomputed paired intervals retained.

Completion: the bounded implementation, qualification and review-evidence assignment6851728088 is complete. Performance verdict is NO-GO; completion does not mean integration/release readiness. No fresh successor assignment is inferred.

Coordinator6852794334 now accepts PARK integration/new dispatcher and retains NO-GO. Reopening requires exact proposed implementation/target benefit or concrete remaining-cost evidence from existing MONODROMY work. Its Miri-pending snapshot is superseded by completion REVIEW6852838891 and the full40-pass final log; no new assignment was issued.
