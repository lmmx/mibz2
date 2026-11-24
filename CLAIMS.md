# CLAIMS (MONODROMY; the main-theorem challenge, with Neon on research/neon-main-theorem)

NOTE for Neon: Coordinator messages are in channel/coordinator-to-neon.md — read it each cycle.

STARTED M-1 2026-11-24T10:33Z the main theorem: statement, premises, closed branches, what a "no" is (PROOF-PROGRAMME §5.11 draft)
STARTED M-2 2026-11-24T10:33Z stream R: best and worst case; the random-content conjecture
PUSHED M-1 (this commit) draft: docs/journal/monodromy/2026-11-24-main-theorem/main-theorem.md; open premises PR, PN, Pglue
PUSHED M-2 (this commit) worst case aR ≥ 1 in 𝓡 (proved, given G1); E1 and E3 closed (argued); only E2's global cell open
STARTED M-3 2026-11-24T10:40Z a lower bound on positional procedures for model-I rows
DONE M-3 (this commit) record 100: c₁ must charge single-row facts of every type (argued); locally keyed multi-row facts closed on model-I rows (proved under model I); global keys open; the full conjecture is model-relative (a conditional effect with an unproduced prerequisite: a declared Λ)
NEEDS M-2 h₁R/h₁ on each block of 𝓕 (#805's 232 distinct blocks + data/admitted-bwt's 6), where R = maximal runs of width 1 whose byte is in [0-9a-fA-F]: report per block h₁, h₁R, and the census's cost weight
DECIDED M-1 2026-11-24T10:45Z parameters fixed under C6 before N-0's data: 𝓕 = all 238 blocks, per block; K = 2; δ = 1/2; state ≤ Open's peak scratch; split not part of the target; anti-cheat clauses 1–5 (main-theorem.md §1, §1a). Stricter reading wins if Neon disagrees
STARTED M-4 2026-11-24 stream R, the block-keyed cell: the swap-invariance lemma
DONE M-4a (this commit) record 101: flip = transposition of two column bytes, σ′ = σ∘(3-cycle), valid iff orbit order, effect = exchange of two adjacent text blocks (proved; brute-force checked); swap-invariance Lemma 2 (proved); Λ_agg block-histogram rules need |𝒫| + 2|D| > ν′(x) to place x (proved, given ν′); ν′ = Θ(n) on hex rows (conjectured under model I). METHOD §6a: a conditional effect with an unproduced prerequisite (ν′ on 𝓕)
BREAK M-4a premise: the handoff's "Hamming distance 3 / cyclic rotation of three symbols" is wrong as a minimum: valid count-preserving flips exist at distance 2 (caab ↔ cbaa); 3 is the number of changed successors
DONE M-5 (this commit) Pglue floor: c₁ ≥ seg_R − |U| over 𝓡 ∪ local ∪ Λ_agg even with segment interiors free (proved as a count); aR ≥ ~0.29 on GUID content (argued); Pglue alone does not decide Pmargin
NEEDS M-4 per block (6 admitted, then 𝓕): type-A/B flips of record 101 §1 with validity by orbit order: (i) fraction of width-1 hex rows that are an endpoint of ≥1 valid flip; (ii) greedy disjoint-span valid-flip count ν′/n; (iii) |U| = rows in origPtr's arc for every valid flip; (iv) gap distribution between consecutive flip cut points along the orbit
WITHDRAWN(answered by N-2 656d5e7) NEEDS M-5 per block: seg_R, seg_N, seg_W (maximal same-stream orbit runs under σ_hex) and seg_R/h₁R
DONE M-6 (this commit) record 102: Neon's N-1 breaks 1–4 accepted; main-theorem.md repaired (G1 takes singleCycle; Pmargin charges c₁(Pglue); Pstate; attribution by run). With glue ≈ 0.26·h₁ (N-2, cited) Pmargin needs aR ≤ ~0.29 (argued)
DECIDED M-1 2026-11-24 §1a clause 6 added (stricter): a yes states dependent loads per block, d(D,c) ≤ d(Open,c). §6: a no must name the part of global E2 it covers; currently partial (coarse aggregates closed, fine-input global open)
DONE M-7 (this commit) repeats-are-wide lemma (proved under model I): ≤ log₁₆N + 2 width-1 rows per repeated hex occurrence; positional copy schemes and r-index φ do not enter the open cell on novel content (argued). METHOD §6a: tested failure within defined scope
