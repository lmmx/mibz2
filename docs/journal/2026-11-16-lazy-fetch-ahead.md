# 2026-11-16 · Pathfinder · A fetch-ahead for the lazy induction: level on the i9, deleted

Direction: the Coordinator's messages of 16 November, relaying Louis (logged on #145): give
the lazy kernel a fetch-ahead, as a bench arm, the default unchanged until the i9 had read
it. The rule was LOG 6905226406 on #145, posted before the data. The arm was PR #459, head
`2e1aa39c058ac45268157a66ec3ba723bb6c0d7f` (branch `perf/lazy-fetch-ahead`, kept); the
patch against master `2caf086b` is `2026-11-16-lazy-fetch-ahead/fetch-ahead.diff.txt`
(SHA-256 `d13db465…2c9f354a`). **The i9 found it level, and under its rule the arm is
deleted.** This entry is the record: the result first, then the design, the safety
argument and the evidence as they stood.

## The result

`just bench-lazy-hint` on the i9-10900K at that head, ten paired rounds, Wikidata (the
Coordinator's LOG 6905559909 has the bundle and the intervals):
`mibz2-workers-lazy-hint` over `mibz2-workers-lazy`, wall time.

| threads | 4 | 8 | 9 | 10 |
|---|--:|--:|--:|--:|
| hint ÷ plain | 1.006 | 1.006 | 1.003 | 0.989 |

Every interval spans 1. The rule asked for an interval wholly below 1 at 8, at 9 and at
10 threads. None is, so the arm is deleted and #459 closed without merging.

**What it says.**
- **The hints cost about 24 instructions a letter and bought nothing measurable,** at any
  count. They did not cost measurably either: the instructions are cheap ones.
- **The slots a front reads through a tagged entry are not where the lazy kernel waits on
  memory,** or the hints do not reach them in time. The run does not tell these apart.
  My guess is the first: the pools are about half a megabyte a block (a third of the
  letters at most, 28 bytes a pair) and are the same memory block after block, so a
  worker's own caches keep most of them. That is a guess; nothing here measured it.
- **So the memory pressure the dose shows (CPU at 10 threads over 1: 1.92 for lazy
  workers) is not in these reads.** What is left is what every worker streams through:
  the words, the streams of entries, and the output.

**What I expected, and said before the data** (6905226406): a small loss at 4 threads and
no expectation at 8 to 10. Level at all four is inside that.

# The arm as it was proposed

## Why

The eager induction spends 16.7 instructions a letter on hints (`ahead!`), because its
misses are slots and words it can name twelve steps before it reads them. The lazy
induction (0.5.0) has none. Its attribution (`2026-11-16-lazy-overhead.md`, on master
`2044ac2b`) puts its read misses, in callgrind's 48 KB first-level model, per letter:

| read | misses a letter |
|---|--:|
| the top front: a top slot, through an entry the bottom front tagged | 0.50 |
| the bottom front: a bottom slot through a tagged entry, then the top slot it names | 0.95 |
| the cell: a word's bytes | 0.40 |
| everything else | 0.59 |

A front's first visit to a letter costs no miss: the letter is its entry, read in stream
order. Its misses are the letters the *other* front reached first, whose cells lie in
slots anywhere in the pools.

## What changes

**Each front looks `FAR_PLACES` (12) places on in its stream.** If that place's entry
names a slot as memory has it then, the slot is hinted.

- **The bottom front** reads its stream's entries in order, so the place 12 on is 12
  entries on. It hints that entry's bottom slot, and, 6 places on (`NEAR_PLACES`), the top
  slot that bottom slot names, which holds the letter. Two hints, because the read is two
  dependent loads.
- **The top front** finds a place's entry through its byte's counter (`nt`). So the
  look-ahead has counters of its own (`nt_far`), moved 12 places on at the start and one
  place a place after that. It hints that entry's top slot.
- **An entry not yet tagged hints nothing useful:** its hint names the buffer's first
  bytes (`slot_or_0`). If the other front tags it later than 12 places before, the read
  misses as before.
- **The words' bytes are not hinted.** A word is known only when its letter is read.

**It is a bench arm, not the default:** `bwt::HintLazyCells`, the arm
`mibz2-workers-lazy-hint`. The plan's kernel (`LazyCells`, `mibz2-workers-lazy`) has no
fetch-ahead, as before. The induction takes the fetch-ahead as a compile-time parameter,
so each copy has no test of it in its loop. `just bench-lazy-hint` runs the two arms at 4,
8, 9 and 10 threads. No cargo feature and no user-facing switch.

**What does not change:** the cells, their order, every word and every offset; the pools;
the bytes a letter (the look-ahead's counters are 1 KB on the stack); below 8 threads,
where the plan keeps the eager kernel.

**One write is added to the arm's first stage:** the output buffer's first `BPAD` (128)
bytes are set to 0. They lie before the entries and were unused. The plan's kernel does
not write them.

## Safety argument (RULES B5)

The new unsafe operations are reads, and hints.

1. **A hint reads nothing.** `prefetch` takes any address; each is formed with
   `wrapping_add`, so no out-of-bounds pointer is made by `add`.
2. **The bottom look-ahead's entry reads are inside the buffer.** They are at
   `BPAD + 4 * (kb - FAR_PLACES)` and `BPAD + 4 * (kb - NEAR_PLACES)` with `kb >= 0`, so at
   or after `BPAD - 4 * FAR_PLACES`, which is at least 0: a constant assertion holds
   `4 * FAR_PLACES <= BPAD`. They are below the entry just read.
3. **What those reads hold is an entry, or 0.** From `BPAD` on they are the stream's
   entries. Before it they are the bytes `slots_lazy` set to 0.
4. **The slot read through one is inside the buffer.** `slot_or_0` gives a tagged entry's
   offset, and 0 for anything untagged, 0 included. An entry the bottom front has not
   reached can only have been tagged by the top front, which writes a bottom slot's
   offset there (`induce_lazy`'s own argument). So `near` is a bottom slot's offset or 0,
   and `near + 4` has four bytes after it inside the buffer either way.
5. **What that slot names is only hinted.** A bottom slot's second field may not be
   written yet when it is read here; the buffer is a `Vec<u8>` grown with `resize`, so
   the bytes are initialised, and the value goes only into a hint, kept to the top slots'
   pool.
6. **The top look-ahead's reads are the top front's own reads, 12 places early.** It runs
   only when `ti >= FAR_PLACES`, so the byte read is a place of the stream. `nt_far` is
   `nt` moved on by the last 12 places at the start, and by one place for each place the
   front takes, so it names for place `ti - 12` the entry `nt` will name when the front
   gets there: below `rr`, by the same argument as `nt`'s.
7. **Under Miri** every one of these loads runs; only the hint itself is compiled out.

## Evidence

**Tests** (21 kernel tests; 22 with `memory-lever`):
- `miri_cells_lazy_fetch_ahead_stays_inside` (new, Miri-listed): the output buffer comes
  in full of `0xFF`, which would read as tagged entries naming far-off slots, on columns
  of fewer letters than the look-ahead and of more. The output is the oracle's, the first
  stage leaves the first `BPAD` bytes 0, and the plan's kernel makes the same cells and
  hints nothing.
- **Every hint is checked** (test builds only): the induction records the slot hinted for
  each place, and asserts, when a front reads that place, that its entry names that slot.
  A tagged entry is never written again, so a hinted slot must be the one read. The test
  requires some to be met on a long enough column (117 on its 65-letter one).
- The lazy kernel's existing tests run both kernels: against the original induction state
  by state, their boundary columns, their route.
- `tests/common/cells.rs` holds both to `Cells`' output, errors and routes, so the Cells
  fuzz does too: 23,189 cases in 60 seconds, seed 42.

**Corpus:** Wikidata decodes to the same bytes at 1, 4, 8 and 12 threads, and the three
workers arms (`-eager`, `-lazy`, `-lazy-hint`) give one hash.

**Miri:** the 13 `cells::tests::miri_cells_*` tests pass, the new one among them, at the
commits before the default and the arm were exchanged. CI's Miri job reads the final head.

**Mutation** (`2026-11-16-lazy-fetch-ahead/mutants.tsv`, script `mutate.py.txt`; debug
profile): 9 mutants. Seven are killed by the tests: the bytes before the entries not set,
either of the look-ahead's counter updates dropped, the top look-ahead run before the
stream's first place, an untagged entry read as a slot, a slot read with its tag, and the
two kernels' inductions exchanged. One does not build: a look-ahead longer than the
bytes before the entries fails the constant's own assertion. One survives:

| survivor | why |
|---|---|
| the hinted top slot not kept to the pool | it changes only where a hint points; nothing is read through it |

A first pass, before the hints were checked against the slots then read, left three more
alive: the two counter updates and the two inductions exchanged. The check kills all three.
(The table was made before the default and the arm were exchanged; the script beside it
has the texts as they are now.)

## What it costs, and what could not be measured

**Instructions** (callgrind, Wikidata, the induction, per letter):

| | instructions |
|---|--:|
| master's lazy induction (0.5.0) | 181.0 |
| this PR, the plan's kernel (`-lazy`) | 198.0 |
| this PR, the arm that fetches ahead (`-lazy-hint`) | 221.5 |

- **The hints cost about 24 instructions a letter** against their own sibling: 10.6 on the
  top side (a byte, a counter read and written, an entry, the hint) and 16.7 on the bottom
  (two entries, a slot, two hints), less what moved elsewhere.
- **The plan's kernel counts 198, not 181, for the same loop.** Its source is master's but
  for the parameter. That is the layout noise of `2026-11-16-lazy-overhead.md` again, and one
  more reason the count is not the judge.

**Whether it pays cannot be told here.**
- Callgrind does not model a hint: its misses are unchanged, on the same lines.
- This container's caches are too large for those misses to cost. An indicative reading
  (10 paired rounds, the forced lazy arm, this build over master): 1.05 [0.98, 1.13] at
  1 thread, 1.01 [0.91, 1.11] at 2, 1.03 [0.95, 1.11] at 4. Direction only.
- CI's bench never runs the lazy kernel: the plan takes it from 8 threads.

So the judge is `just bench-lazy-hint` on the i9: `mibz2-workers-lazy-hint` against
`mibz2-workers-lazy` at 8, 9 and 10 threads (4 as a control), one commit, ten paired
rounds, under the rule posted on #145 before the data.

## Reproduce

```
cargo test --release --lib cells::
cargo +nightly miri test --lib -- miri_cells
python3 docs/journal/2026-11-16-lazy-fetch-ahead/mutate.py.txt .     # leaves the tree as it was
cargo build --release --example lever_counts --features memory-lever
valgrind --tool=callgrind ./target/release/examples/lever_counts corpora/wikidata.bz2 lazy-hint   # or lazy
just bench-lazy-hint                                                                               # on the workstation
```
