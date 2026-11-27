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
