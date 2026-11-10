# 2026-11-10: Is the i9's ceiling memory bandwidth? A simulated ledger, and a titration to test it

**Status: a hypothesis with a ledger, and a registered diagnostic. No workstation data yet.**
- **The hypothesis.** At the plan the i9 ships (three pipelined teams of three), the decoder's
  throughput is set by memory bandwidth: T = W ÷ β, with W the bandwidth the machine can move
  and β the bytes each decoded byte moves through DRAM.
- **The ledger (simulated, with a stated bias).** Run under a cache model sized like the i9's,
  that plan writes 19.6 M cache lines that are not in the last-level cache, for 209.7 MB decoded:
  6.0 bytes written cold a decoded byte, 12 with their write-backs. It reads only 3.7 M lines
  cold. **The traffic is the decoder's sequential buffers, not the T-vector's random reads.**
  The T-vector's *build* is the largest writer (35%), then `memcpy` (20%), the walk's segment
  buffers (14%), MTF's run fills (13%) and RLE1's output (12%).
- **What it would mean.** The model's β of about 13 puts the cap near 2.2 GB/s at the 28–29 GB/s
  the record measured; the i9 gives 1.5. The model under-counts concurrent random reads (below),
  so the true β is higher. If the hypothesis holds, the route past 1.5 GB/s is fewer cold bytes
  written per decoded byte, and each candidate can be priced in the simulator before any
  workstation run.
- **The test (registered below).** `just bench-bw-titrate` runs the shipped decoder beside
  threads that take a measured share of the bandwidth and nothing else. A bandwidth-bound
  decoder slows in proportion; one bound elsewhere does not.

Nothing in the decoder changes.

Labels: **simulated** (callgrind's cache model; its limits are in §2), **the record's** (a
figure from a named entry), **model** (arithmetic from stated assumptions), **checked** (a
test), **local** (timed on the shared VM: indicative only, RULES C5).

## 1. Why this is asked

- The blog's own statement of the open question: "What we have not measured is *why* the blocks
  slow each other down" (`2026-11-07-why-mibz2-stops-scaling.md`).
- The knee entry ranks "a diet for the non-table buffers" last, with "none" as its evidence, and
  notes that the buffer ledger is "capacities and high-water marks, not bytes being accessed at
  the same time" (`2026-11-07-knee-mechanism.md`, E5, H4).
- The counters that could say are gone: `perf_event_paranoid` is 4 on the workstation
  (`2026-11-09-telemetry-check-registration.md`'s result).
- The record's one bandwidth figure, 28–29 GB/s of DRAM traffic past the knee, is from the
  inversion-only harness of 29 October, not the decoder.
- Fusing RLE1 and the CRC into the walk (#183) was closed on CI at about −1.4%. CI's runners
  have no bandwidth wall at 4 threads, so CI could not have shown what it is worth on the i9
  (RULES C3: "CI's grid … doesn't necessarily test the i9's question").

## 2. The ledger (simulated)

**How.** `valgrind --tool=callgrind --cache-sim=yes` on `ab child ARM THREADS corpora/wikidata.bz2`
(the bench's own child process; the corpus's SHA-256 is `82f65ace…c29915`), with a 256 KiB
first-level data cache standing for a core's L2 and one shared last-level cache: 20 MiB, 20-way
(callgrind needs a power-of-two set count; the i9's is 16-way), 64-byte lines. Decoded bytes:
209,700,196. A *cold line* is a last-level miss.

| arm | threads | cold lines read | cold lines written | cold bytes written a decoded byte |
|---|--:|--:|--:|--:|
| `mibz2` (one thread; 16 MiB model) | 1 | 177,419 | 580,313 | 0.18 |
| `mibz2-pipe3-both` (the i9's default plan) | 9 | 3,696,341 | 19,555,532 | **5.97** |
| the same, 16 MiB model | 9 | 5,424,656 | 22,693,371 | 6.93 |
| `mibz2-team2` (4 × 2; 16 MiB model) | 8 | 8,790,609 | 25,216,383 | 7.70 |
| `mibz2-workers` (16 MiB model) | 10 | 4,230,283 | 11,804,206 | 3.60 |

**Where the default plan's cold writes are** (20 MiB model):

| function | cold lines written | share | what it writes |
|---|--:|--:|---|
| `coop::FusedBuild::share` | 6,941,058 | 35.5% | the T-vector, 4 bytes a row |
| `memcpy` | 3,971,789 | 20.3% | the segments' assembly, the reader's copy, input chunks |
| `coop::traverse` | 2,754,609 | 14.1% | the walk's segment buffers |
| `memset` | 2,508,991 | 12.8% | MTF's runs, written into the last column |
| `rle::unrle_avx2` | 2,370,978 | 12.1% | the block's decoded bytes |
| `block::read_item` | 717,551 | 3.7% | the scanner's symbols |

- **Every buffer is written cold, every block.** A row's bytes are 3,276,566 lines in all; the
  segment buffers, the last column and the decoded bytes each miss on 72–84% of that. The
  T-vector, 4 bytes a row, misses on 53% of its lines.
- **The table is not being read cold.** `traverse` reads 203,733 cold lines in the model, 0.001
  a row.
- **Why they are cold (model):** three teams' working sets, the two fronts each, twelve output
  slots and the scanner's window sum to well over 20 MiB (the knee entry's E5: 67 MB peak RSS at
  three teams), and each block cycles through all of it.

**The model's known bias.** Valgrind runs one thread at a time, in slices of 100,000 basic
blocks. A thread walking its table alone for a slice keeps it resident, so **concurrent random
reads are under-counted**. `mibz2-workers` at 10 threads shows it: the model gives it the least
traffic of the four, and on the i9 ten tables are the slowest plan. So:
- the *write* columns are the usable part (sequential streams depend little on the slicing);
- the *read* columns are lower bounds;
- the model cannot rank plans whose difference is live tables.

**Against the record's figures (model).** β from the 20 MiB row is (3.70 + 19.56 + 19.56 written
back) M lines × 64 ÷ 209.7 MB = 13.1. At W = 28.5 GB/s that is a cap of 2.2 GB/s; the i9
measures 1.50 (`bench/release-scaling`, 0.2.6, 10 threads). The gap is in the direction of the
bias, and of anything else that binds. The ledger alone does not show the decoder is
bandwidth-bound. §3's test does that, or refutes it.

## 3. The titration

`examples/bw_titrate.rs` times the shipped decoder, unchanged, through `BzReader::parallel`
(the plan a user gets), beside *hogs*.

- **A hog** owns a 64 MiB buffer and repeats a 2 ms period. For the level's share of the period
  it fills the buffer with non-temporal stores; for the rest it spins. Non-temporal stores go to
  memory without a read of the line first and without entering the caches.
- **So at every level, 0 included, a hog costs one busy CPU and no cache.** Only the memory
  traffic changes with the level. The hogs' bytes are counted, and every cell reports the rate
  they reached.
- **Where the hogs run:** pinned to the CPUs of one physical core, the highest-numbered. They
  share that core's fill buffers with each other. The decoder's threads are not pinned and the
  process's CPU mask is untouched, so the planner sees the machine it always sees.
- **Checked locally:** one hog reaches 18 GB/s on the shared VM, and the decoder beside it keeps
  0.98 of its rate at 1 thread and 0.96 at 2 (2 rounds' medians). That machine is not
  bandwidth-bound at those counts, and the hog's CPU and cache cost are not what a slowdown
  would measure.

**What a bandwidth-bound decoder does (model).** With the hogs moving H, T = (W − H) ÷ β: the
decoder loses H ÷ β. With headroom U below W it loses nothing until H passes U, then falls on
that line. At β between 13 and 19, 10 GB/s of hog traffic costs 0.5–0.8 GB/s of the i9's 1.5.

## 4. The registration (RULES C1's diagnostic class)

**The question:** on the i9, at the thread counts 1, 4, 8 and 10, how much decoded throughput
does the shipped decoder lose to memory traffic it does not make?

**It can qualify no production change.** It times the released plan under an artificial load.

- **The recipe:** `just bench-bw-titrate`, frozen at the commit in `bench/bw-titrate.frozen`:
  `bw_titrate corpora/wikidata.bz2 5 1,4,8,10 0,33,66,100 2 3`.
- **The cells:** five rounds; in each, the thread counts 0 (the hogs alone), 1, 4, 8 and 10; at
  each, the levels 0, 33, 66 and 100 percent; two hogs; three decodes of the corpus a cell. Each
  round rotates the order of the thread counts and of the levels.
- **The figures** (`bench/bw_titrate.py`, which refuses any file that is not exactly this grid):
  per thread count and level, the medians of the hogs' GB/s and the decoder's MB/s; the decoder's
  rate against level 0, paired by round; and the hog traffic per decoded byte lost.
- **The reading, fixed here,** from the 10-thread cells at level 100, with r the median of the
  five rounds' paired ratios:
  - **UNDERPOWERED** if the hogs reached under 8 GB/s beside the decoder: no reading;
  - **CONFOUNDED** if the 1-thread control's ratio is under 0.90: the hogs cost something other
    than bandwidth, and no reading is taken;
  - **BOUND** if r ≤ 0.80;
  - **NOT BOUND** if r ≥ 0.92;
  - **MIXED** otherwise.
- **Predictions, written before any i9 data:**
  - the hogs alone reach 15–30 GB/s at level 100;
  - at 1 thread and at 4, the decoder keeps at least 0.90 of its rate at every level;
  - at 10 threads, r is between 0.45 and 0.75, and the loss grows with the level from the first
    step (no headroom);
  - at 8 threads (four teams of two on that machine), r is no higher than at 10.
- **What each reading directs:**
  - **BOUND:** the ledger's ranking stands as the i9's work list (§5). Each candidate is priced
    in the simulator first; a combined arm then goes to the workstation as a registered
    comparison through the Coordinator.
  - **NOT BOUND:** the cold writes are not what holds the i9. The ledger is kept as a
    description and §5 is closed. The knee's cause is then latency or occupancy inside the
    teams, and the line returns to the Optimiser's wait accounting.
  - **MIXED:** the slope across the three levels is reported and nothing is closed.
- **Limits:**
  - One machine and one corpus.
  - The hogs occupy one of the ten physical cores, so the decoder plans for ten and runs beside
    nine free ones. That is the same at every level, so it cancels in the ratio and not in the
    absolute rates.
  - Non-temporal traffic stands for the decoder's own mix of reads and writes; the memory
    controller may not price them alike.
  - Frequency and thermal state are not controlled beyond the rotation.
  - All five rounds are reported; there is no rerun to a reading.

## 5. If it reads BOUND: the candidates, priced by the ledger (model)

Each line is a share of the default plan's 19.6 M cold written lines. None has been built.

| candidate | what it removes | cold writes removed, at most | the record |
|---|---|--:|---|
| RLE1 and the CRC read the segments in order (no assembly copy) | most of `memcpy`'s assembly, and one n-byte buffer a team | ≤ 20% | #183, closed on CI at −1.4% pooled |
| MTF writes the T-vector's low bytes (no last column) | `memset`'s runs and the column's lines | ≤ 13% | H4 of `2026-10-27-research-synthesis.md`, planned, not built |
| Non-temporal stores for a block's decoded bytes | their read-for-ownership (half their traffic), and their passage through the L3 | ≤ 6% of traffic, plus what they stop evicting | the knee entry's H4 caution: untested |
| A 3-byte T-vector | a quarter of the build's writes | ≤ 9% | the `u` table moved the inversion harness's knee from 4 tables to 6 |
| Fewer output slots in flight (the window) | the cycling of twelve 0.9 MB buffers | unpriced: the simulator can price it | #180 widened it for CI |

The bound on the whole list is the model's: about half the cold writes. On T = W ÷ β that is at
most the same factor on the i9, up to the point where something else binds: the scanner at about
nine times one thread, or the cores themselves.

## 6. What this entry does not claim

- That the decoder is bandwidth-bound. The simulation is consistent with it and biased toward
  under-counting; the titration decides.
- That the candidates of §5 help on machines with headroom. CI's runners have it, which is why
  CI closed the first of them.
- Anything about the T-vector's random reads under ten live tables: the model cannot see them.

## Reproduce

```text
bash bench/corpus.sh
cargo build --release --example ab --example bw_titrate
valgrind --tool=callgrind --cache-sim=yes --D1=262144,8,64 --LL=20971520,20,64 \
  target/release/examples/ab child mibz2-pipe3-both 9 corpora/wikidata.bz2
callgrind_annotate --show=DLmr,DLmw --sort=DLmw callgrind.out.<pid>
target/release/examples/bw_titrate corpora/wikidata.bz2 3 1,2 0,50,100 1 2    # a local smoke run
python3 -m pytest bench/test_bw_titrate.py
```

Base: master `64c1eb4` (0.2.7). The simulations took 2–6 minutes each.
