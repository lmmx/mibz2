# The performance courses, mined against the scanner profile

Louis, on the scanner profile (journal §57): "that is awesome and also suggests we need to mine these courses for opportunities then."

The profile showed that a two-station model (one serial scanner feeding k workers) predicts the decoder's knee from four worker counts. That makes the course material usable directly: its laws and models now have measured inputs. This file goes through the three courses (`../methodology/performance-courses.md`, the sources in `../methodology/sources/`) and keeps each tool that the measurements say has something to give, with a predicted size, the measurement that tests it, and its cost. Tools the measurements rule out are listed too, with why.

All figures are from `scan-profile/run-20261124T223748Z` (i9-10900K, 10 cores, 20 threads, one corpus, unpinned) unless marked as a prediction or an estimate.

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
| Python default k | `available_parallelism()` = 20 here | `src/python.rs` |

## Opportunities, largest first

### 1. The machine bound: the gap is in the topology, not the discipline (Jain ch. 33–34; Harchol-Balter ch. 6–7, balanced job bounds)

**The tool.** The bottleneck law bounds throughput by the busiest server's demand. The balanced job bound adds the other side: total demand over total capacity. The decoder has been judged only against the first.

**Applied.** Against the busiest server (the scanner), efficiency is 0.97: almost nothing to recover by scheduling. Against the whole machine it is not. The workers' total work rises from 1.43 s at k = 8 to 1.92 s at k = 20 because hyperthreads share cores. If that rise is all SMT sharing, a thread with a busy sibling runs at about 0.745 of a lone thread, so a core running two threads delivers about 1.49 lone-thread equivalents, and the machine about 14.9. Total demand is about 1.43 + 0.19 = 1.62 s, so the machine bound is about 1.62 / 14.9 ≈ 0.109 s. **Estimate:** the best wall, 0.191 s, is about 0.57 of the machine bound. The other 43% is lost because one server does all the scanning: it is the topology's, and no discipline reaches it.

**Is the scanner really the limit above the knee?** The design review warns that a busy-time crossing alone doesn't show it: a slow consumer (the in-order reader) could control throughput instead. The profile has the evidence it asks for, in aggregate: at k = 10 the scanner works for 0.945 of the wall, its waits for admission (0.014) and input (0.007) are small, and 1.67 workers are idle on average. If the reader were the limit, the window would fill and the scanner would wait on admission; it doesn't. The per-interval version (workers empty *while* the scanner is producing) comes from stage 1's ordered trace.

**What it means.** This is the size of the prize for a parallel or cheaper scanner on this machine, and a cross-check on the four-arm test: arm C (the scanner's output supplied) ran at 0.806 of the shipped decoder at 10 workers, inside this bound.

**Caveats.** The 0.745 factor also absorbs memory-bandwidth and cache contention; it is an estimate from busy times, not a measured SMT factor.

**Test.** Stage 1 (#841) measures per-visit demands; a run with the scanner's output supplied at k = 10–20 (arm C extended) measures the achievable fraction directly.

### 2. Right-size the worker count (Harchol-Balter ch. 14–15: capacity provisioning, square-root staffing; ch. 27 and the AutoScale paper: idle capacity is not free)

**The tool.** Provision servers for the offered load: R = w/s servers' worth here, plus a margin for variability (square-root staffing gives R + β√R; with C² ≈ 0.07 the margin is small).

**Applied (measured).** R = 7.5. The Python binding's default is every logical CPU, 20 here. Against k = 8, k = 20 takes 0.230 s against 0.191 s (20% longer), 2.14 against 1.61 s of user CPU (+34%) and 74.8 against 54.1 MB of the decode's own peak memory (+38%). From k = 9 the extra workers add CPU and memory and no speed; from k = 16 they slow the scanner (opportunity 3).

**What it needs.** R depends on the data, so a fixed cap is a stopgap. Both s and w are linear in a block's symbols with a small per-block term (the scan fit gives 1.5 ns a symbol), so R can be estimated online from the first few blocks' free counters and the workers' busy times: a renewal-reward estimate (Harchol-Balter ch. 23), time-average rate = E[work a block] / E[time a block].

**Cost.** Small code; it changes a production default, so it needs an adoption registration. Validation needs more than one corpus (opportunity 6).

### 3. Put the bottleneck on the fastest server (Harchol-Balter ch. 1 and 14, "one fast rabbit or many turtles"; the SIGMETRICS 2009 power-allocation paper)

**The tool.** With unequal servers, the bottleneck stage belongs on the fastest one, and capacity at a non-bottleneck stage is worth nothing.

**Applied.** On the plateau the wall is the scanner's work, so the scanner's clock passes 1:1 into the wall. Two measured signs that it isn't getting the best core:
- its decode costs 1.41 ns a symbol at k = 1 and 1.51 ns at k ≥ 3 (7% slower);
- at k ≥ 16 its work rises 23%, with the wall following.

The i9-10900K's all-core turbo is lower than its single-core and favoured-core turbo (Turbo Boost Max 3.0 picks two favoured cores). **Hypothesis:** pinning the scanner to a favoured core, with its SMT sibling kept free, recovers most of both: about 7% on the plateau and the whole 20% step at k ≥ 16.

**Test.** A pinned variant of the scanner profile at k ∈ {8, 10, 16, 20}: scanner on a favoured core with its sibling idle, against unpinned. Cheap, and it decides the hypothesis before any production code. **As the design review requires,** it records each role's CPU, core and sibling, and the achieved frequency, temperature and throttling status with every timing, because placement also changes cache sharing, and SMT sharing and frequency droop must be separated by a placement × verified-frequency contrast, not assumed.

### 4. Model the knee properly: a simulation, with MVA on a documented surrogate (Jain ch. 25, 34; Harchol-Balter ch. 19)

**The tool.** MVA gives the smooth curve between the asymptotes for a closed product-form network. **Correction (ChatGPT Pro's design review, `../external/chatgpt-design-review.md`):** the decoder is not one. W = 4k is an admission ceiling, not a fixed population, and in-order resequencing and the byte gate are state-dependent blocking. So the primary model is a trace-driven discrete-event simulation; MVA is applied to an explicitly documented surrogate network, and its discrepancy from the simulation is reported as approximation error.

**Applied.** The fit wall(k) = c + max(b, a/k) needs c = 32 ms, yet at k = 8 only 2.5 ms of the wall is outside the scanner's busy time and waits. So c is not a real fixed overhead: it is the hard max paying for the knee's curvature. The residuals say the same: −13 ms at k = 8–14, +21 ms at k ≥ 17.

**What it gives.** No speed by itself; it makes the model honest, so the predictions in 1–3 and 5 rest on a validated curve. This is the systems-biology model of programme step 5.

**Test.** The simulation with the measured per-block demands, checked against the 20 measured walls with conservation identities and exact small cases (Jain ch. 25: verification and validation); then MVA on the surrogate, with its error against the simulation.

### 5. Size the window by Little's law (Jain ch. 30, 33; Harchol-Balter ch. 6)

**The tool.** N = X·R: the work in flight needed to keep a server busy is its throughput times the residence time.

**Applied.** At the best point X ≈ 1724 / 0.191 ≈ 9000 blocks a second, and a block resides about s + w ≈ 0.9 ms before it can be read, so about 8–9 blocks in flight keep the scanner busy. The window is 4k slots: 32 at k = 8, 80 at k = 20. Slot waits are already near zero on the plateau, so the window is not binding there. Memory grows about 2.4 MB a worker, not a slot (the 12 MiB ready cap bounds the slots' share), so opportunity 2 is the memory lever; this one matters only if the window's memory becomes significant.

**Classes.** 46% of the blocks are tiny (opportunity 6), and a slot holds a block whatever its size, so the window measured in work is about half its nominal size. Work-based admission (from `queueing-discipline.md`) fixes that if it ever binds.

### 6. Workload characterisation and representativeness (Jain ch. 4–6; Harchol-Balter ch. 18, classed networks)

**The tool.** Choose workloads that exercise the system as real use does; characterise them by clustering their parameters; model distinct job types as classes.

**Applied (measured).** The corpus's blocks fall into two classes: 792 tiny blocks (under 10 kbit, 0.1% of the bits) and 932 real ones. The tiny ones are almost free to the scanner (p10 read 1 µs) and their cost to the workers is not yet measured. The real blocks are not heavy-tailed: CV 0.25–0.29 (C² ≈ 0.07).

**What it means.** Every number above is one corpus. R = w/s, the knee and the prize in opportunity 1 all depend on the mix. Before the adaptive cap (2) is adopted, the profile needs 3–5 corpora chosen to span R: high and low compression ratio, many small members against few large ones. Clustering per-block counters (n, r, h₁, switches) is how to choose them.

### 7. Allocation of variation across corpus, k and machine (Jain ch. 17–19, 21–23)

Already in `../briefs/character.md` for activities; the new use is a 2-factor or 3-factor design over (corpus, k, pinning) with replication, analysed on logs (multiplicative model), so that opportunities 2 and 3 are judged with their interactions. It costs workstation time, so it follows 3 and 6.

## Ruled out by the measurements

- **Size-based scheduling (SJF, SRPT, LPT; task assignment by size, SITA; Harchol-Balter ch. 20, 24, 28–33).** These pay off with high job-size variability, above all heavy tails. The real blocks have C² ≈ 0.07, and efficiency against the busiest server is at least 0.945, so at most about 5% is there to recover.
- **Redundancy and power-of-d (the Redundancy-d paper).** They help when servers' delays are independent and variable; here they are neither, and duplicating work costs the scarce resource.
- **Why scheduling matters so little here.** The decoder is a closed batch system: a fixed population of blocks circulates. Schroeder, Wierman and Harchol-Balter (NSDI 2006, on 15-857's reading list) show that scheduling policies have much less effect in closed systems than in open ones. That is consistent with the 0.945–0.998 efficiency measured.
- **Heavy-tail machinery (Pareto, matrix-analytic methods).** No heavy tail in this corpus. Re-check per corpus under 6.

## Sequence

1. **Now, cheap:** the pinned scanner-profile variant (3) and the adaptive-cap design (2), registered for adoption with the threshold from the profile.
2. **With stage 1 (#841):** MVA and the simulation validated against the 20 walls (4); the per-visit demands confirm the machine-bound estimate (1).
3. **Before adopting 2:** 3–5 corpora chosen by clustering (6), then the factorial over corpus × k × pinning (7).
4. **The large one:** the parallel or cheaper scanner, scoped by the machine bound (1): about 43% on this machine if the estimate holds.
