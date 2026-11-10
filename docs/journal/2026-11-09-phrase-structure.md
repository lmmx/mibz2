# 2026-11-09: The inverse BWT as copies of decoded text: the phrase census, an exact copy walk, and what they bound (closed 2026-11-10)

**Result: a structural finding, a correct new kernel, and no speed-up yet.**
- **The finding (counted, all 233 full Wikidata blocks).** The walk lands on the start of one of
  Ψ's translation cells only 0.0722 times a row. Between two landings it runs parallel to the
  trajectory of the row above, so the text there is a copy of other text in the block. Those
  *phrases* are heavy-tailed: 78% of them are one byte long, yet **69.2% of the output's bytes
  lie in phrases longer than 128 bytes**, and the size-biased mean phrase is 484 bytes.
- **What it allows (counted).** A walk that copies from text it has already decoded needs
  0.078–0.087 dependent events a row, against production's 1.0. It takes a real table step on
  5.9–6.2% of rows and copies 94% of the bytes, 57–69% of them in copies longer than 128 bytes.
- **An exact kernel exists (checked).** `examples/phrase_walk.rs` inverts every one of the
  corpus's 431 blocks byte for byte as `bwt::walk` does, from a table of one 8-byte record a
  cell and a state stream written in text order. It never builds the T-vector.
- **It is slower (local, indicative only).** On the shared VM, alone: 4.6 ns a byte against
  production's 2.6 for build plus room walk, 1.75 times as long. Each event costs about 60
  cycles there, one unoverlapped cache miss and one or two mispredicted branches.
- **The bound (model).** Production's build and walk cost only 70–110 cycles a run of L. No
  run-native design counted here gets under that by a wide margin on one uncontended core. The
  ceiling this entry derives is about 2× the whole decoder, and it needs every stage rebuilt
  around runs.

**Superseded in part: read "Correction and close" first.**

Nothing in the decoder changes. The census and the prototype are examples; nothing calls them.

## Correction and close (2026-11-10)

**The line is closed for inputs like this corpus. The copy walk is correct and does not win.**
What follows corrects the entry below it; the original text is kept, and where the two differ
this section stands (RULES B4).

**1. The census's copy certificate was unsound, and its counts were too kind** (the Researcher's
REVISE, review 6755014327).
- **The fault:** the census accepted a copy for as long as the two trajectories kept a constant
  row distance. Two rows of *different* cells can keep their distance for a step and emit
  different bytes: the Researcher's `aab`. The certificate is the cell, step by step, which is
  what the prototype's room check has always used.
- **A second fault:** one policy chose its source by the length of the copy it would give,
  which no decoder can know. It now takes the nearer row, as the prototype does.
- **Recounted** (`examples/phrase_census.rs` at this commit; per-block lines' SHA-256
  `70ddd5c2…ff1e05c7`):

  | source policy | real steps / n | copies / n | events / n | bytes copied | mean copy | bytes in copies > 128 |
  |---|--:|--:|--:|--:|--:|--:|
  | the cell's first visit | 0.0722 | 0.0328 | 0.1050 | 0.928 | 28.3 | 0.566 |
  | the nearer of its first and latest visits | 0.0722 | 0.0250 | **0.0972** | 0.928 | 37.2 | 0.611 |
  | the nearest visited row, within 64 | 0.0767 | 0.0176 | 0.0944 | 0.923 | 52.4 | 0.671 |

- **So:** about 0.10 events a row, not 0.078–0.087; 93% of bytes copied, not 94%; and exactly
  one real step a cell (0.0722), as §2's argument requires. The figures in the summary above
  and in §3's second table are superseded by these.
- **Not affected:** the phrase histogram (§3's first table), the Ψ^s cell counts, the
  first-split words and the tube packets, none of which used that certificate; and the
  prototype's bytes.

**2. Overlapping the events does not help: the walk is not waiting on memory.**
- **Built:** `examples/phrase_lanes.rs`, the same walk in lanes over 32 segments, with segment
  starts as one-row cells and copies allowed from other lanes' segments up to their frontier.
  Byte-identical to `bwt::walk` on all 431 blocks at 1, 3, 8, 16 and 32 lanes.
- **Timed locally** (indicative only): its walk takes 3.8–4.0 ns a byte at 1, 4, 8, 16 and 32
  lanes alike, against the single chain's 3.3 and production's 1.7–1.9.
- **Counted** (callgrind with branch simulation, the corpus's first 3 blocks, 1,799,936 rows,
  8 lanes): 15.3 instructions and 0.24 mispredicted branches a row, against the 10.4–10.65
  instructions a byte the record gives production's room walk
  (`2026-11-05-partitioned-walk.md`).
- **So §4's "one unoverlapped cache miss" was wrong.** Each event costs about 150 instructions
  and two mispredictions, whatever is in flight. 0.1 events a row at that price is more work
  than one cheap, well-predicted step a row.
- **The table alone is no better:** the same lanes with no copies (a move-structure walk) take
  9.3–9.8 ns a byte, about 20 cycles a step, as #42 found.

**3. Copying decoded text during inversion is published.** Kärkkäinen, Kempa and Puglisi's
Precopy (DCC 2012, §4) copies bytes and per-position states shifted by the row distance from
the first visit of an s-run, and their Table 3 has copy-based inversion overtaking the
interleaved super-alphabet walk only from n/r of about 80. This corpus is at 13.9. What is not
in that paper is the adaptive phrase end found from the source's room, on a table of O(r)
cells. No priority is claimed for copying, and §8's "not searched afresh" is superseded by this
paragraph.

**4. §5's traffic comparison is withdrawn** (the Researcher's finding 2: it set the walk's lines
past any cache against production's misses past L2). Under one cache model
(`2026-11-10-bandwidth-titration.md`'s, at a 2 MiB last-level share, the same 3 blocks), the
laned walk reads 0.041 and writes 0.078 cold lines a row; the plain T-vector walk reads 0.128.
That is no saving.

**5. §6's diagnostic is withdrawn before any run.** `bench-phrase-knee`, its frozen pointer and
its reporter are removed. It would have spent a workstation slot on a kernel that points 2 and 4
already place.

**6. §5's budget table is not reachable by this kernel.** Its walk row assumed overlapped misses
at about 30 cycles a run. The measured structure is instructions and branches, about 210
instructions a run.

**What would reopen it:**
- an input that matters with n/r of 80 or more (the walk's events fall with r, and Precopy's
  crossover is there);
- or a copy event at 40 instructions and one misprediction or fewer, shown by count before any
  timing.

**What survives as useful:** the phrase histogram as a description of the data; `phrase_walk.rs`
and `phrase_lanes.rs` as exact references; and the cells-and-room formulation, should a
high-ratio input come.


Labels: **proved** (the argument is given), **counted** (by the census, with its checks),
**checked** (byte equality on the corpus), **local** (timed on the shared VM: indicative only,
RULES C5), **model** (arithmetic from stated assumptions, never measured).

## 1. Why this was looked for

The record closed three leads on the same idea, each for a reason that left this one open:
- **Forward-chain Precopy** (`2026-10-27-precopy-forward-chains.md`) copied 31.5% of Wikidata's
  bytes and lost 18%. Every walked step paid a neighbour test, only chains of 32 steps or more
  from the row directly below were recorded, and a copying lane left nothing for a third
  occurrence to copy from.
- **The certified-packet census** (`2026-11-09-packet-census.md`) counted *tandem* repeats: a
  packet had to return to its own cell at a constant stride. Its review named the gap: "nonlocal
  repeated substrings need not be tandem repeats", and kept one census "for later (not queued
  for building)".
- **The move structure** (`2026-10-27-move-structure-walk.md`) kept one step a byte and added
  25 instructions to each.

None of them measured how long the walk stays parallel to a trajectory it could copy from. That
is the quantity below.

## 2. The objects (proved)

F convention, as `2026-11-08-run-fed-tvector-and-psi2-census.md` §1: the walk's state is
y₀ = `orig_ptr`, y_{t+1} = Ψ(y_t), and it emits F[y_t]. These are `bwt::walk`'s bytes (checked
on every block).

- **Cells.** A cell is a maximal interval of rows on which Ψ is a translation and F is
  constant. S is the set of cell starts. |S| = r, the runs of L (plus a bucket boundary inside a
  translation, which no block here has: cells/n and runs/n are both 0.0722).
- **Two rows of one cell stay together.** If x < x′ lie in one cell, Ψ(x′) − Ψ(x) = x′ − x and
  both emit the same byte. By induction they emit the same bytes and keep their distance for as
  long as every pair of images lies in one cell: until a cell start lands in (Ψ^i(x), Ψ^i(x′)].
- **Phrases.** Take x = y − 1. The pair (y_t − 1, y_t) separates exactly when y_t ∈ S. So the
  text is cut into r phrases at the times the walk lands on S. Inside a phrase, T[t] = T[t + Δ]
  for one Δ: the walk copies the text at the row above's times. This is the bidirectional
  scheme of size about 2r that the r-index literature derives from the BWT's runs (Gagie,
  Navarro and Prezza's φ). Its source may lie later in the text, so it can't be decoded as
  written.
- **A source that is always earlier.** At time t in cell k at row y, let any row y′ of the same
  cell have been visited at an earlier time q. Then T[t + i] = T[q + i], and y_{t+i} =
  y_{q+i} + (y − y′), for every i until a cell start lands between the two trajectories. The
  source is decoded text by construction. This turns the bidirectional scheme into forward
  copies with no suffix-array samples.
- **When the copy ends, and where the walk is then.** Write the source's state at time q + i as
  (its cell, the rows above it in the cell, the rows below it). With d = y − y′ > 0 the copy
  continues while the source has at least d rows below it, and with d < 0 while it has at least
  |d| above. At the first failure, the walk's row is the source's row plus d, which lies in a
  neighbouring cell, found by stepping over cell lengths. Nothing here needs Ψ.
- **Its states are copies too.** While the copy runs, the walk's state is the source's with d
  added to the rows above and taken from the rows below. So one stream of states in text order
  serves every later copy, whether its positions were walked or copied.

## 3. The census (counted)

`examples/phrase_census.rs`, on `corpora/wikidata.bz2` (SHA-256 `82f65ace…c29915`), the 233
blocks of 65,536 rows or more: 209,697,662 rows. Every copy and every packet it counts is checked
against the trajectory (equal displacement at every step, and the landing row). Its per-block
lines have SHA-256 `28e5a4f9…16dd47` (uncompressed, 233 lines); the run takes about 80 seconds.

**Phrases on the trajectory.** Share of output bytes, and phrases a row, by phrase length:

| length | 1 | 2 | 3–4 | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | > 128 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| bytes | 0.056 | 0.010 | 0.006 | 0.012 | 0.018 | 0.027 | 0.071 | 0.107 | **0.692** |
| phrases / n | 0.0563 | 0.0050 | 0.0018 | 0.0020 | 0.0014 | 0.0012 | 0.0016 | 0.0012 | 0.0018 |

The mean phrase is n/r = 13.9 bytes; the size-biased mean is 484.4. Per block, the share of
bytes in phrases over 128 runs from 0.23 (min) through 0.50 (p10) and 0.73 (median) to 0.80.

**The online copy walk, by how it picks a source.** One *event* is a real step or a copy.

| source policy | real steps / n | copies / n | events / n | bytes copied | mean copy | bytes in copies > 128 |
|---|--:|--:|--:|--:|--:|--:|
| the cell's first visit | 0.0596 | 0.0442 | 0.1039 | 0.940 | 21.3 | 0.516 |
| the better of its first and latest visits | 0.0590 | 0.0277 | **0.0868** | 0.941 | 33.9 | 0.575 |
| the nearest visited row, within 64 (every visit registered) | 0.0620 | 0.0157 | 0.0777 | 0.938 | 59.9 | 0.686 |

- **The floor is the real steps.** 0.0407 n rows are one-row cells, which no copy can reach, and
  about 0.02 n more are first visits. No source policy moves that.
- **Per block** (first-or-latest): events/n is 0.064 at its least, 0.076 at the median, 0.128 at
  p90 and 0.221 at its most; the copied share is 0.82–0.956.

**Three alternatives, counted and set aside:**

| design | what it stores | events / n | why it is set aside |
|---|---|--:|---|
| Ψ^s cells (s bytes a step) | r_s cells with s-byte words | 1/s | r_s/r is 1.22, 1.51, 1.97, 2.71, 3.85, 5.48 at s = 2, 4, 8, 16, 32, 64: well under the bound 1 + 2s(r − 1), but each doubling is a pass over r_s cells, and the table grows to 2.3 B a row at s = 8 |
| first-split words (a cell's bytes until its rows part) | one word a cell | 0.1436 | a wide cell parts early: no word passes 64 bytes, and 0.052 n events are two bytes long |
| tube packets (event-level memory only: a row range, a length, a landing row) | up to 4 packets a cell | 0.27–0.58 | all-or-nothing: a follower outside a packet's tube takes real steps. 19–47% of rows stay real under five opening and closing policies |

The third is the important negative. Exact copies need the source's room at every position, so
they need per-position state. Event-level memory alone gives up most of the gain.

## 4. The prototype (checked; timed locally)

`examples/phrase_walk.rs`:
- **The table:** one u64 a cell in F order (the cell holding its image, the offset there, its
  length, its byte), from a run scan of L, a counting sort and one merge. Cells are cut at 127
  rows, so the state fits a u32: cell (18 bits), rows above (7), rows below (7).
- **The memo:** each cell's first and latest event starts (time and offset), one u64.
- **The walk:** one chain. A real step on a cell's first visit; otherwise a copy from the nearer
  remembered visit, 8 states and 8 bytes a step in AVX2, to the first position where the source
  lacks room.
- **Checked:** all 431 blocks of the corpus, the 198 short ones included, equal to `bwt::walk`
  byte for byte. On them it takes 0.0779 real steps and 0.0370 copies a row (the cut at 127 adds
  cells).
- **Not checked:** anything else. Its `unsafe` loop has had no safety argument, Miri, fuzzing or
  mutation testing (RULES B5). It is a research example and is not proposed for the decoder.

**Timed locally** (the shared 4-vCPU VM, Xeon at 2.1 GHz, 2 MiB L2 a core, decode order, arms
alternating, medians of 3 rounds; ns a byte a thread):

| arm | copies | build | walk | total | MB/s in all |
|---|--:|--:|--:|--:|--:|
| production (`build` + `walk_segments_room`) | 1 | 0.69 | 2.01 | 2.71 | 369 |
| phrase walk | 1 | 1.35 | 3.37 | 4.72 | 212 |
| production | 4 | 0.64 | 1.84 | 2.48 | 1,460 |
| phrase walk | 4 | 1.40 | 3.47 | 4.86 | 793 |

Neither arm inflates at four copies on this machine: its L2 holds half a T-vector and its L3 is
260 MiB. So it says nothing about the i9's knee.

**Where the phrase walk's time goes (model, from the counts):** 0.115 events a row in 3.4 ns is
about 29 ns, or 60 cycles, an event. Each event is one link of a single dependent chain: the
memo, then the source's state (a miss past L2 on a 3.6 MB stream), then the next cell. Production
overlaps 32 such chains; this walk overlaps none.

## 5. What the counts bound (model)

**Production is already cheap per run.** On the i9, alone, the decoder takes 0.524 s for
209.7 MB (`bench/release-scaling`, 0.2.6): about 12.5 cycles a byte at 5 GHz, or 174 cycles a run
of L. The knee entry's stage split puts build and walk at 0.48 and 1.09 ns of 2.14 ns: about 8
cycles a byte, 110 a run. On this VM they are 5.2 cycles a byte, 72 a run.

**A run-native decoder's budget,** at r/n = 0.072, in cycles a run (cycles a byte at the i9's
clock in brackets):

| stage | today | a tuned run-native stage | basis |
|---|--:|--:|---|
| scanner | 18 (1.3) | 18 (1.3) | unchanged |
| MTF | 28 (2.0) | 12 (0.85) | records in place of the column's n bytes (`run_fed::runs_from_symbols`) |
| table | 33 (2.4) | 15 (1.1) | three passes over the runs; the prototype's is 38 with its scan of L |
| walk | 76 (5.5) | 30 (2.2) | 1.08 real steps and 0.5 copies a run, misses overlapped, plus 0.5 c/B of copying |
| RLE1, CRC, the rest | 18 (1.3) | 18 (1.3) | unchanged |
| **total** | **174 (12.5)** | **93 (6.7)** | **1.9×** |

- **The right-hand column is a target, not a result.** The prototype's table and walk are at
  about 38 and 85 cycles a run on the VM's clock. Reaching the column needs overlapped events
  (lanes over segments, as the room walk has) and a table built from MTF's records.
- **Why no design here is a step change on one free core.** The events that remain are the
  block's novelty: 6% of rows that nothing earlier resembles, about one a run. Each is an
  unpredictable step. The walk is 44% of the decoder, so removing it entirely is 1.8×.
- **A hybrid in the T-vector's world is bounded lower.** Keeping the T-vector and skipping only
  long phrases leaves the build untouched and the short 25–30% of bytes walked: about 1.3–1.4×
  on the decoder before the detector's cost. Its detector either probes every step (Precopy's
  loss) or needs the per-position state above.

**Under the i9's memory pressure the arithmetic differs, and is not settled.** Past the knee the
record shows 28–29 GB/s of DRAM traffic. At 8 tables the decoder moved 882 MB/s, which is 0.51
64-byte lines a byte. The same figure comes from the counters: 1.37 reads past L2 a byte, 36% of
them missing the L3. So past the knee, **throughput = DRAM bandwidth ÷ (64 × lines a byte)**.
- **Production's lines:** one a byte in the walk, wherever the table has left the cache.
- **The phrase walk's lines, at worst:** 0.115 events a row at 2–3 lines each, and its streams,
  5 bytes a position written and 5 read: about 0.45 a byte, a third of production's.
- **But its streams are as large as the T-vector** (3.6 MB a block), and one chain meets its
  misses one at a time. At 80 ns a miss that is 9 ns a byte. So the single-chain prototype is
  predicted latency-bound under pressure, with no advantage, and a laned one bandwidth-bound at
  up to 3× production's independent workers. Neither figure is measured.

## 6. A diagnostic for the workstation (registered here; RULES C1's diagnostic class)

**The question:** on the i9, how does each kernel's cost a thread grow with the number of blocks
inverted at once? It is asked of two structurally different kernels in one harness, so that §5's
lines-a-byte model is tested on the second.

**It can qualify no production change.** It times an inversion harness, not the decoder, and one
arm is a research example.

- **The recipe:** `just bench-phrase-knee`, frozen at the commit in `bench/phrase-knee.frozen`.
  It builds the corpus (`bench/corpus.sh`), runs `phrase_walk check` (every block equal, or the
  run is INVALID), then `phrase_walk knee corpora/wikidata.bz2 5 1,2,4,6,8,10`.
- **The arms,** each as N unpinned threads, every thread inverting all 431 blocks in decode
  order from columns made before timing:
  - `production`: `bwt::build` and `bwt::walk_segments_room`, one thread a block;
  - `phrase`: the prototype's table and walk.
- **The order:** five rounds; within a round the grid 1, 2, 4, 6, 8, 10; at each count both
  arms, their order alternating by round and grid position.
- **The figures** (`bench/phrase_knee.py`, from `runs.jsonl`): for each arm and N, the median
  over rounds of the total ns a byte a thread and of MB/s in all; each arm's inflation
  I(N) = median(N) / median(1); and phrase/production in MB/s at each N.
- **The information objective:** I_phrase(10) against I_production(10), and the ratio at N = 10.
  All five rounds are reported. There is no verdict, no pooling with another run and no rerun
  to a result.
- **Predictions, written before any i9 data:**
  - production: I(10) between 3 and 6 (the knee sweep's per-table rates);
  - phrase at N = 1: 1.4–2.0 × production's time;
  - phrase: I(10) between 1.5 and 4;
  - phrase/production in MB/s at N = 10: between 0.5 and 1.3. This straddles 1 on purpose: §5
    cannot say which way the single chain's latency and its smaller line count net out.
- **What each outcome directs:**
  - ratio at 10 of 1.0 or more: the laned walk is built next, registered against the
    cooperative kernel;
  - I_phrase(10) at or above I_production(10): the state stream is contended as the T-vector
    is, and a compact state (§7, item 2) comes before lanes;
  - otherwise: lanes first, since the gap is the unoverlapped chain.
- **Limits:** one machine, one corpus, unpinned, no counters (`perf_event_paranoid` is 4 there).
  Frequency and thermal state are not controlled beyond the alternation.

## 7. Avenues, ranked by what the counts allow

1. **The laned phrase walk** (events overlapped across segments). Segment starts become one-row
   cells, as the move-structure entry did, so no copy crosses one. Ceiling: §5's table, 1.9× the
   decoder with the run-native front; under memory pressure, up to 3× independent workers by the
   line count. Gate: the walk at or under 2.2 c/B on CI's stage harness before any integration.
2. **A compact exact state.** The 4-byte state is what makes the stream as large as a T-vector.
   Two bytes of room a position are enough to end a copy; the cell at a copy's end is then the
   open problem. One route: resolve it through the event log, whose depth is 1 when every copy
   is re-rooted at the walked text it descends from. Uncounted: the log's fragmentation.
3. **Records from MTF, table from records.** `run_fed.rs` already emits the records and proves
   them equal to the column's runs. It saves the column's write and the prototype's scan: about
   20 cycles a run across MTF and the table.
4. **The scanner in parallel.** Not this entry's subject, but the budget above makes it bind: at
   6.7 c/B a worker, ten workers outrun one scanner at 1.3 c/B. lbzip2's bit-aligned search
   for block magics is the known route.
5. **Other inputs.** r/n decides everything here. Text at r/n = 0.23 gains little; versioned
   or templated data, where r/n is far below Wikidata's 0.072, gains more than this corpus can
   show. An admission rule by symbol count (known before the inversion) keeps the T-vector for
   the rest.

## 8. Against the literature (not searched afresh in this entry)

- Kärkkäinen, Kempa and Puglisi's Precopy (DCC 2012, §4) reuses earlier occurrences found from
  the BWT and LCS during a row-wise inversion. The walk here has no row-wise table.
- Nishimoto and Tabei's move structure, and the Langmead group's Movi, take one LF step a
  character in O(r) space. The copy walk takes one event a *phrase of the visited-neighbour
  scheme*, 0.087 a character here.
- Baier's BWT tunnelling removes parallel paths at compression time. This finds them at
  decompression time, in a file that was not tunnelled.

Whether an online, sample-free forward decoding of the run-induced scheme is in print was not
checked. The claim here is the measurement and the kernel, not priority.

## Reproduce

```text
bash bench/corpus.sh                                        # corpora/wikidata.bz2, SHA-256 checked
cargo run --release --example phrase_census -- corpora/wikidata.bz2 > census.jsonl
cargo run --release --example phrase_walk -- check corpora/wikidata.bz2
cargo run --release --example phrase_walk -- time corpora/wikidata.bz2 3 1
```

Base: master `64c1eb4` (0.2.7).
