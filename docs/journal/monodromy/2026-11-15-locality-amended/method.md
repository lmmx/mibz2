# Cells induction locality under the AMENDED rule (LOG 6883444614)

Untimed counting. No timing figures. Nothing in /home/user/mibz2 was changed and nothing was posted.
This extends `../locality` (its port of master's `slots` + `induce_cells`, its greedy B and its
validity checks are reused; `mibz2-master/` is a copy of its `git archive origin/master` export).
Scope: the induction only (rule 5). No claim about the whole decoder.

Labels: **P** proved here · **C** counted (exact for its input) · **Conj** conjecture.

## Stated model

- **Cache.** A fully associative LRU on 64 B lines, write-allocate and write-back. A miss on a read
  or a write is one fill. A residency that ends by eviction while dirty is one dirty eviction.
  Capacities: 256 KiB (4,096 lines; the i9-10900K's per-core L2), 5 MiB (81,920) and 2 MiB
  (32,768), the last two being equal shares of the 20 MiB L3 (a modelling assumption).
  1.25 MiB (20,480) is a labelled sensitivity point only. **The verdict is read at 5 MiB.**
- **Initial state (primary).** The cache is cold before `slots`. `slots` (the order stage) then
  runs through the same LRU: both passes' record reads (the records are a separate allocation,
  placed disjoint and 64-aligned), the identity table's 256 stores, every letter's two 20 B slot
  stores, the mark stores and the two closing stores. The induction then starts **with that cache
  carried over**: what it left resident (and dirty) is neither cold nor counted again. The stack
  and the `cnt` table are left out. Dirty lines that `slots` evicted during `slots` count for
  `slots`, not for the induction.
- **Cold start (sensitivity).** The induction starts from an empty cache.
- **Dirty lines at the phase's end. Rule A (primary):** every line still dirty when the induction
  ends counts as one write-back, whether the induction or `slots` dirtied it. This is the
  conservative choice for R. It is applied the same way to R, H and L. Under rule A that term is
  forced for every order, so it belongs in L (proof below). **Rule B (sensitivity):** end-dirty
  lines are not counted, in R, H or L.
- **Q_model** = 64 × (fills + dirty evictions [+ end-dirty lines under rule A]). The ratios use
  Q_model. They are *modelled* traffic, not DRAM bytes.
- **Trace.** As in ../locality: SLOT = 20 (master), every load and store of the induction with
  its kernel width, attributed to steps. The fetch-ahead's `ahead!` loads are left out, as in the
  earlier primary trace. The 196 admitted Wikidata census blocks are used, and the census
  agreement (n, r, letters, cells) holds on all 196 (**C**).
- **Address models.** Model 1 keeps R's addresses for every step. Model 2 places the fresh
  slots, the arena words and the rules with the kernel's bump cursors in **execution order**,
  which is what the kernel itself would do running that order.

## Results (n-weighted: sums over all 196 blocks, n = 176,398,965)

Rule A, initial state after `slots`. Counts are 64 B lines. H is the K = 64 greedy aimed at that
capacity, evaluated in model 2.

| capacity | R fills | R dirty ev. | R end-dirty | Q_model(R) lines | H fills | H dirty ev. | Q_model(H) | L fill LB | L write-back LB | L | **L/R** | **H/R** | blocks with footprint > C |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 256 KiB | 30,773,253 | 21,104,386 | 690,532 | 52,568,171 | 30,271,297 | 20,773,980 | 51,725,543 | 11,818,981 | 10,295,164 | 22,114,145 | **0.4207** | **0.9840** | 196 |
| 2 MiB | 9,227,690 | 7,694,391 | 6,078,857 | 23,000,938 | 8,982,319 | 7,542,728 | 22,586,866 | 6,479,677 | 12,162,083 | 18,641,760 | **0.8105** | **0.9820** | 196 |
| **5 MiB** | 5,634,530 | 4,844 | 12,562,695 | 18,202,069 | 5,634,517 | 4,826 | 18,202,057 | 5,634,386 | 12,567,409 | 18,201,795 | **0.999985** | **0.999999** | 1 |
| 1.25 MiB (sens.) | 12,871,506 | 10,159,364 | 3,788,044 | 26,818,914 | 12,669,406 | 10,021,658 | 26,435,464 | 8,765,700 | 11,243,673 | 20,009,373 | 0.7461 | 0.9857 | 196 |

In modelled bytes per output byte (64 × lines / n), R / H / L: 256 KiB 19.07 / 18.77 / 8.02;
2 MiB 8.35 / 8.19 / 6.76; 5 MiB 6.604 / 6.604 / 6.604; 1.25 MiB 9.73 / 9.59 / 7.26.

Taking the best of the four greedy orders per block (each one verified) instead of the aimed one
gives H/R 0.9739 / 0.9744 / 1.0000 / 0.9798 (256 KiB / 2 MiB / 5 MiB / 1.25 MiB).

**Sensitivities, as L/R; H/R** (all tables in `out/summary.txt`):

| | 256 KiB | 2 MiB | 5 MiB | 1.25 MiB |
|---|--:|--:|--:|--:|
| rule A, after `slots` (primary) | 0.4207; 0.9840 | 0.8105; 0.9820 | **0.99998; 1.0000** | 0.7461; 0.9857 |
| rule B, after `slots` | 0.4108; 0.9840 | 0.7221; 0.9765 | 1.0000; 1.0000 | 0.6945; 0.9852 |
| rule A, cold start | 0.4316; 0.9857 | 0.9589; 0.9960 | 1.0000; 1.0000 | 0.8682; 0.9938 |
| rule B, cold start | 0.4218; 0.9857 | 0.9250; 0.9956 | 0.9998; 1.0000 | 0.8356; 0.9947 |

**Footprint** (model-1 distinct lines × 64 B): 3.24 / 3.79 / 5.29 MiB (min / median / max).
Every block's footprint exceeds 256 KiB, 1.25 MiB and 2 MiB. Only one block (block 158) exceeds
5 MiB.

## Verdict under the amended rule (5 MiB)

**CLOSED.** The certified L/R is **0.999985 ≥ 0.85** (rule A, after `slots`). It holds as well
under rule B (1.0000) and cold (1.0000 / 0.9998). WORTH is not met: the best verified H/R is 0.999999.

At the other capacities nothing is settled, and they decide nothing under the rule. The intervals
L/R ≤ O/R ≤ H/R are:
- 256 KiB: [0.42, 0.98];
- 2 MiB: [0.81, 0.98];
- 1.25 MiB: [0.75, 0.99].

These intervals are wide because L counts only compulsory traffic.

## Realizability of H (the certificate) — P given C checks

For each block and each of the four greedy orders, two things were checked (**C**: 196 × 4 orders,
0 failures):
- `is_linext`: the order is a linear extension of the step DAG, so every promised-slot / slot-writer
  dependency comes before its step.
- `hazards_ok`: every byte-exact read/write and write/write hazard of R's semantic accesses is
  kept, so every step reads, byte for byte, the value it read in R.

**Model 2 is realizable (P).** Steps keep their accesses, and only allocated objects move, each
object moving as a whole. Objects are disjoint in both layouts, so the hazard check at R's
addresses is the same check at model-2 addresses. Copy16 writes up to 16 B past an object's end,
and in model 2 this slack always lands at or past the current bump cursor, in space no one has
allocated yet. A later object's own write then overwrites it. Slack loads only feed slack
stores. So each step writes the same words (renamed addresses), and slots, arena and rules end
equal to R's up to that renaming: the same grammar and the same derivation tree. Every reported
H is model 2.

**Model 1 is not realizable for these reorders (C).** A full-width check (`hazards_full`) used the
kernel's widths, copy16 slack included. It requires every loaded byte to have R's last writer and
every byte to end with R's last writer. R passes. All 784 greedy orders fail it. At R's fixed
addresses, a step that runs early overwrites, with its copy16 slack, a word that R had not yet
allocated but B has already written. The earlier run's validity rested on semantic bytes only, so
under the amended rule its model-1 B does not count as H. (Model-1 numbers stay in the jsonl as
`m1`, labelled.)

## The certified lower bound L — P

Let Res_c and Dirty_c be the resident and the resident-dirty lines after `slots` at capacity c.
They are the same for every order, because `slots` runs before the induction and does not depend
on the order.

**S_g, the lines every realizable order touches (models 1 and 2):**
- Accesses outside the three bump-allocated regions are at fixed addresses in every order. These
  are the original letters' slots, the identity table and the closing read.
- In every order, each bump region is one contiguous tiling from its start, with the same objects
  of the same sizes. Its guaranteed bytes are:
  - the arena words' own bytes, [dyn0, dyn0 + Σlen);
  - the rules, [rules, rules + 16m);
  - bytes 0..4 of each fresh slot, written when it is cut. Since SLOT = 20 < 64, every line between
    the first and the last fresh slot's start holds a slot start.
- Copy16 slack and the slot fields written later are left out, since their position depends on
  the order.

D_g, the lines every order writes, is built the same way. S_g has 12,567,184 lines, and R's
model-1 set has 12,567,310. Every evaluated order (R, and H in models 1 and 2, warm and cold)
touched all of S_g: 0 missing (**C**).

**Fills.** Any order fills each line of S_g \ Res_c at least once: its first touch in the
induction is a miss. So fills ≥ |S_g \ Res_c|.

**Write-backs, rule A.** Let X_c = D_g ∪ Dirty_c. Every line of X_c is dirty at some moment of the
phase: at the start, or right after its guaranteed write. From then on it stays dirty until it
is evicted (a dirty eviction) or the phase ends (an end-dirty write-back, counted under rule A).
So every order has at least |X_c| counted write-backs, one per line.

**Write-backs, rule B.** At the end at most C lines are resident. Every line of X_c that is not
dirty and resident at the end must have been evicted dirty at least once. So dirty evictions ≥
max(0, |X_c| − C).

**L** = 64 × (|S_g \ Res_c| + the write-back bound of the rule).

What is bounded: Q_model, for every realizable order under master's layout and allocation policy
(R's addresses or bump-in-execution-order). It is *not* bounded for an allocator that reuses dead
slots or packs differently. Such an allocator changes the line set, and that is outside this
bound. The bound uses no relaxation of the realizability constraints.

## LRU write-back check — C (`out/lru-wb-selftest.txt`, `locality --selftest`)

Hand trace W A, R B, R C, W B, R A, R C at 1, 2 and 3 lines:

| | 1 line | 2 lines | 3 lines |
|---|--:|--:|--:|
| fills | 6 | 5 | 3 |
| dirty evictions | 2 | 2 | 0 |
| dirty at end | 0 | 0 | 2 |

At 2 lines, A is evicted dirty by C and B is evicted dirty by the last C. At 3 lines, A and B are
still dirty at the end. The program agrees.

The phase boundary was checked too, at 2 lines after W A, R B, R C. The order stage has 1 dirty
eviction (A), and B and C are resident and clean. Then W B, R A, R C gives 2 fills, 1 dirty
eviction (B) and 0 dirty lines at the end. The program agrees.

The model was also checked on 60,000 random accesses, one in three a write. At 1, 3 and 8 lines,
its fills, dirty evictions and dirty-at-end counts equal those of a naive write-back LRU list.

## Limits

- One corpus (Wikidata). Modelled lines, not hardware: no prefetchers, no set associativity, no
  non-temporal stores, no cross-block warmth before `slots`.
- The fetch-ahead loads, the stack and the `cnt` table are left out. The buffer base and the
  records' base are assumed 64-aligned.
- H is the earlier greedy (K = 64, scored on fills only, seeded with `slots`' cache) aimed at each
  capacity. It is a bounded heuristic. At 256 KiB, 2 MiB and 1.25 MiB the gap to L is open: L is
  compulsory-only, so either the bound or the orders could be improved.
- The DAG claim from ../locality (all legal orders that give R's tree = linear extensions) is still
  **Conj** in general. H's validity does not depend on it: it is checked per order.

## Files

- `tracer/src/main.rs`: the port. It adds the `slots` record reads, the write-back LRU (`WbLru`,
  `evaluate_wb`), `guaranteed` (S_g, D_g), `hazards_full`, `selftest_wb` and the greedy seeded
  with `slots`. `tracer/Cargo.toml`. `mibz2-master/` is the export.
- `out/wb-k64.jsonl`: one line per block. Indices into the arrays are capacities
  [256 KiB, 5 MiB, 2 MiB, 1.25 MiB]. Each line holds:
  - R and R_cold: fills, devict, enddirty;
  - fill_lb, X_dirty, start_resident and start_dirty;
  - H0..H3, the greedy aimed at each capacity: validity flags, and m1, m2, m2_cold.
- `src/summarise.py` → `out/summary.txt`: all four rule × start tables.
- `out/lru-wb-selftest.txt`.
- `census.jsonl`: the input.

Reproduce: `cd tracer && cargo build --release`, then
`locality /home/user/mibz2/corpora/wikidata.bz2 ../census.jsonl 64 1 > ../out/wb-k64.jsonl`,
then `python3 src/summarise.py out/wb-k64.jsonl`.
