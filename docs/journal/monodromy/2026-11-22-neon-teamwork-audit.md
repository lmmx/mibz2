# 2026-11-22 · MONODROMY 90 · The Neon Breakthrough Team's handoff, audited. The contraction algebra is a theorem once restated: κ = 1 is exactly the split-free class, and full absorption is exactly what makes r monotone. Lazard is one case of the monotone class, 1Cut and BDMR lie outside it. The Θ(n) falsification does not reproduce on the committed Lazard. r + n is monotone where r is not (bounded). G1 holds as an asymptotic result beyond bzip2's block cap. G2 holds for read-only sequential streaming only, and says nothing about RAM decoders

**The request.** The Coordinator's CHECKPOINT 7073621502. The source is branch
`research/neon-teamwork-paradigm` at 7bb65f74, file
`docs/journal/2026-11-22-neon-teamwork-paradigm.md` and its artefacts.

**Labels:** **proved** · **verified** (exhaustive on the stated domain) · **reproduced** (the team's
own code, re-run).

**Scripts** (beside this record as `.txt`):
- `contract.py`: the algebra, from first definitions;
- `potential.py`: candidate potentials;
- `phibwt.py`: Φ_BWT, rebuilt.

**Small domain** for §1: every primitive cyclic text over 2 letters to length 12, 3 letters to 9
and 4 letters to 7. Every 2-colouring (Σ_L, Σ_R) of its letters, in two modes:
- **pair contraction:** every bigram a b with a ∈ Σ_L and b ∈ Σ_R becomes one token;
- **block absorption:** every maximal Σ_R block following a Σ_L letter y joins y's token. This is
  Lazard's y x^k when Σ_R = {x}.

A contraction is a **valid BWT relabelling** when the surviving rows of BWT(T), each token on its
first letter's row, are in the rotation order of the contracted text T′ under some token order.
That is "zero row reordering".

## Verdicts

| claim | verdict |
|---|---|
| Cut number: zero bucket splits for all BWTs iff κ = 1 | **Holds** (proved; verified). κ ≥ 2 splits in 43,772 cases |
| BWT–Lazard commutation (min, max and both) | **Holds, in a more general form:** every κ = 1 contraction is a valid relabelling (599,000 cases, 0 invalid) |
| r_{t+1} ≤ r_t for Lazard | **Holds** (proved below): it follows from full absorption, and holds for every κ = 1 block absorption, not only one extreme letter |
| The (κ, e(Σ_R)) classification as a unification | **Holds with correction.** The monotone class is κ = 1 with *full absorption* (every Σ_R occurrence absorbed: block mode, or e(Σ_R) = 0 counting self-loops). 1Cut is κ = 1 with partial absorption (not monotone); BDMR is κ ≥ 1 with bucket splits (not monotone) |
| Dual-polarity uniqueness | **Holds** under the zero-reordering hypothesis |
| The Referee's F3/F4 witnesses (r grows under threshold or BDMR cuts) | **Hold.** My smallest κ = 1 partial-absorption witness is T = 00112, Σ_R = {1, 2}, r 3 → 4: the Referee's |
| Symmetric-Middle-Run: Θ(n) work and symbols for all Lazard modes | **Fails on the committed code.** It reproduces n = 4,224 and r = 129, but Lazard-both ends with 320 symbols (2.48 r), flat in K. Θ(n) is pair contraction's behaviour on runs |
| A potential monotone where r is not | **r + n** (bounded): Δr ≤ \|D\| on all 730,596 valid contractions, with Δr ≤ 2. r − σ, r + σ, the bigram count, text runs and z all fail (on 3,366 r-increase cases) |
| G1: L-complete under DLOGTIME-uniform TC⁰ reductions | **Holds with correction:** it is asymptotic, for unbounded n, and so beyond bzip2's 900,000-row cap. Φ_BWT verified on 1,224 cases |
| "Φ_BWT always yields a valid single-cycle bzip2 BWT" | **Holds with correction:** single-cycle always (T is primitive); a bzip2 block only while n ≤ 900,000 and b + 2 ≤ 256 |
| G2: S = Ω(r / (L p⁴)) for p-pass read-only streaming | **Holds with correction**, for that model only. The split argument is checked; the constants are not re-derived. It does not cover RAM, read-write streams or external memory, and at bzip2 scale it is vacuous (about 35 bits at p = 1) |

## 1. The unified contraction algebra

### 1.1 κ = 1 is exactly the split-free class (proved, verified)

**The token order.** A token [a w] (a ∈ Σ_L, w a Σ_R word) compares as the string a w followed by
the next token's first letter, which lies in Σ_L. Two tokens with the same first letter a then
compare as follows:
- [a w] against [a w′] is decided inside w and w′, or, if one is a prefix of the other, by a Σ_L
  letter against a Σ_R letter;
- [a w] against an unabsorbed a (followed by c ∈ Σ_L) is decided by w's first letter against c.

**The κ = 1 case.** Σ_L is one interval, so every Σ_R letter lies above or below all of it. Each of
those comparisons is then the same whatever c is.
- *So:* a's rows stay contiguous, and the order of T′'s rotations equals the order of the
  surviving rows of T's.
- *Why:* comparing two sequences of tokens is decided at the first differing token, and that
  decision is the same as comparing the strings. This is the Lazard / Lyndon prefix argument.

**The κ ≥ 2 case.** Choose c < b < c′ with c, c′ ∈ Σ_L and b ∈ Σ_R. Any text containing the bigrams
c c, c b and c c′ splits c's bucket. □

**Verified:** 0 invalid in 599,000 κ = 1 cases (pair and block). κ ≥ 2 splits in 43,772 cases;
the smallest is T = 00102, Σ_L = {0, 2}.

### 1.2 Full absorption makes r monotone (proved, verified)

**Full absorption** means every Σ_R occurrence is absorbed. Then the deleted rows are whole
F-buckets of Σ_R, and with κ = 1 they are a prefix and/or a suffix of the rows. Under κ = 1, full
absorption holds exactly in block mode, or in pair mode when no Σ_R → Σ_R bigram occurs.

**Proof that r_{t+1} ≤ r_t.** Take two surviving rows that are adjacent in L′. They are adjacent
in L too, since only a prefix and a suffix were deleted. Consider whether their previous tokens
differ.
- *If either row's L letter is in Σ_L:* the tokens differ only if the L letters differ. That is a
  boundary of L, charged to itself, in the surviving region.
- *If both L letters are in Σ_R:* they are equal, or the boundary is already in L. Walk both rows
  back by LF while they read the same Σ_R letter. They stay adjacent inside deleted buckets until
  they first read different letters. That boundary of L lies inside the deleted region, between
  two deleted rows.
- *Why the charges never collide:* LF is injective, so two pairs never charge the same deleted
  boundary. Deleted boundaries are never counted in L′.

So r′ − 1 ≤ r − 1. □ This covers Lazard (Σ_R one extreme letter, or both extremes), and every
κ = 1 block absorption.

**Verified:** in 299,500 κ = 1 block-mode cases and 62,502 κ = 1 pair-mode cases with full
absorption, r never increased.

**Partial absorption breaks it.** With κ = 1 pair contraction and some Σ_R → Σ_R bigram, r grew in
10,856 of 236,998 cases. The smallest is **T = 00112, Σ_L = {0}, Σ_R = {1, 2}, r 3 → 4**: the
Referee's threshold-cut witness. The proof fails there because a Σ_R bucket is only partly
deleted, so its interior boundaries are still counted.

### 1.3 What the unification classifies

**It is a theorem about BWT-space contractions by a 2-colouring, in two axes:**
1. **κ = 1:** no bucket split, a pure relabelling with whole-row deletion;
2. **full absorption:** r is non-increasing.

**Where each primitive sits:**
- *Lazard:* both properties.
- *1Cut:* κ = 1 with partial absorption, so split-free but not monotone.
- *BDMR:* κ ≥ 1 with bucket splits, so neither, as the Referee's n = 5 and n = 10 witnesses show.

**It does not classify work or rounds.** Nothing in (κ, absorption) bounds Σ_t r_t or the number
of rounds; the open question in the team's §5 stands.

**The "e(Σ_R)" axis as stated needs one correction.** What matters is absorption, not the count of
Σ_R → Σ_R edges. Block absorption makes self-loops harmless, which is why Lazard is monotone with
Σ_R = {x} even though x x bigrams occur.

### 1.4 Dual polarity (holds)

**Why forward polarity is the only choice.** In a contraction without reordering, the token
[a b] must survive on the row of its *first* letter: the rotation of T′ at [a b] is T's rotation at
a. The dual keeps b's row, which is ordered by "b …", and gives it a token whose string starts
with a. That breaks the sort order whenever two such rows compare differently by a and by b. The
team's n = 4 witness (L = 1110) is one instance. So among the two survivors, forward is unique.

### 1.5 The Symmetric-Middle-Run falsification (fails on the committed code)

**What I ran.** I re-ran `referee/audit_alpha_falsify.py`'s family, with two-character tags, against
`alpha/verify_full_and_falsify.py`'s `lazard_decode` as committed:
- *K = 64:* n = 4,352 (not 4,224), r = 250, **tot_syms 324 (1.3 r)**, 27 rounds.
- *Flat in K:* K = 4, 16, 64 and 256 give 322–330 symbols.

**The variant that matches the reported figures.** With one-character tags it gives the reported
n = 4,224 and r = 129, but **320 symbols (2.48 r)** at every K, in 17–66 rounds.

**Why.** The committed Lazard writes y x^k as one power rule (`prod[sid] = (y, x, k)`), so a run M^K
costs one rule. A middle run peels one byte a round only under **pair** contraction, where an M
preceded by M is not absorbed: §1.2's partial-absorption class.

**So the Θ(n) claim fails as stated.** It holds for 1Cut-style pair contraction on runs, not for
Lazard. No artefact on the branch records the 4,211-symbol figure.

### 1.6 A potential monotone where r is not (verified, bounded)

**Measured over all 730,596 valid contractions** (κ = 1 in either mode, plus valid κ ≥ 2 cases):
- r grew in 14,840 of them, by at most 2;
- r grew by at most the number of deleted rows (Δr ≤ |D|) in every one.

So **r + n is non-increasing**, and falls by at least |D| − Δr ≥ 0. A general proof is open; the
charging argument of §1.2 gives Δr ≤ 2|D|.

**No compressibility measure tried is monotone in this class.** On the smaller domain `potential.py`
covers (2 letters to length 10, 3 to 8, 4 to 6), r rose in 3,366 valid contractions. Each of the
following rose in some of those:
- r − σ: 698;
- r + σ: 3,366;
- the distinct-bigram count: 606;
- text runs: 896;
- the LZ77 phrase count z: 192.

## 2. Gamma's G1 and G2

### 2.1 G1 (holds with correction)

**The construction checks out.** I rebuilt Φ_BWT(π, b, K) from the write-up's definition for every
single-cycle π on M = 2–6, with b ∈ {2, 3} and K ∈ {1, 2, 3, 5}: 1,224 cases. All of the
following held:
- T_π is primitive, so its BWT is one LF cycle;
- p = 0;
- LF^{K+L}(x) = π⁻¹(x) on the # rows;
- r ≤ (L + 1) M + 2.

**The sort key depends only on (x, pred x).** A rotation starting inside id(y) reads the rest of
id(y), then a unique #, then $^{K−1}, then id(π(y)). So its key is (the digit suffix of y, #, x)
with x = π(y), as stated.

**The reduction is uniform.** Ranks need counting, which is TC⁰ (not AC⁰), and selection and the
127-capped record prefix sums (iterated addition) are DLOGTIME-uniform TC⁰. Etessami's ORD is
L-complete under FO (DLOGTIME-uniform AC⁰) reductions. So "L-complete under DLOGTIME-uniform TC⁰
reductions" is the right statement.

**The corollaries are sound.** Inverse BWT in TC⁰ would give TC⁰ = L, and the same holds for NC¹.

**The correction: G1 is asymptotic.** A bzip2 block has n ≤ 900,000 and at most 256 symbols, so
every bzip2 instance is of bounded size, where these classes say nothing. G1 is about run-encoded
BWT inversion with unbounded n. It bears on bzip2 only as the statement that no constant-depth
threshold circuit family inverts BWTs of every length.

**"Always a valid single-cycle bzip2 BWT":**
- *Single-cycle:* always.
- *A bzip2 block:* only while n ≤ 900,000 and b + 2 ≤ 256. Any byte string is an acceptable RLE1
  input to the decoder, so RLE1 adds no further constraint.

### 2.2 G2 (holds with correction, for its model only)

**The model** is read-only, p-pass, sequential streaming over I₀ in its record order, with S bits of
working memory and no writable stream. Equivalently, a (2p − 1)-message two-party protocol, Bob's
half of the stream first.

**The split is right.** Each node's out-edge belongs to one side:
- *A-nodes:* the odd π (Alice's), the hairpin and the closing shift (both public);
- *B-nodes:* the even π (Bob's) and the public edges.

L at a row depends only on its rotation's start node and that node's successor. So:
- *The first half of L* (the #_A and $_A rows, and the B-digit rows) is a function of Bob's
  permutations.
- *The second half* is a function of Alice's.
- *The seam* separates disjoint letters, so the run records split cleanly.
- *The cycle is single* by π₁⁻¹…π_t⁻¹π_t…π₁ = id.

**The bound** is the standard t-step pointer-chasing shape: (2p − 1)·S ≥ Ω(N/t²) gives
S = Ω(N/t³) = Ω(r/(L p⁴)). The Product-Rectangle and decoupling lemmas fit that template. I did not
re-derive the stated constants.

**What G2 does *not* cover:**
1. **RAM decoders.** With random access to the read-only input, inversion is in L: O(log n) bits of
   working memory suffice. That is G1's own upper bound. G2 says nothing about Cells, OpenCells or
   any decoder with random access.
2. **Read-write or multi-stream streaming, and external memory with sorting:** these escape it, as
   the team says.
3. **Typical inputs.** It is a worst case over the Hairpin family. Wikidata blocks are not such
   instances.
4. **bzip2 scale.** At n = 900,000, p = 1 and b = 126 (K = 1, L = 2), the family has N ≈ 50,000 and
   the bound is S ≥ N/(256 ln 2 · 8) ≈ **35 bits**. It is meaningful only asymptotically.

## What this decides

Nothing in production. For the Contraction programme:
- *Where a monotone r schedule must stay:* within κ = 1 with full absorption (Lazard-type block
  absorption at the alphabet's ends).
- *Where r grows:* anything that absorbs only part of a Σ_R letter's occurrences, including
  threshold pair cuts.
- *The candidate potential for a work bound:* r + n.
- *The open question* is whether a κ = 1 full-absorption schedule exists whose rounds remove a
  constant fraction of n.
