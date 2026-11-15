# 2026-11-15 · Pathfinder · Lazy letters for Cells: the footprint falls as predicted, the instruction price does not hold, stopped before timing

Assignment: QUEUED 6900823509 on #145. Prices and decision rule: LOG 6901130348, posted before
any count of the candidate. This entry records the design, the candidate kernel as a patch,
its counts, and why it stopped at its own rule. **No timing was taken.**

## The question

Cells lays out a slot for every letter of a block: 20 bytes on top in the kernel's buffer, 8
on the bottom in the output buffer. MONODROMY counted that at most about a third of the
letters are ever cells at once (`monodromy/2026-11-15-width.md`). Can Cells hold slots for
those only, and does that cut a worker's footprint enough to move the team threshold below
10 threads?

## The design

A letter no front has reached needs no slot. It needs 5 bytes, all in the output buffer:

| what | size | order | read |
|---|---|---|---|
| its entry: length, byte, whether it is the walk's row | 4 B | bottom (sorted stably by byte) | by the bottom front, from the last back |
| its byte | 1 B | top | by the top front, from the last back |

**No map from one order to the other is stored.** The sketch in 6900062843 kept the
stable-sort permutation, 4 B a run. It isn't needed:
- within one byte, the top order and the bottom order agree, so the top front finds a
  letter's entry from its byte and a 256-entry counter;
- a letter is reached first by one front only, and that front never reads the entry again.
  So the dead entry holds the slot where the letter's cells begin, for the other front.

**Slots come from two pools,** handed out as pairs with a last-in-first-out free list. A front
that reaches a letter first takes a pair. A pair is given back at an equal step and when a
closed letter is dropped. The step itself is the shipped one, unchanged: same cells, same
order, same words at the same offsets.

The patch is `2026-11-15-lazy-letters/lazy-letters.diff.txt` (SHA-256 `85cdec84…d6ccc860`,
against master `f03b0fdc`). It keeps the shipped induction as a test-only reference.

## What was measured

Wikidata (`corpora/wikidata.bz2`, SHA-256 `82f65ace…c29915`), its 196 blocks Cells admits,
11,090,867 letters, 20,456,299 cells. Instructions are callgrind's, one thread, inclusive of
callees, per letter.

**The footprint claim holds.**

| per letter | shipped | lazy |
|---|--:|--:|
| top slots touched, kernel buffer | 20 B | 6.4 B (pool peak 0.318 of the letters; 0.295 to 0.337) |
| words (arena + rules) | 15.6 B | 15.6 B |
| **besides the output** | **35.6 B** | **22.0 B** |
| in the output buffer | 8 B | 5 B + 2.5 B of pool |

The pool's peak equals MONODROMY's width for the right end (0.318 r median) to three digits.

**The instruction price does not.**

| per letter | shipped | lazy, first build | lazy, second build |
|---|--:|--:|--:|
| first stage (`slots`) | 81.0 | 67.3 | 71.3 |
| induction | 114.3 | 216.6 | 195.3 |
| **first stage + induction** | **195.3** | **283.9** | **266.6 (+36%)** |
| writer | 54.0 | 50.4 | 49.5 |
| whole decoder, whole file | 5.02 G | 5.97 G | 5.76 G (+15%) |

The figures in LOG 6901130348 (59 / 111 / 28) left out callees and some inlined lines. The
table above is the like-for-like one. On either basis the candidate is about a third over.

**Correctness, as far as it was taken:**
- Wikidata decodes to the same bytes at 1, 4 and 12 threads (MD5 of the output equal to
  master's).
- `cells_exhaustive_arbitrary_columns` passes on the lazy route: every binary column to
  length 10, every row.
- Three tests that assert the shipped layout fail, as they must. They were not adapted.
- Not done: the state-by-state comparison against the original induction, a Miri-listed
  test, mutation.

## The rule, and what it says

Rule 2 of LOG 6901130348: the pool's peak at most 0.40 of the letters on every admitted block,
and the first stage plus induction at most 10% over the shipped count. If either fails,
report and stop.

- **Pool:** 0.337 at most. Passes.
- **Instructions:** +36%. Fails.

So the candidate stops here. It was not timed, locally or in CI, and no PR carries it.

## Why the price was wrong

I priced the induction at +25 instructions a letter. It costs +81. Per letter, from the
annotated lines of the second build:

| | instructions |
|---|--:|
| a place of the top stream: byte, counter, entry, tag test | about 10 |
| a place of the bottom stream | about 7 |
| a letter read from its entry: length, word, its entry's index | about 20 (both fronts) |
| taking its pair, linking it, tagging the entry | about 20 |
| giving pairs back | about 3 |
| the step's own lines, against the shipped build's | about +20 |

- **Every letter is visited twice in a stream,** once by each front: once untouched, once
  tagged. The shipped kernel reads a slot each time, with no test.
- **The last row is register pressure.** The lazy loop holds about 30 live values against the
  shipped loop's 19.
- **A floor.** The first three rows are work the design cannot drop. Counting only what
  they must do, I put the floor at +7% to +17% for first stage plus induction. The second
  build is at +36%, and I don't expect tuning to close more than half of that gap.

## What this leaves

**Kept:** the design and the two facts under it (the orders agree within a byte; a dead entry
can carry the head slot). They hold for any kernel that wants original letters implicit,
variant B included.

**Not answered:** whether, at ten workers on the i9, 38% less memory besides the output
outweighs a third more instructions in the induction. That is a clock question on that
machine. The rule I set does not allow me to take it, and I am not relaxing the rule after
its data. Whether to register a new experiment for it is the Coordinator's decision.

**A cheaper cut of the same kind, not tried.** The rules are 9.1 B a letter, a fifth of what
the induction touches, and the induction only ever writes them. Written with non-temporal
stores they would not pass through the cache at all, at no instruction cost. The writer
reads them back in order afterwards. Whether that pays is also a clock question, and a
ten-line patch.

## Addendum, the same day: the lean build (step 2 of the Coordinator's queue)

Rule: LOG 6901919943 on #145, posted before the count. The question was whether the design
reaches its floor: at most +10% over clean master for first stage plus induction.

**What was changed** (`2026-11-15-lazy-letters/lazy-letters-lean.diff.txt`, against master
`8a5e4601`; it replaces the first patch):
- a letter read from its entry inside the step takes its slots in that branch, so the slot
  test after every read is gone; only the two letters read together are still tested;
- an original letter's word is built with two additions.

**The count** (as above; clean master is a build with no counting line in it):

| per letter | clean master | lazy, second build | lazy, lean build |
|---|--:|--:|--:|
| first stage | 81.0 | 71.3 | 71.2 |
| induction | 115.5 | 195.3 | 181.0 |
| **first stage + induction** | **196.4** | **266.6 (+36%)** | **252.2 (+28%)** |

- The pool's peak is unchanged: 0.318 of the letters, 0.337 at most.
- Wikidata's output is identical at 1, 4 and 12 threads.

**The rule's line was 216.0. The lean build is at 252.2,** so lazy letters stay a recorded
result. The two levers gave 14 instructions a letter; the line is 36 further on. What is left
is the design's own work: two stream visits and one pair of slots for every letter.

## Reproduce

```
git checkout f03b0fdca71abf45133d0720802ff3d3255a567a
git apply docs/journal/2026-11-15-lazy-letters/lazy-letters.diff.txt
cp docs/journal/2026-11-15-lazy-letters/lazy_counts.rs.txt examples/lazy_counts.rs
cargo build --release --example bzcat --example lazy_counts
./target/release/examples/lazy_counts corpora/wikidata.bz2          # pool-wikidata.txt
valgrind --tool=callgrind ./target/release/examples/bzcat 1 < corpora/wikidata.bz2 > /dev/null
callgrind_annotate --inclusive=yes callgrind.out.*                   # callgrind.txt
```

Beside this entry: `lazy-letters.diff.txt`, `lazy_counts.rs.txt`, `pool-wikidata.txt`,
`callgrind.txt`.
