# 2026-11-15 · MONODROMY 14 · Equal words are neighbours: a duplicate's live twin is always the next letter in the bottom order. One O(1) comparison catches 99.85% of duplicates, with no false positives, and would cut the word store by 38%

Assignment: QUEUED 6901872654. Can letters with equal words be recognised from the induction's
structure, when the second is created, in O(1) and without a table?

**Answer: yes, structurally.**
- **Where a twin sits.** In the right-end induction, letters carrying one word have contiguous
  images in the bottom order (P). So a newly made word that some live letter already carries
  is carried by the new letter's bottom-order neighbour.
- **Which neighbour.** On every block counted, it is the next one (C).
- **What the check catches.** Comparing the new word with that one letter's word, together with
  the word made by the step just before, catches 99.85% of duplicates. The 0.15% it misses are
  words no live letter holds any more.
- **Its price.** The step-before half is free. The neighbour half needs the bottom order's next
  link, which Cells does not keep.

**Labels:** **P** proved · **C** counted (exact for its input) · **Conj** conjecture · **K** known.

## 1. Contiguity (P)

**Setting.** The right-end induction on [0, m). Each letter α has the word w_α, and every
point y of its image J_α satisfies

  rotation(y) = w_α · rotation(x), for the point x ∈ I_α that y is the image of

(Invariant I, §2 of `2026-11-10-rauzy-veech.md`: reading L along x's excursion spells w_α
backwards, and y is the row that excursion returns to). Here x is a row of the domain, so its
rank is below m. Every point of x's excursion strictly between x and y lies outside the domain,
so its rank is at least m.

**Lemma.** For any word w, the images of the letters whose word is exactly w form one contiguous
block of the bottom order.

*Proof.* Take a letter β with w_β ≠ w and a point y′ ∈ J_β lying in the block of rows whose
rotation begins with w, which is contiguous in rank order. Then w_β and w are comparable, one a
prefix of the other.
- **w_β a proper prefix of w,** say w = w_β · u.
  - For y ∈ J_α (w_α = w), compare rotation(y′) = w_β · rotation(x′) with
    rotation(y) = w_β · u · rotation(x). This compares rotation(x′) with u · rotation(x).
  - rotation(x′) is a domain row, rank < m. u · rotation(x) is a point of x's excursion, rank
    ≥ m. So y′ < y.
  - Every such β's image lies wholly before every exactly-w image.
- **w a proper prefix of w_β.** The mirror case: wholly after.

So nothing lies between two exactly-w images. ∎

**Corollary (P).** When a step makes a letter X whose new word some other live letter already
carries, that letter's image is X's bottom-order predecessor or successor.

## 2. Census (C, all 233 Wikidata blocks; 4 prose blocks)

`dup.rs.txt`, here, is the schedules simulator run in the right-end order, with a 61-bit hash
per word.
- **The check.** Every final word is checked to be a rotation of the block.
- **What it records.** At each step it records whether the new word was made before (a
  duplicate), and which letters at that moment carry it.

**Duplicates are 0.5297 of the steps.** That matches §5.6's Lyndon-SLP count of distinct rules,
0.470 S.

| where the twin is, as a share of duplicates | Wikidata | prose |
|---|--:|--:|
| the word made by the step just before | 0.423 | 0.448 |
| the bottom-order successor of the new letter (the letter that followed A, the step's top letter, before the step) | 0.576 | 0.551 |
| the bottom-order predecessor | 0 | 0 |
| a top-order neighbour | 0 | 0 |
| no live letter (the word is held by no live letter any more) | 0.0015 | 0.0003 |

So the check "equal to the word just made, or to the bottom-next letter's word" catches
**0.99852** of duplicates on Wikidata and 0.99971 on prose.

**By kind of step (Wikidata):**

| step kind | duplicates ÷ S | twin = the word just made | twin = the bottom-next letter | no live twin |
|---|--:|--:|--:|--:|
| top wins | 0.205 | 0.330 | 0.057 | 0.0001 |
| bottom wins | 0.084 | 0.007 | 0.152 | 0.0001 |
| equal | 0.240 | 0.086 | 0.367 | 0.0013 |

The last three columns are shares of all duplicates.

**The twin is not reached through the step's own letters.** Where the twin is the bottom-next
letter, it was made by a step on the same top letter in 0% of cases. It was made by a step on
the same bottom letter in 5% (top wins), 67% (bottom wins) and 2% (equal). So a per-letter
"last cell cut" field would not find it; the bottom order's next link does.

**No false positives.** Equal words have equal Lyndon trees (§5.6's corollary). So if words are
shared, two words are equal exactly when their two child pointers are equal, and the comparison
is exact. A missed duplicate, one of the 0.15%, leaves a word unshared. A later comparison
against it can then miss, but never wrongly match, because equal pointers imply equal words.

## 3. What it is worth (C for the bytes; Conj for the kernel)

**The bytes.** A word costs 16 B as a rule when it is longer than 16 bytes, and its length in
the arena otherwise.

| | Wikidata (233 blocks) | prose |
|---|--:|--:|
| word-store bytes saved by sharing every caught duplicate | **0.381** (per block 0.216 / 0.382 / 0.435) | 0.436 |
| the same, by the step-before check alone | 0.205 | 0.199 |
| long rules among them: share of long rules saved | 0.295 | 0.037 |

Most duplicates are short words, so the arena gains more than the rules.

**Against the coordinator's estimate** of about 15.6 B a letter of words once lazy letters land,
the full check saves about 6 B a letter, and the step-before check alone about 3.2.

**The price in a Cells-style kernel (Conj, not built).**
- **The step-before check is free.**
  - *What it costs:* keep the last cell's (A word, B word) in registers, and compare the new
    cell's pair against it.
  - *What it is:* §6.3 of the Rauzy–Veech entry's "one word for consecutive bottom letters"
    (§5.6's 0.224 S), tried there for time and not kept. Measured on memory it is worth 20% of
    the word store.
- **The neighbour check needs the bottom order's next link.**
  - *Its cost in space:* 4 bytes a bottom slot. The bottom slots now live in the dead output
    buffer (#433), so they cost output-buffer room, not working set. At 12 bytes a bottom slot
    they fit in n bytes up to r/n ≈ 0.083, under the admission limit of 0.10, so a buffer
    would grow on the densest blocks.
  - *Its cost in time:* the relinks that each step makes, each with a next link to keep, plus
    one or two dependent loads for the neighbour's top slot and its word's two fields.
- **The two checks are designs to price, not built results.** Whether either pays at ten
  workers is for the Pathfinder to measure. With lazy letters the layout changes again, and
  the next link would go where the lazy design puts bottom slots.

## Reproduce

```
cp docs/journal/monodromy/2026-11-15-equal-words/dup.rs.txt examples/dup.rs
cargo run --release --example dup -- corpora/wikidata.bz2 1   # under a minute; also corpora/prose.bz2
```

**The outputs.** Per block, in `dup_wikidata.jsonl` and `dup_prose.jsonl`:
- `census`: [steps, duplicates, step-before, top-previous, top-next, bottom-previous,
  bottom-next, any neighbour, live anywhere, dead only, step-before or neighbour, A and B
  words equal];
- `kind`: per step kind, [step-before, bottom-next, other next, none];
- `orig`: the bottom-next twin's maker;
- `bytes`: [all, saved by the full check, saved by sharing every duplicate, long rules, long
  rules saved, saved by the step-before check].
