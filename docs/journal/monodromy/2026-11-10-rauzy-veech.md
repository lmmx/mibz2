# 2026-11-10 · MONODROMY 1 · LF's interval exchange, renormalised at its endpoint: a block's run records become 1.81 r binary rules and then its bytes in 0.82 to 0.86 of production's inversion time (Wikidata, one thread, indicative)

Labels: **P** proved here · **C** counted (exact for its input) · **M** measured (this container,
one thread, indicative, RULES C5) · **Conj** conjecture · **K** known, cited. "PF:" is a path on
the Pathfinder's branch (#313). `examples/rauzy.rs` rebuilds every block named below from the
grammar and asserts its bytes against `bwt::walk`.

## 0. Result

- **Attacked:** "inversion is a traversal of π = LF (or Ψ)". Row walk, k-step cells, the tower
  (PF:`2026-11-10-interval-exchange.md`): all push points or intervals *through* π.
- **Replaced by:** renormalisation of π itself. π = exchange of r intervals (the runs of L); its
  data are two orders of r letters and r lengths. Rauzy–Veech step at the right endpoint:
  compare two lengths, subtract, relink one letter, emit one binary rule. Nothing is evaluated
  under π. Run to one point: a straight-line program (SLP) for the block.
- **C, Wikidata, 233 blocks, 209,697,662 rows:** S = 27,439,723 rules = **1.812 r = 0.1309 n**.
  Per block S/r: 1.29 / 1.73 / 1.84 / 1.91 / 2.08 (min, p10, median, p90, max).
- **C:** the block's CRC from the grammar alone, 0.262 multiplications mod P a row, equal to the
  bytes' CRC on all 233 blocks.
- **P + C:** S in closed form, with no induction: cut LF's cycle after every run start; S =
  Σ over the r stretches of (minima-so-far read forward + read backward − 1) − 1. Equal to the
  induction's count, type by type, on every block (§5.5). The 1.8 is a record statistic.
- **C:** the induction is not one chain. Its 15,141,369 segments order into **93 rounds** (median;
  63 to 337) of independent two-pointer merges; 63.5% of the runs are sent exactly onto one
  run.
- **P + K + C, what the grammar is (§5.6):** the Lyndon tree of the block's least rotation
  (Hohlweg–Reutenauer 2003: the Cartesian tree of its suffix ranks). Every letter's word, at
  every moment, is a Lyndon word. With equal rules shared it is the Lyndon SLP: 0.470 S =
  0.852 r rules.
- **P + C, what a step is (§5.6):** one digram of the text written in run letters, replaced
  at all its occurrences at once: they are one interval of rows. For every order of steps,
  steps = r₂ − 1 + (re-seatings that land on no boundary). The right end has 0.592 r of those;
  "steps that land on a boundary first, then the largest cell" has 0.059 r: **1.279 r steps,
  1.048 of the floor**. Counted, each run checked to end on the block; no kernel takes its
  steps in that order yet.
- **C:** half the steps lie in the first 4 of those rounds. What limits workers is not the
  93 rounds but the chain inside a long merge: taken a step at a time the steps are 3,861
  deep (median), and P workers need between max(depth, S/P) and S/P + depth turns: S over
  the turns is 1.93 to 2.00, 3.6 to 4.0, 6.5 to 8.0, 10.6 to 16.0 at P = 2, 4, 8, 16, pooled
  (§5.4). Another kind of round (independent sets of segments consumed whole: 24 a block,
  1.85 to 2.06 r cells, so not the same steps) was built by a sweep as passes over arrays on
  one thread and lost: order and induction alone 1.99× the column's path (**M**; its program
  is not in the repository); closed as built (§5.4).
- **M, the clock** (all 233 blocks, pooled, one thread, this container, indicative):
  - linked orders, a step that re-seats a letter (§3.1): **1.276×** the column's path
    (`bwt::build` + the room walk), which was production then. It lost (§6.1).
  - **cells** (§3.5: a step's cell is a letter written once; nothing is moved; no point is
    followed), against the column's path, before the writer's fix: 0.756× and 0.738× (§6.2).
  - **cells against production as it now is** (the records' path, 0.2.8; both from the same
    run records; after the fix): order 0.367 + induce 0.937 + write 0.347, 1.677 as one span,
    against 2.049: **0.82×** (0.818, 0.820, 0.817 in three runs made within four minutes).
    **0.84 to 0.86×** in nine runs 34 to 48 minutes later, the machine in a slower state,
    cause unknown (§6.3). Faster on 185 to 197 of the 233 blocks. Chosen by r/n < 0.10, a
    threshold fitted on these same blocks: 0.76 to 0.79× (§6.3). The induction's cost
    follows S/n, not r/n: no rule for other data (§8, 4).
  - Per run of L (C: callgrind, one block each, with callees; the kernel before the fix, fed
    from the run list, against the column's path; misses of a simulated 2 MiB cache, this
    machine's L2). r/n 0.057 (block 1): 186 instructions against 268, 2.1 misses against
    9.1. r/n 0.072 (block 22, the corpus's ratio): 180 against 217, 2.3 against 7.9. r/n
    0.130 (block 23): 164 against 129, 2.6 against 4.0. Production's work is by the row:
    counted by the run it falls with n/r (§6.2).
- **Where else it wins (M, cells ÷ production, §6.3; one run an input):** Fibonacci and
  Thue–Morse 0.02 to 0.03×; period 37 0.31×; JSON of 300 ids (r/n 0.022) 0.47×. At
  break-even: JSON of 3,000 ids (r/n 0.057, S/n 0.147) 0.85×, inside the run-to-run spread.
- **Where it must not be used:** JSON of 20k records (r/n 0.151) 1.7×; prose (0.367) 4.0×;
  uniform bytes 8.9×; `(ab)^k a` 11× (S = n/2, §2: needs the division rule or a step budget).
- **The record's own errors, found by a review of it (§11):** the cells' writer left bytes
  unwritten and its check could not see it (§7 row 1); a citation chain and four theorem
  numbers (§10); the core's clock (§6); three sentences of §2 and §5. By a referee's reading
  (23 findings): a count said to stand at its floor, a step called branch-free that keeps a
  branch, instruction counts quoted from the most favourable block, an admission threshold
  fitted and scored on the same blocks. By a last check of those corrections: the headline
  0.82 did not hold half an hour later (0.82 to 0.86, §6.3), and "S/P turns" was a ceiling,
  not a schedule (§5.4).
- **Status:** the right end's count is fixed by the block (§5.5: 1.812 r) and is not the
  floor: the floor is r₂ − 1 = 1.220 r and another order of steps has 1.279 r (§5.6), so at
  most 1.49× is left in the count; no kernel takes it (§8, 2). The step went from 14 ns to
  7.7: 64 instructions, a three-way branch, two slots and one word written, the scattered
  writes fetched ahead. One container, one thread, indicative
  (RULES C5): the i9, its ten workers and a decoder that calls the kernel are not measured
  (§8).

## 1. Objects

- L ∈ Σⁿ, rows [0, n), runs ρ₁ … ρ_r (maximal equal-byte blocks), λᵢ = |ρᵢ|, bᵢ its byte.
- π = LF: π(x) = C(L[x]) + #{y < x : L[y] = L[x]}. **K** (GNP 2020 Lemma 5.2; Nishimoto–Tabei's
  disjoint interval sequence; Ferenczi–Zamboni 2013 Lemma 3 for clustering words): π is a translation on
  each ρᵢ. So π is a discrete interval exchange (IET) on r letters:
  - TOP order: ρ₁ … ρ_r as they lie in L (the domain's tiling);
  - BOTTOM order: the runs sorted stably by byte (the tiling by images π(ρᵢ)).
- Emission: T[k] = L[π^{−(k+1)}(orig_ptr)], k = 0 … n − 1 (`bwt::walk` steps π⁻¹ from
  orig_ptr). For a BWT image this is x₀ = orig_ptr, x_{k+1} = π(x_k), L[x_k] = T[n − 1 − k]. For
  a column that is no BWT the first form is the definition: n bytes round the cycle of
  orig_ptr, beginning after it; the second holds only when the cycle's length divides n.
- 𝓣 = {run starts}, 𝓑 = π(𝓣) = {image starts}; |𝓣| = |𝓑| = r, 0 ∈ 𝓣 ∩ 𝓑.

## 2. Proposition (P)

**State.** Letters 𝒜, orders <_top, <_bot, lengths λ: 𝒜 → ℕ₊, words w: 𝒜 → Σ⁺ (text order),
m = Σλ. I_α, J_α: α's intervals in the top and bottom tilings of [0, m).

**Invariant I.** π_m, the first-return map of π to [0, m), is x ∈ I_α ↦ J_α by translation; every
x ∈ I_α returns after |w_α| steps, and L[x], L[πx], …, L[π^{|w_α|−1}x] is w_α reversed.

**Step.** A = max_top, B = max_bot, A ≠ B. N a fresh symbol, N → w_A w_B.

| case | lengths | word | order |
|---|---|---|---|
| top wins, λ_A > λ_B | λ_A −= λ_B | w_B := N | bottom: B moves to just after A |
| bottom wins, λ_A < λ_B | λ_B −= λ_A | w_A := N | top: A moves to just after B |
| equal | — | w_B := N | A deleted; bottom: B takes A's place |

**Claim.** I holds after the step, with m' = m − min(λ_A, λ_B).

*Proof, top wins.* J_B = [m − λ_B, m) ⊂ I_A = [m − λ_A, m). For x ∈ I_B: π_m x ∈ J_B, outside
[0, m'); π_m² x ∈ J_A. J_A ∩ J_B = ∅ and J_B = [m', m), so J_A ⊂ [0, m'): the return to [0, m') is
π_m², after |w_B| + |w_A| steps, reading w_B reversed then w_A reversed = (w_A w_B) reversed. For
x ∈ I_α, α ≠ B: π_m x ∈ J_α ⊂ [0, m') (as J_α ∩ J_B = ∅): unchanged. I_A keeps its first λ_A − λ_B
points; their images are the first λ_A − λ_B points of J_A; the rest of J_A is now B's image:
B sits just after A on the bottom. ∎
*Bottom wins.* I_A = [m − λ_A, m) ⊂ J_B. The last λ_A points of I_B go to I_A, then by A's
translation to J_A (disjoint from J_B ⊃ [m', m), so inside [0, m')): return word w_B then w_A
reversed, image J_A. Name that part A: it lies just after B's remaining part on top, and keeps
A's bottom place. ∎ *Equal:* both at once; I_A = J_B is cut off whole. ∎

**Termination (P).** m strictly decreases. Stop at A = B: I_A = J_A, π_m fixes it pointwise, each
point's π-cycle has length |w_A|. Kac: Σ λ_α |w_α| = (points on cycles meeting [0, m)). π one
n-cycle ⇒ one letter, λ = 1, |w| = n, m = 1: the root's expansion is row 0's cycle.
T = v^k (bzip2 sorts rotations, k cycles): the same steps as for v with every length × k, ending
at one letter with λ = k, |w| = n/k (one step for `(ab)^k` itself). `(ab)^k a`, and the blocks
bzip2 makes of `(aaaab)^k` and of zeros (after its RLE1 a period, k' times, and one to five
bytes more; the zeros' block is 19,610 rows, so run with MIN_ROWS 1): bytes exact (C: the
example's assert; §4 lists the first).
**A column that is no BWT** (a decoder is handed any bytes and any row; a corrupt stream):
LF has cycles of several kinds, and A = B occurs with other letters left. The last letter is
then λ_A cycles spelling w_A: if the walk is on it, stop; else drop it from both orders and go
on. **C:** `rauzy --arbitrary`: 200,000 random columns (178,276 with several cycles, a letter
dropped in 103,731), reference and kernel equal to `bwt::walk` on all; 3,000 under valgrind,
clean.

**Counts (P).**
- Equal steps = r − 1 for one cycle: only they delete letters. **C:** 15,141,369 = 15,141,602 − 233.
- r − 1 ≤ S ≤ n − 1 for a BWT image: a step deletes at most one letter, and removes at least one
  point. (With several kinds of cycle, dropped letters take the place of equal steps.)
- S is not O(r polylog) as written: `(ab)^k a` has r = 2, S = n/2 (**C**: 449,991 of
  n = 899,981). The same two letters stay last: q steps = one division, one rule w^q. See §7 #3 and §8, 5.

**The tracked point (P: the Claim's proof followed for the one point's letter; C, §11).**
orig_ptr as (α, o, v): orig_ptr = π^v(start(I_α) + o), 0 ≤ v < |w_α|.
With λ_A, λ_B and |w_B| their values before the step:
- Top wins: if α = A and o ≥ λ_A − λ_B: α := B, o −= λ_A − λ_B, v += |w_B|.
- Bottom wins: if α = A: v += |w_B|. If α = B and o ≥ λ_B − λ_A: α := A, o −= λ_B − λ_A.
- Equal: if α = A: α := B, v += |w_B|.
At the end t₀ = v, and T[k] = E[(k − t₀) mod p], E the root's expansion, p its length.

**No point need be followed (P; C on 200,000 arbitrary columns and every block).** Cut the run
of orig_ptr so that the row is a letter Z of its own (at most two letters more), and mark
its word; a word made with a marked word is marked.
- A letter whose word is marked has one point: all its points share one itinerary through
  the original letters, and that itinerary passes Z, which is one row.
- So the walk's cycle is the closed letter whose word is marked, and Z lies once in
  that word, at a place j read off the marked rules from the root down.
- By Invariant I a letter's word ends with the byte of its own row, and the decoder's first
  byte is that of π⁻¹(orig_ptr), the byte after Z's. So **T is the cycle's word read from
  place j + 1 round**, n bytes (ending with L[orig_ptr] when the cycle's length divides n,
  as in every BWT image).
- A block that is a power v^k: its k tied rows lie in one run. Those above orig_ptr close
  first, as one letter with all their points, and are dropped; the marked letter, one point,
  closes next and ends the induction; those below are a letter left unclosed.
The induction then carries no (α, o, v), and the writer starts at j + 1: nothing is turned
afterwards (§3.5).

## 3. Construction

**3.1 Induction.** Two doubly linked orders over r nodes (λ, |w|, w, prev₂, next₂), 32 bytes a
letter. A step: 2 lengths read, 1 rule, 5 or 6 links written. No rank, no select, no run map, no
n-entry table, no column: the input is (byte, length) per run, which `runs::front` emits (#324; on master since 0.2.8).

**3.2 Grammar → bytes, each byte written once (P).** pos(j) = where rule j's expansion first lies
in E. A rule's children were made before it, so:
- pass 1, rules last to first: pos(root) = 0; pos(left) = min(·, pos(j)); pos(right) = min(·,
  pos(j) + |left|). Correct because the leftmost occurrence of a child lies in the leftmost
  occurrence of some parent.
- pass 2, rules first to last: a child whose pos is not its place inside j is copied from pos
  (already written: it was made earlier); else nothing.

Two sequential passes over S rules, no recursion. Copies never overlap (distinct occurrences
of one symbol are disjoint). **C:** 0.0409 copies a row carry 91.0% of the bytes, mean 22.2 B.
The record's first-visit copy walk counts a similar object (PF:`2026-11-09-phrase-structure.md`,
correction §1: 0.0328 copies, 0.928 of bytes, on its sample); here it is reached without
walking.

**3.3 Small words as bytes.** 70.2% of rules are ≤ 16 bytes (**C**: shares 0.211, 0.198, 0.164,
0.129 at ≤ 2, 4, 8, 16) and hold 0.467 n bytes. The kernel keeps a word of ≤ 16 bytes in an
arena (two 16-byte moves a step) and makes a rule only above: 0.0390 long rules a row, 0.54 a
run.

**3.4 Homomorphisms in O(S) (P; K as a technique: fingerprints composed over SLP rules,
Bille et al. 2013, Lemma 1; counts over an SLP, Erickson–Nayyeri 2013 §5.1).** For any
monoid morphism h: Σ* → M, h(N) = h(left) h(right): S products, not n. CRC-32/BZIP2:
R(w) = w(x)·x³² mod P, X(w) = x^{8|w|} mod P, R(uv) = R(u)X(v) + R(v), X(uv) = X(u)X(v);
crc = ¬(R + ones·X). The rotation costs one descent from the root. **C:** 0.262 multiplications a
row; equal to `crc::crc32` of the walked bytes on all 233 blocks and every synthetic input
that is no power (for a power the example skips the check).
bzip2's stored CRC is over the RLE1-decoded bytes: the RLE1 decoder composes over rules too
(§8, 6), checked in Python beside this entry, not in the example. A CRC taken from the
grammar verifies the input; it no longer checks the passes that write the bytes.

**3.5 The cells form (P for its steps being §2's; C: bytes on every input of §7).** The
second construction. What §3.1 moves, this writes once.
- **State.** Two arrays of 24-byte slots, one an order, each in the order the induction
  reads it (last letter first): (the slot before, λ, |w|, w, the letter's slot in the other
  order). In registers: A and B, what is left of each (ra, rb), their words, the slot before
  each, and for each the slot its next cell takes in the other order.
- **A step is a cell:** the last min(ra, rb) points of both, word w_A w_B. It is §2's loser
  under its new name (top wins: B; bottom wins: the part named A; equal: B), and it is never
  changed again. On top it lies in B's place, on the bottom in A's image, each time to the
  left of the cells already cut from that letter.
- **Where it is written.** The first cell cut from a letter takes the letter's own slot in the
  other order: whatever pointed at the letter now points at its rightmost cell, and nothing
  else is touched. If the letter goes on, its next cell is promised a fresh slot, and the
  letter's "slot before" moves into the promised slot: the last cell cut finds it there. So
  a cell writes two slots, three stores each (λ; |w| and w; the slot in the other order), and
  when a letter goes on (45% of cells) two links more: that of the cell's slot on that
  letter's side, and the promised slot's. It patches nothing. The winner's remainder is in
  registers only.
- **A cell is read at most once, on the side the front reaches first,** as A or as B (a
  letter that closes is met by both fronts: that is the closing test); on the other side its
  slot is by then its own first cell.
- **Closing.** A and B are one letter exactly when the slot A's next cell would take is the
  slot B was read from. It can happen only when both were just read whole (at the start,
  after an equal cell, after a drop): after an unequal cell the loser lies next to the
  winner's remainder (**P**). So the test sits where both are read whole, and nowhere else.
  A marked word ends the induction; otherwise both fronts take their slot before and
  nothing is written (§2's drop). A cell is written before the next letter is read: the
  next slot may be that cell.
- **The three cases are a three-way branch** on ra against rb (equal 55%, top 28%, bottom
  16.5% of cells on Wikidata); the word is made before it, as bytes and as a rule, one of them kept.
- **One buffer,** every reference a 32-bit offset in it: slots, the words kept as bytes, the
  rules. A rule is its two factors as their slots held them, (|w|, w) twice: two stores.
- **Fetch ahead.** The slot 12 before the last in an order holds, as memory has it now, the
  place its letter's cells will be written and its word: both are fetched then. This takes
  about a quarter off a cell's time (M, §6.2: 9.8 → 7.2 to 7.7 ns on 10 blocks, one run each,
  so good to the clock's 13%); a rise with r remains.
- **No point is followed** (§2): the mark rides in |w|'s top bit, so a marked word made by a
  cell is never kept as bytes (only Z's own byte is) and the marked rules are a path from
  the root to Z.
- **The write** walks that path three times: for the row's place j; to write each short
  unmarked factor where it lies from j + 1 on and note where each long one lies; and, after
  §3.2's two passes into the output itself, to copy each long factor that lies again later
  from its first place (a marked rule has no place of its own, so the two passes never copy
  for it: the first build of this writer left those bytes unwritten, §7 row 1).
- **Not done:** the division rule (a streak of one winner is one cell a turn here too); a
  bound on the buffer other than n cells; the decoder's own records (§8, 3: the example
  makes them from the column).

## 4. Counted (C)

`rauzy FILE 1 65536 5`: every block ≥ 65,536 rows.

| input | blocks | r/n | S/r | S/n | coincidences/r | rounds (median) |
|---|--:|--:|--:|--:|--:|--:|
| `wikidata.bz2` | 233 | 0.0722 | 1.812 | 0.1309 | 0.780 | 93 |
| prose (the repository's `.md`, 3.5 MB, `bzip2 -9`; not the bench's `text`) | 4 | 0.3672 | 1.841 | 0.6761 | 0.541 | 98.5 |

Synthetic (blocks ≥ 10⁵ rows):

| input | n | r | S | S/n | note |
|---|--:|--:|--:|--:|---|
| Fibonacci word | 899,982 | 16 | 175 | 0.0002 | |
| Thue–Morse 2²⁰ (2 blocks) | 1,048,576 | 102 | 875 | 0.0008 | |
| period 37, and one byte | 888,001 | 37 | 24,044 | 0.027 | 23,998 steps are one division |
| `(ab)^k a` | 899,981 | 2 | 449,991 | 0.500 | one division |
| `(abac)^k x`, `(abcabd)^k x` | 899,981 | 4, 6 | n/4, n/6 | | one division each |
| JSON-like, 300 ids (2 blocks) | 1,560,097 | 34,903 | 115,983 | 0.074 | S/r 3.32 |
| JSON-like, 3,000 ids (2 blocks) | 1,584,100 | 89,712 | 233,383 | 0.147 | S/r 2.60 |
| JSON-like, 20k records (4 blocks) | 2,850,814 | 430,046 | 751,520 | 0.264 | S/r 1.75 |
| 4 letters, uniform | 899,981 | 652,500 | 875,635 | 0.973 | S/r 1.34 |
| bytes, uniform | 899,981 | 896,440 | 899,976 | 1.000 | S/r 1.00 |

- **S/r is 1.8 on both natural corpora, five-fold apart in r/n.** Not a law: 1.0 (random) to
  3.3 (few-id JSON) to ~10 (Sturmian-like, where S is tiny anyway). §5.5 says what it is.
- **Floor (P, §5.6):** for any order of these steps, steps ≥ r₂ − 1 = |𝓣 ∪ 𝓑| − 1, 1.220 r on
  Wikidata (prose 1.459 r): at most 1.49× is left in the count. S = (r₂ − 1) + S₀, S₀ = 0.592 r
  the boundaries that re-seating makes at the right end. Another order of steps makes
  0.059 r. (This entry first gave r − 1, and "1.81×".)
- Against the record: the tower 8 × 3 is 0.190 n dependent steps on its 20 Wikidata blocks
  (every 12th) and stops at W = n/512 with words materialised at each level (1.58 n bytes
  at 8 × 3); on those same 20 blocks this is 0.1288 n (1.80 a run against 2.65), stops at one
  point, and materialises 0.47 n bytes of small words. The units differ: a
  compare-subtract-relink here, a piece pushed through the run map there. PF:`2026-11-10-chunked-link.md` §5a's bound
  (2r a halving, 0.55 n, "pays only below r/n ≈ 0.03") is 4× above the count.

## 5. Structure: coincidences, segments, rounds

**5.1 (P) |𝓣 ∩ 𝓑| = 2r − r₂**, r₂ = the pieces of π² (maximal intervals on which π² is a
translation with one 2-byte word). The cells of the common refinement of the top and bottom
tilings number |𝓣 ∪ 𝓑|; each cell I_α ∩ J_β pulls back to one piece of π² inside I_β, and two
neighbouring pull-backs differ in a byte. Checked by brute force on 4,897 words (all binary
words to length 8, ternary to 6, random, near-periodic). **C:** 0.780 r on Wikidata, 0.541 r on
prose; the record's r₂/r = 1.22 (run-fed census) is the same number. Read as dynamics: 78% of
LF's discontinuities land on discontinuities after one step.

**5.2 (P) Segments.** Cut [0, m) at 𝓣 ∩ 𝓑. A segment is at once a union of top intervals and a
union of bottom intervals. The endpoint induction consumes the last segment as a two-pointer
merge of its top lengths against its bottom lengths: one step a cell, the equal step exactly
at the segment's end. So
**S = Σ over the r − 1 segments consumed of (cells in it when consumed)**, for a segment that
meets no letter it re-seated itself (§5.4: all but 84 on Wikidata; such a segment grows while
it is consumed).

**Shapes at the start** (**C;** Wikidata, 233 blocks; prose in brackets): of 0.780 r [0.541 r]
segments,
- one run onto one run exactly, (1, 1): **0.635 r** [0.324 r];
- one run tiled by several images, (1, b): 0.077 r [0.106 r];
- one image tiling several runs, (t, 1): 0.054 r [0.070 r];
- both sides plural: 0.014 r [0.042 r].

On Wikidata LF sends 63.5% of the runs exactly onto another run: the exchange is mostly a
permutation of runs (the record's tunnels, as a count of runs), and only 1.8% of its segments
(0.014 r) need a two-sided merge.

**5.3 (P) Coincidences persist.** A step removes an interval that is one set of points in both
tilings and contains no top boundary inside; every other coincidence survives. Hence a
Rauzy step is valid at *any* coincidence, not only at m: letters A (top interval ending
there) and B (image ending there), same table. Steps on disjoint letters commute.

**5.4 (C) The induction's dependency order.** Segment σ needs segment σ' iff σ' re-seated or
shortened a letter σ consumes. Longest chain, 233 blocks: **63 / 78 / 93 / 117 / 337** rounds
(min, p10, median, p90, max); median 606 segments a round. Prose: 79 to 118 rounds for
3·10⁵ segments; uniform bytes: 47; JSON of 20k records: 660 to 944. Same-round segments share
no letter (**P**: a shared letter orders them).
- **The rounds are not what limits workers; a long merge's chain is** (**C;** all 233 blocks).
  Half of a block's steps lie in its first 4 rounds (3 to 5): the tail is long and thin.
  - *Whole merges* in one round share no letter and commute (§5.3): threads may take them
    at no change to the grammar.
  - *A merge's own steps* are a chain as the induction takes them (each needs the one
    before it), and the widest merge is 1,562 steps (median). Counted a step at a time the
    steps are 2,630 / 3,861 / 7,707 deep (min, median, max over blocks), S/depth 15 to 55
    (median 28). Any greedy schedule on P workers then takes between max(depth, S/P) and
    S/P + depth (P − 1)/P turns (**K:** Graham's bound). S over the turns, pooled: **1.93 to
    2.00, 3.6 to 4.0, 6.5 to 8.0, 10.6 to 16.0** at P = 2, 4, 8, 16 (worst block at 16: 8.1 to
    15.4).
  - *If a merge's steps could be split among workers* (it is a merge of two prefix sums;
    not built) a round of w steps would take ⌈w/P⌉ turns: S over those turns is 2.00, 3.99,
    7.97, 15.87 (pooled; worst block 15.67). This entry first gave these as the schedule's:
    they are its ceiling, and on one block the step depth, 7,009, exceeds the 6,800 turns so
    counted at 16.
  - Hand-offs are free in all of these: what a thread pays to take a segment is not
    counted, and a segment is 1.8 steps.
- **Rounds as passes over arrays: built by a third sweep, closed as built on one thread (C, M:
  the sweep's program and run, not in the repository).** A
  round = an independent set of segments (no letter with its top place in one of them and
  its image in one of them, the same or another: a segment that meets a letter it re-seated
  itself is in no round), all consumed at once (**P:** the first return to what is left is then again
  an exchange, each cell a letter). Largest segments first: 24 rounds (median; 23 to 26), the
  rows shrinking ×0.58 a round; exact chains taken whole first: 17. But the letters are read
  again every round, 2.4 to 3.8 r letter-rounds in all, and the cells are 1.85 to 2.06 r, more
  than the right end's. Built as two arrays rebuilt a round, bytes exact on all 233 blocks:
  order and induction, with no write, are 1.99× build + walk (385 to 550 instructions a
  run). As built, the sweep is the one-thread form; what rounds can still buy is one
  block's latency on idle cores.
- A segment that meets a letter it re-seated itself is a genuine Euclid stretch: **84 of
  15,141,369** (1,901 steps of 27,439,723). On real blocks the Euclidean part is absent.
- Counted a step at a time (a step needs the last step on either letter and the one before it
  in its segment): depth median 3,861; widest segment 1,562 steps (median over blocks; 4,232
  at most): one long run absorbing its predecessors, itself a prefix sum.

**5.5 (P; C on every block) The count without the induction.** Cut π's cycle after every run
start: r stretches (x, πx, …, t], t ∈ 𝓣. In the text a stretch is what lies between two
consecutive run-start suffixes, read backward, and its values are those suffixes' ranks. With
p the minima-so-far of a stretch read forward and s read backward:

**S = Σ over stretches (p + s − 1) − 1; top-wins steps = Σ (p − 1); bottom-wins = Σ (s − 1);
equal = r − 1.**

*Proof.* The top boundaries of π_m are the last points below m on the backward orbits from the
run starts (the run start counted); the bottom boundaries the first points below m on the
forward orbits from the image starts. A step moves m to the largest boundary below it; that
boundary then leaves the domain and its orbit's next entry takes its place: so every
boundary, at every m, is passed. Hence y ≥ 1 is passed iff y is a minimum-so-far of an orbit
read backward from a run start or forward from an image start; the nearest run start gives
the weakest condition, so only the stretch holding y matters. m' is a bottom boundary alone
⇔ the top wins; a top boundary alone ⇔ the bottom wins; both ⇔ equal, which is a stretch's
least row. Row 0 is no step. ∎ (The formula is the complexity sweep's; the proof is redone
here.)

- **C:** `admissible` in the example computes the three sums from L alone and asserts them
  against the induction's three counts: equal on all 233 Wikidata blocks, 4 prose, 28
  synthetic.
- **A single-row stretch is a coincidence** (πt ∈ 𝓣): 0.780 r of the stretches, one equal step
  each. The other 0.220 r stretches average 59.3 rows and 3.69 cut steps (prose: 0.459 r, 4.8
  rows, 1.83). So S/r = 1 + 0.220 × 3.69 = 1.81: the "1.8" is the record count of suffix ranks
  inside the longer stretches. Random order would give 2H_g − 2 ≈ 7.3 cuts at g = 59; the ranks
  along a stretch are far from random.
- r − 1 ≤ S ≤ n − 1 are its extreme cases: all stretches single rows; all stretches V-shaped.
- The stretches are the phrases of the BWT's bidirectional scheme (Navarro–Ochoa–Prezza 2021,
  Def. 5): the induction's steps are the left-to-right and right-to-left minima of ISA inside
  each phrase.
- **The induction read in the text (P, Invariant I restated):** at the prefix [0, m) the text
  is cut exactly at the suffixes of rank < m, and the letters' words are the pieces between
  cuts (as letters: two letters may spell one word, and the grammar is not reduced). A step uncuts the suffixes of rank in [m', m), which are all the places where one
  piece w_A meets one piece w_B, and names the joined piece. So the grammar is a pair-merging
  of the text's positions in decreasing order of suffix rank: the largest suffixes lose their
  cut first.

- **The count depends on where the rows begin** (**C;** 10 blocks, every 24th, by a kernel
  with turned orders that is in no commit; **Conj** for the reason). LF is an exchange in the order of the rows turned to begin at any row p, on at
  most two letters more (the run of p and the image holding p are cut there), and the
  induction then ends on p's letter. Turned to orig_ptr it makes 0.98 to 1.20 times the cells
  of the order from row 0: +11% pooled. From row 0 the grammar is the Lyndon tree (§5.6);
  from another row it is the Cartesian tree of a rank order that is no lexicographic order.
  That this must make more cells is not proved. The left end makes more too (§5.6).

**5.6 What the grammar is; what a step is; the count is the schedule's.** Two sweeps of this
mission, each with its own simulator and neither using the example's code. Reproduced here:
`lyndon_cartesian.py.txt` (33,903 words for the Cartesian tree, 34,267 for the Lyndon tree)
and `schedules_geo.rs.txt` ("geo" below: the right end, the largest cell, free-first with four
paid orders, and the two orders that tell "free" from links alone; run on all 233 blocks by
two reviews, its right-end count equal to the example's S on each). What is marked "a sweep"
below was counted by a sweep's own simulator, which is not in the repository.

- **Theorem 1 (P; C: 33,903 words by `lyndon_cartesian.py.txt`; every node of every block by a
  sweep's simulator).** T primitive, V its least rotation,
  key(j) = the rank of the rotation that starts at V[j], 0 < j < n. The derivation tree of
  the right-end grammar is the min-Cartesian tree of key, and the rule of step [m', m) sits
  at exactly the nodes with key in [m', m).
  *Proof.* By Invariant I read in the text (§5.5) the cuts at prefix m are the boundaries
  with key < m. A step removes the cuts with key in [m', m), each a junction w_A | w_B. No two
  of them share a piece: a junction's right piece ends at a cut with key in I_B, its left
  piece at one with key in [m', m) ⊂ I_A, and I_A ∩ I_B = ∅. So removing them together is
  removing them one at a time in decreasing key: the Cartesian tree. ∎
  **C:** 209,697,429 nodes (233 blocks) and 3,535,835 (prose): each node's two children and
  span are its rule's two symbols and length; no mismatch.
- **Theorem 2 (K + C).** That tree is the Lyndon tree of V: its right standard
  factorisation (w = uv, v the least proper suffix, which is the longest proper suffix that
  is a Lyndon word) taken recursively (Hohlweg–Reutenauer 2003: the Lyndon tree of a Lyndon
  word is the Cartesian tree, least value at the root, of its suffix ranks; for a Lyndon
  word suffix order is rotation order). The left standard factorisation gives another tree
  (Dolce–Restivo–Reutenauer 2019).
  Every letter's word at every moment is a Lyndon word, every rule a standard factorisation
  (**C:** 468,657 rules on 34,267 words).
- **Corollary (P).** Equal words have equal trees, so the grammar with equal rules shared is
  the Lyndon SLP (Tsuruta et al. 2020). **C:** D = 12,904,656 distinct rules = **0.470 S =
  0.852 r** (per block 0.43 to 0.53 S; prose 0.303 S = 0.558 r); the strings compared byte for
  byte. For scale, on the same least rotations: LZ77 0.295 r phrases (longest previous
  factor and a byte; 0.385 r without the byte), batch RePair 0.530 r (rules and final
  sequence). S is 4.7 to 6.1 z: this grammar is cheap to make, not small.
- **Where the duplicates are (C).** 0.224 S repeat the rule made just before: 0.175 S are
  top-wins steps (61.6% of them: one top letter taking in consecutive bottom letters that
  carry one word; the runs of a byte lie together on the bottom), 0.045 S the equal step
  that ends such a streak, 0.004 S bottom-wins. 0.306 S repeat an earlier one. Two top-adjacent
  letters never carry one word (**P:** their orbits would stay adjacent through the run
  start that parts them).
- **Theorem 3 (P from §5.5 and Theorem 1; C: 9,335 Lyndon words, by the proofs review).** Call hinges the r text positions whose bytes the run-start
  rows emit: the byte just left of each run-start boundary (row 0's is V's last byte). S =
  the Lyndon-tree nodes on the hinges' paths, leaf to root. Equal steps are the nodes with a hinge on both
  sides; a top-wins or bottom-wins step is a node whose other child lies inside one stretch.
  The r − 1 equal steps, as a tree, are the Cartesian tree of the stretch minima in the
  order LF's cycle meets them.
- **A step is a digram (P).** Write LF's cycle as a cyclic word of n run letters (the run each
  row lies in). The cell I_A ∩ J_B is every occurrence of one digram, and they are one interval
  of rows. r₂ = the distinct digrams. A step replaces all occurrences of a digram by a new
  letter: a step of RePair on the run letters, in O(1) whatever the occurrences number. The
  right end chooses the digram holding the largest row; RePair's own choice is the largest
  cell.
- **Theorem 4 (P): steps = r₂ − 1 + U, for every schedule of cell steps,** U = the rows that
  become a boundary by a re-seating that lands on none (a new digram). *Proof.* A step merges
  one boundary of the common refinement into the next; no other boundary ever ceases; a
  re-seated loser adds at most one; one cell is left. ∎ So r₂ − 1 is the floor, and a step is
  free iff it is equal or its loser lands on a boundary of the other tiling. In §5.5's
  picture, for steps of §5.3's kind (on the intervals that end at the coincidence): a
  stretch's first row is a bottom boundary and its last a top one; a step removes one end of
  a stretch and, for nothing, interior rows, its own or others'; U = the interior rows that
  ever become the first or last survivor of their stretch. The right end makes them the
  minima-so-far; an order that empties a stretch's interior before it touches the ends
  makes none. (For steps on intervals that begin at a coincidence the picture is its
  mirror, the cycle cut after every run end; the count holds for the left end as well.)
- **S is not the schedule's invariant (C).** Smallest case: `abacb` (L = `bbcaa`, r = 3,
  r₂ = 4): right end 4 steps, left end 3. On Wikidata (all 233 blocks unless said; every run
  ends on a word checked to be the block, by bytes or by a 61-bit hash against all its
  rotations):

  | order of steps | steps / r | U / r | counted by |
  |---|--:|--:|---|
  | left end | 2.137 | 0.917 | a sweep |
  | a random coincidence | 1.981 | 0.760 | a sweep |
  | **right end** (this entry's) | **1.812** | 0.592 | the example; geo |
  | equal steps wherever ready, else right end | 1.714 | 0.494 | a sweep (1.657, 0.437 by the referee, with a policy it added to geo: in the session's record, not in the committed file) |
  | free steps to exhaustion once, then right end (8 blocks) | 1.627 | 0.408 | a sweep |
  | largest cell (RePair's choice) | 1.459 | 0.239 | geo |
  | free steps wherever ready, else right end | 1.456 | 0.235 | geo |
  | **free steps wherever ready, else the largest cell** | **1.279** | 0.059 | geo |
  | the same two with "free" told from links alone (§8, 2) | 1.467, 1.302 | 0.247, 0.082 | geo |
  | free first, the paid step on the shortest stretch (8 blocks) | 1.265 | 0.046 | a sweep |
  | floor, r₂ − 1 | 1.220 | 0 | geo |

  The 8 blocks are every 33rd of the 233 (7,199,894 rows), where the right end is 1.807 r and
  the floor 1.220 r: their rows rank against those, and 1.265 there against 1.279 on all
  ranks nothing.
  Prose: right end 1.841, free then largest cell 1.521, floor 1.459. The left end is worse
  than the right on every block (1.08 to 1.27×); two-sided rules and orders by bucket are
  worse than the right end too (8 blocks: 1.84 to 2.12, against the right end's 1.807
  there).
- **The free steps alone (C, 8 blocks):** done to exhaustion before anything else they are
  0.911 r steps (0.809 r in their first layer) and leave 0.176 r letters, 0.309 r cells and
  36.9% of the rows. A pass of free steps and then the right end is 1.627 r; only free and
  paid steps interleaved reach 1.3.
- **Below the floor (C).** Two letters with one word, adjacent in both orders, are one letter
  ("fusion"); it never arises at an end. `aaabaabbabb`: r = 10, least number of steps 8. On
  Wikidata (8 blocks) it takes the best orders found from 1.265 r to 1.240 r, and with split
  steps from 1.253 r to 1.223 r, still 0.2% above r₂ − 1 pooled; one block of the 8 ends
  below r₂ − 1, by 0.16%.
- **What this is worth on the clock: nothing yet.** At the cells step's price (**Conj:**
  unchanged out of order), 0.53 r fewer steps would be 17% of the kernel's time: 0.756 →
  about 0.62 of the column's path. But the cells step is cheap because the sweep reads each
  order from its end and fetches ahead. A step out of order keeps the cell (a segment can be
  left after any step) and loses the sweep: by the counts and an estimated price in
  instructions, not a clock, going from one segment's end to another costs more than the
  steps it saves (§8, 2).

**Consequence.** The serial chain of §3.1 is an artefact of working one coincidence: the same
steps order into ~10² rounds of independent merges (**C**, 233 blocks pooled: 51% of the steps
in the first 4 rounds, 70% in 10, 82% in 20, 91% in 32; the referee's count). Threads may
take whole merges at no change to the grammar (§5.3; same-round merges share no letter). A
merge's own steps stay a chain, 3,861 steps deep at the median, and that is what bounds the
turns (§5.4: between 10.6 and 16 times fewer at P = 16, hand-offs not counted). As passes
over arrays on one thread another kind of round was built and closed as built (§5.4:
1.99×). On one thread the step was bound by memory and by what the compiler kept on the
stack, not by its chain (§6.2).

## 6. Clock (M; one thread, fastest of 5 rounds a block; Xeon "2.1 GHz", 4 cores, 2 MiB L2 a core, a 260 MiB L3 as the container reports it; other agents' work on the other cores in most runs; indicative)

Arms. Production: `bwt::build` + `walk_segments_room` from the column. Kernel, from the run
list: **order** (the runs sorted by byte, both orders laid out: this exchange's "build"),
**induce**, **write**. The scan of the column into runs is apart: 0.484 ns a row here. #324's
front emits the records in place of the column's MTF stage (0.173 against 0.423 ns a row,
PF:`by-data-class` §2): a saving that §6.1 and §6.2 do not take and §6.3 does.
Both arms' live sets fit that L3, so no figure here need be a last-level miss; whether other
tenants of the host take that level is not known (§6.3). The simulated 2 MiB cache of §6.2
is this machine's L2.

**Production moved while this was measured.** §6.1 and §6.2 are against the column's path
(`bwt::build` + the room walk), which was production at this branch's base. Since 0.2.8
(#324 adopted, #352) the decoder inverts a block of few symbols by the records' path:
`runs::fill` from the front's records, then the room walk backward. Every Wikidata block
takes it. The Pathfinder measured it at 0.95 of the column's path on these 233 blocks
(2.26 to 2.27 against 2.37 to 2.40 ns a row, its container), which puts §6.2's 0.756 at
about 0.80 of production. The example now times that arm and feeds the cells kernel from
the same records (its order stage reads them: 127-row cuts joined, the walk's row cut out):
§6.3, after the writer's fix (§7 row 1).

**The core's clock.** A chain of dependent multiplications held in a register (3 cycles
each) runs at 0.96 ns, and a chain of adds agrees: about 3.15 GHz, not the 2.10 the
processor's name and its TSC give (`clock_chain.rs.txt`). Every cycle below is at 3.15.
The host moves it: within the hour the multiplication chain read 1.02 to 1.08 ns in
readings by hand, then 0.99 to 1.05 ns in five logged readings a minute apart: 2.8 to 3.0
GHz. So a count of cycles here is good to 13%, and a figure in ns compares only inside one
run, which is where every ratio below is taken. The ratios move with the machine's state
too, by less (§6.3).
(This entry first gave 2.10, then 2.73: its loop kept the product on the stack,
3.4 to 3.5 cycles an iteration, read as 3. Found by the numbers review.)

### 6.1 The first construction: linked orders (§3.1). 1.276× the column's path on Wikidata.

| Wikidata, 233 blocks, ns a row | build + walk | order | induce | write | kernel | ÷ |
|---|--:|--:|--:|--:|--:|--:|
| pooled | 2.172 | 0.525 | 1.826 | 0.419 | 2.770 | **1.276** |
| r/n < 0.06 (93 blocks, 0.399 of rows) | 2.128 | 0.397 | 1.349 | 0.362 | 2.108 | 0.991 |
| 0.06–0.07 (71, 0.305) | 2.182 | 0.451 | 1.516 | 0.391 | 2.358 | 1.081 |
| 0.07–0.09 (32, 0.137) | 2.191 | 0.570 | 1.997 | 0.453 | 3.020 | 1.378 |
| 0.09–0.12 (19, 0.082) | 2.262 | 0.816 | 2.883 | 0.534 | 4.233 | 1.871 |
| ≥ 0.12 (18, 0.077) | 2.228 | 1.098 | 4.091 | 0.641 | 5.830 | 2.617 |

| other inputs, ns a row | build + walk | kernel | ÷ |
|---|--:|--:|--:|
| Fibonacci | 2.04 | 0.102 | 0.05 |
| Thue–Morse | 1.90 | 0.102 | 0.05 |
| period 37 | 1.76 | 0.481 | 0.27 |
| JSON-like, 300 ids (r/n 0.022) | 1.91 | 1.144 | 0.60 |
| JSON-like, 3,000 ids (r/n 0.057, S/r 2.60) | 2.50 | 3.325 | 1.33 |
| JSON-like, 20k records (r/n 0.151) | 2.04 | 6.321 | 3.10 |
| prose (r/n 0.367) | 2.766 | 20.38 | 7.37 |
| `(ab)^k a` (no division rule) | 1.27 | 9.958 | 7.83 |
| uniform bytes | 3.29 | 53.4 | 16.3 |

- **The write is not the cost:** 0.419 ns a row for the whole block from the grammar, 0.10 on
  the Sturmian inputs (the n bytes themselves).
- **The step is:** 14.0 ns, 66% of the kernel: the order surgery, written as selects: 152
  instructions, 31 stores and, as compiled, one conditional branch (on λ_A = λ_B) a step
  (callgrind, block 1; its model mispredicts it 0.16 times a step). It rises with r, 12.6 ns in the
  lowest band of r/n to 17.5 in the highest. §6.2 finds where it went.
- The kernel's figure is the sum of its stages' fastest rounds: the example times no one
  span for this kernel, and by the cells' own two figures (§6.2) the sum is 1 to 2% low.
- **The budget, in cycles a run** (3.15 GHz, r/n 0.0722): production's build + walk is 95 here
  (the record: 2.48 ns a row on its VM, which it counted as 72 cycles at that VM's nominal
  2.1 GHz and would be 108 at 3.15; 110 on the i9). The kernel is 121: order 23, induce 80 (1.81 steps
  × 44), write 18. Parity alone needs induce ≤ 1.228 ns a row with order and write as they
  are (it is 1.826). Halving production here needs ≤ 47 cycles a run all told. At n/r = 13.9 the row
  walk is already cheap *per run*; that, not the count, is what an r-proportional method
  fights on this corpus.
- **Tried and closed as built (M): joining the (1, 1) chains before the induction**
  (`RAUZY_JOIN=1`; §5.2: one merge of the two tilings, a walk down each chain, the letters left
  renumbered and relinked; bytes exact on all 233 blocks). It joins 0.635 r letters, leaves
  1.101 r steps, and takes the induction from 1.826 to 0.928 ns a row. Its own passes over the
  letters take the order stage from 0.525 to 2.448: about 6.7 ns a letter a pass, in
  mispredicted branches (which run is joined, where a chain ends). Kernel 1.668× the walk. A
  pass over the letters must cost ≤ 3.1 ns a letter to pay; none here does. The construction failed, not the
  count: the 63.5% is still the cheapest work in the block.
- **Live set as built:** 32 r (nodes) + 0.47 n (arena) + 16 B × 0.039 n (rules) + 4 B a rule + n
  (the expansion before rotation): about 4 MB at n = 9·10⁵, r = 6.5·10⁴. Not smaller than
  production's table. The nodes halve with 16-bit links (r < 2¹⁶ on three blocks in four); the
  n goes when the rotation is folded into pass 1.
- **Verdict on this construction** (one chain, linked orders): it loses on Wikidata by 28%, is
  level below r/n ≈ 0.06, and wins from r/n ≈ 0.02 down.
- **Correction made while writing:** the first table of this entry excluded the order stage
  (it had been timed with the scan) and read 1.078×. The exchange's build is the kernel's
  work; 1.276 is the figure.

### 6.2 The second construction: cells (§3.5). Against the column's path, from the run list, before the writer's fix: 0.756× on Wikidata; 1.48× for r/n ≥ 0.12.

Same arms, same blocks, a later run (production 2.244 in it; the list kernel 2.808, 1.251×).
"Whole" is order + induce + write as one timed span; the stages are their own fastest rounds.

| Wikidata, 233 blocks, ns a row | build + walk | order | induce | write | whole | ÷ | lists ÷ (stages summed) |
|---|--:|--:|--:|--:|--:|--:|--:|
| pooled | 2.244 | 0.305 | 1.005 | 0.363 | 1.697 | **0.756** | 1.251 |
| r/n < 0.06 (93 blocks, 0.399 of rows) | 2.198 | 0.240 | 0.778 | 0.315 | 1.358 | 0.618 | 0.986 |
| 0.06–0.07 (71, 0.305) | 2.292 | 0.274 | 0.892 | 0.344 | 1.534 | 0.669 | 1.065 |
| 0.07–0.09 (32, 0.137) | 2.240 | 0.307 | 1.009 | 0.358 | 1.700 | 0.759 | 1.275 |
| 0.09–0.12 (19, 0.082) | 2.266 | 0.442 | 1.462 | 0.460 | 2.379 | 1.050 | 1.803 |
| ≥ 0.12 (18, 0.077) | 2.277 | 0.609 | 2.128 | 0.602 | 3.367 | 1.479 | 2.691 |

- **These two tables were measured before the writer's fix (§7 row 1).** On Wikidata the fix
  adds 126,658 bytes of copies over the 233 blocks and 1.2% to the writer's instructions
  (block 1): the pooled figures stand to that. The rows for period 37, Fibonacci and
  Thue–Morse do not: there the writer timed wrote a fraction of the block (75 of 888,001
  bytes on period 37). §6.3 re-measures them.
- 202 of 233 blocks faster, 0.867 of rows; 19 blocks lie in 0.09 ≤ r/n < 0.12, where the mean
  is 1.05. Per block: 0.46 / 0.57 / 0.66 / 1.12 / 1.82 (min, p10, median, p90, max; between
  this run and the second the ends move 15 to 30%, p10 and p90 3 to 6%, the median 2%).
- **Chosen by r/n** (the front has r before the inversion): cells below 0.10, else the walk:
  1.597 ns a row, **0.712×**; any threshold from 0.08 to 0.12 is within 0.007 of it; the
  better arm chosen block by block with hindsight is 0.711. The threshold is fitted and
  scored on the same 233 blocks of one corpus: the hindsight figure bounds the optimism
  here and says nothing of other data (§8, 4).
- **A second run,** the process pinned to one core and the others idle (during the first,
  other work ran on the machine's other cores): 1.604 against 2.172, **0.738×** (order 0.286,
  induce 0.947, write 0.344); 204 of 233 blocks faster; per block 0.39 / 0.55 / 0.65 / 1.05 /
  2.37; by r/n < 0.10: 0.702; the list kernel 1.199. The table is the first run. Block by
  block the two runs disagree by more than 10% on 89 blocks (block 34 is 1.78, then
  2.37, the slowest of the second run; the slowest of the first is block 23, 1.82): the per-block quantiles are indicative, the pooled figures are
  what repeats.
- Cells: 27,439,756, the reference's S and 33 more (the walk's row cut out, 233 blocks).
- **A cell is 7.68 ns = 24 cycles, 64 instructions.** A run: 74 cycles (order 13.3, induce 43.9,
  write 15.8) against the column path's 98.

| other inputs, ns a row | build + walk | cells | ÷ | (lists ÷) |
|---|--:|--:|--:|--:|
| Fibonacci | 1.905 | 0.039 | 0.02 | 0.05 |
| Thue–Morse | 2.342 | 0.047 | 0.02 | 0.05 |
| period 37, and one byte | 1.988 | 0.366 | 0.18 | 0.23 |
| JSON-like, 300 ids (r/n 0.022) | 1.977 | 0.755 | 0.38 | 0.57 |
| JSON-like, 3,000 ids (r/n 0.057, S/r 2.60) | 2.148 | 1.707 | 0.79 | 1.17 |
| JSON-like, 20k records (r/n 0.151) | 1.829 | 3.215 | 1.76 | 3.20 |
| prose (r/n 0.367) | 2.769 | 10.89 | 3.93 | 7.46 |
| 4 letters, uniform | 2.644 | 17.15 | 6.48 | 11.4 |
| uniform bytes | 3.686 | 36.06 | 9.78 | 16.3 |
| `(ab)^k a` (no division rule) | 1.309 | 10.84 | 8.28 | 8.41 |

One run an input. Production's own figure for these inputs moved by up to 23% either way
between §6.1's run and this one (Thue–Morse 1.90 and 2.342; JSON of 3,000 ids 2.50 and
2.148; prose 2.766 and 2.769): a ratio here is good to about 20%.

**What moved it** (10 blocks, every 24th, the same each time, one run each; instructions
from callgrind on block 1. Run to run the last column moves by about 10%: the two censuses
give 0.87 and 0.86 on these ten blocks for the final kernel, against the 0.78 here. The
order of the rows is what the table shows, not their third digit. The kernels of the rows
between the first and the fifth are in no commit: the cells kernel entered the repository
whole, in its last form.):

| the step | instructions a cell | ns a cell | whole ÷ walk |
|---|--:|--:|--:|
| lists (§6.1) | 152 | 13.4 to 14.9 | 1.29 to 1.33 |
| cells, first form: the step written as selects; slots as three 64-bit fields, two patched a cell; a point followed | 161 | 13.4 | 1.225 |
| one buffer; a three-way branch; no patch (the promise); the slot before read with the letter | 91 | 12.5 | 1.063 |
| no point followed: the walk's row marked, the block written unrotated | 62 | 9.8 | 0.936 |
| fetch ahead, 8 to 16 slots (12 kept) | 64 | 7.2 to 7.7 | 0.78 |
| *(orders turned to orig_ptr in place of the mark: +11% cells, §5.5; per cell of its own)* | 64 | 10.2 | 1.038 |

- **The first two rows cost the same and were not bound by their instructions.** 161 → 91
  bought 7%. The compiler had made the followed point 15 branch-free instructions a cell
  with their operands on the stack, and had put the slot-before of both letters there too:
  the chain from a letter to the next ran through a store and a reload. Removing the point
  (§2) removed the chain: 91 → 62 instructions, 12.5 → 9.8 ns.
- **Then it was memory.** A cell is written to two places a counting sort apart. Fetching
  them when the letter is 12 slots from the front: 9.8 → 7.2 ns. What is left of the rise
  with the block's runs (all 233 blocks, first run; pinned run in brackets): 7.4 [6.9] ns a
  cell for r below 7·10⁴, 7.7 [7.5] to 10⁵, 8.9 [8.4] above.
- **Per run of L (C: callgrind, one block each; the kernel before the fix, fed from the run
  list, against the column's path; the scan of the column, 52 to 61 instructions a run,
  apart; misses of a simulated 2 MiB cache, this machine's L2):**

  | block | r/n | instructions: column's path | cells | misses: column's path | cells |
  |---|--:|--:|--:|--:|--:|
  | 1, self costs (build 85 + room walk 175; order 27 + induce 1.81 × 64 + write 41) | 0.057 | 260 | 184 | 8.7 | 2.1 |
  | 1, with callees | 0.057 | 268 | 186 | 9.1 | 2.1 |
  | 22, with callees (the corpus's ratio) | 0.072 | 217 | 180 | 7.9 | 2.3 |
  | 23, with callees | 0.130 | 129 | 164 | 4.0 | 2.6 |

  Blocks 22 and 23 are the referee's counts (§11). Production's work is by the row, so
  counted by the run it falls with n/r: the gap in instructions halves at the corpus's own
  ratio and is reversed by r/n 0.13. The induction is 64 instructions a cell on all three.
  The predictor callgrind models misses 0.20 branches a cell.
- **Live set:** slots 24 B × (2 r + the cells that took a fresh one, 0.81 r) = 67 B a run; words
  as bytes 0.47 n; rules 16 B × 0.54 r; a position a rule; the output. About 5.5 MB and the
  output at n = 9·10⁵, r = 6.5·10⁴: more than production's table. What differs is how it is
  touched: two forward streams and writes that are fetched ahead.
- **Tried, not kept: one word for consecutive bottom letters that carry one word** (§5.6:
  22% of cells repeat the rule made just before; the 17.5% that are top-wins were tried). Two runs each way on 20 blocks (M;
  the files were not kept): write 0.335 → 0.32, induce 0.91 → 0.92 to 0.96, whole 1.545 →
  1.545 to 1.59. The word was not the cell's cost.
- **Memory safety:** the kernel is `unsafe` (unchecked offsets in one buffer sized for n
  cells). valgrind: 3,000 arbitrary columns and three real blocks, no error; but inside one
  zero-filled allocation kept from block to block it sees neither a write past a region's
  end nor a read of a slot not yet written. The regions' ends are now asserted where the
  induction returns, under debug assertions (the `fuzz` profile of Reproduce). Reviewed once
  by a sub-agent with every access checked against its region (19 million small columns,
  the 233 blocks, prose, the synthetic files): the bounds hold (at most n − 1 cells; at most
  2n − 1 slots a side of 2n + 8; the arena's last store ends at most 16n + 304 bytes in, of
  16n + 368; at most n rules of n + 4). It found what valgrind cannot see: `copy16` used
  `copy_nonoverlapping` on ranges that overlap (a build with debug assertions aborts on
  `bba`, row 0; now a 16-byte read then write, the same instructions), and the unwritten
  bytes of §7 row 1. The buffer is allocated for n cells, 128 n + 4,912 bytes (115 MB at
  n = 9·10⁵); on Wikidata at most a tenth of it is written, and only written pages are
  touched.
- **Not measured:** the i9; ten workers on a shared 20 MB last level, where production's walk
  is the known sufferer. On three blocks, one thread and a simulated private 2 MiB cache
  the cells miss a quarter to two thirds as often: that is no model of a shared level. A
  decoder that calls it.
  (Records from the front in place of the scan: §6.3.)

### 6.3 Against production as it now is: the records' path, from the records, after the writer's fix

Three arms from one block's symbols (one thread, fastest of 5 rounds a block, the process
pinned; three runs of all 233 blocks, the table the one whose pooled ratio is the middle):
- **the column's path:** `mtf::mtf` (the front, apart), then `bwt::build` + the room walk;
- **the records' path, production's since 0.2.8** (every one of the 233 blocks takes it):
  `runs::front` (apart), then `runs::fill` + the room walk backward;
- **cells:** from run records in `runs::front`'s format, which its order stage reads (the
  127-row cuts joined, the walk's row cut out). `runs::Runs` keeps its records private, so
  the example makes the same records from the column, apart. Order + induce + write is one
  timed span ("whole").
All three are asserted against `bwt::walk`, the cells' output complemented first.

| Wikidata, 233 blocks, ns a row | column | records | order | induce | write | whole | ÷ records | ÷ column |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| pooled | 2.176 | 2.049 | 0.367 | 0.937 | 0.347 | 1.677 | **0.818** | 0.771 |
| r/n < 0.06 (93 blocks, 0.399 of rows) | 2.156 | 2.031 | 0.298 | 0.730 | 0.303 | 1.354 | 0.667 | 0.628 |
| 0.06–0.07 (71, 0.305) | 2.185 | 2.046 | 0.328 | 0.814 | 0.324 | 1.492 | 0.729 | 0.683 |
| 0.07–0.09 (32, 0.137) | 2.163 | 2.057 | 0.378 | 0.974 | 0.349 | 1.730 | 0.841 | 0.800 |
| 0.09–0.12 (19, 0.082) | 2.207 | 2.037 | 0.531 | 1.424 | 0.455 | 2.446 | 1.201 | 1.108 |
| ≥ 0.12 (18, 0.077) | 2.226 | 2.157 | 0.687 | 1.915 | 0.541 | 3.173 | 1.471 | 1.425 |

- **0.818 of production, pooled** (0.818, 0.820 and 0.817 in the three runs; 0.836 to 0.858
  in nine later ones: "A later state", below); 0.771 to 0.776 of the column's path in the
  three, 0.788 to 0.814 in the nine. Cells faster than production on 194, 192 and 197 of the 233 blocks. Per block
  (this run): 0.46 / 0.62 / 0.72 / 1.20 / 1.79 (min, p10, median, p90, max).
- Chosen by r/n < 0.10, the records' count against n: 0.765 of production (0.761 to 0.765 in the three runs; 0.765 to 0.792 in the nine).
- The records' path is 0.942 of the column's (the Pathfinder: 0.95), by its fill (0.323 against
  the build's share): the walk is what both keep and what the cells remove.
- The fronts: `mtf::mtf` 0.499, `runs::front` 0.166 ns a row. From the block's symbols, front
  and inversion: records 2.215, cells 1.843.
- Against §6.2: the order stage is 0.367 against 0.29 to 0.31 there, because it now reads the
  records (two passes) where §6.2's was handed the letters; the writer's fix is inside
  `write`.
- At 3.15 GHz, a run of L: production 89 cycles, cells 73 (order 16, induce 41, write
  15).
- **How far to trust it.** One container, one thread, another agent's work on one of the
  other three cores throughout. The pooled ratio of 233 blocks repeated to ±0.2% over three
  runs made within four minutes, and not half an hour later (below); a ten-block sample does not (0.81 to 0.97 over eight runs at eight stack depths, each arm
  moving ±6%, no depth standing out). The hazard the Pathfinder met, a hot stack object
  across a page, which makes a kernel 30% slower or faster from one build to the next, did
  not show; but the depths were 520 bytes apart, set by the environment's size, coarser than
  a narrow band, and its finer sweep (PF:`2026-11-10-team-runs-port/stacksweep.rs.txt`: 80
  bytes a step, three passes) was not run. Against a build's artefact there is this: five
  builds of different stack layout took a census of the 233 blocks, and the column's arm
  was between 2.17 and 2.24 pooled in each run on a quiet machine. The instruction counts of §6.2 are for the kernel before
  the fix and the column's path; not recounted.
- **A census set aside.** Six minutes before the three, a census of the same 233 blocks
  overlapped a commit, whose hooks build and lint on every core. It took 89 s against 41,
  its one-span figure for the cells was 11% above the sum of their stages (1.6% in the
  three), and it read 0.90 (column 2.348, records 2.138, cells 1.926). It was set aside then
  as disturbed, and no figure here counts it.
- **A later state of the machine reads 0.84 to 0.86 (M).** From 34 to 48 minutes after the
  three runs, pinned as before, nine more censuses of the 233 blocks on the same core.
  - The code as this entry leaves it (the kernel of the three runs: only comments and one
    debug assertion differ): **0.846** (column 2.436, records 2.287, cells 1.935; faster on
    189 blocks).
  - The previous commit's binary and this one, back to back, twice: 0.836, 0.857, 0.843,
    0.840. The previous commit's binary, which read 0.817 to 0.820, now reads 0.836 and
    0.843: the move is not the build's. Between the two binaries, +0.009 on two runs each:
    no signal.
  - The core alone, then with three more copies of the census on the other three cores,
    twice: 0.842, 0.853, 0.852, 0.858 (the six copies on the other cores: 0.841 to 0.857).
    The rule was posted before the data (#145, 6771530689): a change of more than 0.02 at
    four copies would bear on §8 item 1's conjecture. It is +0.011 and +0.006 in the two
    pairs: no signal either way. Copies of the census on the other cores do not make the
    slower state, and do not contend here: the ns barely move.
  - In all nine every arm is slower in ns than in any of the three runs. Against the
    table's run, the fastest of the three: production by 6 to 13%, the cells by 9 to 19%
    (the column's path 2.29 to 2.48 against 2.18 to 2.21; the multiplication chain of §6
    read 2.8 to 3.0 GHz against 3.15). In the slowest run the cells' three stages are 17 to
    19% slower than in the table's, the two walks 12.5 and 13.5%.
  - What on the host does this is not known (**Conj**, two candidates: a slower clock, which
    costs a kernel bound by computing more than one that waits on memory, though the chain
    read at most 13% slower and the cells lost up to 19%; or other tenants of the shared
    last level, which cost the kernel that leans on fetching ahead). **So the figure is
    0.82 to 0.86 on this container** in twelve pinned runs, and "±0.2%" was the spread
    inside one state of it. Faster on 185 to 197 of the 233 blocks; chosen by r/n < 0.10:
    0.761 to 0.792.

| other inputs, ns a row (the fixed writer) | production | cells | ÷ |
|---|--:|--:|--:|
| Fibonacci | 1.838 | 0.041 | 0.02 |
| Thue–Morse | 1.749 | 0.050 | 0.03 |
| period 37, and one byte | 1.466 | 0.461 | 0.31 |
| JSON-like, 300 ids (r/n 0.022) | 1.794 | 0.836 | 0.47 |
| JSON-like, 3,000 ids (r/n 0.057) | 1.986 | 1.697 | 0.85 |
| JSON-like, 20k records (r/n 0.151) | 1.969 | 3.337 | 1.69 |
| prose (r/n 0.367; one block of 4 takes the records) | 2.712 | 10.934 | 4.03 |
| 4 letters, uniform (one block of 2 takes the records) | 3.011 | 16.138 | 5.36 |
| uniform bytes (the column: no block takes the records) | 3.367 | 30.114 | 8.94 |
| `(ab)^k a` (no division rule) | 1.115 | 12.512 | 11.22 |

Production here is what the decoder chooses for the block: the records' path where
`runs::takes_runs` admits it, else the column's. One run an input: good to about 20%
(§6.2's note). Period 37, Fibonacci and Thue–Morse replace
§6.2's rows, which timed a writer that left most of those blocks unwritten.

### 6.4 Other constructions, tried by sweeps of this mission. None is in the example.

Each was built or counted by a sub-agent in its own scratch code, asserted byte for byte
against `bwt::walk`; the first two were then attacked by another, with its own census and
an output buffer filled with a wrong byte before each kernel. Clocks are theirs, against
the column's path (the records' arm did not exist for them), on this shared machine.

- **A forest, then a walk ("tower").** A letter is non-branching if its image lies inside
  one top interval; then all its points go next to that letter. **C:** 0.929 r of the
  letters on Wikidata; the rest are roots (0.071 r). **P:** the first return to the roots'
  intervals is an exchange whose letters are the cells of the roots' images, each with the
  word of a path up the forest. The forest needs no bottom order: the letter over the start
  of an image is a rank in a bitmap of the top boundaries. Taken twice it leaves 0.121 n
  rows, walked on 8 lanes. All 233 blocks, bytes equal.
  - Clock, re-measured by the adversary: **0.78 to 0.80 of the column's path** from a run
    list (its builder had read 0.74, against a baseline slowed by other arms in the same
    rotation). **4 to 6% behind the cells kernel** in the same runs (four orderings of the
    arms; 1.5% ahead in a fifth).
  - Instructions a run: 291 against the column path's 266 and the cells' 186. Simulated
    misses a run: 2.2 / 9.5 / 2.1 at 2 MiB (tower / column path / cells);
    8.5 / 19.3 / 5.0 at 256 KiB, the i9's L2.
  - It has no Θ(n) case at a fixed number of levels (`(ab)^k a`: 0.75×, where the cells are
    11×). It fails where the second level leaves much: records of one mutation each
    (r/n 0.015): 1.4 to 1.6×, where the cells are 0.42 to 0.54×.
  - What carries over: the forest by rank; what the first level leaves, U/n, as a test for
    admission.
- **The exact chains joined first, the passes rebuilt without a data-dependent branch,** in
  front of the list kernel: 1.05 to 1.07× the column's path (the adversary's pairing; its
  builder read parity). In front of the cells it would cost 117 instructions a run for the
  72 it replaces. §6.1's verdict on a pre-pass stands, at under half of §6.1's price.
- **Words as lazy tags** (Agol–Hass–Thurston's weights on ranges, carried as words): a letter
  holds two chains of immutable nodes, and consecutive losers of one winner that held one
  chain share the new node. **C:** 1.257 r nodes against 1.812 r cells, bytes equal on all
  237 blocks and 205,000 random columns. No kernel; its price (72 instructions a run for
  the induction against 116) is an estimate that rests on an unwritten 4-ary writer.
- **Equal cells of one row, linked and not merged.** **C:** 79% of the cells are of one row
  (1.432 r: equal 0.842, top-wins 0.363, bottom-wins 0.227) and remove 10% of the rows; the
  other 0.380 r cells remove 90%, 33 rows each. **P:** only an equal one-row cell can be
  left without a write. A sweep that links those and walks the 0.842 r points at the end
  makes 0.970 r cells. Its walk alone, measured: 1.4 ns a point on 4 lanes with evenly
  spaced starts, 5.2 ns on one. Modelled whole: 0.64 to 0.74 of the column's path against
  the cells' 0.756 then; nothing with one lane. Not built into the kernel.
- **Stopping the induction early and walking what is left: closed as priced.** steps(m) + m
  never rises along the induction (**P:** a step removes at least one point), so a walk
  point must cost less than (S − steps(m))/m of a cell: 0.23 at n/4, 0.42 at n/16 (**C**, a
  sweep's). Priced, the gain is under 3% of the kernel: a model, not a clock.
- **Long copies scheduled by the stretches' copy graph: closed on a sweep's count (C).** The
  graph has one strongly connected component holding 99.3% of the copied rows.
- **The multiplicative (Zorich) step:** 94 of Wikidata's 27,439,723 steps are in a streak it
  would shorten. It is what `(ab)^k a` needs and nothing a real block does.
- **Off the corpus r/n does not admit safely (C, the adversary's inputs, cells ÷ the column's
  path):** random letters with geometric run lengths, r/n 0.083: 5.2×. The budget on the
  cells (§8, 4) is what protects, not the threshold.

## 7. Falsifiers run

| # | attack | outcome |
|--:|---|---|
| 1 | bytes ≠ walk on any block | none for the reference and the list kernel: 233 Wikidata blocks (and the corpus's 198 tiny ones in the independent check), 4 prose, 28 synthetic incl. a period and one byte more and n < 20 (none an exact power: six exact-power blocks made by the referee pass too, and columns of several cycles are `--arbitrary`'s); 200,000 arbitrary columns. **The cells kernel: fired, once its check could fire.** As first built its assertion could not see a byte left unwritten (`out` held the list kernel's bytes, already equal to the walk's), and the writer left some: a long factor of a marked rule lying at a later place than its first was noted and never copied. With `out` made wrong first: 178 of 233 Wikidata blocks (126,658 bytes), period 37 (887,926 of 888,001), Thue–Morse, Fibonacci, JSON of 300 ids, one prose block, `--arbitrary` at case 13,584. Found by the memory-safety review (§6.2). Fixed (§3.5: a third walk of the marked rules; `out` is made wrong before the writer in every check); none since on all of the above, the debug-assertion build included |
| 2 | S ≫ r on natural data | no: 1.29–2.08 r |
| 3 | S = Θ(n) on some input | **yes**: `(ab)^k a`, S = n/2, 11× slower (cells; 8× lists). Fix: division rule (q equal-pair steps → one power rule), or a step budget with fallback to the walk. Wikidata: 46 such steps in 27.4 M |
| 4 | text | **fires**: S/n 0.68, 4.0× slower (cells; 7.4 to 7.5× lists). Closed above S ≈ 0.16 n at this step price (§8, 4; on Wikidata that is r/n ≳ 0.09) |
| 5 | the clock on Wikidata | **fired for the linked orders**: 1.276×. **Does not fire for the cells**: 0.82 to 0.86× of production as it now is, by the machine's state, 185 to 197 of 233 blocks (§6.3; 0.756× of the column's path before the writer's fix, §6.2) |
| 5a | the orders turned to orig_ptr, to follow no point | **fires**: +11% cells (§5.5). The mark replaces it |
| 5b | "S is what the block costs, whatever the order of steps" (this entry, before §5.6) | **false**: 1.279 r to 2.137 r by the order; `abacb` already. The floor is r₂ − 1, not r − 1 |
| 6 | bulk-join the exact chains first (the cheap 63.5%) | **fires as built**: 1.668×; letter-passes at 6.7 ns each (§6.1) |
| 7 | prior art | the mechanism is classical; the transfer is not in print (§10) |
| 8 | independent implementation from the spec, invariant checked by brute force | no counterexample (§11) |

## 8. Open, ranked: ceiling · cheapest falsifier · gate

1. **The i9, and its ten workers.** Here: 0.82 to 0.86× of production on one thread (§6.3);
   against the column's path on block 1, 186 instructions a run to 268 and a quarter of the
   modelled misses (at the corpus's r/n, block 22: 180 to 217, and 0.29 of the misses; with
   callees, §6.2). The column's path is 110 cycles a run there (PF:`phrase-structure` §5)
   against 95 to 98 here, on a 256 KiB L2 and 2 MB of last level a worker.
   - **Conj:** the gap widens there (modelled misses a run at 256 KiB, its L2: 5.0 against
     the column path's 19.3, §6.4).
   - Against it (**M**, §6.3): in this container's slower state the cells lost more than the
     walk, 9 to 19% against 6 to 13%. The cause is unknown, so what it says of the i9 is too
     (**Conj**). Four copies here gave no signal.
   - Falsifier: this example's two arms on the workstation, at 1 and at 10 pinned copies;
     the conjecture falls if the cells arm is above 0.9 at 10. Framed with the Coordinator:
     once this is merged, one workstation recipe for hardware counters a stage (no PMU here
     or on its machine) and for 1 against 10 pinned copies, the rule posted on #145 before
     any data (RULES C1a): cells ÷ production at 10 copies no more than at 1. Never run
     here.
   - What this container adds to that design: repeated runs at one count were 0.01 apart
     here, and two states half an hour apart 0.02 to 0.04. The legs should alternate and
     repeat, and a difference under the spread of the repeats decides nothing.
2. **A kernel that takes free steps first.** 1.279 r steps against 1.812 r (§5.6): the
   largest thing left in the count. Its first falsifier passed: "free" told from links
   alone gives 1.467 r and 1.302 r, under the 1.6 r set before the count.
   - **The test is not the obstacle (C).** A step is known free from links alone when the
     far end of its landing place is a coincidence (then the letters over the place are
     walked back from it). Counting only those as free: 1.467 r with the right end as the
     paid step (1.456 with the full test), 1.302 r with the largest cell (1.279).
   - **Nor is the step: a segment can be left after any step.** The boundary that survives
     a step is a coincidence (§5.6), so the winner's remainder can be written back as a
     letter (its length to its slot, a slot for what is left of its image: 5 stores) and
     the sweep can go elsewhere. A step out of order is then still §3.5's cell. (This
     entry's first reading, that it must be §3.1's step with both orders linked both ways,
     was wrong.)
   - **The going elsewhere is (C, 10 blocks).** In the order "free steps wherever ready,
     else the right end", with the test from links: 1.458 r steps, 0.33 r fewer than the
     sweep's 1.790 r on these blocks. But 3.28 r ends of segments are tested (2.25 a step),
     0.53 r are set aside to wait for a row to become a boundary, and two steps in three
     (0.96 r) are taken at another segment than the step before: the remainder written
     back, two letters read cold, the end's partner kept. At 64 instructions a cell, 10 a
     test and 25 a move (the last two estimated) that is 150 instructions a run against
     the sweep's 115, and as modelled nothing is fetched ahead (**Conj:** a queue of ready
     steps could be fetched from; not tried). **As counted and priced, a worklist does not
     pay on one thread.** The price is in instructions, which §6.2 showed did not bind the
     step: an estimate, not a clock. With the largest cell as the paid step (0.71 of the steps, not 0.80) it
     needs a priority queue besides.
   - **What would:** an order of the segments that a sweep can follow (every permutation of
     the segments is one: LF is an exchange of the same letters in it) in which the free
     steps come first of themselves. Whole segments in another order were counted in §5.6
     and gain 5 to 9% (equal steps first: 1.714 r by a sweep, 1.657 r by the referee's
     variant of geo, against 1.812 r); the 20 to 29% is in leaving segments half done.
3. **Feed from run records: done in the example (§6.3).** The order stage reads
   `runs::front`'s records (the 127-row cuts joined, 0.0778 → 0.0722 a row; the walk's row cut
   out). Left: the decoder's own records (`runs::Runs` keeps them private) behind
   `InverseBwt::invert_runs`, the seam the decoder calls after the front, as an experimental
   arm for a whole-decoder A/B. The Pathfinder's to wire or review, if the Coordinator
   agrees; the kernel would want an `unsafe fn` boundary, a bound on its buffer, and 4.
4. **Admission.** On Wikidata r/n < 0.10 (§6.3: 0.76 to 0.79× of production against 0.82 to
   0.86× unchosen), a threshold fitted and scored on the same 233 blocks. Two costs want two
   tests.
   - **The order stage is by the run, and is spent before any cell.** On uniform bytes it is
     13.4 ns a row alone, four times production. So r/n is tested first: the front has r,
     and the test is free. Where to put it off the corpus is not measured.
   - **The induction and the write are by the cell,** and S/r is not a constant (1.3 to 2.1
     on Wikidata, where four blocks in five lie in 1.73 to 1.91; 1.75 to 3.3 on the
     JSON-like inputs; n/2r on `(ab)^k a`). The kernel's whole time is 12.8 ns a cell on
     Wikidata and 11.5 on the JSON of 3,000 ids, against production's 2.0 ns a row:
     break-even S ≈ 0.16 to 0.17 n (**M**, one run each; 0.155 in the slower state of §6.3).
     S/n < 0.16 puts all ten other inputs of §6.3 on the right side of 1, and S/n ÷ 0.16
     prices five of them to 12% (against measured: 0.46 and 0.47, 0.92 and 0.85, 1.65 and
     1.69, 4.2 and 4.0, 6.1 and 5.4). It does not price the rest: Fibonacci, Thue–Morse and
     period 37, where the write is the cost (0.001, 0.005, 0.17 against 0.02, 0.03, 0.31),
     nor uniform bytes (6.3 against 8.9) and `(ab)^k a` (3.1 against 11.2), where a cell
     costs 25 to 30 ns.
   - **On Wikidata the two tests choose alike:** in one census S/n < 0.16, S known in
     hindsight, and r/n < 0.09 admit the same 196 blocks, 0.784 (r/n < 0.10: 0.786). They
     part off the corpus, where r/n alone admits a 5.2× loss (§6.4: random letters with
     geometric runs, r/n 0.083).
   - **So the guard is a budget on the cells, in rows:** S is known only as it is made;
     past 0.16 n, stop and walk. A budget in r is wrong: 4 r, this entry's first figure,
     would refuse Fibonacci (10.9 r), Thue–Morse (8.6 r) and period 37 (650 r), the largest
     wins.
   - **The guard is dear, and it leaks.** A block it refuses has by then cost about nine
     tenths of production's time on Wikidata (**M**, the 37 blocks past 0.16 n: order 0.61 ns
     a row and 0.16 n cells at 7.9 ns, against 2.10) and more off it (the JSON of 20k
     records 1.0, prose 1.35). And 0.16 n is the break-even of Wikidata and the JSON only:
     on `(ab)^k a` a cell costs 25 ns and production 1.1 ns a row, break-even 0.045 n, so a
     block of period 7 to about 20 (S ≈ n/p) would pass the budget and may lose up to 3.6×
     (**Conj:** from those prices; none was run). The division rule (5) is what closes that.
   - **So something cheaper must choose,** and the budget only guards: r/n on this corpus;
     in general a predictor of S/n from the records, not in hand (§6.4's U/n, what the
     forest's first level leaves, is a candidate; §5.5's closed form needs the walk). Not
     built.
5. **Division (Zorich) rule** for equal-pair streaks: under item 4's budget a block of period
   up to 6 is refused (S ≈ n/p > 0.16 n; `(ab)^k a` is 0.5 n) and one of longer period is
   admitted at its full count (period 37: 0.027 n, already 0.31×); with this rule each
   would be a handful of cells (**Conj:** not built). Known for the related accelerated algorithm on k pairings of
   [1, N]: at most 5k²(2 + log₂ N) cycles, each polynomial in k log N (Agol–Hass–Thurston
   2006, proof of Thm 12; it counts orbits and yields no word). The complexity
   sweep derives O(d log(n/d) · min(d, log n)) same-winner streaks for a d-letter discrete
   exchange (**Conj** here: not re-checked). On Wikidata the full-cycle acceleration gains
   nothing (no winner outlasts one turn of its losers); the pair rule is what periodic blocks
   need.
6. **Output passes on the grammar: the algebra is done (C, Python:**
   `2026-11-10-rauzy-veech/rle1_crc_on_grammar.py.txt`**).** RLE1's decoder is a transducer on
   (byte, count 0..4). A nonterminal's summary: its first run's byte and exact length, and two
   continuations (entered not expecting a count; entered expecting one), each a decoded
   length, (R, X) and an exit state. It composes in O(1) products a rule (≤ 24 multiplications
   mod P by the formulas, the powers of x tabulated; 38 to 48 without; the script counts
   none); 75,532 (word, entry state) pairs under random bracketings and 4,878 whole
   blocks equal the bytewise decoder and CRC. This entry's first guess, that the entry state
   dies at the first change of byte, was false: one bit survives, flipped by a run of length
   ≡ 4 mod 5, kept by ≡ 0, forgotten by 1, 2, 3 (`aaaabbbbb`). Not in the example. Ceiling 4% of
   a Wikidata decode (PF:`by-data-class` §1): low. Line offsets for `wikidata-pq` by the same
   morphism: uncounted.
7. **Rounds, threads.** The steps allow 10.6 to 16 times fewer turns at P = 16, a long merge's chain being the
   limit (§5.4), and rounds as array passes are
   closed as built on one thread (§5.4: 1.99×). What is open is the grain for threads: a segment is 1.8
   cells of 24 cycles, so segments must be handed out by the thousand, by stretches of rows
   whose letters do not cross. Uncounted: how many letters cross a cut of the rows into P
   parts. Low rank while blocks outnumber workers.
8. **The equal cells of one row, linked in the sweep and walked at the end** (§6.4): 0.970 r
   cells and 0.842 r points for 1.812 r cells. No pre-pass: the test is ra = rb = 1 where the
   cell would be made. Ceiling about 15% of the kernel, by a model. Falsifier: built into
   `induce_cells`, the whole must fall at least 5% on the 233 blocks in one process; dead if
   a link, its walk point and the extra write reach a cell's price. The chains joined in a
   pre-pass are closed twice over (§6.1, §6.4).
9. **Why row 0, and why the right end** (§5.5, §5.6): both are counted, neither is proved.
   Whether another order of the alphabet makes fewer cells.
10. **Smaller.** Slots of 16 bytes with |w| kept at the word: 33% off the largest region.

## 9. Frame (not used above)

- **P (elementary; checked on the same 4,897 words):** with τ(x) = x + 1 mod n and the product
  applied right to left, the commutator π τ π⁻¹ τ⁻¹: x ↦ π(π⁻¹(x − 1) + 1) fixes x unless
  x ∈ 𝓑, and it moves all of 𝓑 but at most σ points (bucket seams, the wrap). *Proof.* For
  x ∉ 𝓑, x − 1 and x lie in one image, so π⁻¹(x − 1) + 1 = π⁻¹x and x is fixed. For x ∈ 𝓑 the
  commutator sends x to the image start of the run that follows, on top, the run whose
  image ends at x − 1 (on top the last run is followed by the first; for x = 0, x − 1 is
  n − 1). That is x only where those two runs have neighbouring images. For x ≥ 1 the first
  is then the last run of its byte and the second the first run of the next byte: at most
  σ − 1 seams. x = 0 is one point more, fixed when the last run of the largest byte is
  followed by the first run of the smallest. Both occur together and σ is reached (T = abc,
  L = cab: π commutes with τ). ∎ So (τ, π) is a square-tiled surface: one horizontal cylinder (τ is an n-cycle),
  one vertical when π is an n-cycle (**K** as an object: pairs of n-cycles, Delecroix–Goujard–
  Zograf–Zorich 2020; Jeffreys 2021).
- **Its cone points are the commutator's cycles, not the runs** (the sweep's correction to this
  entry's first wording). **C:** on the 233 Wikidata blocks the r image boundaries fall into
  1 to 11 cycles (median 5; prose 4 to 11); genus = (moved − cycles)/2 + 1 ≈ r/2 (moved: the
  points the commutator moves, r less at most σ): 26,500 at
  r = 53,001. A block is a surface of genus r/2 with a handful of cone points of very large
  angle. 𝓣 ∩ 𝓑 ↔ vertical saddle connections of length one.
- **K** (T primitive)**:** SA conjugates π to the rotation p ↦ p − 1 and τ⁻¹ to φ (GNP 2020 Lemma 5.5;
  Navarro–Ochoa–Prezza 2021 Lemma 6): the dual r-IET in text space is the same surface turned a quarter. §5.5 is
  the induction read on that side.
- Rauzy–Veech induction is the first return of the Teichmüller flow; §5 says that on BWT
  surfaces it is dominated by collapsing unit saddle connections. Nothing computational is
  claimed from this section yet.

## 10. Prior art (searched 10 Nov by four sweeps; their reports are in the session record)

- **K:** LF as r translated intervals: GNP 2020 Lemma 5.2 (JACM 67(1) art. 2; Lemma 12 in
  arXiv:1809.02792v2, Lemma 18 in the 2017 preprint arXiv:1705.10382v4, whose numbers this
  entry first gave); Nishimoto–Tabei 2021, Lemma 3 and §3.3. Under the name IET, for
  clustering words: Ferenczi–Zamboni 2013, Lemma 3; multisets: Dolce–Hughes 2025.
- **K:** the step: Rauzy 1979. One step ↔ one binary morphism (an automorphism of the free
  group), for regular IETs: Jullian 2013; Dolce–Perrin 2017, Prop. 4.12 (numbering of
  arXiv:1305.0120v3). The equal case (their degenerate step, "merging"), for any standard
  IET: Dolce–Hughes, arXiv:2511.22588, Prop. 4.1 and 4.4. There also: a finite chain of
  right, left and splitting steps reaches the first-return map to any cylinder, periodic
  components included (Prop. 4.17), the return words being the composed step morphisms
  applied to the letters (Ex. 6.6); clustering multisets as discrete IETs (§3). No
  algorithm, no cost, no general BWT column there.
- **K:** a straight-line program for the crossing word of an integer-weighted exchange-like
  object (a normal curve by its normal coordinates; n triangles, X crossings).
  - Length and time O(n log X) for a *connected* curve, by quadratic word equations:
    Štefankovič 2005 (thesis, Lemma 3.4.2, as Erickson–Nayyeri cite it);
    Schaefer–Sedgwick–Štefankovič, CCCG 2008. Their COCOON 2002 paper gives polynomial size
    only.
  - Length and time **O(n² log X)** for any reduced curve, connected or not, by Euclid-style
    tracing with division (a spiral of depth d through m streets is O(m + log d) rules):
    Erickson–Nayyeri 2013, Thm 4.8, who note that the connected bound does not extend at
    once to disconnected curves.
  - No word: Agol–Hass–Thurston 2006 only count the orbits of k pairings of [1, N], in at
    most 5k²(2 + log₂ N) cycles, each polynomial in k log N (proof of Thm 12; commutative
    weights in Thm 16).
  - Iterating an integer IET n times in time polynomial in k, log N and log n (O(k² log N)
    by its proof, through Erickson–Nayyeri's routines): Bell, in Eppstein 2023 (TheoretiCS 2,
    art. 10), Thm 6.12.
  - The mapping of an RLBWT onto those objects is not in print; it would have to keep
    components, periods and the order of the word.
  - (This entry first put all of these under "an SLP of size O(k log N)", with Bell's
    theorem as 6.15. Corrected from the Researcher's dossier,
    `state/reports/rauzy-literature-6767025138/report.md` @ 79b50c1a, and a second check of
    each source.)
- **K:** inversion in parallel by list ranking on the n rows: Edwards–Vishkin 2012 (O(n)
  work, O(log n) depth, constant alphabet). An n-sized object; §5.4's rounds are r-sized.
- **K, the nearest claim to "copy, do not walk" for inversion:** Kärkkäinen–Puglisi 2011,
  "Cache friendly Burrows–Wheeler inversion": rows of one byte that are adjacent have
  adjacent images; a first pass records such chains and a second copies the text, one chain
  at a time (1.03 to 1.29× on ordinary text). They name several chains at once as open. Not
  an induction: the copies are found by walking. (Pointed out by the Pathfinder.)
- **K, the nearest machines:** surface-dynamics (Delecroix et al.), `int_iet.c`,
  `int_iet_num_cylinders`: induction on integer exchanges over doubly linked intervals, with
  a multiplicative step and heights added per loser; equal lengths remove a label. The same
  machine with heights where this has words; its `iet.py` (`rauzy_move(data=True)`) returns
  the tower substitution a → bc with the rule used here, for unequal lengths only (an equal
  step raises "does not handle degenerate situations": 55% of this entry's cells). Dunfield–Thurston 2006 §7.3–7.4: bands labelled in π₁,
  L(b') = L(b)·L(t) under a splitting, down to "a word representing γ"; and the homology
  labels in time polynomial in log|w|: one rule a step, and a morphism evaluated on it.
- **K:** RLBWT → LZ77 in O(n/log n + r log⁹ n + z log⁹ n): Kempa 2019 Thm 7.4 (the source of the
  record's uncited "O(n/log n + r polylog n)"; vacuous at n = 9·10⁵). Every RLBWT → LZ77
  or RLBWT → text route found takes Θ(n) steps of LF or of its inverse (Policriti–Prezza
  2018, O(n log r); Bannai–Gagie–I 2018; Nishimoto–Tabei, CPM 2019, for LZ77; Nishimoto–Tabei,
  ICALP 2021, Thm 11, for the text, by n moves of FL). GNP 2020's run-length grammar of size O(r log(n/r)) is for the differential
  LCP array, built from the plain array; their text structure is built by an LF-guided scan.
  For the text itself a run-length SLP of size O(r log(n/r)) exists (Navarro–Ochoa–Prezza
  2021, Thm 2 with Thm 7), built from the text; no construction from the run list is known.
- **K:** Precopy (Kärkkäinen–Kempa–Puglisi 2012): O(√(rn) + n/B) misses, with the n-entry LF
  array kept; read off its Table 3, it passes the 8-lane walk with a super-alphabet between n/r ≈ 50
  and 90. Wikidata is 13.9.
- **K, found after the grammar was counted (§5.6):** the Lyndon tree of a Lyndon word is the
  Cartesian tree, least value at the root, of its suffix ranks: Hohlweg–Reutenauer 2003
  (TCS 307(1):173–178; the right Lyndon tree). The Lyndon SLP: Tsuruta, Köppl, Nakashima,
  Inenaga, Bannai, Takeda 2020 (arXiv:2004.05309), built there from the text in O(n) time
  (their O(n lg n) expected is for the self-index on top of it). The Lyndon array as a
  by-product of walking LF, with a stack: Louza, Smyth, Manzini, Telles 2018 (JDA 50). The
  Lyndon SLP → the run-length BWT in O(N), with the remark that the Lyndon SLP has no bound
  in r (a^{n−1}b): Olbrich, ESA 2025 (arXiv:2504.19123). For r = 2 (conjugates of
  Christoffel words) §5.6 is classical: a Christoffel word's standard factorisation is into
  two Christoffel words (Borel–Laubie 1993; Berstel–de Luca 1997). Here the direction is
  RLBWT → tree, for any r: the tree's part above the run starts, made without the walk,
  gives the text; that was not found. Digram replacement at all occurrences: RePair
  (Larsson–Moffat 1999). That a digram's occurrences are one interval of rows is the
  standard fact that cylinders of an interval exchange are intervals (Keane 1975); its use
  as a RePair step at O(1) on run letters was not found.
- **Not found:** endpoint induction used to invert a general BWT column or to decompress an
  RLBWT; a count of S on any corpus; the segment/round structure; output passes on the
  induced grammar. The words "interval exchange", "Rauzy", "first return" do not occur in
  the r-index and move-structure papers read.
- **Found, for clustering words only** (each letter one run of the BWT, so the letters are
  the runs): LF as a discrete IET, and induction named as the faster way to make long
  clustering words (Ferenczi–Zamboni 2013, Lemma 3 and closing remark); a Euclid-like
  recursion on a symmetric discrete IET that counts its orbits, and perfectly clustering
  words from the free-group automorphisms of Rauzy moves (Lapointe, DMTCS 2019; WORDS 2021,
  seen through her slides); return words as composed step morphisms for any standard IET
  (Dolce–Hughes, Prop. 4.17, Ex. 6.6). None takes the runs of an arbitrary column as
  letters, states an algorithm or a cost, or counts anything. §5.2's (1, 1) chains are
  tunnelling blocks whose columns are whole runs (Baier, CPM 2018, Def. 8).
- **In the team's record:** first-return towers by piece-stepping, the grammar as a sentence
  (PF:`interval-exchange` §1; `chunked-link` §5a). Absent: endpoint induction, a built grammar,
  S, CRC on a structure (two readers of pr/313 and the 420-row register).
- **What is claimed as new, and no more:** the transfer to an arbitrary column (its runs as
  the letters, which makes any block a clustering word over its run alphabet; the induction
  run as an inversion; the two-pass write; the block's CRC from this grammar, the
  composition itself being the known fingerprint composition on SLPs; the RLE1 decoder
  composed over rules), the cells form with its marked row (§3.5), the counts of §4–§5, §5.5's reading of the step count as
  records of suffix ranks inside the BWT's phrases, and §5.6's: that this SLP is the Lyndon
  tree above the run starts, that a step is a digram at O(1), and steps = r₂ − 1 + U. The step, its morphism, the linked-interval machine and the SLP from accelerated
  induction are others'.

- **Added 10 Nov, from the second entry (`2026-11-10-lyndon-literature.md`, §3 and §5), which
  read the generalized-Lyndon literature against this one.** Three things claimed above are
  narrower than written.
  - The making of the Lyndon tree of the least rotation from the bottom, by merging on the
    circular word a largest word into its left neighbour, is the **Schützenberger–Melançon
    rewriting** (Melançon, JCTA 59, 1992; Reutenauer, Sém. Lothar. Combin. B54h, 2006, §3.4;
    Lyndon words as the Lazard set that removes the greatest element). Every right-end step
    is one of its legal rises. What is new is its run on the run-length column: LF's row
    order choosing the rise, a rise taken at all its occurrences at once in O(1), S steps,
    and the count.
  - §2's "column that is no BWT" is the inversion of the **extended and bijective BWT** by
    its cycles, largest first (Gil–Scott 2012; Kufleitner 2009, Theorem 8), in print with n
    steps. New: in S steps, each factor with its Lyndon tree.
  - That every letter's word is a Lyndon word follows from two lemmas on next and previous
    smaller suffixes (as Ellert, ESA 2022, §2 states them).
  And two things are sharper: Theorem 3 is a case of the second entry's Theorem 5 (for
  every order of right-handed steps, steps = the tree's inner nodes above a hinge), so no
  order that leaves the Lyndon tree is cheaper than the right end; and the left end of
  §5.6 is the right end for the inverse order of the alphabet.

## 11. Independent check (C; of the sub-agents' programs, `lyndon_cartesian.py.txt`, `schedules_geo.rs.txt` and `rle1_crc_on_grammar.py.txt` are beside this entry, the rest in the session's record)

A second implementation, written from the specification of §2 by a sub-agent that did not
see `examples/rauzy.rs`, in Python, with its own bzip2 parser.

- **Invariant I after every step, by brute force:** for every row of every letter, the true
  first return of LF to [0, m): time, itinerary, landing; the tracked point as
  P = π^v(start + o). 8,378 words (all binary to length 10, ternary to 7, random over 2 to 256
  letters, Fibonacci, Thue–Morse, near-periodic, powers, a^{n−1}b), 248,982 steps: no failure.
- **The harness can fail:** seven wrong variants of the rules (a concatenation swapped, insert
  before for after, the equal case left at the end, a level update dropped) all fail by n = 3.
- **Tracking from any row:** 30,124 (word, row) pairs; every tied row of a power v^k.
- **Real blocks:** all 435 of `wikidata.bz2` and `prose.bz2` decode through its grammar to
  bzip2's bytes; its S on the 233 full Wikidata blocks is 27,439,723, this entry's figure, and
  its split of block 1 is 24,961 / 16,426 / 51,183, the example's.
- **What it found:** the specification covered BWT images only. On 6,825 arbitrary columns the
  stop rule met A = B with letters left in 5,987; the drop rule of §2 is what works (44,584
  (L, P) pairs against the row walk). The example had that rule and had never run it; it is
  now exercised (`--arbitrary`, §2).
- **Also its:** the copies are short where they are many (on 20 Wikidata blocks, counting a
  repeated single byte as a copy: mean 16.2 bytes, median 3, 47% of copies ≤ 2 bytes; copies
  of ≥ 8 bytes carry 83.8% of the output); words built bottom-up would be 27.7 n bytes (the
  example builds none above 16 bytes); grammar depth 49 to 300 a block.

- **Two more, for §5.6,** by two further sub-agents with their own simulators. One checks
  the true first return (LF iterated) after every step of 29,480 random schedules on 3,685
  words, and bytes against `bwt::walk` on 396 runs of real blocks under 33 schedules. The
  other checks every Cartesian-tree node against its rule on every block, and a 61-bit hash
  of each schedule's last word against all rotations of the block. Each derived the floor
  r₂ − 1 and the dependence on the schedule without the other.

- **A review of this entry and of the cells kernel at ae51b11,** by five sub-agents, each
  told to break one thing. What each found is corrected above; this is the list.
  - *Memory safety* (every unchecked access of the kernel re-run with its region checked,
    19 million small columns, every block): the bounds hold. **The writer left bytes
    unwritten and the example's check could not fire** (§7 row 1); `copy16` overlapped.
  - *Citations* (24 sources read): the Researcher's correction of the SLP bounds holds;
    Bell's theorem is 6.12; Tsuruta et al. build the Lyndon SLP in O(n); the GNP lemma
    numbers were a preprint's; the novelty statement was too wide for clustering words
    (§10).
  - *Numbers* (every figure recomputed from the raw files): the counts reproduce; the
    core's clock was 15% off (§6); a ten-block sample is good to 10%, not "a few per cent";
    several figures mixed runs.
  - *Proofs* (its own Python of §2 and §3.5 from the text, 563,332 (column, row) pairs,
    25 million cells: the cells form makes exactly the reference's steps): the mathematics
    and the code hold; three sentences were false as written (the emission for a column
    that is no BWT; what a power closes; an independent set that holds both places of a
    letter), and §3.5 lacked the closing rule.
  - *A referee's reading* (the entry against itself, the example and its own runs: callgrind
    on three blocks, the committed simulator on all 233, exact powers, the regions' ends
    asserted): 23 findings. The sentence on powers was false under a P label; §0 had "the
    count stands at its floor" beside §5.6's floor; §5 had "almost all of its steps in the
    first few rounds" (51% in the first 4); the list kernel's step was called branch-free
    where the compiler keeps a branch; the per-run instruction counts were quoted from the
    most favourable block, and the gap is reversed by r/n 0.13 (§6.2); the admission
    threshold was fitted and scored on the same blocks, and set in r where the cost is in
    cells (§8, 4); valgrind is blind inside one allocation (§6.2); rows of §5.6's table have
    no program in the repository; §6.1's cycles stood at a clock already corrected; labels
    did not fit their sentences; two items of §8 did not render as items.
  - *A last check of those corrections* (three more sub-agents: every new figure recomputed
    from the raw files; the entry read whole for contradictions; the new statements tested
    by brute force): 55 findings, about a dozen marked errors. "Hours later" was 34 to 48
    minutes; a census set aside as disturbed had gone unmentioned (§6.3); "S/P turns" was a
    ceiling that a merge's chain does not allow (§5.4); the cost of a block the budget
    refuses had been taken from the blocks it admits (§8, 4); §9's proof needed its
    composition and its two wraps said. Checked and standing: the tracked point's three
    rules on every row of every primitive word to length 10 (binary) and 7 (ternary); the
    ratios and ranges of §6.3 from the raw files.

## Reproduce

```text
bash bench/corpus.sh                                   # corpora/wikidata.bz2
cargo build --release --example rauzy
target/release/examples/rauzy corpora/wikidata.bz2 1 65536 5 > out.jsonl
taskset -c 3 target/release/examples/rauzy corpora/wikidata.bz2 1 65536 5 > pinned.jsonl
target/release/examples/rauzy corpora/wikidata.bz2 24 65536 5    # 10 blocks; 12: 20; 8: 30
RAUZY_JOIN=1 target/release/examples/rauzy corpora/wikidata.bz2 1 65536 3 > join.jsonl
target/release/examples/rauzy --arbitrary 200000
valgrind -q --error-exitcode=9 target/release/examples/rauzy --arbitrary 3000
cargo build --profile fuzz --example rauzy && target/fuzz/examples/rauzy --arbitrary 20000
head -c 60000 corpora/wikidata.bz2 > block1.bz2        # block 1 whole: it is printed, then the cut panics
valgrind --tool=callgrind --cache-sim=yes --branch-sim=yes --LL=2097152,16,64 \
    target/release/examples/rauzy block1.bz2 1 65536 1
callgrind_annotate --inclusive=yes callgrind.out.PID   # divide by the calls it reports
```
Beside this entry, in `2026-11-10-rauzy-veech/` (each Python file runs as `python3 FILE`):
`identities.py.txt` (§5.1 and §9 by brute force), `rle1_crc_on_grammar.py.txt` (§8, 6),
`lyndon_cartesian.py.txt` (§5.6, Theorems 1 and 2), `synthetic.py.txt` (the fourteen synthetic
files of §4, §6 and §7, byte for byte, and how `prose.bz2` was made), `schedules_geo.rs.txt`
(§5.6, the orders of steps on real blocks: its first lines say how to run it) and
`clock_chain.rs.txt` (§6, the core's clock: `rustc -O`, then run).

**From `out.jsonl` to the tables.** A pooled figure is Σ (ns a row × n) / Σ n over the blocks,
and a ratio is the quotient of two pooled figures. A band is by `runs`/`n`. Production is
`ns_rec` where `takes`, else `ns_walk`. "Chosen by r/n < t" takes `c_whole` for the blocks
under t and production for the rest; hindsight takes the smaller, block by block. The
per-block figures are `c_whole` ÷ production, sorted, at index ⌊p (N − 1)⌋. A cycle is
1/3.15 ns.

**In no commit** (the session's record only): the kernels between the first and the fifth
row of §6.2's "What moved it"; the turned orders of §5.5; the rounds as passes of §5.4; the
sweeps' simulators other than `schedules_geo` and `lyndon_cartesian` (§5.6, §11); the
constructions of §6.4. **In an earlier commit:** §6.2's tables are the example at d1adf10
(`git checkout d1adf10 -- examples/rauzy.rs`: the cells fed from the run list, before the
writer's fix); §6.1's table is `ns_walk` and the `k_*` fields of any commit.

One JSON line a block: `n, runs, steps, top, bottom, equal, dropped, streaks, longest`
(maximal runs of steps with one winner, and the longest)`, left` (the letters left when the
walk's cycle closed)`, coincide` (row 0 not counted)`, shapes, repeats, divisions, depth,
widest, rounds, selfish, selfish_steps, copies, copied, deepest` (the reference writer's
deepest stack)`, hist` (rule lengths: count and bytes by class), `crc_muls`, `closed` (§5.5
asserted),
`cone` (points the commutator moves, its cycles), `par` (the turns the rounds would take with 2,
4, 8, 16 workers were a merge's steps divisible: a ceiling, §5.4), `half` (the rounds holding half the steps), `takes` (`runs::takes_runs`),
`records` (run records, cut at 127), `c_letters`, and ns a row: `ns_walk` (the column's
path: build + room walk), `ns_rec` (the records' path: fill + room walk backward, one span;
`ns_fill` its fill), `ns_mtf` and `ns_front` (the two fronts); the list kernel's `k_runs`
(the scan), `k_order`, `k_induce`, `k_write`, with `k_joined`, `k_steps`, `k_rules`; the cells
kernel's `c_order` (from the records), `c_induce`, `c_write`, `c_whole` (one span), with
`c_cells`; the reference's `ns_runs`, `ns_induce`, `ns_expand` (a rule a step, no arena).
The reference asserts bytes and CRC; both kernels and the records' path assert bytes, the
cells' with the output complemented first.

§6.3's runs are the pinned command; §6.2's second run is the same command on the example
at d1adf10. `rauzy --arbitrary`: columns that
are no BWT, the reference and both kernels against `bwt::walk`; the `fuzz` profile runs it
under debug assertions and overflow checks, the cells' regions' ends among them.
