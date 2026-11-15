# 2026-11-15 · Pathfinder · Cells' rules staged and stored past the cache

Decided by the Coordinator on REVIEW 6901387902 (#145). Decision rule: LOG 6901430556, posted
before any count. This entry holds the design, the safety argument (RULES B5), the counts and
the mutation pass. The timing is CI's paired bench on the PR; none was taken here.

## What changes, and why

The induction by cells writes one rule (16 bytes) for each cell whose word is longer than 16
bytes. It never reads a rule. The writer reads them all afterwards. On Wikidata the rules
are 9.1 bytes a letter, a fifth of the memory the induction touches (`2026-11-15-lazy-letters.md`).

Written as before, each rule's line is first read into the cache and then sits there, where
it displaces slots the induction still needs. So on x86-64 the rules now go past the cache:

1. **A rule is staged.** The induction writes it into a 1 KB ring on its stack, at the rule's
   offset modulo 1 KB. The ring stays in the first-level cache.
2. **A full ring is sent on.** Whole 64-byte lines go to the rules' region with non-temporal
   stores; the bytes before the first line boundary and after the last go with plain stores.
3. **The rest is sent when the induction ends,** before the writer runs, and a store fence
   follows on every way out.
4. **The fetch-ahead no longer fetches a rule.** It hinted the word of a letter a few slots
   on, whatever the word was. At one of its sites, 1.37 M of 11 M hints named a rule, and
   97% of those rules had already been sent. Left alone, the hints would have read the
   rules' lines straight back into the cache. Now a word kept as a rule hints the single
   bytes' line, which is always hot.

**Why staged, not stored one rule at a time.** A non-temporal store shares the core's fill
buffers with cache misses, and the induction misses on most steps. A line left half written
while misses arrive is written out in pieces.

**What does not change:** the cells, their order, every word and every offset; the rules'
region once the induction is over, byte for byte; the writer. Outside x86-64 the induction
writes each rule to its place as before (`STAGED` is false there).

**One layout change:** the rules' region now starts at a multiple of 16, so that no rule
wraps in the ring. That adds at most 15 bytes to the kernel's buffer.

## Safety argument (RULES B5)

New unsafe operations: two 8-byte writes into the ring; `send_rules`, which copies from the
ring to the kernel's buffer; `_mm_stream_si128` and `_mm_sfence`.

1. **A staged rule lies inside the ring.** The region's start is a multiple of 16: `slots`
   rounds it, and `induce_cells` refuses a layout where it is not. A rule's offset `rp`
   advances by 16 only. `STAGE` is a multiple of 16. So `rp % STAGE` is at most `STAGE - 16`,
   and the rule's 16 bytes end inside the ring.
2. **What `send_rules` is given.** A full send has `rp + 16` a multiple of `STAGE` and sends
   `max(rp + 16 - STAGE, rules) .. rp + 16`. The last send is
   `max(rp - rp % STAGE, rules) .. rp`. Both ranges:
   - lie in the rules' region: they start at or after its start and end at or before the
     end of the last rule made, and the region has room for a rule a cell, which the budget
     bounds;
   - lie within one multiple of `STAGE`, so they are contiguous in the ring, from
     `from % STAGE`;
   - hold only rules staged since the last full send. A ring place is written again only
     after `rp` passes the next multiple of `STAGE`, and the send at that multiple has
     happened by then.
3. **The non-temporal stores are aligned.** `send_rules` copies plainly up to the first
   64-byte boundary of the destination, so each `_mm_stream_si128` writes at a multiple of
   16 (a debug assertion checks 64). SSE2 is in every x86-64.
4. **Nothing reads a rule before it is sent.** The induction reads a word's bytes only when
   the cell's word is at most 16 bytes, and then both factors are words kept as bytes in the
   arena: a marked word carries the mark in its length and is never that short. The test
   hook that read a rule back from the buffer now takes it from the two factors, and the
   comparison test checks the whole region after the induction.
5. **Ordering.** The writer runs on the same thread after `induce_cells` returns, and a
   thread reads its own non-temporal stores correctly. The `Fence` guard runs `sfence` on
   every exit of the induction, an unwinding one included, so the stores are ordered before
   the buffer can reach another thread.
6. **Under Miri** the staging and `send_rules` run with plain copies; only the two
   intrinsics are compiled out.

## Evidence

**Tests** (all pass, 113 in the library):
- `cells_selective_exhaustive_state` and `miri_cells_selective_boundaries_reuse` compare
  the induction against the original state by state, and now also the rules' region byte for
  byte once it is over.
- In tests `STAGE` is 128 bytes, so short columns send too. The Miri-listed comparison
  asserts that one induction sends at least three times.
- Miri: the nine `cells::tests::miri_cells_*` tests pass.

**Corpus:** every bench corpus and Wikidata decode to the same bytes at 1, 4 and 12 threads,
and Wikidata's output equals master's. Cells admits the same 196 blocks and makes the same
20,456,299 cells.

**Mutation pass** (`2026-11-15-rules-past-cache/mutants.tsv`, script `mutate.py.txt`; debug
profile, so canaries and debug assertions are on): 16 mutants, 14 killed.

| survivor | why |
|---|---|
| the induction does not check the rules' offset | the check is a second line of defence: `slots` always rounds the offset, and the mutant that removes the rounding is killed |
| no store fence | no test can see store ordering across threads; the fence is there by the intrinsic's contract |

A first pass in the release profile left two more alive: a send that starts before the
region, and whole lines written twice. Canaries and debug assertions are off in that
profile. The debug pass kills the first; the second needed the alignment assertion, which
was added.

**Instructions** (callgrind, Wikidata, one thread, inclusive, per letter;
`2026-11-15-rules-past-cache/callgrind.txt`):

| | master | rules staged | and the fetch-ahead kept off rules (this PR) |
|---|--:|--:|--:|
| induction | 115.5 | 121.4 (+5%) | 133.9 (+16%) |
| whole decoder, whole file | 5.036 G | 5.069 G (+0.6%) | 5.207 G (+3.4%) |

- **Rule 2 of LOG 6901430556** expected at most +5% in the induction. The staging alone is at
  +5%. Keeping the fetch-ahead off the rules adds about 12 instructions a letter, and
  brings it to +16%.
- **I kept that guard.** Without it the experiment does not test its own claim: the hints
  bring the rules back into the cache.
- **The rule made this count a report, not a gate.** CI's bench decides.

## Reproduce

```
cargo test --release --lib cells
cargo +nightly miri test --lib cells::tests::miri_cells
python3 docs/journal/2026-11-15-rules-past-cache/mutate.py.txt .     # leaves the tree as it was
cargo build --release --example bzcat
valgrind --tool=callgrind ./target/release/examples/bzcat 1 < corpora/wikidata.bz2 > /dev/null
```
