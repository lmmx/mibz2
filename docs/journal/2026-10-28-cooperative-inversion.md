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

## 5c. A third pass: what was already known, what's new (recorded before the run)

Another outside literature pass (2026-10-28). Most of it is already in this journal; the
new parts are a footprint measured from source and one more lead of the pair walk's
shape.

**Already recorded:**
- **Weißenberger & Schmidt, ICPP 2024** (doi:10.1145/3673038.3673067). Their GPU inverse
  BWT for standard bzip2 streams was read at 66d263c (`2026-10-27-research-synthesis.md`,
  finding 5): Helman–JáJá list ranking from splitters, then a positional scatter. It was
  recorded as "nothing to transfer to the CPU decoder".
- **KKP 2012 §3's independently decoded parts** assume boundary positions carried with the
  input (auxiliary indices from the forward transform). A bzip2 stream carries only
  `orig_ptr`, so mibz2 finds its segments itself, from pseudo-random flagged starts, and
  joins them by successor links (finding 7 there). libsais's chain decode does the same
  from hashed mark rows.
- **KKP's Precopy** was tried, as #9 (`2026-10-27-precopy-forward-chains.md`). It copies
  31.5% of Wikidata's bytes, so whether the corpus is repetitive enough is already
  measured, not inferred from repetitive test sets.

**New, from source** (`weissenberger/bzip2gpu`, `src/ibwt.cu` on main, 2026-10-28):
- **Splitters:** at a fixed stride, every `LIST_DISTANCE` = 32 rows (`head = tid *
  list_distance`, with the primary index overriding one). A sublist ends at the next
  multiple of 32 it reaches. Their mean length is 32, with no bound on a single one shown.
- **Output:** each byte is scattered to `ranks[i] + reduced_ranks[sublist]`. The local
  rank comes from `helman_jaja_local`, the sublist offsets from a prefix sum in
  `helman_jaja_reduce`.
- **Footprint a row:**
  - 2 B of BWT byte copies (a sort double buffer);
  - 8 B of permutation (`perm`, `perm_alt`, the latter reused as the ranks);
  - 4 B of `sublist_ids`;
  - about 0.25 B of reduced lists and ranks.

  That is about 14 B a row, 12.8 MB for a 900 kB block, against our 4 B a row.
- **Bearing (hypothesis):** three ways exist to pay for independent inversion work on one
  block:
  1. boundary indices carried in the input, which bzip2 lacks;
  2. discover segments and join them (mibz2, libsais);
  3. rank every element and scatter (the GPU decoder).

  On the i9, the third's 3.5× footprint would put about one block in the L3 budget that
  holds four of ours. It is a reference for the output-assembly question only: pursued
  only if a cooperative arm's join (`SegScratch::assemble`'s copy) shows up as a cost.

**New conditional lead: Precopy under pressure.** Precopy turns runs of random T-vector
reads into sequential copies, which is a cut in misses a byte, like the pair walk. It was
rejected on CI's 2-core runners, at +18.0% (1 thread) and +6.0% (4 threads) on Wikidata.
Its penalty shrank as the threads rose. That fits a cost paid in instructions and a saving
paid in misses, and the saving grows past the knee (hypothesis).
- **Reopening condition:** `perf/precopy-chains` (kept) measured at 8–10 workers on the
  workstation, after the registered run and the pair walk's lead. A miss-cutting change
  that loses below the knee can win past it.
- The scaling shape and the absolute figure are reported separately, as for the pair walk.

**Not verified here:** Sibeyn, Guillaume & Seidel's "Practical parallel list ranking"
(1999), cited by the outside pass as showing that list ranking struggles to beat the
sequential walk. It is plausible and consistent with §4's negative findings, but it was
not read.

## 6. Decisions

- wikidata-pq keeps its cap, `min(physical cores, 4)` (lmmx/wikidata-pq#212). Cooperative
  inversion isn't expected to help its split (premonition 1).
- The cooperative pilot's verdict is the workstation's.
- No decoder change until it passes §5.
- **2026-10-28, after the discovery run (§7):** the 4x2 gate passes in discovery; 3x3 is
  nominated beside it. Both are confirmed alone against p4 (§8.1) before anything else. The
  pilot's code stays frozen at c2620b8 for the confirmation, and production is untouched.

## 7. The workstation's discovery run (measured, c2620b8, 2026-10-28T23:39:45Z)

The raw report and its rounds are kept in
[`2026-10-28-cooperative-inversion/results/`](2026-10-28-cooperative-inversion/results/).
The analysis below is [`analyse.py`](2026-10-28-cooperative-inversion/analyse.py) run on
the JSONL. The discovery grid nominates; it doesn't confirm (§5, "Two stages").

**The run:**
- the i9-10900K, CPUs 0–9 (one a physical core), load 1.51 at the start;
- 19 arms × 38 rounds = 722 records, none missing;
- every thread was seen on its CPU in every round (0 misplaced);
- every arm's every block passed its hash in the untimed check round;
- the T-vector phase only (build + walk) on the 431 Wikidata blocks (209.7 MB of BWT).

### 7.1 The gate and the nominee

Paired against p4 in the same round. Geometric means of the ratio, the t-interval, and a
block bootstrap (blocks of 4 rounds) as a check on serial correlation:

| arm | cores | live tables | vs p4 | 95% (t) | 95% (block bootstrap) | worst round | rounds > 1.10 |
|---|--:|--:|--:|--:|--:|--:|--:|
| 4x2 (pre-registered) | 8 | 4 | 1.226 | 1.214–1.238 | 1.211–1.239 | 1.129 | 38/38 |
| 3x3 (best of ours) | 9 | 3 | 1.354 | 1.344–1.364 | 1.342–1.367 | 1.277 | 38/38 |
| s3x3 (libsais, reference) | 9 | 3 | 1.629 | 1.581–1.679 | 1.574–1.681 | 1.022 | 37/38 |
| s4x1 (libsais, 4 independent) | 4 | 4 | 1.083 | 1.074–1.093 | 1.071–1.096 | 0.999 | 10/38 |
| 5x2 | 10 | 5 | 1.046 | 1.033–1.058 | 1.032–1.060 | 0.962 | |
| p8 | 8 | 8 | 0.469 | 0.465–0.474 | | 0.436 | |

In MB/s (medians): p4 1412, 4x2 1736, 3x3 1912, s3x3 2356, p8 662, p10 561.

- **The pre-registered gate's discovery stage passes:** 4x2 is 1.23 × p4, above 1.10 in
  every round. That is above p8 (×2.61 at equal cores) and above p4: the peak moved.
- **3x3 is better again:** 1.104 × 4x2 paired (1.096–1.113), beating it in 38/38 rounds.
  Being the best of about 19 arms, it's biased upwards, so it is confirmed alone (§8.1).

**Predictions (§5):**
- **Met:**
  - p6–p10 below p4 (0.71, 0.47, 0.40): the knee reproduced;
  - 4x1 at 0.865, inside 0.8–0.9;
  - 4x2 above p8, and at least 1.1 × p4;
  - 4x2 above 4x1@1024, by ×1.445: the second core buys it, the finer cut doesn't (4x1@1024
    is 0.98 of 4x1);
  - w4x2 below 4x2 (×1.08), so the parallel build helps here.
- **Failed:** 5x2 at or above 4x2. It's 0.853 of it. Five tables (18 MB) is 0.9 of the L3.
- **Mixed:** the libsais arms at or above ours. s3x3 and s4x1 are above; s4x2 (1.155)
  and s5x2 (0.912) are below 4x2 and 5x2.

### 7.2 Run order, drift and correlation

- **Drift is in the denominator.** p4's median fell 2.1% from the first half of the run to
  the second (1423 → 1394 MB/s). 3x3's held (1912 → 1910), and 4x2's rose 1.2%. So the
  ratios' rise between halves (4x2 1.206 → 1.247, 3x3 1.340 → 1.368, s3x3 1.578 → 1.682)
  is mostly p4 slowing. Every arm's ratio has a small upward trend, +0.2–1.5% per 10 rounds
  (s3x3 +3.0%). That's p4 drifting, not the cooperative arms improving. Its cause isn't
  measured; thermal or turbo budget is a hypothesis.
- **Little serial correlation.** The paired log-ratios' lag-1 autocorrelation is −0.24 to
  +0.18. The block bootstrap's and an AR(1)-widened interval differ from the t-interval
  by at most 0.003 for ours, 0.01 for s3x3.
- **No position effect.** Regressing each ratio on the arm's slot relative to p4's in the
  round order gives at most 0.0026 in log per slot.
- The drift is an order of magnitude below the gains. A two-arm confirmation's order
  (AB, BA, BA, AB, ...) cancels a linear drift every 4 rounds.

### 7.3 Where 3x3's time goes: the walk wins, the build is now 42%

The phase throughputs add as times (1/build + 1/walk reproduces each arm's total within
0.5%). Each phase is MB/s of output over the wall spent in it:

| arm | build MB/s | walk MB/s | walk vs p4's walk | build share of the time |
|---|--:|--:|--:|--:|
| p4 | 5390 | 1927 | 1.00 | 26% |
| 4x2 | 4148 | 3007 | 1.56 | 42% |
| w4x2 | 3627 | 2905 | 1.51 | 44% |
| 3x3 | 4511 | 3332 | 1.73 | 42% |
| w3x3 | 2914 | 3092 | 1.61 | 51% |
| 5x2 | 3734 | 2452 | 1.28 | 40% |

- **The shared walk is the win.** At 3x3 it runs at 1.73 × p4's walk.
- **The parallel build helps, but is the next limit.** 3x3's build, on 9 cores, is 0.84 of
  p4's on 4. Building on one core (w3x3) costs 1.28× in total. If 3x3's build ran at p4's
  5390 MB/s, 3x3 would be about 2060 MB/s, 1.46 × p4. That's a model, from the additive
  times above.
- **Coordination stays small.** The helpers' last start plus the threads' end spread is
  about 4% of a block's walk at 4x2 and 3x3 (as on the VM). It's 10% and 20% in the w
  arms, whose helpers wait out the serial build.
- **Balance.** The busiest thread's bytes over the mean are 1.31 (4x2) and 1.36 (3x3). The
  imbalance is real but bounded by the 512 segments a thread; it costs less than the
  build.

### 7.4 What the grid says about the mechanism, before counters

Three contrasts at a fixed footprint or a fixed core count:
- **Footprint fixed at 4 tables, cores 4 → 8:** 4x1 → 4x2 is ×1.42.
- **Cores fixed at 8, tables 8 → 4:** p8 → 4x2 is ×2.61.
- **Neighbours:**
  - 4x2 → 3x3 (one more core, one fewer table) is ×1.10;
  - 4x2 → 5x2 (two more cores, one more table) is ×0.85.

So above about 4 live tables (0.7 of the L3) the footprint dominates the core count. Below
it, extra cores on the same tables pay. That's what the capacity reading predicts.

It doesn't separate §5's two mechanisms, fewer misses from residency against more misses
overlapped. The ×1.42 at a fixed footprint is consistent with either, so the counters
decide (§8.2).

### 7.5 libsais: the gap is its kernel, not its cooperation

- **Same lift from cooperating:**
  - over each implementation's own 4 independent workers, 3x3 / 4x1 = 1.565 and
    s3x3 / s4x1 = 1.504;
  - cooperating lifts the two by about the same factor.
- **Different kernels:**
  - libsais's kernel at 4 independent workers is 1.083 × production's (s4x1);
  - our cooperative kernel is 0.865 × production's (4x1).
  - 1.083 / 0.865 = 1.25, which accounts for the s3x3 / 3x3 ratio of 1.20.
- **So the next gain on our side is the kernel, which the §5 promotion step already
  requires** (4x1 at least 0.97 × p4). If cooperating's lift held while the kernel reached
  parity, 3x3 would project to about 1.52 × p4. That's a hypothesis: it assumes the lift is
  multiplicative.
- **The pair walk's reopening condition (§5b.1) has partly fired.** libsais's own ψ² walk
  beats production at 4 independent workers on this machine, 37/38 rounds. That's the
  opposite sign to our port on CI's 2-core runners (+40.6% slower). It doesn't yet show
  the 8–10 worker regime that §5b.1 names. Its run is already written there.
- **s3x3's caveats:**
  - Its build is inside its walk, so there's no attribution.
  - It runs on OpenMP.
  - One round (13) fell to 1430 MB/s, with one block group finishing 13 ms before the
    others. That's a stall in one OpenMP team. Without it the worst round is 1.459.
  - Its interval is three times ours.

## 8. Next, in order (pre-registered before any of them runs)

### 8.1 Confirmation (the quiet workstation, the pilot at c2620b8)

Two separate two-arm runs, one after the other, nothing else running:

    bash bench/coop.sh --arms p4,4x2 --rounds 20
    bash bench/coop.sh --arms p4,3x3 --rounds 20

- **The rule for each, from §5:**
  - the paired geometric mean is at least 1.10;
  - the 95% interval's lower bound is above 1.0;
  - the arm beats p4 in all 20 rounds.
- **Analysis:** the same script as §7. The worktree stays detached at c2620b8, so take
  the script without moving it: `git fetch origin && git show
  origin/research/coop-unbwt:docs/journal/2026-10-28-cooperative-inversion/analyse.py >
  /tmp/analyse-coop.py`, then `python3 /tmp/analyse-coop.py coop-<time>.jsonl`. It reports the t-interval, the block bootstrap
  and the halves.
- **4x2's run decides the pre-registered hypothesis. 3x3's validates the nominee.**
- **libsais's s3x3 gets its own exploratory run only after both:**
  `bash bench/coop.sh --arms p4,s3x3 --rounds 20`, then §5b.1's `p4,p8,p10,s4x1,s8x1,s10x1`.
  Neither is part of the gate.

### 8.2 If confirmed: which mechanism (hardware counters)

- **No new kernel code.** Each arm is run alone under `perf stat`, pinned as in the grid,
  as `grid … 1 ARM` and `grid … 21 ARM`. The difference of the counts is 20 timed rounds,
  without the setup and the check round.
- **Events:** bench-session's pairs, each pair in its own pass (100% coverage), and the
  memory controller's reads and writes.
- **Arms:** p4, p8, 4x1, 4x2, 3x3.
- **Per KB of output:**
  - the demand L3 hit share;
  - L3-missing demand reads;
  - reads in flight (outstanding over cycles, summed over cores);
  - occupancy per L3-missing read;
  - DRAM bytes read and written.
- **Predictions:**
  - **Residency:** 4x2's and 3x3's hit share is at least p4's (0.76 in §8 of the workstation
    entry) and well above p8's. L3-missing reads per KB are at or below p4's. The DRAM
    write bytes per output byte return to p4's level.
  - **Overlap:** the hit share stays near p4's or lower, while the reads in flight
    machine-wide are about 2 × p4's, at no lower occupancy per miss.
  - **Both:** each reading leaves its mark, and the design budgets both the live tables
    and the lanes.
- **Guarding the difference (from the outside review):** subtracting one run from another
  is only as good as the two runs are alike.
  - Each arm's 1-round and 21-round runs are repeated 3 times, interleaved. The raw
    counts of every run are kept, not only the difference.
  - A difference counts only if its spread over the repeats is small against it.
  - Every event's time enabled must equal its time running (no multiplexing), or the
    reading is dropped.
  - The grid's own per-round wall in each run is checked against the discovery run's, so
    a counted run is a typical one.
- **What it decides:** whether the scheduler is budgeted by tables against the L3
  (residency) or by lanes per block (overlap), §5's last paragraph. The script is a small
  research addition, written after the confirmation, not before.

### 8.3 Then: the kernel and the build, still in the harness

- 4x1 to at least 0.97 × p4, the promotion's parity condition. libsais's kernel shows
  there's 25% to find (§7.5).
- The build to near p4's per-byte cost. It's 42% of 3x3's time (§7.3).
- Each measured in the grid against p4, as now.

### 8.4 Then: the decoder A/B (inversion phase ≠ end to end)

The grid measures the T-vector phase only. The decoder adds the serial scanner, MTF + RLE2,
RLE1, the CRC, segment assembly and the ordered output.

- **A model, not a measurement:**
  - the decoder at 4 workers ran at 1019 MB/s on the workstation (§8 of the workstation
    entry, an older build and session);
  - p4's T-vector phase runs at 1412 here;
  - if the rest costs the same per byte and doesn't contend, 3x3's phase would put the
    decoder at about 1.23 × its 4-worker figure, not 1.35 ×.
- **Premonitions for the integration (hypotheses):**
  - **The per-block serial stages inside a team.** MTF + RLE2 and the CRC run on one thread
    a block. In a team of 3, two cores idle through them unless the team overlaps the next
    block's MTF with this block's walk, or idle helpers take other work.
  - **Core budget.** 3x3 uses 9 of 10 physical cores, which leaves one for the scanner and
    the writer, or their SMT siblings.
  - **Keep production's path.** At 4 or fewer cores the decoder runs today's code: k = 1 is
    production's walk, not the cooperative kernel at k = 1, which is 0.865 × it. Teams start
    only above the knee.
  - **Ownership.** The shared table's handoffs need the segment oracle, fuzzing and Miri on
    their new ownership (§5, promotion step 2).
- **The A/B:**
  - **The workstation, at 4, 6, 8 and 10 cores, paired rounds.** Production at its best
    (4 workers) against the team decoder at its best. Wall of `-dc` to /dev/null and to a
    file, CPU-seconds, peak RSS, byte-identical output.
  - **CI's 2-core runners and wikidata-pq's 4-core bench, for non-inferiority.** Below the
    knee the team path is off by construction, so CI can't show the gain.
  - The gain lives only on machines with more than 4 cores, and the workstation is where
    it's measured. That's a known gap in "the CI bench is the authority", and it's stated in
    the PR when one exists.
- **No production merge** until the counters (§8.2) say why and the decoder A/B says how
  much survives.

## 9. The confirmation (measured, c2620b8, 2026-10-29, the quiet i9)

Two separate two-arm runs, one after the other, as §8.1 registered them:
- `coop-20261029T081457Z`: p4 against 4x2;
- `coop-20261029T081605Z`: p4 against 3x3.

Both use 20 rounds, with the builds' order alternating, every block's output checked
against the room walk's, and no rounds misplaced. The files are in `results/`, and the
analysis is `analyse.py`, unchanged.

### 9.1 The rule: both pass

| arm | paired geometric mean vs p4 | t 95% | block bootstrap (L = 4) | rounds won | worst round | halves | rule |
|---|--:|--:|--:|--:|--:|--:|---|
| 4x2 | **1.230** | 1.217–1.243 | 1.219–1.238 | 20/20 | 1.186 | 1.227, 1.233 | **pass** |
| 3x3 | **1.323** | 1.312–1.335 | 1.312–1.333 | 20/20 | 1.271 | 1.324, 1.323 | **pass** |

- **4x2 decides the pre-registered hypothesis, and it holds.**
  - Its discovery figure (1.226, 1.214–1.238) reproduces within 0.3%.
- **3x3 is confirmed as the nominee.**
  - It came in 2.3% under its discovery figure (1.354, 1.344–1.364), and the two intervals
    don't overlap. §5's "two stages" anticipated this: the best of about 19 arms is biased
    upwards, which is why it was confirmed alone.
  - Most of the gap is p4, not 3x3. p4's median rose from 1412 to 1463–1465 MB/s, and 3x3's
    from 1912 to 1942.
  - In the 19-arm grid, p4 shared the machine with arms of 8–10 cores between its rounds.
    Here it shares it with one arm.
- **Stable.**
  - Lag-1 autocorrelation is −0.06 and −0.03.
  - The trend is −0.12% and −0.18% per 10 rounds (t −0.1, −0.2).
  - Neither run shows a position effect.
  - p4 drifted −1.1% and −0.7% between halves, while the cooperative arms held (+0.4%, −0.0%).
- **Absolute figures (medians):** p4 1465 and 1463 MB/s, 4x2 1808, 3x3 1942.

So the promotion's step 1 (§5) is met for both. Its step 2 (kernel parity, the build, the
shared table's checks) and step 3 (the decoder A/B) are still ahead.

### 9.2 Where the time goes, again: the build barely scales

| run | arm | build share | walk vs p4's, paired |
|---|---|--:|--:|
| confirmation | 4x2 | 42.7% | 1.578 |
| confirmation | 3x3 | 42.9% | 1.702 |
| discovery | 4x2 | 42.0% | — |
| discovery | 3x3 | 42.5% | 1.73 |
| both | p4 | 26.3–26.6% | 1.000 |

**Per block, the team's build gains little from its cores.** The figures are wall time per
block: a worker's build or walk time over the blocks it took, with N blocks live, in ns/B.

| blocks live | arm | k | build | walk | build's gain over k = 1 | walk's gain over k = 1 |
|--:|---|--:|--:|--:|--:|--:|
| 4 | 4x1 | 1 | 1.019 | 2.246 | — | — |
| 4 | 4x2 | 2 | 0.965 (0.941 confirmed) | 1.331 | **1.06×** | 1.69× |
| 3 | 3x1@1536 | 1 | 0.960 | 2.024 | — | — |
| 3 | 3x3 | 3 | 0.665 (0.662 confirmed) | 0.901 | **1.44×** | 2.25× |
| 4 | p4 (production) | 1 | 0.742 (0.724 confirmed) | 2.076 | | |

The k = 1 rows are discovery's, since the confirmation ran no k = 1 arm. The confirmed
figures in brackets agree within 3%.

- **4x2's team build, on 2 cores, is slower than production's build on 1** (0.94–0.97
  against 0.72–0.74 ns/B a block).
- **The source names what a team adds over one thread** (`coop_build`, issue #81):
  - k > 1 recounts each range (#78, E5);
  - there is a barrier between the copy and the scatter;
  - phase 2's read-modify-write lands, for k > 1, mostly on lines that another thread wrote
    in phase 1 (#81, E11).
  - Which of these costs what is open. E3's build-only counters (`xsnp_hitm`, stores, IMC
    writes) and the E5/E11 variants decide it.
- **What the build is worth (a model, from the additive times above, not a measurement):**
  - If 3x3's build scaled ideally from the cooperative kernel's own k = 1 (0.96 / 3 = 0.32
    ns/B a block), 3x3's block would take 1.20 ns/B instead of 1.54. That is ×1.29, about
    1.70 × p4.
  - If it matched p4's aggregate build rate, it would be 1.38–1.44 × p4.
  - E5's pass line (3x3 ≥ 1.45 × p4, `2026-10-29-expert-review.md`) sits between the two.
- **So the build is now the larger lever, and the kernel the other.**
  - The cooperative kernel's own +18% build and +16% walk on one core are #77, E4.
  - The order §8.3 set (the kernel and the build, in the harness) stands. The build's
    measured non-scaling moves E5 and E11 to the front of it.

### 9.3 A separate observation: one block alone, with the pool idle (informative, not gated)

The single-block table (each block inverted by every arm in turn, the arms rotated) shows the
team's build getting slower as cores are added:

| k | build ns/B (two runs) | walk ns/B | walk's last helper start (µs a block) |
|--:|--:|--:|--:|
| 1 | 0.96, 0.97 | 1.64, 1.71 | — |
| 2 | 1.57, 1.67 | 1.28, 1.39 | 316, 337 |
| 3 | 2.58, 2.61 | 1.07, 1.08 | 190, 192 |
| 4 | 2.30, 2.19 | 0.88, 0.84 | 137, 117 |

Discovery's run showed the same (0.98, 1.75, 2.54, 2.27).

- **In the grid the same team starts its helpers within 7–11 µs, against 117–337 µs here.**
- In this mode, between one block of an arm and its next, the other eleven arms run. A
  helper spins about 2,000 `pause`s (tens of µs on this core), then parks, so it is asleep
  when the next block arrives.
- **Hypothesis:** wake-up latency, possibly a deep C-state exit, plus whatever cold-cache
  cost an idle pool's first touch pays.
  - That would explain the walk's late starts.
  - At about 0.8 ms extra per block at k = 3, it may not be all of the build's slowdown.
- **Why it matters later, not now:**
  - The grid is the decision metric, and its helpers stay warm.
  - A decoder's team goes idle whenever its queue drains: small files, a stream's tail, low
    thread counts. So the decoder A/B (§8.4) needs its idle-to-busy cost measured, and
    perhaps a longer spin while blocks are queued.
- **The test that would separate the causes** (deferred):
  - one block alone, with the helpers never parking;
  - then, separately, with deep C-states disabled for the run (`cpupower idle-set -D` and
    re-enabled after).
  - Each shows the build at k = 2–4 against k = 1.

### 9.4 What follows

- **Exploratory, now that the gate has passed** (§8.1, E2), on the same pilot, one at a time:
  - `just bench-coop --arms p4,4x1,3x3,s4x1,s3x3 --rounds 20`: the kernels side by side
    at a fixed footprint;
  - `just bench-coop --arms p4,p6,p8,p10,s4x1,s6x1,s8x1,s10x1 --rounds 16`: libsais's
    scaling under pressure.
- **The counters (§8.2, E3)** need a script that runs one arm alone under `perf stat`,
  pinned as in the grid. It goes on a successor branch, so this pilot stays as measured.
- **Then §8.3 in the harness:** the build first (E5, E11), then the kernel (E4).

### 9.5 The exploratory runs (measured, same pilot, 2026-10-29; not gated)

These are E2 in `2026-10-29-expert-review.md`, with each run's criterion fixed there before it
ran:
- `coop-20261029T082204Z`: p4, 4x1, 3x3, s4x1, s3x3, 20 rounds;
- `coop-20261029T082917Z`: p4, p6, p8, p10, s4x1, s6x1, s8x1, s10x1, 16 rounds.

**Run 3: the kernels at a fixed footprint.** Each figure is the paired geometric mean with
its 95% interval.

| ratio | this run | discovery |
|---|--:|--:|
| 4x1 / p4 (our cooperative kernel, k = 1) | 0.868 (0.857–0.879) | 0.865 |
| s4x1 / p4 (libsais, independent) | 1.065 (1.055–1.074) | 1.083 |
| 3x3 / p4 | 1.328 (1.302–1.355) | 1.354 |
| s3x3 / p4 | 1.587 (1.540–1.636) | 1.629 |
| **kernel: s4x1 / 4x1** | **1.227** (1.215–1.239) | 1.25 |
| **kernel under cooperation: s3x3 / 3x3** | **1.195** (1.169–1.222) | 1.20 |
| cooperation's lift, ours: 3x3 / 4x1 | 1.531 (1.505–1.557) | 1.565 |
| cooperation's lift, libsais: s3x3 / s4x1 | 1.491 (1.446–1.537) | 1.504 |

- **§7.5's decomposition reproduces.** Cooperation lifts both implementations by about 1.5.
  The gap between them is the kernel, 1.2–1.23.
- **Rounds 18 and 19 were disturbed.**
  - Every arm slowed, by up to 18% for p4 and 31% for s3x3.
  - 3x3's helpers started 236 µs late, against 6–10 µs in the other rounds.
  - libsais had 6 ms of time unaccounted, against under 0.4 ms elsewhere.
  - Something else ran on the machine.
- **The rounds are kept, as the protocol says.** Without them, every ratio moves by at most
  0.016 and every conclusion stands: s3x3 / 3x3 is 1.211 and 3x3 / 4x1 is 1.533.

**Run 4: libsais against production as workers rise (E2's question).**

| N independent workers | sNx1 / pN, paired (95%) | pN / p4 | sNx1 / p4 |
|--:|--:|--:|--:|
| 4 | 1.065 (1.057–1.073) | 1.000 | 1.065 |
| 6 | 1.135 (1.121–1.148) | 0.740 | 0.840 |
| 8 | 1.264 (1.251–1.277) | 0.471 | 0.595 |
| 10 | **1.343** (1.322–1.365) | 0.372 | 0.500 |

- **E2's reading: rising with N.** The criterion fixed in the expert review says that means
  libsais's kernel gains with memory pressure; flat at about 1.08 would have meant a
  constant kernel difference. The lead doubles from 6.5% to 34% as the live tables leave
  the L3.
- **The hypothesis it supports, not proves:**
  - ψ² halves the dependent loads per output byte, so each extra miss a byte is worth more
    the further it goes. libsais makes one load for two bytes.
  - libsais's other walk mechanisms (24 lanes, a round of prefetch lead, its chains) are not
    separated here. E1 (our own pair walk against v0.1.4 and v0.1.5, at 1–10 workers)
    separates the two-bytes-a-load part on our own code.
- **libsais has the same knee.** s6x1 falls to 0.84 × p4, and its best independent figure is
  at 4 workers. So cooperation is still what moves the peak for either kernel. That is why
  s3x3, at 1.59 × p4, is the best arm measured.
- **What this does to E1's prior.**
  - E1's pass line asks the pair walk for ≥ 1.05 × v0.1.4 at its best and at 8–10 workers.
  - libsais's 1.26–1.34 at 8–10 makes the 8–10 half of that plausible on this machine.
  - At 4 workers libsais leads by only 6.5%, so the "at its best" half is the harder one.
- **For the counters (E3):** at pN against sNx1, `mem_load_retired.l3_miss` per output byte
  should fall by about half if ψ² is the mechanism. That joins §8.2's predictions.

## 10. The counters' procedure (E3, #76; written after the confirmation, as §8.2 said)

On the branch `research/coop-counters`. The pilot's kernels are unchanged from c2620b8,
and two harness changes come with the procedure:
- **The grid's helper timings are medians.**
  - The grid's table printed the *last* round's walk imbalance and helper start or end
    spread, not a median.
  - In the five-arm run (§9.5) the last round, 19, was the disturbed one. That is where
    its 236 and 185 µs came from: the other 19 rounds were 5–10 µs, as in the
    confirmations.
  - The table now gives the medians over the timed rounds. The JSONL always had every
    round's.
- **`bench/coop.sh --build-only`** builds the pilot (with libsais) and prints its path and
  CPUs, so the counters script builds exactly what the grid runs.

**The procedure** (`bench/coop_counters.py`, `just bench-coop-counters`) is §8.2's, as
registered:
- Each arm runs alone, pinned as in the grid, as `grid FILE 1 ARM` and `grid FILE 21 ARM`.
  Their difference is 20 timed rounds, without the setup or the check round.
- Each counter pair is a pass of its own beside the fixed counters:
  - bench-imc's five pairs (`reads`, `llc`, `beyond_llc`, `queue`, `tlb`);
  - added after the expert review: `mlp` (`l1d_pend_miss.pending`, `.pending_cycles`),
    `fill` (`.fb_full`, L1-missing loads), `snoop` (`xsnp_hitm`, stores, for #81), and
    top-down level 1 in two passes (`slots`, `frontend`);
  - the memory controller's reads and writes, machine-wide.
- The default arms are p4, p8, 4x1, 4x2 and 3x3, plus s4x1 and s3x3 with libsais.
- There are 3 repeats, with the arms and passes rotated and the short and long runs' order
  alternating.
- **The guards:**
  - every run's raw counts and perf CSV are kept in the JSONL;
  - a reading counted under 99.5% of the time is dropped as multiplexed;
  - a difference whose spread over the repeats exceeds 10% is flagged (†);
  - each counted run's own MB/s is set against the confirmed medians, so a counted run is
    a typical one;
  - a group whose events don't count on the machine is left out and reported, not
    guessed.
- **The phase attribution:** one `perf record` each of p4, 4x1 and s4x1, with the hottest
  user symbols. For libsais that is `calculate_biPSI*`, `decode_chains_symbol_rows` and its
  copies, against our build and walk. This is E3's libsais half, and E1 (§7 of the expert
  review) made it the question: what makes libsais's ψ² cheap where ours isn't.
- **The predictions it reads** are §8.2's, unchanged:
  - **Residency:** 4x2's and 3x3's L3 hit share ≥ p4's and well above p8's;
    L3-missing reads per KB ≤ p4's; DRAM writes per byte back at p4's level.
  - **Overlap:** the hit share near p4's or lower, while L3-missing reads in flight
    machine-wide are about 2 × p4's, at no lower occupancy per miss.
  - **Both:** each leaves its mark.
  - **Added from E2:** at sNx1 against pN, L3-missing loads per KB fall by about half if
    libsais's lead under pressure is ψ²'s.
  - The script prints these readings mechanically, with its thresholds stated. The
    journal interprets them.
- **Checked on the VM** with a stand-in perf (the VM's perf doesn't match its kernel): the
  plumbing, differences, coverage, figures and report, against the pilot's real grid
  runs. The counts there were stand-ins, so nothing about the events is checked until the
  i9's first short run, which is why it comes first:
  `just bench-coop-counters --arms p4 --reps 1 --rounds 3 --no-record`.

### 10.1 The i9's short run, and what it changed (measured, 2026-10-29)

`coop-counters-20261029T085523Z`: p4, 1 repeat, 1 and 3 rounds. The raw counts are in
`results/`.
- **Everything counts.**
  - All 11 passes counted 100% of the time: nothing multiplexed, no failed runs.
  - p4 ran at 1450 MB/s while counted, against its confirmed 1470 (0.986).
- **The subtraction is sound.**
  - The cycles between the two runs are 5.66e9 for 2 rounds. That is the rounds' 0.29 s
    × 4 threads × about 4.9 GHz, so the difference is the timed work, with the cores busy
    throughout.
  - The instructions between them are 9.96–9.97e9 in every pass, so the runs are alike.
- **Two corrections to how the events were read:**
  - **`l1d_pend_miss.fb_full` counts requests, not cycles.** It counts the loads, stores
    and software prefetches that found no free fill buffer. p4's figure is 0.24 such
    requests a cycle, 2.9 an output byte, not "24% of cycles". It's still a strong sign
    that the core's fill buffers bind, but it's a rate.
  - **The demand-load events see only what prefetch missed.**
    - Both native walks prefetch each lane's next entry to L1, and libsais prefetches a
      round ahead.
    - `mem_load_retired.l3_hit` is 0.013 a byte, against one random load a byte in the
      walk. So its "L3 hit share" (0.748, matching the earlier workstation run's 0.76)
      is the hit share of prefetch misses, not of the walk's traffic.
    - The traffic is in `offcore_requests.all_data_rd`, prefetches included: 1.43 reads
      past L2 a byte, each waiting 60 cycles, about 30 in flight machine-wide.
    - `l1d_pend_miss.pending` is demand-only too. That is why it shows 1.8 misses in
      flight while 7.6 reads per core are outstanding past L2.
- **So §8.2's residency reading needs a hit share that includes prefetches.** Two passes
  are added:
  - `prefetch`: `sw_prefetch_access.t0` and `load_hit_pre.sw_pf`, the prefetches and the
    loads that found their line still arriving;
  - `l3_all`: every data read past L2, prefetches included, by where the L3 found it (the
    offcore-response events, under both of perf's names, so one is left out).
  - The report's residency reading uses the prefetch-inclusive share when it counts, and
    says which share it used.
- **p4's figures, indicative from one short run:**
  - IPC 1.76, at 23.8 instructions a byte;
  - an L3-missing read waits about 595 cycles;
  - 0.128 of cycles stalled on an L3 miss;
  - DRAM traffic of 6.6 B read and 3.2 B written per output byte;
  - page walks 1.2% of cycles;
  - cross-core HITM 0.011 a KB, as expected with no sharing.
  - These become evidence only in the full run, beside the other arms.

- **Each counter's units and meaning, in the report itself** (after the second outside reading):
  - every figure states its unit: requests, retired loads, cycles, occupancy, bytes or a
    fraction of issue slots. Counts of requests, cycles and occupancy are not
    interchangeable;
  - every event is quoted as the machine's own PMU tables describe it (`perf list -j
    --long-desc`), so a reading's meaning is the i9's, not a recollection;
  - "software prefetches executed" counts instructions, useful or not: a redundant
    prefetch or one whose line is evicted before use is counted too;
  - "late prefetches" is `load_hit_pre.sw_pf`, a demand load dispatched to a fill buffer a
    software prefetch had allocated. It is read with its quoted definition beside it.
  - The short check runs again before the full run, for the two new passes.

### 10.2 The full run's first lines: neither offcore-response name exists (2026-10-29)

perf 6.8.12 on the i9 rejected both names of the prefetch-inclusive L3 pair:
`offcore_response.all_data_rd.l3_{hit,miss}.any_snoop` and `ocr.all_data_rd.*`.
- The run left both groups out, as built to, and went on with 12 passes: the 11 of the
  short run plus `prefetch`.
- So that run has no prefetch-inclusive L3 hit share.
  - Its residency line uses the demand-only share, which sees only what the prefetches
    missed.
  - It is not read as residency evidence.
  - The report now says so wherever that happens.
- **Two independent replacements, for a short follow-up run of their own:**
  - **`l3_all_raw`:** the same offcore-response events by encoding. That is event 0xB7,
    umask 0x01, with the response MSR from Intel's Skylake-client tables: ALL_DATA_RD
    (0x491: demand data reads, L2 and L3 data prefetches, L1D and software prefetches) by
    supplier, L3_HIT 0x3C0000 or L3_MISS 0x3C000000, any snoop 0x3F80000000. That gives
    0x3F803C0491 and 0x3FBC000491. The kernel moves the second onto the other response
    MSR.
  - **`llc_all`:** the architectural last-level-cache events, perf's generic
    `cache-references` and `cache-misses`. They exist on every Intel core: core requests
    that missed L2, by whether L3 had them, with data, code, RFOs and prefetches
    together.
  - Neither is the demand-only share. If both count, they cross-check each other, and
    `l3_all_raw`'s misses × 64 B against the memory controller's read bytes checks the
    encoding.

### 10.3 How the counters will be read, fixed before they arrive

In this order, and no step before the ones above it:
1. **Validity.**
   - Do the raw encodings count, unmultiplexed?
   - Are their totals plausible? Raw hits plus misses should be of the same order as
     `offcore_requests.all_data_rd`, and raw misses × 64 B of the same order as the memory
     controller's read bytes.
   - A small discrepancy starts an investigation, not a rejection. The controller's reads
     also carry RFO fills, page walks, coherence transfers and background traffic, and
     the PMU's definitions differ at the edges. The read-against-write split keeps
     writebacks out of the reads.
2. **Cross-check.** Do the raw data-read pair and the architectural LLC pair move the same
   way between arms, allowing that the latter also counts code and RFOs?
3. **Mechanism.** What changes between p4, p8, 4x2, 3x3, s4x1 and s3x3, against §8.2's
   predictions.
   - Those predictions are hypotheses, not validity tests.
   - A direction against them, such as p8's hit share at or above p4's, sends the reading
     back to the raw counts, the request mix and the DRAM traffic before anything is
     concluded. It does not discard the counter.
4. **The engineering choice.** Whether the findings favour the build (E5, E11 #81), the
   misses in flight (lanes, prefetch distance) or the hot working set (E7, E9).

### 10.4 The full run (measured, `coop-counters-20261029T093232Z`, 16d4431)

It ran 504 runs: 7 arms × 12 passes × 3 repeats × (1 and 21 rounds). `l3_all` and
`l3_all_ocr` were left out (§10.2).

**Step 1, validity** (§10.3's order). Most of it passes; one class of figures doesn't.
- **Coverage and errors:** every pass was counted 100% of the time, with no failed runs
  and no readings dropped.
- **Typicality:** while counted, the arms ran at 0.93 (p8), 0.95 (4x2) and 0.98–1.005
  (the others) of their confirmed MB/s. The first repeat was the slowest in every arm
  (p4 1395 against 1486 and 1468).
- **Stable figures**, spread under about 5% over the repeats:
  - instructions per byte (spread under 0.2%);
  - the prefetch-inclusive reads past L2: per byte, in flight and the cycles each waits;
  - the fill-buffer rate;
  - DRAM bytes, except p4 and s4x1 (6.6–8.9 and 5.2–8.5 B read per byte).
- **Unstable figures:** the demand-only L3-miss ones (L3-missing demand reads, their
  waits and their reads in flight). For example, s3x3's wait ranged 95–967 cycles and
  p4's in-flight figure 0.62–1.46. They count the small remainder the prefetches miss,
  so they are **not read** here.
- **The stable figures reconcile with the measured throughput through Little's law.**
  Reads per second are reads in flight over the wait, and MB/s is that over reads per
  byte, at each arm's own clock (3.7 GHz × cycles / ref-cycles).

| arm | clock | reads past L2 in flight, machine-wide | each waits | reads past L2 / B | model MB/s | measured | model / measured |
|---|--:|--:|--:|--:|--:|--:|--:|
| p4 | 4.99 GHz | 28.1 | 67 cycles (13.5 ns) | 1.398 | 1493 | 1462 | 1.022 |
| p8 | 4.91 | 70.0 | 366 (74.6 ns) | 1.338 | 702 | 654 | 1.073 |
| 4x1 | 4.91 | 23.4 | 62 (12.6 ns) | 1.516 | 1225 | 1241 | 0.987 |
| 4x2 | 4.90 | 38.8 | 64 (13.1 ns) | 1.553 | 1909 | 1712 | 1.115 |
| 3x3 | 4.90 | 41.9 | 66 (13.4 ns) | 1.518 | 2056 | 1908 | 1.078 |
| s4x1 | 4.94 | 19.4 | 71 (14.4 ns) | 0.860 | 1563 | 1561 | 1.002 |
| s3x3 | 4.91 | 31.1 | 76 (15.5 ns) | 0.891 | 2256 | 2367 | 0.953 |

The model overstates the team arms by up to 11%. The likely reason is the time a team's
helpers spend outside the walk's read stream, in the build's barriers and its serial
prefix sums.

**Step 2, the cross-check:** waiting for the short follow-up's `l3_all_raw` and `llc_all`
(§10.2). Until then the L3 split is inferred, not measured.
- The memory controller's read lines per read past L2 bound the share that went to DRAM
  from above: (DRAM read B per B / 64) / (reads past L2 per B).
- That is p4 ≤ 8.6%, p8 ≤ 40%, 4x2 ≤ 7.2%, 3x3 ≤ 5.2%, s4x1 ≤ 10.9% and s3x3 ≤ 5.4%. The
  controller's reads also carry other traffic, so these are upper bounds.

**Step 3, the mechanism** (provisional, on the stable figures; the L3 split is to come).
Each of §8.2's two readings leaves its mark, at a different contrast:
- **Residency, at a fixed core count** (p8 against 4x2, 8 cores each):
  - a read past L2 waits 366 cycles in p8 and 64 in 4x2;
  - DRAM reads fall from 34.4 to 7.1 B per byte, and writes from 9.8 to 3.7;
  - 8 live tables spill to DRAM and 4 don't.
- **Overlap, at a fixed footprint** (p4 against 4x2, 4 tables each):
  - the wait is the same (67 and 64 cycles), and so are the DRAM bytes (7.7 and 7.1);
  - reads in flight rise 1.38×, from 28.1 to 38.8;
  - so the second core on each table adds misses in flight at L3 latency.
- **3x3 has both against p4:**
  - DRAM reads −34% (7.71 to 5.08 B per byte), writes −42% (3.57 to 2.09);
  - the same wait;
  - 1.49× the reads in flight (28.1 to 41.9).
- **This corrects a reading that went the other way.** The demand-only "L3-missing reads
  in flight" is lower in 3x3 than in p4, but it is one of the unstable figures. The
  prefetch-inclusive occupancy is the stable one, and it rises 1.49×.
- **Per core, a team's cores are not fill-buffer bound; production's are.**
  - p4 averages 7.0 reads in flight a core, against 4.85 in 4x2 and 4.66 in 3x3.
  - Requests found the fill buffers full 0.24 times a cycle in p4, against 0.07–0.08 in
    every cooperative and libsais arm.
  - So production's walk meets full fill buffers about three times as often per cycle.
    That is pressure, not a measured limit: how much throughput relieving them would buy
    isn't measured.
  - Within a team, a core's in-flight count is lower (4.7–4.9 against p4's 7.0) at the same
    wait. That fits a core held back by its own instructions, but it is a reading, not a
    measurement.

**The instruction finding (E4, #77).**
- Instructions per byte are p4 23.8, 4x1 31.8 (+34%), 4x2 35.0, 3x3 35.3 (+48%), s4x1
  21.4 and s3x3 23.2. The spreads are under 0.2%.
- The cooperative kernel's k = 1 penalty (0.87 × p4) is instructions, not memory:
  - 4x1 has +8% reads past L2 per byte and a similar wait;
  - it runs a higher IPC (1.95 against 1.71) on 34% more instructions;
  - it executes 4.55 stores per byte against 3.03.
- A team adds about 3.5 more instructions per byte: the recount (#78), the barriers, the
  segment claims.
- libsais does its whole inversion, build included, in 21.4–23.2 instructions per byte.
- `perf record`, in user cycles:
  - 4x1: `coop_walk` 55.6%, the build's phase 2 (the scatter) 22.2%, phase 1 (the copy) 5.1%;
  - p4: `room_segments` 62.9%, and `Inv::invert` 24.9% (production's build, inlined).

**libsais's edge is fewer reads per byte, at a slightly longer wait.**
- 0.86–0.89 reads past L2 per byte, against our 1.40–1.55, and 0.52 software prefetches
  per byte against our 1.00. That is one dependent load for two bytes (ψ²).
- Each read waits a little longer (71–76 cycles against 62–67), and more of its
  prefetches are late (8.6–12.3% against 3.5–5.9%).
- Its build is not cheap: in s4x1, `calculate_biPSI_marks_symbol_rows` takes 27% and
  `compute_histogram` 6%, against the walk (`decode_chains_symbol_rows`) at 42%.
- Its lead is the walk's reads per byte, at an instruction count our cooperative kernel
  exceeds by half.

**Cross-core snoops (#81) are not a material cost.**
- HITM loads are 0.33 (4x2), 0.84 (3x3) and 2.98 (s3x3) per KB, against 0.005 in p4.
- That is under 0.001 a byte, against 1.5 reads past L2 a byte. At about 100–150 cycles
  each, it is under 0.15 cycles a byte of 3x3's 20.8, under 1%.
- The store side (RFOs to another core's lines) isn't in this event. But #81's case now
  rests on the pass it removes (stores 5.6 per byte in 3x3 against 3.0 in p4), not on
  the handoff.

**Step 4, what it points at** (provisional until step 2):
1. **The cooperative kernel's instructions first (E4, #77).**
   - The teams' cores are neither latency-bound at DRAM (the wait is at L3 level) nor
     fill-buffer bound. They are busy executing 35 instructions a byte, which holds down
     their misses in flight.
   - Production's walk does the same job in 23.8. Closing that gap is the measured lever.
   - The target is production's instructions per byte on the cooperative walk and build.
2. **Then the build's stores and passes (E5 #78, E11 #81).** The case is the extra stores
   and the recount, with the cross-core argument withdrawn.
3. **ψ² only with an instruction budget.** libsais's ≤ 23 instructions a byte (build
   included) at 0.9 reads a byte is the bar. Our port (E1) paid more instructions than it
   saved in reads below the knee.
4. **The hot working set (E7, E9) is not a lever for 3x3 as it is.** Its reads past L2
   already wait at L3 latency, with DRAM at most 5% of them.
   - It is a different lever: a smaller table could keep 4–5 teams' tables resident where
     3 fit now (4x2 → 5x2, say). That raises the blocks in flight, which is a scheduling
     gain rather than a residency one.
   - E7's criterion (the best absolute at 6–10 workers) already covers it. It also needs
     the team arms in its grid, not only independent workers.

### 10.5 The L3 split (measured, `coop-counters-20261029T100349Z`, 3214f36)

These are the two passes from §10.2 (`l3_all_raw`, `llc_all`), on all 7 arms, with 3 repeats:
84 runs. Every run was counted 100% of the time, and none failed. While counted, the arms
ran at 0.90 (p8) and 0.97–1.00 (the others) of their confirmed MB/s.

**Why perf's names failed:** perf 6.8.12's tables for this core carry offcore-response
events only for `demand_data_rd`, `demand_code_rd`, `demand_rfo` and `other`. No request
type includes the prefetches (the grep in the run's notes), so the raw encoding is the
only prefetch-inclusive form on this machine.

**Step 1, validity: each check fixed in §10.3, against the full run (§10.4).**

| arm | raw hits + misses / B | `offcore_requests.all_data_rd` / B | ratio | raw misses × 64 B / B | DRAM read B / B (§10.4) | ratio | `cache-references` / B |
|---|--:|--:|--:|--:|--:|--:|--:|
| p4 | 1.849 | 1.398 | 1.32 | 6.52 | 7.71 | 0.85 | 2.06 |
| p8 | 1.651 | 1.338 | 1.23 | 36.4 | 34.4 | 1.06 | 1.86 |
| 4x1 | 2.056 | 1.516 | 1.36 | 5.71 | 8.23 | 0.69 | 2.35 |
| 4x2 | 2.118 | 1.553 | 1.36 | 6.97 | 7.14 | 0.98 | 2.44 |
| 3x3 | 2.071 | 1.518 | 1.36 | 4.32 | 5.08 | 0.85 | 2.43 |
| s4x1 | 1.186 | 0.860 | 1.38 | 5.70 | 6.01 | 0.95 | 1.56 |
| s3x3 | 1.236 | 0.891 | 1.39 | 3.28 | 3.08 | 1.07 | 1.68 |

- **The totals agree in order, and the ratio is steady (1.23–1.39).**
  - `offcore_requests.all_data_rd` counts demand reads and the L2 prefetchers' reads, but
    not prefetches into L3 (the PMU's description, §10.4).
  - The raw ALL_DATA_RD includes those (bit 7, PF_L3_DATA_RD).
  - That fits the raw count's steady excess of about a third.
- **Raw misses × 64 B against the controller's read bytes: 0.69–1.07.**
  - p8 and s3x3 are 6–7% *above* the controller's figure. That is a small discrepancy
    across two separate runs, and within the full run's repeat spread for p8 (32–38 B).
  - Per §10.3, that prompts a check rather than a rejection. The likely cause is
    run-to-run variation: p8 ran at 0.90 here and 0.93 there.
- **`cache-references` is above the raw total in every arm,** as expected for events that
  also count code reads and RFOs.

**Step 2, the cross-check passes.** The two hit shares rank the arms identically:
3x3 > s3x3 > 4x1 > 4x2 > p4 > s4x1 > p8, raw 0.967 … 0.657 and architectural 0.957 …
0.631.

**Step 3, the mechanism, now measured. It confirms §10.4's provisional reading.**

| arm | L3 hit share, prefetches included | L3-missing data reads / B |
|---|--:|--:|
| p4 | 0.945 | 0.102 |
| p8 | 0.657 | 0.569 |
| 4x1 | 0.957 | 0.089 |
| 4x2 | 0.949 | 0.109 |
| 3x3 | 0.967 | 0.067 |
| s4x1 | 0.925 | 0.089 |
| s3x3 | 0.958 | 0.051 |

- **Residency, at a fixed core count** (p8 → 4x2): the hit share rises from 0.657 to
  0.949, and L3-missing reads per byte fall 81%.
- **Overlap, at a fixed footprint** (p4 → 4x2):
  - the hit share is unchanged (0.945, 0.949), and so are the misses per byte (0.102,
    0.109);
  - reads in flight rise 1.38× at the same wait (§10.4);
  - so 4x2's gain over p4 is overlap.
- **3x3 has both:** the hit share rises to 0.967, misses per byte fall 34% (as the DRAM
  bytes did), and reads in flight rise 1.49×.
- **Against §8.2's predictions, as registered:**
  - **4x2:** its hit share ≥ p4's holds, barely, and it is far above p8's. Its misses per
    byte (1.07 × p4's) miss the ≤ 1.05 line. The DRAM writes are at p4's level. Its reads
    in flight are 1.38× p4's (the prediction said about 2×) at no lower occupancy.
  - **So 4x2 reads as overlap, and 3x3 as both.** That is §8.2's "both: each reading
    leaves its mark".
- **libsais:**
  - s4x1's hit share (0.925) is *below* p4's, yet it misses less per byte (0.089 against
    0.102), because it reads past L2 a third less often.
  - s3x3 misses least of all, at 0.051 a byte, half of p4's.
  - Its residency comes from making fewer reads, not from a better hit share.

**Step 4, the engineering choice:** §10.4's order stands, now on measured residency.
1. **The cooperative kernel's instructions (#77).** 35 a byte against production's 23.8,
   with the L3 already serving 96.7% of 3x3's reads.
2. **The build's stores and passes (#78, #81).**
3. **ψ² only under an instruction budget.** It halves the reads per byte, and with them
   the misses: libsais at s3x3.
4. **A smaller table, as a scheduling lever (more teams resident), not a 3x3 one (#79).**
5. **The scheduler budgets both:**
   - live tables under the L3 (residency: four of these tables hold 0.945, eight 0.657);
   - cores per table (overlap: 1.38–1.49× reads in flight at L3 latency).

### 10.6 The raw pair's excess over `offcore_requests.all_data_rd`: three checks, fixed before they run

§10.5 found the raw ALL_DATA_RD pair 23–39% above `offcore_requests.all_data_rd` in every
arm.
- **The hypothesis:**
  - The raw request mask 0x491 is DMND_DATA_RD (bit 0), PF_L2_DATA_RD (bit 4),
    PF_L3_DATA_RD (bit 7) and PF_L1D_AND_SW (bit 10).
  - The PMU describes `offcore_requests.all_data_rd` as including "L2 prefetchers (not L3
    prefetchers)".
  - So the excess would be bit 7, the L2 streamer's prefetches into L3.
- **The checks** are three passes, `--groups ocr_named,ocr_no_l3pf,ocr_l3pf`, each pair in
  one pass:
  1. **The encoding scheme:** perf's named `offcore_response.demand_data_rd.l3_hit.any_snoop`
     against the raw 0x3F803C0001, its encoding by the same scheme. **Pass:** within 1% in
     every arm. **Fail:** the scheme is wrong, and §10.5's raw figures are withdrawn.
  2. **The excess:** the raw pair without bit 7 (0x411), against
     `offcore_requests.all_data_rd` in the same pass. **Pass:** their ratio is 0.9–1.1 in
     every arm.
  3. **Bit 7 alone (0x080), per byte:** it should account for most of §10.5's gap of 0.45
     (p4) to 0.55 (3x3) requests per byte.
- **What §10.5 rests on meanwhile:** its arm-to-arm directions, which the architectural pair
  reproduces independently.
  - Not its exact request totals.
  - Not the p8 and s3x3 misses × 64 B, which run 6–7% above the controller's bytes (still
    open).

### 10.7 The three checks (measured, `coop-counters-20261029T101634Z`, 9d17560): one passes, two fail

These are the three passes of §10.6, on all 7 arms with 3 repeats: 126 runs. Every run was
counted 100% of the time, and none failed. While counted, the arms ran at 0.88–0.95 of their
confirmed MB/s, below §10.5's 0.90–1.00. The browser was in use during the run. That doesn't
touch the checks, which are ratios of counts taken in the same pass, but it rules this run
out for any throughput figure.

| check | criterion, fixed in §10.6 | measured | verdict |
|---|---|---|---|
| 1. The encoding scheme: raw 0x3F803C0001 against perf's `offcore_response.demand_data_rd.l3_hit.any_snoop` | within 1% in every arm | 1.000 in every arm | **pass** |
| 2. The excess: the raw pair without bit 7 (0x411) against `offcore_requests.all_data_rd` | 0.9–1.1 in every arm | 1.219 (p8) to 1.365 (s3x3) | **fail** |
| 3. Bit 7 alone (PF_L3_DATA_RD), per byte | most of the 0.45–0.55 gap | 0.004 (p8) to 0.036 (4x1) | **fail** |

- **The hypothesis of §10.6 is refuted.** The L2 streamer's prefetches into L3 are 1–7% of
  the gap, and without them the raw pair is still 22–37% above
  `offcore_requests.all_data_rd`.
- **What check 1 establishes:** the raw scheme (request bits, supplier bits, snoop bits)
  encodes this core's offcore-response events as perf's own table does. So the raw pair
  counts what its bits say it counts. The open question is what
  `offcore_requests.all_data_rd` leaves out.
- **What remains, untested:**
  - Bit 10 (PF_L1D_AND_SW) is the L1D prefetcher's and the software prefetches' reads.
    Every walk issues a `prefetcht0` a step, and libsais more.
  - Bit 4 is the L2 prefetcher's reads. The PMU says `offcore_requests.all_data_rd` includes
    it.
  - Whether the two events count at the same point in the request's life (the super queue's
    allocation against the response) is not documented for this core.
  - A per-bit split (0x001, 0x010 and 0x400 alone) would test the first two. It is not run
    now: no decision waits on it (below).
  - Comet Lake's PMU tables and specification update are where a definitive answer would
    be. This run drew on Skylake-client's.
- **What stands, and with what label:**
  - §10.5's arm-to-arm directions stand. The architectural LLC pair, independent of the
    offcore encoding, ranks the arms identically (§10.5, step 2).
  - The mechanism's reading (residency at a fixed core count, overlap at a fixed footprint,
    both in 3x3) rests on those directions and on §10.4's in-flight counts. It doesn't rest
    on the raw totals.
  - The raw hit shares (3x3's 0.967, p8's 0.657) are the raw pair's own figures, which check
    1 validates as an encoding. They are **not reconciled** with the request totals of
    `offcore_requests`. They are quoted with that label from here on, not as a measured
    L3 hit rate of every data read.
- **No further counter runs for now.** The #77 experiment (§12) is measured by throughput
  and instructions retired, neither of which depends on this question.

## 11. Where the cooperative kernel's instructions go (#77): the procedure, before it runs

E3 measured the instructions per output byte (§10.4): p4 23.8, 4x1 31.8, 3x3 35.3, s4x1
21.4. The companion review's caution stands: those totals mix build and walk, so they don't
say which code to change. This procedure attributes them, with no change to any kernel.

**`just bench-coop-profile`** (`bench/coop_profile.py`, branch `research/coop-profile`):
- **What runs:** the arms are p4, 4x1, 3x3 and s4x1, each alone as `grid FILE 11 ARM`,
  pinned as in the grid.
- **Three `perf record` runs per arm, one event each, at a fixed period** (2,000,003), so
  samples × period counts the event:
  - `instructions:u`, the retired instructions;
  - `cycles:u`, the time;
  - `cycle_activity.stalls_mem_any`, the cycles stalled on memory. It is left out if the
    PMU lacks it.
- **Per symbol, per output byte:** each event's count over the bytes of the 11 timed rounds
  and the check round. Each symbol is labelled by phase:
  - walk (`room_segments`, `coop_walk`, libsais's `decode_chains_*`);
  - build (`coop_build`, libsais's `calculate_biPSI*` and histogram; production's build,
    inlined in `Inv::invert`);
  - synchronisation (the pool, OpenMP);
  - copies (`memmove`, `memset`);
  - setup (MTF, Huffman, the grid's own loop and its check hashes), which is outside the
    kernel's work.
- **Per symbol, the hottest instructions** of each arm's top 4 symbols by instructions and by
  cycles, under all three events. The full `perf annotate` output goes into the archive,
  with source lines where the debug info has them.
- **Provenance:** the commit, the binary's SHA-256 and build id, rustc and perf. The
  `perf.data` files stay on the machine and are kept out of the archive.

**Its checks:**
- The per-byte instructions of each arm's kernel symbols (walk + build + synchronisation +
  copies) should be close to E3's totals.
- Setup is excluded from both, but the grid's own per-round code may sit in a "setup" symbol.
  So this is a cross-check, not a criterion.

**What it should answer, with no code changed:**
1. **Of 4x1's +8.0 instructions a byte over p4:** how much is the walk (`coop_walk` against
   `room_segments`), and how much the build (`coop_build`'s copy and scatter against
   production's inlined build)?
2. **Of 3x3's further +3.5 over 4x1:** how much is the k > 1 recount (#78), the pool's spin
   and park loops (`pause` instructions retire, so a spin-wait shows as instructions), and
   the segment claims?
3. **Per phase, retired work against memory stalls:**
   - a symbol with high instructions and few stall cycles is work that can be removed;
   - one dominated by stall cycles is latency, which removing instructions won't buy.
4. **In the hottest instructions:** which sequences have no counterpart in production's
   walk? For example, per-lane state spilled to memory, bounds checks, flag tests or
   segment bookkeeping.

**Then, and only then, a change is proposed:** one sequence, its expected saving in
instructions a byte, and the grid A/B against PR #73's kernel as its measure.
