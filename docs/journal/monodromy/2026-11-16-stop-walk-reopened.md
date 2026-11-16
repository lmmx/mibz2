# 2026-11-16 · MONODROMY 21 · Stop-and-walk reopened: stop on the cell budget the kernel already counts, and write with one lane, each live word expanded once and copied after. 33% fewer word bytes and about 20 fewer instructions a letter than shipped (prototype count)

Question: the Coordinator's LOG 6916535874. Stop-and-walk stopped at its gate: the Pathfinder's
kernel counted +97 instructions a letter at n/32 (REVIEW 6916500604). What would reopen it is a
pass 2 cheaper than the kernel's 104, or a stop that does not cost every cell 18.

**Labels:** **P** proved · **C** counted (exact for its input) · **M** modelled.

## 0. Result

- **The stop is free (P).** The lazy kernel already tests and counts down a cell budget on
  every cell (`if left == 0 { return Err(Refusal::Budget) } left -= 1`).
  - *How:* start `left` at min(budget, ⌈β r⌉). When it reaches 0 below the real budget, take the
    stop path rather than refuse.
  - *Cost:* the test that runs on every cell is unchanged. Only the cold path is new.
- **The two free stops the Pathfinder and I looked for are too late (C).**
  - *A fresh top read:* the first one after m crosses n/32 comes at m ≈ n/76 (median), and 12
    of 233 blocks have none.
  - *The streams running out:* that comes at m ≈ n/3,600 (median), with 96% of the words
    already made.
- **Pass 2 needs one lane (P), and costs 31 to 55 a letter, not 104 (C).**
  - *Why one lane:* at a budget stop there are 0.3 to 1.2 points a letter.
  - *How it walks:* the inverse exchange, upward from the row, writing in the kernel's own
    layout (a short word's bytes at an arena offset with 16 readable; rules).
  - *Each live letter's word is expanded once,* at its first place. Every later point copies it
    from there, 16 bytes a step.
  - *What goes:* there is no pass 1, no ranking, no starts and no segment boundaries.
- **The sum (M on C counts, 30 blocks):**

  | stop at | word bytes not made (233 blocks) | instructions a letter, against shipped |
  |---|--:|--:|
  | 1.4 r cells | 43% (6.5 B a letter) | −22 before reading the exchange; about −12 after |
  | **1.5 r cells** | **33% (5.0 B)** | **−32 before; about −22 after** |
  | 1.6 r cells | 23% (3.4 B) | −30 before; about −20 after |

  "Reading the exchange" is taken at 10 a letter, against the Pathfinder's 11.3, which also
  included ranking. A minus sign means fewer instructions than the shipped decoder.
- **Byte-exact (C):**
  - all 233 Wikidata blocks at β = 1.4, 1.5 and 1.6;
  - every 24th block at m ≤ n/16, n/32 and n/64, and at stream exhaustion.

  The check is against the general writer, itself checked against the block's text.

## 1. The stop (P, C)

**The counter.** `induce_lazy` keeps `left`, the cells still allowed, and refuses at 0. Any stop
written as a number of cells uses that test as it is. Admission has already been decided by then,
and the stop is ⌈β r⌉ cells. The stop path finishes the cell in hand, then reads the exchange.
- *If the fronts' letters are not whole there,* as the Pathfinder's stop requires, it sets
  `left = 1` and goes round once more, until they are. That is a few cells, once a block.

**Why β r and not m.** S is not known in advance, but r is, and S/r is 1.29 to 2.08 on Wikidata.

| β | blocks that stop | m / n at the stop (median) | points a letter |
|---|--:|--:|--:|
| 1.4 | 232 of 233 | 0.082 | 1.23 |
| 1.5 | 232 of 233 | 0.046 | 0.66 |
| 1.6 | 232 of 233 | 0.022 | 0.34 |

A block with S < β r never stops, and runs as shipped.

**The free stops that do not work (C, 233 blocks, `fresh_exhaust_wikidata.jsonl`).**
- *A stream read happens rarely in the tail.* The first top-fresh read after m crosses n/16
  falls at m = n/76 (median; tenth percentile n/2,500), and later still for n/64.
- *Stream exhaustion is free to test* (the kernel already tests a stream's end), but it falls at
  m/n = 0.00027 (median), with 95.5% of the word bytes made.

## 2. The one-lane emitter (P, C)

**Setting.** The stopped exchange read in bottom order:
- a bitmap of image starts over [0, m), with a rank directory;
- for each letter, the offset back to its top interval, its word's length, and its word.

**The walk.** From the point y whose place is known, x = T⁻¹(y) is found by the rank of y. Then
w_{α(x)} lies at [place(y), place(y) + |w|), and place(x) = place(y) + |w|. That is invariant I
read backwards. Every m steps end where they began, at place n.

**Writing upward (P).** A word is written as 16-byte stores from its left end.
- *A store reaches past the word,* into places not yet written, because places only grow.
- *So nothing is overwritten that a later step needs.* There are no segment boundaries, which
  is the overrun the Pathfinder found in the eight-lane version.

**First places (P).**
- *A live letter's first point* records its place, and the word is expanded there from its
  rules: a short leaf is one 16-byte store, and a rule pushes its two factors.
- *Every later point copies* ⌈|w|/16⌉ chunks from the first place.
  - *Why the copy is safe:* the two occurrences are disjoint excursions, and the earlier lies
    wholly below.
  - *Why it is correct:* a chunk reads past f + |w| only where it writes past place + |w|, which
    a later word rewrites.
- *The only table is the first place,* one u32 a live letter. It can live in the letter's dead
  slot.

**The rules' depth exceeds 64 on some blocks.** The first prototype's fixed stack of 64 failed at
β = 1.6, so the expansion stack must be bounded by the grammar's height, not by a constant.

**Instructions** (`instructions.txt`; callgrind, the emitter alone, hardware popcount):

| stop | points a letter | emitter a letter | a point |
|---|--:|--:|--:|
| 1.4 r cells (30 blocks) | 1.19 | 54.7 | 46 |
| 1.5 r cells | 0.64 | 38.3 | 60 |
| 1.6 r cells | 0.32 | 32.6 | 103 |
| m ≤ n/32 (10 blocks) | 0.41 | 31.3 | 77 |

**Against the kernel's pass 2 at n/32:** 104 for pass 2, 16 for pass 1, and ranking, which the
Pathfinder counted together with reading the exchange at 11.3. Its long words were expanded at
every occurrence; here a word is expanded once and copied after.

**Single-lane latency (M).** Each point is two dependent loads, the rank word and the letter's
record, both cache-resident: a directory of m/64 words and about 0.1 r letters. At 0.64 points a
letter that is roughly 10 to 20 cycles a letter. It is not measured here.

## 3. What remains uncounted

- **Reading the stopped exchange into the bottom-order bitmap and records:** taken at 10 a
  letter.
- **The marked row's offset:** the kernel's `SHIFT` and Lemma 2 of `2026-11-16-stop-and-walk.md`.
- **The mark bits in the kernel's rule lengths:** one mask a rule.

These are the Pathfinder's to count, in the kernel.

## Reproduce

```
cp docs/journal/monodromy/2026-11-16-stop-walk-reopened/stopwalk.rs.txt examples/stopwalk.rs
BETA=1.5 cargo run --release --example stopwalk -- corpora/wikidata.bz2 1 > b15.jsonl   # checks pass2s on every block
RUSTFLAGS="-C target-feature=+popcnt,+bmi1,+bmi2,+lzcnt" cargo build --release --example stopwalk
BETA=1.5 valgrind --tool=callgrind --toggle-collect=stopwalk::pass2s ./target/release/examples/stopwalk corpora/wikidata.bz2 8
```

**Fields** (`beta_*_wikidata.jsonl`):
- `beta_stop`: [m, step, word bytes made] at the stop.
- `hybrid`: [K, m, ns a point of pass 1, ns a point of pass 2 (local, not evidence), long
  words, bytes equal].
- `fresh_after`: [[m, steps after crossing] for n/16, n/32, n/64].
- `exhaust`: [m, step, live, word bytes made].
