# Which route production's inverse takes off wikidata, and what the dense band's refusals cost

The `explore` claim, 27 November 2026. Everything the frontier's surface rows C and E price is
the open kernel's (`OpenCells`), measured on one corpus, `corpora/wikidata.bz2` (82f65ace…),
where every one of the 233 full blocks is Open. This note asks how much of that holds on
other inputs: which route each full block takes, and what each route costs against the
one-core kernel (`Segmented`) on the same block.

## Method

`examples/route_census.rs` wraps `OpenCells` in a probe kernel and decodes each file on one
thread (`decode_with`). For every block it logs the route (`column`: the MTF stage kept the
column; `open`; `runs-not-open`: the run route, refused by Open's gates or its dense cap, so
`LazyCells` then `Segmented`), `n`, the run records and the inverse's time. Two builds:
`--features moonshot,census` routes exactly (the census's record shows Open took the block)
but its times carry the census's own O(n) pass; `--features moonshot` times production as
shipped. With `ROUTE_SHADOW=1` each run-route block is inverted again by `Segmented` straight
after, its output checked equal, and both times logged.

Corpora: wikidata, and seven built here from what the container holds, each `bzip2 -9`:
`prose` (6.6 MB, deduplicated text files under /usr/share/doc and Python dist-packages
metadata), `pysrc` (10.7 MB of the Python 3.12 stdlib), `rustsrc` (14.7 MB of the cargo
registry's `.rs` files), `json` (13.6 MB of this repository's `.json`/`.jsonl` files under
docs/ and bench/), `elf` (30 MB of /usr/bin binaries), `dna` (12 MB, uniform ACGT) and
`random` (8 MB). They are not checked in; their SHA-256 prefixes are in the delta. The
machine is a 4-vCPU Xeon @ 2.1 GHz VM, no PMU: shares and ratios only, not adoption evidence.

## Routes (exact, census build; full blocks, n ≥ 100,000)

| corpus | full | column | open | runs, not open | r/n p10 / p50 / p90 |
|---|---|---|---|---|---|
| wikidata | 233 | 0 | 233 | 0 | 0.063 / 0.068 / 0.115 |
| prose | 8 | 0 | 1 | 7 | 0.162 / 0.170 / 0.175 |
| pysrc | 11 | 1 | 1 | 9 | 0.226 / 0.294 / 0.314 |
| rustsrc | 16 | 0 | 8 | 8 | 0.053 / 0.158 / 0.229 |
| json | 16 | 5 | 7 | 4 | 0.053 / 0.070 / 0.217 |
| elf | 33 | 27 | 0 | 6 | 0.232 / 0.261 / 0.413 |
| dna | 14 | 13 | 0 | 1 | 0.724 |
| random | 9 | 9 | 0 | 0 | — |

Off wikidata, Open takes 17 of 107 full blocks (16%). The column route takes 55 and the run
route's fallback 35.

## The dense band refuses every block off wikidata

Open's dense band (r/n 0.15 to 0.28, `DENSE`) opens a block only if its first pass finds at
most min(256,000, 0.3 n) retained rows and 16,384 thick strips (`Cap::dense`). It was fitted to
wikidata's five blocks there (`docs/journal/2026-11-21-open-dense-admission.md`), and they open.
Off wikidata, **25 blocks lie in the band and all 25 are refused** (prose 7, rustsrc 8, json 4,
elf 4, pysrc 2), after the first pass has run to the cap.

That costs time (`run-route-blocks.tsv`, same block, production then `Segmented`):

| corpus, band, route | blocks | production inverse, ms | Segmented, ms | ratio |
|---|---|---|---|---|
| prose, dense, refused | 7 | 21.0 | 17.1 | 1.23 |
| rustsrc, dense, refused | 8 | 28.0 | 20.6 | 1.36 |
| json, dense, refused | 4 | 17.6 | 11.8 | 1.48 |
| elf, dense, refused | 4 | 16.4 | 14.5 | 1.13 |
| pysrc, dense, refused | 2 | 7.0 | 6.9 | 1.02 |
| wikidata, dense, open | 5 | 11.4 | 14.6 | 0.78 |
| wikidata, < 0.15, open | 228 | 314.6 | 657.0 | 0.48 |
| json, < 0.15, open | 7 | 7.2 | 18.8 | 0.38 |
| rustsrc, < 0.15, open | 8 | 17.6 | 23.3 | 0.76 |

(From the timing build: the census build's Open times include the census pass. Routes there
are by Open's count gates; the refused set is the census build's.) The refusals add about
19 ms over 25 blocks, ~0.8 ms a block, 13-48% of what the fallback itself costs. On the whole
of `prose` production decodes 1.09-1.32x slower than `Segmented` alone (best of 9, four
rounds); on the other non-wikidata files the two are within the VM's noise (±15%).

**The refusal is predictable from the run records alone.** The retained rows are the rows of
maximal runs of at most `LIMIT` (16) rows, plus the marked row. Merging consecutive records of
one byte (the cap-127 splits) and summing the thin runs' rows gives that count in one pass
over the records, before any of Open's work. Every refused block's count exceeds its cap (the
least, 257,495 against 256,000, an `elf` block; prose's 311,318 to 380,411), and every block
opened in the band is under it (wikidata's 199,189 to 231,600, which are the census's own `m`
figures in the dense-admission note). The thick strips, at most 8,081, never bind. So the
first pass on these blocks is wasted work a record-count pre-check would skip: claim
`dense-cap-precheck` (idea).

## What this means for the frontier

- Wikidata is the bench's corpus, and on it the result is as known: production's inverse is
  0.48x `Segmented`'s, and the whole decode 0.68x on one thread here.
- Off it, Open's room is a minority of blocks, and where it runs it wins as it does on
  wikidata (0.38-0.76x). Surface rows C and E price a part that about one block in six of these
  inputs reaches. The column route (`elf`, `dna`, `random`) never meets Open, so for such inputs
  the scanner and MTF rows are the whole room.
- The only regression found is the dense band's refusal. No other route loses.

## Caveats

Seven ad-hoc corpora, a few to 33 blocks each, chosen for spread, not representativeness. VM
timings, single thread, best of a few repeats; the shadow ordering (production first, then
`Segmented` on the same block) favours `Segmented` slightly through warm records. The 1.09-1.32x
whole-file figure for prose is noisy. The CI's `text` corpus was not examined.
