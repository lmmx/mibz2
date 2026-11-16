# 2026-11-15 · Pathfinder · Shared words through a ring of eight: a quarter of the word store, three quarters more induction

Asked by the Coordinator on #145 after MONODROMY's count of equal words: price the ring of
eight in the kernel before sharing is closed. The ring lives in memory, not in registers,
so the register argument that closed the step-before check
(`2026-11-15-memory-lever.md`) might not apply. **Counts only. No timing was taken.**

## What was built

A scratch patch on master `caad2bd97`
(`2026-11-15-shared-ring8/shared-ring8.diff.txt`, SHA-256 `5f1ddc6d…afa2b818`). Not a
candidate: no tests, AVX2 assumed at build time.

- **The key** of a cell's word is its two factors' offsets, 32 bits each. Equal keys mean
  equal words.
- **The ring** is one 64-byte line of eight keys and one of the eight words made for them.
  Every cell compares its key with all eight (two 256-bit compares), and then takes the
  place of the oldest.
- **A hit** takes the earlier word's offset. `top` and `rp` do not advance, so the next
  word overwrites the bytes just written.
- **A word made with the walk's row is never shared,** and its key is not kept: the mark is
  in the length, not in the key. The first build kept it, and 181 blocks fell back.

## What it saves

Wikidata, the 196 blocks Cells admits (`2026-11-15-shared-ring8/counts.txt`):

| per letter | master | ring of eight |
|---|--:|--:|
| arena | 6.44 B | 4.25 B |
| rules | 9.14 B | 6.90 B |
| **the word store** | **15.64 B** | **11.15 B (−28.7%)** |

- **A third of the cells hit:** 26.2% as short words, 7.6% as rules.
- **Far more than the step-before check,** which saved 8.7%. MONODROMY's count was right
  that the equal words lie a few steps back, not one.
- The output is the same bytes at 1 and 4 threads.

## What it costs

Callgrind, one thread, inclusive, per letter, both builds with AVX2:

| | master | ring of eight |
|---|--:|--:|
| first stage | 80.0 | 79.9 |
| induction | 115.5 | 202.7 (+76%) |
| writer | 54.0 | 43.2 (−20%) |
| the three | 249.4 | 325.9 (+31%) |
| whole decoder, whole file | 4.957 G | 5.804 G (+17%) |

**Where the 47 instructions a cell go:**

| | a cell |
|---|--:|
| the ring's own lines: key, broadcast, two compares, two masks, the marked test, the hit's index, the word chosen, two stores | about 29 |
| `top +=`, which was one instruction and is now ten | about 9 |
| the rest of the loop, against master's | about 9 |

- **The ring's own work is the larger part.** Two compares are 2 of its 29 instructions.
  The key has to be built and broadcast, the two masks taken out and joined, and the ring
  written twice.
- **It does also carry state in registers.** The ring's pointer, the key, the hit and the
  hit's word are live across the step, and the loop had no register to give them. The
  second and third rows are that.
- **So the Coordinator's condition is met on both counts:** with no register pressure at
  all, the ring's own lines would still add about 54 instructions a letter to an induction
  of 115.

## Verdict

**On instructions, the measure in force when this was counted: it fails.**

| cut | memory besides the output | first stage + induction + writer |
|---|--:|--:|
| lazy letters (the `-lazy` arm, `2026-11-15-memory-lever/counts.txt`) | −13.6 B a letter | +52 instructions a letter (+21%) |
| ring of eight | −4.5 B a letter | +76 instructions a letter (+31%) |

The ring saves a third of what lazy letters save, for half as many instructions again.

**The measure changed the same hour.** The i9 run (LOG 6903318530) found lazy letters
faster than the shipped decoder at 8, 9 and 10 threads despite its instructions, and the
Coordinator ruled (QUEUED 6903426694) that a memory cut is now judged on peak memory and
wall time at 8 to 10 threads. On that measure the ring is not closed. It is a cut to stack
on lazy letters, and what it is worth there is smaller than the table above suggests:

| kernel | live memory besides the output | the ring would take |
|---|--:|--:|
| lazy letters | 22.0 B a letter (6.4 slots, 15.6 words) | 4.5 B (−20%) |
| lazy letters, rules in the output's spare capacity (#449) | 12.8 B a letter (6.4 slots, 6.4 arena) | 2.2 B of live memory (−17%), and 2.2 B of the tail |

It waits behind the production lazy kernel and the free-steps-first order, and is judged
by a clock on the i9 if it is built.

**What would make it cheap:** a way to find an equal word without a search a cell. None is
known. The key is only available once both factors are, which is at the cell itself.

## Reproduce

```
git checkout caad2bd97e236905a15a970b3a3a945877b4316e
git apply docs/journal/2026-11-15-shared-ring8/shared-ring8.diff.txt
RUSTFLAGS="-C target-feature=+avx2" cargo build --release --example bzcat
valgrind --tool=callgrind ./target/release/examples/bzcat 1 < corpora/wikidata.bz2 > /dev/null
callgrind_annotate --inclusive=yes callgrind.out.*
```
