# 2026-11-16 · Pathfinder · Stop-and-walk reopened: through the instruction gate at +12 a letter, and not adopted. Slower at 8, 9 and 20 threads on production's plan; the memory relief is real but pays only at two workers a core

Direction: MONODROMY's HANDOFF 6916839849 on #145 reopened stop-and-walk, on both counts named
in the first attempt's entry (`2026-11-16-lazy-stop-walk.md`): a stop that costs nothing a cell,
and a cheaper writer. The adoption registration was frozen before data:
- first posted as REVIEW 6920355683;
- corrected by the Researcher (REVIEW 6921090225, accepted in full in LOG 6921203037);
- re-frozen as REVIEW 6921622210;
- decided PROCEED (LOG 6922045659).

Louis ran it on the i9 (RULES C5). The verdict is LOG 6922810916.

The arms are on branch `perf/lazy-stop-walk`, head `f3baa922a25c2144b82d98c3d88c8ec298d1333b`
(kept). #493 is closed unmerged.
- **The patch:** against master `e464cc91`, `2026-11-16-stop-walk-adoption/stop-walk.diff.txt`
  (SHA-256 `4cfa9a7c…aa1fc8160f7`).
- **The counting and timing driver:** `count.rs.txt`.
- **The run's verdict as the frozen adjudicator printed it:** `verdict.txt`.

**Not adopted.** The result comes first; then what was built, the instruction count, and what the
diagnostics say the next attempt must remove.

## The result

**Evidence:** `stop-walk/run-20261116T162328Z` on the `evidence` branch at `9010e519`. Its
SHA-256 is `54693ba6…7808817`, checked. The run is at `f3baa922`, with all 500 records, the full
roster. The frozen `bench/stop_walk.py` gives the same verdict on re-reading.

**The judged run:** production's decoder (`mibz2`) against the same decoder, plan and window with
the lazy kernel replaced by stop-and-walk at β = 1.5 (`mibz2-stop15`). It ran on 4 copies of the
Wikidata sample, 10 paired rounds, with no spinner sharing a decoder core. The table gives
`mibz2-stop15` over `mibz2`, wall time, with the 95% interval:

| threads asked | wikidata ×4 | text (guard) | peak memory, wikidata |
|--:|--:|--:|--:|
| 8 | 1.012 [1.004, 1.019] **SLOWER** | 0.996, inconclusive | 1.010 |
| 9 | 1.010 [1.004, 1.016] **SLOWER** | 1.000, inconclusive | 0.991 |
| 10 | 0.991 [0.980, 1.003] inconclusive (p 0.062) | 1.001, inconclusive | 1.020 |
| 12 | 0.997 [0.985, 1.010] inconclusive | 1.003, inconclusive | 1.029 |
| 20 | 1.015 [1.003, 1.027] **SLOWER** | 1.021, inconclusive | 1.010 |

- **No gain:** Holm rejects at no cell. The least p is 0.062, at 10 threads, against α/5 = 0.01.
- **Slower:** at 8, 9 and 20 threads.
- **Memory:** within the gate (1.10) everywhere.

**Verdict: NOT ADOPTED.**

## What the diagnostics say (they decide nothing)

**The memory relief is real.**
- *The dose* (CPU at 10 threads over CPU at 1, forced workers, one copy, round by round): stop15's
  is 0.947 [0.913, 0.982] of lazy's. β = 1.4 is 0.916 and β = 1.6 is 0.945.
- *What that means:* fewer word bytes made does mean less is lost to sharing the machine with nine
  other workers.

**It pays only at two workers a core.** Forced workers, stop15 over forced lazy:
- at 12 threads, 0.986, FASTER;
- at 20 threads, 0.965, FASTER;
- at 8, 9 and 10 threads, 1.018, 1.007 and 1.004.

Production's plan never runs more than one worker a core on the i9: 12 and 20 threads asked are ten
workers.

**The walk's latency is the cost.**
- *At 1 thread,* stop15 is 1.064 [1.032, 1.096], β = 1.4 is 1.104, and β = 1.6 is 1.057.
- *At one worker a core,* that cost is larger than the relief.
- *The local split of the time* (single-threaded, a VM, indicative): at β = 1.5 the induction saves
  about 7 ms a pass, while the walk is 94–103 ms against the writer's 54–63 and the reader adds
  16–19.
- *The walk's chain alone,* rank → popcount → offset → next point with nothing written, is 54 ms of
  the 95, about 20 cycles a point.

**The 10-thread case with the spinner sharing core 9** is 0.999, inconclusive.

**Divergence from the model** (PROTOCOL §11a, as the Coordinator's LOG records it). The model held
that 8 to 10 workers are memory-bound enough for 33% fewer word bytes to repay the extra
instructions. At one worker a core they are not. The relief shows only with two workers a core.
**The term to remove is the walk's serial latency.** That is MONODROMY's HANDOFF 6922450227: two
chains from x_μ, up and down, and a cheaper reader.

## What was built

**1. The stop rides on the cell budget's counter.**
- *What the arms do:* the induction stops at the first fresh pair once it has made ⌈β r⌉ cells
  (β = 1.4, 1.5, 1.6), when that is below the budget.
- *What did not work:* the HANDOFF's "set `left = 1` and go round" put the counter's cold path in
  the hot loop. That cost 16 to 38 instructions on **every** cell, even when the stop never fired
  (instantiations that can never stop, counted against the shipped one).
  - The cause: the loop is at its register limit, and the change spilled `left` to the stack.
- *What does:* the counter stays exactly as shipped. At each fresh pair, `left` is compared with a
  bound kept in the kernel buffer's padding below the top slots (`STOP_AT`). That costs about 2
  instructions a cell. So does keeping the stop's state anywhere the loop can see.

**2. The walk is one lane, downward from x_μ.**
- *Each point copies its word from one place,* 32 bytes whatever its length: a word kept as bytes
  from the arena; a rule from where the walk first met its letter and expanded it (once).
- *What is gone:* pass 1, the ranking and the starts.
- *Where a word is written exactly:* near place 0, where the walk wraps, and in its last 32 bytes.
- *The chain's offsets are their own array,* small enough for the first-level cache.
- *Neither this nor the branch-free copy* changed the walk's time measurably: the chain is the
  latency.

**3. The stopped exchange's reader adds nothing to a worker's peak memory.**
- A cell's bottom end is kept in its own bottom slot, which nothing reads after the bottom pass.
- The tables grow to a block's live letters, not to r.
- *Before this fix,* RSS was 109 against 85 MB at 4 threads locally.

**4. The adoption arm is production's decoder** (`StreamDecoder::new_lazy_as`), with only the lazy
kernel swapped. A unit test checks it on the emulated i9.
- *Diagnostics:* the forced-worker arms.
- *The adjudicator* checks the whole roster and every record before it gives a verdict. It refuses
  the Researcher's fabricated bundle (`bench/test_stop_walk.py`).

**Correctness:**
- *Byte-exact:* all 431 Wikidata blocks at each β, block by block against the shipped lazy
  kernel.
- *The test corpora and samples:* every block at levels 9 and 1, and libbzip2's bytes for each
  file.
- *Arbitrary columns:* 4,000 of them, with the fallback taken thousands of times, in release and
  debug.
- *CI:* green at the run's head.

## The instruction count (Gate 1)

Callgrind, the whole decode of `corpora/wikidata.bz2`, against master's lazy build (6.076 G). The
denominator is 11,090,867 admitted letters.

| | β = 1.4 | **β = 1.5** | β = 1.6 |
|---|--:|--:|--:|
| a letter, against master | +27.3 | **+12.3** | +8.1 |

The gate was +20, so β = 1.5 passed. Callgrind cannot resolve this finely, though: the same source
of the shipped induction counts 6.076 G to 6.288 G across three builds, about ±10 a letter.

**Where MONODROMY's model (−20) and the kernel differ, at β = 1.5,** a letter, against the same
build's shipped kernel:

| part | model | kernel |
|---|--:|--:|
| tail cells not made | −21.5 | −13.2 |
| walk against the writer's 49.5 | −11.2 | +2.6 |
| reading the stopped exchange | +10 | +18.9 |

1. **A tail cell costs about 38 instructions, not 71.** The tail has no stream reads.
2. **The walk's fixed path is about 31 instructions a point,** plus the first expansions.
3. **The reader's 18.9 against an object of 0.12 r letters:** MONODROMY suggests the streams'
   consumed extent is scanned. That is to be profiled (HANDOFF 6922450227 §2).

**The lesson the run adds:** an instruction gate is the wrong gate for a latency-bound loop. The
walk passed it and lost on time.

## Reproduce

```
git checkout f3baa922a25c2144b82d98c3d88c8ec298d1333b    # perf/lazy-stop-walk
cargo test --release --test lazy_stop && cargo test --release --lib stop_tests lazy_as_tests
python3 bench/test_stop_walk.py
git show origin/master:docs/journal/2026-11-16-stop-walk-adoption/count.rs.txt > examples/swcount.rs
cargo build --release --example swcount
./target/release/examples/swcount check corpora/wikidata.bz2          # byte-exact, block by block
valgrind --tool=callgrind ./target/release/examples/swcount b15 corpora/wikidata.bz2
./target/release/examples/swcount time corpora/wikidata.bz2 5         # per-block time, best of 5
just bench-stop-walk                                                   # the i9 run (RULES C5)
```

## After the verdict: two chains do not remove the latency (MONODROMY's HANDOFF 6922450227)

MONODROMY proposed splitting the walk's serial chain into two from x_μ: down by T, up by T⁻¹ through a bottom-order rank. It also proposed profiling the reader. Both were built on the arm, byte-exact on all of Wikidata and the tests, in release and debug. Neither gains.
- **The patch:** against `f3baa922`, `two-chains.diff.txt` (SHA-256 `1eb66259…36ab62a5`).
- **The walk's timer:** `walk-time.rs.txt` (with a few lines of timing in `walk_stopped`).

**The chains alone do overlap.** These are single-threaded on a VM, best of 5 passes over Wikidata at β = 1.5, the walk alone timed. Following the points and writing nothing:
- one chain takes 48.8 ms;
- two chains, a step of each a turn, take 28.4 ms.

**The walk does not get faster.**

| walk | ms |
|---|--:|
| one chain (the run's) | 88.5–93.5 |
| two chains in lockstep, writing as they go | 106 |
| two chains recording each point's letter, then a writing pass with no chain | 90.1–95.9 |
| the shipped writer, for scale | 54–63 |

- **Two chains in lockstep:** each chain's data-dependent branches (a word kept as bytes or a rule, the copy's length) mispredict, and every mispredict flushes both chains.
- **Separate passes:** the single chain's latency was already hidden behind the writes. The walk is bound by producing its bytes, not by the chain.
- **The instructions:** the two-pass version costs +31 a letter over one chain (the bottom rank, the record pass and the meeting); the lockstep version +17.

**Why the writing is the cost.** The walk writes one live word a point: about 33,000 points a block at β = 1.5, 27 bytes on average. The shipped writer expands the final word's rules once and copies whole repeated sub-words. It writes the same n bytes in about two thirds of the time. So the walk's floor is the writer's work plus its own per-point overhead, which no chain split removes. Against it stand the tail cells not made, about 8 ms a pass, and the memory relief, a dose of 0.947.

**The reader (HANDOFF §2).** At β = 1.5 it is about 20 instructions a letter, 160 a live letter (0.12 r), over three passes: the bottom order, the top order, and the records with their checks.
- *Not a stream scan:* the stream steps run about 1.5 M times against 1.33 M live letters.
- *The plain cut:* fusing the top pass with the record pass would save perhaps a third. It is not built, because it cannot close the gap above.

**What would change the arithmetic** is a walk that writes in larger units than one live word: copying a run of points whose words already lie together, as the writer copies a repeated sub-word. That is MONODROMY's to say whether the stopped exchange's structure offers one.
