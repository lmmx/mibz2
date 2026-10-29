---
description: "mibz2 beside other bzip2 decoders on master's latest benchmark run: paired ratios with 95% intervals, per corpus and thread count."
hide:
  - navigation
  - toc
---

# Alternatives

<div id="alternatives-hero" data-src="../../assets/alternatives.json" markdown>
Loading the chart…
</div>

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the
same rounds on one runner ([methodology](index.md)). A ratio above 1× is the alternative
taking longer (or more memory) than mibz2: there, mibz2 is faster. These are
decoder-level results; end-to-end pipeline results are in
[wikidata-pq's tracker](https://lmmx.github.io/wikidata-pq/guide/bench-history/).

| decoder | what runs | threads |
|---|---|---|
| pbz2 | Soof Golan's Rust decoder: `DecoderReader`, or `ParallelDecoder::with_threads` | block-parallel |
| lbzip2 | the C program: `lbzip2 -dc -n T`, read through a pipe | block-parallel |
| indexed_bzip2 | C++, from Python: `IndexedBzip2File(path, parallelization=T)` (`bench/ib.py`; the interpreter's start left out) | block-parallel |
| fbz | Answer.AI's Rust decoder: `fbz::Reader`, `threads: T` | block-parallel |
| lbzip2-rs | the `lbzip2` crate (pure Rust): `StreamingBz2Read`, on the CPUs it may run on (it takes no thread count) | block-parallel |
| libbzip2 | the reference C library, 1.0.8: the `bzip2` crate's bundled source, `MultiBzDecoder` (as `bzip2 -dc` and Python's `bz2` decode) | single-threaded |
| libbz2-rs | libbz2-rs-sys, the pure-Rust port of libbzip2, through the `bzip2` crate 0.6 | single-threaded |
| bz2ngrs | Softleif's safe-Rust port of bz2-ng, at a pinned commit: `DecompressReader` | single-threaded (one thread a stream) |
| bz2ng | bz2-ng's C, at a pinned commit: its `bzip2 -dc`, read through a pipe (the reference bz2ngrs ports) | single-threaded |

**Single-threaded decoders run at 1 thread only.** libbzip2, libbz2-rs, bz2ngrs and
bz2ng have no thread count, so they never decode on 4 threads. At 4 threads, their figures are
their 1-thread runs against mibz2's at 4 threads: in the chart, hollow points with
dashed whiskers, under "single-threaded (1 thread)"; in the tables, marked ¹.

**mibz2's inverse BWT against bz2ngrs's.** Below the decoders' tables, both kernels run
on the same Wikidata blocks, with the build, the walk and the output's assembly timed
apart. bz2ngrs and mibz2 store the same successor link in each 32-bit entry, but a
different byte beside it: bz2ngrs the row's first-column byte, mibz2 its last-column
byte (lbzip2's packing). The table says which phase a whole-decoder difference comes
from ([the study](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-bz2ngrs.md)).

Not included: `bzip2 -dc` and Python's `bz2` module, which are libbzip2 itself (its arm
stands for them); and bz2k, mibz2's own prototype, which it replaced.

--8<-- "docs/generated/alternatives.md"
