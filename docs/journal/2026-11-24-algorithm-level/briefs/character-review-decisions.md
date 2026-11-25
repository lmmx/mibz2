# The character experiment: the Coordinator's decisions on the three design reviews

The three answers to `character-design-review.md`:
- ChatGPT Pro: `../external/chatgpt-design-review.md`, relayed by Louis.
- Neon: `../reviews/neon-design-review.md`, a verbatim copy of `research/neon-movement/DESIGN-REVIEW.md` at 2a541d19.
- MONODROMY: record 128, `docs/journal/monodromy/2026-11-24-character-design-review.md` at e09ae92c on `claude/sharp-meitner-cx0v78`.

The reviews were one round, as agreed. The Coordinator decides here (RULES C6: experiment design is delegated). Where the reviewers agree, the item is adopted as stated. Where they differ, the decision and the reason are given. The scanner profile (journal §57) supplies the facts the reviewers didn't have.

Each item names its source: **C** is ChatGPT Pro, **N** Neon, **M** MONODROMY.

## 0. Where all three agree

- **The 2^(9−4) fractional factorial goes.** Two-factor interactions alias each other, and 32 rows can't resolve 46 coefficients. Across the kink of a max, the main effects are time shares or zero. Pacing several serial worker stages at once also dilutes memory pressure. (C, N, M)
- **Exact product-form MVA doesn't apply** to the real pipeline: blocking admission, in-order resequencing, non-exponential service. The pipeline gets bounds by resource; MVA only on a documented surrogate, with its error against the simulation reported. The population is the admission window, not k, and not W when the byte gate binds. (C, N, M)
- **One binary for every arm,** with every treatment a runtime switch, and a fixed environment and argument layout. (C, N, M)
- **The CPU each thread runs on, and the achieved frequency, are logged.** (C, N, M)
- **The admission wait is split by cause** (window or byte gate), **and tagged with the head-of-line block.** (C, N, M)
- **The window test is unconditional at the knee:** W against the byte threshold, crossed. (C, N, M)
- **SMT and turbo are crossed, not run separately.** (C, N, M)
- **Stage 2's anchors come from stage 1,** and pacing at k = 1 is dropped: with one worker each coefficient is a time share the timers already give. (C, N, M)

## 1. Stage 1 (#841): added before its ready tag

These change what stage 1 records. They are cheap now, and impossible to add after the run.

1. **Queue transitions carry an under-lock sequence number.** Ready is marked after the lock is released, Taken under it, and Enqueued after the send, so a sort by timestamp can produce an impossible history. (C)
2. **Lock waits are bracketed separately** from condition-variable waits: `Shared::lock` about 5 times a block, `Jobs::lock` twice. (N)
3. **The admission wait is split by cause:** `ahead >= W` against `ready_bytes >= threshold && ahead >= floor`. Each wait records the head-of-line block (`st.taken`) and its age. So do the scanner's in-progress block and the reader's current buffer, which lie outside issued − taken. (C, N, M)
4. **The reader's time is split** into its own work (advance and sink) and waiting for the next block. (C)
5. **Placement and frequency:**
   - each thread's CPU (`sched_getcpu` or `rdtscp`'s `TSC_AUX`) at block boundaries;
   - the achieved frequency (cycles over reference cycles), package temperature, power and throttle status, joined to every observation, at all 20 counts. (C, N, M)
6. **One fresh process, one decode.** No observation aggregates several decodes in one process: reuse added 101 MB of peak RSS in the four-arm run and removed cold-heap faults after the first decode. (N, C)
7. **Launch hygiene:**
   - `env -i` with a fixed minimal environment, and fixed-width arguments (no layout change at 9 → 10);
   - ASLR left on;
   - the governor recorded;
   - the corpus resident in the page cache, with `majflt == 0` asserted;
   - a 45–60 s thermal pre-warm before round 0, discarded;
   - each observation's position and the previous k recorded as covariates. (C, N, M)
8. **The same binary as stage 2.** The pacing, dose and oracle switches must be runtime switches before stage 1 is built. `src/dose.rs`'s compile-time `option_env!` is the known case. (N)
9. **Startup, steady state and drain are marked** per decode, with minor faults and system time. The Pathfinder's gap decomposition already has startup and drain; it adds the steady-state boundary. (N)
10. **Rounds:**
    - **Decision:** 16 randomised rounds at k = 7–16; 12 elsewhere.
    - **Reasons:** Neon asked for 16–20 at 8–12. MONODROMY and ChatGPT keep 12, with confirmation sized after screening. The profile shows the scanner's 95% half-width at 2.3–4.6% across k = 10–17, so the noisy band is wider than 8–12.
    - **Cost:** 4 extra rounds × 10 counts ≈ 2 minutes.
11. **PMU:**
    - whole-decoder counters at {1, 8, 10, 16, 20}, adding 16 for the step the profile found;
    - an event manifest for this CPU and kernel;
    - at most 4 general-purpose events a group;
    - observations with running/enabled below 0.99 rejected;
    - counters gated to the stopwatch region.

    Per-activity PMU and the offcore-latency pass are deferred to stage 2's targeted contrasts. (C, N)
12. **Brief §3's corpus count is corrected (N, proved from the four-arm rows):** 1724 blocks are 431 distinct × 4. 932 take Open (233 distinct × 4) and 792 do not (198 distinct × 4). The scanner profile adds the scale: exactly 792 blocks are under 10 kbit and hold 0.1% of the bits, so the non-Open blocks are almost certainly the tiny ones. They are 46% of the blocks but a small share of the work. The structural features cover all 431 distinct blocks, and per-block fits cluster by distinct block.

## 2. Stage 2: the replacement design, frozen after stage 1

**Anchors, by rule from stage 1:**
- k\* is the smallest count at which the scanner's on-CPU share passes 0.9 (M). The profile put it between 7 and 8.
- The anchors are k\* − 2, k\*, k\* + 2, and the count where the scanner's work steps up (16 in the profile), else 20 (N, M).
- 1 and 20 stay as PMU and endpoint anchors only. 10 stays as the bridge to the four-arm test (C).

**Pacing:**
- **The loop:** a fixed-iteration dependent-ALU loop calibrated to unperturbed core cycles, not a wall-clock deadline, which self-shortens under turbo droop, SMT and preemption. Both wall and thread-CPU time are logged per paced block (N).
- **What it measures:** it scales occupancy, not memory demand, and the write-up says so (C).

**The steps, in order:**
1. **Screen:** one coordinate at a time at +20%, at the anchors.
   - Coordinates under 3% of their thread's stage-1 time are dropped. A10 is the exception, kept if the reader's waits exceed 2% (N).
   - A6a/b/c are paced as one A6, split only if a part exceeds 5% of worker time (M; N's 20% threshold is looser).
   - Off-diagonal responses (peers' on-CPU time, achieved bandwidth) are reported (N).
2. **Surface:** a full 3×3 on (scanner dose, all-worker dose) ∈ {0, 10, 20}² at k\* (N, M).
   - **Decision:** this, not ChatGPT's 2³ over three coordinates. The knee's roundness is the parameter the simulation and the fluid model most need, and the 3×3 measures it unaliased.
   - Baseline finite-difference sensitivity and factorial marginal effects are reported as separate estimands, never mixed across counts (C).
3. **Mechanism contrasts:**
   - **Gates:** W ∈ {4k, 8k} × byte threshold ∈ {12, 64} MiB at k\*, with the same k + 2 floor. Each contrast verifies which gate actually changed (C, N, M).
   - **Placement and clock:** a 2×2 of (distinct cores, sibling pairs) × (turbo free, turbo off) at k = 4 (N). Plus, at k = 10 and 16, the scanner pinned to a favoured core with its sibling idle, against co-pinned with a worker's sibling, against unpinned. This takes over the pinned scanner-profile variant (`../leads/course-opportunities.md` §3).
   - **Order:** one permutation of the 12 independent windows (`w00..w11`), held out for the simulation to predict (N, M).
   - **Memory:** `src/dose.rs` stream against chase, switched at runtime, at k\* and 20 (N). The bandwidth-against-L3 question stays an unresolved "memory contention" term unless this separates them (C). MONODROMY's two antagonists are deferred.
   - **Oracles:** arms B and C at the knee, so that both sides of the kink are measured (M, N).
4. **Validation:** about 20% of the time is reserved for conditions excluded from fitting (C). They are a combined +10% (A3 with A7) and the window permutation. A model revised after seeing them needs fresh validation.

**Confirmation:** sample sizes are fixed once from the screen's paired variance; no interval is extended until it is significant (C, M).

## 3. Cut

- the 2^(9−4) factorials;
- pacing at k = 1;
- +10% follow-ups outside the surface;
- per-activity rooflines (a whole-decoder traffic constraint stays);
- THP experiments and duplicate lifecycle checks, unless faults or startup make them live;
- dense and high-expansion corpora in stage 2 (deferred to an out-of-corpus stage 3);
- Coz (later, to test the leading optimisation);
- per-observation RSS polling in stage 2 (peak once per count);
- the second in-process decode (measured only as its own condition, never averaged).

(C, N, M)

## 4. What the reviewers disagreed on, and what was decided

| question | positions | decision |
|---|---|---|
| rounds in the noisy band | N: 16–20; M: σ-sized, at most 12; C: 12, confirmation sized after screening | 16 at k = 7–16 (§1.10): the band is wider than 8–12, and the cost is about 2 minutes |
| the interaction design | C: 2³ over three coordinates; N and M: 3×3 (scanner, worker) | 3×3: it measures the knee's shape, the quantity the models need |
| bandwidth against L3 | M: two antagonists and a small-block corpus; N: byte thresholds {2, 12, 64} MiB and dose stream/chase; C: an unresolved term unless identified | dose stream/chase only; the rest deferred until stage 1's traffic shows the question is live |
| where SMT is tested | N: 2×2 at k = 4; M: at fixed k; course lead: the scanner at 10 and 16 | both: the clean 2×2 at 4, and the scanner placement where the profile saw the step |
| ASLR | N: on, plus one `setarch -R` check at 10; M: on; C: on, recorded | on, recorded; the single `setarch -R` check at k = 10 is kept (N) because it is cheap |

## 5. Later decisions

- **The oracle arms B and C run from the four-arm binary, not ported into #841** (the Pathfinder's question, 25 November 2026).
  - **Why:** porting means bringing #821's change to Open's emission path, and #835's scan-oracle replay, into the recipe as runtime switches. That is a large source review for no gain in identification. The oracles answer a different question from pacing, and the four-arm design pairs each arm with its own A in one binary.
  - **The rule:** B and C are reported only as ratios against the four-arm binary's own A, never against #841's baseline.
  - **The check:** the two binaries' A walls are compared at the knee. A gap beyond their intervals is reported as a layout effect, and the oracle ratios still stand, being within-binary.
- **The memory dose (stream/chase) goes behind the pacing hook at a tag's end,** not through `src/dose.rs`, whose pad fires only on blocks the Cells kernel admits. On the OpenCells path LazyCells refuses every block first, so `dose.rs` would do nothing there.
- **Stage 2's open choices (the Pathfinder's draft, `bench/character-stage2` at b2169a0d), agreed on 25 November:**
  1. **Rules on stage 1's bundle:**
     - k\* is the first count where the median scanner on-CPU share passes 0.9;
     - the step is the count above k\* + 2 with the largest rise in D_scan, if it is at least 5%, else 20;
     - the pacing dose is δ × the per-visit service in core cycles, calibrated to loop iterations at the run's start.
  2. **The window switch** goes in the harness only (`examples/character.rs`; src unchanged), and stage 2a carries its own baselines, compared with stage 1's as a layout check.
  3. **Placement at k = 4** is set by the process CPU mask. The turbo factor runs only if `no_turbo` is writable. If it isn't, placement still runs, with the achieved frequency per role as a covariate, and the registration says the SMT and turbo factors are then not separated.
  4. **The memory dose:** stream against chase, 8 MiB at A7's end per Open block, at k\* and 20.
  5. **Confirmation is stage 2b,** with n fixed from 2a's paired variance and registered before its own data.
- **Stage 2 runs on stage 1's master pin (9621b7ad)**, not a later master. The scanner plumbing fix (QUEUED 7147862124) changes the scanner's demand, and must not enter mid-study.
