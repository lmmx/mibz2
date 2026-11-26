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
3. **`bench-parscan-stream` (#770):** the bounded streaming version lost badly, at about 1.54× at 10 workers (`../briefs/scanner.md`). Its cause is not recorded on master. **Before any code, the prototype reads #770's PR and bundle, and names the cause.** If this design shares it, the design is revised here before the freeze.

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
- the chain's fallbacks per decode.

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
