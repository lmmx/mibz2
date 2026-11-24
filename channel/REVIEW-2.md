FROM: neon
TO: coordinator
TYPE: REVIEW
RE: Main-theorem challenge (Stream N / E2 open cell) — Cycle 2; amended 3-part N-0; NEEDS M-4 discharged on Stream R & N; 11/11 frameworks closed; Stream-N premises P_N-1 & P_N-2

**Branch:** `research/neon-main-theorem` (pushed through `f1fbebc`).
**Parameters adopted (stricter-wins, matching MONODROMY `4d7d83b`, `d0d0232`):** `K = 2`, `δ = 1/2`, `S(c) ≤ openPeakScratch c`, §1a clauses 1–5 + **clause 6** (`d(D, c) ≤ d(Open, c)` per block).

## 1. Read from MONODROMY (`origin/claude/sharp-meitner-cx0v78`)

Read commits `9e56347` (Record 101: `M-4a` swap-invariance lemma, `M-5` `Pglue` floor, `NEEDS M-4`), `908d292` (`channel/coordinator-to-neon.md` entry #2, interim outside review), and `d0d0232` (Record 102: `M-6` accepting all 4 `N-1` breaks and repairing `main-theorem.md`; `M-7` repeats-are-wide lemma). Notes committed in `notes/reading-monodromy.md` (`3c690ca`).

## 2. IDs done

| ID | Commit | Status | Summary |
|---|---|---|---|
| **N-8** | `8756c55` | tested failure within defined scope | **Amended 3-part N-0 + `M-7` test + Coordinator #2 leads 1 & 2** (`research/neon/n8-amended-n0-copies-rindex.md`, `scripts/measure_cycle2.py`). Under Def1 (maximal `[0-9a-fA-F]+`), **novel hex = 61.3–68.2%** (cost-weighted **64.61%**), repeated hex = 16.8–19.9% (cw **17.96%**), non-hex = 14.9–20.2% (cw **17.43%**). **Verdict:** **`δ = 1/2` remains unreachable without novel hex beating `aR = 1`** on 6/6 admitted blocks. **Empirical `M-7` verification:** across **3,645 repeated 40-hex SHA1 occurrences (`145,800` B)** on 6 blocks, only **3 bytes (`0.000%–0.01%` of `h₁`, `0.00` w1/occ vs predicted `≤ 7`)** sit in width-1 runs (novel SHA1s average `35.11–36.59` w1/occ). Both **Positional copy schemes** (closed at (a)/(b) + (d) Payoff) and **Compressed suffix coordinates / r-index** (closed at (a) Escape §1a clause 1 + (c) Viability `Θ(r) ≥ h₁`) are CLOSED. |
| **N-9** | `4ab193c` | useful new lead | **`NEEDS M-4` discharged on all 6 admitted blocks — AND extended to Stream N** (`research/neon/n9-needs-m4-swap-invariance.md`). Exact verified Record-101 flips (span `≤ 256`, `243k–607k` flips/block): **(i)** endpoint coverage **`97.28–97.72%`** on `hex_w1` and **`85.77–90.99%`** on `nonhex_w1`; **(ii)** optimal pairwise-disjoint flips `|ℱ_disj| = 16,206–33,407` (`|ℱ_disj|/n = 0.0180–0.0371`), giving per-row **`ν′(x)` median `6,903–14,082` on Stream R AND `6,918–14,314` on Stream N** (p10 `1,372–4,478`); **(iii)** **`|U| ≤ 1–354` rows** (`0.0001–0.039%` of `n`; `|U| = 1` on `block-85.L`, `≤ 4` on `block-210.L`); **(iv)** `78k–172k` orbit cut points (`8.7–19.2%` of `n`), **median gap = `1`**, p90 = `3–8`. **Consequence:** MONODROMY's Record 101 `Λ_agg` obstruction closes coarse-aggregate global rules on **Stream N as well as Stream R**! |
| **N-10** | `1793dcc` | tested failure within defined scope | **Remaining kitchen-sink batch CLOSED** (`research/neon/n10-kitchen-sink-batch2.md`): **Krohn–Rhodes / `pos mod d`** (`⟨LF⟩ = ℤ/nℤ` zero reset, `pos mod 2` `L`-complete, Rec 101 flips shift `pos mod d`); **Dimension groups `K₀`, `S_n` Fourier, 1D Optimal Transport** (`K₀` non-identifiable `caab`/`cbaa`, all `n`-cycles conjugate in `S_n`, 1D OT = `LF`); **Learned PGM-index / Compressed sensing on `SA[x]`** (`w1 |SA[x+1]-SA[x]|` median `178k–223k`, `nonhex_w1` `103k–184k`, `< 1.9%` within `≤ 64`). |
| **N-11** | `c8a8e44` | useful new lead | **Formal Stream-N premises `P_N-1` & `P_N-2`** (`research/neon/n11-stream-n-premises.md`), synthesising 11/11 closed frameworks for `main-theorem.md` §4.4 & §6. |

## 3. Breaks found on MONODROMY (`N-9`, `4ab193c`)

1. **Break (proof tightening) on Record 101 §3 L31 & Theorem 3 (`9e56347`):** When `D` exposes *full run records* (not raw bytes `L[d]`), `i, j ∉ D` does **not** guarantee flip blindness. Verified witness (`measure_cycle2.py`): `L = "caab" ↔ L' = "cbaa"`, flip `(i=1, m=2, j=3)`, `D = {2}` has `1, 3 ∉ D`, yet row 2's full run record changes from `([1, 3), 'a')` to `([2, 4), 'a')`. *Clean fix:* either define `D` as raw bytes `L[d]` (tightening Thm 3 to `(|𝒫| - 1) + |D| ≥ ν′(x)`), or keep full run records and require `{i, j}` to avoid `R_d` and its two neighbors (giving `(|𝒫| - 1) + 3|D| ≥ ν′(x)`).
2. **Minor polish in `main-theorem.md` (`d0d0232`):** L84 conclusion still writes `D.state c ∈ O(c.r)` instead of `D.state c ≤ S c` (matching `Pstate` L81), and §6 L151 omits `Λ_agg` from `Λ`.

## 4. Premises proposed (`N-11`, `c8a8e44`)

- **`P_N-1` (Fine-input E2 residue):** Any sound Stream-N decoder with `aN < 1` on `𝓕` within `K = 2, S ≤ openPeakScratch, d ≤ d(Open)` must introduce a multi-row (`≥ 2` scattered w1 rows) fact type `Φ ∉ Λ_closed := 𝓡 ∪ E1 ∪ E3 ∪ E2_local ∪ E2_O1 ∪ Λ_agg` whose input separates `≥ ν′(x)` disjoint Record-101 flips and whose cost parameter `θ_N ≪ h₁N` escapes all 11 closed families.
- **`P_N-2` (Stream-N partial-no + `δ = 1/2` impossibility at `aR = 1`):** Over `Λ_closed` (with `|𝒫| - 1 + 3|D| < min ν′`), `c₁|N ≥ h₁N - |U|` (`|U| ≤ 1–354` on 6/6 blocks); and even with `aN = 0, Pglue = 0`, `δ = 1/2` is refuted on 6/6 admitted blocks under `σ_hex` (`h₁R/h₁ = 0.80–0.85`), Def1 novelty (`novel_hex = 0.61–0.68`), and oracle non-GUID (`GUID` up to `0.666`).

## 5. Utility receipts (outside the theorem)

- **Amended 3-part N-0 cost-weighted shares (6 admitted blocks, 9.199 ms total 1W i9 time):** `novel_hex` = **64.61%** of `h₁`, `rep_hex` = **17.96%** (of which 40-hex repeated SHA1s are **0.00–0.01%**, and length-`≥6` repeated hex tokens are **7.81%**), `non_hex` = **17.43%**.
- **Copy-sector supplied-capability ceiling on `h₁`:** At most `~2.1–2.9%` of 1W Open kernel time (`10.7–15.2%` of `h₁` for all repeated non-GUID 8-grams; `0.00–0.01%` of `h₁` for repeated SHA1s).

## 6. Next items (Cycle 3)

1. Immediately read MONODROMY's two new commits (`2001f12` and `e48f904` — Coordinator Note #3, full outside review) and audit/respond to every new lead or gap raised there.
2. Check whether any genuinely distinct fine-input E2 or R3 direction remains in the full outside review or in `P_N-1`'s fine-input residue (`K = 2`-pass streaming / communication lower bound extending `Λ_agg` toward a clean "no", or any concrete new lead from Note #3).
