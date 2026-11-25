*Adopted 24 November 2026. Louis chose "real machine cost" as the meaning of "lower the floor": fewer unpredictable memory reads and scans than one per lone row, on a declared machine model, backed by a bridge proof that the old methods really pay that floor. The old bookkeeping target is kept as version 1. An independent reviewer compared the three answers in [INDEX.md](INDEX.md) and checked disputed facts against the pinned sources. The Coordinator spot-checked one: the supplied-codes result is journal §30, not §29.*

# Council comparison: MONODROMY (111), Neon, ChatGPT Pro

Sources: brief (docs/journal-algorithm-level), MONODROMY `303b48c8`, Neon `4e56a735`, ChatGPT (docs/journal-algorithm-level). Spot-checked against: MONODROMY records and `main-theorem.md` at `f86a4d3b`, Neon `research/neon` at `cc3b76f3`, journal at `2a99cd51`, `PROOF-PROGRAMME.md` and `FRONTIER.md` at `c282b422`, `channel/coordinator-to-neon.md` at `f86a4d3b`, and the public `tra38/sm64-wmotr-abc-proof` README (HEAD, read-only clone, since deleted). **V** = I checked it against the source. **NV** = I did not.

Sizes: MONODROMY's answer is short (about 1.9k words, and it did not read doomslide or Mario first-hand). Neon's and ChatGPT's are each about 8k words.

---

## 1. Definitions and cost model

**AGREE.**
- **Input.** A marked column (n, L, p), with 1 ≤ n ≤ 900,000, p < n and Σ ⊆ {0..255}. LF(i) = C(L[i]) + rank(L[i], i).
- **Correctness.** Exact, deterministic, every byte, on all columns. A CRC is not correctness (M105 §4). Cost is stated on a promised sub-domain.
- **A charge exists only relative to a declared, finite instruction set** with semantics and tariff (M108 §2). ChatGPT has the cleanest definition (1.5): typed bounded operands, exact transitions, explicit memory accesses, a fixed tariff, and lowering rules, so that "naming a high-level op does not make it one instruction".
- **Record 108.** All three keep its dichotomy, and none keeps scalar c₁ under the strict convention as the *sole* currency. All three move to a multi-coordinate cost vector over executions, with passes and state charged explicitly.

**DIVERGE.**

| | MONODROMY | Neon | ChatGPT |
|---|---|---|---|
| Domain | "valid" means single n-cycle; T1 on every block | SingleCycle ⊊ ValidBWT ⊊ Column | D_sc ⊆ D_ker, with D_bz separate |
| Input | L ∈ Σⁿ (bytes) | I₀ = Runs(L), r records (PROOF-PROG O1) | "permitted encoding of L" |
| Currency | (Q, D, P, S): addressed accesses, dependent-chain depth, streamed bytes/n, peak bytes; budget (K, s_max, S₀) | W_core = W_seq + W_dep (non-I/O instructions), plus Q_𝓗, D_𝓗, S, K_sweep; reading I₀ once and writing Out once separated as C_io | full ledger (W, Q_r, Q_w, span, d, D_𝓗, P, S_peak), plus a *fixed* compared component C*, plus P₁₀₈ kept as a separate named ledger |
| 108's fate | replaced; strict "charged as if addressed" was the error | replaced; c₁ demoted to a diagnostic layer | **both kept**, as separately named targets |
| Memory | S₀(L) as a function of (n, r, m) | S_max(c), with write-once Out | pointwise S_peak ≤ S_Open (frozen) |

- **Domain: Neon and ChatGPT are correct.**
  - PROOF-PROG §1.1–1.2 (V) decodes periodic and multi-cycle columns by repeating the marked cycle, cut at n. So single-cycle is a promise, not validity.
  - ChatGPT's witness: a constant text gives LF = id. Neon's characterisation: single cycle ⇔ BWT of a primitive word, which is the Mantaci–Restivo–Sciortino result.
  - MONODROMY's T1 has an error (see "Errors").
- **Input: Neon is better grounded.** The kernel's interface is run records (PROOF-PROG O1, V). This matters for the Rauzy–Veech check (§2).
- **Currency: the substantive split.**
  - *MONODROMY's Q* charges only data-addressed accesses. Cursor sweeps are priced by P, which is bounded by K.
  - *Neon's W_core* charges *every* non-I/O instruction. Under it, any decoder that does O(1) arithmetic per run record pays ≥ r ≥ h₁. So W_core ≤ (1 − δ)h₁ forces multi-record-per-word (packed) processing of the build too. Neon accepts this (§2 B2; open choice 4). But it makes the target "packed computation", not "new basis". And the C_io carve-out is undefined at exactly the point that matters: which instructions that place bytes in Out count as "I/O"?
  - *ChatGPT* names this trap explicitly. In a byte-traffic currency the output alone costs n ≥ h₁. Subtracting a baseline B(c) needs a real decomposition W = B + C*, because W ≥ B and W ≥ κh₁ do not add.
  - **Strongest: ChatGPT's structure (fixed C* inside Π, chosen before candidates), instantiated with MONODROMY's Q/D/P/S split**, which avoids the W_core degeneracy.

**UNIQUE.**
- **MONODROMY:**
  - the cursor/addressed distinction and the dependent-chain depth D;
  - the calibration premise P-cal (a·D + b·Q + c·bytes, fitted to order the known witnesses). It is right, and needed: no one else gives a bridge from counts to time;
  - a gap: the model doesn't say whether *choosing* among s cursors by data value (counting-sort distribution) is streamed. With s_max = |Σ| + O(1) it must be, and that should be explicit.
- **Neon:** write-once Out, with read-back and overwrite charged (right; it closes the copy-from-output loophole, REV1 §1).
- **ChatGPT:**
  - P₁₀₈'s phrase "determine something about that row" has no operational semantics, so P₁₀₈ is an intention, not a function (right; M108 §4 states it in prose only, V);
  - "pure fold to O(1) words" needs a fixed word count and width, or one "word" encodes the column;
  - the edge cases L = bbaaa (single cycle, h₁ = 0; V: LF = [3,4,0,1,2]) and n = 1 (h₁ = 1, so ⌊h₁/2⌋ = 0) need a stated performance domain;
  - randomised decoders need a separate seed quantifier.

## 2. "New basis" and the Rauzy–Veech check

**AGREE.**
- A basis is an existential package with these parts:
  - a representation (MONODROMY: description class Φ, objects ω = (S, h, φ); Neon: 𝒪, Valid, Build, Advance, Emit, θ; ChatGPT: 𝒵, Rep, Build, Use, θ, f);
  - a soundness obligation from Rep to decode;
  - an input-defined parameter (not the algorithm's own running time);
  - a cost law that *includes acquisition*.
- **New object** must be semantic, not syntactic: channel #14's "not rows, not intervals" can be met by renaming.
- **Acquisition** must be charged in the same ledger. A supplied state is a conditional result (Archimedes' Method, Mario's supplied fixture).
- **Walk → Rauzy–Veech passes all three conditions** when the prior basis is the walk (π₀ = n, π = r).

**DIVERGE.**
- **Condition 1 (new object).**
  - *MONODROMY (N1):* Φ ⊄ Φ₀, a description-class non-inclusion. Weakest: a new Φ can still be cost-equivalent.
  - *Neon (B1):* no cost-preserving lowering into the prior calculus, plus an Advance closed on the carrier.
  - *ChatGPT (2.2):* proves that a strict cost separation *implies* no cost-preserving reduction (if every old implementation costs ≥ F and the candidate costs < F, no reduction exists).
  - **ChatGPT's observation makes Neon's B1 largely redundant given B2 plus a Bridge**, and is the stronger formulation. Keep Neon's anti-renaming test as a cheap screen.
- **Condition 3 (acquisition).**
  - *MONODROMY:* (N3) allows cursor reads of every byte. Any reading of "no row-by-row" that forbids them also rejects Rauzy–Veech.
  - *Neon:* Rauzy–Veech builds in one O(r) pass over *run records*, so no row is touched.
  - **Both are right on their own input model.** On the actual kernel interface (run records), Neon's statement holds and MONODROMY's "does touch every row" is an artefact of taking L as bytes. MONODROMY's general point stands: (N3) must bound the old mechanism's *cost*, not forbid touching rows.
- **Cost law strength.** Neon calls Rauzy–Veech's B2 "proved" with O(r) to O(r log r) work plus O(n) spelling. ChatGPT warns the sources don't justify an unconditional "Rauzy–Veech is O(r) total". Output is n bytes, and the induction-description size must stay in the bound. **ChatGPT is more careful.** FRONTIER gives Cells at 1.013–1.019r induction steps (V), but that is measured, not a theorem.

**UNIQUE.**
- **ChatGPT, "crucial relativity"** (right, and important):
  - Rauzy–Veech satisfies the *historical* test against the walk while *failing* the current one: it lies inside 𝓡, so G1 applies to it.
  - A definition that requires the model step to beat h₁ tests the wrong floor.
  - MONODROMY and Neon run the check only against π₀ = n.
- **Neon, sharpness check:** Cells → Open fails B1 (same calculus), matching METHOD's Foundational/Architectural split. Right, and a useful negative control.
- **MONODROMY:** objects must *tile* the output (each position produced exactly once). Right. It is the precondition for P-place, the generalised Pglue, which the others leave implicit.

## 3. "Lowering the floor"

**AGREE.**
- The floor is G1: c₁ ≥ h₁ over 𝓡 on single-cycle columns, row-preserving (M103, V).
- It cannot be beaten inside 𝓡 in c₁. "Lowering" means a decoder outside the class, compared in an execution currency.
- Relocation is excluded by charging Build in the same ledger at a fixed budget.
- Whole-decoder time (clause 3) is a separate empirical conjunct.
- The supplied-codes figures, all three cite correctly (V, J §30): 0.838 at 1 worker, 0.988 [0.957, 1.020] at 10.

**DIVERGE.**
- **Which operational floor exists.** Neon's §3.1 table is the clearest statement of what is known:
  - the rigid floor (G1);
  - the fibre floor (N′ plus Lemma 18.1);
  - the execution floor: trivial Ω(r + n/w), no Ω(h₁) bound known.
  - **All three agree that no execution-model floor of h₁ is proved.** MONODROMY says Floor_Q(𝓡) is unknown for K ≥ 2, because G1 bounds a sum, not Q. ChatGPT says G1 transfers to a machine floor only through a Bridge, C*(B) ≥ κ·c_𝓡(E(τ_B)) for every B in the old operational class, and notes that the compiler direction (≤) proves nothing.
  - **ChatGPT's Bridge is the decisive formulation.** Without it, "below G1's floor in an execution model" compares a machine count against a calculus bound and is not a theorem.
- **Relocation.** Neon gives three worked witnesses in W_core: doubling (j + 2⁻ʲ)m; sweep composition (1 + 1/e)m (M110 says m/e only under the lenient reading, and that e may not survive on Open's map, V); supplied LZ77. MONODROMY: a fall in Q bought by a rise in P, s or S, visible at fixed budget. ChatGPT: C* is invariant under moving code between phases. **They are equivalent in intent.** Neon's examples are the most useful, but its arithmetic counts compositions only, not the O(r) run pass (see §1).
- **Utility.** Neon splits clause 3 into kernel (3a) and end-to-end (3b) at W ∈ {1, 10}. ChatGPT: the 10-worker STOP constrains *that* consumer only, and is not a universal bottleneck theorem. MONODROMY: P-cal priced time decides. **ChatGPT's scope caveat is correct** (J §30 tested one supplied emit order). Neon's split is a sound reporting structure.

**UNIQUE.**
- **ChatGPT's compulsory-output caution** (§1 above). Right.
- **Neon's entropy constraint:** on novel random content, beating h₁ in instruction count requires ω(1) symbols per instruction (packing or multi-byte phrases), never fewer bits. It is argued, not proved: it holds only in a currency where moving each byte costs an instruction. It is still an important constraint that the others omit.

## 4. Main theorem shape

**AGREE.**
- The shape is conditional existence, Mario-style: named premises, each discharged separately; supplied state first (soundness and consumer cost), then acquisition; non-vacuity.
- The N/R/W split leaves the statement and becomes an optional proof device.
- Refuting a premise is not a "no" (M103 §3, MT §6).
- A "no" must quantify universally over a declared admissible class, with uniform programs and a fixed budget, on 𝓖 (not 𝓕 alone). A finite list of failed families is not a no.

**DIVERGE.**
- **What is fixed outside ∃.** ChatGPT: Π (machine, C*, old class, E, κ, K, S, 𝓗, δ, ε) is fixed *before* ∃A, with the warning that ∃A ∃C* is vacuous. Neon fixes its parameters before MeetsTarget. MONODROMY fixes 𝔟 but quantifies ∃𝓘. **ChatGPT is strictly safer.** MONODROMY's "∃ a declared 𝓘" inside the yes lets the candidate choose its own tariff, the very hole ChatGPT warns against.
- **Frozen target in the yes.** ChatGPT requires *both* P₁₀₈ ≤ h₁/2 and C* ≤ κh₁/2 for a frozen yes, or a versioned new target. Neon and MONODROMY replace P₁₀₈. **ChatGPT's position is the C2-consistent one** (see open decision 1).
- **"No" strength.** Neon's tiers (calculus barrier achieved; scoped execution model the realistic target; unconditional word-RAM Ω(h₁) unrealistic) and MONODROMY's "with P unbounded, sorting gives Q = 0, so a no that doesn't fix 𝔟 is false" are complementary and both right. ChatGPT adds the finite-𝓕 narrow no, and that an expectation bound is not a per-block obstruction.

**UNIQUE.**
- **MONODROMY:** a "no" is a pass–memory–access trade-off lower bound for permutation inversion presented as a BWT, and none is known. Right; it locates the no in known complexity territory.
- **ChatGPT:** Rust-to-algorithm correspondence is a separate obligation. Right.

## 5. Source check: the members' findings

All three are faithful on doomslide in outline: Hearers and Learners, the Method as supplied-then-demonstrated, Hilbert's 24th as transformations between proofs, and anthyphairesis as two-interval induction. Neon and ChatGPT read Mario first-hand, MONODROMY second-hand. The merged contradictions list is below.

## 6. Critique of the frozen target, channel #13–14, and R1–R7

**AGREE.**
- c₁ plus the strict convention is decided by convention.
- "Pass" and K = 2 are not operationally defined.
- "Not rows, not intervals" (R1) is syntactic.
- R2's "small where runs fail" is qualitative and measured on six blocks.
- R3's "O(1) streaming passes" with no row-by-row walk neither defines a pass nor charges a build.
- R4 is internally inconsistent: it asks for an execution-model conclusion "under record 108's convention".
- R7 omits the worker count.
- Channel #14's claim that supplied copy plans meet conditions 1–2 is overstated. N-12 itself says z/h₁ = 0.54–0.64 > 0.5 on 5/6 blocks with a free oracle, 0.48 on block-210, and 1.27–1.51 at min_len = 8 (V).
- §6's displayed no-class omits global E2, the cell it calls open (MT L165 vs L161, V).

**DIVERGE.**
- **R4.**
  - *MONODROMY* says R3 ∧ R4 is unsatisfiable, labelled [P].
  - *Neon* says R4 alone is self-contradictory.
  - *ChatGPT* says R4 asks for an execution conclusion from a provenance theorem, and that 108's "no by definition" is itself labelled argued.
  - **V:** M108 §3 is headed "[argued]". The impossibility needs P₁₀₈'s unformalised "determine something about that row". So **[P] is over-labelled. The defect is real, but argued**, and it lies in R4 itself (Neon), not in the conjunction.
- **Channel #14's "a closure with no construction attempt does not count".** Neon says it contradicts the yes/no charter, and keeps structural obstructions as progress. ChatGPT: a near miss is progress, but not partial existence. A scoped obstruction is useful, but not a universal no. MONODROMY is silent. **ChatGPT's reading is balanced and consistent with METHOD's statuses.**
- **Verdict line ("R1–R3 met, R4 partial").** ChatGPT says it cannot establish a lowered floor. Neon replaces it with three tiers (supplied candidate; starts to overthrow; full yes). **Neon's tiers are the usable correction.**

**UNIQUE.**
- **MONODROMY:** R6's "route class R1/R2/R3" collides with the rubric's own R1–R7 labels (V, J §32). Right. Neon reuses the clashing labels without noticing.
- **MONODROMY:** δ against h₁ is not the baseline's cost (Open pays m), so also report Q/m. Right. Neon goes further (REV2: removing all h₁ events still needs h₁/m ≥ 2/3, V at REV2 L264).
- **ChatGPT:** "no advice" still admits a giant static table on a bounded domain, so a uniform unbounded-n extension is needed. Right.

## 7. Open choices: see "Open decisions" below.

---

## Merged contradictions list

| # | Contradiction | Found by | Verified |
|---|---|---|---|
| 1 | c₁(Open): = h₁ (M96 §4, M98 L52, J§12, FRONTIER L35 "Open meets that exactly") vs ≥ m (M102 §3, MT §4.1); FRONTIER and J not reconciled | M, N, G | V (M98 L52, FRONTIER L35, MT L109) |
| 2 | R4 demands a sub-G1 count "under 108's convention", which makes it unreachable; 108 froze strict while #13–14 pivot to construction | M, N, G | V; argued, not proved |
| 3 | "Work-bound / misses aren't emit's cost" (M95, channel #4) still asserted in MT §1a L33 after the toy-transfer correction (0.55–0.78×) | M, N, G | V (MT L33) |
| 4 | Copies can't touch novel nibbles (M102 §2, N-8) vs M108 §1 (short birthday copies) | M, N, G | NV in detail; M108 §4 withdraws, V |
| 5 | 𝓕 size: "238" (HANDOFF L8, J L859) vs 232 distinct, with the 6 admitted among them (N-2). MT §1 L10 says "232 … and the 6 admitted" | N, G | V. Neon misattributes "238" to MT §1 |
| 6 | "Fixed before any data" (HANDOFF, blog) vs MT §1 L19's disclosure that M96's 75–85% hex was seen before δ | N | V (MT L19, HANDOFF L8) |
| 7 | Mario: J §2 places SSL "Astral Rejection" in repo tra38/sm64-wmotr-abc-proof; the repo is a WMotR GOAL-1 (no-A ⇒ no-fly) Rocq result, with GOAL 2 "not started" | N, G | V (README) |
| 8 | MT `theorem main`: c₁W unbound (L92); `≤ S c` (L91) vs `O(c.r)` (L94); clauses numbered 1,2,3,4,6,6b,7,5 | N (G partly) | V |
| 9 | MT §6 L161 still prints \|𝒫\| + 3\|D\| after N-18's sharp 1\|D\| / 2\|D\| was accepted (M108 §1) | N, G | NV for L161 text; acceptance V |
| 10 | MT §6 no-class omits global E2, the cell §5 calls OPEN | N, G | V |
| 11 | "Valid" = single-cycle (M101, M103, MT L18) vs the PROOF-PROG contract (periodic and multi-cycle decoded) | N, G (M adopts the conflation) | V |
| 12 | G1's domain: M96 "every column" vs M103 single-cycle | G | V (M96 L48) |
| 13 | M107 §3 "no event floor" for order-sensitive A vs N-18 Lemma 18.1 fibre floor (M108 §1) | N, G | V (M107 L38–39) |
| 14 | Flip blindness 2\|D\| → 3\|D\| → 1\|D\| / 2\|D\| | N, G | NV |
| 15 | 75–85% of h₁ is hex (M96) vs about 65% novel hex (N-8). Different quantities; ChatGPT: 64.61% and tokenisation-dependent | M, G | NV |
| 16 | 𝓕 as finite theorem domain (MT §1) vs clause 7's 𝓖 | M | V |
| 17 | Orientation: MT §2 has a `mirror` constructor, M103 requires one orientation | G | V (MT L53) |
| 18 | Sorting and list ranking: M98 zero count, M99 inside 𝓡, M103 withdraws family-wide | G | NV |
| 19 | E1 closed generally (M99) vs bulk and packed uses open (M103, M107) | G | V (MT §5 table) |
| 20 | Randomness: column independence (M96) vs text fills (M103); row-sampling bias | G | NV |
| 21 | Theorem N (M106) withdrawn for N′ (M107) | G | V (M107 title) |
| 22 | Gap-rank data = positional problem (M104 §1c) withdrawn (M107) | G | NV |
| 23 | One pass = depth 2 (M108 lenient) vs same-sweep updates (M110) | G | V (M110 L38) |
| 24 | M105's o(\|S\|) schema as the escape criterion vs execution cost | G | NV |
| 25 | N-3: "≈ 40 edges ≪ h₁" with r = 2, impossible since h₁ ≤ r | G | V (N-3 L41) |
| 26 | Channel #14 says copy plans "meet 1 and 2" vs N-12's 5/6 failure | N, G | V |
| 27 | Clause 3 has no worker count, while supplied codes give 0.838 (W = 1) and 0.988 (W = 10) | M, N, G | V |
| 28 | N/R two-stream shape in MT §3 vs records 107–108 | M, N, G | V |

## Errors in members' answers (verified)

- **MONODROMY:**
  - *T1.* It states T[n−1−t] = L[LF^t(o)] "on every block, valid or not". On a column whose marked cycle length ℓ does not divide n, this disagrees with the contract (PROOF-PROG §1.2: decode[j] = decode[j mod ℓ], the cycle repeated from the front, cut at n). Equality needs ℓ | n. Harmless on single-cycle and periodic-valid columns, wrong on corrupt ones, which the contract covers.
  - *Theorem shape.* Item 3 (Q ≤ C·π) and item 4 (π ≤ (1 − δ)h₁) give only Q ≤ C(1 − δ)h₁. For C > 1 this does not imply its own §3 criterion Q ≤ (1 − δ)h₁. The constant must be inside the margin.
  - *Labels.* "R3 ∧ R4 unsatisfiable [P]" outranks its own record (M108 §3 is [argued]).
  - *Minor.* "Lanes at 0.55–0.78×": the figure is branchless appends with lanes (toy-transfer), not lanes alone (ChatGPT's table: lanes alone 5–14%; NV).
  - *Self-imposed scope gap.* It didn't read doomslide or Mario first-hand, and so missed contradiction 7.
- **Neon:**
  - *W_core degeneracy.* As defined, W_core ≤ (1 − δ)h₁ excludes any decoder doing per-run-record work, since r ≥ h₁. The C_io carve-out is unspecified. Its doubling and sweep "relocation" arithmetic counts compositions and omits the run pass.
  - *Attribution.* "238 in MT §1": MT §1 says "232 … and the 6 admitted". "238" is in HANDOFF and J L859.
  - *Doubling.* "Doubling below h₁/2 at K = 3": M108 says *unbounded* passes, j = 2 or 3.
  - *Rauzy–Veech labels.* "B2 holds [proved]" for Rauzy–Veech overstates. Cells' 1.013–1.019r is measured.
  - *Minor.* "0.93h₁ on 𝓕" comes from the six admitted blocks (N-19, fill_remaining ≤ 6.83%), not all of 𝓕.
- **ChatGPT:**
  - *Citation.* It cites "Journal §29" for the supplied-codes result. That result is §30 (§29 is the second outside round).
  - No substantive errors found in the sampled claims: bbaaa, N-3, N-12's 0.48, M108's label, the Mario scope and the MT §6 omission are all V.

## Open decisions after merging

1. **The frozen target's currency.**
   - *Options:* (a) keep P₁₀₈ alone (no by convention); (b) replace it with an execution vector (MONODROMY, Neon); (c) keep v1 frozen and register a versioned v2 in an execution currency, reporting both (ChatGPT).
   - **Recommend (c).** Replacing a C6-frozen target after data is a C2 relaxation. Versioning is not.
   - *Who:* the team can register v2 as an adoption registration (Researcher's review). **Louis** should confirm that v2's currency is what he means by "lower the floor", since it redefines the programme's headline goal.
2. **The operational currency C\*.**
   - *Options:* MONODROMY's Q (addressed accesses) with P ≤ K; Neon's W_core; the full vector, unscalarised.
   - **Recommend Q, D, P, S as the reported vector, with Q as the compared component** and P-cal as the scalar gate. W_core as defined is degenerate. Fix C* in Π before any candidate (ChatGPT).
   - *Who:* the team.
3. **The Bridge theorem.** Which old operational class 𝒜_old, extraction E and κ make G1 an execution floor? All three agree none exists. **Recommend that MONODROMY state and attempt the Bridge for 𝓡's decoders in the Q currency, as the next formal task.** It is the precondition for any "below G1 in an execution model" claim. *Who:* the team.
4. **Pass and cursor semantics.** s_max (MONODROMY: |Σ| + O(1)); fusion and partial-scan rules (ChatGPT); whether data-selected cursors count as streamed. **Recommend |Σ| + O(1)** with an explicit trace-to-pass function, and K = 2 kept with Floor_Q(K) also reported. *Who:* the team.
5. **Domains.** **Recommend** correctness on all columns (D_ker); cost on single-cycle columns of 𝓖, validated on 𝓕 as 232 distinct blocks; a stated policy for h₁ = 0 and small n. All three effectively agree. *Who:* the team.
6. **Memory bound.** Pointwise S ≤ S_Open (ChatGPT, frozen) vs S₀(n, r, m) (MONODROMY). **Recommend** the pointwise bound for v1, and S₀ for v2's asymptotic statement. *Who:* the team.
7. **Packing.** Do multi-symbol-per-word algorithms count as floor-lowering? Neon says yes; the others are silent. **Recommend yes, if the speedup is ω(1) per word and is not fixed-factor SIMD of the same mechanism** (that stays as residual (b)). *Who:* the team.
8. **Utility gate.** W = 10 primary (ChatGPT) vs a 3a/3b split at W ∈ {1, 10} (Neon). **Recommend** reporting both, with W = 10 end-to-end as the ship gate. *Who:* the team (PERFORMANCE-POLICY).
9. **Rubric relabelling and the verdict tiers** (route classes, Neon's three tiers). *Who:* the team (Coordinator).
10. **Lean scope.** Spec first (Neon), or the 𝓡/G1 formalisation. **Recommend Spec first.** *Who:* the team.

## Draft agreed formulation

Fix, before any candidate, a specification Π. It contains:
- a bounded-word machine with a declared instruction set and typed memory (read-only run records, write-once output, working memory);
- a cost vector over complete executions: addressed accesses Q, dependent depth D, passes P and peak state S, with cursor discipline and a trace-to-pass rule;
- the budgets K = 2, s_max and S;
- a uniform text-generated family 𝓖 with failure rate ε, and the validation set 𝓕 (232 blocks);
- δ = 1/2.

The question is whether there exist one uniform, advice-free program A and a representation (𝒵, Rep, Build, Use) with these properties:
- A is exact on every column;
- Build acquires from the run records within the budget;
- Rep ⇒ Use = decode;
- the whole execution's Q is bounded by an input-defined parameter θ;
- that bound is at most (1 − δ)·κ·h₁ on every block of 𝓕 and with probability ≥ 1 − ε on 𝓖, constant included.

The comparison is meaningful only through a Bridge theorem: every decoder of the old class 𝓡 pays Q ≥ κ·c₁ ≥ κh₁ under Π. A yes is a conditional theorem whose premises (soundness, supplied-state cost, native acquisition, Bridge, resources) are discharged one by one, supplied state first. A no quantifies over every program within Π's budget on 𝓖. The frozen c₁/P₁₀₈ target is kept as a separately named version. Whole-decoder time at a pinned worker count is a separate empirical receipt, not part of the theorem.
