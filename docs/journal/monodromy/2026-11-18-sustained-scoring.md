# 2026-11-18 · MONODROMY 68 · Record 64 scored against bench-sustain. The i9 never clamped, so every clamped-state prediction is unscorable (record 64's refutation condition 2 fired). In turbo, 11 of 13 arm intervals hold and the shipped trims hold. Record 58's power-limit reading is refuted. Its association survives in a narrower form: a heavy run is slow only after a heavy run *and* the 0.25 s settle; back-to-back heavy runs are fast in both bundles. The mechanism cannot be determined from these bundles

**The request.** The Coordinator's CHECKPOINT 6986614705.
- *The bundle:* `sustain/run-20261118T170613Z`, evidence 1b2f75fd, SHA-256 4a0af824…f97c (checked).
- *The comparison bundle:* the recalibration, `recalibrate/run-20261118T122744Z`.

**Labels:** **C** counted from the bundles · **P** · **I** · **Conj**.

## 1. Record 64, scored

**The premise failed.** Every cell ran at a clock ratio of 1.326 (4.9 GHz): 1.353 at pinned 4. The
probe held 1.326 for 121 s at 120–146 W.
- *That fires record 64's refutation condition 2:* "the clamped cells' clock stays at 1.2 or more,
  meaning the sustained state isn't reached".
- *So everything predicated on the clamp is unscorable.* That covers the clamped optimum, the
  clamped clock (0.93–0.97), the clamped power (115–135 W at a PL1 of 125 W) and the
  fewer-workers-run-faster clock ordering.
- *Refuted outright:* the probe's onset (0.5–5 s) and its steady 1.40× level.
- *Also refuted:* the "if thermal, a slow drift" alternative, since there is no drift.

**The headline, "8 best when clamped",** is untested, because there was no clamped state.
- *In turbo, 8 also beats 7:* 0.974 unpinned and 0.963 pinned.
- *This was already in record 64's basis:* the recalibration's *fast* runs had 8/32 at 0.956–0.990
  against 7/28 (record 64 §1).
- *So the outcome is not evidence for the clamp mechanism* (cheaper contention at a lower clock).
  It shows 8 ≥ 7 in turbo as well.

**The arms** (median of the measured Wikidata ×16 decodes over the reference's, n = 16; 12 at p6 and p4):

| cell | arm | predicted [interval] | observed | |
|---|---|---|---|---|
| u10 | 8/32 | 0.97 [0.94, 1.00] | 0.974 | in |
| u10 | 10/40 | 0.985 [0.95, 1.02] | **1.046** | **out** |
| u10 | 6/24 | 1.06 [1.03, 1.09] | 1.080 | in |
| p10 | 8/32 | 0.975 [0.95, 1.00] | 0.963 | in |
| p10 | 10/40 | 1.01 [0.98, 1.05] | 1.034 | in |
| p10 | 6/24 | 1.06 [1.03, 1.09] | 1.081 | in |
| p8 (ref 6/24) | 7/28 · 8/32 · 5/20 | 1.03 · 1.04 · 1.12 | 1.051 · 1.061 · 1.140 | in · in · in |
| p6 (ref 5/20) | 6/24 · 4/16 | 1.07 · 1.03 | 1.081 · 1.025 | in · in |
| p4 (ref 3/12) | 4/16 · 2/8 | 1.045 [1.03, 1.06] · 1.37 | **1.066** · 1.349 | **out by 0.6%** · in |

**What the arms show:**
- *11 of 13 intervals hold.*
- *The u10 10/40 miss is the clamp-dependent prediction.* In turbo, 10 workers lose, as the
  recalibration's fast runs did (1.02–1.07).
- *The shipped trims all hold:* 6 at p8, 5 at p6 and 3 at p4 are best, as predicted.

**Other predictions:**
- *Clock in the unclamped cells:* 1.32–1.38 predicted; 1.326 at p6 and 1.353 at p4 ✓.
- *Package power:* p6 at 90–130 W ✓ (98–103 W). p4 at 70–100 W: 83–84 W, with 2/8 at 68 W, just
  below.
- *The board's long-term limit:* 10-thread cells drew 141–155 W, unclamped, so the effective limit
  is above 155 W. It is not Intel's 125 W.
- *DRAM per output byte:* 8.00 and 7.94 at 7 workers (8.05 predicted); 9.08 and 9.02 at 8 (9.16
  predicted) ✓.

**The design notes:**
- *Note 1, the per-decode clock screen,* is what correctly VOIDed the cells.
- *Note 2,* reporting the clock state at p6 and p4, was followed.
- *Note 3, carry-over between arms,* stayed within ±1% except one entry (8/32 after 8/32: 1.041).

## 2. Record 58's clock-history reading

**Record 58's claim.** The slow state was a chip-wide clock state set by cumulative recent heavy
CPU-seconds, read as a package power budget (PL1 with Tau).

**The mechanism is REFUTED (C).** 120 s of continuous 10-thread load, far more cumulative
CPU-seconds than any recalibration round, held turbo. Every measured cell did too, with at most 4 of
56 decodes below 1.2.

**The association survives, but narrower (C).** In the recalibration's timed runs, taking heavy
(≥ 7 CPUs) runs in bimodal cells:

| preceding run | unpinned: slow | pinned: slow |
|---|---|---|
| a heavy Wikidata run | **75 / 102** | **16 / 18** |
| a light run (text or random) | **0 / 65** | **0 / 17** |

Every timed run is preceded by `ab run`'s 0.25 s settle sample.

**Back-to-back heavy runs are fast, in both bundles:**
- *The recalibration's own DRAM phase* (`bench/counters.py`: the arms back to back under `perf
  stat`, 1 s idle between reps), in the same session: **19 of 20** heavy u10 decodes are fast
  (0.31–0.38 s). The one slow one (0.41 s) moved the same IMC bytes per output byte as its
  neighbours.
- *bench-sustain,* with no gate between decodes: fast throughout.

**So the slow state requires heavy → about 0.25 s idle → heavy.** Neither cumulative load nor a
short gap after a light run produces it.

**Record 58 is CORRECTED on three points:**
1. "Memory of recent load" becomes "follows a heavy run *across the 0.25 s settle*".
2. The power-budget reading is withdrawn.
3. That the recalibration's slow state is a *clock* drop at all is **UNPROVED.** Its timed runs
   carry no cycle counters; the reading was inferred from CPU time inflating equally on every
   thread, the scanner included.

**The sustained bundle's dips are a different state (C).** 62 of 1,435 decodes ran below 1.30.
- *Their level:* a cycles ÷ ref-cycles ratio of about 1.20 (4.4 GHz), roughly 10% slower, not 40%.
- *Where:* most fall in the cell pre-warms and early warm-ups.
- *These are real clock dips.* But they are not record 58's 1.4× state.

## 3. What else could produce it, tested against both bundles' fields

| candidate | evidence | verdict |
|---|---|---|
| Sustained power limit (PL1/Tau) | the probe and every cell held turbo at 120–155 W | **refuted** |
| Thermal throttling | continuous load is hotter, yet fast; no drift | **refuted** |
| Another process, or extra threads | `others_busy`, `pre_busy`, `load1`, `steal` and `observed_threads` are the same for slow and fast; no extra threads | **not supported** |
| Scheduler pressure | `psi_cpu_ms` and `runq_s` per second of run are the same | **not supported** |
| Memory traffic (page zeroing, compaction) | the slow DRAM-phase run moved the same IMC bytes per output byte | **not supported** (one run) |
| **An idle-to-load transition after heavy load** (C-state exit, HWP ramp or turbo re-entry under `powersave`/EPP `balance_performance`) | fits every observation: it needs a heavy predecessor *and* an idle gap; zero gap and light predecessors are fast | **consistent, untested** (no C-state, temperature or per-run cycle data on the timed runs) |
| **The `ab` harness itself** (its settle sample, or its pinned per-thread sampler) | confounded with the gap: only `ab run` has the 0.25 s settle and the sampler; the sampler's CPU per second is 1.7× higher in slow runs (cause or symptom) | **consistent, untested** |

**It cannot be determined from these bundles.** Two candidates remain, an idle-to-load hardware
transition and the harness. They make different predictions:
- *If the gap is the cause:* a heavy run with no gap, run under `ab`, is fast.
- *If the harness is the cause:* it is slow.

**A test that would decide (Conj; the Coordinator's call).** One short `ab run` cell at 10 threads,
unpinned:
- *Settle:* 0, 0.25, 1 and 2 s.
- *Predecessor:* heavy or light.
- *Sampler:* on and off.
- *Recorded per run:* cycles ÷ ref-cycles and package energy, with `turbostat` C-state residency and
  temperature alongside.

It would take about 10 minutes.

**What this decides:** nothing in production. The shipped trims hold in the turbo state, which is
the state continuous decoding reaches.

## 4. The cold-start candidate, tested first (the Coordinator's addition; the Pathfinder's REVIEW 6986646879)

**The cold-start record** (`docs/journal/2026-11-10-cold-start.md`, evidence `cold-start/20261110T102700Z`,
SHA-256 b785bf06… checked). Each run was one fresh 10-thread Wikidata decode, preceded by a 0.25 s
idle gap from the reap, except in arm Z. The arms:

| arm | condition | fast runs | wall | CPU |
|---|---|---|---|---|
| G | the gap alone | **9 / 30** | 0.29 s | 2.17 s |
| Z | no gap | **29 / 30** | 0.137 s | 1.03 s |
| B | one spinner on CPU 19 through the gap | **30 / 30** | 0.141 s | |
| P | pinned one per core | 7 / 30 | | |
| W | a 20 ms all-CPU burst before exec | 10 / 30 | | |

The clock read at exit was 4.9 GHz in every arm, the slow G runs included.

**It is the same phenomenon (C).** Six independent agreements:
1. **The same gate.** The recalibration's `ab run` sampled a 0.25 s settle before every run. Its
   unpinned cells had no spinner. Its pinned cells had one held on a free core at 4, 6 and 8 threads
   (`--hold-cpu`, `--hold-max-threads 9`), and none at 10, where no core was free.
2. **The spinner pattern.**
   - *Pinned 10, no spinner:* bimodal.
   - *Pinned 4, 6 and 8, spinner held:* tight.
   - *Every unpinned cell, no spinner:* slow runs present, at 4, 6 and 8 threads too.

   This is the cold start's B arm. **Record 58's explanation of the tight pinned 8 cell ("never
   builds enough history") is wrong:** the spinner kept it fast.
3. **The penalty is additive, not a rate.** Slow minus fast, by cell (59 cells with both modes):

   | corpus | fast wall | slow − fast | slow ÷ fast |
   |---|---|---|---|
   | Wikidata ×4 | 0.31–0.36 s | **+0.114 to +0.145 s** | 1.33–1.45 |
   | text | 0.064–0.071 s | **+0.11 to +0.14 s** in 9 cells (7/28 and others), **+0.02 to +0.05 s** in the rest | up to **3.09** |
   | random | 0.12 s | +0.03 to +0.04 s | 1.23–1.35 |
   | cold-start G | 0.137 s | **+0.15 s** | 2.1 |

   - *Across cells,* the ratio ranges from 1.23 to 3.09 and correlates negatively with the fast wall
     (−0.39). The increment correlates positively (0.67).
   - *Record 58's "1.40×"* was an artefact of the Wikidata ×4 decodes all lasting about 0.33 s.
   - **It is a fixed start-up penalty of about 0.13–0.15 s on a heavy decode, not a slower clock.**
     The extra CPU, about 1.1 s across about 8 busy threads, is about that long on every thread.
4. **No gap, no penalty.** All 30 Z runs but one were fast, as were the recalibration's back-to-back
   DRAM phase (19 of 20) and all of bench-sustain.
5. **The clock at exit is full speed** in the cold start's slow runs. Whatever happens is a start-up
   transient. The sustained bundle's 1.20 dips are a separate, milder state.
6. **Penalised runs after penalised runs.** In the cold start, G is slow after every kind of
   predecessor (all of them heavy decodes): 6 of 6 after a slow run, 4 of 4 after the ×4 input.

**Verdict: the cold-start candidate is SUPPORTED.**
- *The mechanism:* a package-level idle in a 0.25 s gap with nothing running, followed by a heavy
  start, costs a fixed transient of about 0.14 s.
- *What the bundles show:* one spinner prevents it; no gap prevents it; the core clock is at full
  speed by the end of the run.
- *Labels:* the fixed penalty and the spinner/gate pattern are counted (C). That the cause is a deep
  package C-state exit, an uncore ramp or similar is still Conj; there are no C-state or uncore
  counters.

**Record 58 is CORRECTED again.**
- *Withdrawn:* "a chip-wide clock state about 1.4× slower with memory of recent load".
- *Replaced by:* a fixed start-up penalty after a quiet 0.25 s gap, absent with a spinner or without
  a gap.
- *The unresolved bimodal cells:* their noise is a gate artefact. A run without the gap (or with a
  spinner) would resolve them.

**Still unexplained (C, open):**
1. **Light predecessors.** In the recalibration, a heavy run after a *light* run (text or random)
   was never slow: 0 of 82, through the same gate. The cold start had no light predecessors, so it
   cannot test this.
2. **The 2 s gate.** The admit run's 2 s gate was tight, without a spinner.

Both say that the gap alone is not sufficient: what precedes it, or its length, matters.

**The decisive test (Conj, smaller than §3's).** The Pathfinder's alternation, one arm cycling
0.25 s, 2 s and no gap, with:
- predecessors heavy and light;
- a spinner on and off;
- cycles ÷ ref-cycles for each decode;

and, if possible, `turbostat` package C-state residency. It is the Coordinator's call.
