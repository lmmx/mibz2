# 2026-11-24 · MONODROMY 111 · Council answer: the problem, stated formally

Written independently; the other members' answers are unread. Labels: **[P]** proved, **[A]** argued, **[C]** conjectured, **[S]** taken from source. **Not read first-hand:** doomslide's essay and the Mario proof repository. For both I rely on the brief and on `2026-11-24-against-mathematics-source.md` as relayed to me; every use is marked [S, second-hand]. Nothing here proposes a construction.

## 1. Definitions

**Input.** A block is (L, o): L ∈ Σⁿ, Σ ⊆ {0,…,255}, 1 ≤ n ≤ 900,000, o < n [S: PROOF-PROGRAMME §1.2 via Neon N-1]. LF(x) = C[L[x]] + rank_{L[x]}(L, x). The block is **valid** if LF is one n-cycle [P that this is needed: Neon N-1 break 1, record 103].

**Output and correctness (T1).** T ∈ Σⁿ with T[n−1−t] = L[LF^t(o)] for 0 ≤ t < n. A decoder is correct iff it outputs exactly T on **every** block, valid or not, deterministically, with no error probability [S: main-theorem §1; A: record 105 §4 for exactness]. Cost statements are made on valid blocks only.

**Machine.** A word RAM with three memories: the input (read-only), the output (write-only, each cell written once), and working memory of S bytes. Every access is one of two kinds.
- **Streamed:** through a *cursor*, a register that only moves by ±1 between its own uses. At most s cursors are live.
- **Addressed:** at an address computed from data, other than by moving a cursor.

**Cost vector** of a run, all four always reported:
- Q: the number of addressed accesses;
- D: the longest chain of addressed accesses in which each address depends on the value the previous one returned;
- P: streamed bytes divided by n (passes);
- S: peak working bytes.

**Budget** 𝔟 = (K, s_max, S₀): P ≤ K, s ≤ s_max, S ≤ S₀(L).

**Instruction set.** A finite set 𝓘 of operation schemas, each declaring which memories it touches and by which kind of access, its semantics as a function on state, and hence its contribution to (Q, D, P, S). A *decoder over 𝓘* is one uniform program whose every executed step is an instance of a schema. A charge is meaningful only relative to a declared 𝓘 [P: record 108 §2]. The rigid calculus 𝓡 (record 103 §1) is one such set.

**How this resolves record 108's dichotomy** [A]. 108 showed that "is a bulk pass charged?" has no answer inside a single count. Here nothing is uncharged and nothing is charged by convention: a bulk pass costs P (and s), a walk costs Q and D. c₁ splits as c₁ = (composes done through cursors) + (addressed composes and links). "Strict" was charging the first term as if it were the second; "lenient" was dropping it. The pass budget K, which 108 found to be doing the work, becomes an explicit coordinate.

**Calibration premise (P-cal)** [open, a measurement]. Predicted time = a·D-chain steps + b·(Q − those) + c·streamed bytes, with (a, b, c) fitted on the reference machine and required to order the known witnesses correctly (walk, Open, lanes at 0.55–0.78× of emit, T² at E = 1.22). Without P-cal the vector is a diagnostic, not a currency.

## 2. "A new basis", formally

A **basis** is 𝔅 = (Φ, Obj, π, Acq, Run):
- Φ: a *description class*. An object is ω = (S, h, φ): a set S of rows, a height h, and φ ∈ Φ of O(1) words that determines LF^h on S and the bytes emitted along the way.
- Obj(L): the objects used on L. They must **tile**: every output position is produced by exactly one use of one object. Tiling plus soundness of each φ gives T1.
- π: Σⁿ → ℕ, a parameter of the **input**, defined without reference to any algorithm.
- Acq, Run: the program that builds Obj(L) and the one that emits T from it.

𝔅 is a **new basis relative to 𝔅₀ under budget 𝔟** if:
- **(N1) New object.** Φ ⊄ Φ₀: some object's φ is not a Φ₀-description.
- **(N2) New cost law.** Q(Acq) + Q(Run) ≤ C·π(L) for a constant C, and π separates: π(L) ≤ (1 − δ)·π₀(L) on the target family.
- **(N3) Acquisition without the old mechanism.** Acq stays within 𝔟 and its own Q and D are at most C·π(L). It may read every byte of L through cursors.

**Sanity check: walk → Rauzy–Veech** [A; the exact Cells cost law is S: records 92, 98, not rechecked]. Φ₀ = single rows, π₀ = n. Φ = translations of intervals, π = r, the number of runs. (N1) an interval translation is not a row. (N2) Q is governed by r, and r ≪ n on repetitive input. (N3) the run records come from one cursor pass over L with per-symbol counters: Q = 0. All three hold. Note that this acquisition **does** touch every row, through a cursor. Any reading of (N3) that forbids that also rejects the model step.

## 3. "Lowering the floor"

- **Which floor.** G1 [P, record 103]: over 𝓡 on valid blocks, c₁ ≥ h₁, row by row. In the vector model it reads: (cursor composes at width-1 rows) + Q ≥ h₁ for decoders over 𝓡.
- **The floor in question** is Floor_Q(𝒞, 𝔟, L) = the least Q over decoders in class 𝒞 within budget 𝔟. For 𝒞 = 𝓡 its value is **not known** once K ≥ 2: G1 bounds a sum, not Q [P]. Known upper bounds: n (walk), m (Open), m/2 (T²).
- **Lowering it** means exhibiting a decoder, in a declared 𝓘 ⊄ 𝓡, with Q ≤ (1 − δ)·h₁ **at the same 𝔟**.
- **Why that is not relocation.** Relocation is a fall in Q bought by a rise in P, s or S. With 𝔟 held fixed and all four coordinates reported it is visible and disallowed.
- **Relation to time.** Frozen clause 3 stays the only arbiter of utility. P-cal is the bridge. A fall in Q that P-cal prices as slower is not a yes.

## 4. The main theorem's shape

**Theorem (yes).** There is a declared 𝓘, a basis 𝔅 new relative to 𝓡's, and a decoder A over 𝓘 such that:
1. *(T1)* A is exact on every block;
2. *(budget)* A stays within 𝔟 on every valid block;
3. *(law)* Q_A(L) ≤ C·π(L);
4. *(separation)* π(L) ≤ (1 − δ)·h₁(L) with high probability on the uniform family 𝓖 (record 103 §4), and on each block of 𝓕 as validation;
5. *(utility)* under P-cal the predicted whole-decoder time beats master; then the native receipt.

**Premises, discharged one by one** [S, second-hand: the Mario form]: P-obj (objects sound and tiling ⇒ T1); P-acq (N3); P-law; P-param (π measured, then proved on 𝓖); P-place (how object uses reach output positions: this was "Pglue"); P-cal; P-util. "Forbid the obvious mechanism" is (N3). "Payoff first from a supplied state" is premises 3–5 with Obj(L) supplied, before P-acq.

**A "no"** must quantify over **all uniform programs on the machine of §1 within 𝔟**, and assert Floor_Q ≥ (1 − δ)·h₁ on 𝓖. That is a pass–memory–access trade-off lower bound for inverting a permutation presented as a BWT. None is known. With P unbounded, sorting gives Q = 0, so a no that does not fix 𝔟 is false. The only no on record is scoped: G1, class 𝓡, currency c₁.

## 5. Check against the sources

**Matches.** Yes/no minimisation with premises (Mario) [S, second-hand]. "Study the transformation between witnesses" (Hilbert's 24th): a basis is exactly what a witness transformation changes, and record 104 §2 is its instance. Count versus unit price (source record §4): Q versus (a, b, c). The Pythagorean caution: charging by expansion is excluded by defining cost on executions.

**Diverges or adds.** The vector and cursors are mine; no source has them. The frozen text has one count and an undefined "pass". (N1) as a description class replaces "not rows, not intervals", which is syntactic.

**Contradictions between records** (each stated, none resolved silently):
1. c₁(Open): record 98 says h₁ + pieces; record 102 §3 says ≥ m. 102 stands.
2. Record 108 fixes "a pass is charged per row". Rubric R3 demands a build in O(1) streaming passes and R4 demands a count below G1's floor "under record 108's convention". **Under that convention R3 ∧ R4 is unsatisfiable** [P]: any acquisition reads every width-1 row's record, since its byte is recorded nowhere else, so it is charged ≥ h₁ less what a histogram forces.
3. "The emit is work-bound" (record 95, channel #4) against toy-transfer's correction (channel #6). Withdrawn as a blanket claim.
4. "75–85% of h₁ is hex" (record 96) against "novel hex ≈ 65%" (Neon, per the brief). Different quantities; both stand.
5. Record 102 "copies cannot touch novel nibbles" against record 108 §1. 108 stands.
6. 𝓕 as a finite theorem domain (main-theorem §1) against clause 7's uniform family. Clause 7 stands; 𝓕 validates.
7. Main-theorem §3 still has the N/R two-stream shape; records 107–108 no longer use it. It is a proof device and should leave the statement.

## 6. Critique, with corrected statements

| item | defect | corrected |
|---|---|---|
| frozen c₁ ≤ (1 − δ)h₁ | defined only inside a declared 𝓘; decided by a convention (108) | Q ≤ (1 − δ)·h₁ within 𝔟, all four coordinates reported |
| K | "pass" undefined; cursor count absent | P and s_max as in §1 |
| S ≤ Open's scratch | a figure of one implementation | S₀(L) in bytes, stated as a function of (n, r, m) |
| δ against h₁ | h₁ is the calculus floor, not the baseline's cost (Open pays m) | keep δ on h₁ (frozen); also report Q/m |
| condition 1, R1 | "not rows, not intervals" can be met by renaming | (N1) |
| condition 2, R2 | "small exactly where runs fail" over-constrains, and six blocks are not the family | (N2) with π input-defined; measured on 𝓕, claimed on 𝓖 |
| condition 3, R3 | "no row-by-row LF walk" would reject Rauzy–Veech's own acquisition if "row-by-row" means touching every row | (N3): bounds on Acq's Q and D; cursor reads allowed |
| R4 | contradicts R3 (§5 item 2) | "Q below (1 − δ)h₁ at fixed 𝔟" |
| R6 | its "route class R1/R2/R3" reuses the rubric's own labels for different things | rename the route classes |
| R7 | no worker count | state at 1 worker and at the default 10; the supplied-codes diagnostic (0.838 and 0.988) shows they differ in kind |
| missing | placement of objects in the output; exactness; behaviour on invalid blocks; P-cal; a pinned baseline | §1, §4 |

## 7. Open choices

1. **Keep record 108's strict convention, or adopt §1's vector?** Strict keeps the frozen target and makes it unreachable by definition. The vector is a change to a clause I fixed today (not to §1a clauses 1–5), and it is laxer than strict on cursor passes. **Recommend the vector**: it leaves nothing uncharged, makes relocation visible, and is the only reading under which R3 and R4 can both hold. This is the Coordinator's or Louis's decision, not mine to take silently.
2. **s_max.** Options: |Σ| + O(1) (enough to distribute by symbol), or a figure derived from the cache. **Recommend |Σ| + O(1)**, fixed before any candidate is judged.
3. **K.** Frozen at 2. **Keep.** Report Floor_Q as a function of K as well, since the trade is the content.
4. **Theorem domain.** 𝓖 with 𝓕 validating, or 𝓕 alone. **Recommend 𝓖**; 𝓕 alone admits a table.
5. **Scalar or vector verdict.** A yes on Q alone, or on P-cal's priced scalar. **Recommend both, in that order**: Q is the structural claim, the scalar the gate to native work.
6. **Must objects avoid text positions?** No source decides. **Recommend no constraint**; P-place prices it.


## Corrections (after the council comparison)

Three errors confirmed by `council/COMPARISON.md`, and one figure, are corrected in record 112 §5: exactness on non-valid columns (the contract is the marked cycle repeated and cut at n); the constant must sit inside the margin; "R3 ∧ R4 unsatisfiable" is [A], not [P]; 0.55–0.78× is lanes with branchless appends. The text above is left as written.
