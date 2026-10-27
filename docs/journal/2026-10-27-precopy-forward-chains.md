# Forward-chain Precopy in the flagged walk: correct, and slower (#9, #36)

**Result: negative.** KP 2011's copy of repeats, done as the paper does it (forward chains,
per-chain records) in the lanes' order, copies 31.5% of Wikidata's bytes and is still
slower. CI A/B against 0.1.4 (10 paired rounds), with only blocks where at least a quarter
of the rows went in by run stores gated in:

| Corpus | Gated in? | Copied (local) | 1 thread wall | 4 threads wall |
|---|---|---|---|---|
| wikidata | yes | 31.5% | +18.0% | +6.0% |
| text | yes | 28.8% | +9.7% | +4.8% |
| pattern | yes | 0% | +20.4% | +19.9% |
| runs | yes | 0% | +11.1% | +9.5% |
| records | no | 0% | noise | noise |
| random | no | 0% | noise | noise |

Same bytes everywhere. The branch `perf/precopy-chains` is kept as the record.

## The design (the second attempt; the first was #30)

- **Detection.** A lane at row s also reads row s + 1's entry. `t[s+1] == t[s] + 256`
  means the same byte and a link one on (the run lemma), so the next rows are adjacent
  again. That is one step of a chain: one load, usually from the same cache line, an add
  and a compare. Nothing is recorded per row.
- **Marks.** A chain of ≥ 32 steps is recorded (source segment, offset, length, resume row)
  and its first row's neighbour j + 1 is marked (bit 30). Every row has one predecessor, so
  the lane that later reaches j + 1 is the only one that can. It copies the bytes and goes
  on from the row after the chain.
- **Safety.**
  - A chain ends at a flagged neighbour (a start or a mark), so no start is copied over.
  - A chain's source holds only walked bytes (a lane ends its chain before a copy), and it
    lies below the copying lane's write pointer, so copies never overlap.
  - Marked entries are put back after the walk.
  - A periodic text's multi-cycle ψ is handled, since every check is local.
- **Tests.** Run-heavy, periodic and random columns at 3 primary indices, 2 lane counts and
  3 segment counts. The fuzz loop ran 43,939 cases with 0 failures.

## Where the cost went

Wikidata, local walk alone:
- The first version used closures per step: +139%, and +180% even with nothing copied.
- With the step inline, the slow paths out of line and a sentinel entry instead of a bounds
  test: **+30% with 31.5% copied, against +22–27% with 0% copied** (records, random).
- Branch-free chain bookkeeping changed nothing (+31–47%), so mispredictions weren't the
  cost.

So the copies saved almost nothing. The steps a forward chain skips are the cheap ones:
- rows j and j + 1 walking in parallel stay on adjacent table entries, which are cached, or
  fetched for the neighbour test anyway;
- the walk's cost is in its scattered steps, the cache misses, which chains never cover.

Every walked step also pays the neighbour test in a loop that, with prefetching, is bound
by instructions per step.

## The gate

`pattern` (100% run stores) and `runs` (70%) copy nothing. A periodic text's chains are
all short or end at a mark or a start. The run-store share says how repetitive the last
column is, not how much the walk can copy.

## What this closes, and what it doesn't

- **Closed:** Precopy in this walk, in both forms tested (#30 backward, #36 forward), and
  the LZP lead (encoder-side; see [Bloom](2026-10-27-bloom-inverse-bwt.md)).
- **Not closed:** cheaper walk steps, the gap libsais shows
  ([the pair walk](2026-10-27-pair-walk.md)). Precopy's outcome says the same: removing
  cheap steps doesn't help, so the walk's per-step instruction cost and its misses are the
  target.
