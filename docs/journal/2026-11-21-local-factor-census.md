# The local-factor census: lead C of the RH-shape note is dropped

*21 November 2026. The first test of lead C ("Local–global: a catalogue of local factors") in
`2026-11-21-the-shape-of-the-rh-proof.md` (branch `docs/rh-lessons`), against the kill rule that
note registered before the data. Script: [`census.py`](2026-11-21-local-factor-census/census.py).*

## Verdict

**Killed, by a wide margin.** The kill rule: drop the lead if fewer than 50% of retained rows fall
in configurations that determine their output, using a catalogue that fits in 256 KiB. The best
figure over all six blocks, k = 1…4 and m = 4, 8, 16 is **10.5%** (block 0, k = 2, m = 4) in the
decoder's output direction, and 11.6% in the LF direction. Pooled into one catalogue for all six
blocks, it is at most 2.8%. Relaxing "determines" to 95% agreement changes almost nothing (best
10.5%; 11.6% LF). Even that 10.5% is mostly not a local factor at all (below): configurations with
at least two rows that determine their output cover at most **3.3%** of retained rows.

## What was measured

- **Data:** the six authenticated columns of branch `data/admitted-bwt` (each checked by SHA-256
  against `manifest.json` before use).
- **Retained rows:** every row of a maximal run of at most `LIMIT` = 16 rows, as `src/open.rs`
  keeps them. (Open also retains the marked row when it falls in a thick strip: one row a block,
  ignored here.)
- **Configuration at depth k:** the row's offset within its run, and the symbol and length of each
  of the k maximal runs from its own run, in row order. Lengths are exact (a thick run's length
  too); runs past the block's last are a sentinel.
- **Output:** the exact text, by LF inversion with stable rank (the data README's convention, the
  walk of `2026-11-20-working-backwards/costmap.py`). It spells `412157,"id":"Q5412157"},…` on
  block 0, as the README says, and LF is checked to be one cycle. Row x emits T[pos(x)] = L[x].
  - **fwd** (the kill rule's direction): the next m bytes in decoder output order,
    L[x], L[Ψx], …, as Open's `emit` writes them.
  - **bwd** (diagnostic): the LF direction, L[x], L[LF x], …
  - Windows wrap cyclically at the block's end (at most 15 rows a block).
- **Catalogue:** 256 KiB at 32 bytes an entry, a 16-byte output fragment plus up to 16 bytes of key
  and table slot: **8,192 entries**. The fragment is m bytes (at most 16); a key at k = 4 is 9 to
  13 bytes with 16-bit lengths. A tighter 2k + 17 bytes an entry would allow 10.5–13.8 thousand
  entries; that changes no verdict (see "uncapped" below, the upper bound for any catalogue size).
- **The columns:**
  - *configs:* distinct configurations among the retained rows.
  - *top:* share of retained rows in the 8,192 most populous configurations, whatever their
    output.
  - **det** (the kill rule's figure): share of retained rows in output-determining
    configurations (every row of the configuration emits the same m bytes), taking the 8,192 most
    populous such configurations: the best catalogue the rule allows.
  - *det2:* as det, among configurations of at least 2 rows. A one-row configuration determines its
    output trivially; storing it is storing the output, not a local factor.
  - *rel95* (diagnostic): as det, but a configuration counts when its modal m-byte word covers at
    least 95% of its rows.
  - *modal2* (diagnostic): in configurations of at least 2 rows, the share of their rows whose word
    is the modal one, the catalogue's best single guess. It is at least 50% for a 2-row
    configuration by construction, and most k = 4 configurations have 2 rows.
  - *uncapped:* det with no catalogue limit.

## The census, decoder output direction (fwd)

Shares are of each block's retained rows: block 0, 89,874 (10.0% of its rows); block 2, 112,070
(12.5%); block 85, 156,404 (17.4%); block 143, 196,841 (21.9%); block 207, 135,921 (15.1%);
block 210, 204,281 (22.7%). "pooled" is one catalogue for all six blocks' 895,391 retained rows.

| block | k | configs | top | **det** m=4 | det m=8 | det m=16 | det2 m=4 | det2 m=16 | rel95 m=4 | rel95 m=16 | modal2 m=4 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| block-0 | 1 | 5,614 | 100.0% | **2.2%** | 2.1% | 1.9% | 0.6% | 0.3% | 2.2% | 1.9% | 10.7% |
| block-0 | 2 | 46,092 | 57.8% | **10.5%** | 10.2% | 9.7% | 2.1% | 0.9% | 10.5% | 9.7% | 19.7% |
| block-0 | 3 | 68,780 | 32.6% | **9.7%** | 9.6% | 9.4% | 1.0% | 0.5% | 9.7% | 9.4% | 29.3% |
| block-0 | 4 | 86,368 | 13.0% | **9.5%** | 9.5% | 9.3% | 0.7% | 0.3% | 9.5% | 9.3% | 51.0% |
| block-2 | 1 | 6,262 | 100.0% | **1.6%** | 1.5% | 1.4% | 0.4% | 0.2% | 1.6% | 1.4% | 11.9% |
| block-2 | 2 | 66,604 | 46.5% | **9.0%** | 8.4% | 7.8% | 2.6% | 0.9% | 9.1% | 7.9% | 25.8% |
| block-2 | 3 | 92,995 | 24.3% | **8.3%** | 8.0% | 7.6% | 1.5% | 0.5% | 8.3% | 7.6% | 33.6% |
| block-2 | 4 | 108,865 | 10.2% | **8.0%** | 7.8% | 7.5% | 1.0% | 0.3% | 8.0% | 7.5% | 57.2% |
| block-85 | 1 | 5,976 | 100.0% | **1.2%** | 1.1% | 0.9% | 0.3% | 0.0% | 1.2% | 0.9% | 10.6% |
| block-85 | 2 | 72,117 | 53.4% | **7.4%** | 6.6% | 5.5% | 2.8% | 0.4% | 7.9% | 5.5% | 26.4% |
| block-85 | 3 | 119,026 | 27.6% | **7.6%** | 6.9% | 5.6% | 3.1% | 0.6% | 7.9% | 5.6% | 34.2% |
| block-85 | 4 | 146,497 | 11.6% | **7.6%** | 7.0% | 5.6% | 3.1% | 0.6% | 7.8% | 5.6% | 64.4% |
| block-143 | 1 | 6,830 | 100.0% | **0.8%** | 0.7% | 0.7% | 0.1% | 0.0% | 0.8% | 0.7% | 11.6% |
| block-143 | 2 | 90,059 | 52.2% | **5.5%** | 5.0% | 4.4% | 1.9% | 0.3% | 5.6% | 4.4% | 22.5% |
| block-143 | 3 | 153,458 | 22.8% | **5.1%** | 5.0% | 4.5% | 1.5% | 0.5% | 5.1% | 4.5% | 31.1% |
| block-143 | 4 | 188,861 | 8.2% | **4.9%** | 4.8% | 4.4% | 1.1% | 0.4% | 4.9% | 4.4% | 53.9% |
| block-207 | 1 | 5,945 | 100.0% | **1.3%** | 1.2% | 1.1% | 0.2% | 0.0% | 1.7% | 1.1% | 11.7% |
| block-207 | 2 | 52,421 | 64.6% | **8.5%** | 8.2% | 6.2% | 3.2% | 0.3% | 8.6% | 6.2% | 21.6% |
| block-207 | 3 | 96,262 | 32.2% | **8.1%** | 7.9% | 6.5% | 2.8% | 0.7% | 8.1% | 6.5% | 31.7% |
| block-207 | 4 | 126,526 | 12.9% | **7.8%** | 7.6% | 6.3% | 2.4% | 0.5% | 7.8% | 6.3% | 56.2% |
| block-210 | 1 | 6,421 | 100.0% | **0.7%** | 0.7% | 0.6% | 0.0% | 0.0% | 1.5% | 0.6% | 10.7% |
| block-210 | 2 | 70,263 | 65.9% | **6.0%** | 5.7% | 4.4% | 2.5% | 0.5% | 7.0% | 4.4% | 19.2% |
| block-210 | 3 | 128,101 | 36.7% | **6.8%** | 6.6% | 4.7% | 3.3% | 0.9% | 6.9% | 4.7% | 26.3% |
| block-210 | 4 | 182,456 | 12.3% | **6.6%** | 6.4% | 4.6% | 3.0% | 0.8% | 6.6% | 4.6% | 53.9% |
| pooled | 1 | 9,258 | 99.9% | **0.2%** | 0.2% | 0.2% | 0.0% | 0.0% | 0.2% | 0.2% | 8.9% |
| pooled | 2 | 281,321 | 52.9% | **2.1%** | 1.7% | 1.1% | 1.6% | 0.2% | 2.7% | 1.2% | 19.7% |
| pooled | 3 | 564,132 | 26.2% | **2.8%** | 2.0% | 1.3% | 2.5% | 0.5% | 2.9% | 1.3% | 23.5% |
| pooled | 4 | 761,240 | 6.9% | **2.7%** | 2.0% | 1.2% | 2.4% | 0.5% | 2.8% | 1.2% | 42.8% |
## The LF direction (diagnostic), and the uncapped figure

| block | k | det m=4 | det m=16 | det2 m=4 | rel95 m=4 | det, uncapped, fwd m=4 |
|---|---:|---:|---:|---:|---:|---:|
| block-0 | 1 | 2.3% | 2.1% | 0.7% | 2.8% | 2.2% |
| block-0 | 2 | 11.6% | 10.3% | 3.5% | 11.6% | 46.1% |
| block-0 | 3 | 10.5% | 9.7% | 1.9% | 10.5% | 69.6% |
| block-0 | 4 | 9.9% | 9.6% | 1.2% | 9.9% | 93.7% |
| block-2 | 1 | 1.6% | 1.4% | 0.3% | 1.9% | 1.6% |
| block-2 | 2 | 9.7% | 8.6% | 3.4% | 9.9% | 53.4% |
| block-2 | 3 | 8.8% | 8.2% | 2.1% | 8.8% | 78.2% |
| block-2 | 4 | 8.1% | 7.9% | 1.2% | 8.1% | 96.0% |
| block-85 | 1 | 1.2% | 1.1% | 0.3% | 1.3% | 1.2% |
| block-85 | 2 | 7.9% | 7.0% | 3.4% | 8.3% | 37.9% |
| block-85 | 3 | 7.7% | 6.9% | 3.3% | 7.9% | 72.4% |
| block-85 | 4 | 7.5% | 6.8% | 3.0% | 7.7% | 93.0% |
| block-143 | 1 | 0.9% | 0.7% | 0.3% | 1.3% | 0.8% |
| block-143 | 2 | 6.1% | 5.0% | 2.8% | 6.3% | 37.9% |
| block-143 | 3 | 5.5% | 5.0% | 2.0% | 5.5% | 71.8% |
| block-143 | 4 | 5.1% | 4.8% | 1.4% | 5.1% | 94.1% |
| block-207 | 1 | 1.5% | 1.3% | 0.4% | 2.2% | 1.3% |
| block-207 | 2 | 9.4% | 7.8% | 4.4% | 9.6% | 32.9% |
| block-207 | 3 | 8.7% | 7.7% | 3.5% | 8.7% | 64.5% |
| block-207 | 4 | 8.2% | 7.3% | 2.9% | 8.2% | 91.3% |
| block-210 | 1 | 1.8% | 1.1% | 1.1% | 2.6% | 0.7% |
| block-210 | 2 | 7.3% | 6.6% | 4.2% | 8.3% | 29.1% |
| block-210 | 3 | 7.2% | 6.7% | 4.0% | 7.3% | 57.8% |
| block-210 | 4 | 6.8% | 6.3% | 3.3% | 6.8% | 85.9% |
| pooled | 1 | 0.3% | 0.2% | 0.1% | 0.5% | 0.2% |
| pooled | 2 | 2.7% | 2.0% | 2.3% | 3.4% | 24.4% |
| pooled | 3 | 3.3% | 2.3% | 3.1% | 3.4% | 59.0% |
| pooled | 4 | 2.9% | 2.2% | 2.7% | 3.0% | 80.3% |
## Reading it

- **No configuration predicts its output.** At k = 1 a configuration (offset, symbol, length) has
  16–32 rows on average, and almost none of them agree beyond the first byte, which the
  configuration's symbol fixes: modal2 is 8–12% at m = 4. Deepening k splits the configurations,
  and they become determining only by becoming singletons. At k = 4, 86–96% of retained rows are
  in a configuration of their own (the uncapped column), and the catalogue's 8,192 entries then
  cover only 8,192 rows.
- **That is why det plateaus at k ≥ 2.** It is bounded by 8,192 / retained rows (9.1% on block 0,
  4.0% on block 210) plus the few multi-row determining configurations. det2, the part that would
  be a reusable local factor, is 0.0–3.3%, and 0.0–0.9% at m = 16.
- **One catalogue for all blocks is worse, not better:** pooled det2 is at most 2.5%. The local
  shapes do not recur with their outputs across the corpus.
- **Relaxing to 95% does not help.** rel95 is within 1.1 points of det everywhere: configurations
  either agree completely (mostly singletons) or disagree widely.
- **Neither direction works.** The LF direction is no better (best 11.6%, det2 at most 4.4%).

This is the catch the lead named, measured. The output bytes after a row are its context (in the
forward direction, the row's own sorted prefix), and that is fixed by where the row sits in the
global order, C[c] plus its rank, not by the shapes of the runs around it in L. The run structure
near a row says how the rows move under LF, not what they spell.

## Caveats

- **Retained rows are fewer than lead C may have assumed:** 10–23% of the rows of these blocks.
  Even a perfect local factor would act on that share of the walk; the census does not reach the
  question.
- **Configuration as specified only.** Other local keys (runs in the F column, the run's rank
  within its symbol, the neighbouring rows' contexts) were not tried. Any key that carries rank
  information is no longer local in the lead's sense.
- **The catalogue rule is generous:** det takes the best 8,192 determining configurations knowing
  the output. A real catalogue would be built before decoding, or per block at a cost.
- **Exact shares, no sampling.** The census is deterministic; the figures are counts, with no
  uncertainty beyond the choice of definitions above.
- **Reproduce:** `git archive origin/data/admitted-bwt admitted-bwt | tar -x -C <dir>`, then
  `python3 census.py <dir>/admitted-bwt` (numpy; about 75 s on one core). It writes
  `census.json` to the working directory and prints every block's full table, both directions.
