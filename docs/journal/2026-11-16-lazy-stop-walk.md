# 2026-11-16 · Pathfinder · Stop the lazy induction and walk the rest: byte-exact, and +97 instructions a letter at n/32 against a gate of +20

Direction: MONODROMY's HANDOFF 6915365756 on #145, from its result LOG 6915351608 and its
entry `docs/journal/monodromy/2026-11-16-stop-and-walk.md` (#482). The Coordinator queued it
first (LOG 6915367876). The gate came with it, from MONODROMY and the Coordinator: if the built
kernel's whole-decode instructions rise by more than about 20 a letter at n/32, it stops there,
before any i9 run.

The arms are on branch `perf/lazy-stop-walk`, head `a337f307da882f4c7cfcd1ff5887b9add0c03a83`
(kept). The patch against master `f7e0d919` is `2026-11-16-lazy-stop-walk/stop-walk.diff.txt`
(SHA-256 `ba0c6057…fd57e8514f`). The counting driver is `count.rs.txt`.

**The arms are byte-exact, and they cost +97 instructions a letter at n/32. So they go no
further.** The result comes first; the design and the evidence follow.

## The result

Callgrind, the whole decode of `corpora/wikidata.bz2` by `decode_with`. The denominator is the
11,090,867 letters of its 196 blocks the shipped gates admit, MONODROMY's own count.

| | shipped lazy | stop at n/16 | n/32 | n/64 |
|---|--:|--:|--:|--:|
| whole decode | 6.270 G | 7.789 G | 7.348 G | 7.115 G |
| **Δ a letter** | | **+137** | **+97** | **+76** |

**At n/32, a letter:**

| part | shipped | n/32 | Δ |
|---|--:|--:|--:|
| induction | 194.0 | 192.9 | −1 |
| writer, or pass 2 | 49.5 | 103.6 | +54 |
| pass 1 | | 16.1 | +16 |
| reading the stopped exchange, ranking | | 11.3 | +11 |
| zeroing (the exact words' scratch) | | | about +12 |

**What MONODROMY's model got right: the tail.** The cells the stop never makes cost 71
instructions each, the eager figure, and save 19 a letter. Tail share times eager cost, the
model's figure, gives the same.

**What it did not count.**
1. **The stop costs every cell, not just the tail.**
   - An instantiation of the induction that can never stop (`STOP` 30) is 2.353 G against the
     shipped 2.151 G: about 10 instructions a cell, 18 a letter.
   - The loop is at its register limit (`2026-11-16-lazy-overhead.md`). Every way of knowing
     the domain tried cost about the same:
     - per cell, in a register;
     - per step branch;
     - summed on the top front's reads, in memory.
   - So the stop cancels the tail's saving.
2. **Pass 2 on the kernel's words is 104 a letter, not the model's 78.**
   - The prototype wrote from an arena built for it, whose building was not counted: each short
     word right-aligned in a 16-byte slot of its own, each rule a triple with its right factor's
     length.
   - The kernel's words are arena offsets and 16-byte rules, with the mark in their lengths.
   - Each long word is expanded again at every occurrence.
3. **Reading the stopped exchange, ranking and scratch:** about 23 a letter.

**At best,** with every lever (a free stop test −18, staging in place of pass 1 −14, pass 2 at
the model's 78 −26, the overheads halved −12), about +27 at n/32 and +14 at n/64.
- **That n/64 saves 17% of the word bytes,** against the ring's 29% (`2026-11-16-lazy-ring.md`).
- **The ring did not pay at +69 a letter,** and nothing says these bytes pay at +14.
- **So the levers were not built.** A cheaper pass 2 from MONODROMY's numeration line (a long
  word written once and copied, or places with no second walk) changes the arithmetic most.

**A free stop test may exist.** When the top front reads a stream place, only that place lies
right of it, so the domain is the sum of λ over the stream's places below `ti`: a prefix sum
fixed before the induction, with no state a cell. But it is exact only where the top front
reaches its stream, and whether such moments fall on moments both letters are whole often
enough in the tail is not counted.

# The arms as they were built

## What they are

`bwt::StopLazyCells16`, `32` and `64`, with forced twins for tests. They are variants of the
lazy kernel (`LAZY_STOP16..=LAZY_STOP64`, `src/cells_stop.rs`, included in `cells.rs`). The
default is unchanged: the shipped induction is `induce_lazy::<0>`.

1. **The stop.** `induce_lazy::<K>` stops at the first moment both fronts' letters are whole
   (after an equal step, about every other step) with the domain m ≤ n >> K. It records the two
   fronts' links, the streams' next places and the top stream's byte map, and returns.
2. **The stopped exchange.** At that moment no front holds part of a letter, and a front never
   leaves a live letter. So every live letter is one no front has read: a cell, in its two
   slots, which name each other, or a block letter at its stream place.
   - *How it is read:* both orders are read from the fronts' links and streams without changing
     them.
   - *What is built, in top order:* a bitmap of the letters' first points over [0, m) with a
     rank directory; each letter's offset to its bottom interval; its word's length; its word.
3. **Places.** T sends x in a letter's top interval to x + offset. For a point x of a letter
   whose word is w, place(T x) = place(x) − |w| (mod n): w lies just below x's place.
   - *The starts:* 256 evenly spaced, and x_μ, the one point of the marked letter (MONODROMY's
     Lemma 2).
   - *Pass 1,* eight lanes: each start's segment walked to the next start, summing the words'
     lengths.
   - *The ranking:* from x_μ. Its word ends at |w_μ| − j − `SHIFT` (mod n), where j is the walk's
     row's offset in w_μ, read from the marked word's rules. So the places are the output's
     places.
4. **Pass 2,** eight lanes: each point's word written ending at its place, right to left.
   - *A short word* is one 16-byte store ending at its place; *a rule* is expanded right to left,
     on a 32-deep stack on the frame (a heap stack past that).
   - *Where a store may reach:* 15 bytes below its word. So a store is used only where at least
     16 of the segment's own bytes lie below the word, and the word is written exactly
     otherwise, as it is within 16 bytes of place 0.
   - *The prototype's rule was not enough:* it kept each segment's top 16 bytes and put them
     back. Where a segment's last word is shorter than 15 bytes, the word above it reaches past
     it into the next segment, which another lane may have written already. As first built,
     about 150 bytes a block were wrong, in runs of at most 13.
5. **When it does not apply.** If the stopped exchange is not one cycle, pass 1 and the ranking
   see it: the segments' points and bytes don't add up to m and n, or the starts don't close
   into one cycle from x_μ. The block is then decoded again by the shipped induction and writer.
   A block whose induction ends before the stop is written by the shipped writer.
6. **Popcount.** Both passes run with the CPU's popcount where it has one, dispatched at run
   time. The shipped build targets the baseline x86-64.

## Evidence

- **Wikidata.** All 431 blocks of `corpora/wikidata.bz2` (`82f65ace…`), at every stop, are
  identical to the shipped lazy kernel. At n/16, n/32 and n/64, 354, 307 and 266 blocks were
  walked and none fell back. The rest ended their induction before the stop.
- **`tests/lazy_stop.rs`.** Every block of the test corpora and samples, at levels 9 and 1, is
  identical to the lazy kernel through `invert_runs`, with the output buffer holding the last
  block's bytes as a worker's does. Each file through `decode_with` gives libbzip2's bytes.
- **`cells::stop_tests`.** 4,000 arbitrary columns of 64 to 763 rows, most of them several
  cycles. Each arm gives the lazy kernel's result, refusal or bytes. The fallback is taken
  2,902 to 3,698 times an arm, and the walk 8 times.
- **Debug and release.** Both pass, canaries and debug assertions included. The debug build
  caught the rank mask's overflow at bit 63 (`(2 << b) − 1`, MONODROMY's; release wraps to the
  right mask): it is `u64::MAX >> (63 − b)` now.
- **Not done, since the arm stops at its gate:** Miri, the fuzz, the mutation pass, and the
  Guardian's review of the new unsafe code (RULES B5). These are owed before any adoption.

## Reproduce

```sh
git checkout a337f307da882f4c7cfcd1ff5887b9add0c03a83
cargo test --release --test lazy_stop --lib stop_tests
cp docs/journal/2026-11-16-lazy-stop-walk/count.rs.txt examples/zz_count.rs   # (on that commit)
cargo build --release --example zz_count
./target/release/examples/zz_count letters corpora/wikidata.bz2      # 11,090,867 admitted letters
for k in lazy stop16 stop32 stop64; do
  valgrind --tool=callgrind ./target/release/examples/zz_count $k corpora/wikidata.bz2
done
```
