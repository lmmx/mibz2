# 2026-11-24 · MONODROMY 113 · Contraction along composition as a scheme, and four senses of "moving together". On novel random digits none removes the one touch per digit; they differ in whether that touch is addressed, dependent or streamed. No breakthrough is claimed here

Answers channel entry #20 after reading ESSENCE.md. Labels as before. Prior art is cited for each relation; nothing below is presented as new unless it says so.

## 1. The scheme [P that walk → Rauzy–Veech is an instance; record 104 §2]

Data: rows X, the cycle σ = LF, the byte λ(x) = L[x], the start o.

A **togetherness** is a family 𝒞 of row sets, each with:
- a *parametrisation* ι_C : P_C → C from a set P_C on which arithmetic is O(1);
- for some height h, a *transport*: σ^h∘ι_C = ι_{C′}∘τ with τ computable in O(1) from a description of O(1) words;
- a *word rule*: the h bytes emitted from ι_C(p) are an O(1)-described function of p.

**Composition:** (C, h, τ) then (C″, h′, τ′) on C ∩ σ^{−h}(C″). **Recursion:** pick a section Y ⊂ X, replace σ by its first-return map to Y with return words, and repeat on Y.

**What each part must satisfy for the decode to be exact:**
- (S1) every transport and word rule is true (soundness);
- (S2) the paths used tile the orbit of o exactly once;
- (S3) *resolution*: given a row in C, its parameter is found in O(1);
- (S4) *closure*: C ∩ σ^{−h}(C″) is again in 𝒞, and the number of classes does not grow under recursion;
- (S5) *acquisition*: 𝒞 and its transports come from the run records within Π.

**The cost law the scheme gives** [P]: emit costs one addressed access per visit to Y; build costs one transport evaluation per class per level. A scheme pays where **classes are few and Y is small**.

**Walk:** classes are single rows, Y = X. **Rauzy–Veech:** classes are intervals inside a run, P = [0, w), τ a translation, the word constant; (S4) holds because intervals intersect in intervals. On a lone row the class is the row.

## 2. Four other senses, each with what it does to novel random digits

| | sense of "together" | prior art | lone rows move with | contracts on random digits | where the cost goes |
|---|---|---|---|---|---|
| **A** | *same word, any position*: rows sharing their next k bytes; LF^k carries the class, in order, onto one suffix interval | class facts E1 (record 99); BWT tunnelling (FRONTIER's obstruction table); copy plans (Neon N-12, N-16) | rows with the same k-word | only while k ≤ log₁₆N ≈ 4–5; then classes are singletons (record 96 G3) | (S4): classes multiply as \|Σ\|^k; supplied plans still cost 0.54–0.64 per width-1 row |
| **B** | *same byte, order kept*: LF is increasing on each symbol class (record 104 Lemma F) | T² (killed 23 Nov); the ordered frontier (ChatGPT, source record §3; record 104); sweep composition (record 110) | every row with the same byte | everything, at 2 per sweep (doubling) or e per sweep (in place) | passes and cursors: depth k needs \|Σ\|^k cursors or k sweeps (record 112 §3) |
| **C** | *same template offset*: across the m records of a templated text, the rows at one offset form a block ordered by a record permutation π | the frontier is its transport; "BWT inversion is radix sorting" is classical; the statement in §3 I have not found in our records | the same offset in every other record | the scaffold between fields entirely (already wide runs); inside a field, nothing in count | membership: past depth log₁₆ m the block is scattered among the rows |
| **D** | *a rule among the induction's own steps* (the Pell direction): step i+1 obtained from step i | Rauzy paths of self-similar systems are eventually periodic (standard); not tried here | the same step in the next record | the scaffold's induction, from O(m·template) to O(template) | none on random digits: independent uniform digits satisfy no rule [A] |

## 3. Relation C exactly, and the one fact in it worth keeping [P under the template model 𝓖 of record 103 §4]

Let the text be m records of one template with a random field g of d hex digits followed by scaffold and then a field f.

- The rows whose suffix starts just after g form one interval, ordered by the value of the *next* field: row = base + rank_f(f_i).
- The rows whose suffix starts at g form one interval ordered by g itself: row = base′ + rank_g(g_i).
- **So a jump across the field has target base′ + rank_g(w), where w is the field's own word. The target and the word are the same datum.** LF's d steps through the field are a least-significant-digit radix sort of the records by g.

What a decoder must therefore learn per record is the **pairing** of f_i with g_i, about log₂ m bits, not the 4d bits it emits. But the pairing is reachable only by chaining through the field's d digit rows, because head and tail of a random field share nothing else [A].

## 4. Under the referee (Π, record 112)

The digits must each be touched once; that is the output. The senses differ in the *kind* of touch:

| route | Q | D | P |
|---|---|---|---|
| walk | n | n | 1 |
| B, in-place sweep | ≥ ½·min(a⁺, a⁻), about n/e achieved on synthetic text | the same | 2 |
| C, the wave, one round per digit | n (one independent access per row) | number of rounds, about the template length | 2 if each round writes fresh cells |
| C with a full scan of L per round instead | about one per field | small | d |

- Relation C leaves **Q unchanged and collapses D** from n to the template length: it is lanes with as many lanes as records, visited in sorted order. That is a unit-price claim (residual b), not a lower Q.
- Trading its addressed accesses for streamed ones costs one scan per round: P = d.
- **None of A–D brings Q below a constant fraction of the hex rows within K = 2.** I do not have a proof that nothing can.

## 5. What I would pursue, and why

**Relation C's pairing.** It is the only place where the datum a decoder needs (log₂ m bits per record) is far smaller than the work every known route spends (d addressed touches per record). A breakthrough in this problem would have to be a way to obtain that pairing without chaining through the field. I know of none, and §3's last line is the argued reason it may not exist. If it does not, the realistic prize is D, not Q: the wave's dependent depth of a few hundred against the walk's n.

## 6. In passing

- **Record 111's three errors:** corrected in record 112 §5.
- **The cap unit in NEEDS M-17:** I meant **bytes of emitted word**. On that unit Neon's six blocks give 2.056–2.286 retained rows per link, under my own rule of 2.3 on four or more blocks. **Record 110's near miss on Open's retained map is withdrawn.** Proposition S stands for LF over all rows under its stated assumption.
