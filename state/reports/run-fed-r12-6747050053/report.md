# #305 R12: required collector completion is now admitted explicitly

Trigger: [6747050053](https://github.com/lmmx/mibz2/pull/162#issuecomment-6747050053), dispatching 6747037147.
Reviewed head: `14f5924e4af3c66f98254ba73bcedeac3a905eb3`.
Parent: `cb4bccf025a79b712e0d2316eea1338e6f8fc582`.
Operating pin: `a4f97714cc14366984015869d9d094d89488ac57`.

**Technical PROCEED toward freeze.** I accept R12 as closing my material objection in [6746559255](https://github.com/lmmx/mibz2/pull/145#issuecomment-6746559255). No further technical revision is requested for this delta. This does not complete the separately pending authority re-pin, version reconciliation, or final freeze provenance.

## Why the objection is closed

The recipe preserves each required collector's actual status immediately after its bounded invocation. stderr has separate files; it cannot contaminate the status record. The checker now reads that record and requires each of `ab env --facts`, `lscpu`, and `ab sample` exactly once at zero. A failure cannot be erased by a subsequent successful duplicate. Missing, malformed or unregistered evidence accumulates a refusal.

The shell status and parsed verdict both gate entry to the timed portion. The documented DRY escape remains explicitly non-evidentiary. This change does not introduce a gate on clock, temperature, optional post-timing diagnostics or occupancy; it preserves the full-data result.

## Independent verification

- Ran the exact-head admission suite: **34 tests passed**, Python 3.12, 12.640 seconds. This includes the existing quota, type, identity, passive acquisition and child termination regressions.
- Replayed the previous complete-payload fixture against the actual R12 `preflight()`, changing only the status record. **24 cases passed their expected assertions:** two positive controls (canonical and reversed order), and 22 refusals. The refusals cover 17/124/137 for every required collector, each missing collector, each duplicate, each failure followed by success, missing/empty file, unknown collector and malformed status. See `probe.py` and `probe-results.json`. Run with `python3 probe.py /path/to/exact-head-checkout`. This uses repository fixture payloads, with independently assembled status mutations.
- Inspected the complete five-file R12 commit delta, including the recipe's early refusal and archive assertions.
- Read CI submit job [129322117444](https://github.com/lmmx/mibz2/actions/runs/42663840266/job/129322117444). Its **11 recipe tests pass**, including complete stdout followed by nonzero collector termination, refusal before timed records, and absence of `runs.jsonl` and `identity.json`. This is CI evidence, not a fresh local recipe run; the local environment's previously established process-substitution limitation and absent `just` were not worked around.
- CI checked merge `e572cb1f691f4a733d5fc56549987b192aeb41ac`. Both that merge and reviewed head have exact tree **`ecf690d3f76222847a1f766390d89de54d3e7c9a`**. Thus its recipe evidence applies to the reviewed source tree.
- Latest check-run read: **24 success, 3 skipped, no cancellation or pending job**. Both macOS wheel jobs are successful; the reported runner cancellation has cleared. Full names, links and relevant log excerpts are in `ci-evidence.json`.

The author's mutation-test claim was read, not independently rerun. No native build, performance benchmark or workstation capture was performed. No production implementation was changed.

## Freeze boundary

R12 answers the collector-completion question. It cannot supply authenticated authority for a changed repeat/stopping policy under the current pinned C6. Complete the separately announced re-pin before relying on that wider policy. Reconcile #305's BENCH/CHANGELOG version with #302's eventual version before freezing; a new frozen head must carry its own exact provenance and required checks. This review is tied to the head above and is not blanket approval of later semantic changes.

The prior accepted R11 scientific answers stand within their stated scope. Optional diagnostic failures still do not alter the all-pairs timing estimand. No performance benefit, production adoption or permission to execute a timed attempt is inferred here.

## Process record

The state migration head and manifest digest were verified before the receipt. Inbox cursor receipt is committed at `11dc8f1b90c635abb0574117e1a54e3c574a2789`. The channel routing audit has five previously resolved historical misroutes and zero unresolved findings (`audit.json`). Research writes remain on `team-state/researcher`; the automation is unchanged.
