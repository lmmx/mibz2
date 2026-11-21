# Why U16Q's kernel gain shows nothing on CI: the bench never runs the open kernel

*20 November 2026. A read-only measurement on master (1af2fb33) and #664's head (58bd0c43), for
the Coordinator.*

**The finding:** CI's paired bench never calls the open kernel, so a change to `src/open.rs`
can't move its figures. #664's −0.1% (−0.4%, +0.1%) at 1 thread is the null that this predicts.
- **Below 8 workers, production doesn't run Open.** The `mibz2` arm at 1 thread is the serial
  reader, `BzReader::new`, whose scratch is `Scratch<bwt::Cells>`. At 2 and 4 threads it is
  `Plan::Workers(t)`, which inverts with Cells: `Plan::lazy` is false below
  `LAZY_FROM_THREADS = 8`.
- **No exception reaches CI.** The i9's trimmed cells (`trimmed_here`) and the service core
  (`service_target_here`) are both gated on the i9-10900K itself.
- **The open kernel runs from 8 workers**, as `OpenCells`, the lazy route, or on the measured
  i9's trimmed or unpinned cells.

**When Open does run:**
- *Its share:* on the lazy route, one thread, it is about 75% of a decode's compute.
- *What U16Q does to it:* on the whole Wikidata corpus it cuts the open kernel by 11%
  (435 → 387 ms), and every other stage is level.
- *So a whole-decoder bench that ran the lazy route* should see about 7–9% at 1 worker.

## Method

- **The machine:** the cloud container, an Intel Xeon @ 2.10 GHz with 4 vCPUs, one thread per
  core and a 260 MiB L3. It is shared, so local timings are indicative only.
- **The build:** release, as `Cargo.toml` has it (fat LTO). The stage example needs
  `--features moonshot`, which only makes `Runs::records` public.
- **The corpus:** the bench's own `corpora/wikidata.bz2`, SHA-256 `82f65ace…`. It has
  431 blocks, 209.7 M rows before RLE1, and decodes to 209.7 MB.
- **The stage table:** a temporary example, `examples/attrib.rs`, which is not for master.
  - It decodes the file block by block in decode order, as `decode_block` does.
  - It times each stage: the scanner (headers and Huffman); `runs::front` (MTF, RLE2 and the run
    records in one pass, or `mtf::mtf` on the column route); the inverse by each of `Cells`,
    `LazyCells` and `OpenCells` on the same records, each with scratch reused block to block;
    RLE1; and the CRC.
  - The three kernels' outputs are compared on every block, and the CRC is checked.
  - Each block's fastest of 5 passes is kept per stage. The table sums those over blocks.
  - Admission is `OpenCells`' own gate: `recs·20 < 3n && recs·64 ≥ n`.
- **The whole-decoder check:** `ab run`, master against #664's head, with the arms `mibz2` (the
  CI arm) and `mibz2-workers-lazy` (production's `OpenCells` route forced at any thread count),
  at 1, 2 and 4 threads, `--spread`, 6 rounds.

## Stages, one thread, summed over blocks (ms)

The table shows master, the median of three runs, each within about 1.5%.

| blocks | count | rows | scan (Huffman) | front (MTF+RLE2+records) | Cells | LazyCells | **Open** (`OpenCells`) | RLE1 | CRC |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| all | 431 | 209.7 M | 74.5 | 36.2 | 370.6 | 461.5 | **437.7** | 21.7 | 11.6 |
| Open admits | 228 | 205.2 M | 70.1 | 34.4 | 354.8 | 441.7 | **416.0** | 21.1 | 11.2 |
| refused, run route | 155 | 4.5 M | 3.9 | 1.8 | 15.5 | 20.8 | 20.7 | 0.6 | 0.4 |
| column route | 48 | ≈0 | 0.4 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

- **The open kernel admits almost every row.** It takes 228 blocks, 97.9% of the rows (the
  admission journal's 228 blocks and 205.2 M rows).
- **The six admitted blocks match the calibration.** They are corpus blocks 1, 3, 158, 268, 384
  and 387, the manifest's `block_in_corpus`. For example, block 1 takes Open 1.80 ms here
  against `calib.rs`'s 1.43 ms best of 25. The difference is warm against decode-order caches.
- **Open is not the fastest kernel at one thread.**
  - It is 18% slower than Cells (437.7 against 370.6 ms), and 5% faster than LazyCells.
  - That is why production gives it only to many workers, where lazy letters' smaller footprint
    pays.

## U16Q against master, the same table

The table shows two runs each (ms).

| | scan | front | Cells | LazyCells | **Open** | RLE1 | CRC |
|---|--:|--:|--:|--:|--:|--:|--:|
| master | 75.3 / 73.6 | 36.2 / 35.9 | 364.7 / 369.6 | 461.5 / 460.5 | **434.9 / 437.7** | 20.2 / 21.1 | 10.9 / 11.2 |
| #664 head | 73.6 / 76.1 | 35.6 / 37.0 | 358.4 / 368.0 | 455.4 / 464.5 | **386.9 / 388.2** | 22.1 / 20.0 | 10.7 / 10.9 |

- **The open kernel is 11.2% faster across the whole corpus** (436 → 388 ms), in line with the
  six blocks' 9–14%.
- **The columns U16Q doesn't touch stay level, within ±2%.** Those are the scan, front, Cells,
  LazyCells, RLE1 and the CRC.

## Open's share of decode time, one thread

The total is scan + front + inverse + RLE1 + CRC.

| route | total | the inverse | its share |
|---|--:|--:|--:|
| what CI's `mibz2` arm runs (serial, Cells) | 515 ms | Cells 371 | 72% Cells, **0% Open** |
| the lazy route (`OpenCells`) | 582 ms | Open 438 | **75%** |

## What threads=N means

**In the bench:**
- `ab run --threads 1,2,$(nproc)` gives 1, 2 and 4 on CI's 4-vCPU runner (`bench.yml`).
- At 1 thread, `ab`'s `mibz2` arm is `BzReader::new`, the serial reader: every stage runs in
  turn on the calling thread, the scanner included, with no worker threads.
- At t > 1 it is `BzReader::parallel(t)`, that is `StreamDecoder::new(t, 4t, detect)`.
- `--spread` pins a run of t threads one to a core where the machine has t cores.

**In the decoder:**
- **The threads:** `StreamDecoder::new(t)` runs one scanner thread, which reads the headers and
  Huffman in order and is the only sequential stage. Beside it run `Plan::for_threads(t)`'s
  workers, and the reader (the caller's thread) taking blocks in order.
- **The workers below 8:** `Plan::Workers(t)` gives t workers. Each finishes a whole block: front
  (MTF, RLE2 and the records), the inverse, RLE1 and the CRC. Below `LAZY_FROM_THREADS` (8) they
  invert with **Cells**.
- **The workers from 8:** `OpenCells` inverts, which is the open kernel on admitted blocks and
  `LazyCells` elsewhere.
- **Teams:** teams (`Cooperative<k>`) and the service core never use `OpenCells`. They appear on
  6 or more physical cores at 11 or more threads, or with more threads than cores, outside 8–10
  cores.
- **The measured i9-10900K alone** (`trimmed`, `measured_i9`):
  - *unpinned:* at 6 threads, 6 lazy workers; from 10 threads to 20, `Plan::Workers(10)`
    trimmed to **7 lazy workers**;
  - *pinned:* 5 lazy workers at 6 threads, 6 lazy at 8, 7 lazy from 10, and 3 eager at 4.

So the open kernel is production's inverse at 6, 8, 9 and 10–20 threads on the i9, and at 8 or
more workers on other machines. It is never production's at 1–5 threads, and never at 7 on the
i9.

## Is the inverse on the critical path?

**In CI's arms:** no. Open isn't on any path; Cells is.
- At 1 thread everything is serial, and Cells is 72% of it.
- At 2 and 4 threads the workers (front, Cells, RLE1 and CRC, about 440 ms of work) outrun the
  scanner's 75 ms by 3–6×.

**On the lazy route,** each block's inverse sits inside its worker's serial chain, and workers
run blocks in parallel. So the inverse is on the critical path whenever the workers' throughput,
not the scanner's, bounds the decode:
- **The model:** wall ≈ max(scanner, worker work ÷ W) + fill and drain.
  - *The scanner:* about 75 ms here.
  - *The worker work:* about 507 ms (front + Open + RLE1 + CRC).
- **The two balance at W ≈ 7.** That is exactly the i9's trimmed worker count from 10 threads.
- **This model assumes** this machine's stage ratios hold on the i9. It ignores L3 contention,
  which inflates the kernel at many workers, and the reader's copy.

## Prediction against CI

| bench arm | Open on the path? | predicted for a 10% Open gain | observed |
|---|---|--:|--:|
| CI `mibz2`, 1 thread (serial Cells) | no | 0 | −0.1% (−0.4%, +0.1%) |
| CI `mibz2`, 2 and 4 threads (Cells workers) | no | 0 | −0.0%, +0.0% |
| lazy route, 1 worker (`mibz2-workers-lazy`, t=1) | yes | ≈ −8.6% of worker time (−44 of 507 ms), ≈ −7–9% wall | 0.968 [0.923, 1.015] here, 6 rounds |
| lazy route, 2 and 4 workers here | yes | ≈ −7–8% | 0.988 [0.928, 1.053], 0.982 [0.922, 1.045] |

**The local whole-decoder check** is 6 rounds on a shared 4-vCPU machine. Its intervals are
±5–10%, too wide to confirm a 7–9% gain. For scale:
- *Noise:* `mibz2` master against `mibz2` head, whose code paths are identical, read 1.026,
  1.024 and 0.952 at 1, 2 and 4 threads.
- *Direction:* the lazy arm leans the predicted way at every thread count.

The stage table, not this check, is the evidence for the size of the effect.

**The verdict:** the null is explained. CI's −0.1% is the expected zero: the arm it times
inverts every Wikidata block with Cells and never enters `src/open.rs`. Confidence is high.
- *The code paths:* they were read directly (`ab.rs` lines 1106–1107, `read.rs` `new` and
  `parallel_with`, `parallel.rs` `new`, `planned`, `workers`, `Plan::lazy` and `plan_with`, and
  `trim_cell`).
- *The figures:* the unchanged columns of the U16Q stage table bear them out.

It wasn't checked by instrumenting a CI run, for example with a counter of `OpenCells`
inversions.

## Where a kernel gain could appear

1. **CI, with a lazy-route arm.** The cheapest place to see an Open change is to add
   `mibz2-workers-lazy` (or `-open`) as a head-against-base pair, at 1 and 2 threads.
   - *Open's share:* there it is about 75% of compute, so 10% in the kernel is about 7–9% wall,
     easily resolved by a paired 10-round bench.
   - *Its cost:* by this table's ratios, about 0.45–0.5 s a run at 1 thread on CI's runner, and
     about 0.25 s at 2 threads, a pair (head and base) each round. Per CLAUDE.md, an addition to the PR tier
     needs its seconds a round justified against the budget.
2. **The i9 workstation, at 6, 8 and 9 threads.** Production runs `OpenCells` there, with 6–9
   workers against one scanner.
   - *Worker-bound:* by this machine's ratios that is worker ÷ W of 56–85 ms against the
     scanner's 75. At 6 and 8 threads the workers bound the decode, so part of the gain shows
     there.
   - *A model, not a measurement:* the i9's clocks and contention differ.
3. **The i9 at 10–20 threads.** Production runs 7 lazy workers there, and the scanner and the
   workers are about balanced, which is the reason 7 beat 10 workers (a core freed for the front
   end).
   - *The cap:* a 10% kernel gain pushes the workers below the scanner's bound. Expect well
     under the kernel's share, perhaps a few percent at most, and possibly nothing.
   - *An upside:* more cache contention at 7 workers inflates the kernel's share, so the gain
     might show more there than the model says.
   - *Where to look:* it is the right place to check that the gain reaches production's
     default, but not the most sensitive one. Thread counts 6 and 8 should show it more
     clearly.

## Reproducing

```sh
cargo build --release --features moonshot --example attrib   # examples/attrib.rs, not on master
target/release/examples/attrib corpora/wikidata.bz2 5 blk     # summary, then a line per block
cargo build --release --example ab
target/release/examples/ab run runs.jsonl --arms "mibz2,mibz2@HEAD_AB,mibz2-workers-lazy,mibz2-workers-lazy@HEAD_AB" \
  --threads 1,2,4 --spread --rounds 6 --settle 1 corpora/wikidata.bz2
```
