# 2026-11-16 · Pathfinder · Lazy letters' instruction overhead, attributed; a direct path for the commonest step counted worse, and the task stops at its rule

Assignment: QUEUED 6904651994 on #145: cut the lazy induction's overhead (115.5 → 181
instructions a letter) without adding bytes. **The rule in force,** set before any data
(6904651994, tightened in LOG 6904670011 and accepted by the Coordinator): a
`mibz2-workers-lazy2` arm is built if the lazy induction has at least 10% fewer
instructions a letter than master's 181.0, **and** fewer conditional branches plus memory
reads and writes a letter, at equal or fewer bytes, with identical output.
**No timing was taken.**

## Method

Callgrind on master `2044ac2b`, Wikidata's 196 admitted blocks (11,090,867 letters), one
thread, with per-instruction counts and its branch and cache simulators on
(`--dump-instr=yes --branch-sim=yes --cache-sim=yes`). `attribute.py.txt`, here, gives each
instruction to the source line it was inlined into (`addr2line -i`: the outermost frame in
the kernel's file), and the lines to the loop's parts (`ranges_*.json`). The parts sum to
the function's totals exactly. The outputs are `parts_cells.txt` and `parts_lazy.txt`.

## The two inductions, per letter

| | eager | lazy | extra |
|---|--:|--:|--:|
| instructions | 115.5 | 181.0 | +65.5 |
| conditional branches | 7.5 | 19.1 | +11.6 |
| mispredicted (simulated) | 0.42 | 0.88 | +0.46 |
| memory reads | 25.5 | 34.5 | +9.0 |
| memory writes | 21.3 | 32.9 | +11.6 |
| first-level cache misses, reads and writes | 3.4 | 3.4 | 0 |

The eager loop is four branches a cell. The lazy loop is more than ten.

## Where the +65.5 go

| source | eager | lazy | extra |
|---|--:|--:|--:|
| reaching a letter: its slot, or its place in a stream (byte, counter, entry, tag test, the word made from the entry) | 18.7 | 56.3 | +37.6 |
| the eager loop's fetch-ahead hints (the lazy loop has none) | 16.7 | – | −16.7 |
| settling a letter read from its entry: its slots linked, its entry tagged | – | 16.4 | +16.4 |
| taking and giving back pairs of slots | – | 9.3 | +9.3 |
| moves and spills: instructions with no source line, the compiler's own | 9.9 | 29.4 | +19.4 |
| the loop's head | – | 2.0 | +2.0 |
| the cell itself: budget, word, bytes or rule | 49.2 | 46.9 | −2.3 |
| the step's three ends, less the reads counted above | 21.0 | 20.8 | −0.2 |
| **all** | **115.5** | **181.0** | **+65.5** |

- **Most of the extra mispredictions are in the reads** (0.46 of 0.88 a letter): whether
  the other front reached a letter first is a coin the predictor cannot call.
- **Every letter takes one pair of slots and gives one back.**
- **The spills are register pressure, measured:** 19.4 instructions and about 10 memory
  accesses a letter more than the eager loop's.

## What the loop does most

Counted with a throwaway counter at the loop's head, where both letters are whole, per
letter:

| the top letter / the bottom letter | a letter | followed at once by an equal step |
|---|--:|--:|
| both already cells (read from slots) | 0.430 | 0.367 |
| top from a slot, bottom fresh from its entry | 0.167 | 0.143 |
| top fresh from its entry, bottom from a slot | 0.140 | 0.114 |
| both fresh from their entries | 0.262 | 0.217 |

- **53% of all cells are equal steps:** both letters end together.
- **A fresh letter that ends in an equal step never uses its front slot.** It takes a
  pair, one slot of which is only there to be promised if the other letter ends first, and
  gives that slot straight back: 0.69 times a letter.

## The candidate: a direct path for an equal step between two fresh letters

`direct-path.diff.txt` (SHA-256 `b5727e9f…2238cb5ce9`, against master `2044ac2b`). After
the loop's two reads, if both letters came from their entries, are of one length and
neither is the walk's row, the cell is made directly: one pair of slots taken, the
two-byte word copied, the cell's two slots and the two entries written. The general path
takes two pairs there, settles each letter, and gives one pair back.

**It is correct as far as it was taken:** the 20 kernel tests pass, the state-by-state
comparison against the original induction among them; `tests/lazy_workers.rs` and
`tests/cells.rs` pass; Wikidata decodes to the same bytes; the pool's peak is unchanged.
No Miri, mutation or review: it did not get that far.

**The count** (`parts_lazy2_direct_path.txt`), per letter:

| | master's lazy | with the direct path | |
|---|--:|--:|--:|
| instructions | 181.0 | 191.1 | +5.5% |
| conditional branches | 19.1 | 19.5 | +0.4 |
| memory reads and writes | 67.4 | 84.8 | +17.4 |
| mispredicted (simulated) | 0.88 | 1.09 | +0.21 |

| part | master's lazy | with the direct path |
|---|--:|--:|
| settling, and taking and giving pairs | 25.7 | 20.1 |
| the cell and its equal end | 59.4 | 49.3 |
| the direct path, and its test at the loop's head | – | 14.6 |
| the reads | 56.3 | 57.0 |
| **moves and spills** | **29.4** | **38.6** |

**The rule's verdict: it fails both clauses.** The path does what it was priced to do in
the lines it replaces: 15.7 instructions a letter less in settling, pairs, the cell and
its end, for 14.6 of its own. But the compiler's moves and spills grew by 9.3 instructions
and 12.8 memory accesses a letter, and the new test mispredicts. The loop holds more
values than the machine has registers, and another path through it means more of them are
saved and restored around every path.

## What this settles

- **No `-lazy2` arm.** The task stops here, as its rule says.
- **I priced the candidate at −12 instructions a letter before building it** (REVIEW
  6904790860), then at −5 on a closer look, and it counted +10. Pricing this loop by its
  source lines does not work: the lines' own cost moved as priced, and the spills moved
  more.
- **What the overhead is.** Of the +65.5: about 21 are the streams' indirection net of the
  eager loop's hints, 16 the settle, 9 the pairs, 19 the spills. The first three are what
  lazy letters are: a letter touched once in each stream and made into slots once. The
  fourth grows when anything is added.
- **What might take instructions back** is fewer live values in the loop, not fewer
  lines: the loop carries about twenty. I do not have a design for that; the one attempt
  (a pair counter for the pools' four values, on the branch `perf/lazy-workers-on-449`)
  counted worse too.
- **Outside this task's rule:** the lazy loop has no fetch-ahead, where the eager loop
  spends 16.7 instructions a letter on it by choice. Its word copies and its slot reads
  through tagged entries are the candidates. That adds instructions and is a clock
  question.

## Reproduce

```
git checkout 2044ac2b
cargo build --release --example lever_counts --features memory-lever
valgrind --tool=callgrind --dump-instr=yes --branch-sim=yes --cache-sim=yes \
  --callgrind-out-file=cg.lazy ./target/release/examples/lever_counts corpora/wikidata.bz2 lazy
python3 docs/journal/2026-11-16-lazy-overhead/attribute.py.txt cg.lazy \
  target/release/examples/lever_counts "<mibz2::cells::Kernel>::induce_lazy" cells_lazy.rs \
  docs/journal/2026-11-16-lazy-overhead/ranges_lazy.json 11090867
# the eager loop: `cells`, induce_cells, cells.rs, ranges_cells.json
# the candidate: git apply direct-path.diff.txt, then as above with ranges_lazy2.json
```
