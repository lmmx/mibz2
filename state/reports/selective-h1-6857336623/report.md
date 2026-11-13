# H1 closure — selective Cells, assignment 6857336623

**PROCEED for the reviewed native unsafe-path scope. H1 is resolved by independently checked evidence.** This accepts a correction alongside the prior HOLD; it does not rewrite that review or its initial evidence. The scoped native finding also carries explicitly from f2dc198ee22ad13b69a72d42744fb1a0d135b95f to a8a23c11607864cb431f87fff38169e10c41685f, for the reasons below. It does not certify the whole successor, performance, integration or CI.

## Two independent answers to the mismatch

**Exact-source completed suite.** I retrieved results.json, all four full logs, the campaign summary and SHA256.json at evidence commit [51cf6b72857fc511e95b38094c6caea16369d8a4](https://github.com/lmmx/mibz2/tree/51cf6b72857fc511e95b38094c6caea16369d8a4/state/reports/cells-selective-6856176834/cells-selective-mutations-100). All six campaign files match that manifest byte-for-byte under SHA-256. The unmutated source digest is independently the reviewed src/cells.rs digest:

`3bbc63745068713b75bd6cbaac4217d374fbca6d1318a55c27bf9e788858aeee`.

| Run | Compilation/execution observed | Result |
|---|---|---|
| baseline | Executable unittests src/lib.rs, then Running unittests | One boundary/reuse test passed; exit0 |
| missing-short | Same completed compile/run stages | Selected materialization equality fails at src/cells_selective_test.rs:29; exit101 |
| missing-rule | Same completed compile/run stages | Selected materialization equality fails at src/cells_selective_test.rs:29; exit101 |
| masked-mark | Same completed compile/run stages | selected representation assertion fails at src/cells.rs:276, false versus true; exit101 |

The first two logs expose differing retained payloads, rather than a build failure being counted as a kill. These are still payload/classification assertion kills, **not three output-byte kills**. The runner independently fetched at both native commits is identical; it hashes the source before mutation, checks each replacement occurs once, compiles with --no-run before execution, and requires the expected assertion failures. No fresh test execution was necessary or performed in this review.

Producer identifies the suite as Rust1.100.0. The logs establish compilation and test outcomes; they do not independently print rustc --version. I do not promote that compiler-version attribution to a locally reproduced binary identity. H1 concerns source correspondence, now independently resolved.

The evidence commit's recorded author/committer time is2026-11-13T15:18:35Z; the original REVIEW6857261340 was created15:21:44Z. That recorded chronology is consistent with the completed evidence predating the HOLD. The old review inspected5df5a2, not this later bundle; it remains a correct record of that evidence gap.

**Initial-source reconstruction.** I independently fetched [initial-mutation-source.rs](https://github.com/lmmx/mibz2/blob/6da00de4af4ea10565658a1e4dcd6858a939cd58/state/reports/cells-selective-6856176834/initial-mutation-source.rs), hashed its full text to the original `ca1b9116309fcb140d3aa27917cbfe68ff8581bae7eb1d4553a7c954bb6406aa`, and compared it with src/cells.rs fetched at f2dc198. Exactly one ordinary line comment differs, at line602:

- initial: `// the cell's word, A's then B's: as bytes, and as a rule; one is kept`
- reviewed: `// The cell word is A then B; materialise only its retained representation.`

No executable token, cfg, macro, include, assertion, line count or string literal changes. This accounts for the hash discrepancy without attributing it to the unrelated example-formatting diff. The original ca1b campaign is not renamed or overwritten. The supplied mechanical-correspondence JSON is supported by my own comparisons, not accepted merely on its boolean assertions.

## Explicit successor correspondence, confined to the native path

I fetched and SHA-256 checked each of these files independently at **both** f2dc198 and a8a23c:

- src/cells.rs;
- src/cells_original_test.rs;
- src/cells_selective_test.rs;
- bench/cells-selective/mutations.py;
- Cargo.toml;
- Cargo.lock.

All six pairs are byte-identical and match the producer's H1-correspondence.json identities. Full digests and the initial-source diff are in verification.json.

The complete GitHub compare lists only **bench/cells-selective/codegen.py** and **examples/ab_cells/mod.rs**. I read both patches. The former changes symbol matching, compiler metadata checks and parity-log parsing; the latter adds cfg(test) reader counters and richer route reporting. Neither is a source input executed by the runner's `cargo test --locked --lib miri_cells_selective_boundaries_reuse` target. The native Cells code, selected-state comparator, mutation runner and dependency inputs are unchanged. Therefore the prior native argument plus corrected mutation evidence applies to a8a23c explicitly, without a second full unsafe-path review.

The live PR380 read during this check reports exact head a8a23c11607864cb431f87fff38169e10c41685f, open and unmerged. This finding does not follow a future head automatically. I have not re-reviewed the seven-family harness, codegen outputs or performance result. Dispatch reports ordinary CI incomplete; a legacy pre-commit success is not full green. No integration, workstation, queue, benchmark or release clearance follows.

## Disposition and record

Expected: close only the source-identity gap. Observed: exact-source positive/negative test evidence plus a complete one-comment explanation of the earlier digest. The gap is closed; no remaining H1 requirement warrants another native run. The useful next action is for the Coordinator to record this narrow closure while completing the separate harness, CI and performance obligations already in progress.

Original [review84b5f2e](https://github.com/lmmx/mibz2/blob/84b5f2e17f48f6d63413bfeaed0b4e9dc5e2d193/state/reports/selective-b5-6857053711/report.md), REVIEW6857261340, initial evidence and historical failures remain unchanged. This is the original reviewer's acceptance under RULES B5/D, limited to H1 and the native correspondence stated above.

Pinned SPECs/MODE/RULES/PROTOCOL were freshly read; migration head and manifest digest match. Live cursor69cc94969031cff9580318caa67a6c0a2ded766c marks the original assignment finished (the dispatch's different cursor-SHA label is not used as evidence). Receipt 3eb1b6d648cfcabf39c9d678ba9108c52558163d preceded this work. Pinned routing audit since6857053711 found no defects in fresh comments; REVIEW routes to145. No task or authority setting changed.
