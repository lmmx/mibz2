# Station balance off wikidata: decision rule (frozen before the data, RULES C1a)

Written 27 November 2026, 05:40Z, before any `station_balance` output was read.

**Question.** `frontier/DECODER.md`'s knee (scanner and workers co-limit at k = 8, now tying
or flipping to worker-bound) is measured on one corpus, wikidata (17.7× compression). Scanner
work scales with symbols, worker work with output bytes, so the balance point should move
with the compression ratio and content. Where is it on other inputs?

**Measure.** `examples/station_balance.rs`, shipped code, one thread, this VM: per block the
fastest of P passes for the scanner's `next` (S) and `decode_block::<OpenCells>` (W). The
balance point is `k* = W / S` (the worker count at which W/k = S). Corpora, each `bzip2 -9`
unless named: wikidata (the dump's own blocks), wikidata re-encoded by `bzip2 -9`, prose,
pysrc, rustsrc, json, mdjournal (this repo's docs/*.md), logs, elf, dna, random. Three
interleaved rounds of 5 passes; median of rounds reported.

**Rule.** Let r_c = k*_c / k*_wikidata. Over the six realistic corpora
(prose, pysrc, rustsrc, json, mdjournal, logs) and elf:
- median r ≤ 0.75: the knee is corpus-dependent enough that a wikidata-only baseline
  over-states worker binding for typical inputs. Record a DECODER/surface note and open an
  `idea` claim for a multi-corpus station-balance baseline (and a scanner-priority caveat).
- 0.75 < median r < 1.33: wikidata is representative of station balance; record a null.
- median r ≥ 1.33: workers bind harder off wikidata; record and open the same `idea` claim
  with the opposite caveat.

**Wall check (measured, VM).** For any corpus whose VM k* ≤ 2.5, the shipped parallel
decoder (`bzcat`, window 4 × workers) at 3 workers must take ≥ 0.95 of its 2-worker wall
(a scanner plateau). If it does not, the k* prediction is falsified on this VM for that
corpus, and the note says so.

**Translation to the i9 (derived, labelled as such).** k*_c,i9 ≈ k*_wiki,i9 × r_c, with
k*_wiki,i9 ≈ 8 shipped (DECODER.md). It is a prediction, not evidence.
