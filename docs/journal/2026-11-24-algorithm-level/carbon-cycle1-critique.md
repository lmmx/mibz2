# Critique of the Carbon-role cycle 1 (C-1 to C-4) against rubric R1–R7

*Material: `research/carbon-new-basis` at aee62455. An independent reviewer applied the rubric fixed in journal §32. It recomputed every headline from `results/*.json` and read all four C sources. It rebuilt C-1, C-2 and C-4 and ran them on block-0; all three were exact and matched their json rows. The Coordinator spot-checked two findings: c1.c:307, and C-1's `q_tot_over_h1`, which is 3.51–79.2 over 66 rows.*


Paths are relative to `research/carbon/`. "json" means `results/cN_results.json`. All json figures were recomputed. I compiled three binaries with `gcc -O2` and ran them on block-0: C-1 `sweep_k1_L2`, C-2 `RF3_hex16_2sw` and C-4 `u8_L1_exact`. All three reproduced their json rows exactly and printed `exact:true`.

## Table

| | R1 object | R2 parameter | R3 acquisition | R4 floor | R5 proof form | R6 not incremental | R7 utility |
|---|---|---|---|---|---|---|---|
| **C-1 chords** | **partial**: ω_b has S={b}, a single row carrying a stored word and a next pointer. It is a contracted list node (record 109's list-ranking contraction). | **partial**: χ₂/h₁ = 0.229–0.320 is measured. The "+1 per novel token" is labelled [proved] but is only argued (c1 md:16). The cost law fails, because Q is not bounded by C·χ (see R4). | **not**: P = 9.7–37.7 (json `P_passes`), not K=2. Thick pull is a worklist interval-exchange induction with qsort (c1.c:229, :343) and binary search, costing 8.8–77 h₁ of addressed probes. Arena reads at `woff[y]` (c1.c:572) are data-addressed and uncharged. | **not**: total Q/h₁ = 9.4–79.2 at L≤4 (json `q_tot_over_h1`). The best is `sweep_k1_L16` at 3.5–10.8. | **partial**: no theorem. The near miss (thick fragmentation) is named, but the headline omits its cost. | **not**: B_L is the thick→thin frontier. The chords are Open's first-return map induced again onto that subset. Composition is a row-order Gauss–Seidel sweep, which is M-17 (record 110). | **not**: nothing in whole-decoder terms. |
| **C-2 cones** | **not**: the cone D_{u^R}→I_u is a k-context class mapped onto an interval of rows. The stored object is still a per-row 64 B slot (c2.c:524ff). | **not**: π_RF contains an undefined c_arch (c2 md:22) and is never measured in json. | **partial**: no LF-order walk, and the thin side really has Q=0 through cursors. But P = 64–137, s = 513–4609 (record 111 recommends \|Σ\|+O(1)) and S = 61 MB, with thick Q 0.68–3.33 h₁. | **not**: Q_run+Q_thick ≥ 1.50 h₁ on every block (e.g. block-85: 0.829+0.675). | **partial**: RF-1(1) has a correct proof (md:14) but is the standard k-step LF order lemma. Relocation into (P, s, S) is stated honestly (md:61). | **not**: this is the LF^k table on a subalphabet. Record 108 §2 names it ("E1 class fact … at k=2 the T² kernel"). The cone-free `k1_base_2sw` already reaches 0.797–0.898 T² on 6/6, so the cones add about 20%. | **partial**: an honest relocation ledger, but no time figure. |
| **C-3 slabs** | **not**: maximal intervals of rows with a rank directory, and h=1. | **partial**: r_π is measured, but Q does not scale with it: Q_run_thin = m₁₆ and q_run_dir = 1.32–2.03 M. | **met** (trivially): one stream over the run records with s=2, building an index and composing nothing (json q_acq=0, P 1.06–2.75). | **not**: the consumer *is* the row-by-row LF walk (c3.c:258–305, d_run = n). | **partial**: QS-1 is a tautology. h=1 means one byte per use by definition, and "2.000·T², exact to the integer" holds because T² := m₁₆/2. | **not**: this is an FM-index-style rank walk over a quotiented L. | **not** |
| **C-4 unitigs** | **partial**: the unitig is a dBG object, defined exactly. The decoder never uses the name, though: V₁ only *cuts* the same J/word chains that C-1 builds (c4.c:432–470). | **partial**: π_dBG/h₁ = 0.361–0.403 is measured, and Q_run = u₈⁽¹⁾ + h₁,br holds exactly (13,604 = 5,000 + 8,604 on block-0). The "+1 per token" claim is argued, not proved. | **not**: 7 scatter passes plus sweeps give P = 110–145. At L=1 the thick pull is 37.6–92.5 h₁ addressed and S reaches 483 MB. | **not**: total Q ≥ 38 h₁ at L=1. At L=16 Q_run alone is 1.10–2.32 h₁. | **partial**: U-1 has no written proof (md:28–31), only per-row asserts. It is true and immediate, since rows are sorted by context. The near miss (the name→address wall) is real and well named. | **partial**: it is not copy plans. It contracts only the *literal* islands, and leaves the 0.24 h₁ of h₁,br in repeated 8-mers one step each, which is where copies would pay. It is a restriction of C-1's contraction. | **not** |

**Verdicts.** No candidate "starts to overthrow induction". R1–R3 is never met together, and R4 is "not" for all four. C-1, C-2 and C-4 are incremental: chain contraction by Gauss–Seidel sweeps over the T²/Open machinery. C-3 is the walk with a compressed rank index. C-4 holds the only new idea worth keeping: the per-row 8-mer name plus the uniqueness certificate, and the wall it hits.

## Discrepancies (claim vs md vs json)

1. **Headline omits thick acquisition.** The README's "Best Q_acq,thin=0 run" column and CLAIMS report Q_run only.
   - C-1 `sweep_k1_L2` "0.793–0.911 h₁": json `q_tot_over_h1` = 20.4–77.9.
   - C-4 `u8_L1_exact` "Q_acq,u + Q_run = 0.958–1.000 h₁" (c4 md:50): adding q_acq_thick gives 38.5–93.5 h₁.
2. **C-4 thick probes.** The md says "2.9M..5.2M" (c4 md:51). The json shows 1.83M (block-0) to 6.71M (block-143).
3. **"1 pass" and "2 sweeps" against measured P.** json `P_passes` is 9.7–37.7 for C-1, 64–137 for C-2 and 110–145 for C-4. The frozen budget is K=2.
4. **C-3 table mixes configs.** r_π comes from `hex23_pure`; the Acq column comes from `hex23_thin16`, whose r_π/h₁ is 0.326–0.516, not 0.307–0.443. The table also omits q_run_dir (1.3–2.0 M addressed) and d_run = n.
5. **C-2 "beats T² in ONE pass" (0.851–0.990).** This is true, but the cone-free two-sweep baseline already gets 0.797–0.898. The headline credits the cones for what the sweep does.
6. **Accounting.**
   - c1.c:307 has `if (local_d > *d) *d += local_d;`, which is neither a max nor a sum, so D_acq_thick is wrong.
   - C-2 counts D_thick as the number of bands, a different convention.
   - Sorts (qsort, c1.c:229, :343; c2.c:266; c4.c:244) are uncharged.
   - Word materialisation (memcpy of whole chord words per compose) is uncharged.
7. **"[proved]" overuse.** C-1's χ insertion bound and C-4's "+1 per token" are arguments only, and BWT edits are non-local. QS-1 is definitional.
8. **De-confliction claim.** reading-log.md says Carbon leaves M-17 untouched. Yet C-1, C-2 and C-4 all compose with the row-order Gauss–Seidel sweep (`y < x already visited`, c1.c:567, c2.c:572, c4.c:454), which is M-17.
9. **What "byte-exact" checks.** It compares against an in-file textbook LF walk (`invert_reference`), not against an independent decoded artefact. The manifest has no output hash. This is adequate for internal consistency, and the 174 count adds up (66+48+24+36).

## Hidden LF walk?

- **No build contains an LF-order walk.** Every sweep is in row order, x = 0..n−1, with 256 per-symbol cursors (`y = cur[c]++`), which is legitimately Q=0 under record 111.
- **But the old mechanisms are inside:**
  - (i) C-3's consumer is the row walk itself.
  - (ii) The thick side in C-1, C-2 and C-4 is interval-exchange induction (Open/Cells), with sorting and binary search, and it dominates cost at small L.
  - (iii) The exact modes use addressed pointer jumping, J[x]←J[J[x]], which is counted.
  - (iv) The build materialises every chord's literal word (about n bytes in total). The build has therefore decoded the text in fragments, and "Run" only orders them. That is list ranking.
- **(a) on "Q_acq = 0".** It is uncharged by convention, not free. Under record 108 the code's own `strict_c1_acq` is ≥ m_L > h₁, which the authors concede (c1 md:51, c2 md:61). Under record 111 it shows up as P ≫ K and s ≫ |Σ|+O(1): it was relocated, not removed.

## Closing the gaps

- **C-1.** Replace the thick pull with Open's O(bands) endpoint induction at L=16. Report all of Q at P≤2. Then show the chord contraction is not record 109's list ranking, which seems unlikely, since the object is a single row.
- **C-2.** No path as a basis. It is LF^k, so a stride that grows on novel content needs more passes or cursors, which is exactly the K/s trade of record 108. At most, keep it as a T² variant and compare it against `k1_base_2sw`, not T².
- **C-3.** Abandon as a basis, as QS-1 itself concedes. It may be worth keeping as a compressed rank engineering note.
- **C-4.** The live one. Resolve names to addresses only at unitig endpoints (u₈ ≈ 0.14–0.16 h₁ at L=1), for example by a sorted merge, since u_k is globally sorted. Hand h₁,br (≈0.24 h₁) to copy plans. Keep L=16 so the thick side stays cheap. Report the full (Q, D, P, S) at K=2, and prove the per-token bound on 𝓖. Only then is R4 testable.
