# N-13: Supplied Capability 2 (Outside Review §5.2) — Compressed `DSA` (`SA[i] - SA[i-1]`) with Paid Anchors and Kempa Theorem 5.1 Audit

*Cycle 3 supplied-capability measurement answering Coordinator Note #3 / Outside Review §5.2 (`origin/claude/sharp-meitner-cx0v78` `e48f904`): measure `DSA` compressibility `g_{Δ,N}`, `LF`-invariance breaks, distinct values & entropy on `h₁` and `m₁₆`, and audit Kempa Theorem 5.1 against `K = 2` and `S ≤ openPeakScratch`.*

**Script:** [`research/neon/scripts/screen_cycle3_supplied.py`](scripts/screen_cycle3_supplied.py) (exact `DSA[i] = SA[i] - SA[i-1]` census on all 6 admitted blocks of `origin/data/admitted-bwt`).

**Verdict:** **CLOSED at (c) Viability (incompressibility of `DSA` on `h₁` and Stream N) and (b) Passes (Kempa Theorem 5.1)** — `tested failure within defined scope`.
- `DSA[LF(i)] ≠ DSA[i]` breaks at **`1.000 × r`** (every single BWT run boundary on 6/6 blocks!), and `DSA` has **`887,651–899,575` equal-value runs** out of 900,000 rows (`98.6–99.96%` singletons).
- Restricted to `h₁` width-1 rows, **`90.7–94.2%` of all `DSA` values are completely distinct** (`H₀ = 14.79–16.30 bits`, near the `log₂(h₁) = 15.1–16.6 bit` maximum!). Even restricted to **Stream N (`nonhex_w1`)**, **`86.9–93.4%` are completely distinct** (`H₀ = 11.69–13.59 bits`).
- Kempa Theorem 5.1 at optimal `τ = √(n/r) ∈ [2.6, 4.2]` requires **`429k–681k` words** of workspace and **`23.5M–29.0M` operations** (`26–32 × n`), failing `K = 2` and `S ≤ openPeakScratch`.

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - Exact `SA`, `LF`, `r`, `h₁`, `h₁N` (`nonhex_w1`), and `m₁₆` on all 6 admitted blocks (`origin/data/admitted-bwt`, `41c03a5b`).
2. **Real code path:**
   - Baseline `OpenCells` (`LIMIT = 16`, `K = 2`, `S ≤ openPeakScratch`, `d(D, c) ≤ d(Open, c)`).
3. **Assumptions knowingly unrealistic:**
   - Measuring `DSA` distinct-value counts and Shannon `H₀` entropy on an **oracle-supplied** `SA` array sets an information-theoretic lower bound on `g_Δ` (zero acquisition cost assumed).

## 1. Exact `DSA` compressibility & Kempa Theorem 5.1 audit on the 6 admitted blocks (`[measured]`)

| Block | `r` | `LF`-breaks (`DSA[LF(i)] ≠ DSA[i]`) | `DSA` runs (of 900k) | `w1` distinct `DSA` (`%h₁`) | `w1` `H₀` (bits) | `nonhex_w1` distinct (`%h₁N`) | `nonhex_w1` `H₀` (bits) | `m₁₆` distinct (`%m₁₆`) | Kempa 5.1 opt `τ` / ws (words) / ops |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `block-0.L` | 51,184 | 51,184 (`1.00r`) | 887,651 | 32,198 (`91.7%`) | 14.79 | 4,950 (`90.4%`) | 11.69 | 43,962 (`48.9%`) | `4.2` / `429k` / `23.5M` |
| `block-2.L` | 54,532 | 54,532 (`1.00r`) | 894,023 | 32,279 (`94.2%`) | 14.87 | 5,485 (`91.3%`) | 12.04 | 47,495 (`42.4%`) | `4.1` / `443k` / `23.8M` |
| `block-85.L` | 79,375 | 79,375 (`1.00r`) | 899,575 | 51,532 (`92.8%`) | 15.57 | 8,996 (`93.4%`) | 13.02 | 71,662 (`45.8%`) | `3.4` / `535k` / `25.9M` |
| `block-143.L` | 108,335 | 108,335 (`1.00r`) | 897,583 | 67,537 (`93.1%`) | 15.96 | 12,756 (`93.4%`) | 13.45 | 94,875 (`48.2%`) | `2.9` / `624k` / `27.8M` |
| `block-210.L` | 128,732 | 128,732 (`1.00r`) | 897,031 | 88,069 (`90.7%`) | 16.30 | 13,453 (`93.3%`) | 13.59 | 112,787 (`55.2%`) | `2.6` / `681k` / `29.0M` |
| `block-207.L` | 87,111 | 87,111 (`1.00r`) | 897,073 | 57,221 (`91.7%`) | 15.67 | 10,962 (`86.9%`) | 13.06 | 76,175 (`56.0%`) | `3.2` / `560k` / `26.4M` |

## 2. Why compressed `DSA` fails viability and acquisition (`[proved]` + `[measured]`)

1. **`DSA` is incompressible on `h₁` AND on Stream N (`nonhex_w1`) `[measured]`:**
   - Outside Review §5.2 asked whether `g_{Δ,N} + s_N ≪ h₁N` (since `DSA[LF(i)] = DSA[i]` inside BWT runs).
   - Because every width-1 run IS a BWT run boundary (`lf_breaks = 1.000 × r` on 6/6 blocks!), `DSA` resets at **every** width-1 run. Measuring the actual values shows **90.7–94.2%** of `DSA` values on `h₁` (and **86.9–93.4%** on Stream N `nonhex_w1`) are **completely distinct** integers with `11.7–16.3` bits of entropy each!
   - Even on `m₁₆` (all thin rows), `43,962–112,787` distinct `DSA` values exist (`> h₁` on every single block!). Thus **`g_Δ ≥ 0.91 · h₁`** even with **zero** anchors `s_N` charged — completely failing the Rauzy–Veech viability condition `g_Δ + s ≪ h₁`.
2. **Kempa Theorem 5.1 fails both workspace `S` and pass budget `K = 2` `[proved]`:**
   - Kempa (SODA 2019, Thm 5.1) constructs `SA`/`ISA` support from RLBWT in `O(n/τ + rτ)` workspace and `O((n/τ)(log(rτ) + log(n/τ)) + τ² r log(rτ))` time.
   - Because `r/n = 0.057–0.143` is large on Wikidata (unlike highly repetitive pangenomes where `r/n ≈ 10⁻⁴`), balancing `n/τ = rτ` forces `τ = √(n/r) ∈ [2.6, 4.2]`, giving `429k–681k` words of workspace (`~2–3 MB` before constants) and `23.5M–29.0M` operations (`26–32 × n`, `≫ K = 2` passes), plus random output placement overhead.

## 3. Obstruction table entry

| Field | Value |
|---|---|
| **Framework** | Compressed `SA`/`ISA` differences (`DSA`) with paid anchors & Kempa Thm 5.1 (`N-13`, Outside Review §5.2) |
| **Route class** | **R2** (if `DSA` were compactly acquirable) |
| **Gate failed** | **(c) Viability** (`90.7–94.2%` of `w1` `DSA` values distinct, `86.9–93.4%` on `nonhex_w1`; `lf_breaks = 1.000r`; `dsa_runs = 98.6–99.96%` of `n`) and **(b) Passes** (Kempa 5.1: `23.5M–29.0M` ops, `429k–681k` words) |
| **Witness** | All 6 admitted blocks (`screen_cycle3_supplied.py`): `32,198–88,069` distinct `DSA` values on `h₁` (`H₀ = 14.8–16.3` bits) |
| **Scope** | All differential suffix-array (`DSA`/`Φ`-difference) representations and RLBWT suffix-sampling schemes (r-index, Kempa Thm 5.1) on `𝓕` |
| **Escape condition** | Leave suffix-order coordinate differences `SA[i] - SA[i-1]` (which shatter at `1.000r` run boundaries); find a coordinate representation whose differences are invariant across non-adjacent rows |
| **Reach** | Global for `DSA` representations on `𝓕` |
| **Strength** | Measured exact on 6 admitted blocks + argued (Kempa Thm 5.1 parameter scaling) |

## 4. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact `DSA`, `LF`-breaks, distinct counts, and `H₀` on all 6 admitted blocks.
2. **Model assumptions:** Zero acquisition cost assumed for `DSA` viability test.
3. **Method:** Exact `O(n)` `DSA` census + analytical evaluation of Kempa Theorem 5.1 bounds.
4. **Outcome:** Decisively closes Outside Review §5.2: `DSA` does not compress on `h₁` or Stream N (`87–94%` distinct values, `1.000r` LF-breaks).
