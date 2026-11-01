# Recovering the four-worker penalty: a ranked architectural research plan

1 November 2026. **Read-only:** nothing was implemented or run for this plan. Every figure
comes from earlier records, cited below.

- **Labels:** **M** = measured; **D** = derived from measured figures; **I** = inference or
  model.
- **Machine:** the i9-10900K (20 MB L3) unless stated; four workers on four physical cores.

## 0. The budget

| | CPU-s/GB |
| --- | ---: |
| Four workers (attribution run, M) | 3.579 |
| One worker (M) | 2.707 |
| **Penalty** | **0.872** |
| Prioritisation threshold for an architectural direction | 0.25 (about 29%) |
| Ambitious target | 0.50 (about 57%) |

Throughput is busy cores ÷ CPU-s/GB. Two limits follow (D):
- **Efficiency alone tops out at about 1.32×.** Recovering the whole penalty at four workers
  gives 3.579 / 2.707 if utilisation stays the same.
- **Utilisation is the other half.** Four workers run 3.15 of 4 cores busy (M, walk trial).
  The dispatch study found 69.6% of worker time decoding and 18.9% with no job (M).

So **1.5× at four cores needs both levers.** For example, recovering 0.6 CPU-s/GB (×1.2)
and going from 3.15 to 3.9 busy cores (×1.24) gives 1.49×. **2× needs more than four
cores' worth of work**, or a cheaper single-worker kernel; the cooperative policy already
gives 1.26× at 20 threads (M). This plan addresses the efficiency half. The utilisation
half (the serial scanner, idle workers) is named in §6, not solved here.

## 1. What the record says about where the penalty is

| Finding | Value | Label | Source |
| --- | --- | :-: | --- |
| Instructions per byte, w1 → w4 | 32.63 → 33.06 (+1.3%) | M | attribution §3 |
| Cycles per byte | 13.46 → 17.51 | M | same |
| Added cycles that are stalls with a load outstanding | 72% | M | same |
| …with a demand L3 miss outstanding | 25% | M | same |
| Phase rises per row: walk+assembly, build, MTF, RLE1+CRC | +0.572, +0.074, +0.031, +0.046 ns | M | bwt-memory §1 |
| Traversal alone; assembly alone (w1 → w4, CPU-s/GB) | 1.162 → 1.710; **0.041 → 0.147 (3.5×)** | M | P87 S2 |
| **L3-miss stall samples at w4:** copies (`memmove`), reader drain, RLE1+CRC, build, **walk** | **44%, 13%, 10%, 11%, 4%** | M | attribution §5 |
| **DRAM writes per output byte** | **0.6 → 6.9** | M | attribution §4 |
| DRAM reads per output byte (P87 arm A, corrected) | 0.38 → 9.0 | M | P87 §3 |
| T-vectors live at w4: mean, max | 2.19 (7.9 MB), 4 (14.4 MB) | M | attribution erratum |
| Knee: best worker count at -9 / -6 / -3 | 4 / 6 / 6–8, at 14.4 MB of live tables both times | M | workstation-scaling §9 |
| Reads past L2 per byte, -3 against -9 | unchanged (1.25–1.29 against 1.30–1.40); hit share follows live table bytes | M | libsais-knee |
| **2 teams of 2 against 4 workers, on 4 threads** | **0.73 against 0.84 CPU-s (−13%), 891 against 1007 MB/s (−11%)** | M | cooperative-decoder; frontier |
| Reads in flight per core (p4); fill buffers found full | 7.0; 0.24 of cycles | M | coop §10.4 |
| Demand L1D misses in flight, w1 → w4 | 2.19 → 2.17 | M | attribution §3 |
| Walk slowdown per other concurrent walker | +0.173 (R² 0.23) | M, correlational | bwt-memory §2.2 |

**What this changes in the framing.** The walk's lookups are software-prefetched, so its
waiting shows up as general memory stalls, not demand L3 misses. The four-worker regime
has two parts:

1. **Table residency.** The tables crowd the L3. The knee and the hit share follow live
   table bytes, and the walk's cost rises.
2. **Per-block data movement.** About 8 bytes of intermediate data are written per output
   byte (I, from the buffers' sizes): `ll`, the 4-byte T-vector, the segments, `rle` and
   the output. With one worker it stays on chip. With four, it is evidently written back and read again (6.9 B/B of writes),
   and the copies hold most of the L3-miss stalls.

These are different levers, and earlier work attacked only the first: compact tables, ψ²,
the unrolled walker.

## 2. The options, ranked

### R1 · Fewer live tables at four cores: teams of 2 with the MTF overlapped (direction A)

- **Measured bound.** Two teams of 2 on four threads use 13% less CPU than four workers.
  That is the largest CPU saving the project has measured at four cores, and it comes
  despite 47% more instructions per byte in the inversion (35.0 against 23.8, M). But
  throughput is 11% lower.
- **Why throughput drops (M, coop §17.4).** A team's MTF is serial: 22–28% of its round,
  with the second core idle.
- **The new mechanism.** Run the next block's MTF on the team's idle core while the current
  block inverts. This is "batched team" P77 in the register: "overlap one block's MTF with
  another's inversion". It is not a compact-table variant: the table stays 4 bytes a row,
  and fewer of them are live.
- **Ceiling (I).** Up to about 0.45 CPU-s/GB-equivalent (−13%), at a throughput between
  0.89× and about 1.15–1.2×, depending on how much of the 22–28% the overlap hides.
- **Cost.** High: pipelining inside the cooperative scheduler, and one more `ll` per team.
- **Risk.** Coordination overhead, the second core's idle time, memory.

### R2 · Delete per-block data movement: RLE1 straight from the segments (direction A, buffers not table)

- **Measured anchors.**
  - Assembly is the step that grows most under concurrency (3.5×, to 0.147 CPU-s/GB at w4).
  - Copies hold 44% of the L3-miss stall samples.
  - DRAM writes rise 11×.
  - P87 changed only where the assembly writes, and still recovered 0.066 CPU-s/GB.
  - Most of P87's saving (≈0.07 of 0.098, traced) was indirect, appearing in other steps.
- **The new mechanism.** Run RLE1 and the CRC over the segment buffers in order, carrying
  the run state across segment boundaries. Assembly then disappears, along with `rle` and
  the assembled copy of the block: one write and one read of the whole block, per block.
  Optionally, hand decoded blocks to the reader without its drain copy (13% of the L3-miss
  stalls).
- **Ceiling (I).**
  - **Direct:** up to 0.147 CPU-s/GB (the assembly) plus part of RLE1's 0.195.
  - **Indirect:** P87's ratio of indirect to direct savings (about 2.7) suggests more.
  - **Planning range:** 0.2–0.35 CPU-s/GB.
- **Cost.** Moderate. RLE1 across segment boundaries needs careful tests (a run straddling
  a boundary, a 4-byte prefix at a segment's end); it is otherwise local.
- **Relation to P87.** R2 subsumes P87's production trial: with no assembled copy, there is
  no destination to reuse. **I recommend holding the P87 production trial (PR #128) until
  R2's feasibility is known.** Running both would spend the one P87 trial on a design R2
  would replace.

### R3 · More useful requests in flight / prefetch timing (direction B)

- **Evidence against much headroom (M).**
  - Production already keeps about 7 reads in flight per core, and finds the fill buffers
    full 24% of cycles.
  - Demand misses in flight are unchanged from w1 to w4.
  - No prefetch hint or lane count beats the shipped setting: `t1` 0.989×, `nta` 0.61×,
    16 lanes 0.955×.
  - The unrolled walker saves ≤0.6% CPU at w4.
- **What recovery would need (D).** Getting back w1's walk cost at w4's latency needs about
  1.5× the overlap, about 9 in flight. That is near a 10–12-entry fill-buffer limit (I).
- **Ceiling (I).** ≤0.1–0.2 CPU-s/GB.
- **Cheapest open measurement.** A prefetch-distance sweep, never run.
- **Ranking.** Third. A measurement, not an architecture.

### R4 · Schedule which phases overlap (direction C)

- **Evidence.** Correlational only (+0.173 per concurrent walker). A K-permit gate reached
  ≤0.77× the ungated decoder (M, older code).
- **Ceiling (I).** Removing all walk-on-walk overlap would recover ≤0.31 CPU-s/GB, but only
  by idling cores. Traversal and assembly are about 60% of each worker's decode time
  (1.806 of 3.04 ns/row at w4), so walks can't be kept apart without leaving cores idle.
- **Ranking.** Fourth: deprioritised.

### Not revived

None of these has a new mechanism for why it would now behave differently:

| Idea | Why it stays closed (M) |
| --- | --- |
| Compact Ψ | `u` moved the knee from 4 to 6 but ran at u1/o1 0.681 and u4/p4 0.899: per-step byte recovery costs more than the footprint saves |
| ψ² / pair walk | +40.6% / +24.3% wall: a costlier build, and the walk is bound by instructions per step |
| Unrolled walker | NEGATIVE on the i9 |
| Precopy | 18% / 6% slower |

## 3. The discriminating experiment (design only)

**Question.** At four workers, is the target walk's slowdown mainly residency (footprint)
pressure, or request pressure in the shared L3? The answer decides between R1/R2 and R3.

**Shape.**
- **Target:** one real decoder worker on CPU 1, decoding Wikidata at -9, with trace level 2
  for its phases and per-core counters.
- **Neighbours:** CPUs 2–4 run one condition at a time, looping continuously for the
  target's whole decode.
- **Design:** paired, with the order rotated; 10 repetitions per condition.

| Condition | Neighbours | What it varies | Synthetic? |
| --- | --- | --- | :-: |
| N0 | idle | baseline (≈ w1) | — |
| N1 | three real decoders on Wikidata -9 | the production mix: **calibration** | no |
| N2 | three real decoders on **the same Wikidata recompressed at -3** | footprint about a third (1.2 MB tables, smaller buffers), at unchanged reads past L2 per byte (M, knee journal) | **no** |
| N3 | three random-read loops over 1.5 MB each (larger than L2, about 4.5 MB in all), several requests in flight, the rate set to N1's measured L2-miss rate per neighbour core | request pressure with little footprint | yes |

N2 is the key improvement on a purely synthetic matrix. It changes footprint using the
real decoder's own code, so it needs no fidelity argument. Only one synthetic condition
remains.

**Calibration (registered before any run).**
- **(a) N0 matches production.** The target's walk cost under N0 is within ±5% of
  production's w1 walk cost (1.162 ns/row).
- **(b) N1 reproduces the slowdown.** The target's walk slowdown under N1 is at least 70%
  of production's w1 → w4 rise (+0.548 ns/row), so ≥ +0.38.
- **(c) N3 matches its target rate.** N3's measured request rate is within ±20% of N1's.
  N2's is measured and reported; it isn't forced.

If (a) or (b) fails, the harness is unrepresentative and **nothing is interpreted
causally**.

**Measurements.**
- **Target:** traversal, assembly, build and RLE1 ns/row; cycles; demand and offcore
  occupancy; L3-miss stall cycles; fill-buffer-full cycles.
- **Neighbours:** each core's L2-miss and L3-miss rates (perf `-C`).
- **Machine-wide:** IMC traffic, with the corrected units.
- **Perturbation:** the trace's cost against N0 without it (≤3%, as in P87).

**Signatures and what follows.** S(N) is the target's traversal slowdown over N0.

| Outcome | Reading | Prefer | Deprioritise |
| --- | --- | --- | --- |
| S(N2) ≤ 0.4·S(N1), S(N3) ≤ 0.3·S(N1) | footprint dominates | R1 (fewer tables), then R2 | R3, R4 |
| S(N3) ≥ 0.7·S(N1), S(N2) ≥ 0.7·S(N1) | request pressure dominates | R3 (measure, then prefetch distance and request reduction); R2 still cuts traffic | R1 |
| both partial | mixed | the option with the larger modelled payoff: R2 (low risk), then R1 | R4 |
| calibration fails | the synthetic model is inadequate | return to production-native interventions (R2, R1 prototypes) | the neighbour programme |

R2 is useful in every row, because it removes traffic whichever pressure dominates. R1 and
R3 are the options the experiment discriminates between.

**Cost.**
- **Build:** about 300–400 lines: a neighbour launcher, one request generator, the runner
  and the analysis. It reuses the P87 harness, the trace and the counter infrastructure
  (with the corrected IMC parser).
- **i9 time:** about 20–25 minutes. One untimed VM dry run first.

**Stopping rules.**
- **Calibration fails:** stop, report, and don't refine the synthetic conditions.
- **No more conditions:** none are added after seeing results.
- **One run per label.**

## 4. Recommended sequence

1. **No new runs needed.** The fill-buffer and in-flight figures already say R3 has little
   headroom.
2. **Build and freeze the §3 experiment.** Then do an untimed dry run, and stop for
   authorisation.
3. **In parallel, a design-only feasibility note for R2.** RLE1 from segments: the boundary
   cases, the CRC, the reader hand-off, and the expected traffic removed per block.
4. **After the experiment:** prototype the direction its outcome prefers. Each prototype
   gets one registered trial with an upside gate of ≥0.25 CPU-s/GB, or ≥10% throughput at
   equal CPU. One trial; then keep it, close it, or stop.
5. **P87 production (PR #128): hold until step 3.** If R2 is infeasible, run P87's single
   bounded trial as planned.

## 5. What this plan does not claim

- **The 24× DRAM figure is evidence of a changed memory regime.** It does not prove that
  T-vector lookups are served from DRAM; the walk's L3-miss stall share is only 4%.
- **The R1 and R2 ceilings are models.** Each is anchored to one measured figure: the
  teams' −13% CPU, and P87's 0.066 with its indirect share.
- **The overlap association is observational.**

## 6. Beyond efficiency: the other half of 1.5–2×

At four workers, 18.9% of worker time has no job (M), and the scanner (the Huffman pass)
is serial. Any route to 1.5× needs a utilisation lever too, such as a faster or parallel
scan, or a feeder that keeps the workers supplied. That is a separate investigation, listed
here so the efficiency work isn't mistaken for the whole path.
