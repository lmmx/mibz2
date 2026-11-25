# bench-character-stage2: the decoder's character, stage 2 (registration)

*Registered before any data, frozen once stage 1's bundle fixed its anchors: bench-character, #841 at 94cb742d, `character/run-20261125T001135Z.tar.gz` on the evidence branch at 870e61d7 (SHA-256 bb72107b…2204; journal §66). Design: `docs/journal/2026-11-24-algorithm-level/briefs/character-review-decisions.md` §2 (3d273257, with §5's decisions) on `docs/journal-algorithm-level`, from the brief `briefs/character.md` §5–§7b, and the Coordinator's additions of 25 November (journal §66–§67). Assigned to the Pathfinder by the Coordinator. A C1a diagnostic with its rule posted on #145 before its data; it decides nothing. One review round by the Researcher.*

## The pin

Stage 2 runs on stage 1's base: master at 9621b7ad (bench-character's pin, #841 at 94cb742d), with only this branch's harness and driver added. It never takes a later master during the study. The Optimiser's scanner plumbing fix (QUEUED 7147862124) changes the scanner's demand, so taking it mid-study would change the system the anchors and doses were fixed for. The recipe checks that src differs from the pin only in the character feature's files (§5's decision).

## What stage 2 asks

Stage 1 measured the network as it is: each count's demands, waits, occupancies, limits, k_balance, and the workers' idle time while the scanner is active. Stage 2 intervenes, to find which activities *control* throughput at the counts where control changes hands, and through which mechanism. It also measures the knee's shape, which the simulation and the fluid model need. All of it is a means to the models (brief §7): the trace-driven simulation, and the documented surrogate for mean-value analysis. Both are fitted on stage 1's per-block service vectors and judged on conditions held out here.

## Stage 1's bundle applied: the anchors (fixed)

The rules below were drafted before stage 1's data (b2169a0d, agreed by the Coordinator) and applied to its bundle as written:
- **k\* = 8.** The scanner's whole-decode on-CPU share (its `schedstat` run time over the wall) is 0.893 at 7 and 0.967 at 8. The stage-1 report's `u_scan` passes 0.9 at 7 (0.902), but it is a different measure: the steady phase's share from the ledgers, which leaves out the start-up and drain. The registered rule governs, and it agrees with k_balance = 8 [8, 8], with the limit's move from WORKER- to SCANNER-LIMITED at 8, and with the near-optimal set {8}.
- **The step = 16.** The largest rise in the scanner's demand a block above k\* + 2: +7.8%, from 121.9 to 131.4 µs (the rule needs 5%).
- **The anchors: 6, 8, 10 and 16.** 10 is also the bridge to the four-arm test.
- **The coordinates:** all nine kept at every anchor (A2, A3, A5, A6a, A6b, A6c, A7, A9, A10). A6 is split into its parts, since each passes 5% of the workers' time. A8 is under 3% everywhere and is dropped.
- **The achieved frequency:** 4.91 GHz at every anchor (stage 1's cycles over reference cycles; 5.28 at 1 worker).

The run recomputes the plan on the i9 from the same bundle, checked by its SHA-256, with the pacing loop's calibration taken there. The anchors and coordinates depend on the bundle alone, so they are as above; the doses in iterations follow from the calibration.

## Decided before the data, from §2 (rules applied to stage 1's bundle)

**The anchors** (§2, "by rule from stage 1"):
- **k\*** is the smallest count at which the scanner's on-CPU share passes 0.9: the median over stage 1's observations of the scanner thread's time on a CPU (its `schedstat` run time over the decode) divided by the decode's wall. Where no count passes 0.9, k\* is the count with the largest share, and that is recorded.
- **The step**: the count, above k\* + 2, at which the scanner's demand a block (stage 1's D_scan) rises most from the count below, if that rise is at least 5%; else 20. The scanner profile put it at 16.
- **The anchors** are k\* − 2, k\*, k\* + 2 and the step. 1 and 20 stay as endpoint and counter anchors only. 10 stays as the bridge to the four-arm test (it is the oracle arms' count if k\* is not 10; see below).

**The pacing** (§2):
- **The loop:** the binary's `pace` switch, a dependent multiply-add chain of a fixed number of iterations at an activity's end (src/character.rs `treat`). It touches no memory, so it scales occupancy, not memory demand, and the write-up says so (C).
- **The dose's size, in core cycles:** an activity's +δ dose at count k is δ × its mean exclusive service a visit at k in stage 1 (its state time over its visits), converted to cycles at that count's achieved frequency (stage 1's cycles over reference cycles), then to iterations by the loop's own cycles per iteration. The loop is calibrated at the run's start with the hardware counters (`character --calibrate`, the library's loop copied; their disassembly is the same): cycles per iteration over 10⁷ iterations, five times, the median. The compiler folds 8 iterations into one multiply-add with composed constants, still a dependent chain, so an iteration costs about half a cycle. Where no hardware counter counts, the calibration falls back to that 0.5, and that is recorded. The doses never come from slowed runs.
- **The record:** each paced block's wall and thread-CPU time come from the ledgers. The activity's time includes its pacing, so the achieved dose is measured, not assumed.

**The coordinates** (§2 step 1):
- **Candidates:** A2, A3 (the scanner); A5, A6, A7, A8, A9 (a worker); A10 (the reader).
- **A6:** paced as one at A6c's end, its dose δ × (A6a + A6b + A6c). It is split into its three parts only if each part exceeds 5% of the workers' time at that anchor (M).
- **Drops:** a coordinate is dropped at an anchor where it is under 3% of its thread's stage-1 time. The exception is A10, which is kept if the reader's waits exceed 2% (N). A coordinate with no visits (a route the corpus never takes) is dropped.

## Walls and traces apart (journal §66)

Stage 1 classified itself PERTURBED: the ledgers' on/off cost was 1.040 [0.991, 1.091] at 10 and 1.016 [0.985, 1.048] at 20, over only 6 pairs. So stage 2 keeps them apart:
- **Walls:** every wall it judges is a decode with the ledgers off.
- **Traces:** each cell has, in each block, a trace decode of its own with the ledgers on, and the achieved doses and the full response matrix come from the traces.
- **The cost check:** one more base pair a block at each anchor makes it 12 on/off pairs a count, under stage 1's bound (PERTURBED iff the geometric mean exceeds 1.01, or its 95% interval lies wholly above 1.005). PERTURBED there applies to the traces' responses, not to the walls.

## The steps (stage 2a, one run)

1. **The screen** (§2 step 1). At each anchor, a fresh baseline and each kept coordinate at +20%, one at a time. Six randomised blocks: in each, every (anchor, arm) once, in a drawn order. An observation is one decode in its own process, with the ledgers on, so each arm's queues and other stages' service times are measured.
2. **The surface** (§2 step 2): a full 3 × 3 at k\* on (scanner dose, all-worker dose) ∈ {0, 10, 20}%. The scanner dose paces every scanner activity in proportion (A2, A3 and the issue, each by its own δ), and the worker dose every worker activity (A5 to A9 and the hand-offs). Six blocks.
3. **The mechanism contrasts** (§2 step 3), six blocks each:
   - **Gates:** W ∈ {4k, 8k} × the ready bytes' cap ∈ {12, 64} MiB at k\*, with the same k + 2 floor. The ledgers' admission predicates show which gate each wait met, so each arm shows which gate actually changed. *The window is a harness switch new in this binary (see "One binary" below).*
   - **Placement:** at k = 4, the decoder's threads (4 workers, the scanner, the feeder and the reader: 7) confined to 7 logical CPUs on 7 distinct cores, against 8 logical CPUs on 4 cores, both siblings each. The confinement is the process's CPU mask (`taskset`, inherited by every thread), so the library needs no placement hook. Turbo free against turbo off is crossed with it where `intel_pstate/no_turbo` is writable without a password. Where it is not (the run is unattended: no sudo), the placement factor runs alone, and each role's achieved frequency (cycles over reference cycles) is analysed as a covariate. **The SMT and turbo factors are then not separated**: sibling sharing and frequency droop move together, and the report says so (§5's decision). Whether the workstation can make `no_turbo` writable for the run is Louis's to say.
   - **The scanner's placement:** at k = 10 and at the step, the scanner pinned to a favoured core (the highest `acpi_cppc/highest_perf`, else the highest `cpuinfo_max_freq`) with its sibling outside every thread's mask; against the scanner pinned to a worker core's sibling (co-pinned); against unpinned. This takes over the pinned scanner-profile variant (QUEUED 7147262673 B).
   - **Memory:** at k\* and 20, a memory dose at A7's end (each Open block) of 2 MiB of the worker's own scratch, streamed against chased, against none. The dose is the binary's `dose` switch: `src/dose.rs` is inert on this path. 2 MiB is about one core's share of the i9's 20 MiB L3, at equal bytes for both kinds. In the container's preflight, 8 MiB chased swamped the block: it decoded 4.5–6× slower.
   - **Oracles** (§5's decision): arms B (supplied codes) and C (the scanner's output replayed) at the knee, run from the four-arm binary (#835 at its tagged commit), reported only as ratios against that binary's own A. The two binaries' A walls at the knee are compared, and any gap is reported as a layout effect.
4. **The pulses** (journal §67): the dose on places 600–800 alone (of 1,724), at k\* = 8 and at 10, on the scanner (A3) and on emission (A7), beside a pulse base.
   - **The switch:** the binary's `range` switch. Its pacing hook reads each thread's current block from the ledger (a worker's from `Got`, the scanner's from `ScanStart`), so the pulses run with the ledgers on, each block's row kept. This is the one change to the library in this binary.
   - **The dose:** +50%, sized from stage 1's noise. Over places 600–800 the reader's rate varied 3.4% (k = 8) and 10% (k = 10) between decodes. A3 is 77% of the scanner's demand a block at 8, so +50% on it should cut the rate there by about 38%, more than 9 standard errors over 6 decodes even at 10's noise. The 600 blocks before the pulse are its lead-in to the steady state.
   - **Reported from the traces:**
     - **the pulse's effect:** the arm's reader rate over places 600–800 against its pulse base's at the same places in the same block. The places' blocks differ (the corpus's second copy begins at 431), so a rate over another window is no baseline: in the preflight the undosed base's rate over 600–800 was 1.30 times its rate over 400–600;
     - the reader's rate during and after the pulse, over its rate before (places 400–600);
     - the recovery time: from place 800 taken to the first 50-block window back within 5% of the rate before;
     - the job queue's mean depth before and during;
     - when the workers first find the queue empty after the pulse's start.
5. **Validation** (§2 step 4): about 20% of the time on conditions excluded from every fit.
   - A combined +10% on A3 and A7 together, at k\* and k\* + 2.
   - The window permutation: the corpus's 12 streams (one a window) in one fixed permutation drawn from the seed, at k\* and 20. The 12 streams are split at their byte-aligned stream headers and the decode's CRCs check the result.

   A model revised after seeing them needs fresh validation.

## Stage 2b: confirmation

The screen's paired variance fixes the confirmation's sample sizes once (C, M). For each coordinate the screen marks influential (its coefficient's 95% interval excludes zero, or its point estimate exceeds 0.10 in magnitude), the pairs needed for a half-width of 0.10 on its coefficient: n = ⌈(t·s / (0.10 · log 1.2))²⌉, s the screen's paired log-ratio SD. They are capped by the time budget, the largest coefficients first. 2b is a recipe of its own, registered from 2a's data before its own. No interval is extended until significant.

## One binary

All of stage 2a's arms run one binary: stage 1's library with two additions. The harness gains the window switch (examples/character.rs), and the library's pacing hook gains the pulses' block range (src/character.rs `treat`: a range check, and the scanner's block taken from its `ScanStart`). Stage 2a carries its own baselines at every anchor. The stage-1 and stage-2 baselines at k\* are compared, and any gap is reported as a layout or day effect, not a treatment's. The oracle arms run from the four-arm binary, as decided.

## The estimands, kept apart (C)

- **Baseline finite-difference sensitivity** at a count: Ĉᵢ = mean over blocks of (log T_i − log T_0) / log(1 + δ_achieved), δ_achieved measured from the ledgers. The interval comes from the paired blocks.
- **The surface's marginal effects** are reported as the surface's own estimand and never mixed with the screen's coefficients, nor across counts.
- **Small effects** are reported as intervals, never as zeros. Near a switch the responses are one-sided, and the report says so where +δ and −δ would differ (only +δ is run).

## The full response, every arm

For every arm:
- the throughput;
- every other activity's service time;
- the scanner's waits by cause, and the workers' idle time while the scanner is active;
- the buffers' occupancies (admitted, ready bytes);
- the achieved frequency;
- where counted, the memory controller's traffic.

All from the same ledgers as stage 1.

## THE C1a RULE (DRAFT; it decides nothing)

- **Each screened coordinate at each anchor:**
  - **CONTROLS** iff its coefficient's 95% interval lies wholly above 0.10;
  - **NEGLIGIBLE** iff wholly within ±0.10;
  - **UNRESOLVED** otherwise.

  The anchors' controllers, in order, are the control's migration.
- **The surface:** the knee's roundness is the ratio of the (20, 20) cell's slowdown to the sum of the (20, 0) and (0, 20) cells' slowdowns. 1 is additive, as on a smooth max; near 0.5 is a sharp max's kink. It is reported with its block-bootstrap interval.
- **Each mechanism contrast:** its ratio to its baseline with its interval, and **CHANGED** iff the interval excludes 1. The gate contrast also reports which predicate held each wait in each arm.
- **DOSE-MISSED** (a label, not a VOID): a coordinate whose achieved dose (from the ledgers) misses its target by more than 25% at an anchor. Its coefficient uses the achieved dose either way.
- **INCOMPLETE, PERTURBED and VOID** as stage 1's, applied to stage 2's arms. Also **VOID** if any of these:
  - the oracle binary's A differs from this binary's baseline at the knee by more than 3%. That is reported as a layout effect: it is VOID for the oracle ratios only.

## The run

- **Cells:** 77 at the anchors above. The screen has 40: four anchors, a base and nine coordinates each. The surface 9, the gates 4, the placements 2 (4 where turbo can be switched), the scanner's placements 6 (on the i9's favoured core), memory 6, validation 6, the cost check 4 and the pulses 6.
- **Blocks:** 6 randomised. In each, every cell's timed decode and its trace (a pulse's trace alone), in a drawn order, after 1 s of settle each: about 900 decodes.
- **Before and after:** 50 s of discarded pre-warm first. The oracle arms at k\* run after: the four-arm binary, 8 rounds.
- **The time:** about 27 minutes on the i9 with both builds.

## Open for the review

- the dose for the memory contrast (2 MiB at A7's end);
- the oracle arms' wiring to the four-arm binary (its `ab` arms at its tagged commit), to be built at the freeze;
- the step rule's 5%;
- the placement's mask construction on the i9's topology;
- whether the turbo factor is worth a password prompt (the run would then not be unattended).

## The preflight

**The DRY preflight of the recipe as frozen** (`just bench-character-stage2 1`: one block, no settle, a 2 s pre-warm, 4 oracle rounds; at 394c23e5). It ran on a 4-CPU Xeon container (meaning nothing for the i9), and is archived as `2026-11-25-character-stage2-registration/run-20261125T003609Z.tar.gz` (sha256 6960cd7a…6a86, 1.4 MB):
- **Setup:**
  - both builds passed: this binary at the pin, and the four-arm binary at 5731db19 in its own worktree;
  - stage 1's bundle was fetched from the evidence branch and checked by its SHA-256;
  - the calibration fell back to 0.5 cycles an iteration (no PMU here);
  - the plan gave k\* 8, step 16, anchors 6, 8, 10 and 16, and 75 cells (no favoured core and no writable `no_turbo` here, so no scanner-placement or turbo cells).
- **The decodes:** all 144 (69 timed, 75 traces) gave 4 × 209,700,196 bytes, and every trace's identities held. The six pulse traces kept their block rows.
- **The oracle arms:** they ran at 8 with every check matching (0 mismatches). B/A 0.981, C/A 0.812, D/A 0.771 over 4 rounds. The layout check put the four-arm binary's A −0.68% from this binary's baseline.
- **The doses missed their targets here, as expected.** The plan sizes them in cycles at the i9's 4.91 GHz with the i9's calibration, and this VM runs about 2.1 GHz on the fallback calibration, so +20% landed near +9%. On the i9 both are measured there.
- **The verdict:** the report voided the run on the machine alone.

Since the preflight, the report's pulse table gained the paired effect above (the scorer alone).
