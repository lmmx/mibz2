# N-14: Supplied Capability 3 (Outside Review §5.3) — Durand Ordered Return-Word Hierarchies with Paid Exceptions, plus Audit of MONODROMY `2001f12`/`620eacc` and Outside Review

*Cycle 3 supplied-capability measurement answering Coordinator Note #3 / Outside Review §5.3 (`origin/claude/sharp-meitner-cx0v78` `e48f904`), plus verification of MONODROMY Record 102 §3 (`2001f12`) and Record 103 `M-9` (`620eacc`).*

**Script:** [`research/neon/scripts/screen_cycle3_supplied.py`](scripts/screen_cycle3_supplied.py) (exact Durand return-word decomposition on `","` for raw `T` and template-with-holes `T_skel` across all 6 admitted blocks of `origin/data/admitted-bwt`).

**Verdict:** **CLOSED at (c) Viability & (d) Payoff** — `tested failure within defined scope`.
- On raw Durand return words (split on `","`), **`91.7–96.1%` of all `h₁` rows** sit inside **unique** (single-occurrence) return words (`s_raw = 2,895–6,629`).
- Even with an **oracle template-with-holes factorisation** (replacing every GUID, SHA1, `[QPL][0-9]+` entity ID, and 4+ digit number with typed holes), the skeleton is small (`s_skel = 672–3,109`), **but the paid exception width-1 bytes `e_w1` (bytes inside holes + non-hole bytes in unique templates) still account for `91.0–94.6%` of all `h₁` rows (`31,181–91,786` bytes, across `18,055–20,486` hole slots)!**
- Thus `s_skel + e_w1 ≥ 0.93–0.97 × h₁`, leaving at most **`5.4–9.0%` of `h₁`** inside repeated non-hole templates — far below `δ = 50%` and worth `< 1.8%` of 1W Open kernel time!

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - Exact inverted texts `T` and `h₁` width-1 masks on all 6 admitted blocks (`origin/data/admitted-bwt`, `41c03a5b`).
2. **Real code path:**
   - Baseline `OpenCells` (`LIMIT = 16`, `h₁` emit = `19.09%` of 1W kernel time across 232 blocks, `n2-f-baseline-and-needs-m2.md`).
3. **Assumptions knowingly unrealistic:**
   - Both `H1` (raw Durand return words) and `H2` (oracle regex template-with-holes) are **supplied free** from the decoded text `T` (zero acquisition cost, zero exception-placement cost). Failing payoff even as a free oracle decisively closes §5.3 on `𝓕`.

## 1. Exact Durand return-word & template-with-holes exception measurements (`[measured]`)

| Block | `h₁` | Raw `N_occ` (`","`) | Raw `s_raw` | `h₁` in unique raw (`%h₁`) | Skel `s_skel` | Hole slots `e_slots` | `h₁` in holes (`%h₁`) | `h₁` uniq non-hole (`%h₁`) | **Total paid `e_w1` (`%h₁`)** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 35,116 | 31,118 | 2,895 | 33,302 (`94.8%`) | 672 | 19,601 | 30,609 (`87.2%`) | 2,191 (`6.2%`) | **32,800 (`93.4%`)** |
| `block-2.L` | 34,262 | 30,616 | 2,905 | 31,414 (`91.7%`) | 1,026 | 20,162 | 28,332 (`82.7%`) | 2,849 (`8.3%`) | **31,181 (`91.0%`)** |
| `block-85.L` | 55,504 | 30,375 | 4,424 | 51,504 (`92.8%`) | 1,626 | 19,500 | 42,228 (`76.1%`) | 9,554 (`17.2%`) | **51,782 (`93.3%`)** |
| `block-143.L` | 72,578 | 28,233 | 6,629 | 68,655 (`94.6%`) | 3,109 | 20,486 | 41,791 (`57.6%`) | 26,445 (`36.4%`) | **68,236 (`94.0%`)** |
| `block-210.L` | 97,054 | 30,487 | 6,536 | 93,221 (`96.1%`) | 2,168 | 18,055 | 76,616 (`78.9%`) | 15,170 (`15.6%`) | **91,786 (`94.6%`)** |
| `block-207.L` | 62,407 | 28,155 | 4,922 | 59,074 (`94.7%`) | 2,263 | 18,948 | 43,904 (`70.4%`) | 14,155 (`22.7%`) | **58,059 (`93.0%`)** |

**Why it fails `[measured]`:**
- Outside Review §5.3 conjectured that *"reused templates with literal holes might admit small finite hierarchies."*
- The table shows that while the **template skeleton** IS indeed tiny (`s_skel = 672–3,109`), the skeleton ALREADY sits in Open's thick strips (as `working-backwards.md` observed: JSON keys are 33–38% of bytes and 1–4% of thin rows).
- Meanwhile, **`91.0–94.6%` of all `h₁` width-1 rows** sit inside the **paid literal exceptions** (`57.6–87.2%` inside GUID/SHA1/entity/number holes + `6.2–36.4%` inside unique non-hole text templates such as entity labels and descriptions!). Only `5.4–9.0%` of `h₁` is covered by repeated non-hole templates (cost ceiling `< 1.8%` of 1W Open kernel time, before any acquisition cost!).

## 2. Obstruction table entry

| Field | Value |
|---|---|
| **Framework** | Ordered return-word hierarchies with paid exceptions (Durand / S-adic templates with holes) (`N-14`, Outside Review §5.3) |
| **Route class** | **R2** (positional via cumulative hierarchy lengths, if acquired cheaply) |
| **Gate failed** | **(c) Viability & (d) Payoff** (`91.0–94.6%` of `h₁` are paid exceptions `e_w1` even under an oracle template-with-holes factorisation; repeated non-hole templates cover only `5.4–9.0%` of `h₁`) |
| **Witness** | All 6 admitted blocks (`screen_cycle3_supplied.py`): raw `h₁` in unique return words `91.7–96.1%`; template+holes `e_w1 = 31,181–91,786` (`91.0–94.6%` of `h₁`), `e_slots = 18,055–20,486` |
| **Scope** | All ordered return-word / derived-sequence / template-with-holes hierarchies charging literal exceptions on `𝓕` |
| **Escape condition** | Compress the literal exceptions `e_w1` themselves (novel GUIDs, unique Q-IDs, unique descriptions), which have no repeated template structure |
| **Reach** | Global for template-with-holes / Durand return-word hierarchies on `𝓕` |
| **Strength** | Measured exact on 6 admitted blocks (free-oracle upper bound) |

## 3. Audit of MONODROMY `2001f12` / `620eacc` (Record 103 `M-9`) & Peer-Check of Outside Review (`e48f904`)

1. **MONODROMY Record 103 `M-9` (scoped Theorem G1 proof, `620eacc`) — VERIFIED, 0 BREAKS FOUND `[proved]`:**
   - Checked Lemma A (run invariant), Corollary (`|I| = 1`), and DAG-node induction on `P(f, y, j)` for `prim`, `restrict`, `join`, and `compose`. The `restrict` case holds cleanly because the Corollary applies to `f'` itself (`|I_{f'}| = 1` already). The proof is row-preserving (`c₁` restricted to any subset of width-1 rows is at least its size), closing the outside review's four G1 specification issues cleanly.
   - **Minor remaining polish in `main-theorem.md` (`620eacc`):**
     - Line 82 (`Pmargin`) references `c₁W c` which is not bound in `theorem main`'s signature (needs a `DW` / wide-row argument or `c₁W` defined from the combined trace).
     - Line 84 still reads `D.state c ∈ O(c.r)` instead of `D.state c ≤ S c` (matching `Pstate` L81).
     - Record 101 Theorem 3's run-record blindness break (`N-9`, `4ab193c`: `L = "caab" ↔ "cbaa"`, `D = {2}`) still needs its `3|D|` (or raw-byte `D`) repair in Record 101 / `main-theorem.md` §6.
   - **Adopted under stricter-wins:** §1a clauses **6b** (`D_𝓗(D, c) ≤ D_𝓗(Open, c)`) and **7** (no advice; uniform family `𝓖(S, m, k)` of Record 103 §4, validated on `𝓕`), and the stricter `c₁` convention (`2001f12`).
2. **Peer-check of Outside Review (`e48f904`):**
   - **Confirmed:** `c₁` vs `D_𝓗` separation (`LF²` precedent), uniform family / no-advice requirement, and Record 96's size-biased vs geometric sampling catch (`P(w) = w(1-p)²p^{w-1}` for `w ≥ 2`, while `w = 1` is `(15/16)²`).
   - **Rebutted with exact data (`N-8`, `N-12`, `N-13`, `N-14`):**
     - Its conjecture that `σ_hex`'s 75–85% overstates hard `h₁` due to repeated SHA1 hashes is refuted by `N-8` (`8756c55`): across 3,645 repeated 40-hex SHA1 occurrences (`145,800` B), only **3 bytes (`0.00–0.01%` of `h₁`)** are width-1, and **novel hex alone is `61.3–68.2%` of `h₁`** (`cw = 64.61%`).
     - All three of its proposed Stream-N supplied capabilities (§5.1 acyclic LZ77, §5.2 compressed `DSA`, §5.3 Durand return-word hierarchies with exceptions) fail decisively on `h₁` across all 6 admitted blocks even as **free oracles** (`N-12`: `z/h₁ = 0.54–0.64` at `min_len = 2` on 5/6 blocks, `1.27–1.51` at `min_len = 8`; `N-13`: `90.7–94.2%` distinct `DSA` on `h₁`, `86.9–93.4%` on `nonhex_w1`; `N-14`: `91.0–94.6%` of `h₁` are paid exceptions `e_w1`).

## 4. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact on 6 admitted blocks.
2. **Model assumptions:** Free-oracle supplied capability sets strict upper bounds on realisable decoders.
3. **Method:** Exact Durand return-word + typed-hole template factorisation (`screen_cycle3_supplied.py`), plus formal audit of `620eacc`.
4. **Outcome:** Closes the last of the outside review's three Stream-N leads (**14/14 frameworks & supplied capabilities closed across Cycles 1–3**). With `P_N-1` and `P_N-2` (`N-11`) plus `NEEDS M-4` (`N-9`) discharged and `M-9` (`620eacc`) verified, **no unscreened Stream-N lead remains** — preparing for our final Cycle 3 REVIEW unless a genuinely new fine-input E2 or R3 mechanism arises!
