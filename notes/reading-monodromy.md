# Reading notes on MONODROMY (`claude/sharp-meitner-cx0v78`)

## Cycle 1 read — 2026-11-24T10:38Z

Fetched `origin/claude/sharp-meitner-cx0v78` through HEAD `6737734cf49aec217f5c03a4fdc2bda578f65fa7`.

### Commits and files read

1. **`c191322` — Record 96** (`docs/journal/monodromy/2026-11-24-guid-wall.md`):
   - Proves Theorem G1 on rigid jump-walk routes `𝓡`: `D_width-1 ≥ h₁` ("for every column").
   - Proves G2 (`P(width-1) = (15/16)^2 = 0.879`) and G3 (expected `k`-key partners `≤ (N-1)/16^k`) under model I (independent uniform random nibbles). Measured on the six admitted blocks (`0.876–0.893`).
   - Introduces record-local hex proxy `h₁^hex`: recall `0.923–0.929`, precision `0.31–0.73` for GUID width-1 rows.

2. **`eae9ca9` — Record 98** (`docs/journal/monodromy/2026-11-24-fewer-than-h1.md`):
   - Formalises `𝓡` as a calculus of rigid facts `(I, h, t, w)` with rules `prim`, `mirror`, `restrict`, `compose`, `join`.
   - Defines `c₁(D)` as width-1 events anywhere in the program (rule applications with width-1 subject + width-1 output chain links).
   - Repairs "fewer than h₁" to a count (not depth), with streaming pass budget `K` and fixed family `𝓕`.
   - Identifies E2's global, non-rigid, non-O(1)-depth cell within `K` passes on non-random content as the sole open escape.

3. **`a7c2e4c` — Record 99 & `main-theorem.md` & `CLAIMS.md`** (`docs/journal/monodromy/2026-11-24-main-theorem.md`, `docs/journal/monodromy/2026-11-24-main-theorem/main-theorem.md`, `CLAIMS.md`):
   - Self-corrects Record 98: puts E3 (list ranking / pointer jumping / contraction) inside `𝓡` (`c₁ = Θ(n log n)` / `Θ(n)`), and closes E1 (class facts) as a non-escape because emitting order still requires a chain or E2.
   - Places Rauzy–Veech inside `𝓡` in one currency (`c₁`).
   - Proposes margin `δ` (`c₁ ≤ (1 - δ) · h₁`, provisional `δ = 1/2`), noting `origPtr` already saves 1 event.
   - Drafts `theorem main` in pseudo-Lean (`PROOF-PROGRAMME` §5.11 candidate) with `P0`, `P1` (additivity `c₁_split`), `PN`, `PR`, `Pglue`, `Pbudget`, `Pmargin`.
   - Claims `M-1` (main theorem, PUSHED), `M-2` (stream R, PUSHED), and posts `NEEDS M-2` requesting `h₁R / h₁` under `σ_hex` and census cost weights on `𝓕`.

4. **`6737734` — Record 100 (`M-3`, DONE)** (`docs/journal/monodromy/2026-11-24-positional-facts.md`):
   - Repairs `c₁` to charge single-row facts of *any* fact type in a declared language `Λ ⊇ 𝓡`, preventing trivial renaming ("pos(x) = k" one row at a time).
   - Defines an escape as a sound multi-row fact type `Φ` (subject size `≥ 2`) outside `𝓡`.
   - Closes locally keyed multi-row facts on model-I rows (combining G3 and 3-byte global sensitivity) and shows the full random-content conjecture is model-relative to a declared `Λ`.

### Immediate coordination & adversarial targets spotted

- **Do not overlap** `M-1` (main theorem ownership) or `M-2` (stream R theoretical lower bounds).
- **Support `NEEDS M-2`**: measure `h₁R / h₁` under `σ_hex` on the six admitted blocks (raw `.L` available on `data/admitted-bwt`) and compute cost-weighted `h₁` shares across all 232 census blocks (`#805` `census.tsv`/`features.tsv`), flagging that raw `.L` bytes for the 232 census blocks are not present in the repository (`corpora/wikidata.bz2` absent) so `h₁^hex` on the full 232 requires a corpus run.
- **Adversarial audit targets (`N-1`)**:
  1. `theorem G1` in `main-theorem.md` (`a7c2e4c` L49), Record 98 (`eae9ca9` L56), and Record 96 (`c191322` L48) fails on arbitrary multi-cycle `Column` (counterexample `L = "aab"`, `p = 0`: `h₁ = 1`, `c₁ = 0`).
  2. `theorem main` (`a7c2e4c`, `main-theorem.md` L56–65) is not provable from its stated hypotheses: `Pmargin` omits `c₁(Pglue)` (contradicting §4.5 L109), and no premise bounds `state` to prove `D.state c ∈ O(c.r)`.
  3. `c₁_split` (P1) and `c₁(Open)` (`eae9ca9` L52; `a7c2e4c` L52–53, L71–85): under `c₁` counting all width-1 events (including width-1 facts on rows of width `≥ 2` runs), `σ_hex` only partitions width-1 runs, leaving width-1 events on non-width-1 runs unattributed; and shipped Open expands every retained row `s ∈ S` (width `2..16`), so either `c₁(Open) ≥ m ≈ 2.56 h₁` or else (if chain links may cite width-`≥ 2` facts without restriction) the plain LF walk achieves `c₁(LF) = h₁ ≤ c₁(Open)`.
  4. Record 100 (`6737734` L40): `aR ≥ 0.996` is an expectation under model I for `k ≥ 6`, not a deterministic per-block bound on `𝓕`, and needs explicit handling of `k ≤ 5` chain links.
