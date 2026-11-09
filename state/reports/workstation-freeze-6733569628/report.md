# #270 §8 freeze closure — REVISE
Inbox: [6733569628](https://github.com/lmmx/mibz2/pull/162#issuecomment-6733569628), dispatching queued 6733175242.
Reviewed source: **28298a4ad5b155a0cc0804582c9c40fc85400688**.
Governing specs: docs/team/SPEC.md and SPEC.RESEARCHER.md, with MODE/RULES/PROTOCOL, at a4f97714cc14366984015869d9d094d89488ac57.

## Decision
**REVISE the occupancy admission before freezing §8.** The timing registration closes the earlier scientific requirements; the separate capture admission still certifies incomplete or internally inconsistent evidence as interpretable. This is a bounded admission repair, not STOP for the experiment, a changed timing criterion, or a new B5 HOLD.

## What closes
Seven arms, default33 fixed reference, dir primary, ls/hs descriptive; 20 paired rotated rounds at requested 10 threads and self-recorded (3,0,6), exactly 140 timed records; the registered Wikidata digest; geometric paired log intervals with df=19. The LEAD/STOP/INCONCLUSIVE and attribution predicates are retained. In particular, the borrowed t-table is indexed at df=19 for n=20, not df=14.

The recipe refuses unfrozen judged runs, dirty tracked input, measured-input divergence, disallowed overrides and wrong corpus; copies/hashes its binaries; retains an attempt directory and failures; and admits captures separately from timing. Frozen input identity includes examples/, so a master merge after freeze would invalidate that identity. bench/run-fed-leader.frozen still says frozen none.

I ran the two pinned Python suites: **40 tests pass**. The synthetic positive timing fixture is VALID and attributed LEAD with df=19. These are admission tests, not workstation measurements or a fresh performance verdict.

## Counterexamples: capture admission fails open
Starting from the repository's own positive capture fixture, changing only the stated field(s), each following case exits 0, prints “VALID (dir capture): no perturbation shown”, and writes interpretable=true:

| Mutation | Why the conclusion is unsupported |
| --- | --- |
| Remove results[0].perturbation | No perturbation comparison exists. |
| Set full verdict to unknown | Unrecognized evidence becomes positive permission to interpret. |
| Set full verdict to the instrument's actual “too few rounds”, with null interval | Insufficient evidence becomes “no perturbation shown”. |
| Replace all 18 runs with copies of round 0/off | Length remains 18; missing timeline/full rounds are not checked against raw runs. |
| Replace all raw capture wall values with NaN | Invalid clocks coexist with accepted summary and interpretation. |

These are synthetic counterexamples to validation, **not claims that an actual submitted bundle is corrupt**. The native “too few rounds” string is emitted by examples/occupancy.rs when its interval is nonfinite; it is not an invented status.

Mechanism: occupancy_problems trusts complete_rounds and scalar summary counts, checks only raw list length plus plan/error/output, and never verifies the (round,mode,threads) grid or clocks. admit_occupancy defaults absent perturbation to {}, then marks a capture interpretable unless a verdict string starts with “perturbs”. Thus “unavailable” is collapsed into “no perturbation”.

Minimal closure:
- Validate the raw six-round × three-mode grid at 10 threads, once per cell; finite positive clocks needed for perturbation; reconcile completeness with raw cells.
- Require both traced modes' perturbation evidence. Missing/unknown/too-few/nonfinite evidence must not yield interpretable=true or “no perturbation shown”. Refuse malformed captures, or explicitly retain unavailable interpretation separately; do not invalidate otherwise valid timing merely because a capture fails.
- Derive/check the existing perturbation classification from paired raw measurements and finite intervals rather than trusting a free-form verdict string. Preserve the instrument's existing statistical rule.
- Add the supplied witnesses as negative fixtures and a real-schema positive fixture with both numeric throughput intervals. The current positive fixture contains verdict strings but no intervals.
- Correct §8's “three modes ... each twice”: six rounds means six observations of each mode, eighteen total.

No new run, altered threshold, discarded round or retry-to-pass is requested.

## Self-recording: complete evidence, unequal possible cost
Recording the Rust thread's own name removes the inherited /proc comm aliasing mechanism. A helper records after its first task, before publishing done; it cannot count an idle helper as having worked. Source inspection supports that distinction. The quoted 350/350 result is the builder's result; I did not independently rerun the native experiment. Neither names nor those counts establish simultaneous execution.

The stronger cost claim does not follow. src/threads.rs has a single global Mutex<Vec<String>>. All nine threads may contend; String creation and Vec growth can allocate. The leader hook is before work, and the helper hook is after its first task **before done**, so a delayed lock acquisition can extend a team's completion. Equal call counts do not imply equal exposed latency across arms. ab enables recording before decode; these hooks are inside wall timing. Its take/sort/join are after wall timing but before CPU rusage, hence are charged to CPU.

Replace “uncontended”, “costs every arm the same” and “nothing a decode's milliseconds can see” with the actual contract: one enabled recording call per participating thread, timed and potentially contending; all arms use the same mechanism, with unmeasured differential overhead. This observation does not prove material bias or require another timing matrix. It prevents calling the instrument free or claiming its latency cancels. D059's outside-timing recommendation has not literally been met; the deliberate replacement should be acknowledged.

## Freeze ordering and boundary
After the narrow repair, **merge intended master changes before selecting the input freeze**, then inspect the merged measured-input diff and run the relevant checks on that exact source. A pointer-only freeze commit can follow; the runnable head must satisfy the recipe's input comparison. Do not freeze 28298a4a and subsequently absorb measured examples/ changes.

This is not PROCEED yet. The separately dispatched Guardian review of the new src/ hooks, exact-head CI and completed Miri remain their own obligations; D059's earlier extraction review is not approval of this later delta. No code implementation, merge, ready tag, queue write, workstation run, or authority change was performed.

## Reproduction and routing
The bench/ files beside this report are unmodified source snapshots from 28298a4a. From this report directory:
```
python -m unittest discover -s bench -p 'test_run_fed_leader*.py'
python probe.py
```
probe-results.json retains the baseline and all five rejected-by-spec/accepted-by-code witnesses. No native benchmark was run.

Pinned coord.py audit over freshly fetched #145/#160/#161/#162/#222: zero unresolved misroutes, five historical resolved entries. route(to=coordinator,type=REVIEW) selects #145. Publication will be read back exactly and checked with the role-aware routing predicate; the pinned verify function's known hard-coded FROM coordinator requirement is recorded separately, never represented as a full verifier pass.
