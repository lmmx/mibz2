# 2026-11-15 · Pathfinder · Cells' bottom slots and rules in the output's spare capacity

Assignment: QUEUED 6902723569 on #145 (Louis: memory bandwidth is the lever; build cuts now).
Prices and decision rule: LOG 6902797865, posted before any count of the change. This entry
holds the design, the safety argument (RULES B5), the counts and the mutation pass. The
timing is CI's paired bench on the PR; none was taken here.

## What changes, and why

The induction writes one 16-byte rule for each cell whose word is longer than 16 bytes, and
only the writer reads them. On Wikidata they are 9.1 bytes a letter, a quarter of the
kernel's buffer. #433 moved the bottom slots out of that buffer into memory that was dead
anyway. This does the same for the rules.

**Where the dead memory is.** Not in `ll`: on the Cells route a block has no last column,
`ll` is the buffer Cells writes its output into, and the writer fills it while it still
reads the rules. The memory that is dead is the **spare capacity of the block's final
output**: `rle::unrle` appends the block there, after the kernel has returned.

**The layout now:**

| region | where | per letter, Wikidata |
|---|---|--:|
| top slots | the kernel's buffer | 20 B |
| arena (words kept as bytes) | the kernel's buffer | 6.4 B |
| bottom slots | the output's spare capacity (the "tail") | 8 B |
| rules | the tail, after the bottom slots | 9.1 B |

- **The kernel's buffer falls from 35.6 to 26.4 B a letter.**
- **The tail's 17.1 B a letter is 1.03 of a block's rows** at the median, so the slots and
  rules land almost exactly on the bytes `unrle` writes next.
- **`ll` is not touched until the writer.** A refused block leaves it byte for byte as it
  was. Before, a refusal on the budget left the bottom slots in it.

**No value is added to the induction's loop.** The loop already held one pointer for the
bottom slots' buffer; a rule is now written through it. That matters: every other cut tried
this week added state to the loop and paid for it in instructions
(`2026-11-15-memory-lever.md`).

**The offsets.** A word's offset says which kind it is: the writer treats an offset at or
above the rules' as a rule. For that to hold with the rules in another buffer, the tail's
offsets are counted from `shift` bytes before the tail, where `shift` is the kernel
buffer's size. A rule's offset is then above every arena offset, and the writer is
unchanged but for the pointer it reads rules through.

**The plumbing.** `InverseBwt` gains `invert_runs_tail`, which also takes the tail; its
default ignores it. `decode_block` passes the final output. A caller with no tail to lend
(`invert_runs`, the forced route, the tests) gets a buffer of the kernel's own.

**The cells are counted down,** not up: one value in the loop where there were two. It
takes back part of what the changed pointer use costs (below).

## Safety argument (RULES B5)

The new unsafe is the tail: a pointer into a `Vec`'s spare capacity.

1. **Room.** `slots` reserves the layout's bytes of spare capacity before anything is
   written (`tail.reserve`). `induce_cells` checks the capacity again and refuses a tail
   without it. Every access is at the tail's length plus an offset inside that room.
2. **The shifted pointer.** The tail is addressed from `shift` bytes before its start. That
   pointer is made with wrapping arithmetic and is never read or written as it is: every
   use adds an offset of at least `shift`, which brings it back inside the spare capacity.
   The pointer is derived from the tail's own, so it may access the tail's allocation.
3. **Only what was written is read.** Spare capacity holds nothing. What is read there:
   - the letters' bottom slots, all written by `slots`;
   - the 128 bytes before them, which `slots` zeroes, because the induction's fetch-ahead
     reads a slot's place up to 12 slots before the first;
   - rules below `rule_end`, each written by the induction before the writer reads it;
   - in debug and Miri, the canaries `slots` wrote.
4. **The caller's bytes.** The tail's length is never changed and nothing is written below
   it. `reserve` may move the buffer; the bytes below its length move with it.
5. **Between the stages** the tail is not reserved again, so its buffer does not move, and
   each stage takes its pointer afresh from the `Vec`.
6. **No overlap with the output.** The writer writes `ll` while it reads rules from the
   tail. They are two `&mut Vec`s, so two allocations.
7. **When the rules die.** `decode_block` appends to the tail (`finish`) only after the
   kernel has returned. The kernel keeps nothing in the tail between blocks.
8. **Offsets fit.** A layout whose offsets in either buffer would reach `u32::MAX` is
   refused.

## Evidence

**Tests** (114 in the library pass):
- the comparison against the original induction, state by state, unchanged (in its layout
  the rules lie in the kernel's buffer), and now also the rules' region byte for byte;
- `miri_cells_tail_is_the_callers`: the caller's tail keeps its length and bytes, its spare
  capacity is used, the kernel's own tail stays unallocated, a refusal leaves `ll` and the
  tail as they were, and without a tail the kernel uses its own;
- `miri_sequential_default_cells_route`: through the public reader, `decode_block` lends
  the output and the kernel's own tail is never allocated;
- the layout tests, updated; three of them now assert that a refusal leaves `ll` untouched.

**Miri** (run again after the merge of #447): the ten `cells::tests::miri_cells_*` tests and
`miri_sequential_default_cells_route` pass. The tail test starts from a `Vec` with capacity
and no bytes, so a read of anything not written would be reported. With the feature, the
four Miri-listed tests of the experiment's kernels pass too.

**Fuzz:** the Cells mode, 60 seconds, seed 42: 49,661 cases, 26,200 blocks through the
forced route, 425.8 M cells.

**Corpus:** every bench corpus and Wikidata decode to the same bytes at 1, 4 and 12 threads,
and Wikidata's output equals master's.

**Mutation pass** (`2026-11-15-rules-in-output-spare/mutants.tsv`, script `mutate.py.txt`;
debug profile; run again after the merge of #447): 16 mutants, 16 killed. The first row is
the unmutated tree passing. Fifteen
are killed by the tests. The sixteenth, the bytes before the bottom slots not zeroed, reads
memory that was never written: the native tests pass, and Miri reports the read as
undefined behaviour.

**Instructions** (callgrind, Wikidata, one thread, inclusive, per letter;
`2026-11-15-rules-in-output-spare/counts.txt`):

| | master | this change |
|---|--:|--:|
| first stage (`slots`) | 81.0 | 74.2 (−8%) |
| induction | 115.5 | 118.7 (+2.8%) |
| writer | 54.0 | 60.2 (+11.5%) |
| the three | 250.4 | 253.2 (+1.1%) |
| whole decoder, whole file | 5.036 G | 5.067 G (+0.6%) |

- **Rule 2 of LOG 6902797865** allowed 2% a stage. The induction and the writer are over it,
  and I reported that on #145 before opening the PR.
- **The writer** now holds two pointers, one for the arena and one for the rules, where it
  held one. That is its 6 instructions a letter.
- **The induction** uses its second pointer in one more place. Counting cells down took it
  from +4.4% to +2.8%.
- **The first stage** no longer sizes or zero-fills anything in `ll`.

## The memory-lever experiment's two kernels, on the new layout

#447 merged while this was being built, so its two opt-in kernels (`src/cells_lever.rs`) are
moved onto the new layout here. Neither is a default; both stay behind `memory-lever`.

- **The staged kernel** is regenerated from the new `induce_cells`, with the same three
  differences as before (the stage, the fence, the guarded fetch-ahead). Its bottom slots and
  rules lie in the tail too, and it sends its staged rules there.
- **The lazy kernel** keeps its entries, its bytes and its bottom slots' pool in the tail's
  spare capacity, where it kept them in `ll`. Its rules stay in the kernel's buffer.
- **Their tests** pass unchanged but for one layout assertion: both against the original
  induction state by state, the fence model, the pool's tests. The fence check passes, and
  the feature's fuzz ran 24,387 cases in 60 seconds.
- **The four bench arms** give one output hash on Wikidata at 1, 4 and 12 threads.
- **Their mutation table** was run again on this code
  (`2026-11-15-memory-lever/mutants.tsv`, regenerated here; the script's texts were updated
  to the regenerated staged induction): 38 mutants, 35 killed, the same three survivors as
  on master, for the reasons `2026-11-15-memory-lever.md` gives.

**The arms' counts on this layout** (`examples/lever_counts.rs` under callgrind, per letter):

| arm | first stage | induction | writer | whole program |
|---|--:|--:|--:|--:|
| `mibz2-workers-w` (Cells as shipped, this layout) | 71.2 | 118.7 | 60.1 | 4.807 G |
| `-lazy` | 56.0 | 176.3 | 60.1 | 5.276 G |
| `-nt` | 71.2 | 137.2 | 60.1 | 5.012 G |

On master's layout the same harness counted 4.758 G, 5.340 G and 4.963 G.

**What it does to the experiment.** If this merges before the i9 run, every arm's baseline
has the rules in the output's spare capacity. The `-nt` arm then stores its rules past the
cache into memory that `unrle` is about to write, which is a weaker version of its
question: the bypass avoids reading the lines in, and `unrle` still has to. The `-lazy` arm
is unaffected in what it asks.

## Reproduce

```
cargo test --lib cells
cargo +nightly miri test --lib -- miri_cells
python3 docs/journal/2026-11-15-rules-in-output-spare/mutate.py.txt .    # leaves the tree as it was
cargo build --release --example bzcat
valgrind --tool=callgrind ./target/release/examples/bzcat 1 < corpora/wikidata.bz2 > /dev/null
```
