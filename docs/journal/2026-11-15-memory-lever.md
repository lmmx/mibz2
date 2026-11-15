# 2026-11-15 · Pathfinder · The memory lever: two opt-in Cells kernels for an i9 experiment

Assignment: QUEUED 6901989466 on #145, approved by Louis. Decision rule: LOG 6902022809, posted
before any data. This entry holds what was built, the safety arguments (RULES B5), the
evidence, and the record of #444, whose code is one of the two arms. **No timing was taken
here;** the run is `bench-memory-lever` on the i9.

## The question

At 10 threads on the i9, one block a thread takes 1.040× the teams' wall (LOG 6900029992).
Is memory per core what holds it back? CI's runners have 2 physical cores and can't see
ten workers sharing a cache. Two cuts of Cells' memory failed as shipping changes because
they cost instructions at 1 to 4 threads. They are the experiment's two arms.

| arm | kernel | memory, per letter | instructions, per letter |
|---|---|---|---|
| `mibz2-workers-w` | Cells as shipped | 35.6 B besides the output | induction 115.5 |
| `mibz2-workers-w-lazy` | lazy letters: a slot only for a cell | 22.0 B | induction 181.0, first stage 67.5 against 76.3 |
| `mibz2-workers-w-nt` | rules staged and stored past the cache | 35.6 B, of which the rules' 9.1 B bypass the cache | induction 133.9 |

Counted in this branch's build with `examples/lever_counts.rs` under callgrind, on Wikidata's
196 admitted blocks (11,090,867 letters). All three kernels admit the same blocks and make
the same 20,456,299 cells. The lazy pool peaks at 0.318 of the letters (0.295 to 0.337).

## What was built

Everything is behind the cargo feature `memory-lever`, which is off by default.

| piece | where |
|---|---|
| the two kernels, as `bwt::LazyCells` and `bwt::StagedCells` | `src/cells_lever.rs`, included in `src/cells.rs` only with the feature |
| the harness arms `mibz2-workers-w-lazy` and `mibz2-workers-w-nt` | `examples/ab.rs` |
| the recipe | `bench-memory-lever`, `.just/bench.just` |
| the sign reading of the rule's case 3 | `bench/memory_lever.py` |
| the store fence read in the compiled library | `bench/memory_lever_fence.sh` |

**Without the feature the shipped decoder is unchanged.**
- The kernel's three functions count the same instructions as master's, to the instruction
  (induction 1,280,552,604; callgrind, Wikidata, one thread).
- The library holds no `sfence` and none of the experiment's functions
  (`bench/memory_lever_fence.sh`, run in CI).

**With the feature the shipped kernel is still the shipped kernel.** `induce_cells` is not
edited: the staged kernel is a copy of it, `induce_cells_staged`. In the feature build its
induction counts the same 1,280,552,604 instructions. So `mibz2-workers-w` in the bench's
binary runs the induction that ships, and an arm differs from it in the kernel alone.

The design of each kernel is in `2026-11-15-lazy-letters.md` (lazy letters, the lean build
of its addendum) and in the staged kernel's section below.

## Safety argument: lazy letters (RULES B5)

`slots_lazy` and `induce_lazy`, called one after the other by `try_invert` on the same
output buffer, with nothing between them.

1. **The first stage's writes.** It writes a 4-byte entry and a byte for each letter into
   the output buffer, which it has sized for them. It makes the letters its first pass
   counted, from the same records, so each entry's index and each byte's place is below the
   letters' count.
2. **The streams.** The top front reads the bytes from the last one back and checks its
   count before each read. The bottom front does the same with the entries. An entry found
   through a byte (`nt`) has an index below the letters' count: a byte's entries are as
   many as its bytes among the top order's, and each byte is read once.
3. **Every slot offset is one a pool handed out.** A pool hands out its next unused slot,
   after checking its end, or a slot given back. Offsets are then only copied: into links,
   into a letter's other slot, into an entry with `TAG`. A link of 0 means the stream and is
   never followed as a slot. A letter just read from its entry carries the entry's index
   with a marker in place of a slot; it takes its slots before the step uses them.
4. **Which buffer an entry's slot is in.** A front writes a letter's entry once, when it
   reaches the letter first, and never reads a place of its stream twice. So a tagged entry
   the top front reads was written by the bottom front and holds a top slot, and the
   reverse.
5. **A slot given back is dead.** Both front slots at an equal step, and both slots of a
   closed letter. That is the argument of #408 for the promised slot: an ended letter's
   front slot is the last of its order, nothing links to it, and its other slot has just
   taken the cell. An entry may still hold a given-back slot, but only one whose place its
   front has already passed.
6. **The pools are never exhausted.** The pairs in use are one for each letter a front has
   reached that is still alive, so a slot a letter suffices. The bound is checked all the
   same where a new pair is taken.
7. **Sizes.** A block of 2^22 rows or more is refused before anything is laid out, so a
   length fits an entry's 22 bits. The layout is refused at 2^31, so the tag bit is free.
8. **The words** are made as `induce_cells` makes them, at its offsets.

## Safety argument: rules staged and stored past the cache

The induction stages each rule in a 1 KB ring on its stack, at the rule's offset modulo
1 KB, and sends a full ring to the rules' region: whole 64-byte lines with
`_mm_stream_si128`, the bytes before the first line and after the last with plain copies.
The rest is sent when the induction ends. The fetch-ahead hints a word only if it is kept
as bytes, so it does not read the rules' lines back into the cache. Outside x86-64 each
rule is written to its place as before.

1. **A staged rule lies inside the ring.** The region's start is a multiple of 16 (`slots`
   rounds it for this kernel, and the induction refuses a layout where it is not), a rule's
   offset advances by 16, and the ring's size is a multiple of 16.
2. **`send_rules` gets ranges inside the rules' region,** within one multiple of the ring's
   size, holding only rules staged since the last full send.
3. **The stores past the cache are aligned.** Plain copies run up to the first 64-byte
   boundary; a debug assertion checks each line.
4. **Nothing reads a rule before it is sent.** The induction reads a word's bytes only for
   a word kept as bytes.
5. **Ordering (corrected after Guardian D108).** `_mm_stream_si128` asks for an `sfence`
   before any other access to the memory it wrote, by the same thread too, and before code
   that assumes ordinary store ordering. My earlier argument, that a thread reads its own
   such stores correctly, was not what the intrinsic's contract says. What holds:
   - a `Fence` object is made before the first send, and its drop runs `sfence`;
   - it is dropped on every exit of the induction: a word returned, a refusal, an unwinding
     panic;
   - nothing touches the rules' region between a send and that drop. The induction never
     reads a rule, later sends write other bytes, and the writer runs after the induction
     has returned.

**How the fence is checked (D108's closing requirements).**
- **Coverage of every exit, by a model.** In tests `send_rules` sets a mark wherever it
  sends whole lines, and the fence clears it where it runs `sfence`. The writer asserts the
  mark is clear before it reads a rule. `miri_cells_staged_fence_covers_every_exit` takes
  each exit after a send of whole lines (a word, a refusal on the budget, an unwinding
  panic) and checks that the mark is clear afterwards, that exactly one fence ran, and that
  the fence found the mark set.
- **The instruction itself, in the compiled library.** `bench/memory_lever_fence.sh` reads
  the disassembly: the staged induction holds an `sfence` or calls the `Fence` drop that
  does. It runs in CI.
- **Baseline and targeted failures,** in the mutation table below: the unmutated tree
  passes both; removing the fence object or dropping it early fails the model; removing the
  `sfence` from the drop fails the disassembly check.

## Evidence

**Tests:** nine new ones with the feature; the shipped ones pass unchanged with it and
without it.
- both kernels against the original induction, state by state, on every short column
  (`cells_lever_exhaustive_state`), and on the boundary columns with their buffers reused
  (`miri_cells_lever_boundaries_reuse`). The rules' region is compared byte for byte once
  each induction is over;
- both through `try_invert` against the plain walk, on every binary column to length 10 at
  every row;
- the lazy pool: slots inside their pools, never half the letters in slots on long
  columns, a few pairs at most on a column of a hundred closed cycles, and a letter longer
  than 2^21 rows read whole;
- the public route (`LeverCells`) against Segmented, admitted and refused blocks.

**Miri:** the four `miri_cells_lever_*`, `miri_cells_staged_*` and `miri_cells_lazy_*`
tests pass, with canaries after every region.

**Fuzz:** `examples/fuzz.rs`'s Cells mode, built with the feature, holds four more kernels
(both arms, under Cells' admission and under the forced route) to Segmented's partial
output and exact error, and to Cells' and ForcedCells' route counts. 60 seconds, seed 42:
22,913 cases, 12,073 blocks through the forced route, 196.5 M cells. CI runs it for 30.

**Harness:** `ab run` with the four arms on Wikidata at 1, 4 and 12 threads: one output
hash. The recipe's body was run end to end on the local corpus with one round, to check
its shell, the two summaries, the sign reading and the bundle. Its readings were not
looked at as timing.

**Mutation pass** (`2026-11-15-memory-lever/mutants.tsv`, script `mutate.py.txt`; debug
profile with the feature): 38 mutants, 35 killed. Of the four on the fence, three are
killed by the model's test and the fourth, the fence object running no `sfence`, by the
disassembly check. The survivors:

| survivor | why |
|---|---|
| staged: the induction does not check the rules' offset | a second line of defence: `slots` always rounds the offset for this kernel |
| lazy: the pools' bound not checked | a second line of defence: the pools are never exhausted (argument 6) |
| lazy: the top stream's end not checked | a second line of defence: the walk's row's letter is never dropped, so the induction returns before a stream runs out |

A first pass left three more alive, and each got a test: the model's mark never set (the
fence test now checks the fence found it set), no pair given back for a closed letter
(`miri_cells_lazy_closed_letters_give_their_slots_back`), and a letter's length read one
bit short (`cells_lazy_long_letter`).

One mistake on the way, kept here so it isn't repeated: the fence test first panicked after
the induction's first send and expected the mark to be set. A first send can be shorter
than a line, and whether it holds a whole one depends on where the allocator put the
buffer, so the test passed or failed with the build. It now panics after the first send
the model marked, and a full stage's send is marked wherever the buffer lies.

## #444, recorded

#444 offered the staged rules as a shipping change, under LOG 6901430556. Its paired bench
at 1 to 4 threads, and the one re-run the policy allows:

| `wikidata`, wall | first run (EPYC 9V74) | re-run (Xeon 6975P-C) |
|---|--:|--:|
| 1 thread | +3.2% (+0.1%, +6.4%) slower | +5.6% (+5.1%, +6.2%) slower |
| 2 threads | +7.5% (+3.0%, +12.1%) slower | +1.9% (−0.1%, +4.0%) |
| 4 threads | +4.1% (+3.0%, +5.1%) slower | +4.5% (+3.6%, +5.4%) slower |

`text` was level in every cell I saw: all three of the re-run, and 1 and 2 threads of the
first run. (The first run's 4-thread `text` row was cut off in the notification I read, and
the re-run replaced the bot's comment.) Every arm gave the same bytes. By rule 3 of that
LOG the change is closed as a shipping candidate. Its code is the `-nt` arm here.

## How to run

```
git fetch origin && git checkout <the commit the ready tag names>
just submit bench-memory-lever
```

The rule's readings are in the bundle: `summary.txt` (against `mibz2`, the sanity case),
`summary-workers.txt` (each memory arm against `mibz2-workers-w`) and `lever.txt` (the
sign reading of case 3).

## Reproduce the counts and checks

```
cargo test --features memory-lever --lib cells::
cargo +nightly miri test --features memory-lever --lib -- miri_cells_lever miri_cells_staged miri_cells_lazy
bash bench/memory_lever_fence.sh
python3 docs/journal/2026-11-15-memory-lever/mutate.py.txt .
cargo build --release --features memory-lever --example lever_counts
valgrind --tool=callgrind ./target/release/examples/lever_counts corpora/wikidata.bz2 lazy   # or cells, nt
```
