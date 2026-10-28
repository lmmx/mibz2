# Cooperative inversion: several cores on one block's T-vector (#39, #65)

The workstation entry (`2026-10-28-workstation-scaling.md`) located the knee. mibz2 peaks
at 4 workers on the 10-core i9 and falls past them. The cause is the live T-vectors
outgrowing the L3: at the knee, 4 × 3.6 MB ≈ 0.7 of its 20 MB. The cure that entry left
open was "fewer bytes live per worker in the T-vector phase".

This entry asks whether the machine's other cores can be put to work without adding
T-vectors. The idea is to keep N blocks in flight, as now, but invert each on k cores that
share its one table, so that N × k cores keep only N tables live. It also records a
correction, the expert precedents, the premonitions, and a first pilot. The pilot ran on a
machine that can't decide the question. The workstation's pre-registered test can.

Labels as in the other entries: **measured** (a figure from a run, said where);
**source** (a primary source, cited); **hypothesis** (ours, untested).

## 1. What the question is

The owner's framing (2026-10-28), in four parts:

1. Pick the thread count from the cores, but never by giving up a 4-core machine's
   performance.
2. How would one measure cache working sets?
3. Establish what measurement would decide the hypotheses, on CI if possible.
4. Is the knee a readout of the algorithm's design rather than of the machine? Should the
   decoder be redesigned so that it saturates, and holds at least its peak, out to every
   core? That would make "raise the peak" and "ease the knee to the right" two
   hill-climbable targets.

### Answers from what is already measured

- **4:** the knee is not a point where mibz2 does worse than other decoders.
  - **measured** (workstation-scaling §5): at 20 threads mibz2 still delivers 486 MB/s,
    above lbzip2's best, 416.
  - lbzip2 "scales" to 8 because each of its workers makes demands at about a quarter of
    mibz2's rate a worker (80 against 352 MB/s at 2 threads). It reaches the memory
    system's limit later, and lower.
  - What multiplies with cores is the live footprint, not the per-worker cost. Today the
    decoder parallelises only across blocks, one block a worker, so N workers mean N
    T-vectors.
  - That design decision is the one to revisit. The owner's reading is right in spirit:
    the knee is where this decoder's shape meets the L3.
- **2:** the working set has been measured three independent ways:
  - the ledger (`docs/ledger.md`);
  - the counters' hit share, which fits C/W, with an effective 6–7 MB for the walk (§7
    there);
  - the block-size intervention (§9 there): at -9 and -6 the knee sits at the same
    aggregate live T-vector, about 14.4 MB ≈ 0.7 × L3.

  A causal dose-response is stronger evidence than any occupancy counter. Locating the
  knee needs more cores than the budget holds T-vectors for. CI's 2-core runners have
  too few, so the knee's position is a workstation measurement. What CI can carry is the
  early-warning signal: CPU a byte against the arm's own lower thread count, with a
  control decoder (`2026-10-28-early-warning.md`).
- **1:** a scheduler that chooses N blocks in flight and k cores a block can give each
  machine size its own shape:
  - N × k = the physical cores;
  - N ≤ the footprint budget (≈ 0.7 × L3 / T-vector);
  - k = 1 whenever the cores fit the budget.

  On a machine whose cores fit the budget, that is today's decoder exactly, so there is
  no 4-core trade-off by construction. Sharing begins only where today's decoder would
  pass the knee.

### A correction to an external reading

An outside summary of the wikidata-pq release-shaped run (2026-10-28, 4 against 20
decoder threads) said it measured "the native lbzip2 decoder, not mibz2". It measured
mibz2:
- every run's recorded config has `bz2_decoder: native` and the requested
  `bz2_threads`;
- the sweep (wikidata-pq `bench/bz2threads.py`) exits unless the split's own log names
  the native decoder at the requested count.

The summary line it read, "native (lbzip2's command: …)", prints the fallback's command
beside the decoder actually used.

## 2. Expert materials (source)

| work | what it shows | bearing here |
|---|---|---|
| libsais 2.12 (Grebnov), `libsais_unbwt_cyclic_omp` ([header](https://github.com/IlyaGrebnov/libsais/blob/c284ba843dd16c159acb976f17bc19382793ea75/include/libsais.h)) | One bzip2-style cyclic BWT inverted on many threads from only the primary index. It uses marked rows, chains decoded between marks into chunks, then stitched (research notes, §4). | A shipping, practitioner-grade cooperative inversion. Its single-thread kernel was measured on our blocks before (research synthesis, finding 1), its multi-threaded one never. |
| Kärkkäinen & Puglisi 2011; Kärkkäinen, Kempa & Puglisi, DCC 2012 | Both name reconstructing the text "at several places simultaneously" as parallel future work, then interleave the places on one thread instead. | Our 32 lanes are the one-thread form. The cooperative walk is the step they left. |
| Helman & JáJá, "Designing practical efficient algorithms for symmetric multiprocessors" (ALENEX 1999; [walkthrough](https://dev.to/downey/the-helman-and-jaja-list-ranking-algorithm-4486)) | List ranking on P processors from random sublist heads, each sublist walked by one processor. O(n/P) span, with load imbalance from random splitters. | The inverse BWT is ranking one linked list. Our flagged segments are their sublists, and their imbalance warning predicts our tail (§4). |
| Chen, Gibbons, Kozuch et al., "Scheduling threads for constructive cache sharing on CMPs" (SPAA 2007; [record](https://infoscience.epfl.ch/handle/20.500.14299/36961)) | On a shared cache, a scheduler that keeps cores on nearby data (PDF) beats one that keeps them on disjoint data (work stealing) by 1.3–1.6×. | One block a worker is the disjoint-data schedule. k cores on one block is the constructive one. |
| Suleman, Qureshi & Patt, "Feedback-driven threading" (ASPLOS 2008; [pdf](https://utw10235.utweb.utexas.edu/people/suleman/papers/fdt.pdf)) | Bandwidth-aware threading picks the fewest threads that saturate the bus, P = 100 / BU₁ from one thread's bus utilisation, read from counters during a short training loop. | The thread-throttling view behind #65's cap. It prices bandwidth, not capacity, so for this knee it is a bound to check, not the model. |
| Xiang, Ding et al., HOTL (ASPLOS 2013), already in `2026-10-28-early-warning/research-notes.md` | Footprints compose across co-runners; misses don't. | Why live T-vectors add, and why the knee sits at an aggregate footprint. |

## 3. Premonitions: what the next steps will run into (hypothesis unless marked)

1. **For wikidata-pq, cooperative decoding is unlikely to speed the split.**
   - **measured** (wikidata-pq `docs/journal/2026-10-28-bz2-threads.md`, workstation, the
     same slice and counts): the decoder alone at 4 threads takes 1.20 s, while the split
     at 4 takes 4.24 s. The split is bound by its 18 parse workers, not the decode.
   - Its 24% gain from 4 threads came from the CPU the decoder stopped wasting.
   - So the lever there is decode CPU a byte, and wikidata-pq's cap is the right shape.
     Cooperative inversion is a lever for mibz2's own throughput (bzcat-like use on
     many cores), and must not be pushed into wikidata-pq on mibz2 figures alone.
2. **Kernel parity comes before any scheduling gain counts.** The cooperative kernel,
   on one core, must be within a few percent of the production room walk and build.
   Otherwise a gain is overstated against a weak baseline, or eaten by its own cost (§4).
3. **Per-block serial work grows with k:**
   - flagging and clearing the starts (random writes into the table);
   - sorting k × 512 starts;
   - the join (a 900 kB copy);
   - two pool synchronisations a build and one a walk.

   At about 3 ms a block, 100 µs of serial work is 3%, and Amdahl caps k well before the
   cores do.
4. **The parallel build trades local writes for cross-core ones.** A row's link lands in
   its byte's bucket, so most of a thread's writes hit lines another core initialised.
   Measured below: the build doesn't scale on the VM.
5. **Bandwidth is not ruled out as the next ceiling** (arithmetic on
   `imc-20261027T234235Z`, `imc-20261028T013015Z`):
   - DRAM reads are 0.4 B a decoded byte at 1 worker, and 20 at 6 (16.8 GiB/s);
   - demand reads missing the L3 are 10.2 a KB at 4 workers and 27.6 at 6, so about 7 B
     a byte at 4.

   At 4 live blocks and twice the throughput, reads would be about 14 GiB/s plus the
   writes. That is under the 26–27 GiB/s total measured at the plateau, so a 2× isn't
   excluded by bandwidth, but it is not far below it.
6. **Never two cooperating threads on SMT siblings.** Siblings cost about 1.43× CPU a
   block (`2026-10-27-four-thread-ceiling.md`). The pilot runs on one CPU of each core.
7. **The budget is per L3 domain.** On multi-CCD AMD parts each CCD has its own L3, so a
   policy must read `shared_cpu_list`, not the socket's total.

## 4. The pilot (measured, a 4-vCPU cloud VM: Xeon @ 2.1 GHz, 260 MB L3, no SMT shown)

`examples/coop.rs` (research only) builds and walks a block's table on a pool of k
threads. The pool is kept for the run; its threads spin briefly, then park. `bench/coop.sh`
runs it as `just bench-coop`:
- `block`: one block at a time on k cores, against the production inversion and libsais
  2.12.1's on k OpenMP threads;
- `grid`: N blocks in flight on k cores each, against N production workers, on the T-vector
  phase (the scanner and MTF done first, outside the timing).

Every output is checked against the production walk's, byte for byte (`block`) or by hash
(`grid`).

This machine **cannot decide the question.** Its 260 MB L3 holds every live table, so
sharing can only show its costs here. Its 4 vCPUs are shared, so a 4th spinning thread is
unreliable: libsais also falls at 4 in some runs. What it validates is the harness and the
kernel's costs.

### Four negative findings on the way (each fixed in the harness)

1. **A thread spawned a block, with fresh buffers:** 2 cores gave 1.05×, slower than 1
   when pinned to 2 CPUs. The spawn, the cross-thread frees and the page faults cost more
   than the walk gained.
2. **Lanes pushing through `Vec` headers:** with a persistent pool, 2 cores gave 0.47×.
   Neighbouring segments' headers share cache lines, so every byte written moved a line
   between cores. Production writes through its own pointers, and so must any shared walk.
3. **512 segments for k × 32 lanes:** 4 cores gave 1.08×. With only about 4 segments a
   lane, the walk waits on its tail's longest chains, as Helman–JáJá warn. At 16 a lane,
   the room walk's density, it scales.
4. **The grid's output check inside its timing:** each block worker hashed its outputs as
   it went. An arm with fewer blocks in flight did that hashing on fewer threads, which
   penalised exactly the cooperative arms. The first grid figures (p4 647 MB/s on the VM)
   were wrong by that much. The check now runs in an untimed pass over every arm before
   the timed rounds (p4: 931 MB/s).

### The harness's figures (block: `coop-20261028T120750Z.md`, 3 passes; indicative)

| one block on k cores | build ns/B | walk ns/B | vs prod |
|---|--:|--:|--:|
| prod (room walk, 1 core) | 1.27 | 3.38 | 1.00 |
| coop 1 | 1.30 | 4.03 | 0.87 |
| coop 2 | 2.11 | 2.37 | 1.04 |
| coop 3 | 2.08 | 2.02 | 1.13 |
| libsais 1 / 3 / 4 | (with the walk) | 3.81 / 2.39 / 2.16 | 1.22 / 1.94 / 2.15 |

- **The shared walk scales:** 4.03 → 2.37 → 2.02 ns/B at 1 → 2 → 3 cores, 1.7–2.0×.
- **The parallel build doesn't:** 1.30 → 2.1 ns/B, worse at 2 cores than 1. This fits
  premonition 4, and it is the first thing to fix (build on one core while the others
  start the walk's setup, or a bucket-major split).
- **The cooperative kernel costs 13% on one core** (its walk loop lacks the room walk's
  passes).
- **libsais's multi-threaded inversion reaches 1.76× its own one thread at 4.** A
  practitioner's version of the same idea scales on this machine.

Grid, `coop-20261028T130747Z.md` (4 rounds, the check out of the timing; indicative):

| grid: N blocks × k cores | MB/s | vs p4, paired | walk vs p4, paired |
|---|--:|--:|--:|
| p4 (4 production workers) | 931 | 1.00 | 1.00 |
| p2 | 490 | 0.53 | 0.54 |
| 4x1 (the coop kernel, 4 blocks) | 796 | 0.85 | 0.85 |
| 2x2 (2 blocks, 4 cores) | 532 | 0.58 | 0.64 |
| w2x2 (the same, built on one core) | 497 | 0.51 | 0.59 |
| s2x2 (libsais, 2 blocks × 2 threads) | 562 | 0.59 | (its build is in its walk) |

With no capacity pressure, independent blocks win, as expected. This is the control for
the workstation run: there, p4 is the peak and p6–p10 fall.

## 5. The deciding test, pre-registered (the workstation, `just bench-coop`)

Run on the quiet i9, with nothing else open (§1 of the workstation entry). It takes a
few minutes. The grid's arms by default are p2, p4, p6, p8, p10, 4x1, 2x2, 3x2, 4x2, 5x2,
3x3 and 2x4; w4x2, w5x2 and w3x3, the same walks with the table built on one core; and
with libsais, also s4x1, s4x2, s5x2 and s3x3. There are 2 × arms rounds, rotated and
reversed, paired against p4. Each arm's build and walk are reported apart, as phase
throughputs, and every round's figures are kept raw (`coop-<UTC>.jsonl`).

**Predictions if capacity is the knee's mechanism:**
- p6–p10 below p4, reproducing the knee;
- 4x1 at about 0.85–0.9 of p4, the kernel's cost;
- **4x2 above p8**, at equal cores and half the live tables;
- **4x2 at least 1.1 × p4**;
- 5x2 at or above 4x2;
- the libsais arms at or above ours.

**The decisive comparison is 4x2 (and its best neighbour) against p4,** today's best. 4x2
against p8 says whether sharing relieves the contention; only beating p4 says the peak
moved.

**Attributing a miss before applying the rule.** The parallel build is known not to scale
(§4), so a 4x2 that loses doesn't by itself falsify the shared table:
- if w4x2 (the build on one core) beats 4x2, the parallel build is losing it;
- if the walk's phase throughput at 4x2 beats p4's while the arm's total doesn't, the walk
  works and the build or the serial work around it is the bottleneck.

Either way, the next step is the build, not the idea.

**What falsifies it as a lever:**
- 4x2 and w4x2 both ≤ p4 in every round, **and** the walk's own phase throughput at 4x2 no
  better than p4's, means that sharing a table across cores costs more than the footprint
  saves, even under pressure;
- then cooperative inversion closes, and the footprint lever passes to the table's
  representation (`2026-10-28-workstation-scaling.md`, "Left open") or to nothing.

**What promotes it to the decoder:** the best cooperative arm beats p4 by at least 10%,
paired, in every round. Even then, before integration:
- the kernel reaches parity, 4x1 at least 0.97 × p4;
- the build is fixed;
- the room walk's shared-table form passes the segment oracle, fuzzing and Miri on its
  new ownership.

Four-core non-inferiority holds by construction (§1), and is checked on CI's 2-core
runners as for any walk change.

## 6. Decisions

- wikidata-pq keeps its cap, `min(physical cores, 4)` (lmmx/wikidata-pq#212). Cooperative
  inversion isn't expected to help its split (premonition 1).
- The cooperative pilot's verdict is the workstation's.
- No decoder change until it passes §5.
