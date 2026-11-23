# The tiled zero-fill of the block reader's symbols: NOT ADOPTED

*23 November 2026. The result of `bench-block-symbols-tiled-zero-fill` on the i9-10900K.
Registration: `docs/journal/2026-11-23-block-symbols-tiled-zero-fill-registration.md` on
`bench/block-symbols-tiled-zero-fill` at `a257fe1a` (#786), frozen before the data and corrected
once per the acting Researcher's review (#145 REVIEW 7096174288, LOG 7096174751). Evidence:
`block-symbols-tiled-zero-fill/run-20261123T091653Z` on the `evidence` branch at `fd039ccf`,
SHA-256 `5a29d29a43c80eab78374cadec80a8047ff37015f4cca5aa35df344b599f4450`. Verdict LOG
7096821430.*

## The change

#785 (`39aaf55b`) zero-filled `read_item_with_scratch`'s symbol buffer in tiles of 8,192 symbols,
just ahead of the groups that write them, instead of one fill to `max_out` before the first group.
It was split out of #770 (the streaming parallel front end, NOT ADOPTED, LOG 7089082075). #770's
run had measured Wikidata at 0.978 and 0.977 at 4 and 6 threads, where its front end was inactive,
and the tiled fill was taken to be the cause.

## CI's condition (met)

| attempt | wikidata, 1 thread | text, 2 threads |
|---|---|---|
| 1 | −0.5% (−1.6, +0.6) | +4.6% (+0.0, +9.4), flagged |
| 2 | +0.4% (+0.1, +0.7), flagged | +1.2% (−1.7, +4.3) |
| 3 | +0.1% (−0.5, +0.7) | — |

Each attempt flagged a different one-thread cell slower by 0.4–1.7% (attempt 3: text 1t +1.0%,
records 1t +1.4%, random 1t +1.7%), and attempt 2's stage table put the scanner stage at +5.8%
(+5.4, +6.2) at one thread: consistent with a small real cost of the per-group branch.

## The i9 run

Head over base, wall [95%], 16 paired rounds, unpinned above one thread:

| threads | wikidata ×4 | text (guard) |
|--:|---|---|
| 1 | 0.997 [0.988, 1.005] | 1.002 [0.993, 1.012] |
| 2 | 1.008 [0.987, 1.029] | 1.007 [0.994, 1.020] |
| 4 | 1.002 [0.997, 1.007] | 1.005 [0.993, 1.018] |
| **6 (judged)** | **1.008 [1.003, 1.012] SLOWER** | 1.006 [0.993, 1.020] |
| 8 | 1.004 [0.978, 1.031] | 0.993 [0.958, 1.029] |
| 10 | 0.993 [0.977, 1.009] | 1.006 [0.984, 1.028] |

Peak RSS 0.994–1.013 at every count on both corpora. **Verdict: NOT ADOPTED** (6 threads not
FASTER; it is SLOWER).

## What it shows

- **The tiled fill is not where #770's gain came from.** At 4 and 6 threads it gives 1.002 and
  1.008, against #770's 0.978 and 0.977 on the same machine and corpus. The coordinator's
  attribution in LOG 7089082075 was wrong.
- **The fill on master is cheaper than the hypothesis assumed.** Symbol buffers return to the pool
  at their block's length, so master fills only `max_out` less the previous block's length; a whole
  fill happens only for a fresh buffer (the review's point). There was less to save than the
  mechanism claimed, and the per-group branch costs something everywhere.
- **The open lead:** #770's 2.2% below 8 threads came from what the split left out: the worker-loop
  refactor (`run_worker_job`), the `read_item_core` wrapper, or the layout they produce. Testing it
  is a new hypothesis with its own PR and registration, not a retry of this one (C2).

#785 and #786 close unmerged.
