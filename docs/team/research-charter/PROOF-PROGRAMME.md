# The proof programme: one specification, several routes

*22 November 2026. A proposal for the research charter. Nothing here is formalised yet; §4 is the
plan for starting.*

## 0. The idea

Fix a formal specification of inverse-BWT reconstruction. A constructive proof that some
construction meets it is also a description of an algorithm, and different proofs of the same
statement can describe algorithms with different work, dependency depth and memory traffic.
mibz2 already has three routes to the theorem:

- **the LF walk**, the classical decoder (`src/bwt.rs`: `walk`, `walk_back` and the segmented
  walks);
- **cycle decomposition** (Cauchy): the marked cycle is read once and repeated to `n` bytes,
  which is what makes periodic and multi-cycle columns decode correctly (`repeat_cycle` in
  `src/bwt.rs`, `repeat` in `src/open.rs`);
- **the first return to a retained set**, reached through Rauzy–Veech induction: the Open
  kernel (`src/open.rs`, `src/open/endpoint.rs`; hand proof in
  [proof.md](../../journal/2026-11-18-open-return/proof.md)). It is mibz2's largest structural
  gain.

The historical model is the Riemann hypothesis for curves over finite fields. Weil proved it in
1948. Stepanov (1969, for hyperelliptic curves) and Bombieri (1973, in general) later gave
elementary proofs, and Stepanov's auxiliary-polynomial method is the one that carried over to
problems over the integers. The proof that transferred was not the first one.

The refinement that matters here: the useful new proof is often a new proof of one load-bearing
intermediate lemma, not of the whole theorem. In our setting the candidates are how a
permutation is represented, how transformations are composed, and how an orbit is recovered from
local data. §2's T3 names these lemmas, so that a research line can say which one it replaces.

There is a working precedent for agents writing a verified decompressor. kim-em/lean-zip
formally verifies DEFLATE in Lean 4: reportedly about 1,100 theorems with no `sorry`, written by
agents, including a proof that an optimised inflate equals the reference inflate. Its README
argues that a fixed specification lets agents optimise aggressively, because each optimisation
is checked against the same statement. (These figures are from its README and have not been
checked here.)

### Four things kept apart

Throughout this document these are different objects, and none implies another:

1. **A Lean proof of correctness:** a theorem that a construction meets the specification.
2. **An executable algorithm:** what the proof's construction computes. Not every proof has
   useful computational content. A proof that two functions are equal may construct nothing,
   and a proof by `decide` or by classical choice yields no decoder.
3. **The Rust implementation:** `src/`. Differential tests (`src/open/checks.rs`, the B5 tests,
   the `#729` gate) are valuable evidence that Rust matches a model, but they are not formal
   equivalence. Translating Rust into Lean (Aeneas, for example) is a later option, not part of
   this plan.
4. **Cost:** time, memory and traffic. It stays an empirical question, decided by measurement on
   the i9. A proof can suggest a cost model; it does not establish one.

## 1. The shared specification

### 1.1 The contract in `src/`

The kernel contract is `InverseBwt::invert` (`src/bwt.rs`): a block's last column `ll`, its
byte counts `counts`, and the primary index `orig_ptr`, which the caller has checked is below
`ll.len()`. The output is the block's RLE1-coded text: RLE1 decoding and the block CRC are
outside the inverse BWT and outside this specification.

- **The block bound.** A block's header gives `orig_ptr` in 24 bits and the level gives
  `max_len = 100_000 × level` (`src/block.rs`), so `n ≤ 900,000`. The MTF stage refuses a column
  longer than `max_len` (`src/mtf.rs`), and the decoder refuses `orig_ptr ≥ n` before any
  kernel runs (`src/lib.rs`). The Open kernel restates this as its entry condition,
  `0 < n ≤ 900_000` and `p < n` (`OpenCellsOf::cap`, `Kernel::try_run`), and its packed fields
  rely on it (`Band`: 20-bit row fields; `endpoint.rs`: ids below `256 + 2 × 900,000 < 2^22`).
- **The T-vector.** `build` writes, at slot `k`, the byte `L[k]` and the link `Ψ(k)`, the row
  `i` with `LF(i) = k` (a counting sort over `counts`). Links are 24 bits, so `n < 2^24`.
- **The primary index.** bzip2's `origPtr` is the row of the sorted rotation matrix that holds
  the original text. The plain walk (`walk`) starts from `t[orig_ptr]` and emits
  `L[Ψ^(j+1)(orig_ptr)]` for `j = 0, …, n − 1`. On a valid block that is `T[j]`: row `Ψ(p)` is
  the rotation starting at `T[1]`, whose last byte is `T[0]`.
- **Arbitrary columns.** The decoder runs on any column with a valid primary, BWT or not; a
  corrupt block is caught later, by the CRC. Every kernel must give `walk`'s exact output on
  every such column, so that a corrupt block fails identically (`walk_back`'s doc comment;
  the Open kernel's safety note). On a column whose LF has several cycles, `walk` emits the
  marked cycle's bytes repeated and cut at `n`.

The specification is therefore total: it is defined on every `(L, p)` with `0 < n ≤ 900,000`
and `p < n`, and it has a separate theorem (T1) for the case where `L` really is a BWT.

### 1.2 Pseudo-Lean

The block is `Fin n`; bytes are `UInt8` (or `Fin 256`). Ranks are half-open.

```lean
structure Column where
  n  : ℕ
  L  : Fin n → UInt8
  p  : Fin n                       -- bzip2's origPtr
  hn : 0 < n ∧ n ≤ 900_000         -- bzip2's block bound (100k × level, level ≤ 9)

namespace Column
variable (c : Column)

/-- occurrences of byte b in L -/
def count (b : UInt8) : ℕ := (Finset.univ.filter (c.L · = b)).card

/-- C[b]: rows whose byte is below b; F is the column of first bytes -/
def C (b : UInt8) : ℕ := ∑ a in Finset.univ.filter (· < b), c.count a

/-- occurrences of b strictly before row i -/
def rank (b : UInt8) (i : Fin c.n) : ℕ :=
  (Finset.univ.filter fun j => j < i ∧ c.L j = b).card

/-- LF(i) = C[L i] + rank_{L i}(L[0:i]): the stable counting sort of L -/
def LF (i : Fin c.n) : Fin c.n := ⟨c.C (c.L i) + c.rank (c.L i) i, lf_lt c i⟩

theorem LF_bijective : Function.Bijective c.LF     -- L0, §2.3
def lf : Equiv.Perm (Fin c.n) := Equiv.ofBijective c.LF c.LF_bijective
def Ψ  : Equiv.Perm (Fin c.n) := c.lf.symm           -- the T-vector's link

/-- the specification: walk's output -/
def decode : Fin c.n → UInt8 := fun j => c.L ((c.Ψ ^ (j.val + 1)) c.p)
end Column
```

`decode` is the specification every kernel is held to. Two equivalent forms are worth having
from the start, because kernels use both:

- **Backward form** (`walk_back`, and the Open kernel's direction): with `m` the length of
  `p`'s cycle, `decode (m − 1 − k) = L (LF^k p)` for `k < m`, and `decode j = decode (j mod m)`.
  (`walk_back` writes the lap at the end of its buffer; `repeat_cycle` moves it to the front.)
- **Periodic form:** `decode` is periodic with period `m = minimalPeriod Ψ p`, and its first `m`
  values are the bytes of `p`'s cycle.

The BWT itself, for T1:

```lean
/-- rotation r of T, as a word -/
def rot (T : Fin n → UInt8) (r : Fin n) : Fin n → UInt8 := fun j => T (r + j)

/-- σ sorts the rotations: rows in lexicographic order, ties in any order -/
structure SortsRotations (T : Fin n → UInt8) (σ : Fin n ≃ Fin n) : Prop where
  mono : ∀ i j, i ≤ j → rot T (σ i) ≤ₗₑₓ rot T (σ j)

def bwt (T) (σ) : Fin n → UInt8 := fun i => T (σ i + (n - 1))   -- last column
```

A valid input is `(bwt T σ, p)` with `SortsRotations T σ` and `rot T (σ p) = T`. bzip2 chooses
one sort and one `p`; T1 is stated for every choice, since equal rotations (a periodic text) have
equal last bytes and any row holding `T` decodes to it.

## 2. Theorem statements

### 2.1 T1: the LF walk reconstructs the text

```lean
theorem T1 (T : Fin n → UInt8) (σ : Fin n ≃ Fin n) (hσ : SortsRotations T σ)
    (p : Fin n) (hp : rot T (σ p) = T) (hn : 0 < n ∧ n ≤ 900_000) :
    (Column.mk n (bwt T σ) p hn).decode = T
```

The classical proof has two steps:

- **T1a (the LF property).** `rot T (σ (LF i)) = rot T (σ i − 1)`: the row LF maps to holds
  the rotation one position earlier (the same row, when the rotations are distinct). Its content is that prepending a byte `c` preserves the order of rotations
  that end in `c`, so the `k`-th `c` in `L` and the `k`-th `c` in `F` are the same text
  position. Ties between equal rotations are where the proof needs care: any order of equal
  rows is fine, because their rows are interchangeable.
- **T1b.** Hence `rot T (σ (Ψ^(j+1) p)) = rot T (j + 1)`, so `L (Ψ^(j+1) p) = T j`.

The bound `n ≤ 900,000` plays no part in T1; it is kept in `Column` because the Rust kernels
depend on it, and T2 must be stated on exactly the kernels' domain.

Two companion lemmas on arbitrary columns, both already relied on by `src/bwt.rs`:

- **T1c (direction duality).** The backward form and the periodic form of §1.2 equal `decode`.
  This is `walk_back` + `repeat_cycle` = `walk`.
- **T1d (segment join).** For any nonempty set of segment starts on `p`'s cycle, the walks between
  consecutive starts, joined in cycle order from the segment holding `Ψ p`, give `decode`. This is `walk_segments` and
  `walk_segments_room` = `walk`.

### 2.2 T2: the Open kernel equals the LF walk

**Statement.** For every `Column` and every limit `LIMIT > 0`, if the Open construction
completes (it can refuse a block past its cap, `try_run` returning `None`, and the decoder then
falls back to `LazyCells`), its output is `decode`:

```lean
theorem T2 (c : Column) (recs : List Record) (hrecs : expand recs = c.L)
    (limit : ℕ) (hlim : 0 < limit) (out) (h : openDecode recs c.n c.p limit = some out) :
    out = c.decode
```

T2 holds on **every** column with a valid primary, not only BWT columns: the hand proof never
uses the text, and the Rust tests decode arbitrary columns against the plain inverse. So T2
needs no part of T1, and T1 plus T2 give Open's correctness on valid blocks.

The model `openDecode` is an abstract construction: retained set, auxiliary exchange,
elimination, returns, emission. It is not the Rust code; matching it to `src/open.rs` is a
separate step (§2.4).

**Proof obligations,** from [proof.md](../../journal/2026-11-18-open-return/proof.md). Write
`f = LF`, `S` the retained rows, `E = f(S)`, `m = |S|`, `s_j` and `e_k` their elements in
increasing order, and `v = L[s_0] ⋯ L[s_{m−1}]`.

| # | Obligation | Statement | Source |
|---|---|---|---|
| O1 | Records and selection | The run records (each a byte and a width, at most 127 in production) expand to `L`; merging them gives the maximal runs; `split` partitions every row into retained rows (runs of width ≤ `LIMIT`, and `p` as a singleton when it lies in a thick run) and thick strips. `p ∈ S`. | `maximal`, `split`, `each_run` in `src/open.rs` |
| O2 | Runs are translations | On a constant run of `L`, `f(x + 1) = f(x) + 1`. The thick strips are labelled translations, disjoint in domain and in image (because `f` is injective). | proof.md, opening paragraph |
| O3 | Restriction lemma | If `f(s_j) = e_k` then `k = LF_v(j)`: restricting the stable sort of `(L[x], x)` to sources in `S` is the stable sort of `v`. Holds for every `S` and every byte frequency. | proof.md, "Restricting a stable match" |
| O4 | Open paths | With the arrows out of `S` deleted, the path from each entrance `e_k` reaches a unique exit `s_i` in finitely many steps and enters no cycle; `P(k) = i` is a bijection; what remains of the complement is closed cycles disjoint from `S`. | proof.md, same section |
| O5 | First-return factorisation | The first return to `S`, in sorted coordinates, is `g = P ∘ LF_v`, and the return from `j` spells `v[j] w_k`, where `w_k` is the word on the open path from `e_k`. | proof.md, displayed `g(j)` |
| O6 | Spelling and period | The inverse arrows, followed from the index of `p`, spell the marked text forwards, `rev(w_k) v[j]` per arrow. One lap is the marked cycle's period in bytes, and repeating it cut at `n` gives `decode`, including a mark on a fixed point and cycles that meet no other retained row. | proof.md; `repeat` in `src/open.rs` |
| O7 | Independence | `P` and `w` depend only on the complementary strips and the masks `S`, `E`; the retained arrows enter only through `LF_v`. (This is what lets the construction run before any retained arrow is wired.) | proof.md, "Independence" |
| O8 | Auxiliary exchange | The exchange on `m` terminal points followed by a shifted copy of the rows (terminal `k → m + e_k`; `m + x → m + f(x)` on thick strips; `m + s_i → i`) has first return to the prefix equal to `P`, labelled by `w`. Cycles confined to the suffix give no prefix output. | proof.md, "Computing the open map by endpoint induction" |
| O9 | Endpoint induction step | Removing the common suffix of the last domain strip `X` and last image strip `Y` preserves the labelled first return to the prefix, with the composite word `rev(X)` then `rev(Y)`; when `X = Y` the removed interval is fixed and disconnected from the prefix. The prefix boundary is never crossed. | proof.md, same section; `endpoint.rs` |
| O10 | Termination | Each step removes at least one row; the live strip count never increases; the induction ends with only the prefix. | proof.md, same section |
| O11 | Spiral contraction | `q = (aw − 1) / bw` consecutive steps against one partner equal one step producing `x^q y` and width `aw − q·bw` (and the mirror case `x y^q`); `power` by squaring spells `word^q`. Lists, widths and spellings equal the plain loop's. | `eliminate::<true>`, `power` in `endpoint.rs` (#703) |
| O12 | Emission | Following the stored edges from the mark, appending each code's word, stopping at the mark or at `n` bytes, then `repeat`, yields `decode`. | `emit`, `emit_bidirectional`, `repeat` in `src/open.rs`; `outq.rs` |

O1–O7 are statements about permutations and words, and the most natural part to formalise. O8–O11
are the endpoint induction, the hardest part: it is a statement about an interval exchange under
a rewriting loop, and the hand proof is compressed there. O12 connects the dictionary to bytes.

**Resource obligations,** not part of correctness, which the Rust code relies on through its
`assert!`s and packed fields. They are listed separately so that a correctness proof is never
taken to cover them:

- **R1 (band bound).** At most `e + H + s − 1` bundles; with the byte cuts at most `3H + σ`
  (`H` thick strips, `σ` occupied bytes). Asserted in `checks.rs`.
- **R2 (id bound).** Every rule id is at most `256 + 2 × 900,000`, below `2^22` (`ID_BITS`), and
  a spiral makes at most `SPIRAL_RULES = 41` rules.
- **R3 (width certificate).** No step widens a node, so a block whose nodes are at most
  `CERTIFIED_WIDTH` wide never needs the spiral loop (the Mechanisms line's Theorem 19.2a).

### 2.3 T3: the intermediate lemmas, as substitution points

Each lemma below is a point where a different construction could be substituted without
changing T1 or T2's statement. For each: what it says, what fills it now, and the alternatives
already on record (FRONTIER.md). A research line that proposes a new route says which of these
it replaces (§3).

| Point | The lemma | Filled now by | Alternatives on record |
|---|---|---|---|
| **L0. Representing the permutation** | LF is a bijection on `Fin n`, given by the stable counting sort of `L` | The T-vector (`Ψ` links, `build`); the LF table (`runs::fill`); Open's explicit retained edges, XOR-linked (`outq.rs`) | Implicit rank (FM-index-style; four dependent loads a step, rejected as a replacement single-threaded, FRONTIER obstructions); run-length representations of LF |
| **L1. Cycle structure** | `decode` is periodic with the marked cycle's period; one lap plus repetition gives it (T1c) | `repeat_cycle`, `repeat` (Cauchy) | The run network's cycle rank (0.21–0.25r) as a structural invariant; deck and block-system quotients (ruled out globally) |
| **L2. Composition of orbit segments** | Walks between chosen starts, joined in cycle order, equal the whole walk (T1d) | `walk_segments`, the room walk, BSAC's anchor chains | LF² super-alphabet (Kärkkäinen–Kempa–Puglisi); tunnelling; parallel list contraction |
| **L3. Restriction** | O3: a stable match restricted to any subset is the stable match of the restricted column | Open's retained set (thin runs plus `p`) | Other retained sets: switch rows at colour changes (proof.md, last section), repeated cuts (FQ2), the run network's series-reduced core |
| **L4. The induced map** | O4–O7: the first return to `S` factors as `P ∘ LF_v` with path words | Open's open paths | Stochastic complement (Meyer 1989); discrete Morse reduction (gradient paths, algebraic Morse theory over `Z[t, t⁻¹]` or the free group ring); Schur complement |
| **L5. Composition of interval maps** | O8–O11: the labelled first return of an interval exchange to a prefix is computed by repeated endpoint removal | `endpoint.rs` (Rauzy–Veech-style endpoint induction, with spiral contraction) | Cells' whole-exchange Rauzy–Veech induction; the gap tracer (proof.md); Agol–Hass–Thurston interval pairing with words; train-track splitting (Dunfield–Thurston) |
| **L6. Recovering the orbit from local data** | O12: emission from the dictionary, from the mark | One dependent load per retained row, bidirectional | Copies from completed output (Astra C); placement by a coordinate function (blocked by "run labels determine the text", record 84) |

L3, L4 and L5 are where Open replaced the classical route; L0, L2 and L6 are where the live
engineering candidates sit.

### 2.4 From the model to Rust

T2 is about `openDecode`, a model. The Rust code differs from it in representation: capped
records, queues, the arena overlaid on `edges`, `Narrow` and `Wide` links, inline words. Until a
Rust-to-Lean route is taken, the bridge is differential: `checks.rs` audits every induced arrow
of every binary column to length 8 and every ternary column to length 5, at every mark and five
limits, and the B5 tests decode arbitrary columns against the plain inverse. A natural addition,
once the Lean model exists, is to run the same exhaustive columns through the model's `#eval`
and through Rust and compare arrows, not only outputs. That remains testing, and is reported as
such.

## 3. How research lines submit candidates

A construction is submitted as an alternative proof route, in four parts:

1. **The point replaced:** one of L0–L6, or a named lemma of T1 or O1–O12. A new point may be
   proposed when no existing one fits; it must then be stated as a lemma that the rest of the
   proof uses.
2. **The replacement:** the new statement, or the new proof of the same statement, with its
   construction written out.
3. **The hypotheses:** exactly what it assumes beyond the specification (a retained-set shape,
   a bound on widths, a primitive text, a corpus property). A hypothesis that fails on some
   valid columns makes the route partial, and the fallback must be said.
4. **The rest of the proof:** which other lemmas still hold unchanged, and which must be
   re-proved.

Each submission is scored on three separate axes. They are reported side by side, never combined
into one number:

| Axis | Levels |
|---|---|
| **Correctness** | (a) proved in Lean against the specification; (b) hand-proved, with differential tests against `walk` (exhaustive small columns as in `checks.rs`, plus the B5-style arbitrary columns); (c) tested only. A route at (c) is a conjecture. |
| **Computational content** | Whether the proof constructs the decoder, or only shows that one exists. If it constructs one: its dependency structure, meaning the longest chain of dependent steps, how many independent chains there are, and which passes must finish before others start. A proof by contradiction, by `decide` over a bounded domain, or by choice has none. |
| **Measured cost** | The i9 bench against the shipped kernel, with its interval, under PERFORMANCE-POLICY.md. Cost models in the submission are diagnostic only. |

This is also the test against relabelling (the [classical-tools
audit](../../journal/2026-11-22-classical-tools-audit.md), §1 and §7). A historical result
counts for a route only if it supplies a step of the construction or of its proof, named by
lemma and, where it exists, by line of code. Naming the point replaced makes the test concrete:
a citation that replaces no lemma and proves no step is decorative. Under that test, the audit
finds three pre-1950 results doing work in shipped code: the first return (L4), Euclid's
division step and squaring (O11) and cycle decomposition (L1).

## 4. An initial formalisation plan

### 4.1 Where the Lean code lives

A separate small Lake package, either a sibling repository or a `lean/` directory with its own
workflow. The sibling repository is the better default: Mathlib's build cache is several GB,
`elan` and `lake` should not enter `src/`'s CI, and a docs or Rust PR to mibz2 then never waits
on a Lean build. The decision belongs to the Coordinator; a new repository is new
infrastructure and needs Louis's approval (RULES D). Pin one Lean toolchain and one Mathlib
commit, and build in CI only.

### 4.2 Step 1: the specification and T1

Contents: §1.2's definitions, `LF_bijective`, T1a, T1b, T1, and T1c (direction duality).

- **Mathlib to reuse:** `Fin n`, `Finset.card` and `Finset.filter` for counts and ranks;
  `Equiv.ofBijective`, `Equiv.Perm` and its powers; `Function.minimalPeriod` and
  `Equiv.Perm.cycleOf` for the periodic form; lexicographic order on words (`Pi.Lex`, or
  `List.Lex` over `List.ofFn`). Core Lean's `List.mergeSort` and its stability lemmas may give
  the stable sort of O3 directly; check this first, before writing a counting sort by hand.
- **Not in Mathlib, to be written:** the BWT itself, the LF property, and the periodic-text case
  (equal rotations). The latter is where the published informal proofs are thinnest.
- **Checks along the way:** `#eval decode` on the `banana$` example and on small periodic texts;
  a `decide` proof of T1 for all binary texts up to length 6 as a cross-check of the statement.
  Bounded `decide` proofs are checks, not the theorem.
- **Estimate:** 400–900 lines, 25–50 lemmas. A few agent-days.
- **Stop criteria:** stop and revise the statement if the specification needs a definition that
  does not match `walk`'s output on some column; stop and report if T1 exceeds about 2,000 lines
  without the LF property proved, which would mean the representation in §1.2 is wrong for Lean
  and should be redone (for example over `List` rather than `Fin n → UInt8`).

### 4.3 Step 2: T2's permutation lemmas (O2–O7)

- **Contents:** the restriction lemma (O3), the open paths and their bijection (O4), the
  first-return factorisation (O5), spelling and period (O6), independence (O7). O2 is short.
  O1 is deferred to step 3: it is about records, not mathematics.
- **Mathlib to reuse:** `Equiv.Perm.subtypePerm` does not apply (`S` is not invariant), so the
  first-return map on a subset is written here, with its well-definedness from finiteness (the
  orbit of any point returns to `S` if it meets `S`). `Finset.orderIsoOfFin` gives the sorted
  coordinates `s_j`, `e_k`.
- **Estimate:** 1,500–3,000 lines. One to two agent-weeks.
- **Stop criteria:** done when O3–O6 are proved for an arbitrary `S ∋ p`, which already gives a
  verified, executable first-return decoder (slow, but a proved second route to the
  specification). Stop if O3 or O5 needs a hypothesis that proof.md does not state, and record
  the gap in proof.md before going on.

### 4.4 Step 3 and later: the endpoint induction (O8–O12)

This is the expensive part and is not committed to yet. It needs an interval-exchange model with
labelled strips, the rewriting step, its invariant (O9) and its termination measure (O10), then
spiral contraction (O11) as an equality of loops. Estimate 3,000–6,000 lines. Start it only if
step 2 finishes within its estimate, and only if a research line is ready to submit a
replacement for L5, so that the formal statement of L5 is used as soon as it exists. O1 and O12
and the resource obligations (R1–R3) come last.

### 4.5 What the plan does not claim

A finished step 1 and step 2 prove that a first-return construction meets the specification for
every column. They prove nothing about `src/open.rs` itself, which remains covered by its
differential tests, and nothing about speed. Their use is as fixed targets: a research line
proposing a new route to L3–L5 then has a statement to prove against, and a correct route can go
straight to the i9.

## 5. Links

- The hand proof: [docs/journal/2026-11-18-open-return/proof.md](../../journal/2026-11-18-open-return/proof.md)
- The classical-tools audit, whose load-bearing test §3 adopts:
  [docs/journal/2026-11-22-classical-tools-audit.md](../../journal/2026-11-22-classical-tools-audit.md)
- The research frontier, where the alternatives in T3 are recorded: [FRONTIER.md](FRONTIER.md)
- The method: [METHOD.md](METHOD.md)
