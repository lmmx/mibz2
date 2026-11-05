# MTF's list front in two registers: the change, its chain, instructions and local timing

Written before any CI timing of this branch. Every timing below is local (a shared 4-core
Cascade Lake VM, Intel Xeon @ 2.80 GHz, `taskset -c 1`), indicative only. A decision rule
is still to be registered before the first bench comment.

## What we believed

`mtf_avx2` held the whole list in memory. Each index below 32 was a 32-byte load of the
list, a 32-byte load of the list a byte on, a blend, a 32-byte store, and a 1-byte store of
the new front. The next symbol's 32-byte load spans both stores, so it can't be forwarded
from either. It waits for them to commit. That store and reload is on MTF's serial chain,
and #170 and #172 showed that MTF's chain is wall time.

## The change

The list's first 32 bytes live in two registers across the symbols, `lo` (list[0..16])
and `hi` (list[16..32]). An index below 32 is four `pshufb` and two ORs:
`lo' = lo[c0] | hi[c1]` and `hi' = hi[c2] | lo[c3]`. The controls come from a static table
`MOVES`, one 64-byte line per index, and a control byte with its top bit set gives 0.
- **Below 16:** a rotation of `lo[0..=i]`, with `hi` unchanged.
- **16 to 31:** `lo' = [hi[k], lo[0..15]]` and `hi' = [lo[15], hi[0..k], hi[k+1..16]]`.

No store or reload is left on the chain. The new front's byte is read from `lo` (`vmovd`)
for the count and the output, off the chain. A GPR copy of it, `front`, serves the runs.

An index of 32 or more moves list[32..=nn] in memory, as seven aligned 32-byte chunks.
Each chunk is loaded and stored whole, so a later load of one is forwarded from its store.
Each chunk's "a byte on" is made in registers (`vperm2i128` and `vpalignr` with the chunk
before, or with `hi` for the first) and blended in up to nn, by a mask from a static table
of places (`TAIL`).
- **32 to 63:** the first chunk only.
- **64 or more:** all seven, the same work for any index, with no branch on it.

`lo` and `hi` then move a byte on, with list[nn] at the front.

Unchanged:
- the scalar `mtf_scalar`;
- the runs' handling, their two-store short runs (#170) and every check (`RunTooLong`,
  `BlockTooLong`);
- the byte list (#172);
- `mtf_checkpoints_avx2`, the cooperative teams' kernel. It keeps the old in-memory loop
  with its list of indices, and its doc now says so.

Variants tried for 64 or more:
- **`memmove` with a byte store**, as master did over the whole list: slower than master
  for 64 to 127 (9.7 vs 7.0 ns a symbol, synthetic, below);
- **16-byte chunks in a loop to nn's chunk:** worse on large indices, from the loop exit's
  mispredictions;
- **seven chunks for every index of 32 or more:** 7.5 ns a symbol for 32 to 63, against
  3.5 with the first chunk alone.

## The chain, before and after

The hot path is one index below 32 with no run before it. The listings are in `local/`,
cut from the `phases` builds.

| | instructions | loop-carried chain | llvm-mca znver4, cycles/iter | Skylake model |
|---|--:|---|--:|--:|
| master | 29 | a 32-byte load that waits for the commit of the 32-byte store and the 1-byte front store, then `vpblendvb` and the store again | 4.9* | 6.0* |
| this branch | 31 | `vpshufb` (1) → `vpor` (1): 2 cycles, through `lo` and `hi` | 5.2 | 6.0 |

\* llvm-mca models no store-forwarding failure. With `-noalias` it ignores the memory
chain; with `-noalias=false` it serialises every load behind every store (32 vs 27
cycles). So master's figure is only its throughput bound. For the branch, the four table
loads are written apart from their `vpshufb`. Folded, llvm-mca puts the load's latency on
the register operand's path (9 cycles), which the hardware does not. The branch is
throughput-bound, at about 5 cycles, with a 2-cycle chain.

Measured per symbol, on streams of index symbols only (no runs, so no dispatch
mispredictions). Each is the fastest of 20 MTF runs on 800k uniform symbols, ns a symbol;
the builds are aligned to avoid the JCC erratum (below).

| indices | master | this branch |
|---|--:|--:|
| 1-15 | 6.15 | 3.11 |
| 1-31 | 6.16 | 3.06 |
| 32-63 | 6.41 | 3.48 |
| 64-127 | 6.97 | 7.77 |
| 128-255 | 9.16 | 7.70 |
| 1-255 | 9.98 | 8.31 |

At 2.8 GHz, master's 6.2 ns is about 17 cycles a symbol, against a 5-6 cycle throughput
bound: the store and reload is most of it. The branch's 3.1 ns is about 8.7 cycles.

## Where the indices are

Index symbols by class, from the scanner's symbols:

| corpus | index symbols | below 16 | 16-31 | 32-63 | 64-127 | 128+ |
|---|--:|--:|--:|--:|--:|--:|
| wikidata (bench) | 15,148,085 | 84.8% | 13.7% | 1.3% | 0.23% | 0 |
| wikidata-independent | 15,070,348 | 84.8% | 13.6% | 1.4% | 0.26% | 0 |
| text | 1,864,553 | 85.3% | 9.7% | 4.1% | 0.88% | 0.01% |
| runs | 45,311 | 54.1% | 3.3% | 6.4% | 12.4% | 23.8% |
| random | 7,937,963 | 11.8% | 12.5% | 24.8% | 47.3% | 3.7% |
| records | 9,569,608 | 4.5% | 4.9% | 14.7% | 27.2% | 48.7% |

## Measured: instructions

Inclusive Ir of the driver's `measured` (one `decode_block` on a fresh `Scratch`, the
driver of `2026-11-05-deferred-flag-walk/`, rustc 1.97.0, fat LTO), and of `mtf_avx2`.
The input is `wikidata-independent.bz2`, and block 0 of the bench's `runs.bz2`. Master is
d1fd4405. Profiles are in `cg/`.

| block | decode, master | this branch | change | `mtf_avx2`, master → branch | output |
|---|--:|--:|--:|--:|---|
| 1 | 19,273,477 | 19,185,500 | −87,977 (−0.46%) | 2,976,785 → 2,888,808 (−2.96%) | identical |
| 41 | 18,647,563 | 18,568,920 | −78,643 (−0.42%) | 2,687,834 → 2,609,191 (−2.93%) | identical |
| 233 | 19,071,846 | 18,971,231 | −100,615 (−0.53%) | 3,041,366 → 2,940,751 (−3.31%) | identical |
| 400 | 18,651,077 | 18,567,182 | −83,895 (−0.45%) | 2,741,858 → 2,657,963 (−3.06%) | identical |
| 0 (54 B) | 6,881 | 6,813 | −68 | 2,272 → 2,204 | identical |
| 97 (56 B) | 7,321 | 7,239 | −82 | 2,514 → 2,432 | identical |
| 472 (70 B) | 8,069 | 7,967 | −102 | 2,968 → 2,866 | identical |
| runs.bz2 0 | 17,394,484 | 17,609,676 | **+215,192 (+1.24%)** | 2,772,082 → 2,987,274 (+7.76%) | identical |

The gain is in latency, not instructions. `runs.bz2` costs more instructions: 36% of its
indices are 64 or more, each now a fixed seven-chunk pass of about 50 instructions in
place of a `memmove` call.

## Measured locally: time

**The JCC erratum confounds local timing.** This CPU, like Skylake through Comet Lake
(the i9-10900K included), does not cache in its decoded-uop cache a 32-byte block that a
jump crosses or ends on. Master's MTF loop is bound by its store and reload, so where its
jumps fall hardly matters. This branch's loop runs at the front end's speed, so it does:
- the same source, built twice, measured −23% (the `phases` build) and +10% (the
  Callgrind driver build) on the MTF stage against master;
- the driver build has three such jumps on the path of an index with no run before it,
  and the `phases` build none there.

Each figure below is given both ways: built as CI builds it ("default"), and with
`-C llvm-args=-x86-branches-within-32B-boundaries` for both arms ("aligned"), which takes
the erratum out of the comparison. CI's runners are AMD EPYC, which has no such erratum.

**`phases`, MTF + RLE2 stage, Wikidata corpus** (sha256 `82f65ace…` as `bench/wikidata.json`).
This is `bench/stages.py` with 15 rounds, 2 passes, pinned to CPU 1, its four arms
interleaved and reversed every other round. Every value is in `local/stages-pairs.txt`:

| build | master, ns/B | this branch, ns/B | paired change (95% CI) |
|---|--:|--:|--:|
| default | 0.930 | 0.712 | **−23.5% (−25.4%, −21.5%)** |
| aligned | 0.790 | 0.676 | **−14.5% (−18.2%, −10.7%)** |

The default master build is itself 18% slower than the aligned one: its own jumps hit the
erratum in the `phases` binary. The aligned −14.5% is the cleaner estimate. With aligned
builds, no other stage moved beyond noise (walk −3.4% (−9.6, +3.3), build −4.2%, scanner
−0.9%), and the `phases` total moved −4.8% (−11.0%, +1.9%).

Every round's MTF + RLE2 figure, in ns/B:

| round | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 |
|---|--|--|--|--|--|--|--|--|--|--|--|--|--|--|--|
| master | 0.876 | 0.856 | 0.909 | 0.884 | 0.864 | 0.892 | 0.953 | 0.946 | 0.941 | 0.970 | 0.930 | 0.990 | 0.992 | 0.959 | 0.993 |
| branch | 0.666 | 0.715 | 0.692 | 0.673 | 0.664 | 0.702 | 0.700 | 0.721 | 0.674 | 0.725 | 0.716 | 0.751 | 0.769 | 0.806 | 0.709 |
| master, aligned | 0.806 | 0.729 | 0.772 | 0.806 | 0.750 | 0.800 | 0.804 | 0.761 | 0.772 | 0.797 | 0.807 | 0.796 | 0.832 | 0.858 | 0.761 |
| branch, aligned | 0.703 | 0.706 | 0.659 | 0.658 | 0.739 | 0.712 | 0.602 | 0.661 | 0.702 | 0.677 | 0.633 | 0.681 | 0.709 | 0.643 | 0.649 |

**MTF alone, each corpus's blocks held in memory.** Each value is the median of five
runs (six for Wikidata, aligned), each run the fastest of three passes, in ps/B. The
change is the ratio of the medians. The values are in `local/mtf-only-*.txt`:

| corpus | aligned: master → branch | default: master → branch |
|---|--:|--:|
| wikidata | 658 → 575 (−13%) | 682 → 752 (+10%) |
| text | 1,879 → 1,756 (−7%) | 1,816 → 2,214 (+22%) |
| runs | 1,903 → 1,733 (−9%) | 1,836 → 2,116 (+15%) |
| records | 10,366 → 8,711 (−16%) | 12,305 → 11,395 (−7%) |
| random | 9,516 → 9,418 (−1%) | 8,983 → 10,855 (+21%) |

These binaries are the Callgrind driver's, whose default build has the erratum's jumps on
the branch's index path.

**The whole file on one thread** (`BzReader`, Wikidata corpus, 12 paired rounds, the driver):
- aligned: −4.1% (−11.2%, +3.7%);
- default: +5.8% (+0.2%, +11.7%).

## What it says

- **The chain is cut.** An index below 32 now costs about half what it did when no
  misprediction intervenes, with the same bytes. MTF's chain was most of master's
  per-symbol cost.
- **Aligned, every corpus's MTF is faster:** 1% to 16%, and Wikidata's stage by 14.5%.
- **The cost is a front-end-bound loop.** On CPUs with the JCC erratum, its speed now
  depends on where the compiler puts its jumps, and in one build it was slower than master
  on every corpus. CI's AMD runners will show the change without that confound. The i9
  would not.
- **What binds MTF now.** On Wikidata, the dispatch between RUNA/RUNB symbols and index
  symbols mispredicts. Callgrind's branch simulator, on block 1 of
  `wikidata-independent.bz2`, puts 30,440 of this branch's 41,968 conditional
  mispredictions on `s <= 1` and 7,355 on `run != 0`, against 48,464 index symbols
  (master: 30,371 and 7,061 of 41,572).

## Correctness

`tests/mtf_kernels.rs` checks the detected kernel against `mtf_scalar`: the same result,
error kind, last column (on refusal too) and counts. It covers symbol streams built
directly, with no encoder:
- seven index distributions: below 16, 16-31, 32 or more, the register and chunk
  boundaries, uniform, text-like, and alternating classes;
- alphabets of 1 to 256 bytes, and the table both arbitrary bytes (repeats included) and
  sorted;
- runs of 1 to 22 symbols: the 22nd is `RunTooLong`, and 21 RUNBs, 2^22 − 2 bytes, are
  accepted;
- limits cut to refuse with `BlockTooLong` at a run and at a byte;
- streams with and without their end-of-block symbol, and symbols after it;
- every index in turn, each followed by a run of the new front;
- the scanner's symbols for every `samples()` shape at levels 1 and 9.

Eight mutants of the kernel, each failing both `tests/mtf_kernels.rs` (all four or three
of its four tests) and the existing `tests/mtf_checkpoints.rs`:
- a wrong `MOVES` entry for 16-31;
- a wrong rotation below 16;
- the first chunk's boundary byte from the wrong lane;
- six chunks for seven;
- the seven-chunk threshold at 96;
- `lo` shifted by two;
- a stale `front`;
- an inverted mask.
