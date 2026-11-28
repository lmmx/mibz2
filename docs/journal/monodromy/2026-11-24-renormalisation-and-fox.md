# Record 117: the two dynamics leads (channel #25)

*MONODROMY, 24 November 2026. Answers LEADS-dynamics.md (098715a8). Labels: proved / argued / measured / conjectured. No cut in Q is claimed.*

## Lead 1: renormalisation

**Definition.** Fix an anchor word u occurring at least twice in the text T. Let Y be the rows whose rotation starts with u (an interval, so membership is a free test). The operator 𝓡_u sends (L, LF) to the first-return system (Y, LF_Y), with rows renumbered j ↦ j − min Y.

**Lemma R (proved; checked on 3,365 random systems, `.renorm.py.txt`).** Cut T at the occurrences of u into return words w_k. Order distinct words by w ≺ w′ iff w·u < w′·u as strings. Then:
- ≺ is a total order and neither key is a prefix of the other. If w′ = w·z with z non-empty, then z·u cannot begin with u, because u would then occur inside the return word w′.
- The rows of Y, in order, are the sorted rotations of the word sequence S = (w_k) under ≺.
- LF_Y is the LF map of the BWT of S, and its column L′[j] is the return word read (backwards) by the lane from row j.

So "rescale" has an exact meaning: an affine shift of rows plus the relabelling of return words by rank. 𝓡_u maps a decoding problem to a decoding problem of the same kind, with n_u rows.

**Prior art. This is not new.**
- Durand 1998 (derived sequences): a primitive sequence is substitutive iff it has finitely many derived sequences. That is the fixed-point and periodic-orbit statement of this lead, as a theorem. The Fibonacci word is a fixed point of 𝓡.
- Prefix-free parsing (Boucher, Gagie, Kuhnle, Langmead, Manzini, Mun 2019) builds a BWT from the BWT of a parse and its dictionary. SA-IS (Nong, Zhang, Chan 2009) recurses on renamed LMS substrings. Both run 𝓡 in the encoding direction.
- In the decoding direction a tower of 𝓡 is list ranking by contraction (Cole and Vishkin 1986; Anderson and Miller 1990). With one symbol as anchor it is record 114's landmark induction.

**Cost when decoding (proved).** L′ is not given. Reading L′[j] means walking lane j, and the lanes partition all n rows. Acquisition is n − n_u addressed steps; the induced problem costs n_u more. A tower of any height totals Q = n. D falls to the sum of the longest return times per level; S grows by the dictionary. This is record 114's finding, unchanged.

**Lemma R2: width-1 runs are absorbing (proved; checked on 16,049 lanes).** Let J₁(L) be the rows in width-1 runs of L. If lane j meets J₁(L), then L′[j] is a width-1 run of L′.

*Proof.* Adjacent rows with the same letter have adjacent LF images. So while the words of lanes j and j+1 agree, the lanes sit on adjacent rows. At the row in J₁ the neighbour carries a different letter, so the words differ there (or earlier, or in length at the edge of Y). The same holds for j − 1. ∎

**Corollary (proved, for sharing between adjacent co-moving lanes).** In any tower of 𝓡, a step can be generated once for several lanes only on lanes lying wholly inside wide runs of L. Hence h₁(L′) ≥ h₁(L)/R, with R the longest return time, and the rows saved are at most n − h₁. Open already skips those rows. So renormalisation skips scaffold work only, never work on novel fields.

**Templated JSON (argued, for independent random fields).** Take u in the record scaffold. Every return word contains a novel field, so the n_u words are distinct. L′ is then a permutation with no runs, and LF′ is the sort permutation of the records. A second 𝓡 finds nothing to share. Durand's criterion fails as badly as it can: the count of distinct return words grows linearly. Between fields the lanes regather into one contiguous block and cross the scaffold rigidly, but in an order permuted by the field's sort. That permutation is relation C (record 113).

**What renormalisation changes about sense D on random digits: nothing for Q.** Lemma R2 is the precise reason. Equal words at non-adjacent rows of L′ do exist, but learning that two lanes carry the same word costs walking both.

**Verdict.** Does not transfer as a cut in Q. It is exact, it is prior art, and it restates G1 one level up. Its one use is D (contraction depth), already recorded.

## Lead 2: dual numbers

**The number system is Fox's free differential calculus (Fox 1953).** For a word w and a letter c, ∂w/∂c = Σ over occurrences of c in w of the prefix before it. It obeys ∂(uv)/∂c = ∂u/∂c + u·∂v/∂c, which is the dual-number product rule. Send a prefix p to x^|p|. Then ∂L/∂c = Σ x^i over rows i with L[i] = c. The pair (word, its derivatives) composes across a concatenation at constant cost per letter touched.

**Three evaluations, each an object we already have (proved identities):**

| evaluation of ∂/∂c on the column | object | status |
|---|---|---|
| x = 1, on each prefix | occ counts; LF(i) = C[c] + ε(∂L[0..i)/∂c) | the standard table |
| x kept | positions of c: the gap streams, Sadakane's ψ | record 115 |
| d/dx at x = 1, summed | the moment Σ i·L[i] that separates flips | outside review, round 2 |

The CRC is the same construction on the output word: the value of Σ T[i]·x^(n−1−i) mod P, an affine rider under concatenation (record 105). The RLE1 stage is a finite transducer, so its rider is an element of a small transition monoid (Mytkowicz, Musuvathi, Schulte 2014). In Rauzy–Veech terms the abelianised Fox Jacobian of a substitution is its incidence matrix, which is record 115's "dual numbers = RV cocycle".

**What it buys.**
- *Forward-mode evaluation of the MTF output* gives (c, occ) per row in stream order. The walk then uses next = C[c] + occ, with C in a 256-entry table. The LF table is built with no scatter: n addressed writes become streamed, and the text comes out backwards. Prior art, from memory and unverified: Seward, DCC 2001, the merged T-and-L variant. Π's walk bound (record 112) counts the walk only, so the theorem is unaffected.
- *CRC and RLE1 riders* let those stages fuse into an out-of-order emit. P falls; Q does not.
- *Parikh and length riders* give text offsets. The test excludes those as restating the inversion.

**Test against ESSENCE.** Cheap: yes. Not a restatement: yes for occ, ψ, CRC and RLE1. Makes a later stage cheaper: only in P and S, and in the build's addressed writes if the build is a scatter. No rider here touches the walk's Q, and Lemma R2 says why none obtained by composing along LF can: a rider rides a lane, and every lane through J₁ is still walked.

**Verdict.** The analogy is exact and names the family (Fox derivatives of the column and the text). It yields no new rider. Second-order Fox derivatives count ordered pairs of letters; I see no stage that needs them (conjectured useless).

## NEEDS

`NEEDS M-24` Does the production LF build scatter (n addressed writes) or stream (c, occ) and add C at walk time? If it scatters, measure the streamed build with a backward emit on the six admitted blocks, 1 and 10 workers. Rule, fixed before data: under 2% whole-decoder gain on at least four blocks closes it.
