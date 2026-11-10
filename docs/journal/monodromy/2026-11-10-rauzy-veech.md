# 2026-11-10 · MONODROMY 1 · LF's interval exchange, renormalised at its endpoint: a block is 1.81 r binary rules, and is written from them in 0.756 of the walk's time

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
- **K + C, what the grammar is (§5.6):** the Lyndon tree of the block's least rotation
  (Hohlweg–Reutenauer 2003: the Cartesian tree of its suffix ranks). Every letter's word, at
  every moment, is a Lyndon word. With equal rules shared it is the Lyndon SLP: 0.470 S =
  0.852 r rules.
- **P + C, what a step is (§5.6):** one digram of the text written in run letters, replaced
  at all its occurrences at once: they are one interval of rows. For every order of steps,
  steps = r₂ − 1 + (re-seatings that land on no boundary). The right end has 0.592 r of those;
  "steps that land on a boundary first, then the largest cell" has 0.059 r: **1.279 r steps,
  1.048 of the floor**. Counted, each run checked to end on the block; no kernel takes its
  steps in that order yet.
- **C:** the rounds are no limit on a schedule: half the steps lie in the first 4 rounds, and P
  workers taking a round's steps P at a time need S/P turns to within 1% up to P = 16 (§5.4).
- **M, the clock. The first construction lost; the second wins** (kernel from run records
  against `bwt::build` + the room walk, all 233 blocks, pooled, ns a row):
  - linked orders, a step that re-seats a letter (§3.1): 2.770 against 2.172, **1.276×** (§6.1);
  - **cells** (§3.5: a step's cell is a letter written once; nothing is moved; no point is
    followed): order 0.305 + induce 1.005 + write 0.363, 1.697 as one span, against 2.244:
    **0.756×**. 202 of 233 blocks faster (0.867 of rows). Per block 0.46 / 0.57 / 0.66 / 1.12 /
    1.82 (min, p10, median, p90, max). r/n < 0.06 (40% of rows): 0.618. r/n ≥ 0.12: 1.479.
    Chosen by r/n < 0.10, a number the front has before the inversion: **0.712×** (§6.2).
    A second run, pinned to an idle core: 1.604 against 2.172, **0.738×**, 204 blocks.
  - Per run of L: 184 instructions against production's 260; 2.1 simulated last-level misses
    (2 MiB) against 8.7.
- **Where else it wins (M, cells):** Fibonacci and Thue–Morse 0.02×; period 37 0.18×; JSON of
  300 ids (r/n 0.022) 0.38×; of 3,000 ids (0.057) 0.79×.
- **Where it must not be used:** JSON of 20k records (r/n 0.151) 1.76×; prose (0.367) 3.9×;
  uniform bytes 9.8×; `(ab)^k a` 8.3× (S = n/2, §2: needs the division rule or a step budget).
- **Status:** the count stands at its floor for this schedule (§4: at most 1.49× is left in
  it). The step went from 14 ns to 7.7: 64 instructions, a three-way branch, two slots and one
  word written, the scattered writes fetched ahead. One container, one thread, indicative
  (RULES C5): the i9, its ten workers and a front that emits run records are not measured
  (§8).

## 1. Objects

- L ∈ Σⁿ, rows [0, n), runs ρ₁ … ρ_r (maximal equal-byte blocks), λᵢ = |ρᵢ|, bᵢ its byte.
- π = LF: π(x) = C(L[x]) + #{y < x : L[y] = L[x]}. **K** (GNP 2020 Lemma 18; Nishimoto–Tabei's
  disjoint interval sequence; Ferenczi–Zamboni 2013 for clustering words): π is a translation on
  each ρᵢ. So π is a discrete interval exchange (IET) on r letters:
  - TOP order: ρ₁ … ρ_r as they lie in L (the domain's tiling);
  - BOTTOM order: the runs sorted stably by byte (the tiling by images π(ρᵢ)).
- Emission: x₀ = orig_ptr, x_{k+1} = π(x_k), L[x_k] = T[n − 1 − k]. (A column that is no BWT:
  the decoder still writes n bytes, round the cycle of orig_ptr, `bwt::walk`; the formula is
  read round that cycle.)
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
at one letter with λ = k, |w| = n/k (one step for `(ab)^k` itself). `(ab)^k a`, `(aaaab)^k`,
zeros: bytes exact (C, §4).
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
  n = 899,981). The same two letters stay last: q steps = one division, one rule w^q. See §7.

**The tracked point (P).** orig_ptr as (α, o, v): orig_ptr = π^v(start(I_α) + o), 0 ≤ v < |w_α|.
With |w_B| its value before the step:
- Top wins: if α = A and o ≥ λ_A − λ_B: α := B, o −= λ_A − λ_B, v += |w_B|.
- Bottom wins: if α = A: v += |w_B|. If α = B and o ≥ λ_B − λ_A: α := A, o −= λ_B − λ_A.
- Equal: if α = A: α := B, v += |w_B|.
At the end t₀ = v, and T[k] = E[(k − t₀) mod p], E the root's expansion, p its length.

**No point need be followed (P; C on 200,000 arbitrary columns and every block).** Cut the run
of orig_ptr so that the row is a letter Z of its own (at most two letters more), and mark
its word; a word made with a marked word is marked.
- A letter whose word is marked has one point: all its points share one itinerary through
  the original letters, and that itinerary passes Z, which is one row.
- So the walk's cycle is the closed letter whose word is marked, and Z's byte lies once in
  that word, at a place j read off the marked rules from the root down.
- By Invariant I a letter's word ends with the byte of its own row, and by the emission T
  ends with L[orig_ptr]. So **T is the cycle's word read from place j + 1 round**, n bytes.
- A block that is a power v^k closes k letters of one point each; the marked one is the
  walk's, the others are dropped.
The induction then carries no (α, o, v), and the writer starts at j + 1: nothing is turned
afterwards (§3.5).

## 3. Construction

**3.1 Induction.** Two doubly linked orders over r nodes (λ, |w|, w, prev₂, next₂), 32 bytes a
letter. A step: 2 lengths read, 1 rule, 5 or 6 links written. No rank, no select, no run map, no
n-entry table, no column: the input is (byte, length) per run, which #324's front emits.

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

**3.4 Homomorphisms in O(S) (P; K as a technique, e.g. Erickson–Nayyeri 2013 §5.1).** For any
monoid morphism h: Σ* → M, h(N) = h(left) h(right): S products, not n. CRC-32/BZIP2:
R(w) = w(x)·x³² mod P, X(w) = x^{8|w|} mod P, R(uv) = R(u)X(v) + R(v), X(uv) = X(u)X(v);
crc = ¬(R + ones·X). The rotation costs one descent from the root. **C:** 0.262 multiplications a
row; equal to `crc::crc32` of the walked bytes on all 233 blocks and every synthetic input
that is no power (for a power the example skips the check).
bzip2's stored CRC is over the RLE1-decoded bytes: the RLE1 decoder composes over rules too
(§8, 4), checked in Python beside this entry, not in the example. A CRC taken from the
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
  a cell writes two slots (three fields each; five on the side that goes on) and patches
  nothing. The winner's remainder is in registers only.
- **A cell is read once, on the side the front reaches first,** as A or as B; on the other
  side its slot is by then its own first cell.
- **The three cases are a three-way branch** on ra against rb (equal 55%, top 28%, bottom 17%
  of cells on Wikidata); the word is made before it, as bytes and as a rule, one of them kept.
- **One buffer,** every reference a 32-bit offset in it: slots, the words kept as bytes, the
  rules. A rule is its two factors as their slots held them, (|w|, w) twice: two stores.
- **Fetch ahead.** The slot 12 before the last in an order holds, as memory has it now, the
  place its letter's cells will be written and its word: both are fetched then. This is what
  makes the scattered writes free (§6.2).
- **No point is followed** (§2): the mark rides in |w|'s top bit, so a marked word is never
  kept as bytes and its rules are a path from the root to Z.
- **The write** walks that path once, places each unmarked factor where it lies from j + 1
  on, and then is §3.2's two passes, into the output itself.
- **Not done:** the division rule (a streak of one winner is one cell a turn here too); a
  bound on the buffer other than n cells; records from the front (§8).

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

**Shapes at the start (C;** Wikidata, 233 blocks; prose in brackets**):** of 0.780 r [0.541 r]
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
shortened a letter σ consumes. Longest chain, 233 blocks: **63 / 78 / 93 / 119 / 337** rounds
(min, p10, median, p90, max); median 606 segments a round. Prose: 79 to 118 rounds for
3·10⁵ segments; uniform bytes: 47; JSON of 20k records: 660 to 944. Same-round segments share
no letter (**P**: a shared letter orders them).
- **The rounds are no limit (C;** 30 blocks, every 8th**).** Half of a block's steps lie in its
  first 4 rounds (5 at most): the tail is long and thin. With P workers a round of w steps
  takes ⌈w/P⌉ turns; S over the turns is **2.00, 3.99, 7.97, 15.89** at P = 2, 4, 8, 16 (pooled;
  the worst block 15.82 at 16). This is a count of a schedule, with free hand-offs: what a
  thread pays to take a segment is not in it, and a segment is 1.8 steps.
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
  against the induction's three counts: equal on all 233 Wikidata blocks, 4 prose, 25
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

- **The count depends on where the rows begin (C;** 10 blocks, every 24th**; Conj for the
  reason).** LF is an exchange in the order of the rows turned to begin at any row p, on at
  most two letters more (the run of p and the image holding p are cut there), and the
  induction then ends on p's letter. Turned to orig_ptr it makes 0.98 to 1.20 times the cells
  of the order from row 0: +11% pooled. From row 0 the grammar is the Lyndon tree (§5.6);
  from another row it is the Cartesian tree of a rank order that is no lexicographic order.
  That this must make more cells is not proved. The left end makes more too (§5.6).

**5.6 What the grammar is; what a step is; the count is the schedule's.** Two sweeps of this
mission, each with its own simulator and neither using the example's code. Reproduced here:
`lyndon_cartesian.py.txt` (34,267 words) and `schedules_geo.rs.txt` (10 blocks; its right-end
count equals the example's S on each).

- **Theorem 1 (P; C on every node of every block).** T primitive, U its least rotation,
  key(j) = the rank of the rotation that starts at U[j], 0 < j < n. The derivation tree of
  the right-end grammar is the min-Cartesian tree of key, and the rule of step [m', m) sits
  at exactly the nodes with key in [m', m).
  *Proof.* By Invariant I read in the text (§5.5) the cuts at prefix m are the boundaries
  with key < m. A step removes the cuts with key in [m', m), each a junction w_A | w_B. No two
  of them share a piece: a junction's right piece ends at a cut with key in I_B, its left
  piece at one with key in [m', m) ⊂ I_A, and I_A ∩ I_B = ∅. So removing them together is
  removing them one at a time in decreasing key: the Cartesian tree. ∎
  **C:** 209,697,429 nodes (233 blocks) and 3,535,835 (prose): each node's two children and
  span are its rule's two symbols and length; no mismatch.
- **Theorem 2 (K + C).** That tree is the Lyndon tree of U, its standard factorisation
  taken recursively (Hohlweg–Reutenauer 2003: the Lyndon tree of a Lyndon word is the
  Cartesian tree of its suffix ranks; for a Lyndon word suffix order is rotation order).
  Every letter's word at every moment is a Lyndon word, every rule a standard factorisation
  (**C:** 468,657 rules on 34,267 words).
- **Corollary (P).** Equal words have equal trees, so the grammar with equal rules shared is
  the Lyndon SLP (Tsuruta et al. 2020). **C:** D = 12,904,656 distinct rules = **0.470 S =
  0.852 r** (per block 0.43 to 0.53 S; prose 0.303 S = 0.559 r); the strings compared byte for
  byte. For scale, on the same least rotations: LZ77 0.295 r phrases (longest previous
  factor and a byte; 0.385 r without the byte), batch RePair 0.530 r (rules and final
  sequence). S is 4.7 to 6.1 z: this grammar is cheap to make, not small.
- **Where the duplicates are (C).** 0.224 S repeat the rule made just before: one top letter
  taking in consecutive bottom letters that carry one word (61.6% of top-wins steps; the runs
  of a byte lie together on the bottom). 0.306 S repeat an earlier one. Two top-adjacent
  letters never carry one word (**P:** their orbits would stay adjacent through the run
  start that parts them).
- **Theorem 3 (P from §5.5; C).** Call the r run-start boundaries hinges. S = the Lyndon-tree
  nodes on the hinges' paths to the root. Equal steps are the nodes with a hinge on both
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
  picture: a stretch's first row is a bottom boundary and its last a top one; a step removes
  one end of a stretch and, for nothing, interior rows of others; U = the interior rows that
  ever become the first or last survivor of their stretch. The right end makes them the
  minima-so-far; an order that empties a stretch's interior before it touches the ends
  makes none.
- **S is not the schedule's invariant (C).** Smallest case: `abacb` (L = `bbcaa`, r = 3,
  r₂ = 4): right end 4 steps, left end 3. On Wikidata (all 233 blocks unless said; every run
  ends on a word checked to be the block, by bytes or by a 61-bit hash against all its
  rotations):

  | order of steps | steps / r | U / r |
  |---|--:|--:|
  | left end | 2.137 | 0.917 |
  | a random coincidence | 1.981 | 0.761 |
  | **right end** (this entry's) | **1.812** | 0.592 |
  | equal steps wherever ready, else right end | 1.714 | 0.494 |
  | free steps to exhaustion once, then right end (8 blocks) | 1.627 | 0.407 |
  | largest cell (RePair's choice) | 1.459 | 0.239 |
  | free steps wherever ready, else right end | 1.456 | 0.235 |
  | **free steps wherever ready, else the largest cell** | **1.279** | 0.059 |
  | free first, the paid step on the shortest stretch (8 blocks) | 1.265 | 0.045 |
  | floor, r₂ − 1 | 1.220 | 0 |

  Prose: right end 1.841, free then largest cell 1.521, floor 1.459. The left end is worse
  than the right on every block (1.08 to 1.27×); two-sided rules and orders by bucket are
  worse than the right end too (8 blocks: 1.84 to 2.13).
- **The free steps alone (C, 8 blocks):** done to exhaustion before anything else they are
  0.911 r steps (0.809 r in their first layer) and leave 0.176 r letters, 0.309 r cells and
  36.9% of the rows. A pass of free steps and then the right end is 1.627 r; only free and
  paid steps interleaved reach 1.3.
- **Below the floor (C).** Two letters with one word, adjacent in both orders, are one letter
  ("fusion"); it never arises at an end. `aaabaabbabb`: r = 10, least number of steps 8. On
  Wikidata it is worth 0.3%.
- **What this is worth on the clock: not known.** At the cells step's price, 0.53 r fewer
  steps would be 11 of the kernel's 61 cycles a run: 0.74 → about 0.60. But a step out of
  order must find the letter that ends at a given row, which the sweep never asks; both
  simulators keep a hash map by row. Open (§8, 1a).

**Consequence.** The serial chain of §3.1 is an artefact of working one coincidence. The
induction is ~10² rounds of independent merges, almost all of its steps in the first few;
lanes, or threads, are available at no change to the grammar. Not built: on one thread the
step turned out to be bound by memory and by what the compiler kept on the stack, not by
its chain (§6.2).

## 6. Clock (M; one thread, fastest of 5 rounds a block, Xeon "2.1 GHz", 2 MiB L2; indicative)

Arms. Production: `bwt::build` + `walk_segments_room` from the column. Kernel, from the run
list: **order** (the runs sorted by byte, both orders laid out: this exchange's "build"),
**induce**, **write**. The scan of the column into runs is apart: 0.484 ns a row here. #324's
front emits the records in place of the column's MTF stage (0.173 against 0.423 ns a row,
PF:`by-data-class` §2): a saving on the kernel's side that no figure below takes. Production
here is master's column path; #324's records path (closed, not on master) is the nearer
rival for a run-fed kernel and is not an arm.

**The core's clock.** A chain of dependent multiplications (3 cycles each) runs at 1.100 ns:
2.73 GHz, not the 2.10 the processor's name and its TSC give. §6.1's cycles are at 2.10 and
are 1.30 times too few; §6.2's are at 2.73.

### 6.1 The first construction: linked orders (§3.1). It loses.

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
- **The step is:** 14.0 ns, 66% of the kernel: the order surgery, branch-free, 152
  instructions and 31 stores a step (callgrind, block 1). It rises with r, 12.8 ns in the
  lowest band of r/n to 18.9 in the highest. §6.2 finds where it went.
- The kernel's figure is the sum of its stages' fastest rounds; timed as one span it is
  about 1.2% more (1.29).
- **The budget, in cycles a run** (2.1 GHz, r/n 0.0722): production's build + walk is 63 here
  (the record: 72 on its VM, 110 on the i9). The kernel is 81: order 15, induce 53 (1.81 steps
  × 29), write 12. Parity alone needs induce ≤ 1.228 ns a row with order and write as they
  are (it is 1.826). Halving production here needs ≤ 32 cycles a run all told. At n/r = 13.9 the row
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

### 6.2 The second construction: cells (§3.5). It wins.

Same arms, same blocks, a later run (production 2.244 in it; the list kernel 2.808, 1.251×).
"Whole" is order + induce + write as one timed span; the stages are their own fastest rounds.

| Wikidata, 233 blocks, ns a row | build + walk | order | induce | write | whole | ÷ | lists ÷ |
|---|--:|--:|--:|--:|--:|--:|--:|
| pooled | 2.244 | 0.305 | 1.005 | 0.363 | 1.697 | **0.756** | 1.251 |
| r/n < 0.06 (93 blocks, 0.399 of rows) | 2.198 | 0.240 | 0.778 | 0.315 | 1.358 | 0.618 | 0.986 |
| 0.06–0.07 (71, 0.305) | 2.292 | 0.274 | 0.892 | 0.344 | 1.534 | 0.669 | 1.065 |
| 0.07–0.09 (32, 0.137) | 2.240 | 0.307 | 1.009 | 0.358 | 1.700 | 0.759 | 1.275 |
| 0.09–0.12 (19, 0.082) | 2.266 | 0.442 | 1.462 | 0.460 | 2.379 | 1.050 | 1.803 |
| ≥ 0.12 (18, 0.077) | 2.277 | 0.609 | 2.128 | 0.602 | 3.367 | 1.479 | 2.691 |

- 202 of 233 blocks faster, 0.867 of rows. Per block: 0.460 / 0.570 / 0.662 / 1.120 / 1.823.
- **Chosen by r/n** (the front has r before the inversion): cells below 0.10, else the walk:
  1.597 ns a row, **0.712×**; any threshold from 0.08 to 0.12 is within 0.007 of it; the
  better arm chosen block by block with hindsight is 0.711.
- **A second run,** the process pinned to one core and the others idle (during the first,
  other work ran on the machine's other cores): 1.604 against 2.172, **0.738×** (order 0.286,
  induce 0.947, write 0.344); 204 of 233 blocks faster; per block 0.39 / 0.55 / 0.65 / 1.05 /
  2.37; by r/n < 0.10: 0.702; the list kernel 1.199. The table is the first run.
- Cells: 27,439,756, the reference's S and 33 more (the walk's row cut out, 233 blocks).
- **A cell is 7.68 ns = 21 cycles, 64 instructions.** A run: 64 cycles (order 11.5, induce 38.0,
  write 13.7) against production's 85.

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

**What moved it** (10 blocks, every 24th, the same each time; production 2.14 to 2.34 between
runs, so the last column is good to a few per cent; instructions from callgrind on block 1):

| the step | instructions a cell | ns a cell | whole ÷ walk |
|---|--:|--:|--:|
| lists (§6.1) | 152 | 14.0 | 1.29 to 1.40 |
| cells; no branch in the step; slots as three 64-bit fields, two patched a cell; a point followed | 161 | 13.4 | 1.225 |
| one buffer; a three-way branch; no patch (the promise); the slot before read with the letter | 91 | 12.5 | 1.063 |
| no point followed: the walk's row marked, the block written unrotated | 62 | 9.8 | 0.936 |
| fetch ahead, 8 to 16 slots (12 kept) | 64 | 7.2 to 7.7 | 0.78 to 0.81 |
| *(orders turned to orig_ptr in place of the mark: +11% cells, §5.5)* | 73 | 10.2 | 1.038 |

- **The first two rows cost the same and were not bound by their instructions.** 161 → 91
  bought 7%. The compiler had made the followed point 15 branch-free instructions a cell
  with their operands on the stack, and had put the slot-before of both letters there too:
  the chain from a letter to the next ran through a store and a reload. Removing the point
  (§2) removed the chain: 91 → 62 instructions, 12.5 → 9.8 ns.
- **Then it was memory.** A cell is written to two places a counting sort apart. Fetching
  them when the letter is 12 slots from the front: 9.8 → 7.2 ns. What is left of the rise
  with the block's runs (all 233 blocks): 7.3 to 7.5 ns a cell for r below 7·10⁴, 7.7 to 10⁵,
  8.9 above.
- **Per run of L (callgrind, block 1, r/n 0.057):**

  | | instructions | simulated misses of a 2 MiB last level |
  |---|--:|--:|
  | production: build 85 + room walk 175 | 260 | 8.7 |
  | cells: order 27 + induce 1.81 × 64 + write 41 | 184 | 2.1 |

  The predictor callgrind models misses 0.20 branches a cell.
- **Live set:** slots 24 B × (2 r + the cells that took a fresh one, 0.81 r) = 67 B a run; words
  as bytes 0.47 n; rules 16 B × 0.54 r; a position a rule; the output. About 5.5 MB and the
  output at n = 9·10⁵, r = 6.5·10⁴: more than production's table. What differs is how it is
  touched: two forward streams and writes that are fetched ahead.
- **Tried, not kept: one word for consecutive bottom letters that carry one word** (§5.6:
  17% of cells need no new word). Two runs each way on 20 blocks: write 0.335 → 0.32, induce
  0.91 → 0.92 to 0.96, whole 1.545 → 1.545 to 1.59. The word was not the cell's cost.
- **Memory safety:** the kernel is `unsafe` (unchecked offsets in one buffer sized for n
  cells). valgrind: 3,000 arbitrary columns and three real blocks, no error. Not reviewed.
- **Not measured:** the i9; ten workers on a shared 20 MB last level, where production's walk
  is the known sufferer and the model above says this is not; records from the front
  instead of the scan; a decoder that calls it.

## 7. Falsifiers run

| # | attack | outcome |
|--:|---|---|
| 1 | bytes ≠ walk on any block | none, for the reference and both kernels: 233 Wikidata blocks (and the corpus's 198 tiny ones in the independent check), 4 prose, 28 synthetic incl. periodic and n < 20; 200,000 arbitrary columns |
| 2 | S ≫ r on natural data | no: 1.29–2.08 r |
| 3 | S = Θ(n) on some input | **yes**: `(ab)^k a`, S = n/2, 8× slower. Fix: division rule (q equal-pair steps → one power rule), or a step budget with fallback to the walk. Wikidata: 46 such steps in 27.4 M |
| 4 | text | **fires**: S/n 0.68, 3.9× slower (cells; 7.4× lists). Closed for r/n ≳ 0.10 at this step price |
| 5 | the clock on Wikidata | **fired for the linked orders**: 1.276×. **Does not fire for the cells**: 0.756×, 202 of 233 blocks (§6.2) |
| 5a | the orders turned to orig_ptr, to follow no point | **fires**: +11% cells (§5.5). The mark replaces it |
| 5b | "S is what the block costs, whatever the order of steps" (this entry, before §5.6) | **false**: 1.279 r to 2.137 r by the order; `abacb` already. The floor is r₂ − 1, not r − 1 |
| 6 | bulk-join the exact chains first (the cheap 63.5%) | **fires as built**: 1.668×; letter-passes at 10 ns each (§6) |
| 7 | prior art | the mechanism is classical; the transfer is not in print (§10) |
| 8 | independent implementation from the spec, invariant checked by brute force | no counterexample (§11) |

## 8. Open, ranked: ceiling · cheapest falsifier · gate

1. **The i9, and its ten workers.** Here: 0.756× on one thread, 184 instructions a run against
   260, a quarter of the modelled last-level misses. Production's inversion is 110 cycles a
   run there (PF:`phrase-structure` §5) against 85 here, on a 256 KiB L2 and 2 MB of last
   level a worker. **Conj:** the gap widens there. Falsifier: one diagnostic run of this
   example's two arms on the workstation, at 1 and at 10 copies; dead if the cells arm is
   above 0.9 at 10. A recipe and its rule are the Coordinator's and the Pathfinder's to
   frame (PROTOCOL §9, RULES C1a); never run here.
1a. **A kernel that takes free steps first.** 1.279 r steps against 1.812 r (§5.6): the
   largest thing left in the count. It needs, at a step out of order, the letter that ends
   where the loser lands; with each slot holding the slot of the other order that ends at
   its row, the test is one comparison of lengths when the landing place is a coincidence,
   and unknown otherwise (then wait, or take the right end). Uncounted: how many free steps
   that weaker test finds; what a step fetched out of order costs. Falsifier: that count,
   from `schedules_geo.rs.txt` with the test weakened; dead if above 1.6 r.
2. **Feed from run records** (#324 / #348, `u16` records cut at 127: merge neighbours, 0.0779 →
   0.0722 a row). Removes the column and the scan, and is where the kernel's 0.25 ns a row of
   front saving would be taken. A port; the front must cut the walk's row out (§2).
2a. **Admission.** r/n < 0.10 (§6.2), and a budget on the cells (S is known only as it is
   made; S/r is 1.3 to 2.1 on Wikidata and n/2r on `(ab)^k a`): past, say, 4 r, stop and walk.
   The work thrown away is bounded by the budget. Not built.
3. **Division (Zorich) rule** for equal-pair streaks, needed before any admission (or a step
   budget with fallback). Known for the related accelerated algorithm on k pairings of
   [1, N]: at most 5k²(2 + log₂ N) cycles (Agol–Hass–Thurston 2006, Thm 12). The complexity
   sweep derives O(d log(n/d) · min(d, log n)) same-winner streaks for a d-letter discrete
   exchange (**Conj** here: not re-checked). On Wikidata the full-cycle acceleration gains
   nothing (no winner outlasts one turn of its losers); the pair rule is what periodic blocks
   need.
4. **Output passes on the grammar: the algebra is done (C, Python:**
   `2026-11-10-rauzy-veech/rle1_crc_on_grammar.py.txt`**).** RLE1's decoder is a transducer on
   (byte, count 0..4). A nonterminal's summary: its first run's byte and exact length, and two
   continuations (entered not expecting a count; entered expecting one), each a decoded
   length, (R, X) and an exit state. It composes in O(1) products a rule (≤ 24 multiplications
   mod P as counted, the powers of x tabulated; 38 to 48 without); 75,532 (word, entry state) pairs under random bracketings and 4,878 whole
   blocks equal the bytewise decoder and CRC. This entry's first guess, that the entry state
   dies at the first change of byte, was false: one bit survives, flipped by a run of length
   ≡ 4 mod 5, kept by ≡ 0, forgotten by 1, 2, 3 (`aaaabbbbb`). Not in the example. Ceiling 4% of
   a Wikidata decode (PF:`by-data-class` §1): low. Line offsets for `wikidata-pq` by the same
   morphism: uncounted.
5. **Rounds, threads.** The schedule's ceiling is P (§5.4). What is open is the grain: a
   segment is 1.8 cells of 21 cycles, so segments must be handed out by the thousand, by
   stretches of rows whose letters do not cross. Uncounted: how many letters cross a cut of
   the rows into P parts. Low rank while blocks outnumber workers.
6. **The chains.** 0.635 r cells are whole runs onto whole runs (§5.2); a cell is 64
   instructions, a link in a chain walk about 15. Ceiling ≈ 15% of the kernel, and 5% of the
   steps besides (§5.6: equal steps first, 1.714 r). §6.1's attempt failed on its passes; in
   the cells form a chain needs no renumbering (its head's slot and its tail's slot are the
   letter's). Subsumed by 1a if that is built.
7. **Why row 0, and why the right end** (§5.5, §5.6): both are counted, neither is proved.
   Whether another order of the alphabet makes fewer cells.
8. **Smaller.** Slots of 16 bytes with |w| kept at the word: 33% off the largest region.

## 9. Frame (not used above)

- **P (elementary; checked on the same 4,897 words):** with τ(x) = x + 1 mod n, the commutator
  π τ π⁻¹ τ⁻¹ fixes x unless x ∈ 𝓑, and it moves all of 𝓑 but at most σ points (bucket seams,
  the wrap). So (τ, π) is a square-tiled surface: one horizontal cylinder (τ is an n-cycle),
  one vertical when π is an n-cycle (**K** as an object: pairs of n-cycles, Delecroix–Goujard–
  Zograf–Zorich 2020; Jeffreys 2021).
- **Its cone points are the commutator's cycles, not the runs** (the sweep's correction to this
  entry's first wording). **C:** on the 233 Wikidata blocks the r image boundaries fall into
  1 to 11 cycles (median 5; prose 4 to 11); genus = (moved − cycles)/2 + 1 ≈ r/2 (moved: the
  points the commutator moves, r less at most σ): 26,500 at
  r = 53,001. A block is a surface of genus r/2 with a handful of cone points of very large
  angle. 𝓣 ∩ 𝓑 ↔ vertical saddle connections of length one.
- **K** (T primitive)**:** SA conjugates π to the rotation p ↦ p − 1 and τ⁻¹ to φ (GNP Lemma 19;
  Navarro–Ochoa–Prezza Lemma 6): the dual r-IET in text space is the same surface turned a quarter. §5.5 is
  the induction read on that side.
- Rauzy–Veech induction is the first return of the Teichmüller flow; §5 says that on BWT
  surfaces it is dominated by collapsing unit saddle connections. Nothing computational is
  claimed from this section yet.

## 10. Prior art (searched 10 Nov by four sweeps; their reports are in the session record)

- **K:** LF as r translated intervals: GNP 2020 Lemma 18; Nishimoto–Tabei 2021. Under the name
  IET, for clustering words: Ferenczi–Zamboni 2013; multisets: Dolce–Hughes 2025.
- **K:** one Rauzy step ↔ one binary morphism: Rauzy 1979; Jullian 2013; Dolce–Perrin 2017
  Prop. 4.12; the equal ("merging") case on discrete IETs: Dolce–Hughes, arXiv:2511.22588,
  Prop. 4.1. No algorithm or cost there.
- **K:** an SLP of size O(k log N) for an integer-weighted exchange-like object by Euclid-style
  induction with division: Agol–Hass–Thurston 2006 (orbit counting); Schaefer–Sedgwick–
  Stefankovič 2002; Erickson–Nayyeri 2013 Thm 4.8; iterating an integer IET in O(k² log N):
  Bell, in Eppstein 2023 Thm 6.15. The mapping of an RLBWT onto those objects is not in print.
- **K, the nearest machines:** surface-dynamics (Delecroix et al.), `int_iet.c`,
  `int_iet_num_cylinders`: induction on integer exchanges over doubly linked intervals, with
  a multiplicative step and heights added per loser; equal lengths remove a label. The same
  machine with heights where this has words; its `iet.py` returns the tower substitution
  a → bc with the rule used here. Dunfield–Thurston 2006 §7.3–7.4: bands labelled in π₁,
  L(b') = L(b)·L(t) under a splitting, down to "a word representing γ"; and the homology
  labels in time polynomial in log|w|: one rule a step, and a morphism evaluated on it.
- **K:** RLBWT → LZ77 in O(n/log n + r log⁹ n + z log⁹ n): Kempa 2019 Thm 7.4 (the source of the
  record's uncited "O(n/log n + r polylog n)"; vacuous at n = 9·10⁵). Every implemented
  RLBWT → LZ77 or RLBWT → text route takes Θ(n) LF steps (Policriti–Prezza; Bannai–Gagie–I;
  Nishimoto–Tabei). GNP 2020's run-length grammar of size O(r log(n/r)) is for the differential
  LCP array, built from the plain array; their text structure is built by an LF-guided scan.
- **K:** Precopy (Kärkkäinen–Kempa–Puglisi 2012): O(√(rn) + n/B) misses, with the n-entry LF
  array kept; its published crossover against the lane walk is n/r ≈ 50–90. Wikidata is 13.9.
- **K, found after the grammar was counted (§5.6):** the Lyndon tree of a Lyndon word is the
  Cartesian tree of its suffix ranks: Hohlweg–Reutenauer 2003 (TCS 307). The Lyndon SLP:
  Tsuruta, Köppl, Nakashima, Inenaga, Bannai, Takeda 2020 (arXiv:2004.05309), built there
  from the text in O(n lg n) expected time. The Lyndon array as a by-product of walking LF,
  with a stack: Louza, Smyth, Manzini, Telles 2018 (JDA 50). Here the direction is the other
  one: the tree's part above the run starts, made without the walk, gives the text. Digram
  replacement at all occurrences: RePair (Larsson–Moffat 1999); that on run letters a
  digram's occurrences are one interval of rows was not found stated.
- **Not found:** endpoint induction as a BWT inversion or RLBWT decompression; a count of S on
  any corpus; the coincidence/segment/round structure; output passes on the induced grammar.
  The words "interval exchange", "Rauzy", "first return" do not occur in the r-index and
  move-structure papers read.
- **In the team's record:** first-return towers by piece-stepping, the grammar as a sentence
  (PF:`interval-exchange` §1; `chunked-link` §5a). Absent: endpoint induction, a built grammar,
  S, CRC on a structure (two readers of pr/313 and the 420-row register).
- **What is claimed as new, and no more:** the transfer (runs of L as the letters, the block
  as the induction's SLP, the two-pass write, the CRC and RLE1 by morphism), the cells form
  with its marked row (§3.5), the counts of §4–§5, §5.5's reading of the step count as
  records of suffix ranks inside the BWT's phrases, and §5.6's: that this SLP is the Lyndon
  tree above the run starts, that a step is a digram at O(1), and steps = r₂ − 1 + U. The step, its morphism, the linked-interval machine and the SLP from accelerated
  induction are others'.

## 11. Independent check (C)

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

## Reproduce

```text
bash bench/corpus.sh                                   # corpora/wikidata.bz2
cargo build --release --example rauzy
target/release/examples/rauzy corpora/wikidata.bz2 1 65536 5 > out.jsonl
RAUZY_JOIN=1 target/release/examples/rauzy corpora/wikidata.bz2 1 65536 3 > join.jsonl
```
Beside this entry: `2026-11-10-rauzy-veech/identities.py.txt` (§5.1 and §9 by brute force),
`rle1_crc_on_grammar.py.txt` (§8, 4) and `lyndon_cartesian.py.txt` (§5.6, Theorems 1 and 2);
each runs as `python3 FILE`. `schedules_geo.rs.txt` (§5.6, the orders of steps on real
blocks): its first lines say how to run it.

One JSON line a block: `n, runs, steps, top, bottom, equal, dropped, coincide` (row 0 not
counted)`, shapes, repeats, divisions, depth, widest, rounds, selfish, selfish_steps, copies,
copied, hist` (rule lengths: count and bytes by class), `crc_muls`, `closed` (§5.5 asserted),
`cone` (points the commutator moves, its cycles), `par` (the turns the rounds take with 2, 4,
8, 16 workers), `half` (the rounds holding half the steps), and ns a row: `ns_walk` (build +
room walk); the list kernel's `k_runs` (the scan), `k_order`, `k_induce`, `k_write`, with
`k_joined`, `k_steps`, `k_rules`; the cells kernel's `c_order`, `c_induce`, `c_write`, `c_whole`
(one span), with `c_cells`; the reference's `ns_runs`, `ns_induce`, `ns_expand` (a rule a step,
no arena). The reference asserts bytes and CRC; both kernels assert bytes.

`rauzy --arbitrary 200000`: columns that are no BWT, the reference and both kernels against
`bwt::walk`.
