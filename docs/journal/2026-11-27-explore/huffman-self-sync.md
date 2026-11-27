# Huffman self-synchronisation and the selector-count obstruction (explore, 27 November)

**Status: measured, offline, exact.** This closes the "one cheap measurement first" of the
24 November FSM-composition lead
([`leads/structure-and-function.md`](../2026-11-24-algorithm-level/leads/structure-and-function.md),
step 5 and (b)): *the self-synchronisation length of bzip2's Huffman tables on the corpus, and
the selector-count obstruction*. Nobody had taken it. Verdict: **STOP** for within-block parallel
scanning by speculative self-synchronisation, scoped below.

Scripts (pure Python and numpy, no crate code): [`huffman_self_sync.py`](huffman_self_sync.py)
(A–D) and [`huffman_index_hypotheses.py`](huffman_index_hypotheses.py) (E). Each parses every
block's header and tables itself, decodes the true codeword boundaries, and fails unless every
block's data ends exactly at the next 48-bit magic.

Inputs: `wikidata.bz2` (SHA-256 `82f65ace…`, 431 blocks, 24,979,299 symbols,
93,636,008 data bits, 3.749 bits a symbol) and `ab gen` `text.bz2` (`7149b995…`, 45 blocks,
15,914,730 symbols, 3.568 bits a symbol). Seed 20261127. Single VM core; nothing here is timed.

## The question

A Huffman decoder is a finite-state machine over bits. Split a block into k chunks and decode
each from a guessed start state; if paths from wrong states quickly merge with the true one
(self-synchronisation), the k chunks run in parallel and are joined exactly where they meet. For
bzip2 the state is **(bit position, symbol index)**, not the bit position alone: the index picks
the 50-symbol group and therefore the table. Two things must therefore converge.

## Results (wikidata; text in brackets)

| | measurement | result |
|---|---|---|
| A | bit sync, oracle tables, from a random non-boundary bit (492,570 starts [45,000]) | codewords to first true boundary p50 **3**, p90 14, p99 **43**, p99.9 197, max 2,141 [p50 3, p99 34]; bits p50 13, p99 171, mean 25.4 [mean 16.0]. 1.2% hit an invalid (unassigned) code first [0.002%] |
| B | bit sync, one table guessed for the whole run | 99.1% reach a true boundary, p50 2, p99 25 codewords; but of all the guesses for one start, summed, only 0.78 [0.99] land where the guessed table is the true one |
| C | selector structure | 197 blocks with 2 tables, 1 with 4, 233 with 6 [all 6]; consecutive groups share a table 62.3% [42.9%] |
| D | start **on** a true boundary with index off by d; re-locks (true bit and true index) within 5,000 symbols | d = −1: **82.4%**; −25: 37.1%; −50: 15.9%; +1: 49.1%; +7: 5.1%; +25: **0.29%**; +49: 0.06%; +50: 0.05% [same shape: −1 85.8%, +25 0.7%, +50 0]. Median 1,143 symbols to re-lock |
| E1 | predict the group at a bit from the selectors and each table's expected code length (Kraft-normalised Σ2^−l·l), scaled to the block's known data length | group error p50 **8**, p90 18, p99 33, max 46 (symbols p50 404, p99 1,686) [text: p50 62 groups, 3,128 symbols] |
| E2 | all 801 index hypotheses s_hat ± 400 from a random bit; distinct (bit, index) states left (699 chunk starts [90]) | after 256 bits 734; 1,024 bits 592; 4,096 bits 335; 16,384 bits **131** (p90 259). **0/699 collapsed to one.** True state inside the window 343/699 [4/90] |

## What it means

- **The code itself contracts fast.** Bit-level self-synchronisation of bzip2's tables is a few
  codewords (median 3, ~25 bits mean) on both corpora. That part of the lead holds.
- **The index does not.** An index error is preserved exactly while the shifted selector
  schedule agrees with the true one, and changes only through desynchronised stretches. The
  error then performs a walk with a strong one-sided drift: low guesses re-lock often, high
  guesses almost never (D). The drift's sign means a desynchronised stretch emits *more*
  codewords than the true path over the same bits; why is not tested here.
- **The selectors don't locate a chunk.** Expected code lengths place a bit within ±8 groups
  at the median (±62 on text), i.e. hundreds to thousands of index hypotheses, and those
  hypotheses merge slowly: from 801, ~131 distinct states remain after 16 kbit (~7.5% of a
  217 kbit wikidata block), never one. Carrying them costs ~400 hypothesis-decodes per bit
  averaged over that stretch, roughly 30× the whole block's serial scan, to save at most a
  block's scan divided by k. Two orders of magnitude short.
- **And it is dominated anyway.** For any input with more blocks in flight than scanners, the
  2026-11-10 magic-candidate scan (`examples/par_scan.rs`; [record](../2026-11-10-parallel-scan.md))
  already parallelises the scan with **no redundant decoding**: a block's start state is exactly
  known (index 0) at its magic. Within-block speculation could add only single-block latency
  (one block of a small input, or a stream's last block), and even there the obstruction above
  applies. Both approaches move scan CPU onto the workers, which bind at the eight-worker knee
  on wikidata; the pooled-scan KILL's argument applies to both.

## Scope of the STOP

Stopped: within-block parallel Huffman scanning of bzip2 by speculative self-synchronisation,
with index hypotheses located by the selectors and expected code lengths, as a throughput lever.
Not stopped: any scheme that learns the index exactly by other means (none known: bzip2 stores no
per-group offsets); using the cheap bit-level sync for something else (for example, a recovery
tool for a corrupt block, which BWT inversion makes of little use anyway).

## A corpus property noticed on the way

`wikidata.bz2` is 12 streams (one a window) and `wikidata x4` is 48, each starting on a byte
boundary, while the real dump, being lbzip2-written ([encoder dependence](encoder-dependence.md)), is (inferred) a single stream. Nothing in the decoder uses stream heads today
(the scanner walks streams serially), but any future lever that does would be a corpus artefact
on the bench input. Such a lever must be judged on a single-stream corpus.

## Not shown

No timing, no Rust. E1's estimator is one simple choice; a better one (for example, per-group
code-length histograms) would shrink the window, but E2 shows the collapse itself, not the
window, is the expensive part. E2 starts at a random bit, not a true boundary (A says this adds
only a few codewords). D and E are capped at 5,000 symbols and 16,384 bits.

## Raw outputs

`huffman_self_sync.py wikidata.bz2` (default 2,000 starts a block):

```text
blocks: 431
A oracle-table sync: 486435/492570 synced, dead 6135, unsynced 0
  codewords to sync: p50=3 p90=14 p99=43 p99.9=197 max=2141 mean=6.57
  bits to sync:      p50=13 p90=60 p99=171 p99.9=453 max=4624 mean=25.42
  mean bits/symbol 3.749; symbols 24979299; data bits 93636008
B guessed-table: synced 2827122/2853140, at a boundary of the guessed table 480493; codewords p50=2 p90=8 p99=25 p99.9=69 max=2136 mean=4.13
  right-table syncs per start (sum over guesses), mean 0.784
C tables a block: {2: 197, 4: 1, 6: 233}
  consecutive groups sharing a table: 312331/501017 = 0.6234
D phase error: relocked 28068/119086 within 5000 syms (557 without ever leaving the true boundaries), died 2593
  by d: {1: [15071, 7395], 7: [14923, 768], 25: [14986, 44], 49: [14989, 9], 50: [15086, 8], -1: [14627, 12048], -25: [14700, 5453], -50: [14704, 2343]}
  symbols to relock: p50=1143 p90=4219 p99=4923 p99.9=4993 max=5000 mean=1709.54
```

`huffman_index_hypotheses.py wikidata.bz2 --chunks 3`:

```text
blocks 431, chunk starts 699, radius 400 (801 hypotheses)
E1 group-index error |g_true - g_hat|: p50=8 p90=18 p99=33 max=46 mean=9.54
   symbol-index error |s_true - s_hat|: p50=404 p90=917 p99=1686 max=2281
   true index inside the +-400 window: 343/699
E2 after   256 bits: distinct states p50=734 p90=791 max=801; collapsed to one: 0/699; true state present: 340/699
E2 after  1024 bits: distinct states p50=592 p90=758 max=801; collapsed to one: 0/699; true state present: 343/699
E2 after  4096 bits: distinct states p50=335 p90=608 max=801; collapsed to one: 0/699; true state present: 346/699
E2 after 16384 bits: distinct states p50=131 p90=259 max=520; collapsed to one: 0/699; true state present: 370/699
```

`text.bz2`: `huffman_self_sync.py --starts 1000`, then `huffman_index_hypotheses.py --chunks 2`:

```text
blocks: 45
A oracle-table sync: 44999/45000 synced, dead 1, unsynced 0
  codewords to sync: p50=3 p90=10 p99=34 p99.9=74 max=390 mean=4.69
  bits to sync:      p50=11 p90=34 p99=88 p99.9=181 max=882 mean=16.01
  mean bits/symbol 3.568; symbols 15914730; data bits 56782409
B guessed-table: synced 269997/270000, at a boundary of the guessed table 44629; codewords p50=3 p90=8 p99=21 p99.9=47 max=1137 mean=3.94
  right-table syncs per start (sum over guesses), mean 0.992
C tables a block: {6: 45}
  consecutive groups sharing a table: 136635/318272 = 0.4293
D phase error: relocked 2823/11250 within 5000 syms (27 without ever leaving the true boundaries), died 0
  by d: {1: [1346, 898], 7: [1375, 212], 25: [1403, 10], 49: [1405, 0], 50: [1402, 0], -1: [1415, 1214], -25: [1459, 353], -50: [1445, 136]}
  symbols to relock: p50=928 p90=4104 p99=4888 p99.9=4993 max=4998 mean=1553.17
blocks 45, chunk starts 90, radius 400 (801 hypotheses)
E1 group-index error |g_true - g_hat|: p50=62 p90=114 p99=135 max=136 mean=65.13
   symbol-index error |s_true - s_hat|: p50=3128 p90=5690 p99=6754 max=6790
   true index inside the +-400 window: 4/90
E2 after   256 bits: distinct states p50=700 p90=750 max=801; collapsed to one: 0/90; true state present: 4/90
E2 after  1024 bits: distinct states p50=492 p90=638 max=801; collapsed to one: 0/90; true state present: 4/90
E2 after  4096 bits: distinct states p50=242 p90=361 max=801; collapsed to one: 0/90; true state present: 4/90
E2 after 16384 bits: distinct states p50=92 p90=139 max=262; collapsed to one: 0/90; true state present: 4/90
```
