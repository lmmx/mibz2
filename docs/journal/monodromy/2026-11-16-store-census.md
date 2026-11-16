# 2026-11-16 · MONODROMY 39 · A description of the lazy decoder's stores, per letter of Wikidata: about 58 distinct bytes written a letter, of which the output (16), the slots (about 9), the rules (9.2) and the arena (6.5) are the bulk. A short word costs 32 bytes of store width for 5 logical bytes. No lever is claimed

The Coordinator's item 1 (22:59). It is a description, to be read beside the Optimiser's
bench-w12-stores counters; if those fail their rule, nothing below ranks a store to cut.

**Labels:**
- **P:** read from the code on master (`src/cells_lazy.rs`, `src/cells.rs`, `src/runs.rs`);
- **C:** counted, with the source named;
- **M:** a stated assumption.

**Per letter** means per admitted run of Wikidata: n/r ≈ 15.9 rows a letter (r/n ≈ 0.063).

## The census

| phase | what is stored | store instructions a letter | store width, bytes a letter | distinct bytes written a letter | label, source |
|---|---|--:|--:|--:|---|
| induction | short words (≤ 16 B): two 16-byte `copy16` stores each, the arena advancing by the word's length | 2 × 1.272 = **2.54** | 32 × 1.272 = **40.7** | **6.48** | P; C: 1.272 short words and 6.48 arena bytes a letter (#503's census) |
| induction | rules: two 8-byte `wr64` stores each | 2 × 0.572 = 1.14 | 9.15 | **9.15** | P; C: 0.5716 rules a letter (#503) |
| | rules as W12 | | 6.86 | 6.86 | C (#503) |
| induction | slots, the pool's links and the spilled registers | about 29 | not counted | footprint about 8.9 (top slots 6.36, bottom 2.54); each slot rewritten as cells cut it | C: 32.9 memory writes a letter in all (lazy-overhead entry), less the word stores above |
| writer | each rule's first place (u32), filled with NIL then set | about 1.1 | about 4.6 | 2.3, in the dead top slots | P; C: 0.57 rules |
| writer | the output: each byte written once. Short factors are exact-length copies from the arena (`copy_small`), long factors `copy` from an earlier place | about 1 to 2 calls a rule | about 15.9 | **15.9** | P; M: overstores not counted |
| front | the run records, one u16 a run (`runs::front`) | 1 | 2 | 2 | P |
| finish | RLE1's output, the caller's bytes | | about 15.9 times RLE1's expansion | about 15.9 | P; M: expansion near 1 on Wikidata |

**The total of distinct bytes written** is about 6.5 + 9.2 + 8.9 + 2.3 + 15.9 + 2 + 15.9 ≈ **58 bytes a letter**, counting each slot once. W12's 2.29 B is about 4% of that.

## What the census says, as description only

- **The output dominates.** Writing the inverse BWT's output and RLE1's output is about 32 bytes
  a letter, more than half of all bytes written. Both are fixed by n.
- **Short words are store-instruction-heavy and byte-light.** At 2 stores a word and 32 bytes of
  width, they are the largest store width in the induction (40.7 B a letter) for its smallest
  logical part (6.48). Overlapping stores dirty the arena's lines about once, so their line
  traffic is near the logical bytes. Their store-buffer occupancy is near the width.
- **The slots' stores are the most numerous** (about 29 instructions a letter) and are not counted
  in bytes. They are the term the bench-w12-stores counters cannot separate from the words.

Read beside the Optimiser's counters:
- *If store-buffer stalls carry W12's gain,* the store instructions column is the relevant one.
- *If write-backs do,* the distinct bytes column is.

I make no ranking until the counters land.
