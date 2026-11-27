# Does the decoder's baseline depend on which compressor wrote the file?

The `explore` claim, 27 November 2026. Every system number in `frontier/DECODER.md` comes from
one file, `corpora/wikidata.bz2` (82f65ace…), made of Wikimedia's own blocks. The runt note
(`2026-11-26-explore/scan-lookups-and-runts.md`) left "the dump's compressor isn't identified"
open. This note recompresses the corpus's exact 209,700,196 decoded bytes (SHA-256 3f075e30…)
with three encoders and asks whether the scanner (the binding station) and the whole decode
behave the same on each.

Machine: a 4-vCPU Xeon @ 2.1 GHz VM with no PMU. It is bimodal (~45 or ~55 ms for the same
scan), so every figure below is from interleaved rounds, minimum and median reported. These
are shares and ratios, not adoption evidence.

## The files

| file | encoder | bytes | streams | blocks | runts (< 1,000 syms) | SHA-256 |
|---|---|---:|---:|---:|---:|---|
| dump | Wikimedia's (12 windows) | 11,854,933 | 12 | 431 | 198 | 82f65ace… |
| bzip2 | bzip2 1.0.8 `-9` | 11,863,758 | 1 | 234 | 0 | 817e6d89… |
| lbzip2 | lbzip2 2.5 `-9 -n 4` | 11,855,421 | 1 | 421 | 188 | 3241376c… |
| pbzip2 | pbzip2 1.1.13 `-9 -p4` | 11,877,838 | 234 | 434 | 201 | 60f48c09… |

**The dump is lbzip2-shaped.** lbzip2 reproduces the runt structure (one runt per 900,000-byte
input chunk whose RLE1 form exceeds a level-9 block; 188 against the dump's 198, the
difference being the twelve window cuts), the tables-a-block histogram (`[187,0,1,0,233]`
against `[197,0,1,0,233]`), the size to 0.004%, and the code-length histogram to 0.06 points
(1-bit codes 9.11% against 9.17%; reference bzip2 gives 9.70%). Bit identity of blocks was not
checked, so this stays an inference. Reference bzip2 writes no runts. pbzip2 writes one stream
for every 900,000-byte chunk, so it has runts too.

## The scanner is encoder-invariant

`huff_lookup_census` gives shipped (B = 10, K = 2) lookups a symbol of 0.5556, 0.5557, 0.5556
and 0.5558. The symbol counts agree to 0.01% and RUNA/RUNB is 39.4% on all four. In
`scan_runts`, best of 8 interleaved rounds (5 passes each), the whole scan takes 45.0, 44.5,
45.5 and 45.8 ms. The runts cost 0.23–0.33 ms of that. So the surface's S row (0.556 lookups a
symbol, about 1.8 ns a symbol here) carries over to reference-bzip2 and pbzip2 files. This was
an attempt to falsify "scan binds" as an artefact of the one encoder, and the attempt failed.

## The whole decode is not: the window counts places, and runts are places

In `examples/bzcat` (`StreamDecoder::new`), the same bytes decode much faster from the bzip2
file at bzcat's default window of 2 × threads (15 interleaved rounds):

| threads (slots) | dump median (min), ms | bzip2 median (min), ms | ratio |
|---|---|---|---:|
| 1 (2) | 556 (528) | 472 (373) | 1.18 |
| 2 (4) | 300 (282) | 256 (206) | 1.17 |
| 4 (8) | 210 (183) | 171 (160) | 1.23 |

The gap isn't CPU work. With `getrusage` at t1, user time is 509 against 491 ms, and on the
dump user+sys ≈ wall (no overlap) while on the bzip2 file user+sys exceeds wall by 66 ms.
`decode_with` on one thread gives the same inverse time on both files (325.9 against 326.8 ms,
`route_census`). Over the window, at t4 (7–9 interleaved rounds):

| slots | 4 | 6 | 8 | 10 | 12 | 16 | 24 | 32 | 64 |
|---|---|---|---|---|---|---|---|---|---|
| dump / bzip2 (medians) | 1.35 | 1.24 | 1.17 | 1.15 | 1.11 | 1.06–1.09 | 1.07 | 1.01 | 0.99 |

The gap closes only once the window holds about twice the full blocks. In `Shared::full`
every block takes one place, and on the dump 46% of places are runts. So 2 × threads slots on
the dump hold the full blocks of about 1 × threads. At production's window (`SLOTS_PER_THREAD`
= 4, 16 slots at t4) a residual of 6–9% remained in two of three sweeps (about 1% in the
first). `eight-worker-coupling.md` §(a) found the same dilution on the i9 at k = 8 (32 slots,
15–17 full) and priced runt-exempt slots at −0.76% there. On a small, oversubscribed machine
(7 threads on 4 vCPUs) the same mechanism is worth several times that.

## What follows

- **Encoder dependence is a property of runts, not of Huffman coding.** The real-world large
  `.bz2` files (Wikimedia dumps via lbzip2, pbzip2 archives) are the runt-heavy ones.
  Reference-bzip2 files hide the dilution. A bzip2-reencoded copy of the corpus is a cheap
  control for any window or plumbing experiment.
- **Harness drift:** `examples/bzcat.rs` still defaults to 2 × threads. #171
  (`2026-11-05-window.md`) moved production to 4 × threads. Timing bzcat on runt-heavy input
  therefore measures a starved pipeline (+17–23%). `bench/occupancy.sh`'s usage line still says
  "default 2 x threads", but the code uses `SLOTS_PER_THREAD`.
- Claim `runt-exempt-window` (idea) carries the lever, with its stop test.

Not done: no i9 run; no check of block bit identity against lbzip2; no trace of the t4 slot
waits; the window sweep used bzcat, not `BzReader::parallel`.
