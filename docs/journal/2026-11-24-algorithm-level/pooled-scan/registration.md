# The pooled scan: a prototype's design and its kill test (registration, draft for review)

*Drafted 26 November 2026 by the Pathfinder, for the Coordinator's review. This is design only: no code is written until the review closes. It covers thread 3 of `../plan.md` (S + C together: pooling the scan), from Antigravity's proposal 2 (`research/neon-movement/IDEAL-PARTS-2.md` at 7fab624e, §5). A diagnostic under C1a: the kill test stops or continues the thread, and adopts nothing. Any production change is a later adoption registration, shaped as `../briefs/scanner.md` part 2 asks.*

## 1. What the prototype is for

**The matrix's finding** (`../ideal-parts/results/full/README.md` §2): at 8 workers, which is what ships, the scan and construction take turns being the bottleneck.

| | S alone ideal | C alone ideal | S with C ideal | S and C ideal |
|---|---|---|---|---|
| 6 workers | 0.974 | 0.754 | 0.781 | 0.59 |
| **8 workers** | **0.968** | 0.944 | **0.679** | **0.641** |
| 10 workers | 0.919 | 0.947 | 0.801 | 0.76 |

The stage-1 character study gives the mechanism (`../methodology/hybrid-systems/applications/03-structural-levers.md`). At 8 workers:
- the wall is 194.4 ms;
- the scanner is busy 185.0 ms;
- a worker averages 184.7 ms.

The two stations are balanced to within 0.2%.

**So the question is not "is a pooled scan faster than the scanner?"** At 8 workers the scan has only 3.2% of room alone, and no real scan can capture more than its ideal. The question is: **with construction cheap, how much of the scan's room does pooling capture?** That is S's room behind C, 32% at 8 workers (0.679). It is the half of the 36% pair that the scan owns.

Antigravity's kill test for proposal 2 needs the pooled scan to beat the shipped 195 ms by at least 7% at 8–9 workers standalone. That exceeds S's whole measured room at 8 (3.2%), so it would kill any pooled scan by construction. It is replaced below by a test against C's oracle, and kept as a reported figure at 10 workers, where S alone has 8% of room.

## 2. What the record already says (three earlier attempts)

The scan has been taken off its thread three times:

1. **`bench-par-scan`, 10 November** (`../../2026-11-10-parallel-scan.md`, result `../../2026-11-10-par-scan-result.md`).
   - **The design:** one finder thread locates every magic; workers read and decode each candidate's block, Huffman included; the caller's thread walks the chain exactly and accepts a worker's result only at the chain's next bit.
   - **The result:** VALID. Wikidata NOT FASTER (0.981 of the scanner's rate at 10 workers); level 1 FASTER (1.615); records FASTER (1.780).
   - **The finder was free:** 9.3 GB/s on one thread, 0.19% of a decode.
   - **The lesson:** on Wikidata at level 9 the scanner was not binding then. The workers' work has fallen since, which is why S binds now, but only once C is cheaper.
2. **`bench-parscan-c37`, 22 November** (`../../2026-11-22-parscan-c37-registration.md`, #741).
   - **The design:** every worker first scans a share of the whole input for magics, then claims candidates from an atomic cursor and decodes each in full; a joiner on the caller's thread keeps the chain's blocks.
   - **The result:** NOT ADOPTED. It was FASTER at 8 and 10 (0.935, 0.883) but SLOWER at 2–6 (1.11–1.13, on less CPU). Peak RSS grew with threads: 1.28 at 10, 1.99 at 20. CPU rose from 10 threads up (1.08–1.90).
   - **The causes named** (hypotheses, unmeasured):
     - workers and the joiner waiting on each other below the core count;
     - the whole input read first, with 4 × threads decoded blocks in flight;
     - speculative decodes past what the chain could use.
3. **`bench-parscan-stream` (#770):** the bounded streaming version lost badly, at about 1.54× at 10 workers (`../briefs/scanner.md`). Its cause, read from #770's PR and its bundle (below), is:
   - **its scan was serialised across a few threads, not spread over the workers;**
   - **its scan work was duplicated.**

### #770's cause, from its PR and bundle (added 26 November, before any code)

**The design** (#770's body):
- a lead scanner, plus one or two demand-driven helper scanners, read blocks at magic candidates;
- the workers do the rest, as now;
- at most workers + 4 blocks are held ahead of the reader.

**The run:** `parscan-stream/run-20261123T005713Z.tar.gz` on the `evidence` branch, sha256 `e432a2ba…44340bb7`, checked here. Wikidata ×4, unpinned, 16 paired rounds against master: NOT ADOPTED.

| threads | wall, head/base | CPU, head/base | peak RSS, head/base |
|---|---|---|---|
| 4 | 0.978 | 0.980 | 0.996 |
| 6 | 0.977 | 0.980 | 1.005 |
| 8 | **1.605** | 1.043 | 0.832 |
| 10 | **1.537** | 1.093 | 0.881 |
| 20 | 1.540 | 1.099 | 0.887 |

Below 8 threads the front end isn't active (it switched on only from 8 threads on the i9), so 4 and 6 are master's code with the PR's block reader. **From 8 threads up, wall rises about 55% while CPU rises 4–10% and memory falls.** The threads mostly waited.

**The per-thread CPU** (the records' `thread_cpu`, wikidata ×4, one round, seconds):

| | wall | the scanner | the helpers | each worker |
|---|---|---|---|---|
| base, 8 threads | 0.205 | 0.17 | none | 0.18 (×8) |
| head, 8 threads | 0.338 | 0.18 (lead) | 0.07 + 0.08 | 0.14–0.20 (×8) |
| head, 10 threads | 0.340 | 0.20 (lead) | 0.08 + 0.08 | 0.15–0.20 (×8) |

What it shows (inferred from the CPU, not a profile of waits):
- **The scan was serialised.** The lead did as much scanning as the base's serial scanner (0.18–0.20 s against 0.17 s), and the helpers added 0.15–0.16 s on top. The head's wall, 0.34 s, is about the sum of its three scanner threads' CPU (0.33–0.36 s). The lead and helpers took turns behind one chain head rather than running side by side.
- **The scan was duplicated.** Total scanning CPU almost doubled, against the base's 0.17 s, while the workers' CPU stayed the same. So each worker sat idle about 40% of the wall.
- **The pipeline was starved.** Peak RSS fell (fewer blocks in flight, at most workers + 4 ahead), as it would with the workers waiting on scanned blocks.

**Does this design share it?**
- **Not the serialisation.** Here no scanner or helper stage exists: each of the k workers scans its own block, so the scan is spread over k threads. The chain's acceptance stays serial, in the in-order reader, but it reads only each block's start and end bits and CRC, not its symbols.
- **Not the duplication.** Here each candidate is claimed by one worker and decoded once. On Wikidata every candidate is real, so no decode is wasted; the fallback reads only what no worker claimed.
- **Not the window.** Here the window is the shipped one (slots a worker), not workers + 4.
- **Shared: acceptance in order.** A slow block at the chain's head still holds the reader, as in the shipped decoder. The shipped window already absorbs this today, and the reported figures below watch it.

**So §3 stands. Two figures are added to §4's reported list** to catch #770's failure if it recurs:
- the scan's total CPU (every worker's A2 + A3) against the shipped scanner's;
- each worker's idle share of the wall.

A pooled scan that serialises shows K2's slowdown; these two figures say why.

## 3. The design

**Where it lives:** a research feature, `pooled-scan`, in `src/parallel.rs`, with the finder in a new `src/ideal/pooled.rs` or beside the scan oracle. With the feature off, the decoder compiles exactly as shipped. The prototype builds on bench/ideal-matrix-next (#909, with #921), so that C's zero-copy oracle and the character ledgers are in the same binary.

**The pieces, each a response to a failure above:**
- **The feeder finds.** The feeder thread (`A0`), which already reads the input in chunks, runs the magic finder on each chunk as it arrives: #317's AVX2 nibble filter, ported from `examples/par_scan.rs`. It appends candidate bit offsets to a queue ordered by bit. Nothing reads the whole input first (c37's memory).
- **No scanner thread, and no joiner thread.** Workers claim candidates in bit order, but only inside the shipped admission window (`issued − taken ≤ slots`) and the ready-bytes cap. Speculation is bounded by the same window that bounds the shipped decoder, not by 4 × threads (c37's speculation and memory).
- **A worker runs its block end to end.** It reads the item at the candidate (`block::read_item_with_scratch`, the block reader on master since #745), then MTF/RLE2, construction, emission and RLE1/CRC, as a worker does now. It publishes the block to the ready set keyed by its start bit, with its end bit and CRC. The scanner-to-worker `syms` queue goes away.
- **The chain is accepted by the reader, in order.** The in-order reader, which already takes blocks in order, takes the ready block whose start bit is the chain's current end bit. It checks the stream's level bound and combines the CRC. So the chain is walked on the thread that already waits for order, with no extra hand-off (c37's joiner waits).
- **Fallback, exact.** If no ready or claimed block starts at the chain's end bit (a missed magic, a magic across a chunk boundary, a stream header or end-of-stream), the reader reads that item itself with the same calls. The same applies to a block that fails (an error at its bit). The finder is a hint: a false or missed magic costs time and can't change a byte or an error (#317 §2's argument, carried over).
- **One worker, the shipped path.** At 1 worker the pooled scan can only be slower (#317 measured 0.76–0.89), so the prototype keeps the shipped scanner there.

**What it owes before any timing** (VOID otherwise; #317 §2 and §6):
- **exact bytes** on wikidata, wikidata ×4 (four streams), the `ab gen` corpora, level 1, and `records`;
- **exact errors** (`Error { kind, block, bit }`, as the sequential decoder's) on #317's six adversarial files and 4,000 mutated ones;
- **no guess at the level:** workers decode at level 9's bound, and the chain checks the stream's;
- **`Truncated` decided only at the input's end**, and a worker that runs out of bytes waits rather than fails;
- **wasted work reported:** candidates decoded and then dropped, per decode;
- **Miri and the test suite green** with the feature on;
- **no new `unsafe`** outside the finder's AVX2 filter, which #317 already justified.

## 4. The kill test (frozen at this registration, before any code)

**The run.** One binary (the research features plus `pooled-scan`), on the i9, wikidata ×4, unpinned (no scanner thread exists in two arms to pin), with the 100 ms pre-spin. 12 randomised blocks. Workers 6, 7, **8**, 9 and 10 for the kill, and 1 for the shipped path's check. The arms:

| arm | the scan | construction |
|---|---|---|
| `none` | the shipped scanner | real |
| `P` | pooled | real |
| `C` | the shipped scanner | ideal (zero-copy oracle) |
| `PC` | pooled | ideal |
| `SC` | ideal (scan oracle, zero-copy) | ideal |
| `S` | ideal | real (reference) |

**KILL the thread if any of these holds at 8 workers:**
- **K1, too little of the room behind C.** The 95% upper bound of PC/C exceeds √(SC/C), its point. On the log scale, the pooled scan cannot be shown to capture at least half of the scan's room behind construction. With the matrix's 0.679, PC must be about 18% faster than C. The threshold is computed from this run's own SC/C, not the matrix's.
- **K2, a cost alone where it ships.** The 95% lower bound of P/none exceeds 1.03: the pooled scan alone costs more than 3% at 8 workers. Below 8 a worker-count gate can keep the shipped scanner, so regressions at 1–7 are reported, not kills (`../briefs/scanner.md` part 2's gate).
- **K3, memory.** Peak RSS of P over none (median) exceeds 1.10 at any count from 6 to 10 (c37's failure).
- **K4, the finder.** On one thread, the finder takes over 10 ms for wikidata ×4's 47.4 MB (Antigravity's first kill; #317 measured 9.3 GB/s, about 5 ms).

**CONTINUE otherwise:** the pooled scan goes on to be engineered with thread 4's construction work, and to an adoption registration (`../briefs/scanner.md` part 2).

**Reported, deciding nothing:**
- the capture behind C, log(C/PC) ÷ log(C/SC), at each count;
- the standalone ratio P/none at every count, and against Antigravity's figure at 10 (S alone 0.919; its prediction 172–175 ms at 8–9);
- CPU, wasted decodes, and the scanner-to-worker queue's bytes removed;
- the chain's fallbacks per decode;
- the scan's total CPU (the workers' A2 + A3) against the shipped scanner's, and each worker's idle share of the wall (#770's failure, §2).

**VOID:** the exactness obligations above unmet; the machine not the i9; a decode missing or failed; a check pass not byte-identical.

## 5. What the floor run changes, read before the prototype is built

`bench-zero-copy-floor` (#917, queued at 1e27e46f) is the run this design waits on. Three of its readings bear on this one, each fixed here in advance:
- **The judged count.** If the zero-copy all-ideal wall (Residual A) rises from 8 to 10 workers beyond its interval, the memory station binds above 8 even with every part free (Antigravity's third station). The judged count then stays 8, and 9 and 10 are reported only. If it doesn't rise, 9 is judged alongside 8 (both must pass).
- **The plumbing.** If Residual B at 8 workers exceeds 10% of the shipped wall, the scanner queue's removal is a second source of gain. The capture may then exceed 1, and is reported as measured (no cap).
- **The arms' oracles.** C and S here are the zero-copy oracles the floor run validates. If the floor run VOIDs on either oracle's replay, this run waits for the fix.

## 6. Not decided here

- **Production adoption:** a separate adoption registration, after CONTINUE. It carries the brief's rule (FASTER at 10, not SLOWER at 1, 6, 8, 12, 16 and 20, text not SLOWER, peak RSS ≤ 1.10), with the gate set from data.
- **The fusion of scan, MTF/RLE2 and construction's first pass** in the worker's L1/L2 (IDEAL-PARTS-2 §4.2). It is unlocked by this design, but it is thread 4's.
- **Construction's A6b and A6c:** downstream of the block's end, so untouched by any scan (T5.1). They are threads 2 and 4.

## 7. Clarifications, registered before any data (26 November, with the kill test's recipe)

The prototype is built (`prototype.md`, #932) and accepted as the prototype, departures 1–3 included. The Coordinator asked for three clarifications before any data (12:17Z). The recipe, `bench-pooled-scan`, adds what makes them checkable. None of this relaxes §4: K1 to K4 and every VOID stand as frozen.

**1. The pooled arms are exempt from the block-indexed identities.**
- **The cause:** a pooled block is read and decoded by its worker before the chain gives it a place. The pooled path records a block's points only from ready on: the shipped insert and the shipped reader. It records nothing of the points the shipped scanner and its queue make (admitted, issued, enqueued, taken by a worker, routed). The harness counts blocks by their issue, so in a pooled trace it counts none.
- **So these identities are exempt** in the pooled arms (`P`, `PC`) at 2 workers or more: flow, linear order, forced flow, conservation, admission, Little's law and utilisation.
  - A P trace at 4 workers in the build container fails flow, forced flow, conservation, Little's law and utilisation.
  - Linear order and admission pass there only because there are no blocks to check.
  - At 1 worker, P is the shipped path, so every identity applies.
- **What still applies, to every trace:**
  - **the thread counts;**
  - **the timelines:** each thread's life is partitioned exactly into its activities and waits, balanced and in order.
- **The chain thread's own activities and waits:** for the timelines to say what the pooled threads do, the pooled path records these under their own names:
  - `pool_find`, the finder;
  - `pool_copy`, a candidate's bytes copied;
  - `pool_wait`, the chain waiting for a worker's read;
  - with `job_wait` (a worker waiting for work) and `slot_wait` (the chain waiting for room) as the shipped path records them.

  These are appended to the ledger's activities (29 to 31, so the residual moves to 32). Each trace now carries the activities' names, as the floor branch's harness does, so an analysis reads a ledger by name.
- **A failed identity:** a trace that fails an identity applying to it makes the run **INCOMPLETE**, as the floor run treats it. Its traces' figures (reported only) are withheld. The verdict, which comes from the timed walls, stands.
- **The VOIDs:** every VOID of §4 applies to every arm.

**2. K4 is measured directly, not bounded.**
- **The pass:** `examples/pooled_scan.rs --finder` holds all of wikidata ×4 (47,419,732 bytes) in memory and runs the finder alone:
  - on one thread, pinned (`taskset`) to the first CPU of the core whose calibration ran fastest;
  - one untimed pass, then the standard 100 ms pre-spin;
  - then 11 timed passes, each over the whole input.
- **The figures:** the median, and the interval between the 2nd and 10th order statistics (at least 95% coverage for the median: 98.8%). Every pass is recorded in order.
- **K4 kills if the median exceeds 10 ms.**
- **The pass is VOID** if it is missing, does not have 11 passes, did not run on its CPU, or did not find every block magic the decodes count.
- **The finder in the decode:** the chain calls the same finder piece by piece as the input arrives. Its time there is reported from the ledger (`pool_find`), beside K4.

**3. Departure 2's copy is reported.**
- **What is reported:** the chain thread's copy of the candidates' bytes (`pool_copy`, from its ledger in P's traces at each count), beside the estimate made before the data: about 5 ms (`prototype.md`).
- **Beside it:**
  - the bytes copied;
  - the symbol bytes that the shipped scanner queues to the workers (none's traces), which P does not queue.
- **Its status:** reported only.

**4. The judged counts are 8 and 9.**
- **The reading:** §5's reading of the floor run is that Residual A doesn't rise from 8 to 10 workers.
- **The rule:** KILL if K1 or K2 holds at 8 or at 9, so both counts must pass. K3 stays as registered, at any count from 6 to 10.
- **Reported only:** 6, 7 and 10.
- **At 1 worker:** none and P only, the shipped path's check. There P is the shipped scanner, so the other four arms would add nothing.

**5. §5's other readings.**
- **The capture is not uncapped:** Residual B is 5.2 ms at 8 workers, 2.7% of the shipped wall, under 10%. So the capture behind C is reported capped at 1, with the raw figure beside it where it is over.
- **This run doesn't wait:** the floor run's oracles are valid. FCMRD was 179.1 ms at 8, against 194.6 shipped.

**6. The design under test must be the one timed (a VOID).**
- **Why it can be checked:** on wikidata ×4 every magic is real and every stream is at level 9. So a pooled decode at 2 workers or more takes every block from a worker's read.
- **The VOID:** the run is VOID if any P or PC decode at 2 workers or more has a fallback, a degrade or a dropped candidate, or if P at 1 worker, or any unpooled arm, dispatches anything. Otherwise those walls would be the shipped path's walls labelled as the pooled scan's.

**7. Exactness on the machine (a VOID).**
- **Before the decodes,** the recipe runs `tests/pooled_scan.rs` and the finder's unit tests on the i9. The run is VOID if they fail.
- **Miri** is not rerun there: it passed on the finder's scalar path in the build container (`prototype.md`).

**8. The rest of the run, fixed here.**
- **The design:** 12 randomised blocks, traces in the first 3, seed 20261127. Unpinned, with the 100 ms pre-spin and a 120 s limit on each decode.
- **The code:**
  - the planner, `bench/pooled_scan.py`, which uses `bench/ideal_parts.py`'s driver unchanged;
  - the analysis, `analysis/pooled_scan.py`;
  - its checks, `analysis/test_pooled_scan.py`, on synthetic runs: a CONTINUE, each of K1 to K4 alone, each VOID, and the identities' exemption both honoured and violated.
- **The time:** about 25 to 35 minutes.
