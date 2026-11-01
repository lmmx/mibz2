# Why the inverse BWT costs more at four workers: evidence, and one discriminating experiment (proposal)

**Status:** a research note and proposal only (reports 113–124). Nothing is implemented,
run or timed. The read-only analyses in §2 use only archived data:
`docs/decoder-attribution-records`, `occupancy/`, `stat.json` and `record.json`. Their scripts
and outputs are in `2026-11-01-bwt-memory-proposal/`.

## Summary

1. **The penalty is in the walk.** Per row, paired block by block across the attribution
   run's 1- and 4-worker full traces, the walk's thread CPU time rises **1.46×** (1.234 →
   1.806 ns). That is **79%** of the per-block increase. The build rises 1.15×, MTF
   1.07× and the post-walk RLE1/CRC 1.35×.
2. **The walk slows with concurrent walks, not concurrent builds** (correlational). With
   3, 2, 1 or 0 other workers walking, a 4-worker walk costs 1.66, 1.54, 1.41 or 1.29× the
   same block's 1-worker walk. Concurrent builds show no positive association.
3. **The walk's extra cycles are extra latency at constant parallelism.**
   - Reads leaving L2 per byte are unchanged (1.47 → 1.42).
   - Their average occupancy, a latency proxy, rises 38.6 → 57.5 cycles.
   - By Little's law, the walk sustains about 6 misses in flight at both 1 and 4 workers.
   - So Δwalk ≈ (L2-miss requests per byte) × Δlatency ÷ MLP ≈ 0.93 × 19 ÷ 6 ≈ 3
     cycles/byte, about what is measured (≈ 2.7).
4. **What the existing counters cannot identify is where the extra latency comes from.**
   Two explanations both fit the existing counts:
   - more of the walk's prefetched lines served from DRAM rather than L3 (residency: H1, or
     DRAM contention: H5-DRAM);
   - L3 hits getting slower under four-core load (ring and LLC contention: H5-LLC).

   The demand-only events (`STALLS_L3_MISS`, `MEM_LOAD_RETIRED.L3_*`) are blind to the
   walk, whose memory traffic is about 98% prefetch.
5. **The preferred experiment is a controlled neighbour matrix** on one decode-order
   worker, with the walk bracketed by in-process counters.
   - Three neighbours on the other three of the same four cores each apply one controlled
     kind of pressure: real decode, walk-only, build-only, clean capacity (a dose series),
     dirty capacity, L3 request pressure without capacity, and DRAM bandwidth without
     capacity.
   - It needs about 300 lines of new harness code (not authorised) and one registered i9
     capture (about 15 minutes).
6. **Each plausible result selects a different engineering path** (§5.7). It needs a quality
   gate: the real-decode neighbours must reproduce at least 70% of the decoder's measured
   walk inflation. Otherwise the memory-interference framing is wrong, and that is itself
   the decision.

## 1. The opportunity, measured

| CPU-s per GB, the attribution run (i9-10900K, CPUs 1–4) | 1 worker | 4 workers | Increase |
|---|---:|---:|---:|
| Whole decoder | 2.707 | 3.579 | **0.872** |
| …in `decode_block` (phase-timed, this note) | — | — | 0.723 |
| …the walk | 1.234 | 1.806 | **0.572** |
| …the build | 0.497 | 0.571 | 0.074 |
| …MTF | 0.456 | 0.487 | 0.031 |
| …RLE1 and CRC (post-walk) | 0.134 | 0.180 | 0.046 |

- **Per-row phase times ≈ CPU-s per GB.** A full block is 900,000 rows, and its decoded
  size is ≈ 900 kB on this corpus.
- **The build plus walk** (0.646) is consistent with the attribution's 0.68–0.72 CPU-s/GB
  for the inversion "with its copies and kernel time".
- **The budget used below is the walk's 0.57 CPU-s/GB.** That is 16% of the 4-worker
  decoder's CPU cost, not 79% of anything larger.
- **Recovering CPU-s/GB is not the same as throughput.** At w4 the decoder keeps 2.81–3.15
  of 4 cores decoding (`report.txt`; the walk-unroll analysis). So a CPU saving converts to
  throughput only partly, unless utilisation also improves.

## 2. What the existing evidence establishes (and what it doesn't)

### 2.1 Phase split (new; `phases.py`)

- **Method:** 197 full 900k-row blocks, paired by block index. Each block's phase CPU
  time is its median over the 10 runs at each worker count.
- **Robust across blocks:** the walk's ratio has a block median of 1.452, with p10–p90 of
  1.359–1.587.
- **Not descheduling:** wall time and CPU time agree within 1% in every phase at w4.

### 2.2 Concurrency context (new; `context.py`; correlational)

- **The data:** 1,970 4-worker walks, each divided by the same block's median 1-worker walk.
  Regressed on the time-averaged number of *other* workers in each phase during the walk.
- **Coefficients:**
  - +0.173 per other worker walking;
  - +0.029 per other worker building;
  - +0.031 per other worker in MTF;
  - +0.42 per other worker in RLE1/CRC, though that phase occupies only 0.12 of a worker
    on average, so the coefficient is poorly determined;
  - intercept 1.165, R² 0.23.
- **The binned medians are monotone in walkers:** 1.29, 1.41, 1.54, 1.66 with 0–3 others
  walking. Against builders they are flat to falling: 1.48, 1.42, 1.23 with 0–2 others
  building.
- **What this does not show.**
  - **It is association, not cause.** The phase mix covaries with position in the run and
    with block content. Content is controlled by the pairing; position is not.
  - **The intercept above 1** (no concurrent phase, still 16% slower) fits other workers'
    tables staying cached while those workers are idle, but it doesn't prove it.

### 2.3 Counters, process-wide (`stat.json`, wikidata, medians, per decoded byte)

| Event | w1 | w4 | What it can and cannot say |
|---|---:|---:|---|
| cycles | 13.51 | 17.47 | |
| instructions | 32.63 | 33.07 | the work is the same |
| `offcore_requests.all_data_rd` | 1.47 | 1.42 | reads leaving L2 (demand, software prefetch and L2 hardware prefetch). **Unchanged.** |
| `offcore_requests_outstanding.all_data_rd` ÷ the above | 38.6 | 57.5 | occupancy per request: a latency **proxy** over a mixed population, not a measured latency |
| `l1d_pend_miss.pending` ÷ `pending_cycles` | 2.18 | 2.18 | average outstanding L1D misses when any is pending (whole process) |
| `l1d_pend_miss.fb_full` | 1.95 | 3.11 | cycles a demand load found no free fill buffer |
| `mem_load_retired.l3_hit` + `l3_miss` | 0.019 | 0.019 | **demand** loads reaching L3: about 1.4% of the reads leaving L2. The walk's traffic is prefetch |
| `cycle_activity.stalls_l3_miss` | 0.10 | 1.12 | demand-L3-miss stalls. The walk holds about 4% of their samples at w4 and 0% at w1 (`record.json`) |
| IMC reads, writes (B/B) | 1.6, 0.6 | 9.8, 6.9 | system-wide DRAM traffic |

**Event semantics:**
- `STALLS_L3_MISS` and `MEM_LOAD_RETIRED.*` count demand loads.
- A software `prefetcht0` is an offcore request of the `PF_L1D_AND_SW` kind.
- A demand load that finds the line already in flight is a fill-buffer hit, not a demand L3
  miss.
- **So the walk's exposure to DRAM is invisible to every demand-only event in this
  archive.** "The walk has few L3-miss stalls" does **not** mean its lines come from L3.

**What follows quantitatively:**
- **The walk's MLP is constant.** At w1 the walk runs 6.1 cycles a byte (1.234 ns ×
  4.97 GHz); at w4, 8.8 (1.806 × 4.89). With about 0.93 of the 1.42 L2-leaving reads per
  byte being the walk's (one table access a byte, a 256 kB L2 against a 3.6 MB table),
  Little's law gives ≈ 5.9 and 6.1 in flight. The walk is latency-bound at a fixed
  overlap.
- **The extra latency could have either source.**
  - IMC reads bound DRAM-served reads at ≤ 0.153 lines a byte at w4, so at most ≈ 11% of
    the reads leaving L2. With DRAM at ≈ 390 cycles and an L3 hit at ≈ 32, a DRAM share
    rising from ≈ 1.7% to ≈ 7% explains 38.6 → 57.5 cycles with L3 latency unchanged.
  - An unchanged DRAM share with L3 hits slowing to ≈ 50 cycles explains it equally well.
  - **The existing data cannot choose.**

### 2.4 History that constrains the design

The survey is cited to its records; the full list is in the session's survey of the register
(`docs/research-portfolio`, `2026-10-30-research-portfolio-register.md`).

**Prefetch:**
- The `prefetcht0` walk is shipped.
- `prefetcht1` is equivalent on the i9 (0.989), and `prefetchnta` is **0.61×** there.
- Disabling it is 0.88× on the i9 and 15–22% slower on CI with four copies.
- **No prefetch-distance sweep has ever been run** (open in the room-walk README and the
  cooperative-inversion journal).

**Copies:**
- Precopy was rejected (+18%/+6%).
- Removing the assembly copy measured 1.4–2.0% of a decode on CI, under its 5% gate.
- In tension with that, the attribution run puts `memmove` at 11% of the added CPU and 44%
  of L3-miss stall samples at w4.

**Representations:**
- **Pair walk (ψ²):** +24% at w4 on CI and 0.816× on the i9 at 4. Its build costs 3×.
- **Compact Ψ `u` (24-bit, 2.7 MB):** 0.68× per core, against a 0.85 gate. It recovers the
  byte for 16% of the walk at 4 blocks. Its design was rejected; its science stood.
- **N1:** model only. Gate B is unauthorised, and it has no resident-memory saving.

**P74-C and the unrolled walk:** both closed and negative at decoder level on the i9. The
unrolled walk's per-core gain on Intel is ≈ 0.2 c/B.

**Never measured:**
- a dirty-line or writeback counter by source;
- a supplier split (L3 against DRAM) for prefetch requests;
- `MEM_LOAD_RETIRED.FB_HIT`;
- page-walk cycles;
- an i9 walk stage measurement in isolation.

## 3. The hypotheses: what is established, what is not, what would discriminate

| | What the archive establishes | What remains unidentified | Competing explanation for the same data | Smallest discriminator | Supports / falsifies |
|---|---|---|---|---|---|
| **H1** T-vector residency and eviction | Walk latency up 49% at constant L2-miss count; IMC reads ×6. Live tables 2.19 on average (7.9 MB), 4 at most (14.4 MB), in a 20 MB inclusive L3, beside about 3.6 MB a worker of other buffers | Whether the walk's lines come from DRAM more often | H5-LLC: same latency rise with L3 hits slower | Supplier split of the walk's own prefetch requests (`OCR.PF_L1D_AND_SW.L3_HIT/L3_MISS`); clean-capacity neighbours at matched footprint | **Supports:** neighbours holding clean capacity reproduce the walk inflation in proportion to their footprint, and the walk's DRAM share rises. **Falsifies:** capacity neighbours leave the walk unchanged |
| **H2** prefetch interaction | MLP constant (≈ 6); FB-full cycles up 1.6× process-wide | Whether prefetches are late (demand hits in-flight lines), or take buffers from demand | A consequence of H1/H5's latency, not a cause | `MEM_LOAD_RETIRED.FB_HIT` and FB-full **in the walk**, against latency, across arms | **Cause:** MLP falls, or FB-full grows faster than latency explains. **Consequence:** MLP constant and FB-hit tracking latency (then a latency-scaled prefetch *distance*, never tried, is the lever) |
| **H3** intermediate writes and dirty eviction | IMC writes ×11 (0.6 → 6.9 B/B); `memmove` 44% of L3-miss stalls at w4 | Which buffers' lines are written back, and whether writebacks slow the walk | Writebacks as a *consequence* of the same capacity pressure (dirty table and segment lines evicted) | Dirty against clean neighbours at the same footprint | **Supports:** dirty neighbours slow the walk materially more than clean ones of equal size. **Falsifies:** no difference |
| **H4** build/walk overlap | Walk slowdown associates with concurrent *walks* (+0.17 each), not builds (+0.03) | Causation | Builds are rarer (0.36 workers on average): low power to see them | Build-only against walk-only neighbours | **Supports:** build-only neighbours slow the walk at least as much as walk-only ones. **Falsifies:** they slow it much less, as the correlation predicts |
| **H5** uncore and DRAM request contention | DRAM 11.8 GiB/s at w4, below the i9's peak | Whether the latency rise is in the LLC/ring or at DRAM | H1 (DRAM share rising) | (a) neighbours generating L3 hits at a high rate with a small footprint; (b) neighbours generating DRAM bandwidth with no LLC footprint | **H5-LLC:** (a) reproduces the inflation with the walk's DRAM share unchanged. **H5-DRAM:** (b) reproduces it |
| **H6** (another owner) page walks | Nothing (never counted) | 3.6 MB random over 4 kB pages ≈ 900 pages per table; a per-core STLB of 1,536 entries; page-table lines compete in the shared caches | — | `DTLB_LOAD_MISSES.WALK_ACTIVE`, `.WALK_COMPLETED` in the walk | **Supports:** walk-active cycles a byte rise with neighbours (lever: huge pages for `tt`, a small change) |

These are not exclusive. Writebacks (H3) and DRAM reads (H1) can both be consequences of one
working-set pressure, and FB-full (H2) can be a consequence of either. The experiment
measures what each *controlled* pressure does to the same target. It doesn't sum
overlapping stall counters.

## 4. Two candidate designs considered

| Option | What it is | Causal power | Cost |
|---|---|---|---|
| **B (minimal, correlational)** | Rerun the existing attribution runner at w1/w4 with an amended event list: OCR supplier split for `PF_L1D_AND_SW` and demand reads, `FB_HIT`, DTLB walks | Shows whether the DRAM share of L2-leaving reads rises (H1/H5-DRAM against H5-LLC), process-wide. Cannot say *why*, or isolate the walk | No new code; an event-list amendment and one capture |
| **A (preferred, interventional)** | The neighbour matrix below | Separates capacity, dirtiness, L3 pressure, DRAM bandwidth, build against walk, and page walks, *on the walk itself* | About 300 lines of harness, a runner, a VM dry run, one capture |

Option B could rule H5-LLC in or out. But if the DRAM share rises, it still can't say
whether capacity, dirtiness or bandwidth drives it, and that is the choice between the
engineering paths. **A is the one experiment that changes the decision.** B's event list is
folded into A.

## 5. The preferred experiment: a neighbour matrix on one walking core

### 5.1 Hypothesis, and the strongest alternative

**Primary (H1):** the walk's extra cost at four workers is mostly its table lines being served
from DRAM instead of L3, because the four workers' combined footprint exceeds what the
inclusive L3 keeps for each table.

**Strongest alternative (H5-LLC):** the walk's lines still come from L3, and the extra
latency is queueing in the shared ring and LLC slices under four cores' request rate.

They predict opposite responses to the two cleanest manipulations:
- a large clean footprint with a low request rate;
- a high L3 request rate with a small footprint.

### 5.2 Design

**The target** is one thread on CPU 1: the decoder's own stages in decode order, block by
block, over the frozen Wikidata corpus.
- **The stages:** the scanner, MTF, the build, the walk, RLE1 and the CRC, as `examples/phases.rs` does.
- **Each block's walk and build** are bracketed by a `perf_event_open` group, read once at
  each boundary (≈ 1 µs against a ≈ 1 ms walk).
- **Correctness:** output checked against the frozen hash.

**The neighbours** are three single-threaded processes on CPUs 2, 3 and 4. These are the
other three of the decoder's own four physical cores, with their SMT siblings 11–14 idle.
Each is pinned and starts before the target. One condition per arm:

| Arm | Neighbours | Isolates |
|---|---|---|
| A0 | none | the baseline (should match the decoder's w1 walk) |
| A1 | real decode-order loops (the same harness, at other block offsets) | the full 4-core memory environment, without dispatch. **The calibration anchor** |
| A2 | walk-only loops over their own freshly built tables | walk/walk |
| A3 | build-only loops | build/walk (H4) |
| A4 a, b, c | clean holders: each keeps a private 1.8, 3.6 or 5.4 MB array resident by slow random reads | capacity dose–response (H1) |
| A5 | dirty holders: the 3.6 MB of A4b, each touched line written | dirty eviction (H3), against A4b |
| A6 | L3 hammers: walk-only loops over a 0.8 MB table (L2 misses, L3 hits) | LLC/ring pressure without capacity (H5-LLC) |
| A7 | DRAM streams: non-temporal stores plus `prefetchnta` reads over 64 MB, throttled to the w4 decoder's DRAM rate less the target's own | DRAM bandwidth and turnaround without LLC footprint (H5-DRAM) |

**Each round** runs every arm once, in an order randomised per round. There are 10 rounds,
and each condition decodes the full file once per counter group.

### 5.3 Measurements

**On the target, per block, walk and build separately:**
- fixed counters: cycles, instructions, ref-cycles;
- four rotating programmable groups (Skylake has 4 programmable counters a thread):
  1. `L1D_PEND_MISS.PENDING`, `L1D_PEND_MISS.PENDING_CYCLES`, `L1D_PEND_MISS.FB_FULL`,
     `MEM_LOAD_RETIRED.FB_HIT`;
  2. `OFFCORE_REQUESTS.ALL_DATA_RD`, `OFFCORE_REQUESTS_OUTSTANDING.ALL_DATA_RD`, and two
     offcore-response events `PF_L1D_AND_SW` × {`L3_HIT`, `L3_MISS`} (both OCR MSRs);
  3. offcore-response `DEMAND_DATA_RD` × {`L3_HIT`, `L3_MISS`},
     `DTLB_LOAD_MISSES.WALK_ACTIVE`, `DTLB_LOAD_MISSES.WALK_COMPLETED`;
  4. `CYCLE_ACTIVITY.STALLS_L1D_MISS`, `STALLS_L2_MISS`, `STALLS_L3_MISS`, `STALLS_MEM_ANY`.

**System-wide, per arm:** IMC data reads and writes (free-running), so the neighbours'
actual bandwidth is calibrated rather than assumed.

**Primary outcome:** the target walk's **cycles per row**, in core cycles, so it is
frequency-free. Its inflation over A0 is compared with the decoder's measured walk inflation
(1.234 → 1.806 ns per row; ≈ 2.7 cycles per row).

**Unavailable or uncertain, stated in advance:**
- No cache allocation (CAT) or occupancy and bandwidth monitoring (CMT/MBM) on this part:
  LLC occupancy can't be measured or partitioned directly. Hence the holders.
- No per-request latency for prefetches: occupancy per request is a proxy.
- The offcore-response encodings for `PF_L1D_AND_SW` and their L3 supplier bits must be
  **probed and calibrated on the i9 before the capture.** Calibration means random
  prefetched reads over a 2 MB array (L3-resident), which must read as ≥ 90% `L3_HIT`, and
  over 256 MB (DRAM), which must read as ≥ 90% `L3_MISS`. If the calibration fails, the
  supplier split is not interpreted, and the decision rests on the causal arms alone.
- `perf_event_paranoid` is 4 on the i9, so the harness runs under `sudo` like the earlier
  runners.

### 5.4 Expected signatures

Each value is the target walk's inflation over A0, with Δ the decoder's 2.7 cycles/row.

| | A1 | A2 | A3 | A4 a→c | A5 vs A4b | A6 | A7 | Walk DRAM share | FB-hit, MLP |
|---|---|---|---|---|---|---|---|---|---|
| **H1** capacity | ≈ Δ | ≈ Δ | small | rises with footprint, ≈ Δ near the decoder's co-resident size | ≈ | small | small | rises | latency up, MLP flat |
| **H3** dirty eviction | ≈ Δ | < Δ | large | small | **A5 ≫ A4b** | small | moderate | rises | — |
| **H4** builds | ≈ Δ | small | **≥ A2** | — | — | — | — | — | — |
| **H5-LLC** | ≈ Δ | ≈ Δ | — | small | ≈ | **≈ Δ** | small | **flat** | latency up, MLP flat |
| **H5-DRAM** | ≈ Δ | — | — | small | — | small | **≈ Δ** | flat or rises | — |
| **H2** as cause | ≈ Δ | ≈ Δ | — | — | — | — | — | — | **MLP falls** or FB-full outgrows latency |
| **H6** page walks | ≈ Δ | ≈ Δ | — | — | — | — | — | — | walk-active cycles a row ≈ the inflation |

### 5.5 Quality criteria and early stops (registered before any capture)

**Q1 — the target is the decoder's walk.** A0's walk cycles per row must be within ±10% of
the decoder's w1 walk (≈ 6.1 cycles per row), and every block's output must match its hash.
**If not, stop:** the harness doesn't reproduce production conditions.

**Q2 — calibration (the anchor).** A1 must reproduce **≥ 70%** of the decoder's walk inflation
(≥ 1.9 cycles per row).
- **If not, stop and report "not reproduced by neighbours".** The 4-worker walk penalty then
  isn't separate-core memory interference alone. In-process owners take precedence:
  allocation, page faults, the frequency of a fully loaded package, or scheduling.
- That is itself the decision: the memory-system framing is wrong.

**Q3 — neighbour calibration.**
- A7 reaches its target DRAM rate within ±10% (IMC).
- A4 and A5 holders generate less than 10% of A1's L3 request rate, measured from their own
  counters, so they are capacity probes and not request generators.
- A6 generates at least A2's L3 request rate with less than 1.5 MB of footprint each.

**Q4 — noise.** Paired by round, each arm's ratio to A0 has a 95% interval narrower than
±0.05. Otherwise its result is reported as inconclusive.

**Q5 — the supplier split** is interpreted only if its calibration (§5.3) passes.

### 5.6 Safeguards

- **This is a diagnostic:** no change to the decoder, its defaults, its dispatch or its
  release.
- **Correctness is checked** on every target block.
- **Throughput, CPU-s/GB and RSS of the decoder are not measured** here because nothing
  changes. Any engineering change it motivates goes through the established gates: same
  binary, paired A/B on the i9, full-decoder throughput and CPU-s/GB at w4, peak RSS no
  higher, correctness.

### 5.7 The decision matrix

| Result | Interpretation | Engineering direction | Recovery bound against the walk's 0.57 CPU-s/GB |
|---|---|---|---|
| A4 dose–response reaches ≈ Δ at the decoder's co-resident footprint, DRAM share rises, A6 and A7 small | **Residency (H1)** | **Keep the 4-byte T.** Cut the co-resident footprint, ranked by the slope s (Δ cycles a row per MB): (i) walk into L's freed buffer (0.9 MB a worker; REG:813, never run); (ii) bound the segment buffers' ratchet; (iii) stagger concurrent walks (A2's per-walker cost sets the gain) | s × MB removed × 0.2 CPU-s/GB a cycle. For example, s ≈ 0.25 cycles per row per MB and 2.7 MB removed for the three neighbours gives ≈ 0.14 CPU-s/GB (25% of the walk penalty) |
| H1, and the slope says a 25% smaller table saves more walk than compact Ψ's measured per-core cost (≈ 16–19% of the walk at 4 blocks) | Residency strong enough to pay for compaction | **Reopen compact Ψ**, with this measured break-even as its gate | Bounded by the slope; at most the 0.57 |
| A5 ≫ A4b, A3 large | **Dirty eviction (H3)** | Reduce dirty footprint: no `rle` intermediate, segment buffers sized and reused in cache, output written in place. Non-temporal stores stay rejected while no buffer is write-once | Writes are ≈ 6.9 B/B; bounded by the A5 − A4b difference |
| A3 ≥ A2 | **Builds hurt walks (H4)** | Phase scheduling: don't overlap builds with others' walks | Bounded by the A3 effect × the fraction of walks overlapped by builds (≈ 0.36 workers on average) |
| A6 ≈ Δ, DRAM share flat, A4 small | **LLC/ring queueing (H5-LLC)** | Footprint changes won't help. Requests per byte must fall: a two-bytes-per-lookup walk (ψ²), reopened **only** with a build within its budget, since the pair walk failed on its 3× build; and/or a latency-scaled **prefetch distance** (never tried) | ψ² halves requests per byte: up to ≈ half the walk at w1 *and* w4, if its build fits |
| A7 ≈ Δ | **DRAM bandwidth and turnaround (H5-DRAM)** | Reduce DRAM bytes per output byte, writes first | Bounded by A7's share |
| MLP falls or FB-full outgrows latency | **Prefetch or LFB interaction (H2 as cause)** | A prefetch-distance experiment, a mechanism distinct from the rejected hint sweeps | — |
| Walk-active cycles ≈ the inflation | **Page walks (H6)** | Huge pages for `tt` (and `ll`): a small, isolated change, gated as usual | Up to the walk-active share |
| Q2 fails | **Not separate-core interference** | Re-attribute inside the process (allocation and page faults, package frequency, scheduling) before any memory work | — |

**More than one row can hold.** The arms are additive probes, so the result is a ranking
of mechanisms by the walk inflation each reproduces, not a single label.

**How a CPU saving maps to throughput:**
- One cycle a row at w4 is ≈ 0.20 CPU-s/GB.
- Removing the whole walk penalty would take the decoder from 3.58 to ≈ 3.01 CPU-s/GB
  (−16%).
- Throughput rises by less unless the 2.8–3.15 busy cores of 4 also rise.

## 6. Protocol and runner (if authorised)

**To build, not yet authorised:** a branch `research/bwt-interference` holding:
- `examples/interference.rs`: the target and the neighbour modes;
- `bench/interference.py`: per-block counters to per-arm ratios, quality checks, the
  decision table;
- `bench/interference_run.sh` and the `just bench-interference LABEL` recipe;
- a protocol and freeze in the established form: pin, frozen sources and corpora hashes,
  the event list as probed, the calibrations Q1–Q5, and an untimed VM dry run (the VM
  lacks the PMU, so only the plumbing is checked there).

**The command,** after its own authorisation:

```
git fetch origin research/bwt-interference && git switch research/bwt-interference
just bench-interference i9-10900K
```

- **Privileges:** `sudo` once, for `perf_event_open` (paranoid 4) and the IMC.
- **Data:** the walk trial's frozen corpora.
- **Time:** ≈ 15 minutes (10 arms × 10 rounds × 4 counter groups × ≈ 0.6 s, with
  neighbour start-up).
- **Archive:** `target/interference-results/i9-10900K.tar.gz`, holding per-block
  counters, IMC per arm, the calibrations, the neighbours' own rates, and `result.json`.

## 7. Recommendation

**Authorise the implementation of option A and its dry run, not yet the capture.**
- It is the one design in which plausible outcomes choose between keeping the 4-byte table
  with less traffic, phase scheduling, a representation change, and another owner.
- It needs about 300 lines of diagnostic code and no change to the decoder.

**If a zero-code first step is preferred,** option B (the attribution runner with the
amended event list) is the fallback. It answers only whether the DRAM share rises.

**What I expect to learn,** without presuming it: the 4-worker walk penalty decomposed into
a capacity part, an LLC-queueing part, a DRAM part and a page-walk part. Each is stated in
cycles per row against the 0.57 CPU-s/GB budget. Each points at the cheapest lever able to
remove it, or rules a lever out before any engineering is spent.

**Out of scope:** dispatch (closed; wake placement held as a separate untested lead), MPMC,
the condition-variable split, prefetch hint sweeps, Precopy, the pair walk as built, compact
Ψ as built, N1, P74-C and the unrolled walk.
