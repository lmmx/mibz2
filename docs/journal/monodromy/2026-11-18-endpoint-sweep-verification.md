# 2026-11-18 · MONODROMY 70 · #591 (the endpoint sweep) verified independently. The first-arrival theorem holds at every cut. Rebuilding from arrivals alone reproduces every cut, closure and grammar pair on all 233 Wikidata blocks (the study ran 10), the six prose controls and 14,209 small columns. The structural counts are exact, and so is the ledger's arithmetic: both schedules cost more than shipped Cells on every admitted sample block. The algebra holds no smaller fact that Cells' ordered fronts could use

**The request.** The Coordinator's CHECKPOINT 6990514124: an independent verification of Astra's study 7 on
#591 at 6d1646b0, `docs/journal/2026-11-18-cells-endpoint-sweep.md`. Each claim is scored exact,
supported, refuted or unmeasured.

**Labels:** **P** · **C** · **I** · **Conj**.

**Method.** I extended record 69's model (`mixedcheck.rs.txt` beside this record) in two ways.
1. **Identities checked inside the scalar induction.** At every cut I took the absolute left ends,
   which stay put as the induction runs (record 66), and checked four things:
   - the frontier is the same on both sides;
   - a = −d_A > 0 and b = d_B > 0;
   - the child's right ends are t = F − b and b = F − a.
2. **My own sweep, written from the theorem alone.** Its only inputs are the initial letters (runs
   with the marked row split out) and their right ends: the cumulative row count on top, and the
   rows of lower bytes plus a per-byte cursor on the bottom.
   - *The mechanism:* a max-heap of (arrival key, side, band). There are no widths and no links. A
     side keeps its operand until an arrival replaces it.
   - *What is compared:* every cut (A, B, width, with direction implied) and every closure, with
     band identities, against the scalar induction. Children get ids in birth order, so equal
     sequences mean an identical ordered grammar.

## Verdicts

| # | claim | verdict | basis |
|---|---|---|---|
| 1 | The invariant: d = b − t, the child's displacement b − a, first arrival F − min(a, b) | **EXACT** | P (checked), C: 0 failures at 30.8M cuts |
| 2 | The reconstruction: every cut, closure and the word grammar, from arrivals alone | **EXACT.** All **233** blocks (the study ran 10), 6 prose controls, 14,209 small columns | C |
| 3 | The ordering cost: 91,524 frozen newborns outside the current 64-row group; ordering needed | **EXACT** (98.95% over ten blocks); the witnesses reproduce | C |
| 4 | The ledger: bytes, resident units; both schedules worse than eager and lazy | **Bytes and population ratios EXACT** from the study's table, and the conclusion holds **per block**. Resident units are the study's model: **UNMEASURED** in time | C, I |
| Q | A smaller exact fact Cells could use without replacing its ordered fronts? | **No** (§5) | I, from the records |

## 1. The theorem (P, C)

**The proof holds.** I checked it step by step.
- *Persistence:* a cut relabels only the rightmost w rows at the two opposite sites, and closures
  delete a common suffix. So untouched bands keep both right ends.
- *The child's position:* A is the top front and B the bottom front. The child sits at the suffixes
  of B's top site, ending at F − b, and A's bottom site, ending at F − a. That gives
  d_child = b − a = d_A + d_B.
- *The child's first arrival:* h = max(t, b) = F − min(a, b), with its side given by the sign of d.
- *The next frontier is the largest pending key.* Every band that has been a front stays there until
  it is exhausted. Fresh fronts are bands never yet touched. Untouched bands keep their keys.

**Equality.**
- *All three width comparisons use the same formula,* and equal widths do not fix the side.
- *Witnesses reproduce:*
  - `100` at primary 1 and `110` at primary 0 both cut at widths (1, 1), but their first newborn goes
    bottom in one and top in the other;
  - `1100` at primary 0 makes a diagonal from a strict cut.

**Closures are diagonal arrivals** (d = 0). The two sides then share a band.
- *`00`:* primary 0 discards a closure before the marked one; primary 1 doesn't.
- *Small columns:* 12,192 short-period cases, discarded closures included, all exact.

**The marked row.** It is split into a singleton of width 1. It ends the sweep at its own diagonal
arrival.

**Counted:**
- *0 identity or sign failures* over 27,439,756 Wikidata cuts, 3,369,224 prose cuts and every small
  column.
- *At most one event per side at a coordinate,* and a diagonal is always alone (0 violations).

## 2. The reconstruction (C)

**My heap sweep matched the scalar induction exactly, event for event:**
- all **233** large Wikidata blocks (27,439,756 cuts);
- the 6 prose controls;
- 14,209 small columns: binary up to length 9 and ternary up to length 6, every mark.

That covers every cut's operands, width and direction, and every closure.

**The grammar, not only the cuts, is reproduced.**
- *Why identical pairs settle it:* children are numbered in birth order, so identical (A, B)
  sequences mean an identical ordered grammar.
- *What follows:* W16's short-or-rule choice depends only on the grammar. So the 28,710 rule pairs and
  the arena allocation follow (the study's 322,317 arena bytes are implied, not separately recounted).

**Frozen.** 92,570 cuts and 1 closure, the marked one.
- *Newborn first arrivals:* 44,880 top, 47,689 bottom and 1 diagonal, exactly as stated.
- *Ten blocks:* 1,236,063 visited coordinates, the study's figure.

**Census rows recounted:**

| row | frozen | ten blocks |
|---|---|---|
| Both residual widths 1 | 41,602 | 583,714 |
| Smaller width 1 | 70,863 | 977,160 |
| Untouched originals at an equal cut | 11,125 | 149,610 |
| Strict cuts | 41,387 | — |
| Same-direction pairs | 32,273 | — |

All exact; the strict-cut and same-direction counts are from record 67.

## 3. The ordering cost (C)

**Out-of-group newborns.** A newborn's key falls outside the current 64-row group (group of key − 1
≠ group of F − 1) **91,524** times out of 92,570 on frozen. Over ten blocks it is **98.95%**. Both
exact.

**Arrivals are almost never local.** That is why any frontier queue pays an event per cut.

**Posting order is not arrival order.** The witnesses reproduce:
- `1100100` at primary 0 makes 2 top births and 3 bottom;
- `1000` at primary 0 makes 2 bottom births.

The study's 3-then-4 and 3-then-2 orders follow from its coordinates, and my sweep's exact match
confirms them.

## 4. The ledger (C, I)

**Memory, by the stated formulas.** The arithmetic is exact.
- *Grouped:* 16r + 4⌈n/64⌉ + 528 = 875,720 bytes.
- *Direct:* 24n + 16⌈n/64⌉ = 21,823,336 bytes.
- *Shipped eager:* 1,433,152 bytes, 28.0 per letter.

**The population conclusion, recomputed from `prices.csv`** (ten sample blocks; setup plus induction
resident units):

| population | grouped | direct |
|---|---|---|
| **Eager-admitted (8)** | **+77.8%** (every block +76.9% to +78.4%) | **+33.4%** (+32.7% to +34.0%) |
| **Lazy-admitted (9)** | **+58.5%** (+55.8% to +60.0%) | **+18.9%** (+16.3% to +20.2%) |

- *It holds on both populations, block by block.* Every admitted block is worse under both schedules.
- *Line transfers:*
  - grouped saves 783,027 (eager) and 165,926 (lazy) 256 KiB transfers, so a transfer would have to
    outweigh 37 or 188 units to break even;
  - **direct adds transfers** in both populations.
- *The induction's share:* 58.5–58.6% of eager kernel time over the 196 eager-admitted blocks,
  recomputed from the three i9 censuses.

**Assumptions (I).**
- *Unit prices:* resident units are scalar operations plus field accesses at unit price, and line
  fills are counted separately in an ideal 256 KiB cache.
- *What the model leaves out:* latency, memory-level parallelism, prefetch and the i9's real
  hierarchy.
- *Why that matters:* shipped Cells is bound by dependent slot loads (records 47–53). The sweep
  swaps link-chasing for group-chain reads, which are also dependent.
- *The sample:* ten blocks, 8 and 9 admitted. The per-block uniformity of the sign (all within ±2
  points) makes the conclusion robust within the model.
- **The conclusion that neither schedule clears the screen is supported. Its time effect is
  UNMEASURED,** and the study says so.

## 5. Is there a smaller exact fact Cells could use? No

These are candidates the arrival algebra contains. Each is tested against what Cells' ordered fronts
already know.

1. **The displacement d of each band,** and its additivity. Cells keeps no coordinates.
   - *What it would cost:* one field per band and a register frontier.
   - *What it would buy:* the child's arrival key F − min(a, b) and its side, both at birth.
   - *No consumer:* the side doesn't let Cells skip either of the child's two records. The top slot
     carries the payload; the other copy is the *placement site* for the child's own future
     children, so both are needed.
2. **The arrival key as a prefetch hint.** Cells' linked orders already deliver each side's next
   front in arrival order, with one dependent read, prefetched `FAR` slots ahead (`ahead!`,
   `ahead_bottom!`). The key could only duplicate that.
   - *Why there is no shortcut:* 98.95% of newborns arrive outside the current 64-row group. Arrival
     keys are non-local, so the link is the cheapest certificate of "next front".
3. **Diagonal arrivals (d = 0) as closure certificates.** These would be closures known at birth.
   - *Not worth it:* every full-period Wikidata block has exactly one closure, the marked one. Cells
     detects a closure with one slot comparison (`bs == fb`).
4. **Width-free cuts.** Widths are frontier differences. But 77% of frozen cuts have a smaller width
   of 1, and Cells' width test is a register comparison. The sweep removes the *width reads*, not
   comparisons that cost anything.
5. **Equal cuts of untouched originals** (11,125 frozen, 12% of cuts). This is the case of the
   historical lazy direct path, which *raised* instructions per letter (`2026-11-16-lazy-overhead.md`,
   181.0 → 191.1).

**So:** every exact fact in the algebra beyond Cells' ordered fronts is a *coordinate* fact. Using one
either needs an ordering structure, which is the cost the study measures, or adds a field with no
consumer.

**What the algebra shows instead (I).** Cells' linked fronts are an implicit priority queue on arrival
keys, with each pop costing O(1). That is why they beat both explicit schedules here. **That is the
answer from the records: no smaller usable fact.**

**What this decides:** nothing in production. It records a second exact formulation of the induction:
coordinates and displacements in place of widths and links.
