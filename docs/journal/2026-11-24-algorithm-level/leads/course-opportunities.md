# The performance courses, mined against the scanner profile

Louis, on the scanner profile (journal §57): "that is awesome and also suggests we need to mine these courses for opportunities then."

The profile showed that a two-station model (one serial scanner feeding k workers) predicts the decoder's knee from four worker counts. That makes the course material usable directly: its laws and models now have measured inputs. This file goes through the three courses (`../methodology/performance-courses.md`, the sources in `../methodology/sources/`) and keeps each tool that the measurements say has something to give, with a predicted size, the measurement that tests it, and its cost. Tools the measurements rule out are listed too, with why.

All figures are from `scan-profile/run-20261124T223748Z` (i9-10900K, 10 cores, 20 threads, one corpus, unpinned) unless marked as a prediction or an estimate.

## Corrections after ChatGPT Pro's independent reading (25 November)

ChatGPT Pro answered the same question independently, then compared (`../external/chatgpt-course-mining.md`). Its corrections were checked against the code and the arithmetic, and are accepted. The struck or rewritten lines below follow them.

1. **The public default doesn't run 20 workers.** On this i9, unpinned, a request for 10 or more threads builds 7 lazy workers with 28 slots (`StreamDecoder::new`, `trimmed_here`, checked at master 1c2ce5c0). That cap came from the recalibrations, which found extra workers bought nothing. The real default question is **7 against 8 workers**: 0.2069 s against 0.1911 s in the profile (7.6% less wall, 0.9% more CPU, 2.1% more own memory), to be confirmed through the public API, since the profile used explicit workers and a reused output buffer. The 20-against-8 comparison stands as a measurement of explicit workers only.
2. **The machine bound was mis-stated.** The arithmetic (about 0.109 s) reproduces, but it is the best wall's 0.568, not "the best wall is 0.57 of the bound". More importantly, it rests on five unmeasured assumptions (comparable frequency, busy time as transferable demand, known sibling overlap, worker-worker sharing transferring to scanner-worker, no new costs). It is a **conditional capacity scenario for parallel scanning**, not a balanced-job bound and not "43% lost". ChatGPT's own idealised allocations: two scanners and eight workers about 0.172 s (−10%); perfectly flexible sharing about 0.156 s (−18%).
3. **The reader isn't excluded.** "If the reader limited, the window would fill" is false for a finite buffered run. A reader 1% slower than the scanner accumulates about 18 blocks of lead, which fits under both gates. Argued, not proved: substantial reader backpressure is unlikely; a near co-bottleneck reader isn't excluded. Stage 1 now measures the reader's work and wait.
4. **Bottleneck migration.** The scanner's clock does not pass 1:1 into the wall: at 8 workers a 5% cut in Huffman demand already reaches the workers' limit. Restoring 1.41 ns a symbol predicts 3.96% at 8 workers, not 7%. Scanner and worker improvements must be priced together, with the worker count retuned after either. Cutting both by 20% at 8 workers predicts 0.162 s (−15.7%). A 20% worker cut lets 6 workers match today's best.
5. **Variability:** the tiny blocks make the whole mixture's C² about 0.98, not 0.07. And these are scanner-side figures: worker service per block was not measured, so they can't rule out worker-size-based policies. The size-based-scheduling exclusion below is withdrawn to "not supported yet".
6. **Little's law:** 8–9 blocks used service demand, not residence time, and its boundary doesn't match issued − taken. W = 4k throughout the sweep, so memory growth can't be attributed to workers rather than slots. The window question needs a fixed-k window experiment (8 workers, windows 16 against 32).
7. **Smaller corrections:** the decoder is a finite pipeline with feedback-controlled admission, not a closed system in which blocks circulate (a credit-token surrogate would be). Phase-type methods also cover low variability, so they are deferred, not ruled out. JSQ-d is not replication. The fit's 32 ms constant is not a fixed overhead, but its cause is unidentified.

8. **From Neon's independent reading** (`../reviews/neon-course-mining.md`, dd6cc26d), checked against the code:
   - **Two scanner plumbing costs this file missed, about 8% of the scanner's demand, verified in the code:**
     - `ChannelSource::view` (`src/parallel.rs`) drains and re-copies the input window whenever a 2 MiB look-ahead straddles a 4 MiB chunk: 11 blocks, about 10 ms at 8 workers, 96% of the scanner's "remainder";
     - `block.rs` truncates each block's symbol buffer to n, so the next block that takes it from the pool zero-fills it back to max_out: about 5 ms, mostly after the 792 tiny blocks (output buffers already avoid this with `POOLED_FROM`; symbol buffers don't).
     
     Removing both cuts about 15 ms. With bottleneck migration, that is about 6% at 8 workers, and 8–9% where the scanner binds (9–10 workers). Routed for implementation (journal §62).
   - **The machine figure:** at 20 workers only about 9.25 threads are busy on average, so the 0.745 sharing factor is not a 20-thread figure. The physical 10-core pooled figure is 1.628 / 10 = 0.163 s (a 15% gap). With realistic SMT gain for memory-bound work (1.15–1.30×), 0.125–0.142 s. All conditional, as correction 2 says.
   - **The scanner's slowdown is bimodal, not a step:** from 10 workers, each run's scanner lands in a fast mode (0.189–0.193 s) or a slow one (0.223–0.242 s). The slow share rises with k and passes half between 15 and 16. Stage 1's per-thread CPU log stratifies it.
   - **Favoured-core pinning at 8 workers likely recovers 0–2%, not 7%:** Comet Lake sets its all-core turbo package-wide. The step at 16+ is the real target. Stage 2's placement × turbo contrast decides it.
   - **The head-of-line waits:** the small slot waits at 8–12 workers sit exactly 4k blocks behind dense blocks, and the real blocks' sizes are autocorrelated along the stream (lag-1 r = 0.74).

**What this does not abandon.** The 7-worker cap is a symptom of the scanner limit, not a reason to stop: the cap exists because workers past about 7 bought nothing, and raising the scanner's ceiling is exactly what would let it lift. Every ceiling-raising lead stands: a cheaper or parallel scanner, worker mathematics (which lowers the knee and buys the same speed with fewer cores), and joint improvements. What falls is one claimed cheap win (20 → 8) and the over-precise sizes.


## The measured inputs

| quantity | value | where from |
|---|---|---|
| scanner work a decode, b | 0.186 s at k ≤ 8; 0.229 s at k ≥ 16 | timed decodes |
| workers' total work a decode, A | 1.43 s at k = 8; 1.92 s at k = 20 (thread-seconds) | timed decodes |
| service times a block (counts 1–4) | scanner s = 106.6 µs, worker w = 798.3 µs | report |
| knee | predicted 7.49, measured 7.67 | report |
| best wall | 0.191 s at k = 8 | untimed decodes |
| efficiency against the busiest server | 0.945–0.998 | §57 |
| blocks | 1724, of which 792 are tiny (under 10 kbit, 0.1% of the bits) | per-block records |
| the 932 real blocks' variability | CV 0.25–0.29 in symbols, read and decode time (C² ≈ 0.07) | per-block records |
| what the public API runs | ~~20 workers~~ **corrected:** asked for 10 or more threads, unpinned on this i9, `StreamDecoder::new` builds 7 lazy workers with a 28-slot window (`trimmed_here`; exactly 10 may build the service core instead). Python asks for `available_parallelism()` = 20 threads | `src/parallel.rs` `StreamDecoder::new`, `src/python.rs` |

## Opportunities, largest first

### 1. The machine bound: the gap is in the topology, not the discipline (Jain ch. 33–34; Harchol-Balter ch. 6–7, balanced job bounds)

**The tool.** The bottleneck law bounds throughput by the busiest server's demand. The balanced job bound adds the other side: total demand over total capacity. The decoder has been judged only against the first.

**Applied.** Against the busiest server (the scanner), efficiency is 0.97: almost nothing to recover by scheduling. Against the whole machine it is not. The workers' total work rises from 1.43 s at k = 8 to 1.92 s at k = 20 because hyperthreads share cores. If that rise is all SMT sharing, a thread with a busy sibling runs at about 0.745 of a lone thread, so a core running two threads delivers about 1.49 lone-thread equivalents, and the machine about 14.9. Total demand is about 1.43 + 0.19 = 1.62 s, so the pooled-work scenario is about 1.62 / 14.9 ≈ 0.109 s, which is 0.568 of the best wall. **Conjectured (corrected, see correction 2):** a conditional capacity scenario, resting on five unmeasured assumptions, not a bound.

**Is the scanner really the limit above the knee?** The design review warns that a busy-time crossing alone doesn't show it: a slow consumer (the in-order reader) could control throughput instead. The profile has the evidence it asks for, in aggregate: at k = 10 the scanner works for 0.945 of the wall, its waits for admission (0.014) and input (0.007) are small, and 1.67 workers are idle on average. ~~If the reader were the limit, the window would fill and the scanner would wait on admission; it doesn't.~~ (Corrected, see correction 3: this disfavours substantial reader backpressure but doesn't exclude a near co-bottleneck reader.) The per-interval version (workers empty *while* the scanner is producing) comes from stage 1's ordered trace.

**What it means.** This is the size of the prize for a parallel or cheaper scanner on this machine, and a cross-check on the four-arm test: arm C (the scanner's output supplied) ran at 0.806 of the shipped decoder at 10 workers, inside this bound.

**Caveats.** The 0.745 factor also absorbs memory-bandwidth and cache contention; it is an estimate from busy times, not a measured SMT factor.

**Test.** Stage 1 (#841) measures per-visit demands; a run with the scanner's output supplied at k = 10–20 (arm C extended) measures the achievable fraction directly.

### 2. Right-size the worker count (Harchol-Balter ch. 14–15: capacity provisioning, square-root staffing; ch. 27 and the AutoScale paper: idle capacity is not free)

**The tool.** Provision servers for the offered load: R = w/s servers' worth here, plus a margin for variability (square-root staffing gives R + β√R; with C² ≈ 0.07 the margin is small).

**Applied (measured, explicit workers; see correction 1 for what the public API runs).** R = 7.5. With 20 explicit workers, Against k = 8, k = 20 takes 0.230 s against 0.191 s (20% longer), 2.14 against 1.61 s of user CPU (+34%) and 74.8 against 54.1 MB of the decode's own peak memory (+38%). From k = 9 the extra workers add CPU and memory and no speed; from k = 16 they slow the scanner (opportunity 3).

**What it needs.** R depends on the data, so a fixed cap is a stopgap. Both s and w are linear in a block's symbols with a small per-block term (the scan fit gives 1.5 ns a symbol), so R can be estimated online from the first few blocks' free counters and the workers' busy times: a renewal-reward estimate (Harchol-Balter ch. 23), time-average rate = E[work a block] / E[time a block].

**Cost.** Small code; it changes a production default, so it needs an adoption registration. Validation needs more than one corpus (opportunity 6).

### 3. Put the bottleneck on the fastest server (Harchol-Balter ch. 1 and 14, "one fast rabbit or many turtles"; the SIGMETRICS 2009 power-allocation paper)

**The tool.** With unequal servers, the bottleneck stage belongs on the fastest one, and capacity at a non-bottleneck stage is worth nothing.

**Applied.** On the plateau the wall is the scanner's work, so the scanner's clock passes 1:1 into the wall. Two measured signs that it isn't getting the best core:
- its decode costs 1.41 ns a symbol at k = 1 and 1.51 ns at k ≥ 3 (7% slower);
- at k ≥ 16 its work rises 23%, with the wall following.

The i9-10900K's all-core turbo is lower than its single-core and favoured-core turbo (Turbo Boost Max 3.0 picks two favoured cores). **Hypothesis:** pinning the scanner to a favoured core, with its SMT sibling kept free, recovers most of both: about 4% on the plateau at 8 workers (corrected from 7% for bottleneck migration, correction 4) and the 20% step at k ≥ 16.

**Test.** A pinned variant of the scanner profile at k ∈ {8, 10, 16, 20}: scanner on a favoured core with its sibling idle, against unpinned. Cheap, and it decides the hypothesis before any production code. **As the design review requires,** it records each role's CPU, core and sibling, and the achieved frequency, temperature and throttling status with every timing, because placement also changes cache sharing, and SMT sharing and frequency droop must be separated by a placement × verified-frequency contrast, not assumed.

### 4. Model the knee properly: a simulation, with MVA on a documented surrogate (Jain ch. 25, 34; Harchol-Balter ch. 19)

**The tool.** MVA gives the smooth curve between the asymptotes for a closed product-form network. **Correction (ChatGPT Pro's design review, `../external/chatgpt-design-review.md`):** the decoder is not one. W = 4k is an admission ceiling, not a fixed population, and in-order resequencing and the byte gate are state-dependent blocking. So the primary model is a trace-driven discrete-event simulation; MVA is applied to an explicitly documented surrogate network, and its discrepancy from the simulation is reported as approximation error.

**Applied.** The fit wall(k) = c + max(b, a/k) needs c = 32 ms, yet at k = 8 only 2.5 ms of the wall is outside the scanner's busy time and waits. So c is not a real fixed overhead: it is the hard max paying for the knee's curvature. The residuals say the same: −13 ms at k = 8–14, +21 ms at k ≥ 17.

**What it gives.** No speed by itself; it makes the model honest, so the predictions in 1–3 and 5 rest on a validated curve. This is the systems-biology model of programme step 5.

**Test.** The simulation with the measured per-block demands, checked against the 20 measured walls with conservation identities and exact small cases (Jain ch. 25: verification and validation); then MVA on the surrogate, with its error against the simulation.

### 5. Size the window by Little's law (Jain ch. 30, 33; Harchol-Balter ch. 6)

**The tool.** N = X·R: the work in flight needed to keep a server busy is its throughput times the residence time.

**Applied.** At the best point X ≈ 1724 / 0.191 ≈ 9000 blocks a second, and a block resides about s + w ≈ 0.9 ms before it can be read, so about 8–9 blocks in flight keep the scanner busy (~~withdrawn~~: service demand is not residence time; correction 6). The window is 4k slots: 32 at k = 8, 80 at k = 20. Slot waits are already near zero on the plateau, so the window is not binding there. Memory grows about 2.4 MB a worker, not a slot (the 12 MiB ready cap bounds the slots' share), so opportunity 2 is the memory lever; this one matters only if the window's memory becomes significant.

**Classes.** 46% of the blocks are tiny (opportunity 6). ~~So the window measured in work is about half its nominal size~~ (withdrawn: occupancy is residence-weighted; correction 6). Work-based admission (from `queueing-discipline.md`) fixes that if it ever binds.

### 6. Workload characterisation and representativeness (Jain ch. 4–6; Harchol-Balter ch. 18, classed networks)

**The tool.** Choose workloads that exercise the system as real use does; characterise them by clustering their parameters; model distinct job types as classes.

**Applied (measured).** The corpus's blocks fall into two classes: 792 tiny blocks (under 10 kbit, 0.1% of the bits) and 932 real ones. The tiny ones are almost free to the scanner (p10 read 1 µs) and their cost to the workers is not yet measured. The real blocks are not heavy-tailed: CV 0.25–0.29 (C² ≈ 0.07).

**What it means.** Every number above is one corpus. R = w/s, the knee and the prize in opportunity 1 all depend on the mix. Before the adaptive cap (2) is adopted, the profile needs 3–5 corpora chosen to span R: high and low compression ratio, many small members against few large ones. Clustering per-block counters (n, r, h₁, switches) is how to choose them.

### 7. Allocation of variation across corpus, k and machine (Jain ch. 17–19, 21–23)

Already in `../briefs/character.md` for activities; the new use is a 2-factor or 3-factor design over (corpus, k, pinning) with replication, analysed on logs (multiplicative model), so that opportunities 2 and 3 are judged with their interactions. It costs workstation time, so it follows 3 and 6.

## Ruled out by the measurements

- **Size-based scheduling (SJF, SRPT, LPT; task assignment by size, SITA; Harchol-Balter ch. 20, 24, 28–33).** These pay off with high job-size variability, above all heavy tails. **Withdrawn to "not supported yet" (correction 5):** the mixture's C² is about 0.98, and worker service per block was not measured.
- **Redundancy and power-of-d (the Redundancy-d paper).** They help when servers' delays are independent and variable; here they are neither, and duplicating work costs the scarce resource.
- **Why scheduling matters so little here.** (Corrected, correction 7: the decoder is a finite pipeline with feedback-controlled admission, not a closed system.) Schroeder, Wierman and Harchol-Balter (NSDI 2006, on 15-857's reading list) show that scheduling policies have much less effect in closed systems than in open ones. That is consistent with the 0.945–0.998 efficiency measured.
- **Heavy-tail machinery (Pareto, matrix-analytic methods).** No heavy tail in this corpus. Re-check per corpus under 6. Phase-type methods also fit low variability: deferred because simulation is cheaper, not ruled out.

## Sequence

**Revised after the corrections (ChatGPT Pro's §5, adopted):**
1. 7 against 8 workers through the public API (the existing recalibration's design, on current master), logging asked threads, built workers, window and output contract.
2. The scanner-placement contrast, with the timers on and off near 16 workers and achieved frequency (stage 2).
3. The joint improvement frontier at 6, 8 and 10 workers: scanner and worker oracles together, with the reader's activity measured.
4. Stage 1's trace for a validated finite-run simulation; MVA only as a surrogate comparison.
5. More corpora, chosen by measured scanner/worker demand ratio and block classes, before any default changes.

**The original sequence, kept for the record:**

1. **Now, cheap:** the pinned scanner-profile variant (3) and the adaptive-cap design (2), registered for adoption with the threshold from the profile.
2. **With stage 1 (#841):** MVA and the simulation validated against the 20 walls (4); the per-visit demands confirm the machine-bound estimate (1).
3. **Before adopting 2:** 3–5 corpora chosen by clustering (6), then the factorial over corpus × k × pinning (7).
4. **The large one:** the parallel or cheaper scanner, scoped by the machine bound (1): about 43% on this machine if the estimate holds.
