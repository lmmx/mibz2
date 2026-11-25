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

## The first attempt, and the recipe made to fail soft (25 November, before any data)

**The first attempt submitted nothing.** `just submit bench-character-stage2` ran on the i9 at 5931d711 from 01:16:25Z to 01:37:37Z and exited 1, with no bundle posted (#145 comment 7149767176).
- **The cause, from the code:** the recipe fetched stage 1's bundle to `target/character-stage2/stage1.tar.gz`. `just submit` counts the new `target/*/*.tar.gz` a recipe leaves, found two (that file and the run's bundle), refused both and exited 1.
- **The DRY couldn't catch it:** it ran the recipe bare, not under `just submit`.
- **The run's own bundle probably sits on the workstation,** under `target/character-stage2/`. Nobody has seen its data.
- **Which run counts:** the re-run at the new head is this registration's evidence. If the first bundle is recovered, it is archived as a replicate under that label and never substituted.

**The fix.** Stage 1's bundle is now fetched as `stage1.tgz`, so the recipe's bundle is the only `*.tar.gz` it leaves.

**Fail soft.** The design is unchanged: the cells, the doses, the rule and the pin are as frozen.
- **The recipe's phases:** checks, build, four-arm, inputs, stage1, machine, plan, run, oracle and report.
  - Each runs in a subshell of its own, with `set -e` inside it.
  - Each records its exit status and last line in `phases.tsv`.
  - A phase whose prerequisite failed is SKIPPED, and the next phase runs.
- **The EXIT trap:** it always removes the worktree and the concatenated corpus, then bundles the run directory, with the console log whole.
- **The recipe's exit:** 1 if any phase failed, always with its bundle.
- **The driver:** a decode that fails for any reason is recorded as failed and the run carries on. A permuted corpus that can't be made fails its own cells only.
- **The report:**
  - it lists the phases;
  - a VOID run still reports what it can, for diagnosis only, and its last line names the VOID;
  - a missing decode fails its own row;
  - a failed oracle phase fails the oracle section.

**The checks:** `bench/test_character_stage2.py`, 16/16. The new checks:
- a VOID run's partial report;
- a row failed by a missing decode;
- failed phases and the oracle section;
- the driver carrying on past a failing decode and an unpermutable corpus.

**The DRY of the fail-soft recipe** (`just bench-character-stage2 dry`, at c101a058, on the container). It is archived as `2026-11-25-character-stage2-registration/run-20261125T015111Z.tar.gz` (sha256 2fe4ba42…365c).
- Every phase from checks to oracle passed: 144 decodes, and the oracle arms with 0 mismatches (B/A 0.978, C/A 0.819).
- The report was VOID on the machine alone, so the recipe exited 1, with its bundle.
- The run left exactly one new `target/*/*.tar.gz`.

## Stage 2b, registered from 2a's data (25 November, before any of 2b's)

*Its recipe is `just bench-character-stage2b`, with the same binary and the same pin (9621b7ad) as 2a. It was assigned by the Coordinator's QUEUED 7150768252, after LOG 7150763760 and journal §74. A C1a diagnostic: it decides nothing.*

**2a's bundle:** `character-stage2/run-20261125T020224Z.tar.gz`, evidence 54896f60, SHA-256 64162665…0973.
- Every phase passed through the oracle: 888 decodes, none failed.
- Its report crashed on k 16's NaN coefficients. With the `interval()` guard (442302a0) it exits 0 on the bundle's own records.

**What 2a left open:**
- **k 6 is resolved:** A6a, A6b and A7 control; the scanner is negligible.
- **At 8 and 10, only A3 resolves:** 0.53 [0.14, 0.93] at 8 and 0.74 [0.37, 1.11] at 10.
- **The workers' intervals are ±0.4 to ±0.9 there,** and at 16 nothing resolves.

**The registered sizing rule, applied:** n = ⌈(t·s / (0.10 · log 1.2))²⌉ for each influential coordinate, with s 2a's paired log-ratio SD (unpinned).
- **At 8:** A3 76, A6b 66, A6c 116.
- **At 10:** A3 46, A6b 62, A6c 98, A7 8, A10 37.
- **Every one but A7 is far past the budget,** one queue slot of about 25 minutes, or about 16 blocks of 2a's cells at 8 and 10. The rule caps them there.

**Why the scanner is pinned.** 2a's noise above k 6 is the scanner sharing its core or its CPU:
- **The paired SD of log(T/T0):** 0.7% at k 6, 8.1% at 8, 6.3% at 10, and 10.8% at 16.
- **In 2a's base traces at 8:** the decode took 0.191 s in 11 of 12 when the scanner ran alone on its core, and 0.206 s in the one where it shared its CPU.
- **At 10:** 0.198–0.221 s alone, and 0.230–0.259 s with its sibling busy.

So 2b pins the scanner (`scanner_cpu`) to the first CPU of the core whose pinned calibration ran fastest. That's the `cores` phase: the pacing loop, 2×10⁸ iterations 5 times, pinned on each core's first CPU, the median time. The sibling is kept idle, and every other thread is masked off that core by the process's mask.
- **Expected:** on 2a's isolated traces, the paired SD should fall towards 1%, which would put the workers' coefficients at 8 within about ±0.1 in 16 blocks.
- **If it doesn't fall,** 16 blocks still narrow 2a's intervals by √(16/6) ≈ 1.6.
- **What changes:** the operating point is the pinned one. The link arm ties it to 2a's.

**The cells (pinned unless marked):**
- **At 8 and 10:**
  - the base;
  - the screen, every kept coordinate at +20% (as 2a's, from stage 1's service and the run's calibration);
  - the speed-up arms (below), at 10% and 20%;
  - the held-out A3+A7 at +10%.
- **The surface at 8:** 3×3 as 2a's.
- **The link (unpinned):** 2a's base and A3 at 8.
- **The memory dose (unpinned, as 2a):** at 8 and 20.
- **Placement (unpinned):**
  - k 4 with 2a's masks: 7 CPUs on distinct cores against 4 cores' sibling pairs;
  - k 8: 10 distinct against 5 cores' pairs.

That's 55 cells on the i9 in 16 randomised blocks, with traces in the first 2 blocks only (the achieved doses and the identities need no more; every wall is a timed decode with the ledgers off). 990 decodes, about 23 minutes after 50 s of pre-warm.

**The speed-up arms.** Pacing only slows an activity. A scanner made faster by L = 1 + λ is read through homogeneity: if the wall is homogeneous of degree 1 in the activities' times, the scanner alone faster by L is the arm with every other activity paced by L, over the arm with every activity paced by L. In each block, paired against the pinned base:
- **H** = log(T_all / T0) / log L, the check (1 if every activity's time is paced);
- **e_down** = log(T_scan / T0) / log L, the wall's elasticity to the scanner (A2, A3, issue) slowed by L;
- **e_up** = log(T_all / T_others) / log L, its elasticity to the scanner made faster by L;
- **d** = e_down − e_up.

"Others" are the workers' activities (A5, pool, A6a–c, A7, A8, A9, publish) and the reader's A10. "All" are those and the scanner's.

**Dropped:**
- **k 16:** nothing resolved, and four doses missed.
- **The gates:** no effect.
- **The pulses:** done.
- **The cost check:** the walls are the ledgers off, and the traces serve only the doses.
- **The oracle arms.** The four-arm binary is the pin's decoder plus 1,185 lines of feature-gated scan-oracle code in 8 files, so its A wall's +10.9% gap is that build's, not a layout effect. Porting it into this binary would break the pin. The speed-up arms take its place within one binary.

**THE C1a RULE (stage 2b; it decides nothing).** Intervals are t-intervals over the 16 blocks.
- **The screen at 8 and 10:** as 2a's (CONTROLS, NEGLIGIBLE, UNRESOLVED, DOSE-MISSED), the achieved dose from the 2 traced blocks. The anchors' controllers, in order, are the migration.
- **The speed-up, at each k and λ:**
  - **HOMOGENEOUS** iff H's interval lies within [0.85, 1.15]. Then e_up is read as the share of a scanner speed-up by L that reaches the wall.
  - **NOT-HOMOGENEOUS** otherwise, and e_up isn't interpreted.
  - **ASYMMETRIC** iff d's interval excludes 0.
  - **The answer to "how much of a scanner speed-up survives at 8"** is e_up at k 8, λ 10%, with its interval.
- **The surface:** its roundness, as 2a's.
- **The contrasts:**
  - the link's A3 over its base, and the memory and placement arms, as 2a's: **CHANGED** iff the interval excludes 1;
  - the pinned base's wall over the unpinned link base, reported.
- **Validation (held out):** A3+A7 at +10% at 8 and 10, observed against exp((C_A3 + C_A7)·log 1.1) from 2b's own screen.
- **The pin:** each pinned trace's scanner must run on the plan's CPU alone. Otherwise the report labels it **PIN-BROKEN** and names the cells; that's a label, not a VOID.
- **INCOMPLETE** if any trace fails an identity.
- **VOID** if any of these:
  - the machine is not the i9-10900K;
  - a cell's timed decode (any block) or trace (blocks 0–1) is missing, duplicated or failed;
  - an output is not 4 × 209,700,196 bytes;
  - any decode has a major fault;
  - stage 1's bundle is not its SHA-256.

**The checks:** `bench/test_character_stage2.py`, 21/21. The 2b checks:
- a wall homogeneous of degree 1, a smooth max with known e_down 0.776 and e_up 0.628, recovered with H = 1 and labelled ASYMMETRIC;
- an all-paced arm short of its dose, NOT-HOMOGENEOUS;
- a missing decode, VOID;
- a scanner off its pin, PIN-BROKEN;
- plan2b's pin, mask, anchors and the speed arms' nesting.

**The DRY of 2b** (`just bench-character-stage2b dry` at d2ffd2c3: one block, no settle, a 2 s pre-warm; a 4-CPU container, not evidence). It is archived as `2026-11-25-character-stage2-registration/run-20261125T023531Z.tar.gz` (sha256 3e2d4340…e8f2f77d).
- Every phase from checks to run passed, the new cores phase included.
- The plan pinned the scanner to CPU 2. With no SMT here, there was no sibling to idle and no placement cells: 51 cells.
- The run made 102 decodes, all byte-exact, every trace's identities holding. The scanner was on its pinned CPU alone in 43 of 43 pinned traces.
- The report was VOID on the machine alone. The doses and the speed figures mean nothing on a 4-CPU VM running 8 and 10 workers.
- **Since the DRY:**
  - the driver builds the permuted corpus only for a plan that has permuted cells (2b's has none; the DRY logged its absence and carried on, as the fail-soft path should);
  - the report gained the pin's check and its stage in its title.
