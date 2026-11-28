# 2026-11-18 · MONODROMY 67 · #579 (live reuse in Cells' writer) verified independently. The owner certificate equals true last use on every rule. The one-scan schedule is exact on every full-period block, with every source complete when read. The periodic fallback is exact, and the 36-row counterexample reproduces. Every count and ledger figure reproduces from my own model. The writer is 18.9–19.0% of eager kernel time (archived i9); no timed lazy share exists, only instruction shares of 13–16%

**The request.** The Coordinator's CHECKPOINT 6986297212: an independent verification of Astra's
draft #579 at 8eb17de4, `docs/journal/2026-11-18-cells-live-reuse.md`.
- *Conduct:* I have not read the Researcher's verdict, and have not commented on #579.

**Labels:** **P** proved or checked by proof · **C** counted · **I** interpretation.

**Method.** I extended record 66's independent two-order induction (`mixedcheck.rs.txt` beside
this record; linked lists, not #579's transcription) with a writer simulator.
- *Rules,* exactly as stored: products longer than 16 bytes, or marked. Every other product is an
  arena word.
- *The owner certificate:* the same-side child of the next stored rule, checked against the true
  last parent of each rule.
- *The schedule:* the one-scan reverse pass, then the forward command pass, on a fill bitmap. It
  checks four things:
  - every short write lands on an unfilled byte;
  - every copy source is fully filled at the moment of the copy;
  - no byte is written twice;
  - the command store never overtakes the records read.
- *The fallback:* the reachable-offer pass and forward expansion.
- *Ground truth:* LF inversion.
- *Inputs:* the corpus SHA-256s are those of record 66.

## Verdicts

| # | claim | verdict | basis |
|---|---|---|---|
| 1 | Owner theorem and its two-rule certificate, with marked rules, E steps and short products | **VERIFIED** | P (§1); C: 0 certificate errors on 8,176,313 + 531,333 rules, 1.37M stress-test rules, and the small suites |
| 2 | Periodic blocks: full-period guard, exact fallback, 36-row counterexample | **VERIFIED** | P, C (§2) |
| 3 | Copy schedule: every source complete (incl. 903,299 from later addresses); in-place command storage | **VERIFIED.** The certificate *must* use the carried pair, as the study says | P, C (§3) |
| 4 | Counts and ledger; positive on every block in both populations | **VERIFIED, exactly.** The ledger follows the code at source level; what it leaves unpriced is listed | C, P (§4) |
| 5 | Relevance: writer about 19% of eager kernel time | **VERIFIED** (18.9–19.0%, 196 eager-admitted blocks, three i9 censuses). **There is no timed lazy share;** instruction shares are 13–16% | C (§5) |
| 6 | Side results | **VERIFIED:** totals, row 1's split, the zero different-tree rows, the TT and BB forwarding count and its code identity. Rows 2–4 of the classification were not independently re-derived | C, P (§6) |
| 7 | Theory | The owner tree is the spanning tree along which output offsets propagate: record 59's "consistent on a tree". Ownership ends each letter's Zorich streak. **No gap-chain identity** | P, I (§7) |

## 1. The owner theorem (P, C)

**Consecutive uses (P).** A letter enters a cut only as a front.
- *Its next cut:* a strict win keeps it as the same front, so the next cut uses it again on the same
  side. A loss or E exhausts it.
- *No closure in between:* after a strict cut the top front is the winner, and the bottom front is
  D ≠ winner or the new cell. So no letter is last in both orders.

**The certificate (P).**
- *A surviving rule child forces the next product to be a rule.* That product contains the child, so
  it is longer than 16 bytes or marked. So no arena product intervenes, and the next stored rule is
  the next cut.
- *A dead child never reappears.* The shipped rule identities are fresh per letter.
- *Marked rules never survive (P).* Every marked cell has width 1, since its width is at most the
  marked singleton's. A width-1 letter cannot strictly win, so marked children never become
  deferred copies.
- *E steps* exhaust both operands, so the certificate correctly reads "owner".

**Counted:** certificate = true last parent on **every** rule child:
- all 233 Wikidata blocks and all 6 prose controls;
- 20,000 repetitive random texts (1.37M rules);
- the small suites at thresholds 16, 2 and 1 (thresholds 2 and 1 make small grammars
  rule-heavy).

## 2. Periodic blocks and the complete scheme (P, C)

**The mass argument holds (P).**
- *The invariant:* Σ width × word length is conserved by every cut and starts at n.
- *Closures:* each one removes positive mass.
- *So:* if p = n, the marked closure accounts for all of the mass. There is no unmarked closure,
  and every rule feeds the root.
- *Hence:* following owners strictly increases birth index and ends at the root, giving a spanning
  tree. If p < n, the other LF cycles close unmarked, and their rules are unreachable.

**The witnesses** (shipped threshold 16), both reproduced:
- *L = 11 0³³, primary 2 (full period):* one deferred 17-byte copy, from offset 17 to offset 0.
  That is a source *later* in the output. Exact.
- *L = 11 0³⁴, primary 3 (period 18 in 36 rows):* three rules. The unguarded owner schedule
  leaves two rules unassigned and one copy without a source. The fallback is exact.

**The complete scheme** (guard p = n; the reachable-offer fallback otherwise) is exact on every case
I ran:
- *Small columns:* 14,209 cases at each of thresholds 16, 2 and 1. 2,017 have full period (the
  study's figure); the owner schedule never fails on them. The 12,192 short-period cases are exact
  under the fallback. The unguarded owner schedule fails on 106 of them at threshold 1 and 12 at
  threshold 2, as expected.
- *Random columns:* 20,000.
- *Repetitive texts:* 20,000, including 4 periodic.
- *The corpus:* all 239 blocks.

**Why any reachable offer is exact (P).** The forward expansion visits each child before all its
parents. Every offer is a reachable occurrence, so the child's chosen occurrence is complete before
any parent copies from it.

## 3. The copy schedule (P, C)

**Sources:**
- *Every copy's source was complete when read, and disjoint from its destination:* the source was
  fully filled and the destination unfilled at that moment.
- *No output byte was written twice, and the output is exact:* 233 of 233 and 6 of 6.
- *From later addresses:* **903,299** of 2,528,876 copies on Wikidata, and 4,721 on prose.

**In-place storage:**
- *At most one command per parent:* 0 exceptions. A cut has at most one surviving child.
- *The command store never overtakes the records read:* commands ≤ records read at every step, on
  every block.
- *A command* is a 64-bit descriptor plus a 32-bit destination: 12 of the record's 16 bytes.

**A precision (P).** The command count can **equal** the number of records read. Then the record
just read, k + 1, may already be overwritten when rule k is processed. So the certificate's "next
stored rule" must come from the **carried descriptor pair**, not from re-reading record k + 1. The
study specifies this ("retain the later descriptor pair in two registers") and charges the two moves
per rule. An implementation that re-reads the record would be wrong.

## 4. Counts and the ledger (C, P)

**Matches against #579's CSVs.** Every field, block by block:
- *Fields:* rules, marked rules, long edges, copies, copy bytes, literal edges, literal bytes, wraps,
  and copies from later addresses.
- *Blocks:* all 233 Wikidata and 6 prose.
- *Result:* **0 mismatches** against `blocks.csv` and `schedules.csv`.

| | R | K | from later | copy bytes | literal bytes |
|---|---|---|---|---|---|
| Frozen | 28,710 | 9,628 | — | — | — |
| All 233 | 8,176,313 | 2,528,876 | 903,299 | 161,617,350 | 48,080,312 |

**The ledger, recomputed from my counts.**

| population | metadata accesses | structural units | min margin | break-even (units per rule) |
|---|---|---|---|---|
| Frozen | 287,178 → 162,980 (**43.25%**) | 535,922 → 450,471 (**15.94%**) | | |
| All 233 | 81,149,187 → 45,349,632 (**44.12%**) | 151,568,927 → 125,944,041 (**16.91%**) | **76,157** | **2.719** |
| Eager (196) | 43.26% | 15.96% | | |
| Lazy (228) | 44.01% | 16.79% | | |
| Prose | 54.77% | 28.55% | | |

- *Every block is positive in both populations.*
- *Charging 2K more* still leaves a minimum margin of 59,445.

**Does the ledger follow the real writer?** Yes, at source level. I checked `write_cells` at 8eb17de4,
whose shipped writer makes four passes:
1. the marked walk;
2. a marked walk that seeds positions;
3. a reverse pass that reads each rule and takes the minimum position of each long child;
4. a forward pass that reads each rule and its position, looks up the source, compares and copies;

then a third spine walk to re-copy.

How the ledger's terms map onto it:
- *The baseline's 6R + 3J* is these passes, field by field.
- *Both sweeps visit only unmarked reachable rules.* An unmarked parent has no marked child. So 4U
  child-class tests and 2R NIL tests are right.
- *It undercharges the baseline slightly:* marked rules are read on all three spine walks, against
  the ledger's 4R.

**Production uses this writer.** `StreamDecoder::new` uses eager Cells or `LazyCells`, and both call
`write_cells` with 16-byte rules. The 12-byte arm (`LAZY_W12`, its own `write_w12`) is reachable
only through `new_lazy_as` and is not covered.

**Unpriced (I).**
1. **Branches.** The certificate's equality test is data-dependent. About 31% of long-child
   references survive (K/J on Wikidata), so it will mispredict. The shipped forward pass's
   `f != q` test is comparable, so this is a wash at most.
2. **Store order.** Short factors are written in a descending scan instead of ascending. Copies run
   in increasing parent birth, and 36% read from later addresses. Cache effects are unknown.
3. **The grammar is destroyed.** Commands overwrite consumed rules. That is safe only while the
   writer is the grammar's last consumer, as the study notes; research replays would need a copy.
4. **Registers.** The two carried operands and the command cursor add register pressure.
5. **Memory ordering** is not an issue: the writer is single-threaded per block, in worker-private
   buffers.

**No time is claimed, and none follows from the ledger.**

## 5. Relevance (C)

**Eager, timed.** Recomputed from `docs/journal/monodromy/2026-11-10-cells-gap/evidence/i9-census-{1,2,3}.jsonl`
as Σ n·c_write / Σ n·c_whole:

| population | census 1 | census 2 | census 3 |
|---|---|---|---|
| **196 eager-admitted** (#579's flags) | **19.04%** | **18.92%** | **19.01%** |
| all 233 | 18.34% | 18.24% | 18.32% |

- *The figure is VERIFIED.*
- *The other source:* the container phase split (15 November, `2026-11-15-replay-and-team.md`)
  gives writer 29%, slots 21%, induction 50%.
- *A caveat:* the census kernel predates later changes. Its `k_rules` on the frozen block is 28,706,
  against today's 28,710.

**Lazy: no timed writer share is recorded.** The instruction shares on record:

| source | lazy | eager |
|---|---|---|
| Callgrind per letter, 15 November (`2026-11-15-lazy-letters.md`) | 15.1% / 15.7% (two builds) | 21.7% |
| #579's frozen Cachegrind profile (Ir) | 13.1% | 16.1% |

The lazy kernel's longer induction lowers the writer's share. So a given writer saving is worth
proportionally less there.

## 6. Side results (C, P)

**Duplicate short words.**
- *The total:* 12,126,015 repeats among 19,263,443 short words (record 66's count).
- *Equal spelling means an equal derivation tree:* **0 conflicts** across all short words. This
  confirms the two zero rows: the same tree everywhere, as the Lyndon-tree result says.
- *Row 1, 1,865,493, splits exactly* into #575's 650,507 and 1,214,986 TT products. Those are
  untouched refill, s > d, equal unmarked literal operands, and a short product: 0 mismatches per
  block.
- *Not re-derived:* rows 2–4. Their handle-based definitions are the study's. They sum to the total.

**Lookahead family.**
- *The arithmetic checks:* 7H − 4F = 5,569,687; the TT part 3,629,398; and the eager balance
  7H − 7,774,453 − 3 × 1,871,765 = −331,297. My T count is 7,774,453, and my old-literal T count is
  1,871,765.
- *The screen:* 6.80% of constructions, which gives 0.34–0.85% of kernel time.

**Boundary forwarding.**
- *The count:* C = TT + BB transitions = **9,699,621**; 32,273 on the frozen block.
- *The strict cuts:* V = 12,298,371, with B = 4,523,918. The balance 2C − V = 7,100,871.
- *The code identity holds (P):* the strict T writes `wr32(bb, fb, rd32(bb, bs))` and sets
  `bs = fb`, and the next T's `rd32(bb, bs)` reads that same field. Nothing in between writes it:
  - in eager Cells, nothing is written between the two cuts;
  - in lazy Cells, settling writes only the top link and the entry tag.

  BB is the mirror image. Forwarding is valid.
- *Its share:* 2.15% of the eager induction's data reads and 1.55% of the lazy induction's, on the
  frozen profile.

## 7. Theory (P, I)

**Ownership ends a Zorich streak (P).** By the consecutive-use theorem, a rule letter's references
are one run of strict wins on one side, closed by the loss or tie at its owner.
- *Exact consequence:* **K is the number of strict cuts whose surviving winner is an unmarked rule
  letter.** Counted: equal on 233 of 233 blocks, 2,528,876 in total.
- *These streaks* are exactly what Zorich acceleration would compress (record 55: Cells has no
  division step).
- *The deferred copies* are therefore the writer-side cost of the unaccelerated streaks.

**The owner tree is a coordinate tree (P, I).**
- *Offsets along the tree:* the child's offset is the owner's offset plus 0 or the left child's
  length. Being a tree, it has no cycle, so the offsets are consistent with no minimisation. This is
  the same argument as record 59's claim 2: the shared potential is consistent on a tree.
- *Copies sit on the non-tree edges:* K = J − U.
- *First-return reading (I):* as in records 57 and 59, every cut makes a band of a first-return map,
  a tower stacked on the induced interval. A letter's owner is the tower that absorbs its last
  piece. The owner tree records, for each tower, the band where its interval was finally used up.

**Gap chains: no identity (I).**
- *Different objects:* gap chains are adjacent-row LCP growth in rank order. The owner chooses an
  *occurrence* in text order.
- *The link is record 61's dichotomy:* rank-order constructions on one side, a text-order
  (positional) step on the other.
  - The shipped writer's leftmost-occurrence minimum is a text-order computation.
  - #579 shows it is unnecessary: a birth-order (rank-order) schedule suffices, and 36% of its copies
    read from later in the output.
- *So:* it removes one text-order element from Cells' writer. It does not touch the gap-chain
  statistics.

**What this decides:** nothing in production. The schedule is exact and the ledger favourable. Its
time effect is untested, and the eager writer's share bounds it at a few percent of the kernel.
