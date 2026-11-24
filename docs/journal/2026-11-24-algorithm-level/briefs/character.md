# Brief: the decoder's performance character across 1–20 workers, as a network

*The recipe brief merges three independent answers to `briefs/decomposition.md`:*
- *MONODROMY's record 125, with records 123 and 124;*
- *Neon's `DECOMPOSITION.md` on `research/neon-movement` at 00c08750;*
- *ChatGPT Pro's answer, `external/chatgpt-decomposition.md`.*

*Comparison and decisions: journal §51. Assigned to the Pathfinder; it runs after the scanner profile (#838) and before the gated parallel scan.*

## 0. What the three answers agree on, and what they correct

- **The doorman/assemblers split is a valid coarse topology, but not a decomposition.** Model the decoder as a **network of activities, buffers and shared resources**: what transforms each block; when work can run and when credits are released; and what concurrent work competes for.
- **The control-coefficient summation theorem is not a completeness test.** It holds only for homogeneous, differentiable throughput. At a bottleneck switch (J = min(v₁, v₂)) the sum is 2 for slowdowns and 0 for speedups, with nothing missing. It checks time, not resources. **Completeness is checked by accounting identities instead** (§4).
- **NUMA is dropped.** Record the topology once.
- **Block heterogeneity enters as per-block covariates.**
- **A serial tail stage exists:** ordered consumption and output.
- **SMT sharing does not begin cleanly at k > 10.** The decoder runs k + 3 threads (scanner, feeder, consumer), so it begins around k = 8–9.

## 1. Freeze the experimental contract first

**Requested threads are not actual workers** (proved from code by ChatGPT Pro and Neon). `StreamDecoder::new` can change the worker count, kernel, window and topology. On the i9 it can turn requests for 10 or more threads into 7 lazy/Open workers, and `Plan::Workers(k)` changes the kernel with k. So the study has two separately labelled parts:

- **Mechanism (primary):** one fixed implementation with k = 1…20 *actual* workers. Use the existing `mibz2-workers-lazy` path: production OpenCells admission and fallback, window W = 4k. **Not** `mibz2-workers-open`, which forces Open and so changes admission.
- **Public API (secondary, unperturbed):** what a caller requesting 1–20 threads gets under the shipped policy. One cheap sweep.

**Fix and record:**
- the corpus, its order and its residency;
- decoder and buffer lifetime, the output sink, and what the clock includes;
- the build, kernel selection and allocator (unchanged; no allocator changes);
- frequency and power policy (observed, or held fixed);
- placement. The primary condition is unpinned on all 20 logical CPUs; targeted pinned runs follow in §6.

Report steady state, start-up and drain separately. W = 4k changes both workers and in-flight work, so the model carries both.

## 2. Activities, buffers and resources

**Activities.** The tags are used everywhere.

| tag | activity | where | note |
|---|---|---|---|
| A0 | input feeder reads and copies | feeder | input CPU work sits on both feeder and scanner |
| A1 | scanner buffer maintenance and input wait | scanner | |
| A2 | header, bitmap, selectors, tables | scanner | |
| A3 | Huffman symbol decode | scanner | count bits and symbols |
| A4 | dispatch, recycling, pool | scanner/workers | active time apart from blocking |
| A5 | MTF/RLE2 and run-record formation | workers | **one fused activity** (`runs::front` in `decode_owned`); not two |
| A6a | Open first pass: classification, admission | workers | |
| A6b | Open setup, induction, dictionary | workers | |
| A6c | Open retained-edge wiring | workers | scattered RMW on `edges` |
| A7 | emission | workers | route and emitter tags |
| A8 | non-Open inversion (column route, fallback) | workers | rejected Open work is charged before fallback |
| A9 | RLE1/CRC finalisation | workers | **keep the tiled implementation**; don't split it to time it |
| A10 | ordered consumption and output | consumer | a separate agent; its own demand |

**The graph is routed.** Each block takes the run or the column route, and Open is admitted, skipped, or attempted and then falls back. Log every block's route.

**Buffers and feedback.**
- Admission is `issued − taken`, and `taken` advances only when the ordered reader takes the next block in order. So queued, executing and completed-out-of-order blocks all hold admission credit.
- There is also a ready-output threshold of 12 MiB, with a floor of k + 2. A slow early block can therefore stall the scanner through the consumer.
- **A scanner waiting for admission does not by itself mean the workers lack capacity.**

**Shared resources.** These are observed, and a few are perturbed (§6):
- R1, DRAM traffic and bandwidth;
- R2, cache capacity and locality: L2 and the 20 MiB L3. Neon predicts an L3 capacity cliff between k = 10 and k = 16;
- R3, latency and memory-level parallelism;
- R4, core and SMT sharing;
- R5, allocation and virtual memory (zeroing, faults, retained buffers, TLB);
- R6, synchronisation (locks, wake-ups, migrations);
- R7, frequency and power (turbo drop with active cores).

## 3. Stage 1: the baseline, at every count (240 observations)

- **Counts and rounds:** k = 1…20, **12 randomised complete rounds**. Each round has one observation at every count, in randomised order, never a rising sweep. Where a single decode takes only a fraction of a second, aggregate several complete decodes into one observation, keeping the creation and teardown contract.
- **Per-thread buffered events:**
  - block identity, route, stage start and end, queue transitions;
  - input waiting, admission waiting, worker starvation and ordered-output waiting;
  - queued, executing and completed-unconsumed occupancy, as integrals and high-water marks;
  - bits, symbols, BWT rows and output bytes.

  Keep **separate ledgers for per-thread wall states and on-CPU time.**
- **Every observation also records:**
  - wall time and CPU time;
  - correctness;
  - actual workers, route and window;
  - minor and major faults, voluntary and involuntary context switches, migrations;
  - peak RSS and **time-weighted mean RSS** (byte-seconds).
- **Instrumentation cost check:** 6 paired on/off checks at k = 1, 10 and 20. If throughput or queues shift by more than about 0.5–1%, reduce the tracing.
- **PMU, at the six anchors (§5):** two small groups, 3 repeats each (36 observations), recording running/enabled coverage:
  - cycles, instructions, ref-cycles, branches and misses;
  - LLC, L2 and dTLB misses, memory-stall cycles.

  Also read uncore IMC CAS read/write counts where available. LLC misses × 64 is not DRAM traffic, and uncore bytes are not attributable per stage.
- **Structural features** (n, r, h₁, m₁₆, width histogram) are computed outside the timed runs, joined by block identity, and grouped by distinct block. The corpus has 1724 blocks: 431 distinct blocks repeated four times. 932 take Open (233 distinct × 4) and 792 do not (198 distinct × 4; corrected after Neon's design review). Fits cluster by distinct block, never by the 1724 rows.

## 4. Completeness: accounting identities, not a coefficient sum

All of these must hold:
- enqueues − dequeues = occupancy;
- issued − taken = outstanding admitted work;
- every block's successful, rejected and fallback work is charged;
- the integral of occupancy = the sum of residence intervals;
- each thread's timeline is partitioned with no gaps and no double counting.

The coefficient sum is reported only as a conditional diagnostic. It is never normalised to 1.

## 5. Stage 2: interventions at six anchors

**Anchors:** k ∈ {1, 6, 8, 10, 12, 20}.
- 8 lies inside the SMT onset (k + 3 threads), 10 is the four-arm point, and 12 is past it.
- Add 14 only if the baseline shows a transition the anchors miss. That is where Neon predicts the L3 cliff.

**Coordinates (p = 9), disjoint scopes:** A2, A3, A5, A6a, A6b, A6c, A7, A8 and A9. Add A0, A1, A4 or A10 only if the baseline shows material demand or control there.

**The intervention** is a calibrated **boundary-pacing busy loop**:
- it touches no memory;
- its budget is a fixed fraction of that activity's *unperturbed* per-block time, measured in stage 1 and never recomputed from slowed durations;
- the achieved delay and where it falls relative to resource release are recorded.

The result is the response to that intervention, not literally "this code made slower".

**Dose** — decided in journal §51:
- **+20% for the primary pass.** The three answers proposed +10% (ChatGPT Pro), 20% (MONODROMY) and 25% with pinning (Neon). At a paired log-ratio SD of 3% over 12 rounds, the coefficient's 95% half-width is ±0.20 at +10% and ±0.105 at +20% (ChatGPT Pro's formula, recomputed). At 5% SD it is ±0.33 against ±0.17.
- **+10% follow-ups** for the influential coordinates and at the transition counts. Comparing 10% and 20% tests local linearity; near a switch, responses are one-sided.
- **One combined test,** A3 and A7 together at k = 8 and 10. It is held out, for the model to predict.

**Estimate:** Ĉᵢ = mean(log T_{i,δ} − log T₀) / log(1 + δ), against achieved exposure where the dose drifts. The **full response matrix** is recorded for every intervention:
- throughput;
- other activities' service times;
- scanner waiting and worker starvation;
- ready backlog;
- DRAM traffic and memory occupancy.

**Cost:** 12 × (20 + 6·9) = 888 observations (240 of them the shared baseline), plus 36 PMU, plus the follow-ups. Inference bootstraps over complete rounds. Small effects are reported as intervals, never as zeros.

## 6. Targeted checks, not crossed with the grid

Each is run at a few anchors only:
- the window size W, or the 12 MiB ready threshold if that binds;
- pinned against unpinned, and physical cores against SMT siblings at fixed k (R4, separated from R1);
- turbo locked (R7) and transparent huge pages (R5);
- a second in-process decode (R5);
- a fresh process against a warm one, and the real output sink;
- other corpus classes: dense/fallback and high-expansion.

## 7. The model, and how it is judged

1. **Primary:** a **discrete-event simulation** of the network in §2. It covers the finite admission window, routed workers, out-of-order completion, the ordered consumer and its feedback, and the ready-byte cap. It is driven by **complete per-block service vectors in original order**; never resample stages independently.
2. **Constraints:** **a roofline per activity.** Compute/f(k), plus dependent reads × ℓ(k), plus bytes/β(k), from the PMU and IMC data. It is used to reject infeasible predictions.
3. **Contention:** a few shared terms (working set, core sharing, traffic pressure, frequency), added only when residuals demand them.
4. **Fit and held-out counts, fixed before the data:**
   - fit: {1, 2, 4, 6, 8, 10, 12, 16, 20};
   - held out: {3, 5, 7, 9, 11, 13, 14, 15, 17, 18, 19};
   - also held out: the +10% follow-ups, the combined test, and the four-arm arms B, C and D.
5. **The acceptance target is conjectured, and judged once:** at most 3% median and at most 5% maximum relative error on held-out mean runtime, with wait fractions within 5 points. A model that fits throughput but gets the queues wrong fails.
6. **k\*:** estimated non-parametrically, as min{k : the median starvation gap Δ_k > 0}, from per-round scanner-busy minus mean worker-busy time, with a paired bootstrap interval. Also reported: a **near-optimal set**, the counts within 2% of the best. A parametric fit (USL), if any, is made on k = 1–9 only.
7. **Reasoned rather than measured:**
   - J ≤ min(1/a, k/b) and bottleneck migration;
   - the max-plus critical-path form;
   - J ≤ B/q (bandwidth) and J ≤ M/(L·m) (memory-level parallelism);
   - interaction terms are not identified by one-at-a-time experiments, so the constrained network form is the assumption under test.
8. **Coz, or controlled virtual speedups,** for one or two leading targets afterwards. Progress is ordered output bytes. Each finding is validated by a real small change.

## 7a. Amendment (journal §52, from Jain's methodology): operational laws, MVA and a fractional factorial

- §4's identities are the **operational laws**: Little's law, utilisation and forced flow. Check them explicitly per activity and buffer.
- Add **mean-value analysis** of the closed network (window W = 4k) beside the discrete-event simulation, as the analytic model and a check on the simulation.
- At **k = 8 and 10**, replace one-factor-at-a-time with a **2^(9−4) resolution-IV fractional factorial** (32 runs) over the nine activities, at +20%. It estimates main effects clear of two-factor interactions, which one-factor-at-a-time cannot see.
  - The other anchors keep one-factor-at-a-time.
  - If the budget is too tight, use a 16-run resolution-III design plus a fold-over, and say so in the registration.

## 7b. Amendment (journal §53, from the performance-analysis courses)

- Report **device demands** Dᵢ = Vᵢ·Sᵢ per activity at each k. With them report the closed network's **asymptotic bounds**, X ≤ min(1/D_max, N/(D + Z)), the knee N\*, and **balanced-job bounds** (Jain ch. 33–34).
- **Price each candidate change by modification analysis** (Harchol-Balter ch. 7): change Dᵢ, then recompute the bounds and MVA, before building anything.
- Treat blocks on the Open route and on the column or fallback route as **job classes** (classed networks, ch. 18).
- **Analysis:** use multiplicative (log) models and report allocation of variation for every factorial (Jain ch. 18–19). Give a **fluid approximation** of scanner rate against worker demand as a closed-form check on k\*. Use **sample-path** statements (Little's law on the realised trace), not distributional assumptions, for the fixed block sequence.

- Report the **second moment of per-block service time** (or C²_S) per activity beside the mean demands (journal §54). Delay depends on E[S²], not only on E[S] (M/G/1). Long-tail blocks are the mechanism behind the admission stalls.

## 8. Process

- The Pathfinder splits this into recipes: stage 1 with the public-API sweep, then stage 2 with the targeted checks. Each is a C1a diagnostic with its rule posted on #145 before the data.
- Results land as TSV in `math-targets/data/`, with their scripts.
- The Coordinator reviews, merges and tags each recipe.
- The gated parallel scan's registration (`briefs/scanner.md` part 2) uses stage 1's k\*.
