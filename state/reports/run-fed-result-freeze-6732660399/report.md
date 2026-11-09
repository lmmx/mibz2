# #270 CI result and workstation freeze: Researcher review

In reply to inbox comment **6732660399** on #162. Instructions: `docs/team/SPEC.md` and `SPEC.RESEARCHER.md` at `a4f97714cc14366984015869d9d094d89488ac57`.

**The CI judgement stands: VALID (JUDGED), attributed LEAD under §6. REVISE §8 before freezing it as decision evidence.** Keep `dir` primary and `ls` secondary in that continuation. The strongest scientific correction is to separate the registered attribution from a causal claim about direct construction, and to correct the claim that the one-team results agree with the three-team ranking.

## 1. Evidence and independent reconstruction

I read the result journal at `d1260c59047846401ed0e717ba6c66d33b699f70`, the registration and reporter at frozen head `03e52206ac248737bc856f064928adf238265917`, and the front stage, sort, emitter and A/B entry points at that head. This reviews the experiment as run; it does not transfer a clearance to a later implementation.

Downloaded Actions run [42564217298](https://github.com/lmmx/mibz2/actions/runs/42564217298), artifact **13650328398**, `sweep-03e52206ac248737bc856f064928adf238265917`. The downloaded ZIP's SHA-256 matches GitHub's recorded digest:

`97f370cbb4ecbb8b2ed6262e1358d4d608ad497b3c1a1c4e082a26a696d00f4d`.

The frozen reporter, run on the archived `rfl-runs.jsonl` with the archived identity and the exact registered head, exits 0 and reproduces VALID, LEAD and attribution. It checks order, complete cells, exits, outputs, phases, fallback absence and the identity. Separately, `recompute.py` rebuilds the paired log-ratio intervals without importing that reporter: 210 distinct arm/thread/round cells, 15 rounds, seven arms, threads 9 and 3; all exits 0, empty errors, one hash/length (`a8830caee5c511a6`, 209700196). Raw inputs and the independent result accompany this review. No new decoding or timing was run.

The identity records a clean source tree at 03e52206, release with `run-fed`, rustc 1.99.1, equal before/after binary digest `4bcbe3996070c991f9b9584d468af9b3f6c0f8a08c4a914ab0e4d8b63924f81b`, and equal corpus digest `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. This verifies consistency of the archived identity, not a fresh independent build of its binary. Output agreement on this corpus is not a universal correctness or safety proof; D056's B5 scope remains separate.

### The exact registered judgement

| Predicate | Independently recomputed evidence | Result |
|---|---|---|
| Primary `dir/default33`, 9 threads, wall point ≤ .97 and upper < 1 | .966859 [.959422, .974354] | passes |
| Primary CPU upper ≤ 1.05 | .963958 [.960188, .967743] | passes |
| `ctl/default33` wall interval wholly within [.98,1.02] | 1.000606 [.993136, 1.008131] | passes |
| `dir/ctl` wall upper < 1 | .966274 [.960096, .972492] | passes |

Only `dir` has a judged registration outcome. `ls` and `hs` are reported secondaries even where the reporter prints a threshold label; `reg` is a control/separate lead, not an additional primary. The 3-thread cells use `pipe3-both`, not `default33`, which still forces three teams. The copied worker's control passes the specified ±2% fidelity criterion; this does not prove zero copy/instrumentation cost. In particular `ctl/both` is 1.0071 [1.0011,1.0131], compatible with a small cost. Nor does control fidelity subtract the record arms' own instrumentation and synchronisation.

## 2. What the additional contrasts do and do not establish

The supplied paired `dir/reg` and `ls/reg` intervals reproduce. Pair each numerator and denominator within the same round, take logs, and use the df=14 t interval; do not divide marginal interval endpoints. These are fair **additional descriptive contrasts of the actual arms**, disclosed as post-run analyses. They do not change the reference, add an attribution gate retrospectively, or promote `ls` to primary. No simultaneous/multiplicity-adjusted coverage is claimed for this collection.

| Additional wall contrast | Three teams, requested 9 threads | One team, requested 3 threads |
|---|---|---|
| `dir/reg` | .992134 [.986550, .997749] | .999362 [.992310, 1.006464] |
| `ls/reg` | .974647 [.969726, .979592] | 1.015168 [1.001756, 1.028760] |
| `ls/dir` | .982374 [.977432, .987341] | 1.015817 [1.001113, 1.030736] |

**Correct the journal's “3-thread cells agree” sentence.** They agree that these arms beat `both` at their reported point estimates. They do not agree about the incremental record benefit or the arm ranking: `dir` is indistinguishable from `reg` here; `ls` is slower than both. This is not a controlled experiment isolating contention—the team count changes too—but it directly defeats an environment-independent inference from the 9-thread ranking. The smaller CPU ratio of `ls` does not rescue its slower wall at one team.

“Most of the primary's gain is C29” is a plausible lead, not an identified causal decomposition. At 9 threads the geometric means factor exactly as `dir/ctl = (dir/reg)*(reg/ctl)`, because they use the same rounds. That arithmetic does not imply additive, portable mechanism effects. Source makes the limit concrete: `CTLR` uses `mtf_checkpoints_reg` on all blocks; record arms use `mtf_runs` only where the rule takes records and otherwise use the old `mtf_checkpoints`. They also change representation, checkpoints, construction, partitioning, small-block paths and scheduling. Thus “the full `dir` arm is about .79% faster than `reg` here” is supported; “direct fill alone adds .8%” overstates identification. The registered label *attributed* is correct under §6's operational rule, but does not prove that eliminating the F-order array caused the entire 3.3% gain. C29 remains a strong separately testable lead.

## 3. Workstation: REVISE the draft into a complete prospective contract

An attributed CI LEAD satisfies the prerequisite to prepare a freeze. It neither freezes the present text automatically nor authorises an agent run. The literal “unchanged but for its frozen commit” must be read with §8's explicit unfinished admission and traces: those mechanics have to be completed before the final pin. Recommended contract:

1. **Keep the seven arms and their order, `dir` primary, `ls` and `hs` secondary, and `default33` the fixed judged reference.** Retain `reg` to expose the C29 alternative. Do not silently select `ls` after seeing it win on CI. A future `ls` primary is a new prospective registration, informed by this discovery. Keeping it secondary in §8 loses no comparison data.
2. **Freeze requested threads = 10, actual topology = three leaders plus six pipelined helpers.** Ten is the request used by plan selection, not ten decode workers. `ab`'s run-fed arm takes `floor(10/3)` teams of three; `default33` forces that plan. Check actual topology per arm/run, and record the i9 topology, affinity/cpuset/quota and sampler environment. Preserve park/small, slot and input-window settings and the same corpus digest. A different environment is not silently admitted as the registered i9 run.
3. **Make 20 rounds exact, not a mutable recipe default:** 140 timed records, seven per round at requested 10, with the frozen rotation/reversal, no extra cells, no omissions, all successful and equal outputs, phase schema/reach/ownership/fallback checks carried over. The current reporter hard-codes 15 rounds and threads [9,3]; it cannot admit this run by passing `--rounds 20` or by relabelling threads. Give the workstation admission its own explicit schema and tests, rejecting wrong head, order, plan, count, outputs, phases and missing/non-finite evidence. Do not weaken it into UNJUDGED to obtain a reading.
4. **Write the threshold table at 10 before running.** Recommend retaining §6's wall point ≤ .97, wall upper < 1 and CPU upper ≤ 1.05 for LEAD; wall lower > .99 for STOP; otherwise INCONCLUSIVE; and the same control-fidelity/primary-over-control attribution gates. Compute paired log t intervals with df=19 for 20 rounds. These are prospective diagnostic thresholds for this hardware/workload, not a new merge qualification. Do not import the 15-round critical value or adapt thresholds to the outcome. Reserved workload, repeat/multiplicity or estimand changes remain Louis's under C6; this recommendation does not grant authority to change them.
5. **Freeze the executable method and its identity checks.** Record exact implementation, recipe and admission commits; require the registered source state, build features/profile/compiler, clean relevant tree and stable binary/corpus SHA-256 before and after. Replace the draft's `NOT-FROZEN` marker only once the contract is actually frozen. A source repair means a new freeze; failures and partial records remain archived, without reruns to select a passing result.
6. **Keep the proposed ctl/dir occupancy captures separate from timed pairs.** Freeze one decode each, their exact capture method and order, required event fields and time origins, trace completeness checks, and their intended question. They diagnose the target's overlap/critical path, not a 21st round or a performance verdict. A missing trace should block the promised mechanism conclusion and trigger investigation, not a convenient rerun or silent omission. Adding an `ls` trace could be informative but would amend the drafted capture set and must be declared before freeze; it is not required to preserve the existing secondary timing arm.
7. **C5 provenance is part of admission.** Louis runs the exact frozen recipe himself and hands over/submits the bundle; `just submit` archives it immutably on `evidence` with digest and identity. Preserve raw records in order, failed/partial outputs, build/console/environment/identity, reporter results and traces. Verify the submitted bundle's digest and contents, not a comment's claimed head. No agent timing, production change, or further-run authority follows from the CI LEAD or this review.

The precise recipe may be frozen after these omissions are resolved and reviewed. As drafted it lacks enough executable admission to call the workstation outcome decision evidence. I recommend REVISE that gap, not revisiting a passing CI threshold or discarding `ls`.

## 4. Adjacent angles: test placement before inventing a new sort

**Prior art within this code comes first.** `ls` already moves the stable counting sort to the leader's front stage for block i+1, while helpers work on block i. `hs` executes the same count/prefix/scatter kernels across helpers with gates; it is not a comparison-sort baseline. “Move sort earlier” is partly implemented already. The 9-thread result points toward removing work/gates from the helpers' critical path, but does not establish that serial leader work is generally cheaper: `ls` has a longer leader phase and more leader-late runs, yet earlier build readiness here. Mean phase totals, particularly summed helper clocks, cannot be subtracted as critical-path contributions. Paired per-block dependencies and release/build/walk times on the target would test that explanation.

**A bounded next design question is emitter-plus-histogram fusion.** The emitter already accumulates byte row counts; it can in principle accumulate run counts at run openings as well, eliminating the later `count_share` scan for the one-share leader sort. Its final row counts and per-byte run counts then provide the row/slot prefixes, followed by the existing stable scatter. This targets one pass, not an unproven promise to eliminate the whole sort. Keep this as a hypothesis: extra counter updates/dependencies, register pressure, zeroing and cache traffic may cost more than the saved sequential pass, especially on short/run-dense blocks.

**Full final-placement fusion has a dependency barrier.** For byte c, the final F offset is `sum(counts[b] for b<c)` plus c's preceding rows; the compact record-array bucket start also needs all lower bytes' final run counts. Those are generally unknown while the first run is emitted. Per-byte staging, a second pass, or deferred fix-ups are needed; each has a memory/locality/allocation cost. Do not claim a one-pass direct F scatter from the current information. The final run closes at end-of-block, and initial runs, empty blocks, malformed streams and partial errors matter.

An eventual fused/unfused **ls** comparison should hold the register-list kernel, selection rule, partition/fill/walk, counters and timing boundary fixed. Test that whole-decoder result and the predicted removal of one scan, not just a smaller sort timer after moving its work into `front`. Preserve independent tiling/length/ownership validation; sharing emitter counters with the validator risks a correlated bug. It needs its own safety review and prospective experiment. #277's direct-fill work is a separate implementation question, not evidence already supplied by this run.

## 5. Research judgement and limits

The draft expected STOP from local smoke; the registered CI result instead clears the 3% point target with a 3.31% point reduction (95% interval about 2.56–4.06%) and passes attribution. That closes the CI screening question, not the target-machine or production gap. The mechanism lesson is that the full recipe pays on this oversubscribed configuration, with a promising C29 control and an environment-dependent leader-sort ranking. The next workstation comparison has high information value because it tests that dependence under the intended physical-core plan without changing the chosen primary. It is worth preparing; it is not worth calling the unfinished recipe frozen.

This is independent research, not implementation or merge authority. I did not rerun B5 or promote component tests to a new safety clearance. The state migration pin and manifest digest matched; the channel routing audit had no unresolved findings. Publication belongs on #145, never on #162. The pinned routing verifier has a known Coordinator-only sender check; any resulting Researcher-only mismatch is recorded separately from exact-body/destination verification.
