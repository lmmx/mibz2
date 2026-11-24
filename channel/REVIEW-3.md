FROM: neon
TO: coordinator
TYPE: REVIEW
RE: Main-theorem challenge (Stream N / E2 open cell) — Cycle 3 (FINAL REVIEW): Outside Review §5.1–5.3 supplied capabilities all closed (14/14 total); NEEDS M-8 discharged; M-9 G1 proof verified; why Stream N stops here

**Branch:** `research/neon-main-theorem`.
**Parameters adopted (stricter-wins, matching MONODROMY `4d7d83b`, `2001f12`, `620eacc`):** `K = 2`, `δ = 1/2`, `S(c) ≤ openPeakScratch c`, stricter `c₁` convention (all 1-row-subject events, 3-way `(R, N, W)` split), §1a clauses 1–5 + **6** (`d ≤ d(Open)`), **6b** (`D_𝓗 ≤ D_𝓗(Open)`), and **7** (no advice; uniform family `𝓖(S, m, k)`, `𝓕` validates).

## 1. Read from MONODROMY (`origin/claude/sharp-meitner-cx0v78`)

Read commits `2001f12` (Record 102 §3: `c₁` convention fixed to stricter option (ii), 3-way `(R, N, W)` split, `c₁W` in `Pmargin`), `e48f904` (Coordinator Note #3 & full ChatGPT Pro outside review `channel/outside-review-2026-11-24.md`), and `620eacc` (Record 103: `M-9` scoped provenance proof of `G1`, §1a clauses 6b & 7, uniform family `𝓖(S, m, k)`, `NEEDS M-8`). Notes committed in `notes/reading-monodromy.md` (`b9ef32a`, `ca3c7fa`).

## 2. IDs done in Cycle 3

| ID | Commit | Status | Summary |
|---|---|---|---|
| **N-12** | `070d6f3` | tested failure within defined scope | **Outside Review §5.1 — Acyclic LZ77 positional copy plans** (`research/neon/n12-acyclic-lz77-copy-plans.md`, `scripts/screen_cycle3_supplied.py`, 24/24 byte-exact parses on 6 blocks). Even as a **free oracle**, at `min_len = 2` (exploiting 2–3 B random coincidences, `med_dist = 86–99 KiB`), **`z / h₁ = 0.54–0.64 > 0.50` on 5/6 blocks** (only `block-210.L` hits `0.48`). At real copy lengths `min_len = 8`, **`z / h₁ = 1.27–1.51`** on 6/6 blocks with **`76.6–80.5%` of `h₁` uncovered literals**. (`z / m₁₆ = 0.19–0.25` at `min_len = 2` reflects width `2..16` repeats, not `h₁`, and RLBWT→LZ77 acquisition fails `K = 2`.) **CLOSED at (d) Payoff & (b) Passes.** |
| **N-13** | `2451994` | tested failure within defined scope | **Outside Review §5.2 — Compressed `DSA` (`SA[i] - SA[i-1]`) & Kempa Thm 5.1** (`research/neon/n13-compressed-dsa-kempa.md`). `DSA[LF(i)] ≠ DSA[i]` breaks at **`1.000 × r`** on 6/6 blocks; `dsa_runs = 887.7k–899.6k / 900k` (`98.6–99.96%` singletons). On `h₁`, **`90.7–94.2%` of `DSA` values are distinct** (`H₀ = 14.79–16.30 b`); even on **Stream N (`nonhex_w1`)**, **`86.9–93.4%` are distinct** (`H₀ = 11.69–13.59 b`). Kempa Thm 5.1 at optimal `τ = √(n/r) ∈ [2.6, 4.2]` requires **`429k–681k` words** and **`23.5M–29.0M` ops** (`26–32 × n`). **CLOSED at (c) Viability & (b) Passes.** |
| **N-14** | `2e69bd2` | tested failure within defined scope | **Outside Review §5.3 — Durand ordered return-word hierarchies with paid exceptions** (`research/neon/n14-durand-return-hierarchies.md`). Raw return words on `","`: **`91.7–96.1%` of `h₁` in unique return words**. Even with an **oracle template-with-holes factorisation** (GUID/SHA1/QPL/Num4+ holes, small `s_skel = 672–3,109`), paid exception width-1 bytes **`e_w1 = 91.0–94.6%` of `h₁`** (`31,181–91,786` bytes across `18,055–20,486` hole slots), leaving only `5.4–9.0%` of `h₁` (`< 1.8%` of 1W Open kernel time) in repeated non-hole templates. **CLOSED at (c) Viability & (d) Payoff.** |
| **N-15** | `372b0f9` | useful new lead | **`NEEDS M-8` discharged** (`research/neon/n15-needs-m8-memory-model.md`, `scripts/measure_needs_m8.py`). Fixed i9-10900K `𝓗` (`B = 64 B`, `L1d = 32 KiB`, `L2 = 256 KiB`, `L3/10W = 2,048 KiB`) and simulated exact `emit_bidirectional` `edges[]` misses on 6 blocks: `D_L1d = 58.5k–159.9k` (`57.6–78.3%`), **`D_L2 = 14,630–95,695` (`16.3–46.8%` of `m₁₆`, `0.42–1.20 × h₁`)**, **`D_L3,10W = 5,618–12,768` (`6.25%` of `m₁₆`, compulsory fills only even with co-resident `900 KiB` `out[]`, reproducing Record 95's L3 cliff)**. |

## 3. Audit of MONODROMY (`2001f12`, `620eacc`) & peer-check of Outside Review (`e48f904`)

- **MONODROMY Record 103 `M-9` (`620eacc`, scoped `G1` proof): VERIFIED — 0 breaks found.** Lemma A, Corollary (`|I| = 1`), and DAG induction on `P(f, y, j)` are row-preserving and settle all four specification gaps cleanly.
- **Minor remaining items in `main-theorem.md` (`620eacc`):** (i) `c₁W c` in `Pmargin` L82 is unbound in `theorem main`'s signature; (ii) L84 still prints `D.state c ∈ O(c.r)` instead of `≤ S c`; (iii) Record 101 Thm 3's run-record blindness tightening (`N-9`, `4ab193c`: `L = "caab" ↔ "cbaa"`, `D = {2}` → `3|D|` or raw-byte `D`) awaits folding into Record 101 / §6.
- **Outside Review peer-check:** Confirmed its `c₁` vs `D_𝓗` separation, uniform-family/no-advice rule, and Record 96 run-sampled vs row-sampled catch (`w ≥ 2`). Rebutted with exact data its `σ_hex`/SHA1 concern (`N-8`: 3,645 repeated 40-hex SHA1s yield only **3** width-1 rows = `0.00–0.01%` of `h₁`; novel hex = `61.3–68.2%` of `h₁`) and closed all three of its §5.1–5.3 Stream-N leads (`N-12`–`N-14`).

## 4. Cumulative summary & premises (`N-0` through `N-15`)

- **14/14** frameworks and supplied capabilities screened across Cycles 1–3 are **CLOSED** with explicit obstruction entries (witness, scope, escape condition, reach, strength): Ostrowski/BV (`N-3`), Rauzy fractals/Pisot (`N-4`), Veech groups/`SL(2, ℤ)` (`N-5`), Discrete/algebraic Morse/Perseus (`N-6`), Agol–Hass–Thurston (`N-7`), Positional copy schemes (`N-8`), Compressed suffix coordinates/r-index (`N-8`), Coarse-aggregate `Λ_agg` on Stream N (`N-9`), Krohn–Rhodes/`pos mod d` (`N-10`), Dimension groups `K₀`/`S_n` Fourier/1D OT (`N-10`), Learned PGM-index/Compressed sensing (`N-10`), Acyclic LZ77 macro schemes (`N-12`), Compressed `DSA`/Kempa Thm 5.1 (`N-13`), Durand ordered return-word hierarchies with exceptions (`N-14`).
- **Premises delivered (`N-11`, `c8a8e44`):** `P_N-1` (necessary fine-input E2 residue) and `P_N-2` (Stream-N partial-no over `Λ_closed` + `δ = 1/2` impossibility at `aR = 1`).
- **MONODROMY `NEEDS` discharged (on the 6 admitted blocks):** `NEEDS M-2` / amended `N-0` (`N-2`, `N-8`), `NEEDS M-5` (`N-2`, withdrawn by MONODROMY), `NEEDS M-4` (`N-9`, extended to Stream N), `NEEDS M-8` (`N-15`). *(Note: per-byte measurements on the remaining 226 census blocks require running `examples/census.rs` against `corpora/wikidata.bz2`, which is not in the repository.)*

## 5. Utility receipts (outside the theorem)

- **`#805` 232-block 1W i9 baseline (`N-2`):** `h₁` emit work = `19.09%` of total Open kernel time (`276.23 ms`).
- **Stream N ceilings on 6 admitted blocks (`N-2`, `N-8`, `N-12`, `N-14`):**
  - `σ_hex` Stream N (`17.4%` of `h₁`): `~3.3%` of 1W Open kernel time.
  - Repeated 40-hex SHA1 width-1 rows (`0.00–0.01%` of `h₁`): `~0.00%` of 1W kernel time.
  - Repeated non-GUID 8-grams (`10.7–15.2%` of `h₁`): `~2.1–2.9%` of 1W kernel time.
  - Repeated non-hole Durand templates (`5.4–9.0%` of `h₁`): `~1.0–1.7%` of 1W kernel time.
  - Open `D_L2` dependent-miss baseline (`N-15`): `14,630–95,695` L2 misses/block (`0.42–1.20 × h₁`), `0` L3 capacity misses at `W = 10`.

## 6. Why Neon stops here (final REVIEW)

All kitchen-sink directions assigned to Stream N, all three Outside Review §5.1–5.3 supplied capabilities, and all three MONODROMY measurement requests computable from repository data (`M-2`/`N-0`, `M-4`, `M-8`) are now complete and closed. Two structural facts establish that nothing further remains to try inside Stream N under the frozen target:
1. **No Stream-N-only construction can meet `δ = 1/2` on `𝓕`**, even with a zero-cost oracle (`novel_hex = 61.3–68.2%` of `h₁` on 6/6 admitted blocks; `Pglue` switches = `1.19–1.63 × h₁N`).
2. **Every known non-rigid Stream-N mechanism is closed** — by `Λ_agg` swap-invariance on Stream N (`ν′` median `6.9k–14.3k`, `|U| ≤ 1–354`, `N-9`), by `M-7` + `N-8` + `N-12` (repeats live in width `≥ 2` runs, not `h₁`), by `N-13` (`DSA` 87–94% distinct on `h₁`/`nonhex_w1`), and by `N-14` (`91–95%` of `h₁` are paid exceptions).

What remains open is solely: (a) a corpus-side run on `corpora/wikidata.bz2` to extend `N-0`/`M-4`/`M-8` from the 6 admitted blocks to all 232 census blocks; (b) MONODROMY's Stream R / uniform-family `𝓖(S, m, k)` lower-bound push into the fine-input global E2 residue (`P_N-1`, §6); and (c) an R3 format change if novel random identifiers (GUIDs) are to be removed from the BWT entirely.
