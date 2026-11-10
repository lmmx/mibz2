# 2026-11-10 · MONODROMY 1 · LF's interval exchange, renormalised at its endpoint: a block is 1.81 r binary rules

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
- **M, the clock, and it loses:** kernel from run records (order 0.525 + induce 1.826 + write
  0.419 = 2.770 ns a row) against build + room walk 2.172: **1.276×**, pooled. Parity (0.99)
  on blocks with r/n < 0.06 (40% of rows); 2.6× at r/n ≥ 0.12. 62 of 233 blocks faster.
  A run costs production 63 cycles here and the kernel 81.
- **Where it wins as built (M):** Fibonacci and Thue–Morse 0.05×; period 37 0.27×; JSON of 300
  ids (r/n 0.022) 0.60×.
- **Where it must not be used:** prose (r/n 0.367) 7.4×; `(ab)^k` 7.8× (S = n/2, §2).
- **Status:** the count is a floor-level result (§4: at most 1.81× is left in it). The
  construction is not competitive on Wikidata. The price of a step, 14 ns ≈ 6.4 row-steps, is
  the whole question; §5 shows the step need not be serial; §6 shows the first attempt to
  exploit that failed, and why.

## 1. Objects

- L ∈ Σⁿ, rows [0, n), runs ρ₁ … ρ_r (maximal equal-byte blocks), λᵢ = |ρᵢ|, bᵢ its byte.
- π = LF: π(x) = C(L[x]) + #{y < x : L[y] = L[x]}. **K** (GNP 2020 Lemma 18; Nishimoto–Tabei's
  disjoint interval sequence; Ferenczi–Zamboni 2013 for clustering words): π is a translation on
  each ρᵢ. So π is a discrete interval exchange (IET) on r letters:
  - TOP order: ρ₁ … ρ_r as they lie in L (the domain's tiling);
  - BOTTOM order: the runs sorted stably by byte (the tiling by images π(ρᵢ)).
- Emission: x₀ = orig_ptr, x_{k+1} = π(x_k), L[x_k] = T[n − 1 − k].
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
T = v^k (bzip2 sorts rotations, k cycles): A = B with |w_A| = n/k; letters whose cycles the walk
is not on are dropped. `(ab)^k`, `(aaaab)^k`, zeros: bytes exact (C, §4).

**Counts (P).**
- Equal steps = r − 1 for one cycle: only they delete letters. **C:** 15,141,369 = 15,141,602 − 233.
- r − 1 ≤ S ≤ n − 1: a step deletes at most one letter, and removes at least one point.
- S is not O(r polylog) as written: `(ab)^k` has r = 2, S = n/2 (**C**: 449,991 of n = 899,981).
  The same two letters stay last: q steps = one division, one rule w^q. See §7.

**The tracked point (P).** orig_ptr as (α, o, v): orig_ptr = π^v(start(I_α) + o), 0 ≤ v < |w_α|.
- Top wins: if α = A and o ≥ λ_A − λ_B: α := B, o −= λ_A − λ_B, v += |w_B|.
- Bottom wins: if α = A: v += |w_B|. If α = B and o ≥ λ_B − λ_A: α := A, o −= λ_B − λ_A.
- Equal: if α = A: α := B, v += |w_B|.
At the end t₀ = v, and T[k] = E[(k − t₀) mod p], E the root's expansion, p its length.

## 3. Construction

**3.1 Induction.** Two doubly linked orders over r nodes (λ, |w|, w, prev₂, next₂), 32 bytes a
letter. A step: 2 lengths read, 1 rule, ≤ 4 links written. No rank, no select, no run map, no
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
The record's first-visit copy walk counts the same object (PF:`2026-11-09-phrase-structure.md`,
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
row; equal to `crc::crc32` of the walked bytes on all 233 blocks and every synthetic input.
bzip2's stored CRC is over the RLE1-decoded bytes: the RLE1 decoder composes over rules too
(§8, 4), checked in Python beside this entry, not in the example. A CRC taken from the
grammar verifies the input; it no longer checks the passes that write the bytes.

## 4. Counted (C)

`rauzy FILE 1 65536 5`: every block ≥ 65,536 rows.

| input | blocks | r/n | S/r | S/n | coincidences/r | rounds (median) |
|---|--:|--:|--:|--:|--:|--:|
| `wikidata.bz2` | 233 | 0.0722 | 1.812 | 0.1309 | 0.780 | 93 |
| prose (the repository's `.md`, 3.5 MB, `bzip2 -9`) | 4 | 0.3672 | 1.841 | 0.6761 | 0.541 | 101 |

Synthetic (blocks ≥ 10⁵ rows):

| input | n | r | S | S/n | note |
|---|--:|--:|--:|--:|---|
| Fibonacci word | 899,982 | 16 | 175 | 0.0002 | |
| Thue–Morse 2²⁰ (2 blocks) | 1,048,576 | 102 | 875 | 0.0008 | |
| period 37 | 888,001 | 37 | 24,044 | 0.027 | 23,998 steps are one division |
| `(ab)^k` | 899,981 | 2 | 449,991 | 0.500 | one division |
| `(abac)^k x`, `(abcabd)^k x` | 899,981 | 4, 6 | n/4, n/6 | | one division each |
| JSON-like, 300 ids (2 blocks) | 1,560,097 | 34,903 | 115,983 | 0.074 | S/r 3.32 |
| JSON-like, 3,000 ids (2 blocks) | 1,584,100 | 89,712 | 233,383 | 0.147 | S/r 2.60 |
| JSON-like, 20k records (4 blocks) | 2,850,814 | 430,046 | 751,520 | 0.264 | S/r 1.75 |
| 4 letters, uniform | 899,981 | 652,500 | 875,635 | 0.973 | S/r 1.34 |
| bytes, uniform | 899,981 | 896,440 | 899,976 | 1.000 | S/r 1.00 |

- **S/r is 1.8 on both natural corpora, five-fold apart in r/n.** Not a law: 1.0 (random) to
  3.3 (few-id JSON) to ~10 (Sturmian-like, where S is tiny anyway). §5.5 says what it is.
- **Floor (P):** S ≥ r − 1 for any schedule of these steps. So at most 1.81× is left in the count
  on Wikidata. What is undecided is the step's price.
- Against the record: the tower 8 × 3 is 0.190 n dependent steps on Wikidata and stops at
  W = n/512 with words materialised at each level (0.68–1.58 n bytes); this is 0.131 n, stops at
  one point, materialises 0.47 n bytes of small words. PF:`2026-11-10-chunked-link.md` §5a's bound
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
**S = Σ over the r − 1 segments consumed of (cells in it when consumed).**

**Shapes at the start (C;** Wikidata, 233 blocks; prose in brackets**):** of 0.780 r [0.541 r]
segments,
- one run onto one run exactly, (1, 1): **0.635 r** [0.324 r];
- one run tiled by several images, (1, b): 0.077 r [0.106 r];
- one image tiling several runs, (t, 1): 0.054 r [0.070 r];
- both sides plural: 0.014 r [0.042 r].

On Wikidata LF sends 63.5% of the runs exactly onto another run: the exchange is mostly a
permutation of runs (the record's tunnels, as a count of runs), and only 1.4% of its segments
need a two-sided merge.

**5.3 (P) Coincidences persist.** A step removes an interval that is one set of points in both
tilings and contains no top boundary inside; every other coincidence survives. Hence a
Rauzy step is valid at *any* coincidence, not only at m: letters A (top interval ending
there) and B (image ending there), same table. Steps on disjoint letters commute.

**5.4 (C) The induction's dependency order.** Segment σ needs segment σ' iff σ' re-seated or
shortened a letter σ consumes. Longest chain, 233 blocks: **63 / 78 / 93 / 119 / 337** rounds
(min, p10, median, p90, max); median 606 segments a round. Prose: 79 to 118 rounds for
3·10⁵ segments; uniform bytes: 47. Same-round segments share no letter (**P**: a shared letter
orders them).
- A segment that meets a letter it re-seated itself is a genuine Euclid stretch: **84 of
  15,141,369** (1,901 steps of 27,439,723). On real blocks the Euclidean part is absent.
- Counted a step at a time (a step needs the last step on either letter and the one before it
  in its segment): depth median 3,861; widest segment 1,562 steps (one long run absorbing its
  predecessors, itself a prefix sum).

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
  cuts. A step uncuts the suffixes of rank in [m', m), which are all the places where one
  piece w_A meets one piece w_B, and names the joined piece. So the grammar is a pair-merging
  of the text's positions in decreasing order of suffix rank: the largest suffixes lose their
  cut first.

**Consequence.** The serial chain of §3.1 is an artefact of working one coincidence. The
induction is ~10² rounds of independent merges; lanes, or threads, are available at no change
to the grammar. Not built; the first step toward it failed on its constants (§6).

## 6. Clock (M; one thread, fastest of 5 rounds a block, Xeon 2.1 GHz, 2 MiB L2; indicative)

Arms. Production: `bwt::build` + `walk_segments_room` from the column. Kernel, from the run
list: **order** (the runs sorted by byte, both orders linked: this exchange's "build"),
**induce**, **write**. The scan of the column into runs is apart: 0.484 ns a row here, 0.173
from #324's emitter (PF:`by-data-class` §2).

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
| `(ab)^k` (no division rule) | 1.27 | 9.958 | 7.83 |
| uniform bytes | 3.29 | 53.4 | 16.3 |

- **The write is not the cost:** 0.419 ns a row for the whole block from the grammar, 0.10 on
  the Sturmian inputs (the n bytes themselves).
- **The step is:** 14.0 ns (≈ 29 cycles), 66% of the kernel. With the word work removed it is
  still 10.6–11.5 ns: the order surgery, branch-free, about a dozen stores a step. A branchy
  step costs the same in mispredictions. Not cache misses: 11 ns at r = 2·10⁴.
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
  letters take the order stage from 0.525 to 2.448: about 10 ns a letter each, in mispredicted
  branches (which run is joined, where a chain ends). Kernel 1.668× the walk. A pass over the
  letters must cost ≤ 3 ns a letter to pay; none here does. The construction failed, not the
  count: the 63.5% is still the cheapest work in the block.
- **Live set as built:** 32 r (nodes) + 0.47 n (arena) + 16 B × 0.039 n (rules) + 4 B a rule + n
  (the expansion before rotation): about 4 MB at n = 9·10⁵, r = 6.5·10⁴. Not smaller than
  production's table. The nodes halve with 16-bit links (r < 2¹⁶ on three blocks in four); the
  n goes when the rotation is folded into pass 1.
- **Verdict on the clock:** this construction (one chain, linked orders) loses on Wikidata by
  28%, is level below r/n ≈ 0.06, and wins from r/n ≈ 0.02 down. The advantage in kind (0.13 n
  dependent steps, r-sized state, no column) is not disproved and not converted.
- **Correction made while writing:** the first table of this entry excluded the order stage
  (it had been timed with the scan) and read 1.078×. The exchange's build is the kernel's
  work; 1.276 is the figure.

## 7. Falsifiers run

| # | attack | outcome |
|--:|---|---|
| 1 | bytes ≠ walk on any block | none: 233 Wikidata blocks, 4 prose, 30 synthetic incl. periodic and n < 20 |
| 2 | S ≫ r on natural data | no: 1.29–2.08 r |
| 3 | S = Θ(n) on some input | **yes**: `(ab)^k`, S = n/2, 7.8× slower. Fix: division rule (q equal-pair steps → one power rule), or a step budget with fallback to the walk. Wikidata: 46 such steps in 27.4 M |
| 4 | text | **fires**: S/n 0.68, 7.4× slower. Closed for r/n ≳ 0.07 at this step price |
| 5 | the clock on Wikidata | **fires for this construction**: 1.276×. The step is 6.4 row-steps |
| 6 | bulk-join the exact chains first (the cheap 63.5%) | **fires as built**: 1.668×; letter-passes at 10 ns each (§6) |
| 7 | prior art | the mechanism is classical; the transfer is not in print (§10) |
| 8 | independent implementation from the spec, invariant checked by brute force | see §11 |

## 8. Open, ranked: ceiling · cheapest falsifier · gate

1. **Segment-parallel induction.** Arrays, not lists: a round = the two-pointer merge of each
   ready segment; re-seating = a scatter into next round's orders. Ceiling: 32 cycles a run
   all told ⇒ 0.5× on Wikidata, and no dependent chain at all (the i9's currency). §6's
   pre-pass is its warning: every pass branch-free, ≤ 3 ns a letter. Falsifier: an instruction
   and misprediction count of the merge and scatter per cell (callgrind, an untimed probe);
   dead above ~35 Ir or 0.3 mispredictions a cell. Gate: mine.
2. **Feed from run records** (#324 / #348, `u16` records cut at 127: merge neighbours, 0.0779 →
   0.0722 a row). Removes the column and the scan. A port, after 1.
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
   mod P as counted); 75,532 (word, entry state) pairs under random bracketings and 4,878 whole
   blocks equal the bytewise decoder and CRC. This entry's first guess, that the entry state
   dies at the first change of byte, was false: one bit survives, flipped by a run of length
   ≡ 4 mod 5, kept by ≡ 0, forgotten by 1, 2, 3 (`aaaabbbbb`). Not in the example. Ceiling 4% of
   a Wikidata decode (PF:`by-data-class` §1): low. Line offsets for `wikidata-pq` by the same
   morphism: uncounted.
5. **Rounds.** **Conj:** O(log r) for sources with bounded-memory statistics (93 at r = 5·10⁴,
   101 at 3.7·10⁵, 47 on uniform bytes). And §5.5 in text space: an induction that never
   leaves the text's phrases, or a count of S from the front's run records alone.
6. **The i9.** Production's inversion is 110 cycles a run there (PF:`phrase-structure` §5), 63
   here; the kernel is 81 here with 0.13 dependent steps a row against 1. Whether its 81 holds
   on a 256 KiB L2 is unknown. A diagnostic for the Coordinator's queue (PROTOCOL §9, RULES
   C1a) only after 1 or a leaner step; never run here.
7. **Admission** by r/n (known after MTF): nothing to admit until the ratio moves. Not proposed.

## 9. Frame (not used above)

- **P (elementary; checked on the same 4,897 words):** with τ(x) = x + 1 mod n, the commutator
  π τ π⁻¹ τ⁻¹ fixes x unless x ∈ 𝓑, and it moves all of 𝓑 but at most σ points (bucket seams,
  the wrap). So (τ, π) is a square-tiled surface: one horizontal cylinder (τ is an n-cycle),
  one vertical when π is an n-cycle (**K** as an object: pairs of n-cycles, Delecroix–Goujard–
  Zograf–Zorich 2020; Jeffreys 2021).
- **Its cone points are the commutator's cycles, not the runs** (the sweep's correction to this
  entry's first wording). **C:** on the 233 Wikidata blocks the r image boundaries fall into
  1 to 11 cycles (median 5; prose 4 to 11); genus = (r − cycles)/2 + 1 ≈ r/2: 26,500 at
  r = 53,001. A block is a surface of genus r/2 with a handful of cone points of very large
  angle. 𝓣 ∩ 𝓑 ↔ vertical saddle connections of length one.
- **K:** SA conjugates π to the rotation p ↦ p − 1 and τ⁻¹ to φ (GNP Lemma 19; Navarro–Ochoa–
  Prezza Lemma 6): the dual r-IET in text space is the same surface turned a quarter. §5.5 is
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
- **Not found:** endpoint induction as a BWT inversion or RLBWT decompression; a count of S on
  any corpus; the coincidence/segment/round structure; output passes on the induced grammar.
  The words "interval exchange", "Rauzy", "first return" do not occur in the r-index and
  move-structure papers read.
- **In the team's record:** first-return towers by piece-stepping, the grammar as a sentence
  (PF:`interval-exchange` §1; `chunked-link` §5a). Absent: endpoint induction, a built grammar,
  S, CRC on a structure (two readers of pr/313 and the 420-row register).
- **What is claimed as new, and no more:** the transfer (runs of L as the letters, the block
  as the induction's SLP, the two-pass write, the CRC and RLE1 by morphism), the counts of
  §4–§5, and §5.5's reading of the step count as records of suffix ranks inside the BWT's
  phrases. The step, its morphism, the linked-interval machine and the SLP from accelerated
  induction are others'.

## 11. Independent check

_Pending at the time of writing: a second implementation from the written spec, the invariant
checked against a brute-force first-return map after every step. Its outcome is appended below._

## Reproduce

```text
bash bench/corpus.sh                                   # corpora/wikidata.bz2
cargo build --release --example rauzy
target/release/examples/rauzy corpora/wikidata.bz2 1 65536 5 > out.jsonl
RAUZY_JOIN=1 target/release/examples/rauzy corpora/wikidata.bz2 1 65536 3 > join.jsonl
```
Beside this entry: `2026-11-10-rauzy-veech/identities.py.txt` (§5.1 and §9 by brute force) and
`rle1_crc_on_grammar.py.txt` (§8, 4); each runs as `python3 FILE`.

One JSON line a block: `n, runs, steps, top, bottom, equal, dropped, coincide, shapes, repeats,
divisions, depth, widest, rounds, selfish, selfish_steps, copies, copied, hist` (rule lengths:
count and bytes by class), `crc_muls`, `closed` (§5.5 asserted), `cone` (points the commutator
moves, its cycles), and ns a row: `ns_walk` (build + room walk); the kernel's
`k_runs` (the scan), `k_order`, `k_induce`, `k_write`, with `k_joined`, `k_steps`, `k_rules`; the
reference's `ns_runs`, `ns_induce`, `ns_expand` (a rule a step, no arena). The reference asserts
bytes and CRC; the kernel asserts bytes.
