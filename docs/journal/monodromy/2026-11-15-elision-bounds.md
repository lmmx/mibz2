# 2026-11-15 · MONODROMY 5 · What the Cells induction stores and never needs: certified bounds. Large in bytes, nil in lines

Assignment: QUEUED 6893931538 on #145. Untimed: a byte-level trace of master's `slots` and
`induce_cells` (55a58b18 `src/cells.rs`) on the 196 admitted Wikidata census blocks, with the
writer's reads ported from `write_cells`. Labels: **P** proved · **C** counted exactly · **Conj**
conjecture. Full tables, method and reproduction: `2026-11-15-elision-bounds/method.md`.

## Answer

- **In bytes stored, the elidable share is large:** a certified lower bound L = **0.255** of the
  induction's bytes are never read (dead), and with four proved derivation rules the upper
  bound is U = **0.305** of the induction's bytes (0.45 of induction + `slots`).
- **In 64-byte lines it is essentially zero:** 24 of 12,567,310 induction lines hold only
  elidable bytes. Every elidable field shares its line with live fields. **So elision that keeps
  the 20-byte layout removes stores, not footprint or misses. As a route to fewer lines or less
  traffic, its structural ceiling is about 0: stop it.**
- **The one structural route to fewer lines is a layout change** (A8 below): induction lines
  0.747 of today's, at the price of a dependent load on 54% of slot reads, whose cost is a clock
  question and is not measured.

## 1. Dead bytes, L (C; why: P from entry 1 §3.5)

| | induction (1.283 GB stored) | `slots` (0.444 GB) | both |
|---|--:|--:|--:|
| **L**, pooled (per block min / median / max) | **0.2552** (0.2529 / 0.2554 / 0.2563) | 0.4000 | 0.2924 |
| L in lines | 24 of 12,567,310 | — | 249 of 12,567,535 |

In every letter exactly one side's λ, |w|, w and partner field (16 B) is dead: the side the
fronts reach second, which the letter's own first cell overwrites unread (§3.5: a cell is read
at most once, on the side reached first). The dead side is the top one in 52–57% of cases by
store site, so no store site is always dead: dropping the stores directly needs to know which
front arrives first, which is not known when the store is made. Nothing else is dead. Every
slot field is read exactly once; rules are read at least twice.

## 2. Derivable state, U (each rule P; reach C; 0 violations over all blocks)

| rule | status | bytes it removes alone |
|---|---|--:|
| R1: prev = this − SLOT where the slot is neither fresh nor relinked by a promise | **P**; 22,181,342 of 22,181,461 reads (the rest: an order's first slot, never followed) | 88.7 MB |
| R2: an original letter's (\|w\|, w) = (1, arena + 16 + c), c its byte | **P**; 11,090,867 reads, 0 disagree | 177.5 MB |
| R3: λ = 1 for one-row cells and one-row letters (flag in a spare bit) | **P** (λ = min(ra, rb)); all 23,820,633 reads with λ = 1 | 190.6 MB |
| R4: read the bottom slot's (λ, w) through the partner field from the top slot | **P** (§3.5: the first-reached side still holds the letter); 0 disagreements in 31.5 M reads | 378.6 MB |
| \|w\| of a cell from its rule or arena word | **refuted**: arena words carry no length; a rule's length costs the 16-byte rule load the induction never makes | 0 |
| λ equal to a value the reader holds | **refuted**: it equals the partner's remainder only on equal steps (counterexamples: 9,365,628 unequal steps) | 0 |
| the partner field | **refuted**: it encodes the pairing (1,255 of 31.5 M reads equal the same-index slot) | 0 |
| promise links | **refuted**: they carry the moved link and the fresh-slot cursor | 0 |

| | induction | `slots` | both |
|---|--:|--:|--:|
| {R1, R2, R3}, decided when the store is made | 0.0997 | 0.7411 | 0.2646 |
| {R1–R4} | 0.2412 | 0.7705 | 0.3773 |
| **U** = {R1–R4} ∪ L | **0.3050** (0.2993 / 0.3049 / 0.3136) | 0.8706 | **0.4504** |
| U in lines, layout unchanged | 24 of 12.57 M | — | 249 |

U is relative to these rules and counts spare-bit flags as free. It is not a bound over every
re-encoding: packing fields, as the 16-byte slot did, is compression, not elision.

**z, the equal one-row cells, on this cohort:** 9,218,588 = **0.762 r** (per block 0.691 /
0.752 / 0.861 r; the earlier 0.842 r was the older 233-block cohort), 0.451 of cells. Of their
295 MB of slot stores, half are dead (the second side) and half are read once; their λ is
derivable by R3, their w, |w| and partner field are needed.

## 3. The route to fewer lines: layout A8 (lines C; cost Conj)

With R4, bottom slots need only the link and the partner field: 8 bytes, top slots staying 20.
Induction lines fall to **0.747** of today's pooled (per block 0.737 / 0.747 / 0.757); the median
block's footprint from 3.79 to 2.83 MiB, the largest from 5.29 to 3.91 MiB. The price: 53.7% of
slot reads (those that reach a letter first on the bottom) gain one dependent load. It goes to
the top slot the same step writes in every branch, so the line is touched anyway and the
existing fetch-ahead already prefetches that address. Whether that pays is a clock question,
and a different trade from the rejected 16-byte slot.

## 4. Two workers on one block (Conj; no build)

- **Parallelism exists:** about 104 k steps against a median step-DAG depth of 3,861, an average
  width near 27.
- **It cannot reduce traffic:** misses are already essentially all compulsory (#397), so a
  split divides the lines between two cores and adds a cross-core transfer for every slot one
  worker writes and the other reads.
- **Handover could be lock-free:** each slot field is written once and read exactly once (C), so
  a spare-bit "written" flag suffices; that costs a check on every step and a wait where a step
  needs the other worker's slot.
- **Constraints:** a split must be by rounds or segments, since each step consumes both fronts.
  Steps taken out of the sweep keep their open chains and lose the fetch-ahead. Two rule streams
  break the writer's rule that factors have lower indices, so they need a merge pass.
- **Ceiling:** while blocks are at least as many as workers, cooperation cannot beat
  independent blocks, which have the same traffic and no coordination. That agrees with the
  optimistic 0.196 s floor against 0.162 s at 4 cores. Its only room is the last-block tail.

## Limits

One corpus. Lines are modelled (64 B, buffer base 64-aligned), not measured. The writer's reads
are ported, not traced. A field is read when its value reaches a use; fetch-ahead loads are
excluded. A8's compute cost is not measured.
