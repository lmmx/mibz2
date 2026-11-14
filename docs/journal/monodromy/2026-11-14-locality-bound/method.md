# Cells induction: how far below the right end can another legal order take line misses?

Untimed counting research (C1a registration, fixed before data; quoted in the brief). No
timing figures anywhere. Nothing in /home/user/mibz2 was changed; master was exported with
`git archive origin/master` into `mibz2-master/` and used as a path dependency.

Labels: **P** proved here · **C** counted (exact for its input) · **Conj** conjecture.

## Verdict

**CLOSED.** G (5 MiB, n-weighted over all 196 admitted blocks) = **0.9999986** (K = 64 greedy
targeting 5 MiB; fixed-layout model). Under the second address model 0.9999990. The
registered rule: CLOSED if G >= 0.85.

This does not depend on the greedy being good. In the fixed-layout model every order touches
the same set of lines, so its compulsory misses are R's distinct lines, and no order can miss
less than that (**P**). At 5 MiB, R's misses are already exactly its compulsory misses on 195
of the 196 blocks. Pooled, the floor for any legal order is G >= 12,567,310 / 12,567,328 =
0.9999986 (**C**), and the greedy reaches it. At the other two capacities the floor for any
order is G >= 0.8557 at 1.25 MiB and G >= 0.9528 at 2 MiB. So even a perfect offline order
could not reach the PARKED band at 2 MiB or 5 MiB. At 1.25 MiB it would land just above the
CLOSED line (0.8557 >= 0.85).

No validity failure: every B is a linear extension of R's dependency DAG, and it keeps every
byte-level read/write hazard of R. Every step therefore reads the values it read in R and
writes the same bytes, so the rules, the arena and the slots end byte-identical: R's
grammar, derivation tree and writer input. The LRU code matches the hand-computed trace.

## Results (SLOT = 20, master's layout; per output byte, n-weighted: sum of misses / sum of n)

All 196 admitted blocks, n = 176,398,965. The trace is the induction only: no fetch-ahead
loads, no order stage, no writer, cold at each block.

| misses / output byte | 1.25 MiB | 5 MiB | 2 MiB |
|---|--:|--:|--:|
| R (right end, as shipped) | 0.08326 | 0.07124 | 0.07477 |
| compulsory (distinct lines; the same for every order, model 1) | 0.07124 | 0.07124 | 0.07124 |
| R's compulsory share | 0.8557 | **1.0000** (0.9999986) | 0.9528 |
| floor on G for any legal order (compulsory / R) | 0.8557 | 0.9999986 | 0.9528 |
| B = greedy K=64, aimed at 5 MiB (model 1) | 0.08261 | 0.07124 | 0.07430 |
| **G** (that B) | 0.9922 | **0.9999986** | 0.9937 |
| G, same B, model 2 (allocation in execution order) | 0.9917 | 0.9999990 | 0.9939 |
| G, greedy K=64 aimed at 1.25 MiB (model 1; model 2) | 0.9928; 0.9918 | 0.9999986 | 0.9968; 0.9967 |
| G, greedy K=64 aimed at 2 MiB (model 1; model 2) | 0.9886; 0.9887 | 0.9999986 | 0.9940; 0.9943 |
| G, best of the three K=64 greedies per block | 0.9872 | 0.9999986 | 0.9932 |

**The wider window (K = 4096, every 8th admitted block: 25 blocks, n = 22,499,897; C).** On
these blocks R has 0.08473 / 0.07220 / 0.07587 misses per output byte, and the floor for any
order is G >= 0.8521 / 1.0000 / 0.9517.

| G on the 25 blocks (model 1; model 2) | 1.25 MiB | 5 MiB | 2 MiB |
|---|--:|--:|--:|
| K=64, aimed at 1.25 MiB | 0.9952; 0.9938 | 1.0000 | 0.9970; 0.9968 |
| K=64, aimed at 2 MiB | 0.9891; 0.9893 | 1.0000 | 0.9940; 0.9942 |
| K=4096, aimed at 1.25 MiB | 1.1018; 1.0737 | 1.0000 | 1.0373; 1.0262 |
| K=4096, aimed at 2 MiB | 1.1066; 1.0775 | 1.0000 | 1.0242; 1.0167 |

The wide-window greedy is *worse* than R (G > 1). Choosing the step with the fewest misses
now delays every step that brings a cold line. It opens many chains and leaves them half
done, and their slots then come back from far away. One block (block 1) went the other way
at 2 MiB (58,230 against R's 59,084 misses), but pooled it loses. A wider window does not
approach the floor. That is a statement about this heuristic, not a bound.

Variants (R row; full tables in `out/summary-k64.txt`):
- With the fetch-ahead's loads included (`ahead!`, 27.8% of the kernel's loads): R 0.08384 /
  0.07125 / 0.07497.
- With the order stage's stores (`slots`) as a fixed prefix before the induction: R 0.11188 /
  0.07125 / 0.08977. G for the 5 MiB greedy B: 0.9909 / 1.0000 / 0.9900.

Footprint of the induction (distinct lines × 64 B): 3.24 / 3.79 / 5.29 MiB (min, median,
max over blocks). Only block 158 exceeds 5 MiB. It is the one block where R misses more than
compulsory at 5 MiB: 86,759 against 86,741. The greedy removes those 18.

### Reuse distance (LRU stack distance in distinct lines; share of line accesses, pooled)

| bucket | R | B (K=64, aimed at 5 MiB) |
|---|--:|--:|
| 0 (same line again) | 0.5221 | 0.5227 |
| [1, 2) | 0.0480 | 0.0497 |
| [2, 4) | 0.0406 | 0.0401 |
| [4, 8) | 0.1527 | 0.1307 |
| [8, 16) | 0.0648 | 0.0640 |
| [16, 32) | 0.0127 | 0.0232 |
| [32, 64) | 0.0102 | 0.0163 |
| [64, 128) | 0.0091 | 0.0121 |
| [128, 256) | 0.0080 | 0.0095 |
| [256, 512) | 0.0082 | 0.0087 |
| [512, 1K) | 0.0094 | 0.0099 |
| [1K, 2K) | 0.0119 | 0.0121 |
| [2K, 4K) | 0.0167 | 0.0164 |
| [4K, 8K) | 0.0213 | 0.0206 |
| [8K, 16K) | 0.0198 | 0.0199 |
| [16K, 32K) | 0.0081 | 0.0082 |
| [32K, 64K) | 0.00170 | 0.00148 |
| [64K, 128K) | 0.00001 | 0.00000 |
| cold | 0.0345 | 0.0345 |

R's long-distance tail is the two slot orders' re-touches. A slot is written when a cell is
cut and read when the other front reaches it. That gap is set by the two orders' geometry,
and the DAG fixes it: a cell must be written before it is read. The greedy only moves mass
inside the short buckets.

## 1. The set of orders (step 1)

**Engine (Section 2 of the Rauzy–Veech entry, steps at any coincidence).** The state is the
top and bottom orders of letters with lengths and words. A right-handed step at a
coincidence c takes A, the top letter ending at c, and B, the bottom letter ending at c
(A ≠ B). It cuts the last min(λA, λB) points at c and makes N → w_A w_B, as in §2's table
(top wins: B takes N and moves to just after A on the bottom; bottom wins: A takes N and
moves to just after B on top; equal: A deleted, B takes N and A's bottom place). Letters
are the kernel's: the maximal runs of L, with orig_ptr's run cut so that its row is a letter
of its own (`slots`), its word marked. A schedule ends with one letter, the marked one.

**DAG(R).** Run R (always the largest coincidence). Step j depends on the last earlier step
that touched A_j or B_j, with letter identity as in §2: the winner keeps its id; the loser
keeps its id with the new word; on an equal step A is deleted and B goes on.

**Claim tested.** {legal schedules whose derivation tree (nested binary tree over the
bytes, rotation included) equals R's} = {linear extensions of DAG(R)}, as sets of step
sequences. A step is labelled (A id, B id, occurrence of that pair).

**Counted (C)**, `src/orders.py`, output `out/orders-small-words.txt`. All primitive words
(every rotation, so every orig_ptr):

| words | primitive words | all legal schedules (memoised count) | schedules giving R's tree | linear extensions of DAG(R) | words where the sets differ |
|---|--:|--:|--:|--:|--:|
| binary, length 2..10 | 1,964 | 212,150,400 | 10,435 | 10,435 | **0** |
| ternary (≥ 2 letters used), length 2..7 | 3,222 | 3,666,028 | 8,207 | 8,207 | **0** |

The sets are equal on every word, in both directions. No legal schedule makes R's tree
except a linear extension (no "same tree, other steps"), and every linear extension is legal
and makes R's tree. Pruning used for the tree set (exact): a step's new word persists as a
subtree of the final root's derivation, so a step whose word is no subtree of R's tree
cannot end in R's tree. The total schedule count is a memoised count over all branches.

**The cells form's DAG is the same (C).** `cells_R` ports `induce_cells` at slot-field
granularity. On every word above it makes the same cells (length and word pair) as the §2
engine's R, in the same order, and its data DAG has the same transitive closure as DAG(R).
The data DAG is: the previous step when the winner's remainder goes on, plus the step that
last wrote the slot a letter is read from.

Status: the claim is **Conj** in general. It is counted exact on all words above. For the
real blocks it is not needed for validity: each B is checked directly at byte level (§4).

## 2. Address models

- **Model 1, fixed layout (primary).** Every step touches exactly the lines it touches in R:
  R's slot, arena and rule offsets. Only the order of the steps changes. This is optimistic
  for B. A real kernel stepping out of the sweep would have to keep each open chain's
  register state somewhere (remainder, words, the promised slot), and would lose the
  sweep's fetch-ahead. Neither cost is counted.
- **Model 2, allocation in execution order.** Fresh slots (both orders), arena words and
  rules are given addresses by bump cursors in B's execution order, as the kernel's cursors
  would. Original letters' slots stay where `slots` put them. Every access to an allocated
  object is remapped by (object, offset). For R this is R itself.

## 3. Traces (step 3) and the cross-check against memtrace

`tracer/src/main.rs` ports `slots` and `induce_cells` from origin/master on a byte buffer
laid out exactly as the kernel lays it out (PAD, tops, bots, arena, rules; cap from the
record count and the cell budget; SLOT 20 or 24). It records every load and store of the
induction with its offset and width:
- slot reads (4 + 4 + 8 + 4 B);
- the two slots written (λ, word, other);
- the promise's link moves (read bs/ts, write the fresh slot's link and the cell's);
- the copy16 pairs (16-byte loads and stores) for short words;
- the two 8-byte rule stores;
- the fetch-ahead's two loads, flagged.

Prefetch hints are not accesses. Stack, registers and the writer are excluded.

The accesses are attributed to steps. A step owns its own stores and the slot reads of any
letter it reads fresh (the kernel does a loser's next read at the end of the previous step;
it is moved to the step that uses it). A dropped letter's reads go to the next step. The
final closing read is an epilogue, fixed at the end of every order. Each step also gets
byte-exact "semantic" accesses (the words' own bytes, not copy16's slack) for the hazard
check.

Blocks come from decoding /home/user/mibz2/corpora/wikidata.bz2 with `decode_block` and a
capturing kernel that keeps the column and orig_ptr. The records are the column's runs cut
at 127, as `runs::front` emits them. The 196 blocks are those with `"outcome":"admitted"` in
origin/research/cells-census census-wikidata.jsonl.

**Census agreement (C):** records r, letters and cells equal the census on **all 196**
blocks. Cells total 20,456,299.

**Against memtrace (C;** origin/research/cells-memory 95b98eda, trace-cells.jsonl, phase
induce_cells, heap regions tops/bots/arena/rules). Run with SLOT 24, the layout that census
traced (`out/crosscheck.txt`):

| quantity | port (SLOT 24) | memtrace | ratio | worst block |
|---|--:|--:|--:|--:|
| cells | 20,456,299 | 20,456,299 | 1.0000 | 0 |
| stores | 182,381,648 | 182,381,648 | **1.0000** | 0 |
| distinct lines touched by the induction | 14,540,786 | 14,539,858 | **1.0001** | 0.0001 |
| 2 MiB LRU misses in the induction (after `slots`) | 12,027,974 | 12,205,768 | 0.985 | 0.026 |
| loads | 226,884,120 | 191,808,949 | 1.18 | — |
| bytes read / written | 1.120 G / 1.283 G | 1.358 G / 1.446 G | 0.82 / 0.89 | — |

Stores, cells and lines agree. Loads and bytes are not comparable one for one, for two
reasons. First, the compiled kernel merges and widens loads (e.g. the fetch-ahead's two
adjacent 4-byte loads into one). Second, under `slot24` the "other" field is 8 bytes; the
port reads and writes it as 4 bytes at every SLOT, which changes no line. The 1.5%
difference in 2 MiB misses comes from three things the port leaves out: the records and the
`cnt` table in the LRU stream before the induction, the buffer's real base alignment (the
port takes it as 64-aligned), and memtrace's bucket edge (d + 1 >= 2^15).

## 4. Order search (step 4) and validity

- **B, greedy.** Start from DAG(R)'s sources. At each turn take the first K ready steps in
  R's order (K = 64 on all 196 blocks; K = 4096 on every 8th admitted block, 25 blocks, aimed
  at 1.25 and 2 MiB only). Choose the one with the fewest lines not resident in the current
  LRU state at the target capacity. Ties go to the smallest summed age of its lines, then
  R's order. K = 1 is exactly R. The greedy runs once per target capacity (5, 1.25 and
  2 MiB), and every order is evaluated at all three. The registered G uses the one aimed at
  5 MiB.
- **Exact bound instead of exact search.** For any linear extension in model 1, misses at
  any capacity >= distinct lines (**P**: each line's first touch misses, and the line set is
  order-independent). At 5 MiB R attains that bound on 195/196 blocks and B attains it on all
  196. So B is an exact optimum at 5 MiB, and no search can lower G there. A full exact
  search over linear extensions of real blocks or truncations was not run: it was not needed
  at 5 MiB, and at 1.25/2 MiB it was bounded out (the floor above is what is reported).
- **Validity (C, every B reported):**
  - `is_linext`: every dependency precedes its step.
  - `hazards_ok`, byte-exact, over all of R's semantic accesses: for every byte, every
    write stays after all earlier accesses and every read stays after the last earlier
    write.
  
  Passing both means each step reads in B exactly the values it read in R, so it writes the
  same bytes. The rules region, the arena and the slots end byte-identical to R's: the same
  grammar, the same derivation tree, the same writer input and output. 0 failures over
  196 × 3 K=64 orders and 25 × 2 K=4096 orders. R itself also passes. In model 2 the same
  steps read the same values (renamed addresses only). Its writer, given rules allocated in
  execution order, still has each rule's factors before it, as the writer requires.

## 5. Cache model and its check

A fully associative LRU on 64-byte lines. Capacities are 20,480, 81,920 and 32,768 lines
(1.25, 5 and 2 MiB). It is cold per block. A miss is a first touch or a stack distance (distinct
other lines since the last access) >= capacity. Exact stack distances use a Fenwick tree
over access times. `out/lru-selftest.txt`:

- **Hand trace** `A B C A D B A`. By hand: distances cold, cold, cold, 2, cold, 3, 2. Misses at
  2/3/4 lines = 7/5/4. Resident at 3 lines after it: {A, B, D}. The program agrees.
- **Random check.** 60,000 accesses (300 traces) against a naive LRU list at 1, 3 and 8
  lines: misses and residency (the greedy's threshold test) are equal.

## Limits

- **One corpus** (Wikidata), as registered. Modelled lines, not hardware.
- The model has no prefetchers, no set associativity, no write-allocate distinction, and no
  cross-block warmth (as in the memtrace census).
- **Model 1 is optimistic for B** (§2). Even so it cannot beat the compulsory floor, which
  is what decides the verdict.
- **Greedy bounded:** K = 64 everywhere and K = 4096 on 25 blocks. No exact search on
  truncations. The 5 MiB answer is exact anyway (floor attained). At 1.25/2 MiB the greedy's
  distance to the floor is shown, not closed.
- **The DAG claim** is counted on small words only (Conj in general). The real-block orders
  are validated directly at byte level, independent of it.
- **The trace** leaves out the stack and assumes a 64-aligned buffer base. It excludes
  fetch-ahead loads from the primary trace (included in a variant). It excludes `slots`
  (included as a prefix in a variant) and the writer.

## Files

- `src/orders.py`: step 1 (Section-2 engine, DAG, brute force, the cells-form port).
  `out/orders-small-words.txt`: its output.
- `tracer/src/main.rs`: the port, tracer, LRU, greedy, validity checks and model 2.
  `tracer/Cargo.toml`.
- `out/slot20-k64.jsonl`: per block (all 196). Census agreement, access counts, R (primary /
  with fetch-ahead / with prefix), and for each greedy target `B_k64_c{0,1,2}` (c0 = 1.25,
  c1 = 5, c2 = 2 MiB): validity, model 1, model 1 with prefix, and model 2. Each entry has
  `misses` [1.25, 5, 2 MiB], `cold` and `hist` (log2 buckets, last = cold).
- `out/slot20-k4096-every8.jsonl`: the same for K = 4096 (targets c0, c2) on every 8th block.
- `out/slot24-R.jsonl`, `src/crosscheck.py`, `out/crosscheck.txt`: the memtrace
  cross-check.
- `out/summary-k64.txt`, `out/summary-k64-k4096.txt`: pooled tables (`src/summarise.py`).
- `out/lru-selftest.txt`: the hand-checked LRU trace (`locality --selftest`).
- `census.jsonl`, `memtrace-trace-cells.jsonl`: the inputs, read from the two research
  branches.

Reproduce: `cd tracer && cargo build --release`, then
`locality wikidata.bz2 census.jsonl 20 64 1` (all blocks, K=64),
`CAPI=0,2 locality ... 20 4096 8`, `locality ... 24 0 1` (cross-check);
`python3 src/orders.py 10 7`.
