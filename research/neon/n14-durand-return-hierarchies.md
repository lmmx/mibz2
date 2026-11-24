# N-14: Supplied Capability 3 (Outside Review §5.3) — Durand Ordered Return-Word Hierarchies with Paid Exceptions, plus Audit of MONODROMY `2001f12`/`620eacc` and Outside Review

*Cycle 3 supplied-capability measurement answering Coordinator Note #3 / Outside Review §5.3 (`origin/claude/sharp-meitner-cx0v78` `e48f904`), plus verification of MONODROMY Record 102 §3 (`2001f12`) and Record 103 `M-9` (`620eacc`).*

**Script:** [`research/neon/scripts/screen_cycle3_supplied.py`](scripts/screen_cycle3_supplied.py) (exact return-word decomposition on the single delimiter `","` for raw `T` and for one 4-regex typed-hole skeleton `T_skel` across all 6 admitted blocks of `origin/data/admitted-bwt`).

**Verdict (narrowed in N-17):** **CLOSED for the tested single-delimiter (`","`) return-word split and its 4-regex typed-hole skeleton at (c) Viability & (d) Payoff** — `tested failure within defined scope`. **Does NOT close every possible return-word delimiter, multi-level Durand/S-adic derived hierarchy, or non-delimiter hierarchical parse.**
- On raw return words split on `","`, **`91.7–96.1%` of all `h₁` rows** sit inside **unique** (single-occurrence) segments (`s_raw = 2,895–6,629` `[measured]`).
- Under the tested **oracle 4-regex template-with-holes factorisation** (replacing GUID, SHA1, `[QPL][0-9]+`, and `[0-9]{4,}` with typed holes), the comma-split skeleton is small (`s_skel = 672–3,109` `[measured]`), **but the paid exception width-1 bytes `e_w1` (bytes inside those four hole classes + non-hole bytes in unique comma-split templates) still account for `91.0–94.6%` of all `h₁` rows (`31,181–91,786` bytes across `18,055–20,486` hole slots `[measured]`)**.
- Thus for this specific split, `s_skel + e_w1 ≥ 0.93–0.97 × h₁`, leaving **`5.4–9.0%` of `h₁`** inside repeated non-hole comma-split templates (`[measured]`, `< 1.8%` of 1W Open kernel time under N-2's attribution model `[argued]`).

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - Exact inverted texts `T` and `h₁` width-1 masks on all 6 admitted blocks (`origin/data/admitted-bwt`, `41c03a5b`).
2. **Real code path:**
   - Baseline `OpenCells` (`LIMIT = 16`, `h₁` emit = `19.09%` of 1W kernel time across 232 blocks under N-2's attribution model, `n2-f-baseline-and-needs-m2.md`).
3. **Assumptions knowingly unrealistic:**
   - Both `H1` (raw `","`-split return words) and `H2` (4-regex typed-hole `","`-split templates) are **supplied free** from `T` (zero acquisition cost, zero exception-placement cost).
   - **Scope limit (N-17):** Only the single-byte delimiter `","` and the four hand-picked regex hole classes (`GUID_RE`, `SHA1_RE`, `ENTITY_RE`, `NUM4_RE`) were tested. Multi-level derived sequences, alternative delimiters, or grammar/S-adic parses that factor inside `","`-segments were **not** tested here.

## 1. Exact comma-split return-word & 4-regex template-with-holes measurements (`[measured]`)

| Block | `h₁` | Raw `N_occ` (`","`) | Raw `s_raw` | `h₁` in unique raw (`%h₁`) | Skel `s_skel` | Hole slots `e_slots` | `h₁` in holes (`%h₁`) | `h₁` uniq non-hole (`%h₁`) | **Total paid `e_w1` (`%h₁`)** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 35,116 | 31,118 | 2,895 | 33,302 (`94.8%`) | 672 | 19,601 | 30,609 (`87.2%`) | 2,191 (`6.2%`) | **32,800 (`93.4%`)** |
| `block-2.L` | 34,262 | 30,616 | 2,905 | 31,414 (`91.7%`) | 1,026 | 20,162 | 28,332 (`82.7%`) | 2,849 (`8.3%`) | **31,181 (`91.0%`)** |
| `block-85.L` | 55,504 | 30,375 | 4,424 | 51,504 (`92.8%`) | 1,626 | 19,500 | 42,228 (`76.1%`) | 9,554 (`17.2%`) | **51,782 (`93.3%`)** |
| `block-143.L` | 72,578 | 28,233 | 6,629 | 68,655 (`94.6%`) | 3,109 | 20,486 | 41,791 (`57.6%`) | 26,445 (`36.4%`) | **68,236 (`94.0%`)** |
| `block-210.L` | 97,054 | 30,487 | 6,536 | 93,221 (`96.1%`) | 2,168 | 18,055 | 76,616 (`78.9%`) | 15,170 (`15.6%`) | **91,786 (`94.6%`)** |
| `block-207.L` | 62,407 | 28,155 | 4,922 | 59,074 (`94.7%`) | 2,263 | 18,948 | 43,904 (`70.4%`) | 14,155 (`22.7%`) | **58,059 (`93.0%`)** |

**What this shows (and does not show) (`[measured]` + `[argued]`):**
- For the `","` split, the **template skeleton** is compact (`s_skel = 672–3,109` `[measured]`), because repeated JSON scaffolding already sits mostly in Open's thick strips (as `working-backwards.md` observed: JSON keys are `33–38%` of bytes and `1–4%` of thin rows).
- Meanwhile, **`57.6–87.2%` of `h₁`** sits inside the four literal-payload hole classes (`GUID`, `SHA1`, `[QPL][0-9]+`, `[0-9]{4,}`), and another **`6.2–36.4%` of `h₁`** sits in unique non-hole `","`-segments (`[measured]`).
- **What stays open outside this test (`[argued]`):** Choosing a finer delimiter (e.g. `":"`, `"\""`, or `"-"`), applying a multi-level derived-word hierarchy, or factoring repeated sub-phrases inside unique `","`-segments could shrink the `6.2–36.4%` unique non-hole share. However, even if a finer hierarchy compressed **100%** of non-hole `h₁` bytes, the `57.6–87.2%` of `h₁` inside novel literal holes (GUID/SHA1/entity/number payloads) remains unpaid unless the holes themselves are factored or replaced — which matches `N-8`'s independent `61.3–68.2%` novel-hex floor (`[argued]`).

## 2. Obstruction table entry (narrowed in N-17, all six fields)

| Field | Value |
|---|---|
| **Framework** | Single-delimiter (`","`) return-word split & 4-regex template-with-holes factorisation (`N-14`, narrowed in `N-17`) |
| **Route class** | **R2** (positional via cumulative hierarchy lengths, if acquired cheaply) |
| **Gate failed** | **(c) Viability & (d) Payoff** (`91.0–94.6%` of `h₁` are paid exceptions `e_w1` under the tested `","` + 4-regex split; repeated non-hole `","`-templates cover only `5.4–9.0%` of `h₁`) |
| **Witness** | All 6 admitted blocks (`screen_cycle3_supplied.py`): raw `","`-split `h₁` in unique segments `91.7–96.1%`; 4-regex template+holes `e_w1 = 31,181–91,786` (`91.0–94.6%` of `h₁`), `e_slots = 18,055–20,486` (`57.6–87.2%` of `h₁` in holes alone) |
| **Scope** | **The tested one-level `","` delimiter split** (raw and with the 4-regex hole mask `GUID_RE`, `SHA1_RE`, `ENTITY_RE`, `NUM4_RE`) on the 6 admitted blocks. **Explicitly excluded:** multi-level Durand/S-adic hierarchies, alternative delimiters, or sub-segment grammars that factor inside unique `","`-segments |
| **Escape condition** | Use a finer delimiter or multi-level S-adic hierarchy to factor the `6.2–36.4%` unique non-hole `h₁` share, **and** pair it with a non-template mechanism for the `57.6–87.2%` literal-hole `h₁` bytes (GUIDs/IDs) |
| **Reach** | **Local to the tested `","`-split + 4-regex hole factorisation on the 6 admitted blocks** (not global over all return-word hierarchies) |
| **Strength** | **Measured exact** on 6 admitted blocks (free-oracle upper bound for this split) + **argued** (hole-payload residual vs `δ = 1/2`) |
| **Transferable insight** | Template-with-holes factorisation cleanly separates structural scaffolding (`s_skel = 672–3,109`, mostly already wide in Open) from literal payload slots (`18k–20.5k` holes carrying `57.6–87.2%` of `h₁`); any hierarchy that leaves literal holes as paid exceptions is capped by the non-hole `h₁` fraction (`12.8–42.4%`, below `δ = 50%` on 6/6 blocks) regardless of delimiter choice |

## 3. Audit of MONODROMY `2001f12` / `620eacc` (Record 103 `M-9`) & Peer-Check of Outside Review (`e48f904`)

1. **MONODROMY Record 103 `M-9` (scoped Theorem G1 proof, `620eacc`) — VERIFIED, 0 BREAKS FOUND `[proved]`:**
   - Checked Lemma A (run invariant), Corollary (`|I| = 1`), and DAG-node induction on `P(f, y, j)` for `prim`, `restrict`, `join`, and `compose`. The `restrict` case holds cleanly because the Corollary applies to `f'` itself (`|I_{f'}| = 1` already). The proof is row-preserving (`c₁` restricted to any subset of width-1 rows is at least its size), closing the outside review's four G1 specification issues cleanly.
   - **Minor remaining polish in `main-theorem.md` (`620eacc`):**
     - Line 82 (`Pmargin`) references `c₁W c` which is not bound in `theorem main`'s signature (needs a `DW` / wide-row argument or `c₁W` defined from the combined trace).
     - Line 84 still reads `D.state c ∈ O(c.r)` instead of `D.state c ≤ S c` (matching `Pstate` L81).
     - Record 101 Theorem 3's run-record blindness break (`N-9`, `4ab193c`: `L = "caab" ↔ "cbaa"`, `D = {2}`) — folded into Record 107 as `|𝒫| + 3|D| > ν′` (see also `N-18` for the sharp `2|D|` coefficient under pairwise-disjoint spans).
   - **Adopted under stricter-wins:** §1a clauses **6b** (`D_𝓗(D, c) ≤ D_𝓗(Open, c)`) and **7** (no advice; uniform family `𝓖(S, m, k)` of Record 103 §4, validated on `𝓕`), and the stricter `c₁` convention (`2001f12`).
2. **Peer-check of Outside Review (`e48f904`, updated after round 2 `78abe2c`):**
   - **Confirmed:** `c₁` vs `D_𝓗` separation (`LF²` precedent), uniform-family/no-advice requirement, and Record 96's size-biased vs geometric sampling catch (`P(w) = w(1-p)²p^{w-1}` for `w ≥ 2`, while `w = 1` is `(15/16)²`).
   - **Conceded in round 2 (`N-16`, `N-17`):** `N-12` had a cyclic-vs-suffix SA bug (repaired in `N-16`), `N-13`'s `DSA` distinctness/`H₀` bound applies to literal-terminal representations only, and `N-14` tested one `","`-split + 4-regex mask only.
   - **Rebuttal that stood in round 2 (`N-8`):** Outside Review round 2 §I(f) conceded the repeated-SHA1 rebuttal (`3 / 145,800` B = `0.00–0.01%` of `h₁`; novel hex = `61.3–68.2%` of `h₁`).

## 4. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact on 6 admitted blocks for the `","` delimiter and 4-regex hole mask.
2. **Model assumptions:** Free-oracle supplied capability sets upper bounds only for the tested split.
3. **Method:** Exact `","`-split return-word + typed-hole template factorisation (`screen_cycle3_supplied.py`), plus formal audit of `620eacc`.
4. **Outcome (N-17):** Scope cleanly narrowed to the single tested `","`-split + 4-regex hole mask (all six obstruction fields supplied); no new framework screen opened.

