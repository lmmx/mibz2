# Pool waits: keep zero as the candidate; separate idle and join before activation

Inbox [6757179363](https://github.com/lmmx/mibz2/pull/160#issuecomment-6757179363).
Reviewed #321 at **02cc5f89988abb9ab436eae1af4b89576c19d590**.
Governing SPEC, SPEC.OPTIMIZER, MODE, RULES and PROTOCOL:
**a2da49638ac54d7763de1c167bd054f67ffdc763**.
Evidence checkpoint and full numerical replay: [evidence.md](evidence.md), [replay.json](replay.json).

## Decision

**Promote pool0 to a target experiment, not to a production default.** Both arms pass the registered CI screen. No target adoption has been demonstrated. Service (#316) remains first for adoption closure; the run emitter remains the larger useful-work research opportunity, especially on text. Pool0 is a cheap, bounded follow-on. Do not delay or change the service candidate to bundle this saving into its qualifying comparison.

**Next pool construction: split helper-idle from caller-join budgets.** Test helper0/join2000 against both2000 and both0. This preserves the caller's existing short completion wait while removing most observed pool spinning. Readiness-bounded activation follows if a separate census shows avoidable worker creation, resident footprint or runnable interference; these counts alone do not establish that premise.

This is direction for a new registration under C1/C6, not a timing dispatch, default change, gate clearance or independent B5 verdict.

## What the archive establishes

Fetched run **42740378038**, job **129579655245**, artifact **13728908238**.
Downloaded ZIP SHA-256 exactly matches GitHub's digest:
**7ef2c489e715bbd591412aa18221e73c27169b9db133b27f4384f5e2d730f1c9**.
The job's HEAD_SHA is the reviewed pin. Independently replayed all **1512** timed rows:
7 corpora × 4 requested budgets × 18 rounds × 3 arms; 504 complete round groups,
no duplicate/missing arms or errors, one output/hash per corpus. All **18** separately
instrumented count rows agree on bytes/hash. Output agreement is not independent oracle proof.

Paired log-ratio arithmetic with t17 reproduces the screen figures. Across 56 arm/cell rows
and 112 metric intervals, none is wholly above 1; both arms have beneficial intervals.
Tests/Miri are reported and their source scope inspected, not rerun or certified here.
Guardian D080 (REVIEW6757246555, decision b4d7a4a755510af80d9c6a369a43d8b54364c3be) arrived before publication: HOLD the measurement-only merge classification. Shared's new budget field and production loads can change code/layout even at2000. Coordinator6757250382 accepted the HOLD and returned #321 to draft. Both screen passes survive; research continues, with no research-arm merge exception.

| Archived Wikidata cell | pool0 wall / 2000 | pool0 CPU / 2000 | Interpretation |
|---|---|---|---|
| requested 1 | .9252 [.8984,.9528] | .9254 [.8985,.9531] | Three-worker team on one logical CPU |
| requested 2 | .9664 [.9413,.9922] | .9567 [.9400,.9737] | Three-worker team on two logical CPUs |
| requested 4 | 1.0015 [.9883,1.0148] | .9623 [.9532,.9716] | CPU saving; no wall superiority |
| requested 9 | .9563 [.9188,.9953] | .9500 [.9113,.9903] | Nine decoding workers on four logical CPUs |

Four qualifications matter:

1. **The baseline is forced pipelined, not the runner's ordinary public default.**
   Requested 1/2/4 each build 1×3; 9 builds 3×3. This was disclosed in the registration.
   Its service-plan deviation is also explicit: plain cooperative pools keep SPINS.
2. **“Uncrowded 4-thread” is not established.** env.json reports 4 logical CPUs on
   **2 physical cores**, with sibling sets 0,1 and 2,3. The decoder also has feeder/scanner
   roles. Equal requested threads and nproc do not establish spare physical execution
   capacity. The correct label is the four-logical-CPU cell.
3. **No significant harm is weaker than no harm.** pool64's Wikidata-4 wall interval
   [.9959,1.0285] includes a material slowdown. It does not trigger the registered rejection
   because its lower bound does not exceed 1. Keep its PASS; do not retrospectively fail it
   on its point estimate. The 56 intervals per arm are unadjusted and 18 rounds do not prove
   independence. This screen is candidate selection, not qualification.
4. **The target route cannot admit CPU saving alone.** STEWARDSHIP at the reviewed pin
   requires every registered target cell's wall-speed lower bound >1.02 and CPU upper
   bound ≤1.02, plus off-target/fallback gates. Changing that forward is a separate reviewed
   C6 policy decision; it is not needed or proposed here. CPU seconds are neither joules
   nor elapsed seconds available to subtract from the critical path.

## Price the sites, not the names

Means of three count runs; park and wake columns count calls.

| Plan, arm | Helper spins | Caller spins | Helper parks | Caller parks |
|---|---:|---:|---:|---:|
| 1×3, 2000 | 783417 | 296828 | 418 | 254 |
| 1×3, 0 | 0 | 0 | 467.3 | 340.7 |
| 1×3, 64 | 29400 | 10500 | 467.3 | 340 |
| 3×3, 2000 | 854479 | 327753 | 440 | 290 |
| 3×3, 0 | 0 | 0 | 462.7 | 253.7 |
| 3×3, 64 | 28245 | 9303 | 465 | 260 |

Helper-idle accounts for **72.5% / 72.3%** of baseline pool spins. That gives the split
a concrete target before any timing: retain roughly three quarters of the removable spin
work while restoring the caller's original wait budget. It does not guarantee those event
frequencies survive the change.

At 1×3, pool64's **entire 39900 counted spins are in waits that went on to park**.
At 3×3 the same is true of its 37548 spins. It has no observed short-wait success to pay for
that residual work. Its caller parks are essentially pool0's at 1×3. Thus 64 is not currently
an evidenced latency compromise. It stays archived as a tested arm, not the lead.

The direct pool0/pool64 exploratory contrast at Wikidata-4 is wall
.9895 [.9712,1.0081], CPU .9927 [.9761,1.0096]; text-4 CPU is
1.0106 [.9948,1.0266]. **No general dominance established.** These post-screen paired
contrasts inform ranking only; they do not amend the registered A-referenced verdicts.

The baseline 1×3 calibration is 22.56 ns/spin; multiplying 1080245 pool spins gives
**.02437 s**, about **87%** of the raw timed Wikidata-4 mean CPU difference **.02814 s**.
Helper-only gross estimate: **.01767 s**; retained caller spin: **.00670 s**.
At 3×3 the corresponding gross pool/helper estimates are **.02681/.01938 s**.
These are scale checks, not measured causal allocations or hard upper bounds:

- spin_cost_ns takes the minimum wall elapsed time of three idle atomic-load/spin_loop
  probes, not the contended waiting threads' CPU consumption;
- the count executable has wait-stats instrumentation and no timed ab --spread placement;
- unpark calls are identical by the publication protocol, not evidence of identical kernel
  wakes or wake latency; a park call can consume a token immediately or return spuriously;
- unchanged walk policy does not imply unchanged walk wait events: 1×3 walk spins change
  from 1.204M to 1.114M with pool0. Scheduling and overlap redistribute the work.

For wall time, the relevant model is
**removed exposed contention − added exposed wake/dispatch latency − induced work**.
It is not saved CPU divided by worker count. Join wakes can delay the next block; helper-idle
wakes can delay launching the next team run too. Both must be charged. More parks at 1×3 and
fewer at 3×3 are a reason to split sites, not to fit a universal park price to two totals.

## Which i9 cells decide

Do this only after the two-modes diagnostic is reported and reviewed, and only via a frozen
C5 recipe run by Louis. Keep all modes; no fast-only selection, outcome-derived state labels
or post-hoc shorter grid. Researcher review must settle dependence, order, carryover and
rounds before data. The 18-round CI variance is not an i9 sample-size justification.

**First, a bounded diagnostic**, common fixed build and exact input hashes:

| Context | Fixed comparison | Question |
|---|---|---|
| Forced 1×3, full target affinity | 2000 vs 0 | Is an exposed caller/helper wake penalty visible with spare cores? |
| Forced 3×3, full target affinity | 2000 vs 0 | Does the actual pipelined population turn the CPU saving into wall benefit? |
| Actual public default, requested 10 | old/master2000, candidate2000, candidate0 | Does public dispatch reach the treatment, and what does the new field/code/layout cost before changing its value? |

Use Wikidata and text in each context: **14R decodes** (two forced-plan contexts × two arms × two corpora, plus public10 × three arms × two corpora), R fixed balanced rounds; setup/count/safety work is additional. The old/master-to-candidate2000 contrast is a code/layout control, not assumed identity. Candidate0 must ultimately beat old/master directly. If public default
and forced 3×3 are experimentally identical, prove that before freeze and remove the redundant
diagnostic contrast then, not after seeing its data. Do not remove the old/master code/layout control on the basis of equal2000 budgets. Separate instrumented counts should match
each timed plan/affinity, record site counts and actual thread/plan identity, and not be pooled
into timing.

**Choose the baseline before building the next candidate.** If service has already been
adopted, the public-10 comparison must be service2000 vs service0 at its actual plan;
#321's pipelined PoolWait plumbing does not establish that treatment reaches service.
That requires its own scoped source/safety review. If service is still pending, the diagnostic
can assess legacy pipelined0, but it cannot claim an additive saving for service.

**Qualification remains a separate fresh campaign.** Freeze a narrow, hardware-justified
predicate, preferably requested budget10 on the measured i9 class; do not globally change
SPINS or silently alter plain pools. Propose target Wikidata and text at public10, and the
public9/public11 predicate transitions on both corpora. That gives N=2 target cells,
K=4 target intervals, M=4 fallback wall contrasts; **12R decodes** for this two-arm
qualification grid, before any identity controls. Short/small-block and malformed-input
behaviour must be covered independently; explicitly justify residual workload extrapolation.
Every prospective required cell must pass the existing route. A switch from this two-workload
proposal to another matrix must be decided/reviewed before data.

Off-target CI remains the complete registered standing grid, with valid uncertainty and
fresh exact-head evidence. #316's sharded machinery is reusable tooling, not transferable
performance evidence. Its Researcher REVISE6756808726 still requires a nonqualifying
dependence/provenance pilot and final design; R600 or R960 is not a ready-made qualification
budget. Do not spend that full route on pool0 until the target diagnostic shows plausible
wall superiority.

## Next construction and whole-decoder ranking

**Site split before activation.** Construction-time immutable helper and join budgets;
no per-event autotuner, new notification suppression, flag or publication order. Three arms:
A=(2000,2000), B=(0,0), D=(0,2000), with identical walk policy and task partition.

- Hypothesis: D removes most idle spinning while recovering any B caller-wake penalty.
- CI falsifier: D fails to remove helper spins, introduces correctness/liveness failure,
  or shows significant wall harm versus A. If D does not recover wall versus B and simply
  adds caller CPU, keep B; do not search more budgets.
- Mechanism falsifier: D's join waits/latency do not recover despite restored budget; then
  the caller-spin explanation is inadequate. Record per-site successful-spin and actual
  blocking information separately from park calls.
- Cost: 3 arms × 2 plans × 2 corpora × R rounds = **12R decodes**, plus separate untimed counts
  and safety. Reuse existing both0/both2000 arms only as fresh concurrent controls. Safety
  must cover both mixed budgets, panic/cancel/unwind and publication negative controls.
- No combination credit: this replaces the all-sites policy; it is not an extra gain added
  to pool0's observed saving.

**Activation is second within the wait line.** Existing pools are already lazy. A useful new
activation policy must avoid creating/waking a helper for a run that lacks enough ready work,
not merely park an already parked helper. It adds readiness decisions, launch latency and
possibly less parallel work. First census creations, useful tasks per worker lifetime,
ready-work at activation, peak live scratch, and critical-worker supply. Build only if that
shows an avoidable cost beyond the .018–.019 s helper-spin estimate already attacked here.
Do not hold borrowed team work across an unreviewed lifetime/protocol change.

**Portfolio:** service first for adoption closure; run emitter first for larger kernel
opportunity; pool site split as the cheaper bounded diagnostic; activation after its premise
is counted. Pathfinder's by-data-class record at 2e67726b1 (blob
8515286b36c83ee4480a287476c4b3ec99aec5d6) strengthens text coverage: its stage1 emitter ratio
.987/3.00≈.329 and front-end share39% motivate the line. Its 21–30% whole-decoder forecast
is a serial/local model, not parallel target evidence. Records shows no stage1 win. Charge
run production, table fill, scratch, fallback and exposed critical-worker work; no additive
service/emitter/pool forecast. The closed directory attempt stays closed.

Expected: CPU reduction without exposed wake harm. Observed: CPU reduction, wall improvement
in constrained cells, no demonstrated target wall gain. Gap closed: the pool wait cost is
priced and a viable candidate exists; **zero target qualification cells closed**. Next value:
a small site split distinguishes removable idle waste from critical join latency, while the
higher-return service and emitter lines retain priority.

## Scope and protocol record

No constructors implemented; no compilation, new timings, tests, merges, dispatches or
automation changes. This is not an independent safety review of my earlier proposal.
Source audit: registration b1c9f940e58fb14dc47d9e1254d1072e6e074ce8,
workflow 5f3b052760d5ea886f5851f827f74b1d7ea06093,
coop b57d6af076aded2928721d24e001f6a41c7ee835,
ab 07a9a65b7ee094068da21ad02f21986c2dbd7b6a.
Supplementary #316 draft inspected at 7c582152ca4b3f44674084b4e75362e930645f34
(registration blob 0fe04fb55e208dbafd9b6d8bbf5788e32e67d974), not treated as frozen.

Pinned routing audit flags LOG6757065549 in #162; Coordinator correction6757075751
already records the handling. No re-dispatch. Pinned verify hardcodes FROM coordinator:
route/check and exact publication read-back are recorded without claiming optimizer
verify-pass. Migration head and manifest digest matched before the receipt.
