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
threads. The pool is kept for the run; its threads spin briefly, then park. With
`COOP_CPUS` set, as `bench/coop.sh` sets it to one CPU of each physical core, every thread
is pinned to its own CPU (`sched_setaffinity`), and where each ran is checked. A run with a
thread seen off its CPU fails its check pass. `bench/coop.sh` runs it as `just bench-coop`:
- `block`: one block at a time on k cores, against the production inversion and libsais
  2.12.1's on k OpenMP threads;
- `grid`: N blocks in flight on k cores each, against N production workers, on the T-vector
  phase (the scanner and MTF done first, outside the timing). Its figure is the T-vector
  phase's throughput, not the decoder's: a grid gain is a candidate for a decoder A/B,
  not a decoder gain.

Every output is checked against the production walk's, byte for byte (`block`) or by hash
(`grid`).

This machine **cannot decide the question.** Its 260 MB L3 holds every live table, so
sharing can only show its costs here. Its 4 vCPUs are shared, so a 4th spinning thread is
unreliable: libsais also falls at 4 in some runs. What it validates is the harness and the
kernel's costs.

### Six negative findings on the way (each fixed in the harness)

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
5. **Blocks dealt to workers in a fixed rotation:** the grid gave worker w every Nth
   block, so a worker slowed for any reason kept its whole share. Its busiest and idlest
   block workers differed by 0.38 against 0.22 s. The blocks are now taken from one
   counter, as the decoder's workers take jobs from one queue. Busy times now match within
   1%, and p4 measured 1264 MB/s.
6. **Threads free to share a CPU, and idle pools spinning:** `taskset` limited the process
   to one CPU of each core, but not one thread to a CPU. The idle pools of other arms also
   spun for about 3 ms after each job. With threads pinned and the spin cut to about
   2,000 iterations before parking, `coop 4` went from erratic (0.65–1.15× prod) to 1.67×.
   `coop 1`'s walk went from 6.6 to 4.0 ns/B, where it had been slowed by a spinning
   neighbour (the VM's vCPUs may share physical cores).

### The harness's figures (`coop-20261028T144113Z.md`: threads pinned, blocks from one counter; indicative)

| one block on k cores (3 passes) | build ns/B | walk ns/B | vs prod | vs its 1 thread | busiest thread's bytes / mean | last start, end spread (µs a block) |
|---|--:|--:|--:|--:|--:|--:|
| prod (room walk, 1 core) | 1.45 | 3.51 | 1.00 | | | |
| coop 1 | 1.51 | 3.99 | 0.90 | 1.00 | | |
| coop 2 | 1.93 | 2.54 | 1.11 | 1.23 | 1.25 | 37, 28 |
| coop 3 | 2.26 | 1.93 | 1.18 | 1.31 | 1.56 | 46, 42 |
| coop 4 | 1.27 | 1.69 | 1.67 | 1.86 | 1.85 | 75, 48 |
| coop 1 with coop 2's, 3's, 4's segments | 1.50 | 4.11 / 4.21 / 4.34 | 0.88 / 0.87 / 0.85 | 0.98 / 0.96 / 0.94 | | |
| libsais 1 / 2 / 3 / 4 | (with the walk) | 4.00 / 3.52 / 2.63 / 2.34 | 1.24 / 1.41 / 1.88 / 2.12 | 1.00 / 1.13 / 1.52 / 1.71 | | |

- **The shared walk scales:** 3.99 → 2.54 → 1.93 → 1.69 ns/B from 1 to 4 cores, 2.4× at 4.
- **The finer cut is not what buys it.** One core walking 4 cores' segments is 6% slower,
  not faster. So the gain is the cores', and slightly understated.
- **Coordination is small against a block's walk** (about 2 ms):
  - the last helper starts 37–75 µs in;
  - the threads end within 28–48 µs of each other.
- **The byte shares are uneven (1.25–1.85) but the ends are not.** Some threads are
  slower, and the shared counter gives them fewer segments, as it is meant to. On this VM
  that is a placement effect, not a tail.
- **The parallel build** is erratic here: 1.93 and 2.26 ns/B at 2 and 3 cores, 1.27 at 4,
  against 1.51 on one. It is still the first thing to fix.
- **The cooperative kernel costs 10–14% on one core**, as its walk loop lacks the room
  walk's passes.

Grid (6 rounds; the paired mean with its 95% interval):

| grid: N blocks × k cores | MB/s | vs p4, paired | walk vs p4, paired |
|---|--:|--:|--:|
| p4 (4 production workers) | 1264 | 1.00 | 1.00 |
| p2 | 599 | 0.51 ±0.05 | 0.51 |
| 4x1 (the coop kernel, 4 blocks) | 942 | 0.79 ±0.10 | 0.80 |
| 2x2 (2 blocks, 4 cores) | 641 | 0.53 ±0.05 | 0.63 |
| w2x2 (the same, built on one core) | 629 | 0.52 ±0.06 | 0.60 |
| 2x1@1024 (2 blocks, 1 core each, 2x2's segments) | 506 | 0.42 ±0.04 | 0.41 |
| s2x2 (libsais, 2 blocks × 2 threads) | 655 | 0.55 ±0.05 | (its build is in its walk) |

With no capacity pressure, independent blocks win, as expected. This is the control for
the workstation run: there, p4 is the peak and p6–p10 fall.

## 5. The deciding test, pre-registered (the workstation, `just bench-coop`)

Run on the quiet i9, with nothing else open (§1 of the workstation entry). It takes about
5–10 minutes. Every thread is pinned to its own physical core and checked.

**The discovery grid's arms (by default):**
- production: p2, p4, p6, p8, p10;
- the cooperative kernel: 4x1, its cost at today's footprint;
- the candidates: 4x2, 5x2, 3x3;
- the same walks with the table built on one core: w4x2, w5x2, w3x3;
- the equal-segment controls, one core a block with 4x2's, 5x2's and 3x3's segments:
  4x1@1024, 5x1@1024, 3x1@1536;
- with libsais: s4x1, s4x2, s5x2, s3x3.

There are 2 × arms rounds, rotated and reversed, paired against p4.

**What each arm reports:**
- its throughput;
- its build and walk as phase throughputs;
- the walk's busiest thread's share and its coordination (the helpers' start lag, the
  threads' end spread);
- the rounds where any thread was seen off its CPU.

The raw JSONL also keeps each round's busiest and idlest block worker, and the wall left
unaccounted.

**Predictions if capacity is the knee's mechanism:**
- p6–p10 below p4, reproducing the knee;
- 4x1 at about 0.8–0.9 of p4, the kernel's cost;
- **4x2 above p8**, at equal cores and half the live tables;
- **4x2 at least 1.1 × p4**;
- 4x2 above 4x1@1024, so the second core buys it and the finer cut doesn't;
- 5x2 at or above 4x2;
- the libsais arms at or above ours.

**The decisive comparison is 4x2 (and its best neighbour) against p4,** today's best. 4x2
against p8 says whether sharing relieves the contention; only beating p4 says the peak
moved.

**Attributing a miss before applying the rule.** The parallel build is known not to scale
(§4), so a 4x2 that loses doesn't by itself falsify the shared table:
- if w4x2 (the build on one core) beats 4x2, the parallel build is losing it;
- if the walk's phase throughput at 4x2 beats p4's while the arm's total doesn't, the walk
  works and the build or the serial work around it is the bottleneck;
- if the coordination columns are a large share of a block's walk (they are about 3% on
  the VM), the handoffs are.

Each of those makes the build or the handoffs the next step, not the idea.

**What falsifies it as a lever:**
- 4x2 and w4x2 both ≤ p4 in every round, **and** the walk's own phase throughput at 4x2 no
  better than p4's, means that sharing a table across cores costs more than the footprint
  saves, even under pressure;
- then cooperative inversion closes, and the footprint lever passes to the table's
  representation (`2026-10-28-workstation-scaling.md`, "Left open") or to nothing.

**Two stages.** The discovery grid only nominates. With about 19 arms, the best of them is
biased upwards. A candidate is then confirmed alone against p4:

    just bench-coop --arms p4,CANDIDATE --rounds 20

That is 20 balanced rounds, with the paired mean and its 95% interval.

**What promotes it to the decoder:**
1. the confirmation's paired mean is at least 1.10 × p4, its 95% interval's lower bound
   is above 1.0, and it beats p4 in every round;
2. then, before integration:
   - the kernel reaches parity, 4x1 at least 0.97 × p4;
   - the build is fixed;
   - the room walk's shared-table form passes the segment oracle, fuzzing and Miri on its
     new ownership;
3. and the decoder itself, with the scanner, MTF, RLE1, the CRC and the ordered output,
   is A/B'd on the workstation at 4–10 cores. A T-vector phase gain counts only as far as
   it survives there.

Four-core non-inferiority holds by construction (§1), and is checked on CI's 2-core
runners as for any walk change.

**If it wins, measure why before building on it.** Cooperative inversion can raise
throughput in two ways:
- fewer misses, with 4 live tables in place of 8 and the L3 hit share held at p4's;
- more misses in flight per table, with 8 cores' fill buffers overlapping one table's
  latency.

The two may coexist, and they call for different designs:
- the first, a scheduler that budgets live tables by the L3 (§1);
- the second, more lanes per block, however they are spread.

So a winning arm is first profiled against p4 and p8 with the existing counters
(`just bench-imc`'s L3 hit share, misses per KB and occupancy per miss) before any
scheduler is built on either reading.

## 5b. Conditional leads (recorded before the run; none changes it)

From a second outside literature pass (2026-10-28), each checked against its primary
source here. Each says when it would be pursued. None adds an arm to the registered run.

1. **The pair walk, under pressure.**
   - **Source** (libsais 2.12.1 at c284ba8, `libsais.c`): the cyclic inverse that bzip2
     needs, `libsais_unbwt_cyclic` → `libsais_unbwt_main` → `libsais_unbwt_core`, does use
     the bigram (ψ²) machinery. Its per-row table is still one 32-bit word a row (`P`,
     n + 1 words), the same 4 B/row as our T-vector. Each dependent `P[p]` load yields two
     output bytes (`libsais_unbwt_decode_1`, L11689: the bigram recovered through
     `fastbits[p >> shift]` and a short search of `bucket2`). Its extra footprint is
     shared tables, not rows: `bucket2` (65,536 × 4 B = 256 KB) and `fastbits` (2¹⁷ × 2 B
     ≈ 256 KB). So the worry that "a bigger two-symbol table undoes the cache gain" doesn't
     apply to libsais's design. The table is the same size, and the dependent misses a
     decoded byte halve.
   - **Our own history** (`2026-10-27-pair-walk.md`): our ψ² port was rejected at +40.6%
     (1 thread) and +24.3% (4 threads), on CI's 2-core runners. The stated reason was
     that "our walk is bound by instructions per step, not by memory latency… so halving
     the dependent loads buys almost nothing". That holds below the knee. Past it, the
     workstation measured the walk memory-bound (IPC 1.06 → 0.50, L3 hit share
     0.76 → 0.38 from 4 to 10 workers). That is the regime where halving misses a byte
     could pay, and the pair walk was never measured there.
   - **Reopening condition (hypothesis):** the pair walk at 6–10 independent workers on the
     workstation. It needs no code: libsais's own pair walk is in this harness. After the
     registered run, `bash bench/coop.sh --arms p4,p8,p10,s4x1,s8x1,s10x1 --rounds 12`
     asks whether libsais at 8–10 workers holds up where mibz2 falls. If it does,
     `perf/pair-walk` (kept) is re-measured at 8–10 workers. Under the journal's rule, a
     closed idea reopens only on a named condition; this is that condition, named now.
2. **Work-optimal parallel decompression exists in theory; its practice is unshown on
   commodity cores.**
   - **Source:** Edwards & Vishkin, "Parallel algorithms for Burrows–Wheeler compression
     and decompression": the UMD technical report (2012) and the *Theoretical Computer
     Science* article (doi:10.1016/j.tcs.2013.10.009; the DOI resolves to TCS).
     Decompression is O(log n) depth and O(n) work, via list ranking and prefix sums.
   - Their empirical companion ("Empirical speedup study of truly parallel data
     compression", UMD, 2013) measured 13× over bzip2 for decompression on the XMT
     prototype only. Its claim about commercial hardware is an inference ("70x
     improvement over what current commercial parallel hardware can achieve"), not a
     measurement.
   - **Bearing:** depth is not the problem here. The 32 lanes and the segments already give
     ample parallelism. What decides is the misses, traffic and synchronisation per
     decoded byte that the parallel form adds, which is what §5's gates price.
3. **Practical list ranking.**
   - **Source:** Bader-Research/ListRanking (README). It is Helman–JáJá over Reid-Miller and
     Blelloch's sparse ruling sets, "scalable performance up to 16 processors" despite
     contention for memory, and on SMPs "performance is primarily a function of
     non-contiguous memory accesses".
   - The README does not say how sublists are chosen or work divided; the source would.
   - **Bearing (hypothesis):** our segments are their sublists. Our dynamic counter is one
     shared cache line, touched once a segment (about 1,760 bytes walked), which is far
     from the per-byte sharing of negative finding 2.
   - **Pursued only if** a winning or near-winning arm shows a large coordination share
     (§4's columns, about 3% of a block's walk on the VM). Then the test is batched
     dispatch (b segments a fetch) at fixed cores, tables and bytes.
4. **Read-only sharing against shared writes.** Once flagged, the shared T-vector is only
   read during the walk, so it needs no line ownership to move between cores. The
   writable shared state is:
   - the segment counter;
   - each segment's buffer header, which is written at take, grow and finish only;
   - the successor array;
   - the per-thread share and time slots, written once a block.

   If coordination is high on the workstation, `perf c2c` on the winning arm says which
   line it is.

## 6. Decisions

- wikidata-pq keeps its cap, `min(physical cores, 4)` (lmmx/wikidata-pq#212). Cooperative
  inversion isn't expected to help its split (premonition 1).
- The cooperative pilot's verdict is the workstation's.
- No decoder change until it passes §5.
