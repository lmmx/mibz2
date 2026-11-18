# 2026-11-18 · MONODROMY 69 · #582 (Cells ownership schedules) verified independently. Copy work is the same for every ownership tree, exactly. The depth recurrence is the exact minimum (brute force agrees on every enumerable case). The first-occurrence walk is exact on every block, periodic ones included, with no guard. The output-line fills reproduce to within 1 line. The comb's stack grows linearly, bounded by R − 1 and attained. The shared-cache and bookkeeping gains are model results, fragile in sign

**The request.** The Coordinator's CHECKPOINT 6988550050: an independent verification of Astra's study 6
on #582 at b55030a9, `docs/journal/2026-11-18-cells-owner-schedules.md`. Each claim is scored as exact,
supported, refuted or unmeasured.

**Labels:** **P** proved or checked by proof · **C** counted · **I** · **Conj**.

**Method.** I extended record 67's independent model (`mixedcheck.rs.txt` beside this record) with:
- *the reachable grammar of the marked root;*
- *four ownership trees:* last-use, first-occurrence, minimum-rounds and minimum-span;
- *the depth recurrence,* checked against a bottom-up readiness computation, and against brute force
  over all trees wherever there are at most 4,096 of them;
- *the first-occurrence walk itself,* with a cursor, tags and an explicit stack, checked on a fill
  bitmap and against LF inversion;
- *both two-phase schedules;*
- *an output-only cache model:* cold, fully associative LRU, 512 lines of 64 bytes, with write
  allocation.

## Verdicts

| # | claim | verdict | basis |
|---|---|---|---|
| 1 | Copy work is the same for every expansion-once tree; frozen 9,628 copies, 735,582 bytes | **EXACT.** "Last-use is optimal" is true but vacuous: every tree is | P, C |
| 2 | Exact minimum dependency depth; frozen 4 rounds against 5 | **EXACT** (the recurrence equals brute force on every enumerable case) | P, C |
| 3 | First-occurrence walk: 0 deferred commands, frozen stack at most 23, exact bytes and readiness | **EXACT** on all 233 blocks, not just 10; periodic blocks need no guard. **One wording correction** (§3) | P, C |
| 4 | Locality: 426,530 → 426,408 by owner, 180,974 by walk order | **EXACT** for 426,530 and 180,974. 426,408 is reproduced to within 1 line (426,407), depending on an unstated order of commands within one parent | C |
| 5 | Ledger: −17.96% fields (233 blocks); −5.70% shared-cache fills; 0.66–5.23% bookkeeping | **Fields EXACT.** The cache figures: model arithmetic verified, **UNMEASURED** on hardware. The bookkeeping range: **fragile in sign** | C, I |
| 6 | The counterexample family; bound the stack | **EXACT.** The stack is bounded by R − 1, and the comb attains it, so the bound is tight and linear. An in-place variant would need constant extra space (Conj) | P, C |

## 1. Copy work (P, C)

**Every tree gives the same copy work (P).** Each nonroot reachable rule v has exactly one selected
incoming edge. Its other d_v − 1 references are copies of |v| bytes. So:
- *copies:* K = E − R + 1, the same for every tree;
- *bytes:* Σ (d_v − 1)|v|, the same for every tree;
- *short writes:* 2R − E.

**Counted:**
- *Corpus:* the walk, last-use, first-occurrence and both minimum trees all give these values on all
  233 blocks. Frozen: K = 9,628, 735,582 bytes, R = 28,710 and P = 18,046.
- *Stress tests:* 14,209 small columns at thresholds 16, 2 and 1, and 16,000 repetitive texts at
  thresholds 1, 2, 4 and 16. The walk's copies and bytes equal K and Σ (d_v − 1)|v| every time.

**Last-use's extra optima:**
- *Source groups:* 1,309 on frozen, which equals the number of reused rules, so it is the minimum.
- *Owner before its deferred references:* this follows from record 67's consecutive-use theorem.

**In-place capacity extends to any tree (P).** I checked the proof.
- *The count:* in a suffix of t parents, t − 1 selected edges stay inside, so at most t + 1 slots
  remain.
- *Why at least one of those is not a command:* if any reachable rule lies outside the suffix, the
  selected path to it from the root leaves the suffix by a selected edge. If none does, the suffix's
  earliest-born rule has only terminal children.
- *Counted:* commands never exceed records read for the first-occurrence tree, on any block.

## 2. Minimum dependency depth (P, C)

**The recurrence.** dist(v) = max(a, b + w_v), with a parent attaining a selected, processed in
descending birth order.
- *It is monotone in every parent distance,* so induction gives the minimum for every vertex at once.
- *Checked against brute force:* 0 mismatches over 10,554 enumerable small columns at each threshold,
  and 3,995, 3,995, 3,626 and 2,574 enumerable repetitive texts at thresholds 16, 4, 2 and 1. Both
  rounds (w = 1) and span (w = |v|) were checked.
- *Checked against the independent bottom-up readiness computation:* consistent on every block.
- *How often last-use is above the minimum,* on the repetitive texts: 1,486 of 4,000 at threshold 1,
  but 1 of 4,000 at 16.

**The study's figures, reproduced:**

| | last-use | minimum |
|---|---|---|
| Frozen, rounds | 5 | **4** |
| Frozen, span | 4,786 bytes | **4,444 bytes** |
| Ten blocks, rounds | 4–6 | 3–4 |
| W16 witness 1⁴0⁶⁷ at primary 3 (2 copies, 35 bytes) | 2 | 1 |

On the ten blocks, first-occurrence takes 4–5 rounds.

**What it decides (I).** Rounds assume unlimited workers and atomic whole-source copies. No parallel
writer exists. The serial copy work is identical (§1), so the minimum-depth tree is a scheduling
property only.

## 3. The first-occurrence walk (P, C)

**Readiness (P).** A tag hit on a rule whose expansion is still incomplete would mean re-entering an
ancestor. Birth indices strictly decrease along a path, so that is a cycle, which cannot exist. So
every copy source is complete.

**Counted:**
- *Exact output, every source complete, no double writes:*
  - all **233** blocks (the study ran 10);
  - 14,209 small columns × 3 thresholds;
  - 16,000 repetitive texts.
- *Commands:* zero deferred, by construction.
- *Stack peaks:* equal to the study's on all ten blocks (23, 26, 24, 25, 23, 28, 20, 30, 21, 23).
  Frozen: 23.

**Periodic blocks: well defined, and no guard is needed.**
- *The order:* the walk reads the marked root from its grammatical left end. That root is the
  period-p marked closure word, defined for every block.
- *Unreachable rules* (the other LF cycles when p < n) are never visited.
- *Counted:* exact on all **12,192 short-period small cases** at every threshold.
- *Unlike last-use,* which needs record 67's full-period guard, the walk needs no guard.

**A wording correction.** "First occurrence in *cyclic output* order" should read "first in the root's
left-to-right spelling".
- *Why:* the physical output begins one byte after the mark. The walk's order is that order rotated to
  start at the root's left end.
- *Consequence:* a factor spelled after the mark lands near physical address 0, yet is *later* in the
  walk. The owner is the earliest occurrence in spelling, not the lowest address.
- *The study's 4,765* is a count of exactly this kind: last-use commands whose source is later in the
  root's spelling. Reproduced.

## 4. Output locality (C)

**The output-only line model, independent of any rule or arena layout** (ten blocks, every 24th):

| schedule | the study | mine |
|---|---|---|
| Last-use, study 5's two phases | 426,530 | **426,530** |
| First-occurrence owners, two phases | 426,408 | **426,407** (left command first within a parent); 426,341 (right first) |
| The first-occurrence walk | 180,974 | **180,974** |

- *What the study's own data say:* changing owners leaves output locality unchanged (−0.03%). Walking
  in output order cuts output fills 58%.
- *The one-line gap* in the middle row is the order of a parent's two deferred commands, which the
  study doesn't state. It is immaterial.

## 5. The ledger (C, I)

**Metadata fields.** The walk has 3R + K + 2P against last-use's 4R + 5K.
- *All 233 blocks:* 45,349,632 → 37,205,841, **−17.96%**.
- *Per block:* −8.05% to −20.94%. Frozen −19.10%; ten blocks −17.51%. **EXACT.**

**Are the visited marks and pending operands priced?** Yes. The tag store is R, the tag test on every
visit (repeats included) is R + K reads, and the pending pushes and pops are 2P. Stack growth is
charged separately (620 relocation fields, 60 allocation calls).

**The shared-cache model** (rules, arena, output, scratch). Recomputed from the study's
`first-visit.csv`:

| | last-use | walk |
|---|---|---|
| Fills | 716,908 | 676,064 (**−5.70%**) |
| Rule fills | 143,262 | 315,205 |
| Short-arena fills | 67,493 | 155,149 |
| Output fills | 442,244 | 205,320 |

- *Fills plus writebacks:* −8.72%.
- *Break-even:* the advantage vanishes at a premium of 0.157 fill units per extra rule or arena fill.
- *Status:* this arithmetic checks out. I did not rebuild the cache simulation itself, which depends
  on the layout. **UNMEASURED** on hardware.

**The bookkeeping range, 0.66–5.23%: fragile.** The study's README says that charging a second type
test on each pushed left operand *reverses* the price-1 advantage. So the sign rests on fusing that
test in code.

**The cache model's assumptions (I).**
- *Cold and fully associative.* The i9's L1D is 32 KiB 8-way, so conflicts are ignored.
- *No prefetch.* Sequential output is the prefetcher's best case, and the walk's scattered rule and
  arena reads are its worst.
- *L1 fills are not DRAM traffic.* The output (about 0.9 MB) overflows L2 and fits L3, so most model
  fills are L2 or L3 hits.
- *The decisive risk is latency.* The walk's extra reads are **dependent** loads: the next address
  comes from the descriptor just read. Last-use's reverse scan is sequential. A fill-count model cannot
  price that.

**Context, not a premise.** #581 (last-use) benched 4.8% faster at one thread. The walk would need its
own timing.

## 6. The counterexample and the stack bound (P, C)

**The comb.** L = 1ⁿ⁻¹0 at primary n − 2.
- *Counted for n = 4, 5, 8, 20, 50 and 200:* R = n − 1, K = 0, P = n − 2, stack peak n − 2.
- *The balance:* 4(n − 1) − (5n − 7) = 3 − n. At n = 4 that is 12 fields against 13. **EXACT.**

**The bound (P).**
- *The proof:* every pending descriptor belongs to a distinct ancestor still under first expansion,
  with a rule as its left child. Those ancestors lie on one root-to-node path of strictly decreasing
  birth. So **stack ≤ (grammar height − 1) ≤ R − 1,** which is at most 8(R − 1) bytes.
- *Tightness:* the comb attains R − 1. So the worst case is Θ(R): linear, not small.
- *Observed:* repetitive random texts with n ≤ 200 reached 177.
- *On Wikidata* it is 20–30. That is an empirical property, not a guarantee.

**A way to avoid it (Conj).**
- *The right descriptor is already stored:* the tag overwrites only the first field of a spent
  record, so the second field still holds the pending right descriptor. The stack really only needs
  *return links.*
- *Where a link could go:* the tag's zeroed low half. Lengths are below 2²², and MARK is bit 31, so
  bit 30 is free to mark a tag, and a 30-bit rule index fits beside it.
- *Why it is safe:* a tag hit reads only the high half (the offset), and incomplete ancestors are
  never hit (§3).
- *The result:* the stack lives in place, in constant extra space. The cost is one extra field read
  per pop, to fetch the parent's right descriptor.
- *Status:* a sketch. Not implemented.

**What this decides:** nothing in production. Each of the three candidate writers has an exact,
verified correctness argument:
- last-use (#581), with a full-period guard;
- the first-occurrence walk, with no guard;
- the minimum-depth tree, for any future parallel writer.

The choice between last-use and the walk is a timing question the models cannot settle.

**Reading (I).** The walk brings back a *text-order* traversal: grammar decompression with memoised
first occurrences. That is record 61's dichotomy again. It trades the sequential (rank-order,
birth-order) rule scan for sequential output, and the cost moves to dependent rule reads.
