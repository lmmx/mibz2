# bench-character-stage2: the decoder's character, stage 2 (DRAFT registration)

*DRAFT. It freezes only once stage 1's bundle (bench-character, #841 at 94cb742d, tag ready/bench-character) has landed. That bundle fixes every quantity below that is written as a rule. Design: `docs/journal/2026-11-24-algorithm-level/briefs/character-review-decisions.md` §2 (3d273257, with §5's oracle decision) on `docs/journal-algorithm-level`, from the brief `briefs/character.md` §5–§7b. Assigned to the Pathfinder by the Coordinator, 25 November. A C1a diagnostic with its rule posted on #145 before its data; it decides nothing. One review round by the Researcher.*

## What stage 2 asks

Stage 1 measured the network as it is: each count's demands, waits, occupancies, limits, k_balance, and the workers' idle time while the scanner is active. Stage 2 intervenes, to find which activities *control* throughput at the counts where control changes hands, and through which mechanism. It also measures the knee's shape, which the simulation and the fluid model need. All of it is a means to the models (brief §7): the trace-driven simulation, and the documented surrogate for mean-value analysis. Both are fitted on stage 1's per-block service vectors and judged on conditions held out here.

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

## The steps (stage 2a, one run)

1. **The screen** (§2 step 1). At each anchor, a fresh baseline and each kept coordinate at +20%, one at a time. Six randomised blocks: in each, every (anchor, arm) once, in a drawn order. An observation is one decode in its own process, with the ledgers on, so each arm's queues and other stages' service times are measured.
2. **The surface** (§2 step 2): a full 3 × 3 at k\* on (scanner dose, all-worker dose) ∈ {0, 10, 20}%. The scanner dose paces every scanner activity in proportion (A2, A3 and the issue, each by its own δ), and the worker dose every worker activity (A5 to A9 and the hand-offs). Six blocks.
3. **The mechanism contrasts** (§2 step 3), six blocks each:
   - **Gates:** W ∈ {4k, 8k} × the ready bytes' cap ∈ {12, 64} MiB at k\*, with the same k + 2 floor. The ledgers' admission predicates show which gate each wait met, so each arm shows which gate actually changed. *The window is a harness switch new in this binary (see "One binary" below).*
   - **Placement:** at k = 4, the decoder's threads (4 workers, the scanner, the feeder and the reader: 7) confined to 7 logical CPUs on 7 distinct cores, against 8 logical CPUs on 4 cores, both siblings each. The confinement is the process's CPU mask (`taskset`, inherited by every thread), so the library needs no placement hook. Turbo free against turbo off is crossed with it where `intel_pstate/no_turbo` is writable without a password. Where it is not, the turbo factor is skipped and recorded (the run is unattended: no sudo).
   - **The scanner's placement:** at k = 10 and at the step, the scanner pinned to a favoured core (the highest `acpi_cppc/highest_perf`, else the highest `cpuinfo_max_freq`) with its sibling outside every thread's mask; against the scanner pinned to a worker core's sibling (co-pinned); against unpinned. This takes over the pinned scanner-profile variant (QUEUED 7147262673 B).
   - **Memory:** at k\* and 20, a memory dose at A7's end (each Open block) of 2 MiB of the worker's own scratch, streamed against chased, against none. The dose is the binary's `dose` switch: `src/dose.rs` is inert on this path. 2 MiB is about one core's share of the i9's 20 MiB L3, at equal bytes for both kinds. In the container's preflight, 8 MiB chased swamped the block: it decoded 4.5–6× slower.
   - **Oracles** (§5's decision): arms B (supplied codes) and C (the scanner's output replayed) at the knee, run from the four-arm binary (#835 at its tagged commit), reported only as ratios against that binary's own A. The two binaries' A walls at the knee are compared, and any gap is reported as a layout effect.
4. **Validation** (§2 step 4): about 20% of the time on conditions excluded from every fit.
   - A combined +10% on A3 and A7 together, at k\* and k\* + 2.
   - The window permutation: the corpus's 12 streams (one a window) in one fixed permutation drawn from the seed, at k\* and 20. The 12 streams are split at their byte-aligned stream headers and the decode's CRCs check the result.

   A model revised after seeing them needs fresh validation.

## Stage 2b: confirmation

The screen's paired variance fixes the confirmation's sample sizes once (C, M). For each coordinate the screen marks influential (its coefficient's 95% interval excludes zero, or its point estimate exceeds 0.10 in magnitude), the pairs needed for a half-width of 0.10 on its coefficient: n = ⌈(t·s / (0.10 · log 1.2))²⌉, s the screen's paired log-ratio SD. They are capped by the time budget, the largest coefficients first. 2b is a recipe of its own, registered from 2a's data before its own. No interval is extended until significant.

## One binary

All of stage 2a's arms run one binary: stage 1's library (src unchanged from 94cb742d) with one more harness switch, the window, in examples/character.rs. Stage 2a carries its own baselines at every anchor. The stage-1 and stage-2 baselines at k\* are compared, and any gap is reported as a layout or day effect, not a treatment's. The oracle arms run from the four-arm binary, as decided.

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

## Open for the review

- the dose for the memory contrast (2 MiB at A7's end);
- the oracle arms' wiring to the four-arm binary (its `ab` arms at its tagged commit), to be built at the freeze;
- the step rule's 5%;
- the placement's mask construction on the i9's topology;
- whether the turbo factor is worth a password prompt (the run would then not be unattended).
