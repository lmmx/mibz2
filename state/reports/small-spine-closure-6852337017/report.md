# Narrow B5 evidence closure: PR379

Assignment6852337017; parent6852069758 and REVIEW6852258044.
Policy pin a2da49638ac54d7763de1c167bd054f67ffdc763.
Observed 2026-11-13.

**Disposition: close the parent's missing local Miri/mutation evidence objection for writer835984e849bd4200c0e74b76fb67a336cd82ae2c.** The independently accepted omission proof, prior native/differential/forced-fuzz evidence and now inspected raw Miri/mutation records satisfy this targeted B5 delta review. No concrete remaining B5 objection to that writer is identified. This is not a general proof of induction, an all-CI PASS, or permission to adopt/merge. CI Miri remains incomplete and the new benchmark preparation failed.

The same writer conclusion carries to fixture-only successor4f6e6ff10cacb1a9ac9e1a9c70cdd496d1b75359. No later head is covered by this review. The F01 withdrawal6790022921 and the accepted conditional omission argument stand. Prior involvement remains disclosed: Researcher supplied F01 counterexample/review and adjacent-emitter research, but did not implement this candidate or these tests.

## Provenance and source correspondence

Evidence examined at immutable90395635d82c3e039122e97003c544fe61c98984, under state/reports/cells-small-spine-6851728088. This review reads source.json, both mutation drivers at the writer commit, both baselines, every individual mutant raw log, both result files, the completed Miri log and retained failure records. See evidence.json for source identities/statuses and inspected-files.json for the exact records.

Decoded the connector's base64 representation of src/cells.rs at835984e:48251 bytes, SHA2567543fdd68d3c99ed88799da7726340c0cd8a34d8f6ffe465ae0c1231ead2e23d. It equals source.json and both independently generated mutation-suite source_sha256 fields. Each driver copies source into an isolated tree, hashes that source before mutation, requires unique replacement anchors, builds the mutant and then executes its named test. The newer driver additionally requires exit101 and equality-failure text. The actual raw failures below corroborate the intended targets.

The Miri log is a producer execution record associated with this source manifest; it is not a new execution by me. It names the correct six witnesses and ends with a complete successful result. It does not embed a per-invocation source hash or full toolchain/flag transcript. That provenance limit is explicit; the manifest, exact matching mutation baseline and matching candidate witness definitions provide consistent binding. It is sufficient for this narrow records review, not a claim of independently rerunning Miri.

The GitHub835984e..4f6e6ff comparison contains only tests/service.rs: two corpora(1) calls become corpora(2), with explanatory comments. The >=4 block-start assertions remain; src/cells.rs, decoder, mutation drivers and frozen experimental input files do not change. Therefore carry-forward concerns unchanged writer code, not silently replacing its reviewed head.

Correction to the prior report's local snapshot hash: spine-review/head.rs held one extra final newline added when materializing text. Its80e0b5f4... digest was the review snapshot, not the exact Git blob. Verified byte-for-byte that snapshot equals the48251-byte blob plus one newline. This does not change the earlier whitespace-normalized source checks. The authoritative writer digest is7543fdd6... above.

## Actual completed witnesses

cells-small-miri.log reports all six:
admission_and_invalid_records; exact_fit_and_reuse; long_writer; long_writer_production_admission; slots_mark_and_links; small_spine_boundary_and_reuse.
Final result:6 passed,0 failed,0 ignored,93 filtered;47.94s. Warnings are unused service-test items and Miri's ignored optimization level, not UB diagnostics. This establishes completed local Miri coverage of both guard routes, empty/retained storage and original long/F01 witnesses. It does not turn the cancelled CI run into a PASS.

Both mutation baselines compile and pass8 Cells tests. Every mutant has successful compilation followed by an actual runtime failure; no timeout/build error is credited.

| Campaign / mutant | Raw result and target | Classification |
|---|---|---|
| New missing-small-copy | src/cells.rs:869 output equality, column [1,0^16], row1; complemented stale255/254 bytes remain where expected0/1 | Byte-correctness kill, before reach assertions |
| New missing-long-recopy | :1039 and :1089 output equality; repeated a/b segments retain complemented158/157 bytes in both long and production-admitted tests | F01 byte-correctness kill, counters retained |
| Original uncounted-cell | allocator reports free(): invalid next size; child SIGABRT, cargo101 | Actual failing compiled mutant; not a byte-equality or Miri kill |
| Original lost-mark | :1153 mark-count equality0 vs1 | Invariant kill |
| Original wrong-backlink | :868 unwrap receives Budget in exhaustive oracle helper | Unexpected-refusal kill; not output equality |
| Original wrong-rotation | :869 byte equality on column[1,0], row0, output[1,0] vs[0,1] | Byte-correctness kill |
| Original region-overrun | :545 cells region exhausted | Region-capacity assertion kill |
| Original missing-spine-recopy | same :1039/:1089 F01 output mismatches | Byte-correctness kill |

These are two and six compiled mutant executions, not eight distinct fault mechanisms: the F01 omission is repeated across suites. The original suite's killed booleans mean nonzero test exit, not universally byte-equality failure. Its heap-corruption outcome is deliberate mutant behavior, not evidence of UB in the unmutated candidate. The baselines and completed Miri witnesses matter to that distinction.

The archived seed42 log now independently read reports27380 cases, production admitted2700/forced14451, consistent with the previously attributed claim. Parent review already independently inspected CI seed1's26021 cases,8 Cells unit tests and5 differential tests on the tree identical to835984e. No new fuzzing was performed here.

## Failed and partial records stay failed and partial

- cells-small-native-initial.log has7 passing tests before the new witness exists; it cannot establish the new coverage.
- cells-small-native.log fails the new fixture's largest_plain assertion27 vs16. The bounded search log then identifies exact16/17 fixtures; native-fixed passes8. The initial failure remains recorded.
- cells-small-tests.log fails all-features linking on missing-lpython3.12. It is not a successful all-features run.
- cells-small-default-tests.log fails parallel::plan_tests::the_policy_here with “5 cores of 4 CPUs”.
- fourcpu-tests has105 passing library tests,1 ignored and2 filtered, then the two service starts.len()>=4 failures. Do not call it an unfiltered full-suite PASS.
- fourcpu-final-tests is a partial snapshot ending during mtf_checkpoints, despite its “final” filename. It is not whole-suite completion.
- service-fixed reports all10 service tests passing; ordinary harness log reports27 passing tests.
- The previously reported read-only Miri cache failure has no separate raw log in this evidence directory. Retain it as reported history; do not claim the archive independently proves that particular failure. This does not invalidate the later complete six-witness Miri record.

## Fresh CI snapshot and remaining limits

At the checked4f6e6ff source:
- Rust131871813421: success. Step readback confirms Tests, Harness tests, Experimental cells harness tests, guarded-service producer/judge, scan tests and benchmark-alternatives lock all ran successfully. This resolves the prior snapshot's skipped-harness status on the successor, not retroactively on835984e.
- Miri131871926397: still in progress. No full CI Miri PASS.
- Benchmark131871812772: failure. Actual log at12:03:53UTC shows FileExistsError in pin_inputs.py:21 work.mkdir(), because target/cells-pinned-generator exists. Failure occurs during experimental input preparation, before its reader parity/test and Cells timing step. Existing-directory failure is observed; cached origin is the implementer's diagnosis, not proved by the traceback alone.
- Other available check statuses include successful fuzz-smoke, packaging, Occupancy, walk-stats and documentation. No whole-PR green verdict.

Old Miri131868261279 is cancelled, not PASS; old Rust131868260462 remains the recorded service-fixture failure. A new preparation fix was announced in REVIEW6852379143; it is outside the two reviewed source identities and is not pre-approved here.

The first experimental result remains negative for global/default enablement as recorded by the Coordinator: no performance acceptance follows from B5 closure. There is no matched old-Cells arm establishing a causal guard slowdown. This correction does not rerun, reinterpret or certify timing/RSS evidence. MONODROMY disposition, PR365 DEFER/slice-one and all standing integration/adoption gates remain separate.

Expected in this correction: obtain the missing source-bound safety records. Actual: exact hash correspondence, completed local Miri and substantive mutant failures inspected; that evidence gap closes. No measured performance gap is credited. Next highest value is completing/classifying existing final-head CI and preserving the negative result, not repeating the whole source review or launching another mutation programme.

## Publication record

Migration frozen legacy head and manifest digest verified before receipt627b8b76ca64fa12c0170e077fb3e0d974896480. Pinned router audited fresh connector-fed channel/inboxes: five resolved historical findings and the known historical researcher LOG6757065549 (unresolved only in this old pinned router), no new misroute. REVIEW routes to145. Exact-body/author/issue verification and the known coordinator-only FROM verifier limitation will be recorded with the finished cursor. No automation setting was changed.
