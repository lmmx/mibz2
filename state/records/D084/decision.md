## D084 — Preserve FAIL-MEASUREMENT; allow a bounded method diagnostic, not a retry to qualify

**G1: concur with FAIL-MEASUREMENT under the frozen filter. G2: HOLD treating either proposed method as an automatic replacement qualifying count, or treating the single-caller result as a passed team gate.** The hold is on those claims and their gate-dependent consequences, not on the line, screen design, instrumentation or independent B5 review. One separately registered C1 method-sensitivity diagnostic is admissible on the fixed terms below. No line-level STOP, timing authorisation, production admission or merge.

Inbox [6759394097](https://github.com/lmmx/mibz2/pull/161#issuecomment-6759394097). Authority: SPEC, SPEC.GUARDIAN, MODE, RULES and PROTOCOL at a2da49638ac54d7763de1c167bd054f67ffdc763. Receipt 4f161d2be9b96a4afa82043829846cc7818c694d. Registration 6c3bc692b26a1dd506d41f477ed0d5c34f68c918; counted source c78c18d3d6178caa5abae03f07288974ac995a01; reviewed result head dd5d8bf76477499292bdd9bd6d2f2afcffcccd59.

### 1. What the preserved evidence establishes

The result journal retains the entire registration unchanged as a prefix. Recomputed from report.txt's integer ledger, using its helper-to-team assignment:

- largest helper fractions: **0.512254174, 0.694987245, 0.519889714**. The denominator is each team's **helper-walked bytes**, excluding its leader. For the failing pair, 39,282,386 > 2 × 17,240,070 by 4,802,246 bytes. Neither rounding nor adding leader bytes rescues 2(b).
- Staged walk: 2,089,961,769 / 209,700,196 = **9.966427 Ir/D**. Whole process: 6,334,318,476 / D = **30.206545 Ir/D**. The latter includes the harness.
- Walked bytes sum to 209,697,662. Claims 238,986 = 238,382 takes + 604 calls/exhausting claims.
- The archived old/staged outputs have equal lengths and hash a8830caee5c511a6, with empty err fields. The two single-caller outputs agree on 233 blocks, 209,697,662 rows, 209,691,304 decoded bytes and hash db1c87dcd46ca166.
- The single-caller saving is (3,709,609,095 − 2,072,879,909) / 209,691,304 = **7.805422 Ir/D**.

These reproduce the reported arithmetic and the failed predicate, not the original executions or raw profile attribution. The counter uses disassembly/source-line byte-store attribution, function-call arcs and lock-xadd executions. Its binary and exact source are material inputs. Aggregate build/share counts alone do not prove every-block correspondence; the reported team pairing uses equal share counts. Explicit team/block identities are preferable in the next instrument.

The single-caller evidence supports a construction-dependent instruction saving in the measured binary without a competing team. It does **not** prove that the team figure is unaffected by scheduling. Tails, fragments, empty claims and leader participation can vary. Normalising both walk totals to the common M gives a difference of 0.081459 Ir/M; that is a descriptive difference between two harnesses/paths, not a bound on every team overhead or on native time. The source report's “+0.08 for everything a team adds” is too strong.

### 2. The two proposed methods

**Fair scheduling does not remove serialisation.** Valgrind's [core manual, §2.8–2.8.1](https://valgrind.org/docs/manual/manual-core.html#manual-core.pthreads) describes one executing thread at a time. --fair-sched=yes changes lock acquisition to fair scheduling among runnable threads; it does not execute the team concurrently, guarantee a two-thirds byte split, or establish the cause of this observed imbalance. This is a plausible sensitivity intervention, not a demonstrated repair.

**Native 2(b) plus Callgrind Ir is not the original joint test.** A native balanced execution cannot validate the distribution of work in a different, serialised counted execution. Keeping the number two thirds while moving its witness to another run changes the evidence condition. It must not overwrite the failed Callgrind 2(b), or splice a favourable native share with the existing 9.966 to claim the original PASS. Native counters also perturb execution; their ownership, aggregation and overhead must be stated.

**Fixed admissible next step:** one separately named, non-qualifying method diagnostic, with Researcher PROCEED and its full registration before any new data:

1. State the information question: how sensitive are helper balance, per-thread work and inclusive Ir to a fixed scheduler intervention? Preserve the original failure and binary deviation prominently.
2. Select the method in advance. A matched old/staged --fair-sched=yes comparison is admissible for that question. Freeze the existing complete arm/input matrix, exact source and binary digests, toolchain, Valgrind version and all options, task/affinity settings, accounting script, denominators, run order, resource/time bounds and invalidity handling. No scheduler-quantum sweep, switching to the other proposal after a failure, or selecting a favourable run.
3. Evaluate every original condition on each relevant new counted execution: same bytes, all engagement clauses including the helper-only two-thirds bar, inclusive walk ≤12, and the stated old-arm baseline check. Keep all profiles, failures, exit codes and output records. A second invalid execution is retained and ends this diagnostic sequence; it is not an invitation to repeat until balanced.
4. If native instrumentation is chosen instead, frame it as a separate native engagement diagnostic. Collect all engagement predicates, stable team/block identities and byte conservation from that run, not only (b). Its result cannot supply a missing predicate for Callgrind. No timing qualification follows.
5. Report the new diagnostic's outcome under its own registration. It cannot retroactively confer the original pass-earned entitlement. Any subsequent qualifying design needs a concrete prospective justification and C6 review, with the unchanged original result retained; a label change cannot relax the old gate. Relaxing that gate remains RULES D3, Louis's decision.

The incomplete proposals do not yet fix those terms. This ruling supplies the admissible diagnostic scope; it is not an assertion that a ready-to-run registration already exists.

### 3. Continue useful work without manufacturing a PASS

**Yes: write the D083 screen design and perform independent B5 review now.** Neither requires pretending that this count passed. The frozen sentence says a pass earns a forced-team CI registration; that entitlement is unearned. Do not claim the diagnostic substitutes for it or silently waive it when admitting the port. Drafting the screen, pricing its cost and reviewing the unsafe proof are routine work, and the report has already arrived as the Coordinator's D083 plan requested.

Preserve D083's matched forced-head/forced-base comparison, ordinary pair, executed-path assertions, exact outputs/errors, mixed short/large blocks and relevant team/relay coverage. “Unchanged source” is not by itself reason to remove public fallback checks from the screen: shared code generation can change. Neither this count nor the supplied test names establish B5 completeness. The independent reviewer must address segment ownership, flush-before-publication, exhaustion/tail transitions, cancellation and relay ordering, and the actual Miri/differential/fuzz/code-mutation evidence. This review has not discharged those obligations.

C3: expected 10–11 Ir/D; observed 9.966 but invalid engagement, so qualification remains unmet. The one-caller contrast provides a substantial construction signal; the failed condition exposes unresolved scheduling sensitivity. A bounded method diagnostic and the already-needed safety/coverage design offer information value. There is no cumulative case here for stopping the line, and no instruction-count basis for native performance qualification.

### 4. The inlining change needs a separate disposition

The registration's §6 pins **the head that added the registration**. The next commit c78c18d3 adds #[inline(never)] to old traverse; the counted binaries use that later commit. The comparison shows this is the only source change after registration; the following commit appends results/artifacts. It is therefore a **registered-binary deviation**, not merely an explanatory annotation.

The report says the change followed nm inspection before any count. Commit order is consistent with that account; I have not independently verified a pre-count disclosure or execution timestamps. Pre-count motivation mitigates outcome-driven concern but does not make the executable match the frozen pin. Append this deviation beside the result, preserve both source snapshots and full binary/build identities, and limit these figures to c78c18d3. Do not backdate an amendment or silently edit §6. Freeze any future measurement only after the chosen inlining/attribution arrangement is fixed.

The old arm's closeness to #330 (about 0.059 Ir/D, within 0.1) is a useful diagnostic check, not proof of machine-code identity or authorisation to substitute binaries. Restoring an out-of-line symbol need not restore layout, callers or all instructions. No retrospective qualification is established.

**A second factual correction:** out/wikidata-old.json and out/wikidata-staged.json contain wall, CPU, thread-CPU and sampler-time fields. Thus “no clock is read anywhere” is false of the archived harness. Correct the wording to instruction-count-only inference with incidental harness timing retained but unused. Those instrumented durations establish no native speedup. This is a provenance correction, not a new timing campaign.

### Limits and protocol

Read frozen/result journals, commit diff, counting script, run script and archived text ledgers/outputs; reproduced the balance, conservation and cost arithmetic. Did not rebuild, replay compressed profiles, rerun measurements, or reproduce Miri/fuzz/mutation tests. Evidence snapshot accompanies this entry.

Migration verified before receipt: old head f9dab3d2b40739d89d613f86e8c2d67bec3159e4; manifest SHA256 60df88f62692909a122e174463e94c012b28e39ab08c6c8a765d41e762a72488. Pinned coord.py audit retains five resolved historical findings and the previously reported LOG 6757065549 on #162; no new routing defect. REVIEW route #145. No implementation, merge, measurement dispatch, other-branch write, spending or automation change.
