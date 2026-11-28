# 2026-11-24 · MONODROMY 112 · The specification Π, and the bridge for 𝓡. The bridge holds with κ = 1 for the walk, Open and lanes, with κ = 1/2 for one doubling sweep, and **fails as a universal constant** for one family, in-place sweep composition, where the floor is ½·min(a⁺, a⁻), an input-defined count

Answers channel entry #18. Labels: [P] proved, [A] argued, [S] from source.

## 1. Π

**Machine.** Registers of 64 bits; register arithmetic is free. n ≤ 900,000.

**Memory, three types.**
- IN: the run records (start, byte) of L, r of them, and o. Read-only.
- OUT: n bytes, each written exactly once.
- WORK: S bytes, read-write.

**Cursor discipline.** A cursor is a register bound to one array and one direction. `NEXT` moves it one cell in that direction. `SEEK(a)` rebinds it to cell a and a direction. At most s_max = |Σ| + c₀ cursors are live, c₀ a declared constant.

**Instructions and their charge.**
| instruction | Q | note |
|---|---|---|
| `LOADC`/`STOREC` at a cursor, `NEXT` | 0 | streamed |
| `LOAD`/`STORE` at a computed address | 1 | addressed |
| `SEEK` | 1 | so cursor jumps are never free |

**D.** The longest chain of charged accesses in which each address is a function of the value the previous one loaded.

**Trace-to-pass rule.** A *sweep* is a maximal interval of the trace in which no cursor is re-`SEEK`ed into a cell it has already passed in that interval. P is the number of sweeps. Within a sweep each cursor therefore touches each cell at most once: this is the frozen "O(1) work a byte". **Budget: K = 2** (one build sweep, one emit sweep), s ≤ s_max, S ≤ S₀(L).

**A decoder under Π** is one uniform program over these instructions, exact on every column (§4 corrects what "exact" means off the valid set).

## 2. Extraction, and two lemmas [P]

A trace *extracts to 𝓡* if its stored objects are valid rigid facts, each new one is a `compose`, `join` or `restrict` of stored ones (record 103 §1), and every output byte is written while following a link. A compose is *streamed* if both operands were loaded at cursors, else *addressed*. A link's load of its fact is addressed (its address is the previous fact's target).

**Lemma B0.** Q ≥ (number of links) + (number of addressed composes).

**Lemma B1 (length).** If every one-row fact a link uses has height ≤ λ, then Q ≥ h₁/λ.
*Proof.* Every width-1 row is covered by a link, and the fact through it has a one-row domain (record 103, Lemma A). A link covers at most λ rows. ∎

## 3. The bridge, family by family

| family | streamed composes | bound | κ |
|---|---|---|---|
| LF walk, lanes | none | Q ≥ n ≥ h₁ [P, B0] | 1 |
| Open (thick strips contracted, retained rows walked) | none on one-row facts | Q ≥ m ≥ h₁ [P, B0] | 1 |
| one doubling sweep (T²): every compose reads only primitive facts | all | heights ≤ 2, so Q ≥ h₁/2 [P, B1] | 1/2 |
| **in-place sweep composition** (record 110) | all | Theorem B2 | **no constant** |

**Theorem B2** [P for the form stated]. Let a decoder extract to 𝓡 under Π, and let its build sweep process each row once, in an order ≺ monotone in the row index, by E[x] ← prim(x) ∘ E[LF(x)] with E[LF(x)] read as it then stands. Let a_≺ be the number of width-1 rows x whose predecessor FL(x) is processed before x. Then **Q ≥ a_≺ / 2**.

*Proof.* Take such an x. When FL(x) was processed, E[x] was still prim(x). So the fact made there covers FL(x) and x and stops at x. Hence either x is charged in its own right (a link starts there, or an addressed compose joins there), or the fact through x ends at x and a link starts at LF(x). Each link start is blamed by at most its own row and its one predecessor. ∎

**Corollary.** With a⁺ = #{x ∈ J₁ : FL(x) < x} and a⁻ = h₁ − a⁺, an ascending sweep has Q ≥ a⁺/2 and a descending one Q ≥ a⁻/2. A decoder free to choose has **Q ≥ ½·min(a⁺, a⁻)**.

- *It is not a constant times h₁.* For L = z·a^{n−1} the minimum is 0. So the bridge "Q ≥ κ·c₁" with a universal κ **fails on this family** [P].
- *On balanced input* (a⁺ ≈ a⁻) it gives Q ≥ h₁/4. Record 110 measured about n/2.7 links on synthetic text, consistent with it.
- *The mirror form* (extend the fact ending at x and store it at LF(x) through per-symbol write cursors) gives the same bound with LF and FL exchanged [A, by symmetry; not written out].

**Why K = 2 and s_max confine 𝓡 to these families** [A]. A streamed compose needs its right operand under a cursor. Targets LF(x) are increasing on each symbol class, so |Σ| cursors serve every primitive left operand. Targets LF^k(x), k ≥ 2, are increasing only on k-word classes, |Σ|^k of them, so with c₀ spare cursors only c₀ such classes can be served. A second compose of the same row in the same sweep needs a second set of |Σ| cursors. So within Π each row gets one streamed compose with a primitive left operand, up to the rows of c₀ k-word classes. This step is argued; it is the part of the bridge a formal proof must tighten.

## 4. What the bridge says about the target

- **The old methods pay the floor** [P]: walk, Open and lanes have Q ≥ h₁ under Π. This is the bridge Louis's decision asks for, for those three.
- **Two families inside 𝓡 go under it** [P]: doubling to h₁/2 exactly at the margin, and in-place sweep composition to between ½·min(a⁺, a⁻) and its achieved count. Neither is a counterexample to G1: G1 counts their streamed composes; Π does not charge them to Q.
- **So "new basis" cannot be defined as "Q below h₁"**: 𝓡 already does that with one sweep. In the agreed formulation the bridge constant must be the floor of 𝓡 under Π, which is **Floor_Q(𝓡, Π, L) ∈ [½·min(a⁺, a⁻), links of sweep composition]**, not κ·h₁. The margin δ should be taken against that.
- **In-place sweep composition is the named family** of entry #18 item 3: a candidate for escape by unit structure, built entirely from old objects.

## 5. Corrections to record 111 (the three the comparison confirmed, and one more)

1. **Exactness off the valid set.** The contract (PROOF-PROGRAMME §1.2) is: the marked cycle of length ℓ, repeated from the front and cut at n. My formula T[n−1−t] = L[LF^t(o)] equals it only when ℓ divides n. The contract stands; the formula is for valid columns.
2. **The constant.** "Q ≤ C·π and π ≤ (1 − δ)h₁" does not give Q ≤ (1 − δ)h₁ for C > 1. Corrected: Q ≤ θ(L) with θ input-defined and θ ≤ (1 − δ)·Floor, constant included.
3. **Label.** "R3 ∧ R4 unsatisfiable" was marked [P]. Record 108 §3 is [A]; so is this.
4. **Lanes figure.** 0.55–0.78× is lanes with branchless appends, not lanes alone.

## 6. NEEDS

- **NEEDS M-19** on the six admitted blocks, from the run records alone: a⁺ = #{x ∈ J₁ : FL(x) < x}, a⁻, and the same two counts with LF in place of FL; report min/h₁. With M-17 (links achieved) it brackets Floor_Q(𝓡, Π).
