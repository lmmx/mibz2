# Cells induction: which stored state is dead or derivable? (untimed counting)

No timing. Nothing in /home/user/mibz2 was changed and nothing was posted. Kernel: origin/master
55a58b18, `src/cells.rs` (copy in `cells.rs`). It differs from the export in
`../locality/mibz2-master` only in two `phase-counters` cfg scopes, so the induction is
unchanged. SLOT = 20, GAP = 0 (release). Corpus: the 196 `admitted` Wikidata blocks of
origin/research/cells-census `census-wikidata.jsonl` (`census.jsonl`, byte-identical to
locality's copy).

Labels: **P** proved · **C** counted (exact for this input) · **Conj** conjecture.

## Verdict

- **Bytes stored by the induction (C):**
  - L (dead) = **0.255** pooled, 0.253 / 0.255 / 0.256 (min / median / max per block).
  - U (dead, or covered by a proved rule) = **0.305** pooled, 0.299 / 0.305 / 0.314.
  - Implementable without an oracle, decided at the store: U_static = **0.241**, 0.236 / 0.241 / 0.250.
  - With `slots` included: L 0.292, U 0.450, U_static 0.377.
  - So the elidable share of bytes is not small.
- **Lines (what the 3.2–5.3 MiB footprint and the misses are made of; C):**
  - L = U = **24 lines of 12,567,310**, i.e. 2·10⁻⁶ of the footprint.
  - Every elidable field sits in a 64-B line that also holds live fields of the same or an
    adjacent slot.
  - **Elision in place, keeping the 20-B slot layout, has a structural ceiling of about 0 in
    footprint and in misses.** What it can remove is stores and bytes (up to 24% of
    induction bytes, implementable), not lines. If the elision PR is meant to cut footprint
    or misses, the structure says stop.
- **The one structural route to fewer lines is a layout change (C on lines, cost Conj):**
  - Read the bottom slot's (λ, w) through `other` from the top slot (rule R4 below, **P**,
    counted with 0 disagreements).
  - The bottom slot then needs only (prev, other): 8 B. Layout "A8" (20-B top slots, 8-B
    bottom slots).
  - Footprint: **0.747** of today's lines, pooled; per block 0.737 / 0.747 / 0.757
    (−25.3%; median 3.79 → 2.83 MiB).
  - Price: one dependent load on 53.7% of slot reads (the bottom-first ones). That load
    goes to a line the same step writes anyway (B's top slot, the cell's destination in
    every branch).
  - Whether that pays is a clock question. The 16-B slot was rejected on compute; this is
    a different trade, and it is not measured here.

## 1. Field-level liveness (C)

Method: `tracer/src/main.rs` ports `slots`, `induce_cells` and the load pattern of
`write_cells`.
- **Stores.** Each stored byte is one "byte event".
- **Loads.** Each load is attributed to the events it reads. It is *semantic* when its
  value is used. Not semantic:
  - copy16's slack beyond a word;
  - fields a drop or close branch ignores (0 drops occur on this cohort);
  - the fetch-ahead (excluded; its value is only a prefetch address).
- **The writer's reads count as reads:**
  - rules: 16 B per `rule()` call, on its three spine walks and both passes over the
    reached rules;
  - arena bytes: `copy_small`'s `lx` bytes;
  - the final byte.
  - The writer reads no slot field.
- **Dead** = never semantically read.
- **Store level.** For copy16, dead counts only whole 16-B stores with no live byte. The
  slack is reported apart.

Census agreement on all 196 blocks: n, r, letters and cells equal (C). No load read a
never-written byte.

| fraction of bytes stored (pooled · min / median / max) | induction (1.283 GB) | `slots` (0.444 GB) | both |
|---|--:|--:|--:|
| **L** dead | **0.2552** · 0.2529 / 0.2554 / 0.2563 | 0.4000 · 0.4000 / 0.4000 / 0.4000 | 0.2924 · 0.2901 / 0.2925 / 0.2953 |
| copy16 slack (dead bytes inside live 16-B stores; not in L) | 0.2962 · 0.2835 / 0.2962 / 0.3098 | 0 | 0.2201 |
| L in lines (lines holding only dead bytes, no load) | 24 / 12,567,310 · 0 / 0 / 0 | — | 249 / 12,567,535 |

**Which stores are dead (C, and P from §3.5):** in every letter, exactly one side's (λ, |w|,
w, other) is dead: 16 of the 20 B on the side the fronts reach second. That side is the
letter's own first cell, and its first cell overwrites it unread.
- This holds for `slots` (16 of the 40 B per original letter) and for every cell (16 of
  the 32 slot bytes per cell).
- The dead side is 52–57% top, depending on the site. No store site is dead on every
  instance (L_static ≈ 0). Dropping L's stores therefore needs to know which front gets
  there first, which is not known when the store is made.
- Nothing else is dead at store level: promise prevs, links, rules and arena copies all
  have reads. Every slot field read is read exactly once (no slot byte is read more than
  once). Rules are read at least twice (the writer).

## 2. Derivation rules

| rule | status | condition (decidable at the store) | covers | bytes removed alone (pooled) |
|---|---|---|---|--:|
| (d) **R1** prev = this − SLOT | **P**, C: 22,181,461 reads apply, 22,181,342 agree; the 119 others are the first slot of an order (prev = itself, never followed), not covered | non-fresh slot, not rewritten by a promise link (flag: bit 1 of `other`, whose low 2 bits are spare because slot offsets are multiples of 4) | all of `slots`' prev stores (both sides) | 88.7 MB = 0.200 of `slots`, 0.051 of all |
| (e) **R2** original word: (\|w\|, w) = (1 [\| MARK], arena+16+c), c = the bucket of the bottom slot (the top side reaches it via `other`), MARK at the two z slots in registers | **P**, C: 11,090,867 reads, 0 disagree | the slot holds no cell yet (flag: bit 0 of `other`) | `slots`' \|w\|, w on both sides | 177.5 MB = 0.400 of `slots`, 0.103 of all |
| (e) \|w\| of a cell from its rule or arena entry | **refuted** | arena words carry no length. A rule's length is the sum of its factors' lengths, which needs the 16-B rule the reader does not hold (the induction never reads rules). Counterexample: every cell | — | 0 |
| (a) **R3** λ = 1 of a one-row cell (and of an original letter with λ = 1) | **P**: λ = min(ra, rb) = 1 by definition. C: covers all 23,820,633 reads with λ = 1 | λ = 1 at the store. Flag in \|w\| bit 30 for cells (\|w\| ≤ n < 2²⁴), in a spare high bit of `other` for originals (the layout is < 2²⁸ for n < 2²⁴) | λ of one-row cells and λ = 1 originals | 190.6 MB = 0.100 of induction, 0.141 of `slots`, 0.110 of all |
| (b) λ equals something the reader holds | **refuted** | λ is the comparand of the three-way branch: it equals the partner's remainder on exactly the equal steps (11,090,671 of 31,546,970 reads), and the reader needs λ to know that. Counterexample: any unequal step (9,365,628). Only λ = 1 is structural (R3) | — | 0 |
| (c) `other` of first cells / of any slot | **refuted** | `other` holds the pairing (where the letter's next cell goes), set by the partner letter's position. Same-index-in-the-other-region holds on 1,255 of 31.5 M reads; no local rule. It is instead the key that makes R4 work | — | 0 |
| promise prevs (fresh slot's prev; link = the fresh cursor) | **refuted** | the value moved is the old prev and the cursor is global state | — | 0 |
| (f) **R4** bottom (λ, w) read through `other` from the top slot | **P** (§3.5: a letter's slot on its second side is overwritten only by its own first cell, which is cut after the letter is read, so the first-side slot still holds the letter when the second side is read first). C: 0 disagreements over 31.5 M λ and w reads | always: bottom slots never store λ, w | bottom λ, \|w\|, w of `slots` and of every cell | 378.6 MB = 0.191 of induction, 0.300 of `slots`, 0.219 of all |
| (f) symmetric R5 (top read through the bottom) | **P**, same counts | — | — | the same bytes (A and B are byte-symmetric); A is better for lines (more bottom slots: 16.93 M vs 14.62 M) |
| (f) copy16 slack | not a store elision | exact-length copies would drop 0.296 of induction bytes, but no store instruction and no line (the next word overwrites the slack in the same lines) | — | (bytes only) |

z, the equal one-row cells (C):
- **z = 9,218,588 = 0.762 r** (0.831 letters, 0.451 cells); per block 0.691 / 0.752 /
  0.861 r. The memo's 0.842 r was on the older 233-block cohort.
- All one-row cells: 1.321 r (0.782 of cells).
- Their slot stores (295 MB): exactly half are dead (the second side); the other half is
  read once.
- Of what is read, λ is always derivable (R3). Their w, \|w\| and `other` are needed
  (R4 moves the bottom side's to the top).
- So in this kernel, as §8.8 says, an equal one-row cell still needs its writes. Only its λ
  and its second side go.

**U** (dead, or every read covered; read-throughs make the bytes they load live; checked
per byte; violations 0):

| U, fraction of bytes stored (pooled · min / median / max) | induction | `slots` | both |
|---|--:|--:|--:|
| U_static, set C = {R1, R2, R3} (decided at the store, no oracle) | 0.0997 · 0.0893 / 0.0994 / 0.1167 | 0.7411 · 0.7255 / 0.7392 / 0.7676 | 0.2646 · 0.2506 / 0.2636 / 0.2943 |
| U_static, set A = C + R4 | **0.2412** · 0.2356 / 0.2411 / 0.2498 | 0.7705 · 0.7627 / 0.7696 / 0.7837 | **0.3773** · 0.3693 / 0.3765 / 0.3955 |
| **U** = U_static ∪ L (the same for sets C, A and B) | **0.3050** · 0.2993 / 0.3049 / 0.3136 | 0.8706 · 0.8628 / 0.8696 / 0.8838 | **0.4504** · 0.4422 / 0.4496 / 0.4692 |
| U in lines (layout unchanged) | 24 / 12,567,310 | — | 249 lines |
| layout A8 (8-B bottom slot), lines / today's | **0.7474** · 0.7374 / 0.7467 / 0.7567 | — | — |

R4 is the implementable form of most of L. It does not need to know which side is dead: per
letter it drops 12 of L's 16 dead bytes whichever side the fronts reach first. Only the
dead side's 4-B `other` still needs the oracle.

U is relative to the rules above: (a)–(f), plus spare-bit flags assumed free. It is not a
bound over every conceivable re-encoding. Packing fields, as in the 16-B slot, is
compression, not elision.

Pooled by region and field: `out/summary.txt`, table "By region/field". Each rule alone:
`out/rules-bytes.txt`.

## 4. Two workers on one block's induction (Conj, no build)

- **Parallelism exists in the DAG (C).** About 104k steps against a median depth of 3,861:
  an average width of about 27. A two-way split is not limited by the critical path, and
  its floor is W/2 per worker.
- **What any split must pay:**
  - **Line transfers between cores.** Today's misses are essentially all compulsory (locality
    run: 5 MiB misses = distinct lines on 195 of 196 blocks). Splitting a block therefore
    does not reduce its line traffic. It divides that traffic between two cores and adds a
    transfer for every slot that one worker writes and the other reads.
  - **Waiting.** A step whose input slot comes from the other worker must wait until that
    slot is written.
- **The slot dataflow is single-producer, single-consumer (C here).** Every slot field is
  written by one cell step and read exactly once. The handover could therefore be a "written"
  bit in a spare bit of λ or `other`, with no locks: no extra bytes, but a check on every
  step.
- **The steps run in a fixed order.** The fronts consume both orders in sequence, and each
  step reads one letter from each front. So the work cannot be split by front, only by
  rounds or segments (same-round segments commute).
  - A step taken out of the sweep must keep its open chain's state somewhere: remainder,
    words, promised slot (locality §2, model 1). It also loses the fetch-ahead.
  - Bump allocation must be split per worker. The writer relies on a rule's factors having
    lower indices, so two rule streams need a merge or a topological pass in the writer.
- **The ceiling.** The optimistic two-team floor (0.196 s against 0.162 s independent at
  4 cores) already shows that while blocks ≥ workers, cooperation cannot beat independent
  blocks. Those already have zero coordination and the same total line traffic.
  Cooperation can only shorten the last block's latency, the tail. Its gain is bounded by
  the tail's share of wall time minus the cost of the step checks and transfers.
- **Elision does not change this.** R4 adds only same-step reads, and A8's footprint cut
  helps both schemes equally.

## Limits

- One corpus. Lines are modelled (64 B, the buffer base taken as 64-aligned); not hardware.
- The port's writer reads are ported from `write_cells`, not traced against memtrace (the
  induction's stores and lines were cross-checked by the locality run).
- The semantic model is mine: a field counts as read when its value reaches a use. The
  fetch-ahead is excluded.
- The A8 line count remaps bottom-slot offsets and keeps every other region's addresses; its
  compute cost is not measured.
- Bounded work: one pass over the 196 blocks, one core, about 100 s a run, 4 runs.

## Files

- `tracer/src/main.rs`, `tracer/Cargo.toml`: the port, the liveness and derivation checks,
  and layout A8. The dependency is `../../locality/mibz2-master`, read only.
- `out/blocks.jsonl`: raw per-block data.
  - `t["site.field"]` = [bytes, dead, once, >1, dead-phys, cov R1, R2, R3, elid C, A, B,
    static C, A, B, ceil C, A, B, in-dead-store].
  - The probes, `lines_ind`, `lines_a8`.
  - Sites: 0/1 slots top/bot (λ > 1), 34/35 slots top/bot (λ = 1), 2 mark, 3 first prev,
    4 byte table, 5 + 4k + 2·one-row + fresh for k = eq_ts, eq_bs, gt_ts, gt_bs, lt_bs,
    lt_ts, 29 promise fresh prev, 30 promise link, 31/32 copy16, 33 rules.
- `out/per-block.jsonl`: per-block L, U, U_static, slack, lines, A8.
- `out/summary.txt` (`src/summarise.py`), `out/rules-bytes.txt` (`src/rules_bytes.py`).
- Reproduce: `cd tracer && cargo build --release --offline`, then
  `target/release/elision /home/user/mibz2/corpora/wikidata.bz2 ../census.jsonl > ../out/blocks.jsonl`
  and `python3 src/summarise.py out/blocks.jsonl`.
