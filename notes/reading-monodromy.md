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

## Cycle 1 second read — 2026-11-24T10:45Z

Fetched `origin/claude/sharp-meitner-cx0v78` commits `4d7d83b`, `a96e969`, `dd1d6ea`:

1. **`a96e969` — `channel/coordinator-to-neon.md`:**
   - Requests `N-0`: measure `h₁R / h₁` under `σ_hex` across `𝓕` (per block + cost-weighted aggregate) and state whether `δ = 1/2` is reachable without stream R beating `aR = 1`.
   - Note: we already computed this in `N-2` (`656d5e7`): exact on the 6 admitted blocks (`0.7978–0.8514`, cost-weighted `0.8256`), with `#805` 232-block census weights computed and `corpora/wikidata.bz2` noted absent locally for per-byte `h₁R` on the remaining 226 blocks. Verdict: **`δ = 1/2` is unreachable without `aR < 1`** (even under an oracle GUID/non-GUID split, since `block-0.L` has 66.6% GUID width-1 runs).
   - Directs Neon & MONODROMY to fix `δ, K, 𝓕, S` under the stricter-wins rule; `Pglue` is fair game for Neon.

2. **`4d7d83b` — Frozen parameters in `main-theorem.md` §1 & §1a:**
   - `K = 2` (stricter than provisional `K = 3`), `δ = 1/2`, `S(c) ≤ openPeakScratch c`, plus anti-cheat clauses 1–5 (`c₁` charges every fact type; no bundling; no off-ledger cost; depth ≠ count; never relax under C2).
   - **Adopted under stricter-wins:** all Neon E2 screens (`N-3`..`N-7` and subsequent cycles) now screen against **`K = 2`** (while also recording `K = 3` where relevant so no result is ambiguous), **`δ = 1/2`**, **`S ≤ openPeakScratch`**, and anti-cheat clauses 1–5.

3. **`dd1d6ea` — `docs/journal/monodromy/HANDOFF-2026-11-24.md`:**
   - Notes MONODROMY's next item `M-4` (swap-invariance lemma for aggregate-only global rules on stream R) and confirms our `N-2` finding on `Pglue` (MONODROMY estimates ~10 switches per GUID ≈ 1/3 of `h₁R`; our `N-2` measured exact `all_sw / h₁ = 0.513–0.543` and `w1_switches / h₁N = 1.19–1.63` on all six admitted blocks!).
   - Also notes that anti-cheat clause 2 ("distinguished input") is informal and collapses for general global rules — consistent with our `N-1` audit.

## Cycle 2 read — 2026-11-24T10:54Z

Fetched `origin/claude/sharp-meitner-cx0v78` through HEAD `d0d0232` (`9e56347`, `908d292`, `d0d0232`):

1. **`9e56347` — Record 101 (`M-4a`, `M-5`):**
   - Proves the **Swap-Invariance Lemma** (`docs/journal/monodromy/2026-11-24-swap-invariance.md`): a valid count-preserving flip is a Hamming-distance-2 byte transposition `i < j` with one intervening `a`-or-`b` row `m`, giving `σ' = σ ∘ (3-cycle)` valid iff orbit order matches, exchanging two adjacent text blocks. Lemma 1: `pos'(x) = pos(x)` iff `x` lies in `origPtr`'s arc.
   - **Theorem 3 (`Λ_agg(𝒫, D)`):** any sound coarse-aggregate rule placing `x` needs `|𝒫| + 2|D| > ν′(x)`. Requests **`NEEDS M-4`** (measure valid flip coverage, greedy disjoint-span `ν′/n`, `|U|`, and orbit cut-point gaps on the 6 admitted blocks).
   - **M-5 (`Pglue` floor):** `c₁ ≥ seg_R(c) - |U|` over `𝓡 ∪ local ∪ Λ_agg`; predicts `aR ≥ ~0.29` on GUIDs from glue alone.

2. **`908d292` — `channel/coordinator-to-neon.md` entry #2 (interim outside review):**
   - Amends `N-0`: report `h₁` in 3 parts per block (**novel hex**, **repeated hex**, **non-hex**) with kernel-time-weighted aggregate.
   - Adds §1a clause 6 (adopted stricter by MONODROMY): a yes must state dependent loads per block `d(D, c) ≤ d(Open, c)`.
   - Flags two logical gaps in `main-theorem.md` (already caught in `N-1` and repaired in `M-6`).
   - Prioritises screening **positional copy schemes** (r-index `φ`/toeholds) and **compressed suffix coordinates** (sampled SA, r-index) before the remaining kitchen sink.

3. **`d0d0232` — Record 102 (`M-6`, `M-7`, repaired `main-theorem.md`):**
   - Accepts all 4 of Neon's `N-1` breaks (`8ec0f0b`) and repairs `main-theorem.md` (`singleCycle` in `G1`, `c₁(Pglue)` in `Pmargin`, `Pstate`, run-based attribution for `Mirror`). Withdraws `NEEDS M-5` as answered by `N-2` (`656d5e7`).
   - Proves **M-7 (repeats-are-wide lemma)** under model I: at most `log₁₆ N + 16/15 + 1 ≈ 6–7` width-1 rows per repeated hex occurrence. Argues positional copy schemes and r-index `φ` do not enter the open cell on novel content.

### Cycle 2 coordination & adversarial targets spotted

- **Adopt §1a clause 6** (`d(D, c) ≤ d(Open, c)` per block) under stricter-wins.
- **Do not duplicate `M-4a` or `M-7`:** fulfill `NEEDS M-4` directly (and test whether `ν′(x) = Θ(n)` extends `Λ_agg`'s closure to Stream N!), fulfill amended 3-part `N-0` + test `M-7`'s `≤ 7` prediction empirically, and audit Record 101 Theorem 3's run-record blindness condition (`L = "caab" ↔ "cbaa"`, `D = {2}` breaks `i, j ∉ D` when `D` exposes full run records rather than raw bytes).

## Cycle 3 read — 2026-11-24T11:03Z

Fetched `origin/claude/sharp-meitner-cx0v78` commits `2001f12` and `e48f904`:

1. **`2001f12` — Record 102 §3 & `main-theorem.md` (`c₁` convention fixed):**
   - Adopts Neon's stricter option (ii) from `N-1` break 3.3: `c₁` counts every 1-row-subject event in runs of *any* width, and links always count (`c₁(LF walk) = n`, `c₁(Open) ≥ m`, Record 98 L52 superseded).
   - Makes `σ` three-way `(R, N, W)` and adds `c₁W c` to `Pmargin`.
   - *Minor audit note:* `main-theorem.md` L82 introduces `c₁W` as an unbound symbol in `theorem main` (needs a `DW` / `W`-stream premise or definition) and L84 still prints `D.state c ∈ O(c.r)` instead of `≤ S c`.

2. **`e48f904` — `channel/coordinator-to-neon.md` entry #3 & `channel/outside-review-2026-11-24.md` (full ChatGPT Pro review):**
   - Recommends keeping `c₁` as rigid-calculus diagnostic while adding a dependent-miss conjunct `D_𝓗` (matching §1a clause 6), replacing finite `𝓕` with a uniform structural family (to exclude `𝓕`-indexed lookup advice), defining what `K` counts (working-array scans/sorts), and fixing the "no" to cover global E2.
   - Directs Stream N screening of **three specific supplied capabilities** (measuring descriptor/anchor/exception counts on expensive rows `h₁` and `m₁₆` and full-path payoff *before* acquisition work):
     1. Acyclic positional copy plans (LZ77 / Kempa–Prezza attractors and macro schemes, `q_copy`);
     2. Compressed `SA`/`ISA` differences (`DSA`) with paid anchors (`g_{Δ,N} + s_N`, Kempa Thm 5.1 audit);
     3. Ordered return-word hierarchies with paid exceptions (`(s, d, e)`, Durand / S-adic).
   - *Peer-claim check:* our `N-8` (`8756c55`) directly rebuttals the review's concern (§4 L134–136) that `σ_hex` mixes in large repeated SHA1 width-1 content: repeated 40-hex SHA1s contribute only `3 / 145,800 B` (`0.00–0.01%` of `h₁`), and novel hex alone is `61.3–68.2%` of `h₁`. Its Model-I run-length sampling bias catch (L141: size-biased `k(1-p)²p^{k-1}` vs geometric `(1-p)p^{k-1}`) is mathematically exact (Record 96 L74 mixed them for `w ≥ 2`, while `w = 1` was right).

### Cycle 3 second read — 2026-11-24T11:08Z

Fetched `origin/claude/sharp-meitner-cx0v78` commit `620eacc` (Record 103, `M-9`):
- Audited `M-9` scoped provenance proof of Theorem G1 (`docs/journal/monodromy/2026-11-24-g1-scoped.md`): **verified, 0 breaks found** (Lemma A, Corollary `|I| = 1`, DAG induction on `P(f, y, j)` is row-preserving and handles `prim`/`restrict`/`join`/`compose` cleanly).
- Adopted §1a clauses **6b** (`D_𝓗(D, c) ≤ D_𝓗(Open, c)`) and **7** (no advice; uniform family `𝓖(S, m, k)` of Record 103 §4, validated on `𝓕`) under stricter-wins.
- Noted `main-theorem.md` minor polish items (`c₁W` unbound in `theorem main` L82, `D.state c ∈ O(c.r)` in L84) and that `N-9` (`4ab193c`) already discharged `NEEDS M-4` referenced in Record 103 §4.
