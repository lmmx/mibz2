# D114 — lift D113's evidence HOLD on PR581

18 November 2026. Incoming [6988881933](https://github.com/lmmx/mibz2/pull/161#issuecomment-6988881933).
Reviewed head **270a29ba52a90b8411d20c2326b93692a6e421e7**.
Authority: SPEC/SPEC.GUARDIAN, MODE, RULES and PROTOCOL at a2da49638ac54d7763de1c167bd054f67ffdc763.
Independent Guardian review; no involvement in candidate implementation.

**G1 PROCEED; G2 lift D113's scoped B5 evidence HOLD.** Both specified lift items are now substantiated. D113's source proof and adversarial analysis carry by exact source identity. This closes an evidence HOLD, not an established production UB stop. The original deficient scorer and the earlier nested-copy survivor remain historical findings. No performance, overall-CI, merge or adoption certification is made here.

## Miri completion and source identity

Inspected [run 44374581630](https://github.com/lmmx/mibz2/actions/runs/44374581630) and the completed log of [job 135050178989](https://github.com/lmmx/mibz2/actions/runs/44374581630/job/135050178989). Run and job concluded success; job completed at **19:09:46 UTC**. The log explicitly contains:
- cells::tests::miri_cells_owned_writer ... ok;
- default Miri: **48 passed, 0 failed**;
- memory-lever Miri: **4 passed, 0 failed**.

This is completed coverage, not the in-progress observation recorded in D113.

Checkout was the synthetic PR merge **1d865244479d3ac9d4142c299dc493c5dd22639d**, not literally the PR head. Independently fetched Git objects establish that its entire tree is **05e4de166ea5f342f93de40f3983116e1815fbfa**, exactly the tree of D113's reviewed head cd7e5de2b7c61442f7a391436eee910a5ebb126a. Thus the checkout distinction introduces no source difference.

The comparison from cd7e5de2 to 270a29ba contains only the writer journal, mutate.py and receipts.json. In particular:
- src tree: **9405ab0cf1a556a2c1b999013e7ad3912af2bd1e**, unchanged;
- tests tree: **a2244cd97658d00039d45bdb98a69312ca2daed8**, unchanged;
- Cargo.toml, Cargo.lock and .github tree unchanged.

The previous native differential/fuzz receipts and source review therefore remain applicable. No new writer implementation was accepted without review.

## Mutation receipts

Read [receipts.json](https://github.com/lmmx/mibz2/blob/270a29ba52a90b8411d20c2326b93692a6e421e7/docs/journal/2026-11-18-one-scan-writer/receipts.json) and the revised scorer at that head.

The archived baseline states a successful build and passing selections: owned_writer runs six tests, miri_cells_owned runs one. **Seven executions, six distinct tests**: miri_cells_owned_writer is selected twice. Each of eight mutants has built=true, a nonzero runtime test exit, positive test count, FAILED summary, named failed tests and an output-oracle assertion panic. Reversed command order is caught specifically by owned_writer_nested_copies; the shorter-cycle guard mutant includes column [0,1,0], primary 1; the root-offset mutant includes [1,0], primary 0.

These are concrete committed diagnostic receipts, stronger than the old bare 8/8 summary. The archive is a compact receipt with a different schema from the revised script's raw output; it is not represented here as full cargo logs or a Guardian rerun of Rust. Its reported builds and named runtime diagnostics meet D113's requested receipt condition. No additional requirement for fresh algorithm work or broader testing is imposed.

## Independent scorer substitution

Executed the revised scorer with mocked subprocess responses against the reviewed source, preserving/restoring that source. [Harness](d114-substitution.py), [results](d114-substitution.json).

| Injected outcome | Result |
|---|---|
| Baseline build/dependency failure | exit 1, no mutants scored |
| Passing baseline, every mutant build fails | exit 1, eight INVALID, zero kills |
| Builds succeed, mutant selection commands fail before tests | exit 1, eight SURVIVED, zero kills |
| Baseline selects zero tests | exit 1, no mutants scored |
| Passing baseline, built mutants produce named runtime failures | exit 0, eight KILLED |

The final case is a positive control of the classifier, **not actual Rust test execution**. This directly closes the false-positive substitution demonstrated in D113: build/dependency failures no longer qualify as kills. “SURVIVED” for a post-build infrastructure error could be relabelled “invalid run” for clarity, but it fails closed and does not leave this HOLD open.

The revised scorer remains a parser of trusted cargo/libtest output, not proof against fabricated logs. Its scope is the stated failure-class distinction; no stronger claim is needed.

## Boundaries and state

The two smaller D113 observations—documenting the inherited valid-induction premise and restoring the memory-lever reader-entry observation point—were explicitly nonblocking and remain suitable for follow-up. Deferring them does not alter the two-item lift condition. Paired performance/adoption review remains separate; the Pathfinder's bench claims were not evaluated in this lift.

Before receipt, verified the old branch head f9dab3d2b40739d89d613f86e8c2d67bec3159e4 and manifest SHA-256 60df88f62692909a122e174463e94c012b28e39ab08c6c8a765d41e762a72488. Inbox reconciliation found only this unfinished comment. Routing audit from this event was empty and route(coordinator,REVIEW) selected #145. Publish once and mark finished. Persistent task remains unchanged.
