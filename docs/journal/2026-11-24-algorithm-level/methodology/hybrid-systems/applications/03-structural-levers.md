# Structural levers for reorganising the decoder

These are independent engineering interpretations, not a précis of the unread chapters of Goebel, Sanfelice and Teel. The book's Chapter 1 motivates an explicit description of flows and jumps; it supplies no decoder speedup or optimality result here.

Code and programme references use commit [9c38ad6c18237a1b923f9b762a1f634864d3b8a4](https://github.com/lmmx/mibz2/tree/9c38ad6c18237a1b923f9b762a1f634864d3b8a4). Numerical inputs come from the [stage 1 report](../../../math-targets/data/character-stage1/report.txt), rather than the older scanner-only profile. **Measured** means reported instrumented observations; **proved conditional** means arithmetic or an implication within an explicitly stated model; **argued** means supported interpretation; **conjectured** means a proposed benefit requiring a test.

The report marks instrumentation **PERTURBED at 10 and 20 workers**. Rounded medians are useful planning inputs, not one sample path. All numerical scenarios below freeze the measured service demands except the named change. They are not physical performance ceilings.

## Start with the existing programme

The [experiment programme](../../../briefs/experiment-programme.md) already orders scanner plumbing, stage 2 mechanisms, public-API worker/kernel recalibration, and the joint frontier. The structural work belongs inside that sequence. It should generate a different computation, ownership arrangement or admission rule; another naming scheme for the same queues is not a result.

At eight actual lazy/Open workers, stage 1 reports wall time 0.1944 s, scanner busy time 0.1850 s, and mean worker busy time 0.1847 s. At ten, these are 0.1993, 0.1903 and 0.1713 s. Their closeness at eight is why a scanner-only improvement can expose worker work immediately.

## 1. Change what crosses the scanner–worker boundary

**Proved from code.** [Scanner::next](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/stream.rs) calls `read_item_with_scratch` before producing a block event. Its payload is a fully decoded vector of Huffman symbols, defined by [Symbols](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/block.rs). This implementation couples discovering the next block boundary to interpreting all current-block symbols.

**Conjectured structural alternative.** Pass validated block descriptors or composable chunk-transition summaries to workers, retaining a small ordered validator. The obligations are exact boundaries, selector/group position, stream state, and error order. A search for block magic alone is not a valid descriptor construction. Within-block composition must carry the symbol count controlling table changes; bit synchronisation alone does not recover that state.

This revisits the existing [parallel-scanner brief](../../../briefs/scanner.md) and its #741/#770 history. Start with the queued input-copy/symbol-buffer fix, then price a different boundary against the changed baseline.

**Conditional price.** If that fix removes 15 ms of scanner service at ten workers and leaves the observed 9 ms gap above the largest demand unchanged, the planning estimate is:

```text
max(0.1903 - 0.015, 0.1713) + 0.0090 = 0.1843 s
```

That is 7.53% less runtime than 0.1993 s. At eight, the largest demand falls only from 0.1850 to 0.1847 s: the same assumption predicts 0.1941 s, not a 15 ms saving. These are conjectured scenarios, not forecasts. Remeasure worker service after the scanner change.

## 2. Change the intermediate object inside each worker

**Proved from code.** [decode_owned](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/lib.rs) already forms run records, inverts into an owned output buffer, and performs RLE1/CRC work with substantial fusion. Splitting every named activity into its own server would add transfers and live objects.

**Conjectured structural alternative.** Retain information acquired during run formation or inversion that eliminates later emission work; alternatively move a genuine independent subtask across a boundary. Charge acquisition, metadata, passes, dependent reads, fallback and buffer lifetime under the [existing referee](../../../essence.md). Preserving an invariant under a hybrid jump does not establish that its representation is cheap.

**Conditional price.** At eight workers, A7 emission occupies 0.0695 s per worker. A 20% reduction lowers mean worker demand from 0.1847 to 0.1708 s, but leaves the scanner's 0.1850 s demand unchanged. The frozen-demand model predicts no improvement in its bottleneck term. Combine that reduction with the hypothetical 15 ms scanner reduction: the largest demand becomes 0.1708 s. Preserving the original 9.4 ms gap gives 0.1802 s, or 7.30% less runtime. This illustrates masking; it does not establish that either implementation exists.

Use the already planned joint frontier to qualify candidates after the scanner changes.

## 3. Separate compute admission from retained output memory

**Proved from code.** [Shared::full](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs) blocks admission when:

```text
issued - taken >= window
OR
(ready_bytes >= 12 MiB AND issued - taken >= workers + 2)
```

The reader releases admission when it takes the next ordered result. The ready-byte threshold is soft: already admitted work continues, and retained pools and the reader's current buffer are outside that byte count.

**Conjectured structural alternative.** Give compute work and output storage separate reservations, each with an owner and a release event. A worker could return compute credit before ordered retirement, while its output retains a byte reservation. Preserve a bounded way for the earliest unfinished block to obtain the resources needed to complete; otherwise later results can monopolise memory and deadlock the redesign.

**Measured price signal.** Scanner admission waits total 3.8 ms at eight and 5.4 ms at ten, approximately 1.95% and 2.71% of their walls. Even treating all of those intervals as removable critical-path delay offers a small direct-time target; changing credits may also change contention, which this arithmetic does not price.

Mean admitted occupancy at eight is 19.47, with high-water mark 32. It does not justify shrinking the window to eight slots. Reuse stage 2's window-by-byte-gate contrast before implementing a new credit architecture. Every original sweep uses window $4k$, so it cannot attribute memory growth separately to workers and slots.

## 4. Transfer output ownership instead of copying bytes again

**Proved from code.** `StreamDecoder::read` copies into the caller's buffer. `read_block` already transfers a whole output vector and accepts a spent vector for recycling; `BzReader`'s `BufRead` path uses it. This is an existing mechanism to extend, not a newly discovered zero-copy algorithm.

**Argued opportunity.** Align the downstream sink with block ownership, retaining the ordered-output and corrupt-input contracts. Account separately for taken, consumed and recycled: admission release does not imply the caller has relinquished the allocation.

**Measured price signal.** Reader copy/decoder work is 35.9 ms at eight and 46.5 ms at ten; reader waits are 157.8 and 151.7 ms. These copy costs overlap the scanner and workers. Removing 46.5 ms of reader work does not imply 46.5 ms less wall time. In the frozen-demand model the reader is already below the scanner. The potential benefit is less traffic, contention or retention, requiring the real sink's measurement. Keep this behind scanner work unless that evidence changes its priority.

## 5. Make execution modes explicit and separate scanner capacity

**Proved from code; measured in the API sweep.** Requested threads do not specify the running machine. On the admitted i9, requests of ten or more construct seven lazy workers; requesting seven selects a different, eager kernel. Model actual workers, kernel, topology and window as discrete configuration, not a single requested-count parameter.

**Conjectured structural alternative.** Reserve scanner execution capacity separately from worker capacity. If a later adaptive mode switch is justified, specify its guard, ownership-preserving reset and minimum residence policy. Hysteresis may suppress repeated switches; it does not prove optimal throughput.

The instrumented mechanism sweep takes 0.2324 s at twenty actual workers and 0.1944 s at eight: 16.35% less runtime, with the twenty-worker result as denominator. This is not the public-default saving. In the separate public sweep, requested twenty takes 0.2095 s, requested eight 0.1924 s, and requested nine 0.1902 s: reductions of 8.16% and 9.21%, respectively, relative to requested twenty. These diagnostic comparisons motivate the existing adoption registration; they do not replace it.

## What the state must distinguish

Similar mean occupancy at eight and twenty workers accompanies very different starvation. **Argued:** mean occupancy alone is a poor predictor. A stronger, exact counterexample concerns the count itself: two states can each contain one ready block, but in one it is the next required result and in the other it is a later result. Only the first enables ordered consumption. Different residual work gives another same-count counterexample.

That proves this count does not determine the exact next transition. It does not prove that no useful scalar coordinate or reduced model exists. The practical task is to retain the smallest state that predicts completion, blockage and resource ownership under held-out interventions.

