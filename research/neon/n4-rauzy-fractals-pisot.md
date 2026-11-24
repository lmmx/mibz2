# N-4: E2 Screen 2 — Rauzy fractals and Pisot substitutions

*Directed kitchen-sink screen for Stream N (E2 open cell) under frozen parameters `K = 2` (also checked at `K = 3`), `δ = 1/2`, `S(c) ≤ openPeakScratch c`, and anti-cheat clauses 1–5 (`origin/claude/sharp-meitner-cx0v78` `4d7d83b`, `main-theorem.md` §1–1a).*

**Script:** [`research/neon/scripts/screen_cycle1_frameworks.py`](scripts/screen_cycle1_frameworks.py) (Tribonacci Pisot toy + factor complexity `p(k)` on all 6 admitted blocks of `origin/data/admitted-bwt`, both full text and Stream N with GUIDs / SHA1s stripped).

**Verdict:** **PASSES (a) Escape** (`R2` on Pisot/torus-coded words), **CLOSED at (b) Passes** (non-stationary directive acquisition from `(L, p)` reduces to Rauzy–Veech induction inside `𝓡`), and **CLOSED at (c) Viability** (even on pure Stream N with all GUIDs and SHA1s removed, factor complexity `p_{T_N}(16) = 42,600–155,526`, `1,300×–4,700×` above Pisot linear complexity `O(k)`). Status: `tested failure within defined scope`.

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - Exact inverted texts `T` of the six admitted Wikidata blocks (`origin/data/admitted-bwt`, `41c03a5b`), and two Stream-N-filtered texts: `T_N1` (each RFC 4122 v4 GUID replaced by sentinel `0x01`) and `T_N2` (GUIDs replaced by `0x01` and 40-hex SHA1s replaced by `0x02`).
   - Tribonacci Pisot substitution word (`a → ab, b → ac, c → a`, `n = 927`, `T_11`) as positive escape witness.
2. **Real code path:**
   - Baseline: `OpenCells` (`LIMIT = 16`, 1W i9 kernel time `276.23 ms` across 232 blocks; `n2-f-baseline-and-needs-m2.md`).
3. **Assumptions knowingly unrealistic:**
   - Stripping GUIDs and SHA1s into single sentinel bytes is strictly *generous* to Pisot/Rauzy-fractal viability (it removes the highest-entropy substrings for free). Failing viability even on `T_N2` therefore strictly bounds viability on the actual Stream N rows.

## 1. Directed four-gate screen

### (a) Escape: Does it give facts outside `𝓡`? — **YES** `[proved]`

For a primitive unimodular Pisot substitution `σ : A* → A*` (dominant eigenvalue `β > 1`, all `d - 1` conjugates strictly inside the unit disk) satisfying the pure discrete spectrum / coincidence condition (Rauzy 1982; Arnoux–Ito 2001), projection of the broken line onto the contracting hyperplane `H_c ≅ ℝ^{d-1}` defines the Rauzy fractal `ℛ = ⊔_{a ∈ A} ℛ_a`, and the shift is measurably conjugate to a translation `x ↦ x + v` on the torus `𝕋^{d-1} = H_c / Λ`.

- Consequently, `T[k] = a ⟺ (x_0 + k · v) mod Λ ∈ ℛ_a` determines every position directly by torus arithmetic, **outside `𝓡`** (E2 positional fact).
- As demonstrated in `docs/journal/2026-11-20-toy-transfer.md` (`toy2.py`), a torus/circle rotation coded by a many-piece partition can exhibit `Θ(n)` narrow/width-1 runs while retaining `O(1)` positional evaluation (`c₁ = 0 < h₁`) once `(v, Λ, {ℛ_a})` are known.
- **Toy verification (`screen_cycle1_frameworks.py`) `[measured]`:** Tribonacci word (`n = 927`): BWT has `r = 14`, `h₁ = 1`, and factor complexity `p(k) = {1:3, 2:5, 4:9, 8:17, 12:25, 16:33} = 2k + 1` (exact linear Pisot complexity).

### (b) Passes: Can it be computed within `K = 2` (or `K = 3`) streaming passes in `S ≤ openPeakScratch`? — **NO** `[argued]`

- For a *stationary* `d`-letter Pisot substitution of small `d`, the frequency vector `(count(a)/n)` is computable in 1 pass (`O(σ)` words) and approximates the Perron–Frobenius eigenvector, from which simultaneous Diophantine approximation recovers `v` without walking `LF`.
- However, an arbitrary finite block `(L, p)` in `𝓕` is **not** a stationary Pisot substitution: its `S`-adic directive sequence `(σ_1, σ_2, ..., σ_t)` is non-stationary. The generalised `S`-adic Rauzy fractal depends on the entire directive sequence `(σ_1, ..., σ_t)`, and recovering that sequence from `(L, p)` IS Rauzy–Veech induction — which falls back inside `𝓡` (`c₁ ≥ h₁` by G1) and violates `K = 2` (and `K = 3`).
- Moreover, by `3-byte global sensitivity` (FRONTIER, Epsilon E1), the byte frequency vector `(count(a)/n)` alone cannot determine global order on valid BWT columns (two columns at Hamming distance 3 with identical byte histograms have different global order). Thus no 1-pass frequency-only torus recovery is sound on `Column`.

### (c) Viability (the Rauzy–Veech hindsight) — **NO** `[measured]`

**Parameter cost scales with:** A `d`-letter Pisot substitution (or `M`-piece coding of a translation on `𝕋^{d-1}`) has **zero topological entropy** and **polynomial / linear factor complexity** `p_T(k) ≤ C · k` (for Pisot substitutions, `C = O(d)`). To represent a block with empirical factor complexity `p_T(k)` via a torus/fractal partition, the effective alphabet / partition complexity `M` must grow with `p_T(k)` (`d_eff = Ω(p_T(k) / k)` for linear Pisot systems, or `M = Ω(p_T(k)^{1/(d-1)})` in dimension `d - 1`).

**Measured on the 6 admitted blocks (`screen_cycle1_frameworks.py`) `[measured]`:**

| Block | `p_T(4)` Full | `p_T(16)` Full | `p_{N1}(16)` (no GUID) | `p_{N1}(16)/16` | `p_{N2}(16)` (no GUID, no SHA1) | `p_{N2}(16)/16` |
|---|---:|---:|---:|---:|---:|---:|
| Tribonacci toy (`n=927`) | 9 | 33 | — | — | — | **2.06** |
| `block-0.L` | 37,487 | 86,240 | 49,810 | 3,113.1 | 42,600 | **2,662.5** |
| `block-2.L` | 39,467 | 95,167 | 62,119 | 3,882.4 | 53,590 | **3,349.4** |
| `block-85.L` | 54,210 | 127,212 | 78,672 | 4,917.0 | 68,523 | **4,282.7** |
| `block-143.L` | 68,614 | 197,086 | 167,919 | 10,494.9 | 155,526 | **9,720.4** |
| `block-210.L` | 78,355 | 211,048 | 121,185 | 7,574.1 | 111,730 | **6,983.1** |
| `block-207.L` | 58,847 | 156,075 | 109,176 | 6,823.5 | 99,988 | **6,249.2** |

Even after stripping **all** GUIDs and **all** SHA1 hashes (`N2`), the remaining Stream N text has `p_{N2}(16) = 42,600–155,526` (`p_{N2}(16)/16 = 2,663–9,720`, exceeding `h₁N = 5,478–14,424` and comparable to `r` itself!), because Wikidata entities contain thousands of distinct Q-IDs, labels, descriptions, timestamps, and coordinates. Thus the partition/fractal complexity required to encode Stream N on a torus is **not small** on `𝓕` — it scales with `Θ(r)`, violating the Rauzy–Veech viability criterion.

## 2. Obstruction table entry

| Field | Value |
|---|---|
| **Framework** | Rauzy fractals and Pisot substitutions (`N-4`) |
| **Route class** | **R2** (escapes `𝓡` via torus translation on Pisot words), closed on `𝓕` by acquisition and viability |
| **Gate failed** | **(b) Passes** (non-stationary `S`-adic acquisition reduces to RV in `𝓡`; 1-pass frequency recovery blocked by 3-byte global sensitivity) and **(c) Viability** (factor complexity explosion on Stream N) |
| **Witness** | All 6 admitted Wikidata blocks (`block-0.L`..`block-210.L`), even after stripping GUIDs and SHA1s: `p_{N2}(16) = 42,600–155,526` (`p(16)/16 = 2,663–9,720` vs `2.06` for Tribonacci) |
| **Scope** | All stationary Pisot / Rauzy-fractal torus-translation decoders and low-dimensional zero-entropy symbolic-dynamics coordinate models applied to `𝓕` (full block or Stream N) |
| **Escape condition** | Restrict to a zero-entropy / bounded-factor-complexity sub-language (which fails applicability on `𝓕`), or supply a positive-entropy non-torus coordinate mechanism whose parameter is `≪ h₁` on `𝓕` |
| **Reach** | Global over low-dimensional zero-entropy torus codings (`p(k) = O(k^{d-1})`) |
| **Strength** | Argued (acquisition reduction + 3-byte sensitivity) and measured (exact factor complexity on 6 admitted blocks + Tribonacci toy) |

## 3. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact inverted texts and stripped Stream N texts on all 6 admitted blocks.
2. **Model assumptions:** Using `p(k)/k` as the viability proxy for Pisot/Rauzy-fractal complexity is conservative (any exact `d`-letter Pisot substitution satisfies `p(k) = O(k)`).
3. **Method:** Positive escape verification on Tribonacci (`p(k) = 2k+1`), acquisition check against `𝓡` and 3-byte sensitivity, and direct factor-complexity measurement on `T`, `T_N1`, and `T_N2`.
4. **Outcome:** Genuine R2 escape on toy Pisot words, cleanly killed at (b) and (c) on `𝓕` even for pure Stream N. No premise `P_N` survives.
