# 2026-11-10 · MONODROMY 1 · LF's interval exchange, renormalised at its endpoint: a block is 1.81 r binary rules

Labels: **P** proved here · **C** counted (exact for its input) · **M** measured (this container,
indicative, RULES C5) · **Conj** conjecture · **K** known, cited. `examples/rauzy.rs` rebuilds
every block named below from the grammar and asserts its bytes against `bwt::walk`.

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
- **C:** the induction is not one chain. Its 15,141,369 segments order into **93 rounds** (median;
  63 to 337) of independent two-pointer merges.
- **M:** first kernel, runs → bytes: induce 1.958 + write 0.421 = 2.379 ns a row against build +
  room walk 2.207: **1.078×, a loss of 8%**, pooled. 0.85 on blocks with r/n < 0.06 (40% of
  rows), 2.14 at r/n ≥ 0.12. 137 of 233 blocks faster. Not yet a win on this corpus.
- **Step price** 15.0 ns ≈ 6.8 row-steps; break-even 6.2. One chain today; §5 says it need not be.
- **Text is out:** prose r/n 0.367, S/n 0.676, kernel 6.3× slower. Stays the row walk.

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
is not on are dropped. `ab`×450k, `aaaab`×180k, zeros: bytes exact (C, §4).

**Counts (P).**
- Equal steps = r − 1 for one cycle: only they delete letters. **C:** 15,141,369 = 15,141,602 − 233.
- S ≤ n − 1: every step removes ≥ 1 point.
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
The same count as the record's first-visit copy walk (PF:`2026-11-09-phrase-structure.md`,
correction §1: 0.0328 copies, 0.928 of bytes, on its own sample): one object, reached without
walking.

**3.3 Small words as bytes.** 70.2% of rules are ≤ 16 bytes (**C**: shares 0.211, 0.198, 0.164,
0.129 at ≤ 2, 4, 8, 16) and hold 0.47 n bytes. The kernel keeps a word of ≤ 16 bytes in an arena
(two 16-byte moves a step) and makes a rule only above: 0.0390 long rules a row, 0.54 a run.

**3.4 Homomorphisms in O(S) (P; K as a technique, e.g. Erickson–Nayyeri 2013 §5.1).** For any
monoid morphism h: Σ* → M, h(N) = h(left) h(right): S products, not n. CRC-32/BZIP2:
R(w) = w(x)·x³² mod P, X(w) = x^{8|w|} mod P, R(uv) = R(u)X(v) + R(v), X(uv) = X(u)X(v);
crc = ¬(R + ones·X). The rotation costs one descent from the root. **C:** 0.262 multiplications a
row; equal to `crc::crc32` of the walked bytes on all 233 blocks and every synthetic input.
bzip2's block CRC is over the RLE1-decoded bytes: the RLE1 transducer's state must be composed
in (open, §8).

## 4. Counted (C)

`rauzy FILE 1 65536 5`: every block ≥ 65,536 rows.

| input | blocks | r/n | S/r | S/n | coincidences/r | rounds (median) |
|---|--:|--:|--:|--:|--:|--:|
| `wikidata.bz2` | 233 | 0.0722 | 1.812 | 0.1309 | 0.780 | 93 |
| prose (the repository's `.md`, 3.5 MB, `bzip2 -9`) | 4 | 0.3672 | 1.841 | 0.6761 | 0.541 | 101 |

Synthetic, one block each (n ≈ 9·10⁵ unless said):

| input | r | S | S/n | note |
|---|--:|--:|--:|---|
| Fibonacci word | 16 | 175 | 0.0002 | |
| Thue–Morse 2²⁰ | 54 | 485 | 0.0005 | |
| period 37 | 37 | 24,044 | 0.027 | 23,998 steps are one division |
| `(ab)^k` | 2 | 449,991 | 0.500 | one division |
| `(abac)^k x`, `(abcabd)^k x` | 4, 6 | n/4, n/6 | | one division each |
| JSON-like, 300 ids | 19.7k | 66k | 0.073 | S/r 3.35 |
| JSON-like, 20k records | 136k | 235k | 0.261 | S/r 1.73 |
| 4 letters, uniform | 652k | 876k | 0.973 | S/r 1.34 |
| bytes, uniform | 896k | 900k | 1.000 | S/r 1.00 |

- **S/r is 1.8 on both natural corpora, five-fold apart in r/n.** Not a law: 1.0 (random) to
  3.4 (few-id JSON) to ~10 (Sturmian-like, where S is tiny anyway).
- **Floor (P):** S ≥ r − 1 for any schedule of these steps: a step deletes at most one letter.
  So at most 1.81× is left in the count on Wikidata. What is undecided is the step's price.
- Against the record: the tower 8 × 3 is 0.190 n dependent steps on Wikidata and stops at
  W = n/512 with words materialised at each level (0.68–1.58 n bytes); this is 0.131 n, stops at
  one point, materialises 0.47 n bytes of small words. PF:`2026-11-10-chunked-link.md` §5a's bound
  (2r a halving, 0.55 n, "pays only below r/n ≈ 0.03") is 4× above the count.

## 5. Structure: coincidences, segments, rounds

**5.1 (P) |𝓣 ∩ 𝓑| = 2r − r₂**, r₂ = the pieces of π² (maximal intervals on which π² is a
translation with one 2-byte word). The cells of the common refinement of the top and bottom
tilings number |𝓣 ∪ 𝓑|; each cell I_α ∩ J_β pulls back to one piece of π² inside I_β, and two
neighbouring pull-backs differ in a byte. **C:** 0.780 r on Wikidata, 0.541 r on prose; the
record's r₂/r = 1.22 (run-fed census) is the same number. Read as dynamics: 78% of LF's
discontinuities land on discontinuities after one step. Read on the surface (§9): unit saddle
connections.

**5.2 (P) Segments.** Cut [0, m) at 𝓣 ∩ 𝓑. A segment is at once a union of top intervals and a
union of bottom intervals. The endpoint induction consumes the last segment as a two-pointer
merge of its top lengths against its bottom lengths: one step a cell, the equal step exactly
at the segment's end. So
**S = Σ over the r − 1 segments consumed of (cells in it when consumed).**

**5.3 (P) Coincidences persist.** A step removes an interval that is one set of points in both
tilings and contains no top boundary inside; every other coincidence survives. Hence a
Rauzy step is valid at *any* coincidence, not only at m: letters A (top interval ending
there) and B (image ending there), same table. Steps on disjoint letters commute.

**5.4 (C) The induction's dependency order.** Segment σ needs segment σ' iff σ' re-seated or
shortened a letter σ consumes. Longest chain, 233 blocks: **63 / 78 / 93 / 119 / 337** rounds
(min, p10, median, p90, max); median 606 segments a round. Prose: 79 to 118 rounds for
3·10⁵ segments. Same-round segments share no letter (**P**: a shared letter orders them).
- A segment that meets a letter it re-seated itself is a genuine Euclid stretch: **84 of
  15,141,369** (1,901 steps of 27,439,723). On real blocks the Euclidean part is absent.
- Counted a step at a time (a step needs the last step on either letter and the one before it
  in its segment): depth median 3,861; widest segment 1,562 steps (one long run absorbing its
  predecessors, itself a prefix sum).

**Consequence.** The serial chain of §3.1 is an artefact of working one coincidence. The
induction is ~10² rounds of independent merges; lanes, or threads, are available at no change
to the grammar. Not built (§8, 1).

## 6. Clock (M; one thread, fastest of 5 rounds a block, Xeon 2.1 GHz, 2 MiB L2; indicative)

Arms: `bwt::build` + `walk_segments_room` from the column; the kernel from the run list
(`Kernel::induce`, `Kernel::write`). Run extraction from the column is listed apart: the
example's scan and bucket sort are 0.850 ns a row, #324's emitter is 0.173 (PF:`by-data-class`
§2).

| Wikidata, 233 blocks, ns a row | build + walk | induce | write | kernel ÷ walk |
|---|--:|--:|--:|--:|
| pooled | 2.207 | 1.958 | 0.421 | **1.078** |
| r/n < 0.06 (93 blocks, 0.399 of rows) | 2.23 | 1.52 | 0.38 | 0.85 |
| 0.06–0.07 (71, 0.305) | 2.18 | 1.67 | 0.39 | 0.95 |
| 0.07–0.09 (32, 0.137) | 2.15 | 1.94 | 0.42 | 1.10 |
| 0.09–0.12 (19, 0.082) | 2.26 | 3.04 | 0.55 | 1.59 |
| ≥ 0.12 (18, 0.077) | 2.26 | 4.22 | 0.62 | 2.14 |

| other inputs | build + walk | kernel | ÷ |
|---|--:|--:|--:|
| Thue–Morse | 2.05–3.81 | 0.17–0.52 | 0.09–0.14 |
| Fibonacci | 4.42 | 0.69 | 0.15 |
| period 37 | — | — | 0.23 |
| JSON-like, 300 ids (r/n 0.022) | 1.81–1.96 | 1.10–1.21 | 0.56–0.67 |
| prose | 2.915 | 18.26 | 6.26 |

- **The write is not the cost:** 0.421 ns a row for the whole block from the grammar.
- **The step is:** 15.0 ns (≈ 31 cycles), 82% of the kernel. With the word work removed it is
  still 10.6–11.5 ns: the order surgery, branch-free, about a dozen stores a step. A branchy
  step costs the same in mispredictions. Not cache misses: 11 ns at r = 2·10⁴.
- **Verdict on the clock:** this construction (one chain, linked orders) is at parity on
  Wikidata, ahead below r/n ≈ 0.065, behind above. The advantage in kind (0.13 n dependent
  steps, r-sized state, no column) is not disproved and not yet converted.

## 7. Falsifiers run

| # | attack | outcome |
|--:|---|---|
| 1 | bytes ≠ walk on any block | none: 233 Wikidata blocks, 4 prose, 14 synthetic incl. periodic and n < 20 |
| 2 | S ≫ r on natural data | no: 1.29–2.08 r |
| 3 | S = Θ(n) on some input | **yes**: `(ab)^k`, S = n/2. Fix: division rule (q equal-pair steps → one power rule), or a step budget with fallback to the walk. Wikidata: 46 such steps in 27.4 M |
| 4 | text | **fires**: S/n 0.68, 6.3× slower. Closed for r/n ≳ 0.07 at this step price |
| 5 | the clock on Wikidata | **fires for this construction**: 1.078×. The step is 6.8 row-steps against a break-even of 6.2 |
| 6 | prior art | the mechanism is classical; the transfer is not in print (§10) |
| 7 | independent implementation from the spec, invariant checked by brute force | see §11 |

## 8. Open, ranked: ceiling · cheapest falsifier · gate

1. **Segment-parallel induction.** Arrays, not lists: a round = the two-pointer merge of each
   ready segment; re-seating = a scatter into next round's orders. Ceiling: step ≤ 5 ns ⇒ kernel
   ≈ 1.08 ns a row ≈ 0.49× on Wikidata, and no dependent chain at all (the i9's currency).
   Falsifier: an instruction count of the merge loop and scatter per cell (callgrind on an
   untimed probe); dead above ~35 Ir a cell. Gate: mine.
2. **Feed from run records** (#324 / #348, `u16` records cut at 127: merge neighbours, 0.0779 →
   0.0722 a row). Removes the column and the scan. Falsifier: none needed, a port. Gate: after 1.
3. **Admission.** r is known after MTF: r/n < 0.065 admits 70% of Wikidata's rows at 0.85–0.95
   today; plus a step budget. Worth nothing until 1 moves the ratio. Not proposed.
4. **Division (Zorich) rule** for equal-pair streaks; whether a cycle of k losers needs the full
   acceleration. **Conj:** with it, S = O(r log n) (K for normal curves: Stefankovič; Erickson–
   Nayyeri 2013 Thm 4.8). Falsifier: a family with a loser cycle ≥ 2 that defeats the pair rule.
5. **Output passes on the grammar:** RLE1's transducer composed over rules (state: last byte,
   repeat count, count-pending), then the stored CRC and the decoded length with no pass over
   bytes. Ceiling 4% of a Wikidata decode (PF:`by-data-class` §1): low. Line offsets for
   `wikidata-pq` by the same morphism: uncounted.
6. **Why 1.8.** S = Σ cells over segments; initial cells = 2r − |𝓣∩𝓑| = 1.22 r for 0.78 r
   segments. No bound. **Conj:** rounds = O(log r) for sources with bounded-memory statistics.
7. **The i9.** Dependent loads 0.13 a row, state 32 r + rules. A diagnostic only once a kernel
   wins locally; queued through the Coordinator (PROTOCOL §9), never run here.

## 9. Frame (not used above)

- **P (elementary):** with τ(x) = x + 1 mod n, the commutator π τ π⁻¹ τ⁻¹ fixes x unless x ∈ 𝓑, up
  to the ≤ σ bucket seams and the wrap. So (τ, π) is a square-tiled surface: one horizontal
  cylinder (τ is an n-cycle), one vertical when π is an n-cycle, its cone points among the r
  image boundaries. Runs ↔ singularities; 𝓣 ∩ 𝓑 ↔ vertical saddle connections of length one.
- **K:** SA conjugates π to the rotation p ↦ p − 1 and τ⁻¹ to φ (GNP Lemma 19): the dual r-IET in
  text space is the same surface turned a quarter.
- Rauzy–Veech induction is the first return of the Teichmüller flow; §5 says that on BWT
  surfaces it is dominated by collapsing unit saddle connections. Nothing computational is
  claimed from this section yet.

## 10. Prior art (searched 10 Nov; the sweep's report is in the session record)

- **K:** LF as r translated intervals: GNP 2020 Lemma 18; Nishimoto–Tabei 2021. Under the name
  IET, for clustering words: Ferenczi–Zamboni 2013; multisets: Dolce–Hughes 2025.
- **K:** one Rauzy step ↔ one binary morphism: Rauzy 1979; Jullian 2013; Dolce–Perrin 2017
  Prop. 4.12; the equal ("merging") case on discrete IETs: Dolce–Hughes, arXiv:2511.22588,
  Prop. 4.1. No algorithm or cost there.
- **K:** an SLP of size O(k log N) for an integer-weighted exchange-like object by Euclid-style
  induction with division: Agol–Hass–Thurston 2006 (orbit counting); Schaefer–Sedgwick–
  Stefankovič 2002; Erickson–Nayyeri 2013 Thm 4.8; iterating an integer IET in O(k² log N):
  Bell, in Eppstein 2023 Thm 6.15. The mapping of an RLBWT onto those objects is not in print.
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

## 11. Independent check

_Pending at the time of writing: a second implementation from the written spec, the invariant
checked against a brute-force first-return map after every step. Its outcome is appended below._

## Reproduce

```text
bash bench/corpus.sh                                   # corpora/wikidata.bz2
cargo build --release --example rauzy
target/release/examples/rauzy corpora/wikidata.bz2 1 65536 5 > out.jsonl
```
One JSON line a block: `n, runs, steps, top, bottom, equal, dropped, coincide, repeats,
divisions, depth, widest, rounds, selfish, selfish_steps, copies, copied, hist` (rule lengths:
count and bytes by class), `crc_muls`, and ns a row: `ns_walk` (build + room walk), `k_runs`,
`k_induce`, `k_write` (the kernel), `ns_runs`, `ns_induce`, `ns_expand` (the reference, a rule a
step). The reference asserts bytes and CRC; the kernel asserts bytes.
