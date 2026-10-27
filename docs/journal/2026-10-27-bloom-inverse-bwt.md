# Charles Bloom, "Faster Inverse BWT" (2021), against mibz2's results

A research source, read for #6 and #9.
- Source: https://cbloomrants.blogspot.com/2021/03/faster-inverse-bwt.html (blog post, March
  2021).
- Related: Kärkkäinen & Puglisi, "Cache Friendly Burrows-Wheeler Inversion" (CCP 2011),
  https://www.cs.helsinki.fi/u/tpkarkka/publications/ccp2011.pdf, read in full in
  [the literature reading](2026-10-27-literature-reading.md).

Below, **measured** marks figures from Bloom's post or from our CI, and **suggested** marks
ideas nobody has tested.

## What the post measures (Bloom's figures, measured)

**Setup.** Variants named *width × streams*: bytes out per step (1, 2 or 4, via LF, LF²
or LF⁴ tables) × interleaved chains (1, 4 or 8). Core i7 3770 at 3.4 GHz, one thread,
cycles per byte, without large pages.

| Data | 1×1 | 1×4 | 2×4 | 4×4 | 1×8 | 2×8 | 4×8 |
|---|---|---|---|---|---|---|---|
| enwik7 | 301.2 | 116.0 | 66.8 | 47.0 | 73.2 | 46.0 | 36.1 |
| dickens | 350.6 | 123.7 | 70.0 | 47.9 | 79.8 | 46.8 | 35.8 |
| rand_16M | 401.6 | 130.6 | 80.4 | **297.3** | 84.1 | 55.0 | **273.4** |

- **Interleaving.** "Going to 4 streams provides around a 2.7X speedup, 8 streams gets us
  about 4X" (enwik7). This is the mechanism of our flagged segmented walk (#4, #5), which
  runs 32 lanes with prefetching.
- **Wider steps.** They help compressible data, where adjacent table entries share cache
  lines ("dword output is only faster for compressible data where the cache coherence
  property above works well"), and fail badly on random data.
- **Construction.** It's described but not timed separately: "we now do 3/2 N iterations
  instead of N" for 2-wide, and an LF2 pass plus an LF4 pass for 4-wide.
- **Large pages.** They take enwik7 4×8 to 28.2 c/B.

## Against mibz2 (our measurements)

- The post's random-data collapse at 4-wide matches our ψ² port (#34), which was slower
  on every corpus. Its random and records arms were +49% and +21% at one thread.
- Bloom's best is about 28–36 c/B on 2012 hardware with 8 chains. mibz2's whole
  inverse BWT, build and 32-lane walk, is ≈13 c/B on Wikidata at 0.1.4 (local, one
  thread, a newer CPU). The two figures aren't directly comparable: different CPUs, data
  and builds.
- His 2-wide results beating 1-wide at 8 streams didn't reproduce for us. Our walk already
  keeps 32 misses in flight and is bound by instructions per step, so a wider step's extra
  per-step work cancels the halved step count (see
  [the pair walk](2026-10-27-pair-walk.md)).

## Precopy and LZP: what the post says, and what it doesn't

- **Precopy (KP 2011): not tested.** The post's own words: "'precopy' also makes N-stream
  interleaving much more complex. I have not tested it."
- **LZP: suggested, and encoder-side.** "BWT of long string recurrances can be sped up
  with an LZP pre-pass. We will assume this has been done where appropriate." An LZP
  pre-pass transforms the text *before* the forward BWT, at compression time, and its
  inverse runs after the inverse BWT. It changes the compressed format.
- **What this means for mibz2:** it can't apply to decoding standard bzip2 streams, the
  Wikidata dumps included. Our decoder sees only the BWT of the original text.

**A decoder-side reading, ours (suggested, untested):** discover repeat candidates before
the walk rather than during it. What a decoder can know before the walk is which adjacent
rows run in parallel: `t[s + 1] == t[s] + 256` (equal byte, link + 1). That is a
sequential scan of the T-vector.
- **What it can't tell:** how long a chain is in text order. That needs the walk's own
  link-following.
- **Cost against in-walk detection:** the forward-chain Precopy (#9) tests the same
  condition inside the walk with one compare against `t[s + 1]`, usually on the cache
  line the walk has just loaded. A separate pass would add a full sequential read of the
  table and gain nothing.
- **The earlier prototype's overhead** (#30) came from recording every walked row's
  position and searching backwards, not from detection. The forward-chain design doesn't
  record per row.
- **When this would matter:** if the forward-chain experiment shows the per-step compare
  is the cost. That is a measurable question: the walk's c/B on `records` and `random`,
  which have no chains, with the compare forced on against off.

## Limits of the evidence

- Bloom's figures are from one CPU, 2012-era, with blog-level detail. Construction isn't
  timed apart from the walk.
- Neither Precopy nor LZP is measured in the post.
